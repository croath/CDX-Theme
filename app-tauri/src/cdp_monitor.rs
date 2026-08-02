//! Background CDP reachability monitor — started when the Tauri app launches.
//! Probes both Codex (app://) and WorkBuddy (file://) remote-debugging ports.

use crate::app_state::{AppState, CdpServerStatus, CdpTargetInfo, DualCdpStatus};
use crate::injector::{
  DEFAULT_CDP_PORT, DEFAULT_WORKBUDDY_CDP_PORT, TargetUrlKind, wait_for_targets_with,
};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

const POLL_INTERVAL: Duration = Duration::from_secs(2);
const PROBE_TIMEOUT_MS: u64 = 1_200;

/// Spawn a long-running task that probes configured Codex + WorkBuddy CDP ports.
pub fn start(app: AppHandle) {
  tauri::async_runtime::spawn(async move {
    update_once(&app).await;
    loop {
      tokio::time::sleep(POLL_INTERVAL).await;
      update_once(&app).await;
    }
  });
}

async fn update_once(app: &AppHandle) {
  let (codex_port, workbuddy_port) = app
    .try_state::<AppState>()
    .map(|s| (s.cdp_port(), s.workbuddy_cdp_port()))
    .unwrap_or((DEFAULT_CDP_PORT, DEFAULT_WORKBUDDY_CDP_PORT));

  let (codex, workbuddy) = tokio::join!(
    probe("codex", codex_port, TargetUrlKind::App),
    probe("workbuddy", workbuddy_port, TargetUrlKind::File),
  );

  let snapshot = DualCdpStatus { codex, workbuddy };
  if let Some(managed) = app.try_state::<AppState>() {
    managed.set_dual_cdp_status(snapshot.clone());
  }
  let _ = app.emit("cdp-status", &snapshot);
}

async fn probe(app_id: &str, port: u16, kind: TargetUrlKind) -> CdpServerStatus {
  match wait_for_targets_with(port, PROBE_TIMEOUT_MS, kind).await {
    Ok(targets) => CdpServerStatus {
      app: app_id.to_string(),
      connected: true,
      port,
      target_count: targets.len(),
      targets: targets
        .into_iter()
        .map(|t| CdpTargetInfo {
          id: t.id,
          title: t.title,
          url: t.url,
        })
        .collect(),
      message: format!(
        "{} CDP reachable ({})",
        display_name(app_id),
        kind.label()
      ),
    },
    Err(e) => CdpServerStatus::disconnected(app_id, port, e),
  }
}

fn display_name(app_id: &str) -> &'static str {
  match app_id {
    "workbuddy" => "WorkBuddy",
    _ => "Codex",
  }
}
