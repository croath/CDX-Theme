use crate::analytics;
use crate::app_state::{AppState, DualCdpStatus};
use crate::codex_launch;
use crate::image_cache;
use crate::injector::{self, APP_CODEX, APP_WORKBUDDY, InjectOptions, load_theme_package};
use crate::settings_store;
use crate::theme_catalog;
use crate::theme_tool;
use cdx_theme_types::ThemeMetadata;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{AppHandle, Emitter, Manager, State};

fn inject_options_for_port(port: u16) -> InjectOptions {
  InjectOptions {
    port,
    // Themes with multi-MB hero/texture need headroom for CDP WS + atob→blob.
    timeout_ms: 120_000,
  }
}

/// Normalize UI / IPC host app id (`codex` default, `workbuddy`).
fn normalize_target_app(target_app: Option<&str>) -> Result<&'static str, String> {
  match target_app
    .unwrap_or(APP_CODEX)
    .trim()
    .to_ascii_lowercase()
    .as_str()
  {
    "" | "codex" | "chatgpt" => Ok(APP_CODEX),
    "workbuddy" | "work-buddy" | "wb" => Ok(APP_WORKBUDDY),
    other => Err(format!(
      "unsupported target app `{other}` (supported: codex, workbuddy)"
    )),
  }
}

fn port_for_app(state: &AppState, app_id: &str) -> u16 {
  if app_id == APP_WORKBUDDY {
    state.workbuddy_cdp_port()
  } else {
    state.cdp_port()
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

/// Current CDP server status for Codex + WorkBuddy (updated by background monitor).
#[tauri::command]
pub async fn cdp_status(state: State<'_, AppState>) -> Result<DualCdpStatus, String> {
  Ok(state.dual_cdp_status())
}

/// Lightweight install detection for Codex / ChatGPT and WorkBuddy (paths only, no CDP).
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostAppsDetect {
  pub codex_installed: bool,
  pub workbuddy_installed: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub codex_path: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub workbuddy_path: Option<String>,
}

/// Per-host applied theme ids for the UI.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppliedThemesDto {
  pub codex: Option<String>,
  pub workbuddy: Option<String>,
}

/// Current applied theme id(s) per host app.
#[tauri::command]
pub async fn get_applied_themes(app: AppHandle) -> Result<AppliedThemesDto, String> {
  let applied = settings_store::applied_themes(&app);
  Ok(AppliedThemesDto {
    codex: applied.codex,
    workbuddy: applied.workbuddy,
  })
}

/// Detect whether Codex / ChatGPT and WorkBuddy desktop apps are installed.
#[tauri::command]
pub async fn detect_host_apps() -> Result<HostAppsDetect, String> {
  tokio::task::spawn_blocking(|| {
    let codex = codex_launch::find_chatgpt_app();
    let workbuddy = codex_launch::find_workbuddy_app();
    HostAppsDetect {
      codex_installed: codex.as_ref().is_some_and(|p| p.is_file() || p.exists()),
      workbuddy_installed: workbuddy
        .as_ref()
        .is_some_and(|p| p.is_file() || p.exists()),
      codex_path: codex.map(|p| p.to_string_lossy().into_owned()),
      workbuddy_path: workbuddy.map(|p| p.to_string_lossy().into_owned()),
    }
  })
  .await
  .map_err(|e| format!("detect host apps task failed: {e}"))
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

/// Persist Codex CDP port and relaunch ChatGPT with the new `--remote-debugging-port` if needed.
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

#[tauri::command]
pub async fn get_workbuddy_cdp_port(state: State<'_, AppState>) -> Result<u16, String> {
  Ok(state.workbuddy_cdp_port())
}

/// Persist WorkBuddy CDP port and relaunch WorkBuddy with remote debugging if needed.
#[tauri::command(rename_all = "snake_case")]
pub async fn set_workbuddy_cdp_port(
  port: u16,
  app: AppHandle,
  state: State<'_, AppState>,
) -> Result<u16, String> {
  if !settings_store::is_valid_port(port) {
    return Err(format!("invalid port {port} (use 1024–65535)"));
  }

  let mut settings = settings_store::load(&app);
  settings.workbuddy_cdp_port = port;
  settings_store::save(&app, &settings)?;
  state.set_workbuddy_cdp_port(port);

  match codex_launch::ensure_workbuddy_debugging(&app, port).await {
    Ok(msg) => tracing::info!("{msg}"),
    Err(e) => tracing::warn!("ensure WorkBuddy on port {port}: {e}"),
  }

  analytics::track_cdp_port_changed(port);
  Ok(port)
}

/// Apply theme to a host app (`target_app`: `codex` default, or `workbuddy`).
///
/// Codex: write config → ensure CDP → inject skin.
/// WorkBuddy: ensure CDP → inject skin only (no Codex config.toml).
///
/// If the theme is not installed locally, pass `theme_url` so it can be downloaded into
/// `{local_data}/themes` first (recommend catalog flow).
#[tauri::command(rename_all = "snake_case")]
pub async fn apply_theme(
  theme_id: String,
  theme_url: Option<String>,
  target_app: Option<String>,
  app: AppHandle,
  state: State<'_, AppState>,
) -> Result<bool, String> {
  let host = normalize_target_app(target_app.as_deref())?;
  let package =
    theme_catalog::ensure_theme_package_path(&app, &theme_id, theme_url.as_deref()).await?;
  let theme = load_theme_package(&package)?;
  let port = port_for_app(&state, host);

  if host == APP_CODEX {
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

    match codex_launch::ensure_codex_debugging(&app, port).await {
      Ok(msg) => tracing::info!("{msg}"),
      Err(e) => {
        return Err(format!(
          "could not reach Codex CDP on port {port}: {e}. Open Codex/ChatGPT with remote debugging, then retry."
        ));
      }
    }

    let opts = inject_options_for_port(port);
    if let Err(e) = injector::apply_loaded_theme_for_app(APP_CODEX, &theme, opts).await {
      tracing::error!("CDP inject failed: {e}");
      return Err(format!(
        "config applied for `{}`, but Codex skin inject failed: {e}",
        theme.id
      ));
    }
  } else {
    // WorkBuddy: CDP inject only (no Codex appearance config).
    match codex_launch::ensure_workbuddy_debugging(&app, port).await {
      Ok(msg) => tracing::info!("{msg}"),
      Err(e) => {
        return Err(format!(
          "could not reach WorkBuddy CDP on port {port}: {e}. Open WorkBuddy with remote debugging, then retry."
        ));
      }
    }

    let opts = inject_options_for_port(port);
    if let Err(e) = injector::apply_loaded_theme_for_app(APP_WORKBUDDY, &theme, opts).await {
      tracing::error!("WorkBuddy CDP inject failed: {e}");
      return Err(format!(
        "WorkBuddy skin inject failed for `{}`: {e}",
        theme.id
      ));
    }
  }

  // Record applied theme for this host (used by UI + auto-reapply monitor).
  settings_store::set_applied_theme(&app, host, Some(theme.id.clone()))?;
  tracing::info!("theme apply complete id={} host={host}", theme.id);
  analytics::track_theme_applied(&theme.id, theme_url.is_some(), true);

  Ok(true)
}

/// Restore default skin for a host app (`target_app`: `codex` default, or `workbuddy`).
///
/// Codex: restore `config.toml` appearance → ensure CDP → remove inject.
/// WorkBuddy: ensure CDP → remove inject only (no Codex config).
#[tauri::command(rename_all = "snake_case")]
pub async fn restore_theme(
  target_app: Option<String>,
  app: AppHandle,
  state: State<'_, AppState>,
) -> Result<bool, String> {
  let host = normalize_target_app(target_app.as_deref())?;
  let port = port_for_app(&state, host);

  if host == APP_CODEX {
    let restore_result = theme_tool::restore(&app)?;
    tracing::info!(
      "theme-tool restore ok config={} backup={} appearance_changed={}",
      restore_result.config,
      restore_result.backup,
      restore_result.appearance_changed
    );

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

    let opts = inject_options_for_port(port);
    if let Err(e) = injector::restore_default_theme_for_app(APP_CODEX, opts).await {
      tracing::warn!("Codex CDP remove after restore: {e}");
    }
  } else {
    match codex_launch::ensure_workbuddy_debugging(&app, port).await {
      Ok(msg) => tracing::info!("{msg}"),
      Err(e) => {
        return Err(format!(
          "could not reach WorkBuddy CDP on port {port}: {e}. Open WorkBuddy with remote debugging, then retry."
        ));
      }
    }

    let opts = inject_options_for_port(port);
    if let Err(e) = injector::restore_default_theme_for_app(APP_WORKBUDDY, opts).await {
      return Err(format!("WorkBuddy skin restore failed: {e}"));
    }
  }

  // Clear applied marker for this host only (stops auto-reapply).
  settings_store::set_applied_theme(&app, host, None)?;
  tracing::info!("theme restore complete host={host}");
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

/// Full analytics snapshot (enabled flag + anonymous distinct_id + build-time configured).
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
  let bun_path = crate::theme_builder_store::resolve_bundled_bun(&app);
  let codex = cdx_theme_core::list_codex_sessions_async_with(Some(200), bun_path).await?;
  crate::theme_builder_store::list_intersection(&app, codex, limit)
}

/// Theme Builder: list Codex models from local cache (`~/.codex/models_cache.json`).
#[tauri::command]
pub async fn list_codex_models() -> Result<cdx_theme_core::CodexModelsList, String> {
  tokio::task::spawn_blocking(cdx_theme_core::list_codex_models)
    .await
    .map_err(|e| format!("list_codex_models task failed: {e}"))
}

/// Theme Builder: delete a tracked session (app data registry + workspace folder).
#[tauri::command(rename_all = "snake_case")]
pub async fn delete_theme_builder_session(
  session_id: String,
  app: AppHandle,
) -> Result<bool, String> {
  let session_id = session_id.trim().to_string();
  if session_id.is_empty() {
    return Err("session id is empty".into());
  }
  tokio::task::spawn_blocking(move || {
    crate::theme_builder_store::delete_session(&app, &session_id)
  })
  .await
  .map_err(|e| format!("delete session task failed: {e}"))??;
  Ok(true)
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
// IPC surface keeps discrete args (matches app-ui api.rs); grouping would break invoke shapes.
#[allow(clippy::too_many_arguments)]
pub async fn codex_chat(
  prompt: String,
  session_id: Option<String>,
  workspace_path: Option<String>,
  workspace_id: Option<String>,
  wait_ms: Option<u64>,
  model: Option<String>,
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

  if let Some(ref id) = resume_id
    && !crate::theme_builder_store::is_tracked(&app, id)
  {
    return Err(
      "session is not a Theme Builder session (not found in app data). Start a theme build first."
        .into(),
    );
  }

  // Resolve ACP cwd: explicit arg → stored workspace for resume → error for new chat.
  let mut cwd_path = workspace_path
    .as_deref()
    .map(str::trim)
    .filter(|s| !s.is_empty())
    .map(std::path::PathBuf::from);
  if cwd_path.is_none()
    && let Some(ref id) = resume_id
  {
    cwd_path =
      crate::theme_builder_store::workspace_path_for(&app, id).map(std::path::PathBuf::from);
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

  // App-bundled CLI for skill pack/apply/probe + bundled Bun for ACP.
  let cdxthemex = crate::theme_builder_store::resolve_cdxthemex(&app)?;
  let cli_dir = cdxthemex
    .parent()
    .map(|p| p.to_path_buf())
    .ok_or_else(|| "cdxthemex parent dir missing".to_string())?;
  let bun_path = crate::theme_builder_store::resolve_bundled_bun(&app);
  let mut path_prepend = vec![cli_dir];
  if let Some(ref bun) = bun_path
    && let Some(dir) = bun.parent()
  {
    let dir = dir.to_path_buf();
    if !path_prepend.iter().any(|d| d == &dir) {
      path_prepend.push(dir);
    }
  }

  // First turn: wrap with skill bootstrap so Codex uses the bundled skill + CLI.
  // Follow-ups: keep messages short — only a brief summary in the chat reply.
  let wire = if resume_id.is_none() {
    crate::theme_builder_store::skill_bootstrap_prompt(
      &prompt,
      &cwd.display().to_string(),
      &cdxthemex,
    )
  } else {
    format!(
      "{prompt}\n\n\
       [Reply style] User-facing UI: reply with a short plain-text summary only \
       (2–5 lines). No code, no CSS, no actions/tool lists, no file paths."
    )
  };

  let wid = workspace_id.filter(|s| !s.trim().is_empty()).or_else(|| {
    cwd
      .file_name()
      .and_then(|n| n.to_str())
      .map(|s| s.to_string())
  });

  let model = model
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty());

  tracing::info!(
    chars = wire.len(),
    wait_ms = wait_ms.unwrap_or(180_000),
    session = resume_id.as_deref().unwrap_or(""),
    model = model.as_deref().unwrap_or(""),
    cwd = %cwd.display(),
    cdxthemex = %cdxthemex.display(),
    "theme builder → ACP session/prompt"
  );

  // Live ACP transcript → frontend via Tauri event.
  let app_stream = app.clone();
  let on_stream: cdx_theme_core::CodexStreamCallback = std::sync::Arc::new(move |text: String| {
    let payload = serde_json::json!({
      "text": text,
      "done": false,
    });
    let _ = app_stream.emit("theme-builder-acp-stream", payload);
  });

  let mut result = cdx_theme_core::codex_chat_send_and_wait_with(
    &wire,
    cdx_theme_core::CodexChatOptions {
      session_id: resume_id.clone(),
      cwd: Some(cwd.clone()),
      wait_ms,
      path_prepend,
      extra_env: vec![
        ("CDXTHEME".into(), cdxthemex.to_string_lossy().into_owned()),
        ("CDXTHEMEX".into(), cdxthemex.to_string_lossy().into_owned()),
      ],
      on_stream: Some(on_stream),
      model,
      bun_path,
    },
  )
  .await?;

  // Final stream tick so the UI can mark the turn complete if needed.
  let _ = app.emit(
    "theme-builder-acp-stream",
    serde_json::json!({
      "text": result.reply,
      "done": true,
    }),
  );

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
  if result.submitted
    && let Some(pkg) = crate::theme_builder_store::find_newest_theme_package(&cwd)
  {
    tracing::info!(
      package = %pkg.display(),
      "theme builder package ready for manual apply"
    );
    result.package_path = Some(pkg.to_string_lossy().into_owned());
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
/// themes library (`app_data_dir/themes`), then apply it to the selected host app.
#[tauri::command(rename_all = "snake_case")]
pub async fn apply_built_theme(
  workspace_path: String,
  package_path: Option<String>,
  target_app: Option<String>,
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

  let host = normalize_target_app(target_app.as_deref())?;
  tracing::info!(
    package = %pkg.display(),
    workspace = %cwd.display(),
    target_app = host,
    "theme builder apply_built_theme — install + apply"
  );

  let meta = theme_catalog::install_theme_package_file(&app, &pkg)?;
  apply_theme(meta.id.clone(), None, Some(host.to_string()), app, state).await?;

  Ok(ApplyBuiltThemeResult {
    theme_id: meta.id,
    theme_name: meta.name,
    package_path: pkg.to_string_lossy().into_owned(),
    applied: true,
  })
}

/// Save a user-uploaded hero image into a Theme Builder workspace (`theme/assets/hero.*`).
#[tauri::command(rename_all = "snake_case")]
pub async fn save_theme_builder_hero(
  workspace_path: String,
  file_name: String,
  content_base64: String,
) -> Result<crate::theme_builder_store::SavedHeroImage, String> {
  let workspace_path = workspace_path.trim().to_string();
  if workspace_path.is_empty() {
    return Err("workspace_path is empty".into());
  }
  let cwd = std::path::PathBuf::from(&workspace_path);
  let file_name = file_name.trim().to_string();
  if file_name.is_empty() {
    return Err("file_name is empty".into());
  }
  tokio::task::spawn_blocking(move || {
    crate::theme_builder_store::save_hero_image(&cwd, &file_name, &content_base64)
  })
  .await
  .map_err(|e| format!("save hero task failed: {e}"))?
}
