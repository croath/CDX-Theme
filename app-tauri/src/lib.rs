pub mod analytics;
pub mod app_state;
pub mod cdp_monitor;
pub mod codex_launch;
pub mod commands;
pub mod image_cache;
pub mod injector;
pub mod paths;
pub mod settings_store;
pub mod theme_builder_store;
pub mod theme_catalog;
pub mod theme_lib;
pub mod theme_package;
pub mod theme_tool;
pub mod types;

use app_state::AppState;
use serde::Serialize;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};
use std::time::Duration;
use tauri::menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Emitter, Manager, RunEvent};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_updater::{Update, UpdaterExt};

/// How often to poll the remote updater endpoint while the app is running.
const UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(60 * 60);

/// Menu item id for manual update checks (app / Help menu).
const MENU_CHECK_UPDATES: &str = "check_for_updates";

/// Webview event for the left-nav update notification card.
const APP_UPDATE_EVENT: &str = "app-update";

/// Shared updater flags for the hourly loop, menu action, and UI commands.
struct UpdaterState {
  /// True after package is downloaded and ready to install (needs restart after install).
  update_ready: AtomicBool,
  /// Guards concurrent automatic/manual checks.
  checking: AtomicBool,
  /// Guards concurrent downloads.
  downloading: AtomicBool,
  /// Guards concurrent install.
  installing: AtomicBool,
  /// Version string for the pending / ready update.
  staged_version: std::sync::Mutex<Option<String>>,
  /// Current version string paired with the pending update.
  current_version: std::sync::Mutex<Option<String>>,
  /// Optional release notes for the UI card.
  release_notes: std::sync::Mutex<Option<String>>,
  /// Version the user chose "Later" for (session-only; re-prompt on manual check).
  deferred_version: std::sync::Mutex<Option<String>>,
  /// Pending update object from `updater.check()` (needed for download/install).
  pending_update: std::sync::Mutex<Option<Update>>,
  /// Downloaded package bytes (after download, before install).
  downloaded_bytes: std::sync::Mutex<Option<Vec<u8>>>,
  /// Last known download progress (for late-joining UI).
  last_progress: std::sync::Mutex<UpdateProgressSnapshot>,
}

#[derive(Clone, Debug, Default)]
struct UpdateProgressSnapshot {
  downloaded: u64,
  total: Option<u64>,
  percent: Option<u8>,
}

impl UpdaterState {
  fn new() -> Self {
    Self {
      update_ready: AtomicBool::new(false),
      checking: AtomicBool::new(false),
      downloading: AtomicBool::new(false),
      installing: AtomicBool::new(false),
      staged_version: std::sync::Mutex::new(None),
      current_version: std::sync::Mutex::new(None),
      release_notes: std::sync::Mutex::new(None),
      deferred_version: std::sync::Mutex::new(None),
      pending_update: std::sync::Mutex::new(None),
      downloaded_bytes: std::sync::Mutex::new(None),
      last_progress: std::sync::Mutex::new(UpdateProgressSnapshot::default()),
    }
  }

  fn staged_version(&self) -> Option<String> {
    self.staged_version.lock().ok().and_then(|g| g.clone())
  }

  fn current_version(&self) -> Option<String> {
    self.current_version.lock().ok().and_then(|g| g.clone())
  }

  fn release_notes(&self) -> Option<String> {
    self.release_notes.lock().ok().and_then(|g| g.clone())
  }

  fn mark_deferred(&self, version: &str) {
    if let Ok(mut guard) = self.deferred_version.lock() {
      *guard = Some(version.to_string());
    }
  }

  fn is_deferred(&self, version: &str) -> bool {
    self
      .deferred_version
      .lock()
      .ok()
      .and_then(|g| g.clone())
      .is_some_and(|v| v == version)
  }

  fn clear_deferred(&self, version: &str) {
    if let Ok(mut guard) = self.deferred_version.lock()
      && guard.as_deref() == Some(version) {
        *guard = None;
      }
  }

  fn set_pending(&self, update: Update) {
    if let Ok(mut guard) = self.staged_version.lock() {
      *guard = Some(update.version.clone());
    }
    if let Ok(mut guard) = self.current_version.lock() {
      *guard = Some(update.current_version.clone());
    }
    if let Ok(mut guard) = self.release_notes.lock() {
      *guard = update.body.clone().filter(|s| !s.trim().is_empty());
    }
    if let Ok(mut guard) = self.downloaded_bytes.lock() {
      *guard = None;
    }
    if let Ok(mut guard) = self.last_progress.lock() {
      *guard = UpdateProgressSnapshot::default();
    }
    self.update_ready.store(false, Ordering::Relaxed);
    if let Ok(mut guard) = self.pending_update.lock() {
      *guard = Some(update);
    }
  }

  fn mark_downloaded(&self, bytes: Vec<u8>) {
    if let Ok(mut guard) = self.downloaded_bytes.lock() {
      *guard = Some(bytes);
    }
    if let Ok(mut guard) = self.last_progress.lock() {
      guard.percent = Some(100);
    }
    // Downloaded packages cannot be deferred away — always show install.
    if let Some(version) = self.staged_version() {
      self.clear_deferred(&version);
    }
    self.update_ready.store(true, Ordering::Relaxed);
  }

  fn take_download_pair(&self) -> Option<(Update, Vec<u8>)> {
    let update = self.pending_update.lock().ok().and_then(|g| g.clone())?;
    let bytes = self
      .downloaded_bytes
      .lock()
      .ok()
      .and_then(|mut g| g.take())?;
    Some((update, bytes))
  }

  fn snapshot_status(&self) -> AppUpdatePayload {
    let version = self.staged_version().unwrap_or_default();
    let current_version = self.current_version().unwrap_or_default();
    let body = self.release_notes();
    let progress = self
      .last_progress
      .lock()
      .ok()
      .map(|g| g.clone())
      .unwrap_or_default();

    if self.installing.load(Ordering::Relaxed) {
      return AppUpdatePayload::installing(current_version, version, body);
    }
    if self.update_ready.load(Ordering::Relaxed) {
      return AppUpdatePayload::ready(current_version, version, body);
    }
    if self.downloading.load(Ordering::Relaxed) {
      return AppUpdatePayload::downloading(
        current_version,
        version,
        body,
        progress.downloaded,
        progress.total,
        progress.percent,
      );
    }
    if self.pending_update.lock().ok().is_some_and(|g| g.is_some()) {
      return AppUpdatePayload::available(current_version, version, body);
    }
    AppUpdatePayload::idle()
  }
}

/// Payload pushed to the webview for the sidebar update card.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdatePayload {
  /// `idle` | `available` | `downloading` | `ready` | `installing` | `error`
  pub phase: String,
  pub current_version: String,
  pub version: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub body: Option<String>,
  pub downloaded: u64,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub total: Option<u64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub percent: Option<u8>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error: Option<String>,
}

impl AppUpdatePayload {
  fn idle() -> Self {
    Self {
      phase: "idle".into(),
      current_version: String::new(),
      version: String::new(),
      body: None,
      downloaded: 0,
      total: None,
      percent: None,
      error: None,
    }
  }

  fn available(current: String, version: String, body: Option<String>) -> Self {
    Self {
      phase: "available".into(),
      current_version: current,
      version,
      body,
      downloaded: 0,
      total: None,
      percent: None,
      error: None,
    }
  }

  fn downloading(
    current: String,
    version: String,
    body: Option<String>,
    downloaded: u64,
    total: Option<u64>,
    percent: Option<u8>,
  ) -> Self {
    Self {
      phase: "downloading".into(),
      current_version: current,
      version,
      body,
      downloaded,
      total,
      percent,
      error: None,
    }
  }

  fn ready(current: String, version: String, body: Option<String>) -> Self {
    Self {
      phase: "ready".into(),
      current_version: current,
      version,
      body,
      downloaded: 0,
      total: None,
      percent: Some(100),
      error: None,
    }
  }

  fn installing(current: String, version: String, body: Option<String>) -> Self {
    Self {
      phase: "installing".into(),
      current_version: current,
      version,
      body,
      downloaded: 0,
      total: None,
      percent: Some(100),
      error: None,
    }
  }

  fn error_with(
    current: String,
    version: String,
    body: Option<String>,
    error: String,
    ready: bool,
  ) -> Self {
    Self {
      phase: if ready {
        "ready".into()
      } else {
        "error".into()
      },
      current_version: current,
      version,
      body,
      downloaded: 0,
      total: None,
      percent: if ready { Some(100) } else { None },
      error: Some(error),
    }
  }
}

fn emit_app_update(app: &AppHandle, payload: &AppUpdatePayload) {
  if let Err(e) = app.emit(APP_UPDATE_EVENT, payload) {
    tracing::warn!(error = %e, "updater: failed to emit app-update event");
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UpdateCheckSource {
  /// Hourly background poll — quiet when already up to date.
  Automatic,
  /// System menu — always surface a dialog result.
  Manual,
}

/// Build a `RUST_LOG` filter (same directive syntax as cargo / env_logger).
///
/// Examples: `info`, `debug`, `cdx_theme_app=debug,warn`
/// Default when unset: `info`.
fn rust_log_filter() -> env_filter::Filter {
  let mut builder = env_filter::Builder::new();
  match std::env::var("RUST_LOG") {
    Ok(spec) if !spec.trim().is_empty() => {
      builder.parse(&spec);
    }
    _ => {
      builder.filter_level(log::LevelFilter::Info);
    }
  }
  builder.build()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  let env_filter = Arc::new(rust_log_filter());
  let max_level = env_filter.filter();
  let rust_log_spec = std::env::var("RUST_LOG").unwrap_or_else(|_| "info (default)".into());

  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_updater::Builder::new().build())
    .plugin({
      let env_filter = env_filter.clone();
      tauri_plugin_log::Builder::new()
        // Accept everything up to the max directive; fine-grained enable is below.
        .level(max_level)
        .filter(move |metadata| env_filter.enabled(metadata))
        .targets([
          Target::new(TargetKind::Stdout),
          Target::new(TargetKind::LogDir {
            file_name: Some("cdxtheme".into()),
          }),
          Target::new(TargetKind::Webview),
        ])
        .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
        .build()
    })
    .setup(move |app| {
      // No macOS private API / no transparent window.
      // Overlay titlebar + solid window background; the web UI paints under the traffic lights.
      if let Some(window) = app.get_webview_window("main") {
        // Match light shell background (oklch ~0.985 green-tinted white → RGB approx).
        // Dark mode is painted by the web content full-bleed under the overlay chrome.
        let _ = window.set_background_color(Some(tauri::window::Color(248, 250, 246, 255)));

        // Open Web Inspector automatically in `tauri dev` / debug builds.
        #[cfg(debug_assertions)]
        {
          window.open_devtools();
          tracing::debug!("webview DevTools opened (debug build)");
        }
      }

      tracing::info!(
        rust_log = %rust_log_spec,
        max_level = ?max_level,
        debug_assertions = cfg!(debug_assertions),
        "CDXTheme starting"
      );

      let settings = settings_store::load(app.handle());
      let port = settings.cdp_port;
      app.manage(AppState::new(port));
      app.manage(UpdaterState::new());
      tracing::debug!("CDP port from settings: {port}");

      // Ensure user theme drop-in folder exists: {local_data}/themes
      if let Err(e) = theme_catalog::ensure_user_themes_dir(app.handle()) {
        tracing::warn!("user themes dir: {e}");
      }

      if let Err(e) = setup_app_menu(app.handle()) {
        tracing::warn!("app menu setup failed: {e}");
      }

      // Background: analytics init, then updates (hourly), then CDP monitor (do not auto-launch ChatGPT).
      let handle = app.handle().clone();
      tauri::async_runtime::spawn(async move {
        analytics::Analytics::init(&handle).await;
        start_updater_loop(handle.clone());
        cdp_monitor::start(handle);
      });

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      commands::retrieve_local_theme_list,
      commands::fetch_remote_theme_catalog,
      commands::resolve_cached_image,
      commands::cdp_status,
      commands::set_window_appearance,
      commands::get_cdp_port,
      commands::set_cdp_port,
      commands::apply_theme,
      commands::restore_theme,
      commands::download_theme,
      commands::install_theme,
      commands::delete_theme,
      commands::get_analytics_enabled,
      commands::get_analytics_state,
      commands::set_analytics_enabled,
      commands::track_event,
      commands::codex_chat,
      commands::apply_built_theme,
      commands::save_theme_builder_hero,
      commands::list_codex_sessions,
      commands::list_codex_models,
      commands::delete_theme_builder_session,
      commands::get_codex_session,
      commands::start_theme_build,
      commands::check_theme_builder_runtime,
      commands::install_bun_for_theme_builder,
      get_app_update_status,
      download_app_update,
      install_app_update,
      dismiss_app_update,
    ])
    .build(tauri::generate_context!())
    .expect("error while building tauri application")
    .run(|_app, event| {
      if matches!(event, RunEvent::Exit) {
        // Best-effort flush so buffered events are not lost on quit.
        tauri::async_runtime::block_on(analytics::Analytics::shutdown());
      }
    });
}

/// Build the native app menu (macOS menu bar / Windows window menu) with Check for Updates.
fn setup_app_menu(app: &AppHandle) -> tauri::Result<()> {
  let pkg_info = app.package_info();
  let config = app.config();
  let about_metadata = AboutMetadata {
    name: Some(pkg_info.name.clone()),
    version: Some(pkg_info.version.to_string()),
    copyright: config.bundle.copyright.clone(),
    authors: config.bundle.publisher.clone().map(|p| vec![p]),
    ..Default::default()
  };

  let check_updates = MenuItem::with_id(
    app,
    MENU_CHECK_UPDATES,
    "Check for Updates…",
    true,
    None::<&str>,
  )?;

  #[cfg(target_os = "macos")]
  let app_submenu = Submenu::with_items(
    app,
    pkg_info.name.clone(),
    true,
    &[
      &PredefinedMenuItem::about(app, None, Some(about_metadata.clone()))?,
      &PredefinedMenuItem::separator(app)?,
      &check_updates,
      &PredefinedMenuItem::separator(app)?,
      &PredefinedMenuItem::services(app, None)?,
      &PredefinedMenuItem::separator(app)?,
      &PredefinedMenuItem::hide(app, None)?,
      &PredefinedMenuItem::hide_others(app, None)?,
      &PredefinedMenuItem::separator(app)?,
      &PredefinedMenuItem::quit(app, None)?,
    ],
  )?;

  #[cfg(not(target_os = "macos"))]
  let file_submenu = Submenu::with_items(
    app,
    "File",
    true,
    &[
      &PredefinedMenuItem::close_window(app, None)?,
      &PredefinedMenuItem::quit(app, None)?,
    ],
  )?;

  let edit_submenu = Submenu::with_items(
    app,
    "Edit",
    true,
    &[
      &PredefinedMenuItem::undo(app, None)?,
      &PredefinedMenuItem::redo(app, None)?,
      &PredefinedMenuItem::separator(app)?,
      &PredefinedMenuItem::cut(app, None)?,
      &PredefinedMenuItem::copy(app, None)?,
      &PredefinedMenuItem::paste(app, None)?,
      &PredefinedMenuItem::select_all(app, None)?,
    ],
  )?;

  #[cfg(target_os = "macos")]
  let view_submenu = Submenu::with_items(
    app,
    "View",
    true,
    &[&PredefinedMenuItem::fullscreen(app, None)?],
  )?;

  let window_submenu = Submenu::with_items(
    app,
    "Window",
    true,
    &[
      &PredefinedMenuItem::minimize(app, None)?,
      &PredefinedMenuItem::maximize(app, None)?,
      #[cfg(target_os = "macos")]
      &PredefinedMenuItem::separator(app)?,
      &PredefinedMenuItem::close_window(app, None)?,
    ],
  )?;

  #[cfg(target_os = "macos")]
  let help_submenu =
    Submenu::with_items(app, "Help", true, &[] as &[&dyn tauri::menu::IsMenuItem<_>])?;

  #[cfg(not(target_os = "macos"))]
  let help_submenu = Submenu::with_items(
    app,
    "Help",
    true,
    &[
      &check_updates,
      &PredefinedMenuItem::separator(app)?,
      &PredefinedMenuItem::about(app, None, Some(about_metadata))?,
    ],
  )?;

  #[cfg(target_os = "macos")]
  let menu = Menu::with_items(
    app,
    &[
      &app_submenu,
      &edit_submenu,
      &view_submenu,
      &window_submenu,
      &help_submenu,
    ],
  )?;

  #[cfg(not(target_os = "macos"))]
  let menu = Menu::with_items(
    app,
    &[&file_submenu, &edit_submenu, &window_submenu, &help_submenu],
  )?;

  app.set_menu(menu)?;
  app.on_menu_event(on_menu_event);
  Ok(())
}

fn on_menu_event(app: &AppHandle, event: MenuEvent) {
  if event.id().as_ref() != MENU_CHECK_UPDATES {
    return;
  }
  tracing::info!("updater: Check for Updates selected from menu");
  let handle = app.clone();
  tauri::async_runtime::spawn(async move {
    run_updater_check(&handle, UpdateCheckSource::Manual).await;
  });
}

/// Spawn a background task that checks for updates immediately, then every hour.
///
/// After a package is downloaded (`ready`), the install card stays visible. Hourly
/// checks still run so a *newer* remote version can replace the staged download.
fn start_updater_loop(app: AppHandle) {
  tauri::async_runtime::spawn(async move {
    loop {
      run_updater_check(&app, UpdateCheckSource::Automatic).await;
      tokio::time::sleep(UPDATE_CHECK_INTERVAL).await;
    }
  });
}

/// Check for a newer app build via `tauri-plugin-updater` and notify the UI card.
///
/// When an update is available the left-nav card is shown (Download → progress → Install).
/// Once downloaded, install status is always kept until a newer version is found.
/// Manual checks also dialog on up-to-date / errors (unless a download is staged).
async fn run_updater_check(app: &AppHandle, source: UpdateCheckSource) {
  let Some(state) = app.try_state::<UpdaterState>() else {
    tracing::warn!("updater: UpdaterState not managed");
    return;
  };

  // Mid-download / mid-install: only re-surface progress, do not start another check.
  if state.downloading.load(Ordering::Relaxed) || state.installing.load(Ordering::Relaxed) {
    let payload = state.snapshot_status();
    tracing::info!(phase = %payload.phase, "updater: re-emitting in-progress update state");
    emit_app_update(app, &payload);
    return;
  }

  // Downloaded and ready to install: always keep the install card visible, then
  // still poll the remote so a *newer* version can replace the staged package.
  let ready = state.update_ready.load(Ordering::Relaxed);
  if ready {
    emit_app_update(app, &state.snapshot_status());
  }

  // Pending "available" (not yet downloaded): automatic respects Later; manual re-shows.
  // When ready we always continue to a remote check (do not early-return here).
  if !ready
    && state
      .pending_update
      .lock()
      .ok()
      .is_some_and(|g| g.is_some())
    && let Some(version) = state.staged_version() {
      if source == UpdateCheckSource::Automatic && state.is_deferred(&version) {
        tracing::debug!(
          version = %version,
          "updater: user deferred this version; skipping automatic prompt"
        );
        return;
      }
      if source == UpdateCheckSource::Manual {
        state.clear_deferred(&version);
      }
      // Re-surface known available state; still re-check remote below so we pick up
      // a newer build if the catalog moved on.
      emit_app_update(app, &state.snapshot_status());
    }

  if state
    .checking
    .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
    .is_err()
  {
    tracing::info!("updater: check already in progress");
    if source == UpdateCheckSource::Manual {
      // Prefer re-showing staged install/available over a "please wait" dialog.
      let payload = state.snapshot_status();
      if payload.phase != "idle" {
        emit_app_update(app, &payload);
      } else {
        show_info_dialog(
          app,
          "Checking for Updates",
          "An update check is already in progress. Please wait.",
        );
      }
    }
    return;
  }

  let result = run_updater_check_inner(app, source).await;
  state.checking.store(false, Ordering::SeqCst);

  if let Err(e) = result {
    tracing::warn!("updater: {e}");
    // Keep install-ready visible even if the remote check failed.
    if state.update_ready.load(Ordering::Relaxed) {
      emit_app_update(app, &state.snapshot_status());
    } else if source == UpdateCheckSource::Manual {
      show_info_dialog(app, "Update check failed", &e);
    }
  }
}

async fn run_updater_check_inner(app: &AppHandle, source: UpdateCheckSource) -> Result<(), String> {
  tracing::info!(?source, "updater: checking for updates…");

  let updater = app
    .updater()
    .map_err(|e| format!("Failed to create updater client: {e}"))?;

  let update = updater
    .check()
    .await
    .map_err(|e| format!("Update check failed: {e}"))?;

  let Some(state) = app.try_state::<UpdaterState>() else {
    return Err("UpdaterState not managed".into());
  };

  let Some(update) = update else {
    // Remote says no update. If we already downloaded a package, keep install status.
    if state.update_ready.load(Ordering::Relaxed) {
      let payload = state.snapshot_status();
      tracing::info!(
        version = %payload.version,
        "updater: no newer remote update; keeping install-ready status"
      );
      emit_app_update(app, &payload);
      return Ok(());
    }
    tracing::info!("updater: no update available (already up to date)");
    if source == UpdateCheckSource::Manual {
      let version = app.package_info().version.to_string();
      show_info_dialog(
        app,
        "You're up to date",
        &format!("CDXTheme {version} is the latest version."),
      );
    }
    return Ok(());
  };

  tracing::info!(
    current = %update.current_version,
    latest = %update.version,
    target = %update.target,
    url = %update.download_url,
    "updater: update available"
  );
  if let Some(body) = update.body.as_deref()
    && !body.trim().is_empty() {
      tracing::info!("updater: release notes:\n{body}");
    }

  let staged = state.staged_version();

  // Already downloaded this exact version → always keep install status.
  if state.update_ready.load(Ordering::Relaxed)
    && staged.as_deref() == Some(update.version.as_str())
  {
    let payload = state.snapshot_status();
    tracing::info!(
      version = %update.version,
      "updater: staged download still current; keeping install-ready status"
    );
    emit_app_update(app, &payload);
    return Ok(());
  }

  // Downloaded an older package, but remote has a newer version → replace with available.
  if state.update_ready.load(Ordering::Relaxed)
    && staged.as_ref().is_some_and(|v| v != &update.version)
  {
    tracing::info!(
      staged = %staged.as_deref().unwrap_or("?"),
      latest = %update.version,
      "updater: newer version found; replacing staged download"
    );
    state.clear_deferred(&update.version);
    let current = update.current_version.clone();
    let version = update.version.clone();
    let body = update.body.clone().filter(|s| !s.trim().is_empty());
    state.set_pending(update); // clears downloaded bytes + ready flag
    emit_app_update(app, &AppUpdatePayload::available(current, version, body));
    return Ok(());
  }

  // Same pending (not downloaded) version: keep card, do not reset.
  if staged.as_deref() == Some(update.version.as_str())
    && state
      .pending_update
      .lock()
      .ok()
      .is_some_and(|g| g.is_some())
  {
    if source == UpdateCheckSource::Automatic && state.is_deferred(&update.version) {
      tracing::debug!(
        version = %update.version,
        "updater: user deferred this version; skipping automatic prompt"
      );
      return Ok(());
    }
    if source == UpdateCheckSource::Manual {
      state.clear_deferred(&update.version);
    }
    emit_app_update(app, &state.snapshot_status());
    return Ok(());
  }

  // Automatic polls: skip re-prompting if the user already chose Later for this version.
  if source == UpdateCheckSource::Automatic && state.is_deferred(&update.version) {
    tracing::debug!(
      version = %update.version,
      "updater: user deferred this version; skipping automatic prompt"
    );
    return Ok(());
  }

  // Manual check after Later should show the card again.
  if source == UpdateCheckSource::Manual {
    state.clear_deferred(&update.version);
  }

  let current = update.current_version.clone();
  let version = update.version.clone();
  let body = update.body.clone().filter(|s| !s.trim().is_empty());
  state.set_pending(update);

  let payload = AppUpdatePayload::available(current, version.clone(), body);
  tracing::info!(version = %version, "updater: showing in-app update notification");
  emit_app_update(app, &payload);
  Ok(())
}

/// Current update notification state (for UI mount / late join).
#[tauri::command]
fn get_app_update_status(app: AppHandle) -> AppUpdatePayload {
  app
    .try_state::<UpdaterState>()
    .map(|s| s.snapshot_status())
    .unwrap_or_else(AppUpdatePayload::idle)
}

/// Download the pending update package (progress is streamed via `app-update` events).
#[tauri::command]
async fn download_app_update(app: AppHandle) -> Result<(), String> {
  let Some(state) = app.try_state::<UpdaterState>() else {
    return Err("UpdaterState not managed".into());
  };

  if state.update_ready.load(Ordering::Relaxed) {
    emit_app_update(&app, &state.snapshot_status());
    return Ok(());
  }

  if state
    .downloading
    .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
    .is_err()
  {
    return Err("A download is already in progress".into());
  }

  let update = {
    let guard = state
      .pending_update
      .lock()
      .map_err(|_| "Failed to lock pending update".to_string())?;
    guard.clone()
  };

  let Some(update) = update else {
    state.downloading.store(false, Ordering::SeqCst);
    return Err("No update available to download".into());
  };

  let current = update.current_version.clone();
  let version = update.version.clone();
  let body = update.body.clone().filter(|s| !s.trim().is_empty());

  emit_app_update(
    &app,
    &AppUpdatePayload::downloading(
      current.clone(),
      version.clone(),
      body.clone(),
      0,
      None,
      Some(0),
    ),
  );

  let downloaded = AtomicUsize::new(0);
  let last_logged_pct = AtomicU8::new(0);
  let last_emitted_pct = AtomicU8::new(0);

  tracing::info!(version = %version, "updater: downloading…");
  let result = update
    .download(
      |chunk_len, content_len| {
        let total_dl = downloaded.fetch_add(chunk_len, Ordering::Relaxed) + chunk_len;
        let total_dl_u64 = total_dl as u64;
        let percent = content_len
          .filter(|t| *t > 0)
          .map(|total| ((total_dl_u64 * 100) / total).min(100) as u8);

        if let Ok(mut guard) = state.last_progress.lock() {
          guard.downloaded = total_dl_u64;
          guard.total = content_len;
          guard.percent = percent;
        }

        if let Some(pct) = percent {
          let prev_log = last_logged_pct.load(Ordering::Relaxed);
          if pct >= prev_log.saturating_add(10) || pct == 100 {
            last_logged_pct.store(pct, Ordering::Relaxed);
            tracing::info!(
              downloaded = total_dl,
              total = ?content_len,
              pct,
              "updater: download progress"
            );
          }
          // Emit more often for the UI progress bar (~2% steps).
          let prev_emit = last_emitted_pct.load(Ordering::Relaxed);
          if pct >= prev_emit.saturating_add(2) || pct == 100 {
            last_emitted_pct.store(pct, Ordering::Relaxed);
            emit_app_update(
              &app,
              &AppUpdatePayload::downloading(
                current.clone(),
                version.clone(),
                body.clone(),
                total_dl_u64,
                content_len,
                Some(pct),
              ),
            );
          }
        } else if total_dl == chunk_len || total_dl % (512 * 1024) < chunk_len {
          if total_dl == chunk_len || total_dl % (2 * 1024 * 1024) < chunk_len {
            tracing::info!(downloaded = total_dl, "updater: downloading…");
          }
          emit_app_update(
            &app,
            &AppUpdatePayload::downloading(
              current.clone(),
              version.clone(),
              body.clone(),
              total_dl_u64,
              content_len,
              None,
            ),
          );
        }
      },
      || {
        tracing::info!(
          bytes = downloaded.load(Ordering::Relaxed),
          "updater: download finished, verifying signature…"
        );
      },
    )
    .await;

  state.downloading.store(false, Ordering::SeqCst);

  match result {
    Ok(bytes) => {
      tracing::info!(
        version = %version,
        bytes = bytes.len(),
        "updater: download complete — ready to install"
      );
      state.mark_downloaded(bytes);
      emit_app_update(&app, &AppUpdatePayload::ready(current, version, body));
      Ok(())
    }
    Err(e) => {
      let msg = format!("Download failed: {e}");
      tracing::warn!("updater: {msg}");
      emit_app_update(
        &app,
        &AppUpdatePayload::error_with(current, version, body, msg.clone(), false),
      );
      Err(msg)
    }
  }
}

/// Install a previously downloaded update and restart the app.
#[tauri::command]
async fn install_app_update(app: AppHandle) -> Result<(), String> {
  let Some(state) = app.try_state::<UpdaterState>() else {
    return Err("UpdaterState not managed".into());
  };

  if state
    .installing
    .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
    .is_err()
  {
    return Err("Install already in progress".into());
  }

  let pair = state.take_download_pair();
  let Some((update, bytes)) = pair else {
    state.installing.store(false, Ordering::SeqCst);
    // Re-store readiness if bytes were missing but flag said ready.
    if state.update_ready.load(Ordering::Relaxed) {
      // Bytes already taken or missing — fail cleanly.
    }
    return Err("No downloaded update ready to install. Download the update first.".into());
  };

  let current = update.current_version.clone();
  let version = update.version.clone();
  let body = update.body.clone().filter(|s| !s.trim().is_empty());

  emit_app_update(
    &app,
    &AppUpdatePayload::installing(current.clone(), version.clone(), body.clone()),
  );

  tracing::info!(version = %version, "updater: installing…");
  let install_result = update.install(&bytes);
  state.installing.store(false, Ordering::SeqCst);

  if let Err(e) = install_result {
    // Put bytes back so the user can retry Install.
    state.mark_downloaded(bytes);
    if let Ok(mut guard) = state.pending_update.lock() {
      *guard = Some(update);
    }
    let msg = format!("Install failed: {e}");
    tracing::warn!("updater: {msg}");
    emit_app_update(
      &app,
      &AppUpdatePayload::error_with(current, version, body, msg.clone(), true),
    );
    return Err(msg);
  }

  tracing::info!(
    version = %version,
    "updater: install complete — restarting"
  );
  app.restart();
}

/// Dismiss the update card for this version (session-only; manual check can re-show).
///
/// Cannot dismiss once a package is downloaded — install status stays until the user
/// installs or a newer remote version replaces it.
#[tauri::command]
fn dismiss_app_update(app: AppHandle) -> Result<(), String> {
  let Some(state) = app.try_state::<UpdaterState>() else {
    return Err("UpdaterState not managed".into());
  };

  if state.downloading.load(Ordering::Relaxed) || state.installing.load(Ordering::Relaxed) {
    return Err("Cannot dismiss while download or install is in progress".into());
  }

  // After download, always keep install-ready visible.
  if state.update_ready.load(Ordering::Relaxed) {
    tracing::info!("updater: dismiss ignored — update already downloaded (install ready)");
    emit_app_update(&app, &state.snapshot_status());
    return Ok(());
  }

  if let Some(version) = state.staged_version() {
    state.mark_deferred(&version);
    tracing::info!(version = %version, "updater: user dismissed update notification");
  }

  emit_app_update(&app, &AppUpdatePayload::idle());
  Ok(())
}

fn show_info_dialog(app: &AppHandle, title: &str, message: &str) {
  let mut builder = app
    .dialog()
    .message(message)
    .title(title)
    .kind(MessageDialogKind::Info);
  if let Some(window) = app.get_webview_window("main") {
    builder = builder.parent(&window);
  }
  builder.show(|_| {});
}
