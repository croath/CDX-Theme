pub mod analytics;
pub mod app_state;
pub mod cdp_monitor;
pub mod codex_launch;
pub mod commands;
pub mod image_cache;
pub mod injector;
pub mod paths;
pub mod settings_store;
pub mod theme_catalog;
pub mod theme_lib;
pub mod theme_package;
pub mod theme_tool;
pub mod types;

use app_state::AppState;
use std::sync::Arc;
use tauri::{Manager, RunEvent};
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_updater::UpdaterExt;

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
      tracing::debug!("CDP port from settings: {port}");

      // Ensure user theme drop-in folder exists: {local_data}/themes
      if let Err(e) = theme_catalog::ensure_user_themes_dir(app.handle()) {
        tracing::warn!("user themes dir: {e}");
      }

      // Background: analytics init, then updates, then CDP monitor (do not auto-launch ChatGPT).
      let handle = app.handle().clone();
      tauri::async_runtime::spawn(async move {
        analytics::Analytics::init(&handle).await;
        run_updater_check(&handle).await;
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

/// Check for a newer app build via `tauri-plugin-updater` and log every stage.
///
/// On success with an available update: download → verify → install, with progress logs.
async fn run_updater_check(app: &tauri::AppHandle) {
  tracing::info!("updater: checking for updates…");

  let updater = match app.updater() {
    Ok(u) => u,
    Err(e) => {
      tracing::warn!("updater: failed to create updater client: {e}");
      return;
    }
  };

  let update = match updater.check().await {
    Ok(update) => update,
    Err(e) => {
      // Common in `tauri dev` / unsigned builds; keep as warn so release failures stand out.
      tracing::warn!("updater: check failed: {e}");
      return;
    }
  };

  let Some(update) = update else {
    tracing::info!("updater: no update available (already up to date)");
    return;
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

  use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
  let downloaded = AtomicUsize::new(0);
  let last_logged_pct = AtomicU8::new(0);

  match update
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
  {
    Ok(()) => {
      tracing::info!(
        version = %update.version,
        "updater: install complete — restart the app to run the new version"
      );
    }
    Err(e) => {
      tracing::error!("updater: download/install failed: {e}");
    }
  }
}
