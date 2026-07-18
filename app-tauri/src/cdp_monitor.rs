//! Background CDP reachability monitor — started when the Tauri app launches.

use crate::app_state::{AppState, CdpServerStatus, CdpTargetInfo};
use crate::injector::wait_for_targets;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

const POLL_INTERVAL: Duration = Duration::from_secs(2);

/// Spawn a long-running task that probes the configured Codex CDP port.
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
  let port = app
    .try_state::<AppState>()
    .map(|s| s.cdp_port())
    .unwrap_or(crate::injector::DEFAULT_CDP_PORT);

  let snapshot = probe(port).await;
  if let Some(managed) = app.try_state::<AppState>() {
    managed.set_cdp_status(snapshot.clone());
  }
  let _ = app.emit("cdp-status", &snapshot);
}

async fn probe(port: u16) -> CdpServerStatus {
  match wait_for_targets(port, 1_200).await {
    Ok(targets) => CdpServerStatus {
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
      message: "Codex CDP reachable".into(),
    },
    Err(e) => CdpServerStatus {
      connected: false,
      port,
      target_count: 0,
      targets: vec![],
      message: e,
    },
  }
}
