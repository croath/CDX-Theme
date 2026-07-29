//! Theme Builder session registry + per-build workspaces under Tauri `app_data_dir`.
//!
//! Layout:
//! ```text
//! {app_data_dir}/theme_builder/
//!   sessions.json
//!   {workspace_id}/                 # ACP cwd for one build
//!     AGENTS.md
//!     .agents/skills/cdxtheme-theme/  # bundled skill copy
//!     theme/                          # theme-dir (from theme-starter)
//! ```
//!
//! Only sessions recorded here are list candidates; they must also still appear
//! in Codex's session history to be shown.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, path::BaseDirectory};
use uuid::Uuid;

const ROOT_DIR: &str = "theme_builder";
const SESSIONS_FILE: &str = "sessions.json";
const SKILL_INSTALL_REL: &str = ".agents/skills/cdxtheme-theme";
const THEME_DIR_NAME: &str = "theme";

/// One Theme Builder session we have started or continued via ACP.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackedSession {
  /// Codex / ACP session id.
  pub id: String,
  pub title: String,
  /// Random workspace folder name under `theme_builder/`.
  #[serde(default)]
  pub workspace_id: String,
  /// Absolute path to the workspace (ACP cwd).
  #[serde(default)]
  pub workspace_path: String,
  #[serde(default)]
  pub created_at: String,
  #[serde(default)]
  pub updated_at: String,
}

/// Result of preparing a new Theme Builder workspace (Start theme build).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreparedWorkspace {
  pub workspace_id: String,
  pub workspace_path: String,
}

/// Result of saving a user-uploaded hero image into a Theme Builder workspace.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedHeroImage {
  /// Path relative to workspace root, e.g. `theme/assets/hero.jpg`.
  pub relative_path: String,
  /// Path for theme.json `images.hero`, e.g. `assets/hero.jpg`.
  pub theme_asset_path: String,
  pub file_name: String,
}

const MAX_HERO_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoreFile {
  #[serde(default)]
  sessions: Vec<TrackedSession>,
}

fn root_dir(app: &AppHandle) -> Result<PathBuf, String> {
  let dir = app
    .path()
    .app_data_dir()
    .map_err(|e| format!("app data dir: {e}"))?
    .join(ROOT_DIR);
  fs::create_dir_all(&dir).map_err(|e| format!("create theme_builder dir: {e}"))?;
  Ok(dir)
}

fn store_path(app: &AppHandle) -> Result<PathBuf, String> {
  Ok(root_dir(app)?.join(SESSIONS_FILE))
}

fn now_iso() -> String {
  use std::time::{SystemTime, UNIX_EPOCH};
  let secs = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|d| d.as_secs() as i64)
    .unwrap_or(0);
  chrono_now(secs)
}

fn chrono_now(secs: i64) -> String {
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

fn load_file(app: &AppHandle) -> StoreFile {
  let Ok(path) = store_path(app) else {
    return StoreFile::default();
  };
  let Ok(raw) = fs::read_to_string(&path) else {
    return StoreFile::default();
  };
  serde_json::from_str(&raw).unwrap_or_default()
}

fn save_file(app: &AppHandle, store: &StoreFile) -> Result<(), String> {
  let path = store_path(app)?;
  let raw = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
  fs::write(&path, raw).map_err(|e| format!("write {}: {e}", path.display()))
}

/// Locate bundled skill (resource) or dev tree `assets/skill`.
pub fn resolve_skill_source(app: &AppHandle) -> Result<PathBuf, String> {
  if let Ok(path) = app.path().resolve("skill", BaseDirectory::Resource) {
    if path.is_dir() && path.join("SKILL.md").is_file() {
      return Ok(path);
    }
  }
  // Dev: app-tauri/../assets/skill
  let dev = Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets/skill");
  let dev = dev.canonicalize().unwrap_or(dev);
  if dev.is_dir() && dev.join("SKILL.md").is_file() {
    return Ok(dev);
  }
  Err("bundled theme skill not found (expected resource `skill/` or repo `assets/skill/`)".into())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
  fs::create_dir_all(dst).map_err(|e| format!("mkdir {}: {e}", dst.display()))?;
  let entries = fs::read_dir(src).map_err(|e| format!("read_dir {}: {e}", src.display()))?;
  for entry in entries {
    let entry = entry.map_err(|e| e.to_string())?;
    let ty = entry.file_type().map_err(|e| e.to_string())?;
    let from = entry.path();
    let to = dst.join(entry.file_name());
    if ty.is_dir() {
      copy_dir_recursive(&from, &to)?;
    } else if ty.is_file() {
      if let Some(parent) = to.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
      }
      fs::copy(&from, &to)
        .map_err(|e| format!("copy {} → {}: {e}", from.display(), to.display()))?;
    }
  }
  Ok(())
}

/// Resolve the app-bundled `cdxthemex` sidecar (next to main binary / staged binaries).
pub fn resolve_cdxthemex(app: &AppHandle) -> Result<PathBuf, String> {
  // 1) Same directory as the running app binary (production + most `tauri dev` setups).
  if let Ok(exe) = std::env::current_exe() {
    if let Some(dir) = exe.parent() {
      for name in ["cdxthemex", "cdxthemex.exe"] {
        let p = dir.join(name);
        if p.is_file() {
          return Ok(p.canonicalize().unwrap_or(p));
        }
      }
    }
  }

  // 2) Tauri resource / resource dir (some layouts).
  if let Ok(dir) = app.path().resource_dir() {
    for name in ["cdxthemex", "cdxthemex.exe"] {
      let p = dir.join(name);
      if p.is_file() {
        return Ok(p.canonicalize().unwrap_or(p));
      }
    }
  }

  // 3) Dev: app-tauri/binaries/cdxthemex-<triple>
  let binaries = Path::new(env!("CARGO_MANIFEST_DIR")).join("binaries");
  if binaries.is_dir() {
    if let Ok(rd) = fs::read_dir(&binaries) {
      for entry in rd.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("cdxthemex-") || name == "cdxthemex" || name == "cdxthemex.exe" {
          let p = entry.path();
          if p.is_file() {
            return Ok(p.canonicalize().unwrap_or(p));
          }
        }
      }
    }
  }

  // 4) PATH fallback (cargo install / local dev).
  if let Some(p) = which_on_path("cdxthemex") {
    return Ok(p);
  }

  Err(
    "bundled cdxthemex CLI not found (expected next to CDXTheme binary, or app-tauri/binaries/)"
      .into(),
  )
}

fn which_on_path(bin: &str) -> Option<PathBuf> {
  let path = std::env::var_os("PATH")?;
  for dir in std::env::split_paths(&path) {
    #[cfg(windows)]
    {
      for ext in [".exe", ".EXE", ""] {
        let mut name = std::ffi::OsString::from(bin);
        if !ext.is_empty() {
          name.push(ext);
        }
        let candidate = dir.join(&name);
        if candidate.is_file() {
          return Some(candidate);
        }
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

fn write_agents_md(
  workspace: &Path,
  skill_rel: &str,
  theme_rel: &str,
  cdxthemex: &Path,
  run_sh: &Path,
) -> Result<(), String> {
  let cli = cdxthemex.display().to_string();
  let run = run_sh.display().to_string();
  let theme_abs = workspace.join(theme_rel);
  let content = format!(
    r#"# Theme Builder workspace

This folder is a **CDXTheme Theme Builder** project created by the CDXTheme app.

## Skill

Use the installed skill:

- **Skill path:** `{skill_rel}/` (read `{skill_rel}/SKILL.md` first)
- Skill name: **cdxtheme-theme**
- Follow its Theme generation order and guardrails.

## Paths (resolve these before pack/apply)

| Variable | Path |
| --- | --- |
| **`/path/to/cdxthemex`** | `{cli}` |
| **`CDXTHEME` env** | `{cli}` |
| **theme-dir** | `{theme_rel}/` (absolute: `{theme_abs}`) |
| Skill starter | `{skill_rel}/assets/theme-starter/` |
| **`/path/to/run.sh`** | `{run}` |
| Probe scripts | `{skill_rel}/scripts/probe/` |

**Always use the absolute `/path/to/cdxthemex` above** (app-bundled CLI). Do not invent another binary path.

```bash
export CDXTHEME="{cli}"
# or call directly:
"{cli}" theme pack {theme_rel} -o output/theme.cdxtheme --force
```

Probes:

```bash
export CDXTHEME="{cli}"
"{run}" chat-layout
"{run}" work-layout
```

## How to work

1. Read `{skill_rel}/SKILL.md` and required references.
2. Work in **theme-dir** (`{theme_rel}/`): already scaffolded from theme-starter. Do not edit the skill starter in place as the shipped theme.
3. Edit **theme-dir** only: `theme.json`, `codex/` + `workbuddy/` partials, `assets/`.
4. Pack / apply / verify with the **bundled** CLI only:
   ```bash
   "{cli}" theme pack {theme_rel} -o output/theme.cdxtheme --force
   "{cli}" apply -t ./output/theme.cdxtheme
   "{cli}" verify layout
   ```
5. Keep packages declaration-only (no remote CSS, no theme JS).

## Reply style (UI-facing)

Chat replies are shown to end users. **Be extremely terse.**

- Final reply: **plain-language summary only** (about 2–5 short lines or bullets).
- Include only: theme name, mood/palette in words, what changed.
- **Do not** paste code, CSS, JSON, shell commands, or file trees.
- **Do not** list tool/actions, file paths, absolute paths, or workspace paths.
- **Do not** narrate every step. Work silently via tools; then one short summary.

## User request

The user will describe the look they want in chat. Apply their intent using the skill.
"#,
    skill_rel = skill_rel,
    theme_rel = theme_rel,
    cli = cli,
    run = run,
    theme_abs = theme_abs.display(),
  );
  fs::write(workspace.join("AGENTS.md"), content).map_err(|e| format!("write AGENTS.md: {e}"))
}

/// Create `{app_data_dir}/theme_builder/{random_id}`, install bundled skill, scaffold theme-dir.
pub fn prepare_workspace(app: &AppHandle) -> Result<PreparedWorkspace, String> {
  let skill_src = resolve_skill_source(app)?;
  let cdxthemex = resolve_cdxthemex(app)?;
  let workspace_id = Uuid::new_v4().to_string();
  let workspace = root_dir(app)?.join(&workspace_id);
  fs::create_dir_all(&workspace).map_err(|e| format!("create workspace: {e}"))?;

  // Install skill under .agents/skills/cdxtheme-theme
  let skill_dst = workspace.join(SKILL_INSTALL_REL);
  copy_dir_recursive(&skill_src, &skill_dst)?;

  // Make probe scripts executable when present.
  let run_sh = skill_dst.join("scripts/probe/run.sh");
  #[cfg(unix)]
  if run_sh.is_file() {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = fs::metadata(&run_sh) {
      let mut perms = meta.permissions();
      perms.set_mode(0o755);
      let _ = fs::set_permissions(&run_sh, perms);
    }
  }

  // theme-dir = theme/ (copy starter so agent can edit without touching skill tree)
  let theme_dir = workspace.join(THEME_DIR_NAME);
  let starter = skill_dst.join("assets/theme-starter");
  if starter.is_dir() {
    copy_dir_recursive(&starter, &theme_dir)?;
  } else {
    fs::create_dir_all(&theme_dir).map_err(|e| e.to_string())?;
  }

  write_agents_md(
    workspace.as_path(),
    SKILL_INSTALL_REL,
    THEME_DIR_NAME,
    &cdxthemex,
    &run_sh,
  )?;

  // Empty output/ for packs
  let _ = fs::create_dir_all(workspace.join("output"));

  // Convenience: env file for probes (CDXTHEME absolute path)
  let _ = fs::write(
    workspace.join(".cdxtheme-cli"),
    format!("{}\n", cdxthemex.display()),
  );

  let workspace_path = workspace
    .canonicalize()
    .unwrap_or(workspace)
    .to_string_lossy()
    .into_owned();

  tracing::info!(
    workspace_id = %workspace_id,
    path = %workspace_path,
    skill = %skill_src.display(),
    cdxthemex = %cdxthemex.display(),
    "theme builder workspace prepared"
  );

  Ok(PreparedWorkspace {
    workspace_id,
    workspace_path,
  })
}

/// Derive a short title from the user prompt / description.
///
/// Prefers the Theme Builder `Description:` block so sessions are named after the
/// user's description instead of the fixed "Create a Codex theme…" template line.
pub fn title_from_prompt(prompt: &str) -> String {
  let prompt = prompt.trim();
  if prompt.is_empty() {
    return "Untitled theme build".into();
  }

  // Hero-flow wire prompt embeds the user text under "Description:".
  if let Some(rest) = after_marker(prompt, "Description:") {
    if let Some(line) = first_title_line(rest) {
      return truncate_title(line);
    }
  }

  // Skill bootstrap ends with "User:\n{text}".
  if let Some(rest) = after_marker(prompt, "User:") {
    if let Some(line) = first_title_line(rest) {
      return truncate_title(line);
    }
  }

  // Fall back to first meaningful non-boilerplate line.
  if let Some(line) = first_title_line(prompt) {
    return truncate_title(line);
  }
  "Untitled theme build".into()
}

fn after_marker<'a>(text: &'a str, marker: &str) -> Option<&'a str> {
  let idx = text.find(marker)?;
  Some(text[idx + marker.len()..].trim_start())
}

fn first_title_line(text: &str) -> Option<&str> {
  for line in text.lines() {
    let line = line.trim();
    if line.is_empty() {
      continue;
    }
    if is_boilerplate_title_line(line) {
      continue;
    }
    return Some(line);
  }
  None
}

fn is_boilerplate_title_line(line: &str) -> bool {
  let lower = line.to_ascii_lowercase();
  line.starts_with('[')
    || line.starts_with('-')
    || lower.starts_with("create a codex theme from my hero")
    || lower.starts_with("hero image")
    || lower.starts_with("when done")
    || lower.starts_with("workspace")
    || lower.starts_with("you must")
    || lower.starts_with("theme-dir")
    || lower.starts_with("use the app-bundled")
    || lower.starts_with("goal:")
    || lower.starts_with("other useful")
    || lower.starts_with("keep packages")
    || lower.starts_with("## ")
    || lower.starts_with("prefer tools")
    || lower.starts_with("final message")
    || lower.starts_with("never")
    || lower.starts_with("no markdown")
    || lower.starts_with("if something failed")
    || lower.starts_with("set theme.json")
    || lower.starts_with("use var(--cdxtheme")
    || lower.starts_with("derive accent")
    || lower.starts_with("file:")
    || lower.starts_with("reply style")
}

fn truncate_title(line: &str) -> String {
  let one_line: String = line.chars().take(72).collect();
  let one_line = one_line.trim();
  if one_line.is_empty() {
    "Untitled theme build".into()
  } else {
    one_line.to_string()
  }
}

/// Build the first-turn system/user wire prompt that points Codex at the skill + bundled CLI.
pub fn skill_bootstrap_prompt(user_text: &str, workspace_path: &str, cdxthemex: &Path) -> String {
  let cli = cdxthemex.display();
  format!(
    "[CDXTheme Theme Builder]\n\
     Workspace (ACP cwd): {workspace_path}\n\
     You MUST follow the skill at `.agents/skills/cdxtheme-theme/SKILL.md` in this workspace.\n\
     theme-dir is `theme/` (already scaffolded from theme-starter). Edit theme-dir only.\n\
     Use the app-bundled CLI only (absolute path):\n\
       {cli}\n\
     Goal: implement the theme, then pack it so CDXTheme can install it:\n\
       \"{cli}\" theme pack theme -o output/theme.cdxtheme --force\n\
     Other useful commands:\n\
       \"{cli}\" apply -t ./output/theme.cdxtheme\n\
       export CDXTHEME=\"{cli}\" && .agents/skills/cdxtheme-theme/scripts/probe/run.sh chat-layout\n\
     Keep packages declaration-only (no remote CSS / theme JS).\n\
     When finished, leave a packed `.cdxtheme` under `output/`.\n\
     \n\
     ## Reply style (critical — user-facing UI)\n\
     Your chat reply is shown in the CDXTheme app. Output **only a short plain-text summary**.\n\
     Rules:\n\
     - Prefer tools/edits; almost no narration while working.\n\
     - Final message: **2–5 short lines or bullets max** (theme name, mood/colors, what changed).\n\
     - **Never** include: code, CSS, JSON, shell commands, file trees, tool/action lists.\n\
     - **Never** include: absolute paths, relative paths, workspace paths, package paths.\n\
     - No markdown code fences. No step-by-step monologue.\n\
     If something failed, one plain sentence — still no paths or code.\n\n\
     User:\n{user_text}"
  )
}

/// Write a user-uploaded hero image into `{workspace}/theme/assets/hero.<ext>`.
///
/// `content_base64` may be raw base64 or a `data:*;base64,...` URL.
pub fn save_hero_image(
  workspace: &Path,
  original_name: &str,
  content_base64: &str,
) -> Result<SavedHeroImage, String> {
  if !workspace.is_absolute() {
    return Err(format!(
      "workspace_path must be absolute: {}",
      workspace.display()
    ));
  }
  if !workspace.is_dir() {
    return Err(format!("workspace not found: {}", workspace.display()));
  }

  let b64 = strip_data_url_base64(content_base64)?;
  use base64::Engine;
  let bytes = base64::engine::general_purpose::STANDARD
    .decode(b64.trim())
    .map_err(|e| format!("invalid base64 hero image: {e}"))?;
  if bytes.is_empty() {
    return Err("hero image is empty".into());
  }
  if bytes.len() > MAX_HERO_BYTES {
    return Err(format!(
      "hero image exceeds {}MB limit",
      MAX_HERO_BYTES / (1024 * 1024)
    ));
  }

  let ext = hero_extension(original_name, &bytes)?;
  let assets = workspace.join(THEME_DIR_NAME).join("assets");
  fs::create_dir_all(&assets).map_err(|e| format!("create theme assets dir: {e}"))?;

  // Remove prior hero.* so we don't leave stale formats behind.
  if let Ok(entries) = fs::read_dir(&assets) {
    for entry in entries.flatten() {
      let name = entry.file_name();
      let name = name.to_string_lossy();
      if name.starts_with("hero.") {
        let _ = fs::remove_file(entry.path());
      }
    }
  }

  let file_name = format!("hero.{ext}");
  let dest = assets.join(&file_name);
  fs::write(&dest, &bytes).map_err(|e| format!("write hero image: {e}"))?;

  tracing::info!(
    path = %dest.display(),
    bytes = bytes.len(),
    "theme builder hero image saved"
  );

  Ok(SavedHeroImage {
    relative_path: format!("{THEME_DIR_NAME}/assets/{file_name}"),
    theme_asset_path: format!("assets/{file_name}"),
    file_name,
  })
}

fn strip_data_url_base64(input: &str) -> Result<&str, String> {
  let s = input.trim();
  if s.is_empty() {
    return Err("hero image payload is empty".into());
  }
  if let Some(rest) = s.strip_prefix("data:") {
    let (_, b64) = rest
      .split_once(',')
      .ok_or_else(|| "invalid data URL for hero image".to_string())?;
    if b64.is_empty() {
      return Err("hero image data URL has empty payload".into());
    }
    Ok(b64)
  } else {
    Ok(s)
  }
}

fn hero_extension(original_name: &str, bytes: &[u8]) -> Result<&'static str, String> {
  let from_name = Path::new(original_name)
    .extension()
    .and_then(|e| e.to_str())
    .map(|e| e.to_ascii_lowercase());
  let from_magic = match bytes {
    [0xFF, 0xD8, 0xFF, ..] => Some("jpg"),
    [0x89, b'P', b'N', b'G', ..] => Some("png"),
    [b'R', b'I', b'F', b'F', ..] if bytes.len() > 11 && &bytes[8..12] == b"WEBP" => Some("webp"),
    [b'G', b'I', b'F', b'8', ..] => Some("gif"),
    _ => None,
  };

  let ext = from_magic
    .or_else(|| {
      from_name.as_deref().and_then(|e| match e {
        "jpg" | "jpeg" => Some("jpg"),
        "png" => Some("png"),
        "webp" => Some("webp"),
        "gif" => Some("gif"),
        _ => None,
      })
    })
    .ok_or_else(|| "unsupported hero image type (use JPEG, PNG, WebP, or GIF)".to_string())?;

  Ok(ext)
}

/// Upsert a session after a successful Theme Builder ↔ Codex turn.
pub fn record_session(
  app: &AppHandle,
  session_id: &str,
  title_hint: Option<&str>,
  workspace_id: Option<&str>,
  workspace_path: Option<&str>,
) -> Result<(), String> {
  let session_id = session_id.trim();
  if session_id.is_empty() {
    return Ok(());
  }
  let now = now_iso();
  let mut store = load_file(app);
  let mut applied_title: Option<String> = None;
  if let Some(existing) = store.sessions.iter_mut().find(|s| s.id == session_id) {
    existing.updated_at = now;
    if let Some(hint) = title_hint.map(str::trim).filter(|s| !s.is_empty()) {
      if should_replace_session_title(&existing.title, hint) {
        existing.title = hint.to_string();
        applied_title = Some(hint.to_string());
      }
    }
    if let Some(wp) = workspace_path.map(str::trim).filter(|s| !s.is_empty()) {
      if existing.workspace_path.is_empty() {
        existing.workspace_path = wp.to_string();
      }
    }
    if let Some(wid) = workspace_id.map(str::trim).filter(|s| !s.is_empty()) {
      if existing.workspace_id.is_empty() {
        existing.workspace_id = wid.to_string();
      }
    }
  } else {
    let title = title_hint
      .map(str::trim)
      .filter(|s| !s.is_empty())
      .unwrap_or("Untitled theme build")
      .to_string();
    applied_title = Some(title.clone());
    store.sessions.push(TrackedSession {
      id: session_id.to_string(),
      title,
      workspace_id: workspace_id.unwrap_or("").trim().to_string(),
      workspace_path: workspace_path.unwrap_or("").trim().to_string(),
      created_at: now.clone(),
      updated_at: now,
    });
  }
  store
    .sessions
    .sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
  save_file(app, &store)?;

  // Keep Codex history name in sync with the Theme Builder description/title.
  if let Some(title) = applied_title.filter(|t| !t.trim().is_empty()) {
    if let Err(e) = cdx_theme_core::rename_codex_session(session_id, &title) {
      tracing::warn!(
        session = %session_id,
        error = %e,
        "failed to rename Codex session thread_name"
      );
    }
  }
  Ok(())
}

fn should_replace_session_title(existing: &str, hint: &str) -> bool {
  let existing = existing.trim();
  if existing.is_empty() || existing == "Untitled theme build" || existing == "Untitled session" {
    return true;
  }
  // Legacy bug: title was taken from the fixed hero-flow template first line.
  if existing
    .to_ascii_lowercase()
    .starts_with("create a codex theme from my hero")
  {
    return true;
  }
  // Allow upgrading a truncated title when the new hint is more descriptive.
  if hint.len() > existing.len() && hint.starts_with(existing.trim_end_matches('…')) {
    return true;
  }
  false
}

pub fn is_tracked(app: &AppHandle, session_id: &str) -> bool {
  let id = session_id.trim();
  if id.is_empty() {
    return false;
  }
  load_file(app).sessions.iter().any(|s| s.id == id)
}

pub fn workspace_path_for(app: &AppHandle, session_id: &str) -> Option<String> {
  let id = session_id.trim();
  load_file(app)
    .sessions
    .into_iter()
    .find(|s| s.id == id)
    .map(|s| s.workspace_path)
    .filter(|p| !p.is_empty())
}

/// Remove a Theme Builder session from app data, its workspace folder, **and Codex**.
///
/// Codex cleanup uses `codex delete --force` when available, plus `session_index.jsonl`
/// and rollout file removal under `~/.codex`.
pub fn delete_session(app: &AppHandle, session_id: &str) -> Result<(), String> {
  let session_id = session_id.trim();
  if session_id.is_empty() {
    return Err("session id is empty".into());
  }

  let mut store = load_file(app);
  let idx = store
    .sessions
    .iter()
    .position(|s| s.id == session_id)
    .or_else(|| {
      // Prefix match for shortened ids shown in the UI (e.g. first 8 chars).
      if session_id.len() >= 8 {
        let matches: Vec<usize> = store
          .sessions
          .iter()
          .enumerate()
          .filter(|(_, s)| s.id.starts_with(session_id) || session_id.starts_with(&s.id))
          .map(|(i, _)| i)
          .collect();
        if matches.len() == 1 {
          Some(matches[0])
        } else {
          None
        }
      } else {
        None
      }
    });
  let Some(idx) = idx else {
    // Still try Codex delete in case only host history remains.
    if let Err(e) = cdx_theme_core::delete_codex_session(session_id) {
      tracing::debug!(session = %session_id, error = %e, "codex delete for absent app session");
    }
    tracing::info!(session = %session_id, "theme builder session already absent");
    return Ok(());
  };
  let removed = store.sessions.remove(idx);
  save_file(app, &store)?;

  // Best-effort workspace cleanup (must stay under theme_builder root).
  if let Err(e) = remove_workspace_dir(
    app,
    removed.workspace_path.as_str(),
    removed.workspace_id.as_str(),
  ) {
    tracing::warn!(
      session = %session_id,
      error = %e,
      "theme builder session removed from registry but workspace cleanup failed"
    );
  } else {
    tracing::info!(
      session = %removed.id,
      "theme builder session deleted from app data"
    );
  }

  // Also remove from Codex history so it does not linger in CLI / Desktop.
  if let Err(e) = cdx_theme_core::delete_codex_session(&removed.id) {
    // App-side delete already succeeded; do not fail the IPC call.
    tracing::warn!(
      session = %removed.id,
      error = %e,
      "theme builder session removed from app data but Codex delete failed"
    );
  }

  Ok(())
}

fn remove_workspace_dir(
  app: &AppHandle,
  workspace_path: &str,
  workspace_id: &str,
) -> Result<(), String> {
  let root = root_dir(app)?;
  let root_canon = root.canonicalize().unwrap_or_else(|_| root.clone());

  let candidates: Vec<PathBuf> = {
    let mut out = Vec::new();
    let wp = workspace_path.trim();
    if !wp.is_empty() {
      out.push(PathBuf::from(wp));
    }
    let wid = workspace_id.trim();
    if !wid.is_empty() {
      out.push(root.join(wid));
    }
    out
  };

  for cand in candidates {
    if !cand.exists() {
      continue;
    }
    let canon = cand.canonicalize().unwrap_or_else(|_| cand.clone());
    if !canon.starts_with(&root_canon) {
      return Err(format!(
        "refusing to delete workspace outside theme_builder: {}",
        canon.display()
      ));
    }
    // Never delete the root itself.
    if canon == root_canon {
      return Err("refusing to delete theme_builder root".into());
    }
    if canon.is_dir() {
      fs::remove_dir_all(&canon)
        .map_err(|e| format!("delete workspace {}: {e}", canon.display()))?;
      tracing::info!(path = %canon.display(), "theme builder workspace deleted");
    }
  }
  Ok(())
}

/// Find the newest `.cdxtheme` package under a Theme Builder workspace.
///
/// Prefers `output/` (skill pack target), then walks the whole workspace.
/// Skips skill/theme source trees that are unlikely to hold a packed package.
pub fn find_newest_theme_package(workspace: &Path) -> Option<PathBuf> {
  if !workspace.is_dir() {
    return None;
  }

  let mut best: Option<(PathBuf, std::time::SystemTime)> = None;

  let consider = |path: PathBuf, best: &mut Option<(PathBuf, std::time::SystemTime)>| {
    if !path.is_file() {
      return;
    }
    let is_pkg = path
      .extension()
      .and_then(|e| e.to_str())
      .is_some_and(|e| e.eq_ignore_ascii_case("cdxtheme"));
    if !is_pkg {
      return;
    }
    let mtime = fs::metadata(&path)
      .and_then(|m| m.modified())
      .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
    match best {
      Some((_, best_t)) if mtime <= *best_t => {}
      _ => *best = Some((path, mtime)),
    }
  };

  // Prefer packed output first.
  let output = workspace.join("output");
  if output.is_dir() {
    if let Ok(entries) = fs::read_dir(&output) {
      for entry in entries.flatten() {
        consider(entry.path(), &mut best);
      }
    }
  }
  if best.is_some() {
    return best.map(|(p, _)| p);
  }

  // Fall back: shallow walk of workspace (skip heavy skill trees).
  fn walk(dir: &Path, depth: u32, best: &mut Option<(PathBuf, std::time::SystemTime)>) {
    if depth > 4 {
      return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
      return;
    };
    for entry in entries.flatten() {
      let path = entry.path();
      let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
      if name == "node_modules" || name == ".git" || name == ".agents" {
        continue;
      }
      if path.is_dir() {
        walk(&path, depth + 1, best);
      } else {
        let is_pkg = path
          .extension()
          .and_then(|e| e.to_str())
          .is_some_and(|e| e.eq_ignore_ascii_case("cdxtheme"));
        if !is_pkg {
          continue;
        }
        let mtime = fs::metadata(&path)
          .and_then(|m| m.modified())
          .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        match best {
          Some((_, best_t)) if mtime <= *best_t => {}
          _ => *best = Some((path, mtime)),
        }
      }
    }
  }

  walk(workspace, 0, &mut best);
  best.map(|(p, _)| p)
}

/// Sessions in **both** app data registry and Codex history.
pub fn list_intersection(
  app: &AppHandle,
  codex_sessions: Vec<cdx_theme_core::CodexSessionSummary>,
  limit: Option<usize>,
) -> Result<Vec<cdx_theme_core::CodexSessionSummary>, String> {
  let limit = limit.unwrap_or(50).clamp(1, 200);
  let mut store = load_file(app);

  let codex_by_id: HashMap<String, cdx_theme_core::CodexSessionSummary> = codex_sessions
    .into_iter()
    .map(|s| (s.id.clone(), s))
    .collect();
  let codex_ids: HashSet<&str> = codex_by_id.keys().map(|s| s.as_str()).collect();

  let before = store.sessions.len();
  store.sessions.retain(|s| codex_ids.contains(s.id.as_str()));
  if store.sessions.len() != before {
    let _ = save_file(app, &store);
  }

  let mut out: Vec<cdx_theme_core::CodexSessionSummary> = Vec::new();
  for tracked in &store.sessions {
    let Some(codex) = codex_by_id.get(&tracked.id) else {
      continue;
    };
    let title = if !tracked.title.trim().is_empty() {
      tracked.title.clone()
    } else if !codex.title.trim().is_empty() {
      codex.title.clone()
    } else {
      "Untitled theme build".into()
    };
    let updated_at = if tracked.updated_at >= codex.updated_at {
      tracked.updated_at.clone()
    } else if !codex.updated_at.is_empty() {
      codex.updated_at.clone()
    } else {
      tracked.updated_at.clone()
    };
    out.push(cdx_theme_core::CodexSessionSummary {
      id: tracked.id.clone(),
      title,
      updated_at,
      path: codex.path.clone(),
      workspace_path: if tracked.workspace_path.is_empty() {
        None
      } else {
        Some(tracked.workspace_path.clone())
      },
    });
  }

  out.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
  out.truncate(limit);
  Ok(out)
}
