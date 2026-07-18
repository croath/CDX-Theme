//! Launch Codex / ChatGPT with remote debugging — thin Tauri wrapper over core.

use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// Ensure Codex is reachable on `port`, launching (or restarting) ChatGPT if needed.
pub async fn ensure_codex_debugging(app: &AppHandle, port: u16) -> Result<String, String> {
  let log = launch_log_path(app).ok();
  cdx_theme_core::launch::ensure_codex_debugging_with_log(port, log).await
}

/// Force-quit and relaunch Codex so it reloads config appearance settings.
pub async fn restart_codex_debugging(app: &AppHandle, port: u16) -> Result<String, String> {
  let log = launch_log_path(app).ok();
  cdx_theme_core::launch::restart_codex_debugging_with_log(port, log).await
}

pub use cdx_theme_core::launch::find_chatgpt_app;

fn launch_log_path(app: &AppHandle) -> Result<PathBuf, String> {
  let dir = app
    .path()
    .app_data_dir()
    .map_err(|e| format!("app data dir: {e}"))?;
  Ok(dir.join("codex-launch.log"))
}
