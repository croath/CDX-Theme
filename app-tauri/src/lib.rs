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
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Manager, RunEvent};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_updater::UpdaterExt;

/// How often to poll the remote updater endpoint while the app is running.
const UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(60 * 60);

/// Menu item id for manual update checks (app / Help menu).
const MENU_CHECK_UPDATES: &str = "check_for_updates";

/// Shared updater flags for the hourly loop and the system-menu action.
struct UpdaterState {
  /// True after download+install succeeded; app needs restart.
  update_ready: AtomicBool,
  /// Guards concurrent automatic/manual checks.
  checking: AtomicBool,
  /// Staged version string when `update_ready` is true.
  staged_version: std::sync::Mutex<Option<String>>,
  /// Version the user chose "Later" for (session-only; re-prompt on manual check).
  deferred_version: std::sync::Mutex<Option<String>>,
}

impl UpdaterState {
  fn new() -> Self {
    Self {
      update_ready: AtomicBool::new(false),
      checking: AtomicBool::new(false),
      staged_version: std::sync::Mutex::new(None),
      deferred_version: std::sync::Mutex::new(None),
    }
  }

  fn mark_ready(&self, version: &str) {
    if let Ok(mut guard) = self.staged_version.lock() {
      *guard = Some(version.to_string());
    }
    self.update_ready.store(true, Ordering::Relaxed);
  }

  fn staged_version(&self) -> Option<String> {
    self.staged_version.lock().ok().and_then(|g| g.clone())
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
      commands::delete_theme_builder_session,
      commands::get_codex_session,
      commands::start_theme_build,
      commands::check_theme_builder_runtime,
      commands::install_bun_for_theme_builder,
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
  app.on_menu_event(|app, event| on_menu_event(app, event));
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
/// After a successful download/install, further checks are skipped until restart
/// (the staged update is already ready).
fn start_updater_loop(app: AppHandle) {
  tauri::async_runtime::spawn(async move {
    loop {
      let already_ready = app
        .try_state::<UpdaterState>()
        .map(|s| s.update_ready.load(Ordering::Relaxed))
        .unwrap_or(false);
      if already_ready {
        tracing::debug!("updater: update already staged; waiting for restart");
      } else {
        run_updater_check(&app, UpdateCheckSource::Automatic).await;
      }
      tokio::time::sleep(UPDATE_CHECK_INTERVAL).await;
    }
  });
}

/// Check for a newer app build via `tauri-plugin-updater` and log every stage.
///
/// When an update is available: notify the user → on accept download → verify → install,
/// then prompt to restart. Manual checks also dialog on up-to-date / errors.
async fn run_updater_check(app: &AppHandle, source: UpdateCheckSource) {
  let Some(state) = app.try_state::<UpdaterState>() else {
    tracing::warn!("updater: UpdaterState not managed");
    return;
  };

  if state.update_ready.load(Ordering::Relaxed) {
    tracing::info!("updater: update already staged — prompting restart");
    let version = state
      .staged_version()
      .unwrap_or_else(|| app.package_info().version.to_string());
    notify_update_ready(app, &version).await;
    return;
  }

  if state
    .checking
    .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
    .is_err()
  {
    tracing::info!("updater: check already in progress");
    if source == UpdateCheckSource::Manual {
      show_info_dialog(
        app,
        "Checking for Updates",
        "An update check is already in progress. Please wait.",
      );
    }
    return;
  }

  let result = run_updater_check_inner(app, source).await;
  state.checking.store(false, Ordering::SeqCst);

  if let Err(e) = result {
    tracing::warn!("updater: {e}");
    if source == UpdateCheckSource::Manual {
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

  let Some(update) = update else {
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
  if let Some(body) = update.body.as_deref() {
    if !body.trim().is_empty() {
      tracing::info!("updater: release notes:\n{body}");
    }
  }

  // Automatic polls: skip re-prompting if the user already chose Later for this version.
  if source == UpdateCheckSource::Automatic {
    if let Some(state) = app.try_state::<UpdaterState>() {
      if state.is_deferred(&update.version) {
        tracing::debug!(
          version = %update.version,
          "updater: user deferred this version; skipping automatic prompt"
        );
        return Ok(());
      }
    }
  }

  let should_install = ask_install_update(
    app,
    &update.current_version,
    &update.version,
    update.body.as_deref(),
  )
  .await;
  if !should_install {
    tracing::info!(
      version = %update.version,
      "updater: user deferred install"
    );
    if let Some(state) = app.try_state::<UpdaterState>() {
      state.mark_deferred(&update.version);
    }
    return Ok(());
  }

  use std::sync::atomic::{AtomicU8, AtomicUsize};
  let downloaded = AtomicUsize::new(0);
  let last_logged_pct = AtomicU8::new(0);
  let version = update.version.clone();

  tracing::info!(version = %version, "updater: user accepted — downloading…");
  update
    .download_and_install(
      |chunk_len, content_len| {
        let total_dl = downloaded.fetch_add(chunk_len, Ordering::Relaxed) + chunk_len;
        if let Some(total) = content_len {
          if total > 0 {
            let pct = ((total_dl as u64 * 100) / total).min(100) as u8;
            // Log every ~10% to avoid spam.
            let prev = last_logged_pct.load(Ordering::Relaxed);
            if pct >= prev.saturating_add(10) || pct == 100 {
              last_logged_pct.store(pct, Ordering::Relaxed);
              tracing::info!(
                downloaded = total_dl,
                total,
                pct,
                "updater: download progress"
              );
            }
          }
        } else if total_dl == chunk_len || total_dl % (2 * 1024 * 1024) < chunk_len {
          // No Content-Length: log first chunk and roughly every 2MB.
          tracing::info!(downloaded = total_dl, "updater: downloading…");
        }
      },
      || {
        tracing::info!(
          bytes = downloaded.load(Ordering::Relaxed),
          "updater: download finished, verifying signature…"
        );
      },
    )
    .await
    .map_err(|e| format!("Download/install failed: {e}"))?;

  tracing::info!(
    version = %version,
    "updater: install complete — restart the app to run the new version"
  );
  if let Some(state) = app.try_state::<UpdaterState>() {
    state.mark_ready(&version);
  }
  notify_update_ready(app, &version).await;
  Ok(())
}

/// Ask the user whether to install a newly discovered update.
async fn ask_install_update(
  app: &AppHandle,
  current: &str,
  latest: &str,
  release_notes: Option<&str>,
) -> bool {
  let mut body =
    format!("CDXTheme {latest} is available (you have {current}).\n\nInstall this update now?");
  if let Some(notes) = release_notes {
    let notes = notes.trim();
    if !notes.is_empty() {
      // Keep the dialog readable; full notes are in the log.
      let preview: String = notes.chars().take(400).collect();
      let ellipsis = if notes.chars().count() > 400 {
        "…"
      } else {
        ""
      };
      body.push_str(&format!("\n\n{preview}{ellipsis}"));
    }
  }

  tracing::info!(current, latest, "updater: prompting user to install update");
  show_confirm_dialog(app, "Update available", &body, "Install", "Later").await
}

/// Show a restart dialog after an update has been downloaded and staged.
async fn notify_update_ready(app: &AppHandle, version: &str) {
  let title = "Update ready";
  let body =
    format!("CDXTheme {version} has been downloaded. Restart the app to apply the update.");

  tracing::info!("updater: showing restart dialog");
  let confirmed = show_confirm_dialog(app, title, &body, "Restart now", "Later").await;
  if confirmed {
    tracing::info!("updater: user chose restart now");
    app.restart();
  } else {
    tracing::info!("updater: user deferred restart");
  }
}

/// Blocking-style confirm dialog bridged into async via oneshot.
async fn show_confirm_dialog(
  app: &AppHandle,
  title: &str,
  message: &str,
  ok_label: &str,
  cancel_label: &str,
) -> bool {
  let (tx, rx) = tokio::sync::oneshot::channel();
  let mut builder = app
    .dialog()
    .message(message)
    .title(title)
    .kind(MessageDialogKind::Info)
    .buttons(MessageDialogButtons::OkCancelCustom(
      ok_label.into(),
      cancel_label.into(),
    ));

  if let Some(window) = app.get_webview_window("main") {
    builder = builder.parent(&window);
  }

  builder.show(move |confirmed| {
    let _ = tx.send(confirmed);
  });

  rx.await.unwrap_or(false)
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
