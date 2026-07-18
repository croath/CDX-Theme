//! `cdxtheme apply` — ensure host CDP, then inject a theme package.

use crate::cdp::wait_for_targets;
use crate::error::{CoreError, Result};
use crate::inject::DEFAULT_CDP_PORT;
use crate::inject::{self, InjectOptions, InjectRunResult};
use crate::launch;
use std::path::Path;

/// Apply a portable theme package to a host app via CDP.
///
/// 1. Probe CDP on `port`
/// 2. If unreachable, launch (or restart) the host app with remote debugging
/// 3. Inject the theme CSS/skin into live renderer targets
pub async fn apply_theme(
  app: &str,
  theme_path: &Path,
  port: Option<u16>,
  timeout_ms: u64,
) -> Result<InjectRunResult> {
  let app = app.trim().to_ascii_lowercase();
  if app != "codex" {
    return Err(CoreError::msg(format!(
      "unsupported --app `{app}` (supported: codex)"
    )));
  }

  if !theme_path.is_file() {
    return Err(CoreError::msg(format!(
      "theme package not found: {}",
      theme_path.display()
    )));
  }

  let port = port.unwrap_or(DEFAULT_CDP_PORT);
  if !(1024..=65535).contains(&port) {
    return Err(CoreError::msg(format!(
      "invalid port {port} (use 1024–65535)"
    )));
  }

  // 1) Detect CDP; open app if needed.
  match wait_for_targets(port, 1_500).await {
    Ok(targets) => {
      tracing::info!(
        port,
        targets = targets.len(),
        "CDP connected (app:// targets)"
      );
    }
    Err(_) => {
      tracing::info!(port, "CDP not reachable; ensuring Codex is open");
      let msg = launch::ensure_codex_debugging(port)
        .await
        .map_err(CoreError::msg)?;
      tracing::info!("{msg}");
    }
  }

  // 2) Inject theme (app currently only codex; validated above).
  let _ = app;
  let options = InjectOptions { port, timeout_ms };
  inject::apply_theme_package(theme_path, options)
    .await
    .map_err(CoreError::msg)
}
