//! `cdxtheme apply` / `restore` — ensure host CDP, then inject or remove a theme.

use crate::cdp::{TargetUrlKind, wait_for_targets_with};
use crate::error::{CoreError, Result};
use crate::inject::{self, InjectOptions, InjectRunResult, default_cdp_port_for_app};
use crate::launch;
use cdx_theme_types::{APP_CODEX, APP_WORKBUDDY};
use std::path::Path;

fn validate_port(port: u16) -> Result<u16> {
  if !(1024..=65535).contains(&port) {
    return Err(CoreError::msg(format!(
      "invalid port {port} (use 1024–65535)"
    )));
  }
  Ok(port)
}

fn normalize_app(app: &str) -> Result<String> {
  // APP_CODEX / APP_WORKBUDDY are already lowercase string constants.
  match app.trim().to_ascii_lowercase().as_str() {
    "codex" => Ok(APP_CODEX.to_string()),
    "workbuddy" => Ok(APP_WORKBUDDY.to_string()),
    other => Err(CoreError::msg(format!(
      "unsupported --app `{other}` (supported: codex, workbuddy)"
    ))),
  }
}

fn url_kind_for_app(app: &str) -> TargetUrlKind {
  match app {
    APP_WORKBUDDY => TargetUrlKind::File,
    _ => TargetUrlKind::App,
  }
}

/// Ensure the host app is reachable over CDP (launch/restart if needed).
async fn ensure_cdp(app: &str, port: u16) -> Result<()> {
  let kind = url_kind_for_app(app);
  match wait_for_targets_with(port, 1_500, kind).await {
    Ok(targets) => {
      tracing::info!(
        app,
        port,
        targets = targets.len(),
        kind = kind.label(),
        "CDP connected"
      );
      Ok(())
    }
    Err(_) => {
      tracing::info!(app, port, "CDP not reachable; ensuring host is open");
      let msg = match app {
        APP_WORKBUDDY => launch::ensure_workbuddy_debugging(port)
          .await
          .map_err(CoreError::msg)?,
        _ => launch::ensure_codex_debugging(port)
          .await
          .map_err(CoreError::msg)?,
      };
      tracing::info!("{msg}");
      Ok(())
    }
  }
}

/// Apply a portable theme package to a host app via CDP.
///
/// 1. Probe CDP on `port` (default: 9335 for codex, 9336 for workbuddy)
/// 2. If unreachable, launch (or restart) the host app with remote debugging
/// 3. Inject the theme CSS/skin into live renderer targets
pub async fn apply_theme(
  app: &str,
  theme_path: &Path,
  port: Option<u16>,
  timeout_ms: u64,
) -> Result<InjectRunResult> {
  let app = normalize_app(app)?;

  if !theme_path.is_file() {
    return Err(CoreError::msg(format!(
      "theme package not found: {}",
      theme_path.display()
    )));
  }

  let port = validate_port(port.unwrap_or_else(|| default_cdp_port_for_app(&app)))?;
  ensure_cdp(&app, port).await?;

  let options = InjectOptions { port, timeout_ms };
  inject::apply_theme_package_for_app(&app, theme_path, options)
    .await
    .map_err(CoreError::msg)
}

/// Restore the host skin: ensure CDP, then remove injected theme DOM/CSS.
///
/// This is the inverse of [`apply_theme`]'s inject step. It does **not** rewrite
/// `config.toml` appearance keys (use [`crate::set_appearance_theme`] for mode).
///
/// Default host is Codex (`app://` on port 9335). Pass `app = "workbuddy"` for
/// WorkBuddy (`file://` on port 9336).
pub async fn restore_theme(
  app: Option<&str>,
  port: Option<u16>,
  timeout_ms: u64,
) -> Result<InjectRunResult> {
  let app = normalize_app(app.unwrap_or(APP_CODEX))?;
  let port = validate_port(port.unwrap_or_else(|| default_cdp_port_for_app(&app)))?;
  ensure_cdp(&app, port).await?;
  let options = InjectOptions { port, timeout_ms };
  inject::restore_default_theme_for_app(&app, options)
    .await
    .map_err(CoreError::msg)
}
