use crate::app_state::{AppState, CdpServerStatus};
use crate::codex_launch;
use crate::image_cache;
use crate::injector::{self, InjectOptions, load_theme_package};
use crate::settings_store;
use crate::theme_catalog;
use crate::theme_tool;
use cdx_theme_types::ThemeMetadata;
use tauri::{AppHandle, Manager, State};

fn inject_options(state: &AppState) -> InjectOptions {
  InjectOptions {
    port: state.cdp_port(),
    // Themes with multi-MB hero/texture need headroom for CDP WS + atob→blob.
    timeout_ms: 120_000,
  }
}

/// Runtime local theme list: scan builtin + user `.cdxtheme` packages.
#[tauri::command]
pub async fn retrieve_local_theme_list(
  app: AppHandle,
  _state: State<'_, AppState>,
) -> Result<Vec<ThemeMetadata>, String> {
  theme_catalog::discover_themes(&app)
}

/// Remote recommend catalog from `https://s3.cdxtheme.com/themes/index.json`.
/// Pass `force = true` to bypass the in-memory 2-minute cache.
/// Preview images are resolved through the local disk cache (`data:` URLs).
#[tauri::command(rename_all = "snake_case")]
pub async fn fetch_remote_theme_catalog(
  force: Option<bool>,
  app: AppHandle,
  _state: State<'_, AppState>,
) -> Result<Vec<ThemeMetadata>, String> {
  theme_catalog::fetch_remote_theme_catalog(&app, force.unwrap_or(false)).await
}

/// Resolve any image URL to a local `data:` URL (disk-cached for HTTP(S)).
/// Use when a UI surface still has a remote preview URL (e.g. before catalog localization).
#[tauri::command(rename_all = "snake_case")]
pub async fn resolve_cached_image(url: String, app: AppHandle) -> Result<String, String> {
  image_cache::resolve_to_data_url(&app, &url).await
}

/// Current CDP server status (updated by background monitor).
#[tauri::command]
pub async fn cdp_status(state: State<'_, AppState>) -> Result<CdpServerStatus, String> {
  Ok(state.cdp_status())
}

/// Sync native window background with the UI theme (no transparent window / private API).
/// Keeps the macOS overlay titlebar area from flashing the wrong color under traffic lights.
#[tauri::command(rename_all = "snake_case")]
pub async fn set_window_appearance(dark: bool, app: AppHandle) -> Result<(), String> {
  let Some(window) = app.get_webview_window("main") else {
    return Ok(());
  };
  // Approximate CSS --background (light / dark) as solid RGBA.
  let color = if dark {
    tauri::window::Color(28, 33, 32, 255) // ~oklch(0.145 0.015 150)
  } else {
    tauri::window::Color(248, 250, 246, 255) // ~oklch(0.985 0.004 120)
  };
  window
    .set_background_color(Some(color))
    .map_err(|e| e.to_string())?;
  Ok(())
}

#[tauri::command]
pub async fn get_cdp_port(state: State<'_, AppState>) -> Result<u16, String> {
  Ok(state.cdp_port())
}

/// Persist CDP port and relaunch Codex with the new `--remote-debugging-port` if needed.
#[tauri::command(rename_all = "snake_case")]
pub async fn set_cdp_port(
  port: u16,
  app: AppHandle,
  state: State<'_, AppState>,
) -> Result<u16, String> {
  if !settings_store::is_valid_port(port) {
    return Err(format!("invalid port {port} (use 1024–65535)"));
  }

  let mut settings = settings_store::load(&app);
  settings.cdp_port = port;
  settings_store::save(&app, &settings)?;
  state.set_cdp_port(port);

  match codex_launch::ensure_codex_debugging(&app, port).await {
    Ok(msg) => tracing::info!("{msg}"),
    Err(e) => tracing::warn!("ensure Codex on port {port}: {e}"),
  }

  Ok(port)
}

/// Apply theme: write config → restart Codex only if appearance changed → CDP inject skin.
///
/// If the theme is not installed locally, pass `theme_url` so it can be downloaded into
/// `{local_data}/themes` first (recommend catalog flow).
#[tauri::command(rename_all = "snake_case")]
pub async fn apply_theme(
  theme_id: String,
  theme_url: Option<String>,
  app: AppHandle,
  state: State<'_, AppState>,
) -> Result<bool, String> {
  let package =
    theme_catalog::ensure_theme_package_path(&app, &theme_id, theme_url.as_deref()).await?;
  let theme = load_theme_package(&package)?;

  // 1) Write baseTheme appearance into ~/.codex/config.toml
  let apply_result = theme_tool::apply_loaded(
    &theme,
    &theme_tool::codex_config_path(),
    &theme_tool::config_backup_path(&app)?,
  )?;
  tracing::info!(
    "theme-tool apply ok theme={} applied={} config={} backup={} appearance_theme_changed={} config_changed={}",
    apply_result.theme,
    apply_result.applied,
    apply_result.config,
    apply_result.backup,
    apply_result.appearance_changed,
    apply_result.config_changed
  );

  let port = state.cdp_port();

  // 2) Restart ChatGPT only when `appearanceTheme` (light/dark) changed.
  //    Chrome / code theme updates do not require a restart; ensure CDP only.
  // if apply_result.appearance_changed {
  //   match codex_launch::restart_codex_debugging(&app, port).await {
  //     Ok(msg) => tracing::info!("{msg}"),
  //     Err(e) => {
  //       return Err(format!(
  //         "appearanceTheme updated for `{}`, but Codex restart failed (mode may not update until restart): {e}",
  //         theme.id
  //       ));
  //     }
  //   }
  //   // Give the SPA a moment after relaunch before CDP inject.
  //   tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
  // } else {
  match codex_launch::ensure_codex_debugging(&app, port).await {
    Ok(msg) => tracing::info!("{msg}"),
    Err(e) => {
      return Err(format!(
        "could not reach Codex CDP on port {port}: {e}. Open Codex/ChatGPT with remote debugging, then retry."
      ));
    }
  }
  // }

  // 3) Inject live CSS skin via CDP (use already-loaded theme; CodeDrobe applyTheme pattern).
  let opts = inject_options(&state);
  if let Err(e) = injector::apply_loaded_theme(&theme, opts).await {
    tracing::error!("CDP inject failed: {e}");
    return Err(format!(
      "config applied for `{}`, but Codex skin inject failed: {e}",
      theme.id
    ));
  }

  // Record applied theme id for UI state
  settings_store::set_applied_theme_id(&app, Some(theme.id.clone()))?;
  tracing::info!("theme apply complete id={}", theme.id);

  Ok(true)
}

/// Restore: restore config → restart Codex only if appearance changed → remove skin.
#[tauri::command]
pub async fn restore_theme(app: AppHandle, state: State<'_, AppState>) -> Result<bool, String> {
  let restore_result = theme_tool::restore(&app)?;
  tracing::info!(
    "theme-tool restore ok config={} backup={} appearance_changed={}",
    restore_result.config,
    restore_result.backup,
    restore_result.appearance_changed
  );

  let port = state.cdp_port();

  // Restart when restored managed appearance keys differ from current config.
  if restore_result.appearance_changed {
    match codex_launch::restart_codex_debugging(&app, port).await {
      Ok(msg) => tracing::info!("{msg}"),
      Err(e) => {
        return Err(format!(
          "config restored, but Codex restart failed (appearance may not update until restart): {e}"
        ));
      }
    }
  } else {
    match codex_launch::ensure_codex_debugging(&app, port).await {
      Ok(msg) => tracing::info!("{msg}"),
      Err(e) => tracing::warn!("ensure Codex for restore remove: {e}"),
    }
  }

  // Best-effort: strip any leftover injected skin
  let opts = inject_options(&state);
  if let Err(e) = injector::restore_default_theme(opts).await {
    tracing::warn!("CDP remove after restore: {e}");
  }

  // Clear applied theme marker
  settings_store::set_applied_theme_id(&app, None)?;

  Ok(true)
}

/// Download a remote theme package into the user library (`local_data/themes`).
#[tauri::command(rename_all = "snake_case")]
pub async fn download_theme(
  theme_url: String,
  app: AppHandle,
  _state: State<'_, AppState>,
) -> Result<ThemeMetadata, String> {
  theme_catalog::download_theme_to_library(&app, &theme_url).await
}

/// Install a portable multi-app theme package (raw JSON text) into the user themes library.
#[tauri::command(rename_all = "snake_case")]
pub async fn install_theme(
  file_name: String,
  content: String,
  app: AppHandle,
) -> Result<ThemeMetadata, String> {
  // Content is validated by import (JSON deserialize); filename is optional.
  let meta = theme_catalog::import_codex_theme_content(&app, &file_name, &content)?;
  tracing::info!(
    "installed theme id={} name={} location={}",
    meta.id,
    meta.name,
    meta.location
  );
  Ok(meta)
}

/// Delete a user-installed theme package from the local library.
#[tauri::command(rename_all = "snake_case")]
pub async fn delete_theme(theme_id: String, app: AppHandle) -> Result<bool, String> {
  theme_catalog::delete_installed_theme(&app, &theme_id)?;
  tracing::info!("deleted installed theme id={theme_id}");
  Ok(true)
}
