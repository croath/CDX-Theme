//! Launch Codex / ChatGPT desktop with remote debugging for CDP injection.
//! Cross-platform: macOS (ChatGPT.app) and Windows (desktop + Microsoft Store Appx).

use crate::cdp::wait_for_targets;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

/// Default Codex desktop remote-debugging port (matches app).
pub const DEFAULT_CDP_PORT: u16 = 9335;

/// Ensure Codex is reachable on `port`, launching (or restarting) ChatGPT if needed.
/// Uses the default log path under `~/.cdxtheme/codex-launch.log`.
pub async fn ensure_codex_debugging(port: u16) -> Result<String, String> {
  ensure_codex_debugging_with_log(port, None).await
}

/// Like [`ensure_codex_debugging`], with an optional launch log path.
pub async fn ensure_codex_debugging_with_log(
  port: u16,
  log_path: Option<PathBuf>,
) -> Result<String, String> {
  if wait_for_targets(port, 1_500).await.is_ok() {
    return Ok(format!("Codex already exposing CDP on port {port}"));
  }

  // If ChatGPT is running without this debug port, restart it with the flag.
  if is_chatgpt_running() {
    tracing::info!("ChatGPT is running without CDP on {port}; restarting with remote debugging");
    return restart_codex_debugging_with_log(port, log_path).await;
  }

  tracing::info!("Opening ChatGPT with remote debugging on port {port}");
  launch_codex_debugging(port, log_path.as_deref()).await
}

/// Force-quit and relaunch Codex with remote debugging.
pub async fn restart_codex_debugging(port: u16) -> Result<String, String> {
  restart_codex_debugging_with_log(port, None).await
}

/// Like [`restart_codex_debugging`], with an optional launch log path.
pub async fn restart_codex_debugging_with_log(
  port: u16,
  log_path: Option<PathBuf>,
) -> Result<String, String> {
  if is_chatgpt_running() {
    quit_chatgpt();
    tokio::time::sleep(Duration::from_millis(800)).await;
    wait_until_chatgpt_exited(Duration::from_secs(15)).await;
    tokio::time::sleep(Duration::from_millis(500)).await;
  }
  launch_codex_debugging(port, log_path.as_deref()).await
}

async fn launch_codex_debugging(port: u16, log_path: Option<&Path>) -> Result<String, String> {
  for _ in 0..20 {
    if !port_in_use(port) {
      break;
    }
    tokio::time::sleep(Duration::from_millis(250)).await;
  }
  if port_in_use(port) {
    return Err(format!("port {port} is already in use by another process"));
  }

  let exe = find_chatgpt_app().ok_or_else(|| {
    #[cfg(target_os = "windows")]
    {
      "Codex/ChatGPT app not found. Install the OpenAI Codex (ChatGPT) desktop app \
       from the Microsoft Store or desktop installer, then try again."
        .to_string()
    }
    #[cfg(target_os = "macos")]
    {
      "Codex/ChatGPT app not found (looked for com.openai.codex / ChatGPT.app)".to_string()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
      "Codex/ChatGPT app not found on this platform".to_string()
    }
  })?;

  if !exe.is_file() {
    return Err(format!("ChatGPT executable missing: {}", exe.display()));
  }

  let log_path = log_path
    .map(Path::to_path_buf)
    .unwrap_or_else(default_launch_log_path);
  if let Some(parent) = log_path.parent() {
    let _ = fs::create_dir_all(parent);
  }

  let log_file = OpenOptions::new()
    .create(true)
    .append(true)
    .open(&log_path)
    .map_err(|e| format!("open launch log: {e}"))?;
  let log_err = log_file
    .try_clone()
    .map_err(|e| format!("clone launch log: {e}"))?;

  {
    let mut f = OpenOptions::new()
      .create(true)
      .append(true)
      .open(&log_path)
      .ok();
    if let Some(ref mut f) = f {
      let _ = writeln!(
        f,
        "\n--- launch {} port={port} exe={} ---",
        chrono_like_now(),
        exe.display()
      );
    }
  }

  let mut cmd = Command::new(&exe);
  cmd
    .arg("--remote-debugging-address=127.0.0.1")
    .arg(format!("--remote-debugging-port={port}"))
    .stdout(Stdio::from(log_file))
    .stderr(Stdio::from(log_err));

  #[cfg(target_os = "windows")]
  {
    use std::os::windows::process::CommandExt;
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
  }

  cmd
    .spawn()
    .map_err(|e| format!("failed to launch ChatGPT ({}): {e}", exe.display()))?;

  for _ in 0..90 {
    if wait_for_targets(port, 400).await.is_ok() {
      tokio::time::sleep(Duration::from_millis(1_200)).await;
      return Ok(format!(
        "Launched ChatGPT with --remote-debugging-port={port}"
      ));
    }
    tokio::time::sleep(Duration::from_millis(400)).await;
  }

  Err(format!(
    "ChatGPT launched but CDP on port {port} not ready within ~35s (see {})",
    log_path.display()
  ))
}

fn default_launch_log_path() -> PathBuf {
  let base = std::env::var_os("HOME")
    .or_else(|| std::env::var_os("USERPROFILE"))
    .map(PathBuf::from)
    .unwrap_or_else(std::env::temp_dir);
  base.join(".cdxtheme").join("codex-launch.log")
}

fn chrono_like_now() -> String {
  use std::time::{SystemTime, UNIX_EPOCH};
  let secs = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|d| d.as_secs())
    .unwrap_or(0);
  format!("unix:{secs}")
}

/// Locate the ChatGPT / Codex desktop executable.
pub fn find_chatgpt_app() -> Option<PathBuf> {
  #[cfg(target_os = "macos")]
  {
    return find_chatgpt_app_macos();
  }
  #[cfg(target_os = "windows")]
  {
    return find_chatgpt_app_windows();
  }
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  {
    None
  }
}

#[cfg(target_os = "macos")]
fn find_chatgpt_app_macos() -> Option<PathBuf> {
  let home = std::env::var_os("HOME").map(PathBuf::from);
  let candidates = [
    PathBuf::from("/Applications/ChatGPT.app"),
    home
      .as_ref()
      .map(|h| h.join("Applications/ChatGPT.app"))
      .unwrap_or_default(),
  ];
  for c in candidates {
    let exe = c.join("Contents/MacOS/ChatGPT");
    if exe.is_file() {
      return Some(exe);
    }
  }
  if let Ok(output) = Command::new("mdfind")
    .arg("kMDItemCFBundleIdentifier == \"com.openai.codex\"")
    .output()
  {
    if output.status.success() {
      let text = String::from_utf8_lossy(&output.stdout);
      for line in text.lines() {
        let p = PathBuf::from(line.trim());
        let exe = p.join("Contents/MacOS/ChatGPT");
        if exe.is_file() {
          return Some(exe);
        }
      }
    }
  }
  None
}

#[cfg(target_os = "windows")]
fn find_chatgpt_app_windows() -> Option<PathBuf> {
  let mut candidates: Vec<PathBuf> = Vec::new();

  if let Some(local) = std::env::var_os("LOCALAPPDATA").map(PathBuf::from) {
    candidates.push(local.join("Programs").join("ChatGPT").join("ChatGPT.exe"));
    candidates.push(local.join("Programs").join("Codex").join("ChatGPT.exe"));
    candidates.push(local.join("Programs").join("Codex").join("Codex.exe"));
    candidates.push(
      local
        .join("Microsoft")
        .join("WindowsApps")
        .join("ChatGPT.exe"),
    );
  }
  if let Some(pf) = std::env::var_os("ProgramFiles").map(PathBuf::from) {
    candidates.push(pf.join("ChatGPT").join("ChatGPT.exe"));
    candidates.push(pf.join("OpenAI").join("ChatGPT").join("ChatGPT.exe"));
    candidates.push(pf.join("Codex").join("ChatGPT.exe"));
  }
  if let Some(pf86) = std::env::var_os("ProgramFiles(x86)").map(PathBuf::from) {
    candidates.push(pf86.join("ChatGPT").join("ChatGPT.exe"));
    candidates.push(pf86.join("OpenAI").join("ChatGPT").join("ChatGPT.exe"));
  }

  for c in &candidates {
    if c.is_file() {
      return Some(c.clone());
    }
  }

  if let Some(exe) = find_chatgpt_via_appx() {
    return Some(exe);
  }

  if let Some(exe) = find_via_where(&["ChatGPT.exe", "Codex.exe"]) {
    return Some(exe);
  }

  None
}

#[cfg(target_os = "windows")]
fn find_chatgpt_via_appx() -> Option<PathBuf> {
  let script = r#"
$ErrorActionPreference = 'SilentlyContinue'
$p = Get-AppxPackage -Name OpenAI.Codex |
  Sort-Object {[version]$_.Version} -Descending |
  Select-Object -First 1
if (-not $p) {
  $p = Get-AppxPackage |
    Where-Object { $_.Name -match 'OpenAI\.(Codex|ChatGPT)' -or $_.PackageFullName -match 'ChatGPT' } |
    Sort-Object {[version]$_.Version} -Descending |
    Select-Object -First 1
}
if (-not $p) { exit 1 }
$exe = Join-Path $p.InstallLocation 'app\ChatGPT.exe'
if (-not (Test-Path -LiteralPath $exe)) {
  $exe = Join-Path $p.InstallLocation 'ChatGPT.exe'
}
if (Test-Path -LiteralPath $exe) { Write-Output $exe; exit 0 }
exit 1
"#;

  let mut cmd = Command::new("powershell");
  cmd
    .args([
      "-NoProfile",
      "-NonInteractive",
      "-ExecutionPolicy",
      "Bypass",
      "-Command",
      script,
    ])
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::null());
  apply_no_window(&mut cmd);

  let output = cmd.output().ok()?;
  if !output.status.success() {
    return None;
  }
  let path = String::from_utf8_lossy(&output.stdout)
    .lines()
    .map(str::trim)
    .find(|l| !l.is_empty())
    .map(PathBuf::from)?;
  if path.is_file() { Some(path) } else { None }
}

#[cfg(target_os = "windows")]
fn find_via_where(names: &[&str]) -> Option<PathBuf> {
  for name in names {
    let mut cmd = Command::new("where.exe");
    cmd
      .arg(name)
      .stdin(Stdio::null())
      .stdout(Stdio::piped())
      .stderr(Stdio::null());
    apply_no_window(&mut cmd);
    let Ok(output) = cmd.output() else {
      continue;
    };
    if !output.status.success() {
      continue;
    }
    for line in String::from_utf8_lossy(&output.stdout).lines() {
      let p = PathBuf::from(line.trim());
      if p.is_file() {
        if fs::metadata(&p).map(|m| m.len() > 0).unwrap_or(false) {
          return Some(p);
        }
      }
    }
  }
  None
}

fn is_chatgpt_running() -> bool {
  #[cfg(target_os = "macos")]
  {
    return Command::new("pgrep")
      .args(["-f", "/ChatGPT.app/Contents/MacOS/ChatGPT"])
      .output()
      .map(|o| o.status.success() && !o.stdout.is_empty())
      .unwrap_or(false);
  }
  #[cfg(target_os = "windows")]
  {
    process_image_running("ChatGPT.exe") || process_image_running("Codex.exe")
  }
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  {
    false
  }
}

#[cfg(target_os = "windows")]
fn process_image_running(image: &str) -> bool {
  let mut cmd = Command::new("tasklist");
  cmd
    .args(["/FI", &format!("IMAGENAME eq {image}"), "/NH"])
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::null());
  apply_no_window(&mut cmd);
  cmd
    .output()
    .map(|o| {
      let s = String::from_utf8_lossy(&o.stdout).to_ascii_lowercase();
      s.contains(&image.to_ascii_lowercase())
    })
    .unwrap_or(false)
}

fn quit_chatgpt() {
  #[cfg(target_os = "macos")]
  {
    let _ = Command::new("osascript")
      .args(["-e", "tell application id \"com.openai.codex\" to quit"])
      .status();
    let _ = Command::new("pkill")
      .args(["-f", "/ChatGPT.app/Contents/MacOS/ChatGPT"])
      .status();
  }
  #[cfg(target_os = "windows")]
  {
    for image in ["ChatGPT.exe", "Codex.exe"] {
      let mut cmd = Command::new("taskkill");
      cmd
        .args(["/IM", image, "/F", "/T"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
      apply_no_window(&mut cmd);
      let _ = cmd.status();
    }
  }
}

async fn wait_until_chatgpt_exited(timeout: Duration) {
  let deadline = tokio::time::Instant::now() + timeout;
  while tokio::time::Instant::now() < deadline {
    if !is_chatgpt_running() {
      return;
    }
    tokio::time::sleep(Duration::from_millis(250)).await;
  }
}

fn port_in_use(port: u16) -> bool {
  match std::net::TcpListener::bind(("127.0.0.1", port)) {
    Ok(listener) => {
      drop(listener);
      false
    }
    Err(_) => true,
  }
}

#[cfg(target_os = "windows")]
fn apply_no_window(cmd: &mut Command) {
  use std::os::windows::process::CommandExt;
  const CREATE_NO_WINDOW: u32 = 0x0800_0000;
  cmd.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
fn apply_no_window(_cmd: &mut Command) {}
