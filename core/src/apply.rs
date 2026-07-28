//! `cdxtheme apply` / `restore` — ensure host CDP, then inject or remove a theme.

use crate::cdp::wait_for_targets;
use crate::error::{CoreError, Result};
use crate::inject::DEFAULT_CDP_PORT;
use crate::inject::{self, InjectOptions, InjectRunResult};
use crate::launch;
use std::path::Path;

fn validate_port(port: u16) -> Result<u16> {
  if !(1024..=65535).contains(&port) {
    return Err(CoreError::msg(format!(
      "invalid port {port} (use 1024–65535)"
    )));
  }
  Ok(port)
}

/// Ensure Codex is reachable over CDP (launch/restart host if needed).
async fn ensure_cdp(port: u16) -> Result<()> {
  match wait_for_targets(port, 1_500).await {
    Ok(targets) => {
      tracing::info!(
        port,
        targets = targets.len(),
        "CDP connected (app:// targets)"
      );
      Ok(())
    }
    Err(_) => {
      tracing::info!(port, "CDP not reachable; ensuring Codex is open");
      let msg = launch::ensure_codex_debugging(port)
        .await
        .map_err(CoreError::msg)?;
      tracing::info!("{msg}");
      Ok(())
    }
  }
}

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

  let port = validate_port(port.unwrap_or(DEFAULT_CDP_PORT))?;
  ensure_cdp(port).await?;

  // Inject theme (app currently only codex; validated above).
  let _ = app;
  let options = InjectOptions { port, timeout_ms };
  inject::apply_theme_package(theme_path, options)
    .await
    .map_err(CoreError::msg)
}

/// Restore the host skin: ensure CDP, then remove injected theme DOM/CSS.
///
/// This is the inverse of [`apply_theme`]'s inject step. It does **not** rewrite
/// `config.toml` appearance keys (use [`crate::set_appearance_theme`] for mode).
pub async fn restore_theme(port: Option<u16>, timeout_ms: u64) -> Result<InjectRunResult> {
  let port = validate_port(port.unwrap_or(DEFAULT_CDP_PORT))?;
  ensure_cdp(port).await?;
  let options = InjectOptions { port, timeout_ms };
  inject::restore_default_theme(options)
    .await
    .map_err(CoreError::msg)
}
