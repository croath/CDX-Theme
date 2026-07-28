use crate::analytics;
use crate::app_state::{AppState, CdpServerStatus};
use crate::codex_launch;
use crate::image_cache;
use crate::injector::{self, InjectOptions, load_theme_package};
use crate::settings_store;
use crate::theme_catalog;
use crate::theme_tool;
use cdx_theme_types::ThemeMetadata;
use serde_json::Value;
use std::collections::HashMap;
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

  analytics::track_cdp_port_changed(port);
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

  // 3) Inject live CSS skin via CDP (use already-loaded theme).
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
  analytics::track_theme_applied(&theme.id, theme_url.is_some(), true);

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
  analytics::track_theme_restored(true);

  Ok(true)
}

/// Download a remote theme package into the user library (`local_data/themes`).
#[tauri::command(rename_all = "snake_case")]
pub async fn download_theme(
  theme_url: String,
  app: AppHandle,
  _state: State<'_, AppState>,
) -> Result<ThemeMetadata, String> {
  match theme_catalog::download_theme_to_library(&app, &theme_url).await {
    Ok(meta) => {
      analytics::track_theme_downloaded(Some(&meta.id), true);
      Ok(meta)
    }
    Err(e) => {
      analytics::track_theme_downloaded(None, false);
      Err(e)
    }
  }
}

/// Install a portable multi-app theme package (raw JSON text) into the user themes library.
#[tauri::command(rename_all = "snake_case")]
pub async fn install_theme(
  file_name: String,
  content: String,
  app: AppHandle,
) -> Result<ThemeMetadata, String> {
  // Content is validated by import (JSON deserialize); filename is optional.
  match theme_catalog::import_codex_theme_content(&app, &file_name, &content) {
    Ok(meta) => {
      tracing::info!(
        "installed theme id={} name={} location={}",
        meta.id,
        meta.name,
        meta.location
      );
      analytics::track_theme_installed(&meta.id, true);
      Ok(meta)
    }
    Err(e) => {
      analytics::track_theme_installed("unknown", false);
      Err(e)
    }
  }
}

/// Delete a user-installed theme package from the local library.
#[tauri::command(rename_all = "snake_case")]
pub async fn delete_theme(theme_id: String, app: AppHandle) -> Result<bool, String> {
  match theme_catalog::delete_installed_theme(&app, &theme_id) {
    Ok(()) => {
      tracing::info!("deleted installed theme id={theme_id}");
      analytics::track_theme_deleted(&theme_id, true);
      Ok(true)
    }
    Err(e) => {
      analytics::track_theme_deleted(&theme_id, false);
      Err(e)
    }
  }
}

/// Whether anonymous product analytics is enabled for this install.
#[tauri::command]
pub async fn get_analytics_enabled(app: AppHandle) -> Result<bool, String> {
  Ok(settings_store::load(&app).analytics_enabled)
}

/// Full analytics snapshot (opt-in + anonymous distinct_id + build-time configured).
#[tauri::command]
pub async fn get_analytics_state(app: AppHandle) -> Result<analytics::AnalyticsState, String> {
  Ok(analytics::Analytics::state(&app))
}

/// Persist analytics preference and update the in-process gate immediately.
#[tauri::command(rename_all = "snake_case")]
pub async fn set_analytics_enabled(enabled: bool, app: AppHandle) -> Result<bool, String> {
  analytics::Analytics::set_enabled(&app, enabled)?;
  Ok(enabled)
}

/// Generic UI event capture (e.g. page views). Properties must be JSON-serializable scalars/objects.
#[tauri::command(rename_all = "snake_case")]
pub async fn track_event(
  name: String,
  properties: Option<HashMap<String, Value>>,
) -> Result<(), String> {
  let name = name.trim();
  if name.is_empty() || name.len() > 100 {
    return Err("invalid event name".into());
  }
  // Allow only product-safe event names from the UI surface.
  let allowed = matches!(name, "page_viewed" | "ui_theme_toggled" | "locale_changed");
  if !allowed {
    return Err(format!("event `{name}` is not allowed from the UI"));
  }
  let props = properties.unwrap_or_default();
  if name == "page_viewed" {
    let page = props
      .get("page")
      .and_then(|v| v.as_str())
      .unwrap_or("unknown");
    analytics::track_page_viewed(page);
  } else {
    analytics::capture(name, props);
  }
  Ok(())
}

/// Create `{app_data_dir}/theme_builder/{random_id}` with bundled skill + theme scaffold.
#[tauri::command]
pub async fn start_theme_build(
  app: AppHandle,
) -> Result<crate::theme_builder_store::PreparedWorkspace, String> {
  tokio::task::spawn_blocking(move || crate::theme_builder_store::prepare_workspace(&app))
    .await
    .map_err(|e| format!("start_theme_build task failed: {e}"))?
}

/// Theme Builder: list sessions that exist in **both** app data and Codex history.
#[tauri::command(rename_all = "snake_case")]
pub async fn list_codex_sessions(
  limit: Option<usize>,
  app: AppHandle,
) -> Result<Vec<cdx_theme_core::CodexSessionSummary>, String> {
  let codex = cdx_theme_core::list_codex_sessions_async(Some(200)).await?;
  crate::theme_builder_store::list_intersection(&app, codex, limit)
}

/// Theme Builder: load a Codex session transcript for the chat view.
/// Only sessions tracked in app data (and still present in Codex) are allowed.
#[tauri::command(rename_all = "snake_case")]
pub async fn get_codex_session(
  session_id: String,
  app: AppHandle,
) -> Result<cdx_theme_core::CodexSessionDetail, String> {
  let session_id = session_id.trim().to_string();
  if session_id.is_empty() {
    return Err("session id is empty".into());
  }
  if !crate::theme_builder_store::is_tracked(&app, &session_id) {
    return Err(
      "session is not a Theme Builder session (not found in app data). Start a theme build first."
        .into(),
    );
  }
  let workspace_path = crate::theme_builder_store::workspace_path_for(&app, &session_id);
  let mut detail = tokio::task::spawn_blocking({
    let session_id = session_id.clone();
    move || cdx_theme_core::load_codex_session(&session_id)
  })
  .await
  .map_err(|e| format!("load session task failed: {e}"))??;
  detail.workspace_path = workspace_path;
  Ok(detail)
}

/// Theme Builder: send a prompt to Codex over ACP (`codex-acp` / official adapter).
///
/// - `workspace_path`: absolute Theme Builder workspace (skill + theme-dir); required for new chats
/// - `session_id`: resume via `session/load` (cwd taken from stored workspace when omitted)
/// - On success, auto-saves under `{app_data_dir}/theme_builder/sessions.json`
#[tauri::command(rename_all = "snake_case")]
pub async fn codex_chat(
  prompt: String,
  session_id: Option<String>,
  workspace_path: Option<String>,
  workspace_id: Option<String>,
  wait_ms: Option<u64>,
  app: AppHandle,
  _state: State<'_, AppState>,
) -> Result<cdx_theme_core::CodexChatResult, String> {
  let prompt = prompt.trim().to_string();
  if prompt.is_empty() {
    return Err("prompt is empty".into());
  }
  let resume_id = session_id
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty());

  if let Some(ref id) = resume_id {
    if !crate::theme_builder_store::is_tracked(&app, id) {
      return Err(
        "session is not a Theme Builder session (not found in app data). Start a theme build first."
          .into(),
      );
    }
  }

  // Resolve ACP cwd: explicit arg → stored workspace for resume → error for new chat.
  let mut cwd_path = workspace_path
    .as_deref()
    .map(str::trim)
    .filter(|s| !s.is_empty())
    .map(std::path::PathBuf::from);
  if cwd_path.is_none() {
    if let Some(ref id) = resume_id {
      cwd_path =
        crate::theme_builder_store::workspace_path_for(&app, id).map(std::path::PathBuf::from);
    }
  }
  let Some(cwd) = cwd_path else {
    return Err(
      "workspace_path is required for a new Theme Builder chat (call start_theme_build first)"
        .into(),
    );
  };
  if !cwd.is_absolute() {
    return Err(format!(
      "workspace_path must be absolute: {}",
      cwd.display()
    ));
  }

  // App-bundled CLI for skill pack/apply/probe.
  let cdxthemex = crate::theme_builder_store::resolve_cdxthemex(&app)?;
  let cli_dir = cdxthemex
    .parent()
    .map(|p| p.to_path_buf())
    .ok_or_else(|| "cdxthemex parent dir missing".to_string())?;

  // First turn: wrap with skill bootstrap so Codex uses the bundled skill + CLI.
  let wire = if resume_id.is_none() {
    crate::theme_builder_store::skill_bootstrap_prompt(
      &prompt,
      &cwd.display().to_string(),
      &cdxthemex,
    )
  } else {
    prompt.clone()
  };

  let wid = workspace_id.filter(|s| !s.trim().is_empty()).or_else(|| {
    cwd
      .file_name()
      .and_then(|n| n.to_str())
      .map(|s| s.to_string())
  });

  tracing::info!(
    chars = wire.len(),
    wait_ms = wait_ms.unwrap_or(180_000),
    session = resume_id.as_deref().unwrap_or(""),
    cwd = %cwd.display(),
    cdxthemex = %cdxthemex.display(),
    "theme builder → ACP session/prompt"
  );

  let mut result = cdx_theme_core::codex_chat_send_and_wait_with(
    &wire,
    cdx_theme_core::CodexChatOptions {
      session_id: resume_id.clone(),
      cwd: Some(cwd.clone()),
      wait_ms,
      path_prepend: vec![cli_dir],
      extra_env: vec![
        ("CDXTHEME".into(), cdxthemex.to_string_lossy().into_owned()),
        ("CDXTHEMEX".into(), cdxthemex.to_string_lossy().into_owned()),
      ],
    },
  )
  .await?;

  if let Some(ref sid) = result.session_id {
    let title = crate::theme_builder_store::title_from_prompt(&prompt);
    if let Err(e) = crate::theme_builder_store::record_session(
      &app,
      sid,
      Some(title.as_str()),
      wid.as_deref(),
      Some(cwd.to_string_lossy().as_ref()),
    ) {
      tracing::warn!("theme builder session save failed: {e}");
    } else {
      tracing::info!(session = %sid, "theme builder session saved to app data");
    }
  }

  // Detect packed theme for the UI Apply button (do not install/apply here).
  if result.submitted {
    if let Some(pkg) = crate::theme_builder_store::find_newest_theme_package(&cwd) {
      tracing::info!(
        package = %pkg.display(),
        "theme builder package ready for manual apply"
      );
      result.package_path = Some(pkg.to_string_lossy().into_owned());
    }
  }

  Ok(result)
}

/// Result of installing + applying a Theme Builder package.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyBuiltThemeResult {
  pub theme_id: String,
  pub theme_name: String,
  pub package_path: String,
  pub applied: bool,
}

/// Install the newest `.cdxtheme` from a Theme Builder workspace into the user
/// themes library (`app_data_dir/themes`), then apply it to Codex.
#[tauri::command(rename_all = "snake_case")]
pub async fn apply_built_theme(
  workspace_path: String,
  package_path: Option<String>,
  app: AppHandle,
  state: State<'_, AppState>,
) -> Result<ApplyBuiltThemeResult, String> {
  let workspace_path = workspace_path.trim().to_string();
  if workspace_path.is_empty() {
    return Err("workspace_path is empty".into());
  }
  let cwd = std::path::PathBuf::from(&workspace_path);
  if !cwd.is_absolute() {
    return Err(format!("workspace_path must be absolute: {workspace_path}"));
  }
  if !cwd.is_dir() {
    return Err(format!("workspace not found: {workspace_path}"));
  }

  let pkg = if let Some(p) = package_path
    .as_deref()
    .map(str::trim)
    .filter(|s| !s.is_empty())
  {
    let path = std::path::PathBuf::from(p);
    if path.is_file() {
      path
    } else {
      return Err(format!("theme package not found: {p}"));
    }
  } else {
    crate::theme_builder_store::find_newest_theme_package(&cwd).ok_or_else(|| {
      format!(
        "no .cdxtheme package in workspace — generate first (expected under output/): {}",
        cwd.display()
      )
    })?
  };

  tracing::info!(
    package = %pkg.display(),
    workspace = %cwd.display(),
    "theme builder apply_built_theme — install + apply"
  );

  let meta = theme_catalog::install_theme_package_file(&app, &pkg)?;
  apply_theme(meta.id.clone(), None, app, state).await?;

  Ok(ApplyBuiltThemeResult {
    theme_id: meta.id,
    theme_name: meta.name,
    package_path: pkg.to_string_lossy().into_owned(),
    applied: true,
  })
}
