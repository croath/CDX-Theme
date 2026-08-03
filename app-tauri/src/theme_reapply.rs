//! Background inject health check + auto-reapply for hosts with a stored applied theme.
//!
//! Focus: WorkBuddy loses skin on reload/relaunch; when CDP is reachable and verify fails,
//! re-inject the last applied package. Codex is handled the same way when a theme is recorded.

use crate::app_state::AppState;
use crate::injector::{
  self, APP_CODEX, APP_WORKBUDDY, InjectOptions, TargetUrlKind, load_theme_package,
  verify_theme_for_app, wait_for_targets_with,
};
use crate::settings_store;
use crate::theme_catalog;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

/// How often to check inject status (independent of CDP status poll).
const CHECK_INTERVAL: Duration = Duration::from_secs(4);
/// Minimum gap between reapply attempts per host (avoid thrash on flaky CDP).
const REAPPLY_COOLDOWN: Duration = Duration::from_secs(12);
/// Short CDP / verify timeouts — monitor must stay snappy.
const CDP_PROBE_MS: u64 = 1_500;
const VERIFY_TIMEOUT_MS: u64 = 4_000;
const INJECT_TIMEOUT_MS: u64 = 120_000;

#[derive(Default)]
struct HostReapplyState {
  /// Last time we attempted reapply (success or fail).
  last_attempt: Option<Instant>,
  /// Theme id that last verified as OK on this host.
  last_ok_theme: Option<String>,
  /// In-flight reapply (skip concurrent checks).
  busy: bool,
}

static STATE: LazyLock<Mutex<HashMap<String, HostReapplyState>>> =
  LazyLock::new(|| Mutex::new(HashMap::new()));

/// Spawn background loop that keeps host skins in sync with stored applied themes.
pub fn start(app: AppHandle) {
  tauri::async_runtime::spawn(async move {
    // Small delay so CDP monitor / window settle first.
    tokio::time::sleep(Duration::from_secs(3)).await;
    loop {
      check_all(&app).await;
      tokio::time::sleep(CHECK_INTERVAL).await;
    }
  });
}

async fn check_all(app: &AppHandle) {
  let (codex_port, workbuddy_port) = app
    .try_state::<AppState>()
    .map(|s| (s.cdp_port(), s.workbuddy_cdp_port()))
    .unwrap_or((
      injector::DEFAULT_CDP_PORT,
      injector::DEFAULT_WORKBUDDY_CDP_PORT,
    ));

  // Prefer WorkBuddy (product request); also maintain Codex when recorded.
  let jobs = [
    (APP_WORKBUDDY, workbuddy_port, TargetUrlKind::File),
    (APP_CODEX, codex_port, TargetUrlKind::App),
  ];

  for (host, port, kind) in jobs {
    if let Err(e) = check_host(app, host, port, kind).await {
      tracing::debug!(host, "theme reapply check: {e}");
    }
  }
}

async fn check_host(
  app: &AppHandle,
  host: &str,
  port: u16,
  kind: TargetUrlKind,
) -> Result<(), String> {
  let Some(theme_id) = settings_store::applied_theme_for(app, host) else {
    // Clear OK marker so a later apply re-verifies cleanly.
    if let Ok(mut map) = STATE.lock() {
      map.remove(host);
    }
    return Ok(());
  };

  // Only act when CDP is already up — do not auto-launch hosts.
  if wait_for_targets_with(port, CDP_PROBE_MS, kind)
    .await
    .is_err()
  {
    if let Ok(mut map) = STATE.lock() {
      if let Some(st) = map.get_mut(host) {
        st.last_ok_theme = None;
      }
    }
    return Ok(());
  }

  // Cooldown / busy guard
  {
    let mut map = STATE.lock().map_err(|e| e.to_string())?;
    let st = map.entry(host.to_string()).or_default();
    if st.busy {
      return Ok(());
    }
    if st.last_ok_theme.as_deref() == Some(theme_id.as_str()) {
      // Recently verified OK — still re-check, but no need to skip entirely.
    }
    if let Some(last) = st.last_attempt
      && last.elapsed() < REAPPLY_COOLDOWN
      && st.last_ok_theme.as_deref() != Some(theme_id.as_str())
    {
      // Failed or not ok recently; wait out cooldown before another inject.
      // Still allow verify-only path when last_ok is set.
    }
  }

  // Resolve local package (no download during background reapply).
  let package = match theme_catalog::local_theme_package_path(app, &theme_id) {
    Ok(p) => p,
    Err(e) => {
      tracing::warn!(host, theme_id = %theme_id, "applied theme package missing: {e}");
      return Ok(());
    }
  };

  let theme = load_theme_package(&package)?;
  // Skip if package has no target for this host.
  match host {
    APP_WORKBUDDY if theme.workbuddy().is_none() => {
      tracing::debug!(theme_id = %theme_id, "skip reapply: package has no workbuddy target");
      return Ok(());
    }
    APP_CODEX if theme.codex().is_err() => {
      tracing::debug!(theme_id = %theme_id, "skip reapply: package has no codex target");
      return Ok(());
    }
    _ => {}
  }

  let public = theme.public();
  let verify_opts = InjectOptions {
    port,
    timeout_ms: VERIFY_TIMEOUT_MS,
  };

  let ok = match verify_theme_for_app(host, Some(&public), verify_opts).await {
    Ok(run) => run.targets.iter().any(|t| {
      t.result
        .get("pass")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    }),
    Err(e) => {
      tracing::debug!(host, "verify failed: {e}");
      false
    }
  };

  if ok {
    if let Ok(mut map) = STATE.lock() {
      let st = map.entry(host.to_string()).or_default();
      st.last_ok_theme = Some(theme_id);
    }
    return Ok(());
  }

  // Needs reapply — respect cooldown
  {
    let mut map = STATE.lock().map_err(|e| e.to_string())?;
    let st = map.entry(host.to_string()).or_default();
    if st.busy {
      return Ok(());
    }
    if let Some(last) = st.last_attempt
      && last.elapsed() < REAPPLY_COOLDOWN
    {
      return Ok(());
    }
    st.busy = true;
    st.last_attempt = Some(Instant::now());
    st.last_ok_theme = None;
  }

  tracing::info!(
    host,
    theme_id = %theme_id,
    port,
    "theme inject missing or stale — auto-reapplying"
  );

  let inject_opts = InjectOptions {
    port,
    timeout_ms: INJECT_TIMEOUT_MS,
  };
  let result = injector::apply_loaded_theme_for_app(host, &theme, inject_opts).await;

  if let Ok(mut map) = STATE.lock() {
    if let Some(st) = map.get_mut(host) {
      st.busy = false;
      if result.is_ok() {
        st.last_ok_theme = Some(theme_id.clone());
      }
    }
  }

  match result {
    Ok(_) => {
      tracing::info!(host, theme_id = %theme_id, "auto-reapply ok");
      Ok(())
    }
    Err(e) => {
      tracing::warn!(host, theme_id = %theme_id, "auto-reapply failed: {e}");
      Err(e)
    }
  }
}
