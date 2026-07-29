//! Theme Builder → Codex via the **Agent Client Protocol** (ACP).
//!
//! Spawns the Codex ACP adapter (`codex-acp`, or `bunx` / `npx` of
//! `@agentclientprotocol/codex-acp`) and talks to it with the official
//! [`agent-client-protocol`] Rust SDK:
//! `initialize` → `session/new` | `session/load` → `session/prompt`.
//!
//! Session list / transcript still fall back to `~/.codex` on disk when the
//! adapter does not implement `session/list` or load history.

use crate::launch::find_chatgpt_app;
use agent_client_protocol::schema::ProtocolVersion;
use agent_client_protocol::schema::v1::{
  ContentBlock, InitializeRequest, ListSessionsRequest, LoadSessionRequest, NewSessionRequest,
  PromptRequest, RequestPermissionOutcome, RequestPermissionRequest, RequestPermissionResponse,
  SelectedPermissionOutcome, SessionNotification, SessionUpdate, TextContent,
};
use agent_client_protocol::{AcpAgent, AcpAgentConfig, Agent, Client, ConnectionTo};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::timeout;

/// Result of a single Theme Builder → Codex (ACP) round-trip.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexChatResult {
  /// Whether the ACP prompt was submitted successfully.
  pub submitted: bool,
  /// Final assistant message (streamed agent text + tool summary).
  pub reply: String,
  /// Always `1` when a non-empty reply was captured, else `0`.
  pub assistant_count: usize,
  /// True when the turn completed within the wait budget.
  pub stable: bool,
  /// Human-readable status (for UI / logs).
  pub message: String,
  /// ACP agent command that was used (when known).
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub binary: Option<String>,
  /// ACP / Codex session id (for `session/load` on the next turn).
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub session_id: Option<String>,
  /// Stop reason from the agent when available.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub stop_reason: Option<String>,
  /// Absolute path to a `.cdxtheme` package found in the workspace after this turn.
  /// Host fills this when the agent packed a theme; apply is a separate user action.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub package_path: Option<String>,
  /// Theme id installed from workspace package (host fills after manual apply).
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub installed_theme_id: Option<String>,
  /// Display name of the installed theme (host fills after manual apply).
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub installed_theme_name: Option<String>,
  /// Whether the installed theme was also applied (host fills after manual apply).
  #[serde(default)]
  pub applied: bool,
}

/// One row in the Theme Builder session list.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexSessionSummary {
  pub id: String,
  pub title: String,
  pub updated_at: String,
  /// Absolute path to the rollout JSONL when known.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub path: Option<String>,
  /// Theme Builder workspace root (`app_data_dir/theme_builder/{id}`) when known.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub workspace_path: Option<String>,
}

/// A single message loaded from a Codex session transcript.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexSessionMessage {
  /// `user` | `assistant` | `system`
  pub role: String,
  pub content: String,
}

/// Full session payload for the chat view.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexSessionDetail {
  pub id: String,
  pub title: String,
  pub updated_at: String,
  pub messages: Vec<CodexSessionMessage>,
  /// Theme Builder workspace root when known.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub workspace_path: Option<String>,
}

/// Resolve the Codex CLI binary: ChatGPT-bundled first, then `PATH`.
/// Used to put Codex on PATH for the ACP adapter process.
pub fn find_codex_cli() -> Result<PathBuf, String> {
  if let Some(bundled) = find_bundled_codex_cli() {
    if bundled.is_file() {
      return Ok(bundled);
    }
  }
  if let Some(path) = which("codex") {
    return Ok(path);
  }
  Err(
    "Codex CLI not found. Install the ChatGPT desktop app (bundles `codex`) \
     or install the Codex CLI and ensure `codex` is on PATH."
      .into(),
  )
}

fn find_bundled_codex_cli() -> Option<PathBuf> {
  #[cfg(target_os = "macos")]
  {
    if let Some(exe) = find_chatgpt_app() {
      let resources = exe
        .parent()
        .and_then(|p| p.parent())
        .map(|contents| contents.join("Resources").join("codex"));
      if let Some(ref p) = resources {
        if p.is_file() {
          return Some(p.clone());
        }
      }
    }
    for app in [
      PathBuf::from("/Applications/ChatGPT.app"),
      PathBuf::from("/Applications/Codex.app"),
    ] {
      let p = app.join("Contents/Resources/codex");
      if p.is_file() {
        return Some(p);
      }
    }
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
      let p = home.join("Applications/ChatGPT.app/Contents/Resources/codex");
      if p.is_file() {
        return Some(p);
      }
    }
    return None;
  }

  #[cfg(target_os = "windows")]
  {
    if let Some(exe) = find_chatgpt_app() {
      let dir = exe.parent()?;
      for candidate in [
        dir.join("codex.exe"),
        dir.join("codex"),
        dir.join("resources").join("codex.exe"),
        dir.join("Resources").join("codex.exe"),
        dir.join("resources").join("codex"),
      ] {
        if candidate.is_file() {
          return Some(candidate);
        }
      }
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA").map(PathBuf::from) {
      for rel in [
        "Programs/ChatGPT/codex.exe",
        "Programs/ChatGPT/resources/codex.exe",
        "Programs/Codex/codex.exe",
        "Programs/Codex/resources/codex.exe",
      ] {
        let p = local.join(rel);
        if p.is_file() {
          return Some(p);
        }
      }
    }
    return None;
  }

  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  {
    let _ = find_chatgpt_app;
    None
  }
}

fn which(bin: &str) -> Option<PathBuf> {
  let path = std::env::var_os("PATH")?;
  for dir in std::env::split_paths(&path) {
    #[cfg(windows)]
    {
      let pathext = std::env::var("PATHEXT").unwrap_or_else(|_| ".EXE;.CMD;.BAT".into());
      for ext in pathext.split(';').filter(|e| !e.is_empty()) {
        let mut name = std::ffi::OsString::from(bin);
        name.push(ext);
        let candidate = dir.join(&name);
        if candidate.is_file() {
          return Some(candidate);
        }
      }
      let candidate = dir.join(bin);
      if candidate.is_file() {
        return Some(candidate);
      }
    }
    #[cfg(not(windows))]
    {
      let candidate = dir.join(bin);
      if candidate.is_file() {
        return Some(candidate);
      }
    }
  }
  None
}

fn codex_home() -> PathBuf {
  if let Ok(home) = std::env::var("CODEX_HOME") {
    let p = PathBuf::from(home);
    if !p.as_os_str().is_empty() {
      return p;
    }
  }
  dirs_home()
    .map(|h| h.join(".codex"))
    .unwrap_or_else(|| PathBuf::from(".codex"))
}

fn dirs_home() -> Option<PathBuf> {
  std::env::var_os("HOME")
    .or_else(|| std::env::var_os("USERPROFILE"))
    .map(PathBuf::from)
}

/// Fallback cwd only when the host does not pass a workspace path.
fn default_theme_builder_cwd() -> PathBuf {
  let dir = std::env::temp_dir().join("cdxtheme-theme-builder");
  let _ = std::fs::create_dir_all(&dir);
  dir
}

#[cfg(windows)]
const PATH_SEP: char = ';';
#[cfg(not(windows))]
const PATH_SEP: char = ':';

fn prepend_path_env(path_env: &mut String, dir: &Path) {
  let prepend = dir.display().to_string();
  if path_env
    .split(PATH_SEP)
    .any(|p| p == prepend || Path::new(p) == dir)
  {
    return;
  }
  if path_env.is_empty() {
    *path_env = prepend;
  } else {
    *path_env = format!("{prepend}{PATH_SEP}{path_env}");
  }
}

const CODEX_ACP_PKG: &str = "@agentclientprotocol/codex-acp@latest";

/// Host runtime needed to spawn the Codex ACP adapter (`codex-acp` / bunx / npx).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeBuilderRuntimeStatus {
  /// True when Theme Builder can spawn the ACP adapter without installing Bun.
  pub ready: bool,
  pub has_codex_acp: bool,
  pub has_bun: bool,
  pub has_bunx: bool,
  pub has_npx: bool,
  /// Preferred runner label: `codex-acp` | `bunx` | `bun` | `npx`.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub runner: Option<String>,
  /// Absolute path of the preferred runner when known.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub runner_path: Option<String>,
  /// Short status for UI / logs.
  pub message: String,
}

/// Probe host for `codex-acp`, Bun (`bun` / `bunx`), or Node (`npx`).
pub fn check_theme_builder_runtime() -> ThemeBuilderRuntimeStatus {
  // Include ~/.bun/bin even when the app process PATH was set before install.
  let _ = bun_bin_dir();

  let codex_acp = which("codex-acp");
  let bun = resolve_bun();
  let bunx = resolve_bunx();
  let npx = resolve_npx();

  let has_codex_acp = codex_acp.is_some();
  let has_bun = bun.is_some();
  let has_bunx = bunx.is_some();
  let has_npx = npx.is_some();
  let ready = has_codex_acp || has_bun || has_bunx || has_npx;

  let (runner, runner_path) = if let Some(p) = codex_acp {
    (Some("codex-acp".into()), Some(p.display().to_string()))
  } else if let Some(p) = bunx {
    (Some("bunx".into()), Some(p.display().to_string()))
  } else if let Some(p) = bun {
    (Some("bun".into()), Some(p.display().to_string()))
  } else if let Some(p) = npx {
    (Some("npx".into()), Some(p.display().to_string()))
  } else {
    (None, None)
  };

  let message = if ready {
    match runner.as_deref() {
      Some("codex-acp") => "codex-acp is available".into(),
      Some("bunx") | Some("bun") => "Bun is available (bunx)".into(),
      Some("npx") => "Node.js npx is available".into(),
      _ => "Runtime ready".into(),
    }
  } else {
    "Bun (bunx) or Node.js (npx) is required to run Theme Builder".into()
  };

  ThemeBuilderRuntimeStatus {
    ready,
    has_codex_acp,
    has_bun,
    has_bunx,
    has_npx,
    runner,
    runner_path,
    message,
  }
}

/// Download and install Bun into `~/.bun`, trying multiple mirrors (official, GitHub, jsDelivr).
///
/// Returns an updated runtime status. Safe to call when Bun is already present.
pub async fn install_bun_for_theme_builder() -> Result<ThemeBuilderRuntimeStatus, String> {
  if resolve_bun().is_some() || resolve_bunx().is_some() {
    return Ok(check_theme_builder_runtime());
  }

  tracing::info!("installing Bun for Theme Builder (multi-mirror)…");
  install_bun_multi_mirror().await?;

  let status = check_theme_builder_runtime();
  if status.has_bun || status.has_bunx {
    tracing::info!(
      runner = ?status.runner_path,
      "Bun installed for Theme Builder"
    );
    Ok(status)
  } else {
    Err(
      "Bun installer finished but `bun`/`bunx` was not found under ~/.bun/bin. \
       Install from https://bun.sh then restart CDXTheme."
        .into(),
    )
  }
}

/// Build the ACP agent process config (Codex adapter).
///
/// Preference order:
/// 1. `codex-acp` on PATH
/// 2. `bunx` / `bun x` (user-installed Bun)
/// 3. `npx -y` (user-installed Node/npm)
///
/// Does **not** auto-install Bun — Theme Builder UI gates on
/// [`check_theme_builder_runtime`] / [`install_bun_for_theme_builder`].
///
/// `path_prepend` directories (e.g. folder of bundled `cdxthemex`) are put first on PATH.
/// `extra_env` is merged into the agent process environment.
fn build_acp_agent(
  path_prepend: &[PathBuf],
  extra_env: &[(String, String)],
) -> Result<(AcpAgent, String), String> {
  let mut path_env = std::env::var("PATH").unwrap_or_default();
  // Host-provided dirs first (bundled cdxthemex).
  for dir in path_prepend.iter().rev() {
    prepend_path_env(&mut path_env, dir);
  }
  // Ensure ChatGPT-bundled `codex` is visible to the adapter.
  if let Ok(codex) = find_codex_cli() {
    if let Some(dir) = codex.parent() {
      prepend_path_env(&mut path_env, dir);
    }
  }
  // Prefer known Bun install dir if present (even when not yet on PATH).
  if let Some(dir) = bun_bin_dir() {
    prepend_path_env(&mut path_env, &dir);
  }

  if let Some(local) = which("codex-acp") {
    return make_acp_agent(
      AcpAgentConfig::new(&local),
      local.display().to_string(),
      &path_env,
      extra_env,
    );
  }

  let has_bun = resolve_bun().is_some() || resolve_bunx().is_some();
  let has_npx = resolve_npx().is_some();

  if has_bun {
    return acp_via_bunx(&path_env, extra_env);
  }
  if has_npx {
    return acp_via_npx(&path_env, extra_env);
  }

  Err(
    "Theme Builder needs Bun (`bunx`) or Node.js (`npx`) to run codex-acp. \
     Open Theme Builder and use Install Bun, or install from https://bun.sh."
      .into(),
  )
}

fn make_acp_agent(
  cfg: AcpAgentConfig,
  label: String,
  path_env: &str,
  extra_env: &[(String, String)],
) -> Result<(AcpAgent, String), String> {
  let mut cfg = cfg.env("PATH", path_env);
  for (k, v) in extra_env {
    cfg = cfg.env(k, v);
  }
  Ok((AcpAgent::new(cfg), label))
}

fn bun_install_root() -> Option<PathBuf> {
  dirs_home().map(|h| h.join(".bun"))
}

fn bun_bin_dir() -> Option<PathBuf> {
  bun_install_root()
    .map(|h| h.join("bin"))
    .filter(|p| p.is_dir())
}

fn resolve_bun() -> Option<PathBuf> {
  which("bun").or_else(|| {
    dirs_home()
      .map(|h| h.join(".bun").join("bin").join(bun_exe_name()))
      .filter(|p| p.is_file())
  })
}

fn resolve_bunx() -> Option<PathBuf> {
  which("bunx").or_else(|| {
    dirs_home()
      .map(|h| h.join(".bun").join("bin").join(bunx_exe_name()))
      .filter(|p| p.is_file())
  })
}

fn resolve_npx() -> Option<PathBuf> {
  which("npx")
}

#[cfg(windows)]
fn bun_exe_name() -> &'static str {
  "bun.exe"
}
#[cfg(not(windows))]
fn bun_exe_name() -> &'static str {
  "bun"
}

#[cfg(windows)]
fn bunx_exe_name() -> &'static str {
  "bunx.exe"
}
#[cfg(not(windows))]
fn bunx_exe_name() -> &'static str {
  "bunx"
}

/// Platform triple used by Bun release assets / `@oven/bun-*` packages.
fn bun_release_target() -> Result<&'static str, String> {
  #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
  {
    return Ok("darwin-aarch64");
  }
  #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
  {
    return Ok("darwin-x64");
  }
  #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
  {
    return Ok("linux-aarch64");
  }
  #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
  {
    return Ok("linux-x64");
  }
  #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
  {
    return Ok("windows-x64");
  }
  #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
  {
    return Ok("windows-aarch64");
  }
  #[allow(unreachable_code)]
  Err(format!(
    "unsupported platform for Bun install ({}-{})",
    std::env::consts::OS,
    std::env::consts::ARCH
  ))
}

/// Install Bun by trying install scripts and direct binary mirrors in order.
async fn install_bun_multi_mirror() -> Result<(), String> {
  let mut errors: Vec<String> = Vec::new();

  // 1) Official / mirrored install scripts (handle PATH, bunx shim, unzip).
  match install_bun_via_scripts() {
    Ok(()) if resolve_bun().is_some() || resolve_bunx().is_some() => return Ok(()),
    Ok(()) => errors.push("install script finished but bun binary missing".into()),
    Err(e) => {
      tracing::warn!(error = %e, "Bun install script path failed");
      errors.push(e);
    }
  }

  // 2) Direct binary download (GitHub releases zip + jsDelivr / unpkg npm packages).
  match install_bun_via_direct_download().await {
    Ok(()) if resolve_bun().is_some() || resolve_bunx().is_some() => return Ok(()),
    Ok(()) => errors.push("direct download finished but bun binary missing".into()),
    Err(e) => {
      tracing::warn!(error = %e, "Bun direct download failed");
      errors.push(e);
    }
  }

  Err(format!(
    "failed to install Bun from all mirrors. Tried official script, GitHub, and jsDelivr. \
     Last errors: {}. Install manually from https://bun.sh",
    errors.join(" | ")
  ))
}

fn install_bun_via_scripts() -> Result<(), String> {
  #[cfg(windows)]
  {
    // PowerShell installers — try official hosts in order.
    const SCRIPT_URLS: &[&str] = &[
      "https://bun.sh/install.ps1",
      "https://bun.com/install.ps1",
      // jsDelivr mirror of oven-sh website install script when published
      "https://cdn.jsdelivr.net/gh/oven-sh/bun@main/src/cli/install.ps1",
    ];
    let mut last = String::from("no script URL tried");
    for url in SCRIPT_URLS {
      let cmd = format!("irm {url} | iex");
      tracing::info!(url, "trying Bun PowerShell installer");
      match std::process::Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &cmd])
        .status()
      {
        Ok(status) if status.success() => {
          if resolve_bun().is_some() || resolve_bunx().is_some() {
            return Ok(());
          }
          last = format!("{url}: success but bun not found");
        }
        Ok(status) => last = format!("{url}: exit {status}"),
        Err(e) => last = format!("{url}: {e}"),
      }
    }
    return Err(format!("PowerShell Bun install failed ({last})"));
  }

  #[cfg(not(windows))]
  {
    // curl | bash installers — try multiple script hosts.
    const SCRIPT_URLS: &[&str] = &[
      "https://bun.sh/install",
      "https://bun.com/install",
      // GitHub raw (install script lives on bun.sh; also try oven-sh docs mirrors)
      "https://raw.githubusercontent.com/oven-sh/bun/main/src/cli/install.sh",
      "https://cdn.jsdelivr.net/gh/oven-sh/bun@main/src/cli/install.sh",
    ];
    let mut last = String::from("no script URL tried");
    for url in SCRIPT_URLS {
      tracing::info!(url, "trying Bun install script");
      let shell = format!("curl -fsSL {url} | bash");
      match std::process::Command::new("bash")
        .args(["-lc", &shell])
        .status()
      {
        Ok(status) if status.success() => {
          if resolve_bun().is_some() || resolve_bunx().is_some() {
            return Ok(());
          }
          last = format!("{url}: success but bun not found");
        }
        Ok(status) => last = format!("{url}: exit {status}"),
        Err(e) => last = format!("{url}: {e}"),
      }
    }
    Err(format!("bash Bun install failed ({last})"))
  }
}

async fn install_bun_via_direct_download() -> Result<(), String> {
  let target = bun_release_target()?;
  let home = dirs_home().ok_or_else(|| "HOME/USERPROFILE not set".to_string())?;
  let install_root = home.join(".bun");
  let bin_dir = install_root.join("bin");
  std::fs::create_dir_all(&bin_dir).map_err(|e| format!("create ~/.bun/bin: {e}"))?;

  let dest = bin_dir.join(bun_exe_name());
  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(180))
    .redirect(reqwest::redirect::Policy::limited(10))
    .user_agent("CDXTheme-ThemeBuilder/0.1")
    .build()
    .map_err(|e| format!("http client: {e}"))?;

  // Prefer zip from GitHub (full release layout), then raw binary from npm CDNs.
  let zip_urls = [
    format!("https://github.com/oven-sh/bun/releases/latest/download/bun-{target}.zip"),
    // jsDelivr GitHub release proxy
    format!("https://cdn.jsdelivr.net/gh/oven-sh/bun-releases@latest/bun-{target}.zip"),
    // ghproxy-style is unreliable; skip. npmmirror zip of GitHub:
    format!("https://npmmirror.com/mirrors/bun/latest/bun-{target}.zip"),
  ];

  let mut last_err = String::new();
  for url in &zip_urls {
    tracing::info!(%url, "downloading Bun zip");
    match download_bytes(&client, url).await {
      Ok(bytes) if bytes.len() > 1_000_000 => match extract_bun_zip_to_bin(&bytes, &bin_dir) {
        Ok(()) => {
          ensure_bunx_shim(&bin_dir)?;
          if dest.is_file() {
            return Ok(());
          }
          last_err = format!("{url}: extracted but {dest:?} missing");
        }
        Err(e) => last_err = format!("{url}: extract failed: {e}"),
      },
      Ok(bytes) => last_err = format!("{url}: unexpected size {}", bytes.len()),
      Err(e) => last_err = format!("{url}: {e}"),
    }
  }

  // Raw executable from @oven/bun-* npm packages (jsDelivr / unpkg).
  let npm_pkg = format!("@oven/bun-{target}");
  let exe = bun_exe_name();
  let binary_urls = [
    format!("https://cdn.jsdelivr.net/npm/{npm_pkg}/bin/{exe}"),
    format!("https://unpkg.com/{npm_pkg}/bin/{exe}"),
    format!("https://registry.npmmirror.com/{npm_pkg}/latest"),
  ];

  for url in &binary_urls {
    // npmmirror registry returns JSON — skip pure registry URL for binary write.
    if url.contains("registry.npmmirror.com") {
      continue;
    }
    tracing::info!(%url, "downloading Bun binary");
    match download_bytes(&client, url).await {
      Ok(bytes) if bytes.len() > 1_000_000 => {
        std::fs::write(&dest, &bytes).map_err(|e| format!("write bun: {e}"))?;
        #[cfg(unix)]
        {
          use std::os::unix::fs::PermissionsExt;
          let mut perms = std::fs::metadata(&dest)
            .map_err(|e| format!("stat bun: {e}"))?
            .permissions();
          perms.set_mode(0o755);
          std::fs::set_permissions(&dest, perms).map_err(|e| format!("chmod bun: {e}"))?;
        }
        ensure_bunx_shim(&bin_dir)?;
        if dest.is_file() {
          return Ok(());
        }
      }
      Ok(bytes) => last_err = format!("{url}: unexpected size {}", bytes.len()),
      Err(e) => last_err = format!("{url}: {e}"),
    }
  }

  Err(format!("direct Bun download failed ({last_err})"))
}

async fn download_bytes(client: &reqwest::Client, url: &str) -> Result<Vec<u8>, String> {
  let resp = client
    .get(url)
    .send()
    .await
    .map_err(|e| format!("GET {url}: {e}"))?;
  if !resp.status().is_success() {
    return Err(format!("GET {url}: HTTP {}", resp.status()));
  }
  resp
    .bytes()
    .await
    .map(|b| b.to_vec())
    .map_err(|e| format!("read body {url}: {e}"))
}

fn extract_bun_zip_to_bin(zip_bytes: &[u8], bin_dir: &Path) -> Result<(), String> {
  // Write zip to a temp file and use system unzip / Expand-Archive (no zip crate dep).
  let tmp_dir = std::env::temp_dir().join(format!("cdxtheme-bun-{}", std::process::id()));
  let _ = std::fs::remove_dir_all(&tmp_dir);
  std::fs::create_dir_all(&tmp_dir).map_err(|e| format!("temp dir: {e}"))?;
  let zip_path = tmp_dir.join("bun.zip");
  std::fs::write(&zip_path, zip_bytes).map_err(|e| format!("write zip: {e}"))?;

  #[cfg(windows)]
  {
    let expand = format!(
      "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
      zip_path.display(),
      tmp_dir.display()
    );
    let status = std::process::Command::new("powershell")
      .args(["-NoProfile", "-Command", &expand])
      .status()
      .map_err(|e| format!("Expand-Archive: {e}"))?;
    if !status.success() {
      let _ = std::fs::remove_dir_all(&tmp_dir);
      return Err(format!("Expand-Archive exit {status}"));
    }
  }
  #[cfg(not(windows))]
  {
    let status = std::process::Command::new("unzip")
      .args([
        "-o",
        &zip_path.to_string_lossy(),
        "-d",
        &tmp_dir.to_string_lossy(),
      ])
      .status()
      .map_err(|e| {
        format!("unzip failed ({e}). Install `unzip` or use the official Bun installer.")
      })?;
    if !status.success() {
      let _ = std::fs::remove_dir_all(&tmp_dir);
      return Err(format!("unzip exit {status}"));
    }
  }

  // Release zip layout: bun-<target>/bun[.exe]
  let exe_name = bun_exe_name();
  let found = find_file_named(&tmp_dir, exe_name)
    .ok_or_else(|| format!("could not find {exe_name} inside downloaded zip"))?;
  let dest = bin_dir.join(exe_name);
  std::fs::create_dir_all(bin_dir).map_err(|e| format!("create bin dir: {e}"))?;
  std::fs::copy(&found, &dest).map_err(|e| format!("copy bun: {e}"))?;
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(&dest)
      .map_err(|e| format!("stat bun: {e}"))?
      .permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&dest, perms).map_err(|e| format!("chmod bun: {e}"))?;
  }

  let _ = std::fs::remove_dir_all(&tmp_dir);
  Ok(())
}

fn find_file_named(root: &Path, name: &str) -> Option<PathBuf> {
  let mut stack = vec![root.to_path_buf()];
  while let Some(dir) = stack.pop() {
    let entries = std::fs::read_dir(&dir).ok()?;
    for entry in entries.flatten() {
      let path = entry.path();
      if path.is_dir() {
        stack.push(path);
      } else if path.file_name().and_then(|n| n.to_str()) == Some(name) {
        return Some(path);
      }
    }
  }
  None
}

fn ensure_bunx_shim(bin_dir: &Path) -> Result<(), String> {
  let bun = bin_dir.join(bun_exe_name());
  if !bun.is_file() {
    return Ok(());
  }
  let bunx = bin_dir.join(bunx_exe_name());
  if bunx.is_file() {
    return Ok(());
  }
  #[cfg(windows)]
  {
    std::fs::copy(&bun, &bunx).map_err(|e| format!("create bunx.exe: {e}"))?;
  }
  #[cfg(unix)]
  {
    // Official installer links bunx → bun.
    let _ = std::fs::remove_file(&bunx);
    if std::os::unix::fs::symlink("bun", &bunx).is_err() {
      std::fs::copy(&bun, &bunx).map_err(|e| format!("create bunx: {e}"))?;
    }
  }
  Ok(())
}

fn acp_via_bunx(
  path_env: &str,
  extra_env: &[(String, String)],
) -> Result<(AcpAgent, String), String> {
  // Prefer bunx; fall back to `bun x` when bunx shim is missing.
  if let Some(bunx) = resolve_bunx() {
    let label = format!("{} {}", bunx.display(), CODEX_ACP_PKG);
    return make_acp_agent(
      AcpAgentConfig::new(&bunx).args([CODEX_ACP_PKG]),
      label,
      path_env,
      extra_env,
    );
  }
  if let Some(bun) = resolve_bun() {
    let label = format!("{} x {}", bun.display(), CODEX_ACP_PKG);
    return make_acp_agent(
      AcpAgentConfig::new(&bun).args(["x", CODEX_ACP_PKG]),
      label,
      path_env,
      extra_env,
    );
  }
  Err(
    "Bun is required to run codex-acp but was not found after install. \
     Install from https://bun.sh and restart CDXTheme."
      .into(),
  )
}

fn acp_via_npx(
  path_env: &str,
  extra_env: &[(String, String)],
) -> Result<(AcpAgent, String), String> {
  let npx = resolve_npx().ok_or_else(|| "npx not found".to_string())?;
  let label = format!("{} -y {CODEX_ACP_PKG}", npx.display());
  make_acp_agent(
    AcpAgentConfig::new(&npx).args(["-y", CODEX_ACP_PKG]),
    label,
    path_env,
    extra_env,
  )
}

/// List saved sessions (ACP `session/list` when available, else `~/.codex`).
pub fn list_sessions(limit: Option<usize>) -> Result<Vec<CodexSessionSummary>, String> {
  let limit = limit.unwrap_or(50).clamp(1, 200);
  // Filesystem is reliable offline and does not require spawning npx.
  list_sessions_from_disk(limit)
}

/// Async list that prefers ACP `session/list`, falls back to disk.
pub async fn list_sessions_async(limit: Option<usize>) -> Result<Vec<CodexSessionSummary>, String> {
  let limit = limit.unwrap_or(50).clamp(1, 200);
  match list_sessions_via_acp(limit).await {
    Ok(list) if !list.is_empty() => Ok(list),
    Ok(_) => list_sessions_from_disk(limit),
    Err(e) => {
      tracing::debug!("ACP session/list unavailable ({e}); using ~/.codex index");
      list_sessions_from_disk(limit)
    }
  }
}

async fn list_sessions_via_acp(limit: usize) -> Result<Vec<CodexSessionSummary>, String> {
  let (agent, _) = build_acp_agent(&[], &[])?;
  // List without cwd filter so all Codex history is available for intersection.
  let result = Client
    .builder()
    .on_receive_request(
      async move |request: RequestPermissionRequest, responder, _| {
        auto_approve_permission(request, responder);
        Ok(())
      },
      agent_client_protocol::on_receive_request!(),
    )
    .connect_with(agent, |connection: ConnectionTo<Agent>| async move {
      connection
        .send_request(InitializeRequest::new(ProtocolVersion::V1))
        .block_task()
        .await?;

      let resp = connection
        .send_request(ListSessionsRequest::new())
        .block_task()
        .await?;

      let mut out = Vec::new();
      for s in resp.sessions.into_iter().take(limit) {
        out.push(CodexSessionSummary {
          id: s.session_id.to_string(),
          title: s
            .title
            .filter(|t| !t.trim().is_empty())
            .unwrap_or_else(|| "Untitled session".into()),
          updated_at: s.updated_at.unwrap_or_default(),
          path: None,
          workspace_path: None,
        });
      }
      Ok(out)
    })
    .await
    .map_err(|e| format!("ACP session/list: {e}"))?;

  Ok(result)
}

fn list_sessions_from_disk(limit: usize) -> Result<Vec<CodexSessionSummary>, String> {
  let home = codex_home();
  let index_path = home.join("session_index.jsonl");
  let mut sessions: Vec<CodexSessionSummary> = Vec::new();

  if index_path.is_file() {
    let text = std::fs::read_to_string(&index_path).map_err(|e| e.to_string())?;
    for line in text.lines() {
      let line = line.trim();
      if line.is_empty() {
        continue;
      }
      let Ok(v) = serde_json::from_str::<Value>(line) else {
        continue;
      };
      let id = v
        .get("id")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
      if id.is_empty() {
        continue;
      }
      let title = v
        .get("thread_name")
        .and_then(|x| x.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("Untitled session")
        .to_string();
      let updated_at = v
        .get("updated_at")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
      let path = find_rollout_path(&home, &id).map(|p| p.display().to_string());
      sessions.push(CodexSessionSummary {
        id,
        title,
        updated_at,
        path,
        workspace_path: None,
      });
    }
  }

  if sessions.is_empty() {
    for path in scan_rollout_files(&home) {
      if let Some(meta) = read_session_meta_from_rollout(&path) {
        sessions.push(meta);
      }
    }
  }

  sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
  sessions.truncate(limit);
  Ok(sessions)
}

/// Set Codex `thread_name` for a session (session_index + rollout `session_meta`).
///
/// Theme Builder uses this so the host session list shows the user's description
/// instead of a generic first-line prompt.
pub fn rename_codex_session(session_id: &str, thread_name: &str) -> Result<(), String> {
  let session_id = session_id.trim();
  let name = thread_name.trim();
  if session_id.is_empty() {
    return Err("session id is empty".into());
  }
  if name.is_empty() {
    return Err("thread name is empty".into());
  }

  upsert_session_index_name(session_id, name)?;
  if let Some(path) = find_rollout_path(&codex_home(), session_id) {
    if let Err(e) = patch_rollout_thread_name(&path, name) {
      tracing::debug!(error = %e, path = %path.display(), "rollout thread_name patch skipped");
    }
  }
  tracing::info!(session = %session_id, name = %name, "codex session renamed");
  Ok(())
}

/// Permanently remove a Codex session (index entry, rollout file, and CLI delete when available).
pub fn delete_codex_session(session_id: &str) -> Result<(), String> {
  let session_id = session_id.trim();
  if session_id.is_empty() {
    return Err("session id is empty".into());
  }

  let mut errors: Vec<String> = Vec::new();

  // Prefer official CLI so SQLite / app-server state stays consistent.
  match find_codex_cli() {
    Ok(codex) => {
      let status = std::process::Command::new(&codex)
        .args(["delete", "--force", session_id])
        .status();
      match status {
        Ok(s) if s.success() => {
          tracing::info!(session = %session_id, "codex delete --force ok");
        }
        Ok(s) => {
          let msg = format!("codex delete --force exit {s}");
          tracing::warn!(session = %session_id, %msg);
          errors.push(msg);
        }
        Err(e) => {
          let msg = format!("codex delete failed to start: {e}");
          tracing::warn!(session = %session_id, %msg);
          errors.push(msg);
        }
      }
    }
    Err(e) => {
      tracing::debug!(error = %e, "codex CLI missing; filesystem delete only");
      errors.push(e);
    }
  }

  // Always clean disk artifacts (index + rollout), even if CLI already did.
  if let Err(e) = remove_session_index_entry(session_id) {
    errors.push(e);
  }
  if let Err(e) = remove_session_rollouts(session_id) {
    errors.push(e);
  }

  // Success if the session is gone from disk, even when CLI was unavailable.
  let still_indexed = session_index_has(session_id);
  let still_rollout = find_rollout_path(&codex_home(), session_id).is_some();
  if !still_indexed && !still_rollout {
    tracing::info!(session = %session_id, "codex session deleted from disk");
    return Ok(());
  }
  if still_indexed || still_rollout {
    return Err(format!(
      "failed to fully delete Codex session `{session_id}` (index={still_indexed}, rollout={still_rollout}): {}",
      errors.join(" | ")
    ));
  }
  Ok(())
}

fn session_index_path() -> PathBuf {
  codex_home().join("session_index.jsonl")
}

fn session_index_has(session_id: &str) -> bool {
  let path = session_index_path();
  let Ok(text) = std::fs::read_to_string(path) else {
    return false;
  };
  text.lines().any(|line| {
    serde_json::from_str::<Value>(line.trim())
      .ok()
      .and_then(|v| {
        v.get("id")
          .and_then(|x| x.as_str())
          .map(|s| s == session_id)
      })
      .unwrap_or(false)
  })
}

/// Upsert `thread_name` for `session_id` in `~/.codex/session_index.jsonl`.
fn upsert_session_index_name(session_id: &str, thread_name: &str) -> Result<(), String> {
  let path = session_index_path();
  let home = codex_home();
  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent).map_err(|e| format!("create CODEX_HOME: {e}"))?;
  }

  let mut rows: Vec<Value> = Vec::new();
  let mut found = false;
  if path.is_file() {
    let text = std::fs::read_to_string(&path).map_err(|e| format!("read session_index: {e}"))?;
    for line in text.lines() {
      let line = line.trim();
      if line.is_empty() {
        continue;
      }
      let Ok(mut v) = serde_json::from_str::<Value>(line) else {
        continue;
      };
      let id = v
        .get("id")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
      if id == session_id {
        if let Some(obj) = v.as_object_mut() {
          obj.insert("thread_name".into(), Value::String(thread_name.to_string()));
          obj.insert(
            "updated_at".into(),
            Value::String(humantime_iso(std::time::SystemTime::now())),
          );
        }
        found = true;
      }
      rows.push(v);
    }
  }

  if !found {
    rows.push(serde_json::json!({
      "id": session_id,
      "thread_name": thread_name,
      "updated_at": humantime_iso(std::time::SystemTime::now()),
    }));
  }

  let mut out = String::new();
  for v in rows {
    out.push_str(&serde_json::to_string(&v).map_err(|e| e.to_string())?);
    out.push('\n');
  }
  let tmp = home.join("session_index.jsonl.tmp");
  std::fs::write(&tmp, out.as_bytes()).map_err(|e| format!("write session_index tmp: {e}"))?;
  std::fs::rename(&tmp, &path).map_err(|e| format!("replace session_index: {e}"))?;
  Ok(())
}

fn remove_session_index_entry(session_id: &str) -> Result<(), String> {
  let path = session_index_path();
  if !path.is_file() {
    return Ok(());
  }
  let text = std::fs::read_to_string(&path).map_err(|e| format!("read session_index: {e}"))?;
  let mut out = String::new();
  let mut removed = false;
  for line in text.lines() {
    let line = line.trim();
    if line.is_empty() {
      continue;
    }
    let keep = match serde_json::from_str::<Value>(line) {
      Ok(v) => {
        let id = v.get("id").and_then(|x| x.as_str()).unwrap_or("");
        if id == session_id {
          removed = true;
          false
        } else {
          true
        }
      }
      Err(_) => true,
    };
    if keep {
      out.push_str(line);
      out.push('\n');
    }
  }
  if removed {
    std::fs::write(&path, out.as_bytes()).map_err(|e| format!("write session_index: {e}"))?;
    tracing::info!(session = %session_id, "removed from session_index.jsonl");
  }
  Ok(())
}

fn remove_session_rollouts(session_id: &str) -> Result<(), String> {
  let home = codex_home();
  let mut paths = scan_rollout_files(&home);
  // Also catch any remaining match if scan missed archived copies.
  paths.extend(scan_rollout_files(&home).into_iter().filter(|p| {
    p.file_name()
      .and_then(|n| n.to_str())
      .is_some_and(|n| n.contains(session_id))
  }));
  paths.sort();
  paths.dedup();

  let mut deleted = 0usize;
  for path in paths {
    let matches = path
      .file_name()
      .and_then(|n| n.to_str())
      .is_some_and(|n| n.contains(session_id));
    if !matches {
      continue;
    }
    match std::fs::remove_file(&path) {
      Ok(()) => {
        deleted += 1;
        tracing::info!(path = %path.display(), "deleted codex rollout");
      }
      Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
      Err(e) => return Err(format!("delete rollout {}: {e}", path.display())),
    }
  }
  if deleted == 0 {
    tracing::debug!(session = %session_id, "no rollout files found to delete");
  }
  Ok(())
}

/// Patch `thread_name` / `title` inside the first `session_meta` line of a rollout.
fn patch_rollout_thread_name(path: &Path, thread_name: &str) -> Result<(), String> {
  let text = std::fs::read_to_string(path).map_err(|e| format!("read rollout: {e}"))?;
  let mut out_lines: Vec<String> = Vec::new();
  let mut patched = false;
  for line in text.lines() {
    if patched {
      out_lines.push(line.to_string());
      continue;
    }
    let trimmed = line.trim();
    if trimmed.is_empty() {
      out_lines.push(line.to_string());
      continue;
    }
    match serde_json::from_str::<Value>(trimmed) {
      Ok(mut v) if v.get("type").and_then(|t| t.as_str()) == Some("session_meta") => {
        if let Some(payload) = v.get_mut("payload").and_then(|p| p.as_object_mut()) {
          payload.insert("thread_name".into(), Value::String(thread_name.to_string()));
          payload.insert("title".into(), Value::String(thread_name.to_string()));
          patched = true;
          out_lines.push(serde_json::to_string(&v).map_err(|e| e.to_string())?);
        } else {
          out_lines.push(line.to_string());
        }
      }
      _ => out_lines.push(line.to_string()),
    }
  }
  if !patched {
    return Ok(());
  }
  let mut body = out_lines.join("\n");
  if text.ends_with('\n') {
    body.push('\n');
  }
  std::fs::write(path, body.as_bytes()).map_err(|e| format!("write rollout: {e}"))?;
  Ok(())
}

/// Load transcript + meta for a session id (from `~/.codex` rollouts).
pub fn load_session(session_id: &str) -> Result<CodexSessionDetail, String> {
  let session_id = session_id.trim();
  if session_id.is_empty() {
    return Err("session id is empty".into());
  }
  let home = codex_home();

  let mut title = "Untitled session".to_string();
  let mut updated_at = String::new();
  if let Ok(list) = list_sessions_from_disk(200) {
    if let Some(s) = list.into_iter().find(|s| s.id == session_id) {
      title = s.title;
      updated_at = s.updated_at;
    }
  }

  let path = find_rollout_path(&home, session_id)
    .ok_or_else(|| format!("session rollout not found for id `{session_id}`"))?;

  let messages = parse_rollout_messages(&path)?;
  if updated_at.is_empty() {
    if let Ok(meta) = std::fs::metadata(&path) {
      if let Ok(mtime) = meta.modified() {
        updated_at = humantime_iso(mtime);
      }
    }
  }

  Ok(CodexSessionDetail {
    id: session_id.to_string(),
    title,
    updated_at,
    messages,
    workspace_path: None,
  })
}

fn scan_rollout_files(home: &Path) -> Vec<PathBuf> {
  let mut out = Vec::new();
  let roots = [home.join("sessions"), home.join("archived_sessions")];
  for root in roots {
    if !root.is_dir() {
      continue;
    }
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
      let Ok(rd) = std::fs::read_dir(&dir) else {
        continue;
      };
      for entry in rd.flatten() {
        let p = entry.path();
        if p.is_dir() {
          stack.push(p);
        } else if p
          .file_name()
          .and_then(|n| n.to_str())
          .is_some_and(|n| n.starts_with("rollout-") && n.ends_with(".jsonl"))
        {
          out.push(p);
        }
      }
    }
  }
  out.sort_by(|a, b| {
    let ma = a
      .metadata()
      .and_then(|m| m.modified())
      .ok()
      .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
    let mb = b
      .metadata()
      .and_then(|m| m.modified())
      .ok()
      .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
    mb.cmp(&ma)
  });
  out
}

fn find_rollout_path(home: &Path, session_id: &str) -> Option<PathBuf> {
  let needle = session_id.to_string();
  scan_rollout_files(home).into_iter().find(|p| {
    p.file_name()
      .and_then(|n| n.to_str())
      .is_some_and(|n| n.contains(&needle))
  })
}

fn read_session_meta_from_rollout(path: &Path) -> Option<CodexSessionSummary> {
  let text = std::fs::read_to_string(path).ok()?;
  let mut id = String::new();
  let mut title = String::new();
  for line in text.lines().take(40) {
    let Ok(v) = serde_json::from_str::<Value>(line) else {
      continue;
    };
    if v.get("type").and_then(|t| t.as_str()) == Some("session_meta") {
      if let Some(payload) = v.get("payload") {
        if let Some(s) = payload.get("id").and_then(|x| x.as_str()) {
          id = s.to_string();
        }
        if let Some(s) = payload
          .get("thread_name")
          .or_else(|| payload.get("title"))
          .and_then(|x| x.as_str())
        {
          title = s.to_string();
        }
      }
    }
  }
  if id.is_empty() {
    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
      let parts: Vec<&str> = name.split('-').collect();
      if parts.len() >= 5 {
        let candidate = parts[parts.len() - 5..].join("-");
        if candidate.len() >= 32 {
          id = candidate;
        }
      }
    }
  }
  if id.is_empty() {
    return None;
  }
  if title.is_empty() {
    title = "Untitled session".into();
  }
  let updated_at = path
    .metadata()
    .ok()
    .and_then(|m| m.modified().ok())
    .map(humantime_iso)
    .unwrap_or_default();
  Some(CodexSessionSummary {
    id,
    title,
    updated_at,
    path: Some(path.display().to_string()),
    workspace_path: None,
  })
}

fn parse_rollout_messages(path: &Path) -> Result<Vec<CodexSessionMessage>, String> {
  let text = std::fs::read_to_string(path).map_err(|e| format!("read rollout: {e}"))?;
  let mut messages: Vec<CodexSessionMessage> = Vec::new();

  for line in text.lines() {
    let line = line.trim();
    if line.is_empty() {
      continue;
    }
    let Ok(v) = serde_json::from_str::<Value>(line) else {
      continue;
    };
    let ty = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
    let payload = v.get("payload");

    match ty {
      "response_item" => {
        let Some(p) = payload else { continue };
        if p.get("type").and_then(|t| t.as_str()) != Some("message") {
          continue;
        }
        let role_raw = p.get("role").and_then(|r| r.as_str()).unwrap_or("");
        let role = match role_raw {
          "user" | "human" => "user",
          "assistant" | "agent" => "assistant",
          "system" | "developer" => continue,
          _ => continue,
        };
        let content = extract_message_text(p);
        if content.trim().is_empty() {
          continue;
        }
        if content.contains("<environment_context>")
          || content.contains("<permissions instructions>")
        {
          continue;
        }
        push_dedup(&mut messages, role, content);
      }
      "event_msg" => {
        let Some(p) = payload else { continue };
        let et = p.get("type").and_then(|t| t.as_str()).unwrap_or("");
        let (role, text) = match et {
          "user_message" => (
            "user",
            p.get("message")
              .or_else(|| p.get("text"))
              .and_then(|x| x.as_str())
              .unwrap_or("")
              .to_string(),
          ),
          "agent_message" | "assistant_message" => (
            "assistant",
            p.get("message")
              .or_else(|| p.get("text"))
              .and_then(|x| x.as_str())
              .unwrap_or("")
              .to_string(),
          ),
          _ => continue,
        };
        if text.trim().is_empty() {
          continue;
        }
        push_dedup(&mut messages, role, text);
      }
      _ => {}
    }
  }

  Ok(messages)
}

fn extract_message_text(payload: &Value) -> String {
  let content = match payload.get("content") {
    Some(c) => c,
    None => {
      return payload
        .get("text")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();
    }
  };
  if let Some(s) = content.as_str() {
    return s.to_string();
  }
  let mut out = String::new();
  if let Some(arr) = content.as_array() {
    for item in arr {
      if let Some(s) = item.as_str() {
        out.push_str(s);
        continue;
      }
      let ty = item.get("type").and_then(|t| t.as_str()).unwrap_or("");
      if matches!(ty, "input_text" | "output_text" | "text") {
        if let Some(t) = item.get("text").and_then(|t| t.as_str()) {
          if !out.is_empty() {
            out.push('\n');
          }
          out.push_str(t);
        }
      }
    }
  }
  out
}

fn push_dedup(messages: &mut Vec<CodexSessionMessage>, role: &str, content: String) {
  let content = content.trim().to_string();
  if content.is_empty() {
    return;
  }
  if let Some(last) = messages.last() {
    if last.role == role && last.content == content {
      return;
    }
  }
  messages.push(CodexSessionMessage {
    role: role.to_string(),
    content,
  });
}

fn humantime_iso(t: std::time::SystemTime) -> String {
  match t.duration_since(std::time::UNIX_EPOCH) {
    Ok(d) => format_unix_secs(d.as_secs() as i64),
    Err(_) => String::new(),
  }
}

fn format_unix_secs(secs: i64) -> String {
  let days = secs.div_euclid(86_400);
  let tod = secs.rem_euclid(86_400) as u32;
  let hour = tod / 3600;
  let min = (tod % 3600) / 60;
  let sec = tod % 60;
  let z = days + 719_468;
  let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
  let doe = (z - era * 146_097) as u64;
  let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
  let y = yoe as i64 + era * 400;
  let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
  let mp = (5 * doy + 2) / 153;
  let d = doy - (153 * mp + 2) / 5 + 1;
  let m = if mp < 10 { mp + 3 } else { mp - 9 };
  let y = if m <= 2 { y + 1 } else { y };
  format!("{y:04}-{m:02}-{d:02}T{hour:02}:{min:02}:{sec:02}Z")
}

fn auto_approve_permission(
  request: RequestPermissionRequest,
  responder: agent_client_protocol::Responder<RequestPermissionResponse>,
) {
  if let Some(opt) = request.options.first() {
    let _ = responder.respond(RequestPermissionResponse::new(
      RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(opt.option_id.clone())),
    ));
  } else {
    let _ = responder.respond(RequestPermissionResponse::new(
      RequestPermissionOutcome::Cancelled,
    ));
  }
}

/// Callback for live ACP transcript text (agent message + tool summary).
pub type CodexStreamCallback = Arc<dyn Fn(String) + Send + Sync + 'static>;

/// Options for a Theme Builder ACP turn.
#[derive(Clone, Default)]
pub struct CodexChatOptions {
  pub session_id: Option<String>,
  pub cwd: Option<PathBuf>,
  pub wait_ms: Option<u64>,
  /// Directories prepended to the agent process PATH (e.g. folder of bundled `cdxthemex`).
  pub path_prepend: Vec<PathBuf>,
  /// Extra env vars for the agent process (e.g. `CDXTHEME=/abs/path/cdxthemex`).
  pub extra_env: Vec<(String, String)>,
  /// Invoked whenever the turn transcript changes (for live UI streaming).
  pub on_stream: Option<CodexStreamCallback>,
}

impl std::fmt::Debug for CodexChatOptions {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CodexChatOptions")
      .field("session_id", &self.session_id)
      .field("cwd", &self.cwd)
      .field("wait_ms", &self.wait_ms)
      .field("path_prepend", &self.path_prepend)
      .field("extra_env", &self.extra_env)
      .field("on_stream", &self.on_stream.as_ref().map(|_| "<callback>"))
      .finish()
  }
}

/// Run one Theme Builder turn over ACP against Codex.
///
/// - No `session_id`: `session/new` then `session/prompt`
/// - With `session_id`: `session/load` then `session/prompt`
/// - `cwd` is the Theme Builder workspace root (skill + theme-dir); falls back to a temp dir
/// - Streams `agent_message_chunk` into the reply string
/// - Auto-approves permission requests
pub async fn send_and_wait(
  prompt: &str,
  session_id: Option<&str>,
  cwd: Option<&Path>,
  wait_ms: Option<u64>,
) -> Result<CodexChatResult, String> {
  send_and_wait_with(
    prompt,
    CodexChatOptions {
      session_id: session_id.map(|s| s.to_string()),
      cwd: cwd.map(Path::to_path_buf),
      wait_ms,
      ..Default::default()
    },
  )
  .await
}

/// Like [`send_and_wait`], with PATH/env for the bundled CDXTheme CLI.
pub async fn send_and_wait_with(
  prompt: &str,
  options: CodexChatOptions,
) -> Result<CodexChatResult, String> {
  let prompt = prompt.trim();
  if prompt.is_empty() {
    return Err("prompt is empty".into());
  }
  if prompt.len() > 32_000 {
    return Err("prompt is too long (max 32k characters)".into());
  }

  let wait_ms = options.wait_ms.unwrap_or(180_000).clamp(10_000, 600_000);
  let (agent, agent_label) = build_acp_agent(&options.path_prepend, &options.extra_env)?;
  let cwd = options
    .cwd
    .filter(|p| p.is_absolute())
    .unwrap_or_else(default_theme_builder_cwd);
  if !cwd.is_dir() {
    std::fs::create_dir_all(&cwd).map_err(|e| format!("create workspace cwd: {e}"))?;
  }
  let resume = options
    .session_id
    .as_deref()
    .map(str::trim)
    .filter(|s| !s.is_empty());

  tracing::info!(
    agent = %agent_label,
    chars = prompt.len(),
    wait_ms,
    resume = resume.unwrap_or(""),
    cwd = %cwd.display(),
    "theme builder → ACP session/prompt"
  );

  // Collect streamed agent text + tool activity for the chat UI.
  let transcript = Arc::new(Mutex::new(TurnTranscript::default()));
  let transcript_for_handler = transcript.clone();
  let on_stream = options.on_stream.clone();
  let last_streamed = Arc::new(Mutex::new(String::new()));
  let last_streamed_handler = last_streamed.clone();
  let prompt_owned = prompt.to_string();
  let resume_owned = resume.map(|s| s.to_string());

  let work = Client
    .builder()
    .on_receive_notification(
      async move |n: SessionNotification, _cx| {
        let rendered = {
          if let Ok(mut t) = transcript_for_handler.lock() {
            t.ingest_update(n.update);
            t.render()
          } else {
            String::new()
          }
        };
        if !rendered.is_empty() {
          if let Some(cb) = on_stream.as_ref() {
            let mut last = last_streamed_handler
              .lock()
              .unwrap_or_else(|e| e.into_inner());
            if *last != rendered {
              *last = rendered.clone();
              cb(rendered);
            }
          }
        }
        Ok(())
      },
      agent_client_protocol::on_receive_notification!(),
    )
    .on_receive_request(
      async move |request: RequestPermissionRequest, responder, _| {
        auto_approve_permission(request, responder);
        Ok(())
      },
      agent_client_protocol::on_receive_request!(),
    )
    .connect_with(agent, move |connection: ConnectionTo<Agent>| {
      let prompt_owned = prompt_owned.clone();
      let resume_owned = resume_owned.clone();
      let cwd = cwd.clone();
      async move {
        connection
          .send_request(InitializeRequest::new(ProtocolVersion::V1))
          .block_task()
          .await?;

        let session_id = if let Some(id) = resume_owned {
          connection
            .send_request(LoadSessionRequest::new(id.clone(), cwd.clone()))
            .block_task()
            .await?;
          id
        } else {
          let resp = connection
            .send_request(NewSessionRequest::new(cwd.clone()))
            .block_task()
            .await?;
          resp.session_id.to_string()
        };

        let prompt_resp = connection
          .send_request(PromptRequest::new(
            session_id.clone(),
            vec![ContentBlock::Text(TextContent::new(prompt_owned))],
          ))
          .block_task()
          .await?;

        let stop = format!("{:?}", prompt_resp.stop_reason);
        Ok((session_id, stop))
      }
    });

  let (session_id, stop_reason) = match timeout(Duration::from_millis(wait_ms), work).await {
    Ok(Ok(pair)) => pair,
    Ok(Err(e)) => {
      // Still surface any partial stream if we got one.
      let partial = transcript.lock().map(|t| t.render()).unwrap_or_default();
      if !partial.trim().is_empty() {
        return Ok(CodexChatResult {
          submitted: true,
          assistant_count: 1,
          stable: false,
          message: format!("ACP error (partial reply): {e}"),
          reply: partial,
          binary: Some(agent_label),
          session_id: None,
          stop_reason: None,
          package_path: None,
          installed_theme_id: None,
          installed_theme_name: None,
          applied: false,
        });
      }
      return Err(format!(
        "ACP error: {e}. Install Bun (https://bun.sh) or put `codex-acp` / `npx` on PATH. \
         Sign in with `codex login` if needed."
      ));
    }
    Err(_) => {
      let partial = transcript.lock().map(|t| t.render()).unwrap_or_default();
      if !partial.trim().is_empty() {
        return Ok(CodexChatResult {
          submitted: true,
          assistant_count: 1,
          stable: false,
          message: format!(
            "Codex ACP turn timed out after {}s (partial reply)",
            wait_ms / 1000
          ),
          reply: partial,
          binary: Some(agent_label),
          session_id: None,
          stop_reason: Some("timeout".into()),
          package_path: None,
          installed_theme_id: None,
          installed_theme_name: None,
          applied: false,
        });
      }
      return Err(format!(
        "Codex ACP turn timed out after {}s.",
        wait_ms / 1000
      ));
    }
  };

  let mut reply = transcript.lock().map(|t| t.render()).unwrap_or_default();

  // Fallback: load latest assistant text from Codex rollout if stream was empty.
  if reply.trim().is_empty() {
    if let Ok(detail) = load_session(&session_id) {
      if let Some(last) = detail
        .messages
        .iter()
        .rev()
        .find(|m| m.role == "assistant" && !m.content.trim().is_empty())
      {
        reply = summarize_for_ui(&last.content);
      }
    }
  }

  if reply.trim().is_empty() {
    reply = "Done.".into();
  }
  tracing::debug!(%stop_reason, "ACP turn finished");

  Ok(CodexChatResult {
    submitted: true,
    assistant_count: 1,
    stable: true,
    message: "Codex reply ready".into(),
    reply,
    binary: Some(agent_label),
    session_id: Some(session_id),
    stop_reason: Some(stop_reason),
    package_path: None,
    installed_theme_id: None,
    installed_theme_name: None,
    applied: false,
  })
}

/// Accumulates ACP `session/update` traffic into a user-visible summary.
///
/// Only agent message text is shown — no tools/actions/plans (those stay internal).
#[derive(Default)]
struct TurnTranscript {
  agent_text: String,
}

impl TurnTranscript {
  fn ingest_update(&mut self, update: SessionUpdate) {
    match update {
      SessionUpdate::AgentMessageChunk(chunk) => {
        if let Some(text) = content_block_text(&chunk.content) {
          self.agent_text.push_str(&text);
        }
      }
      // Thoughts, tools, and plans are intentionally omitted from the UI stream.
      SessionUpdate::AgentThoughtChunk(_)
      | SessionUpdate::ToolCall(_)
      | SessionUpdate::ToolCallUpdate(_)
      | SessionUpdate::Plan(_) => {}
      _ => {}
    }
  }

  fn render(&self) -> String {
    summarize_for_ui(&self.agent_text)
  }
}

/// Strip code, paths, and noise so the Theme Builder UI only shows a short summary.
fn summarize_for_ui(raw: &str) -> String {
  let without_code = strip_fenced_code(raw);
  let without_paths = redact_paths(&without_code);
  // Drop lines that look like shell, code, or action logs.
  let mut lines: Vec<String> = without_paths
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .filter(|line| !is_noisy_summary_line(line))
    .map(|s| s.to_string())
    .collect();

  // Cap length for the UI.
  if lines.len() > 8 {
    lines.truncate(8);
  }
  let mut out = lines.join("\n");
  if out.chars().count() > 800 {
    out = out.chars().take(800).collect::<String>() + "…";
  }
  out.trim().to_string()
}

fn strip_fenced_code(text: &str) -> String {
  let mut out = String::with_capacity(text.len());
  let mut in_fence = false;
  for line in text.lines() {
    let trimmed = line.trim_start();
    if trimmed.starts_with("```") {
      in_fence = !in_fence;
      continue;
    }
    if in_fence {
      continue;
    }
    out.push_str(line);
    out.push('\n');
  }
  out
}

fn redact_paths(text: &str) -> String {
  text
    .lines()
    .map(|line| {
      line
        .split_whitespace()
        .map(|tok| {
          if token_looks_like_path(tok) {
            "…"
          } else {
            tok
          }
        })
        .collect::<Vec<_>>()
        .join(" ")
    })
    .collect::<Vec<_>>()
    .join("\n")
}

fn token_looks_like_path(tok: &str) -> bool {
  let t =
    tok.trim_matches(|c: char| matches!(c, '`' | '"' | '\'' | ',' | '.' | ';' | ':' | ')' | '('));
  if t.is_empty() {
    return false;
  }
  let lower = t.to_ascii_lowercase();
  // Absolute paths
  if t.starts_with('/') || t.starts_with("~/") {
    return true;
  }
  if t.len() > 2 && t.as_bytes()[1] == b':' && (t.as_bytes()[2] == b'\\' || t.as_bytes()[2] == b'/')
  {
    return true;
  }
  if t.starts_with("\\\\") {
    return true;
  }
  // Relative path-ish tokens
  if t.contains('/') || t.contains('\\') {
    return true;
  }
  lower.ends_with(".cdxtheme")
    || lower.ends_with(".css")
    || lower.ends_with(".json")
    || lower.ends_with(".sh")
    || lower.ends_with(".js")
    || lower.ends_with(".ts")
}

fn is_noisy_summary_line(line: &str) -> bool {
  let t = line.trim();
  if t.is_empty() {
    return true;
  }
  // Action / tool style lines we used to inject
  if t.starts_with("**Actions**") || t.starts_with("• ") || t.starts_with("◇ ") {
    return true;
  }
  // Shell-ish / CLI
  if t.starts_with('$') || t.starts_with("# ") || t.starts_with("export ") {
    return true;
  }
  if t.starts_with("cdxthemex")
    || t.starts_with("bunx ")
    || t.starts_with("npx ")
    || t.starts_with("bun ")
  {
    return true;
  }
  // Code-ish
  if t.contains('{') && t.contains('}') && (t.contains(':') || t.contains(';')) {
    return true;
  }
  if t.starts_with("```") {
    return true;
  }
  // Whole line is only a path / filename
  if token_looks_like_path(t) {
    return true;
  }
  false
}

fn content_block_text(block: &ContentBlock) -> Option<String> {
  match block {
    ContentBlock::Text(t) => {
      if t.text.is_empty() {
        None
      } else {
        Some(t.text.clone())
      }
    }
    ContentBlock::ResourceLink(link) => {
      let name = if link.name.trim().is_empty() {
        link.uri.clone()
      } else {
        link.name.clone()
      };
      Some(format!("[{name}]({})", link.uri))
    }
    ContentBlock::Resource(res) => {
      // Prefer embedded text resources when present.
      // Structure varies; best-effort via debug/json is too heavy — skip if no simple text.
      let _ = res;
      None
    }
    ContentBlock::Image(_) | ContentBlock::Audio(_) => None,
    _ => None,
  }
}
