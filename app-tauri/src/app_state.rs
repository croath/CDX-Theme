use serde::Serialize;
use std::sync::Mutex;

use crate::injector::{DEFAULT_CDP_PORT, DEFAULT_WORKBUDDY_CDP_PORT};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CdpTargetInfo {
  pub id: String,
  pub title: String,
  pub url: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CdpServerStatus {
  /// Host app id: `codex` or `workbuddy`.
  #[serde(default)]
  pub app: String,
  pub connected: bool,
  pub port: u16,
  pub target_count: usize,
  pub targets: Vec<CdpTargetInfo>,
  pub message: String,
}

impl CdpServerStatus {
  pub fn disconnected(app: &str, port: u16, message: impl Into<String>) -> Self {
    Self {
      app: app.to_string(),
      connected: false,
      port,
      target_count: 0,
      targets: vec![],
      message: message.into(),
    }
  }
}

impl Default for CdpServerStatus {
  fn default() -> Self {
    Self {
      app: "codex".into(),
      connected: false,
      port: DEFAULT_CDP_PORT,
      target_count: 0,
      targets: vec![],
      message: "Starting CDP monitor…".into(),
    }
  }
}

/// Combined CDP reachability for both host apps.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DualCdpStatus {
  pub codex: CdpServerStatus,
  pub workbuddy: CdpServerStatus,
}

impl Default for DualCdpStatus {
  fn default() -> Self {
    Self {
      codex: CdpServerStatus {
        app: "codex".into(),
        port: DEFAULT_CDP_PORT,
        message: "Starting CDP monitor…".into(),
        ..Default::default()
      },
      workbuddy: CdpServerStatus {
        app: "workbuddy".into(),
        port: DEFAULT_WORKBUDDY_CDP_PORT,
        message: "Starting CDP monitor…".into(),
        ..Default::default()
      },
    }
  }
}

pub struct AppState {
  cdp: Mutex<DualCdpStatus>,
  cdp_port: Mutex<u16>,
  workbuddy_cdp_port: Mutex<u16>,
}

impl Default for AppState {
  fn default() -> Self {
    Self::new(DEFAULT_CDP_PORT, DEFAULT_WORKBUDDY_CDP_PORT)
  }
}

impl AppState {
  pub fn new(cdp_port: u16, workbuddy_cdp_port: u16) -> Self {
    let dual = DualCdpStatus {
      codex: CdpServerStatus {
        app: "codex".into(),
        port: cdp_port,
        message: "Starting CDP monitor…".into(),
        ..Default::default()
      },
      workbuddy: CdpServerStatus {
        app: "workbuddy".into(),
        port: workbuddy_cdp_port,
        message: "Starting CDP monitor…".into(),
        ..Default::default()
      },
    };
    Self {
      cdp: Mutex::new(dual),
      cdp_port: Mutex::new(cdp_port),
      workbuddy_cdp_port: Mutex::new(workbuddy_cdp_port),
    }
  }

  pub fn cdp_port(&self) -> u16 {
    self.cdp_port.lock().map(|g| *g).unwrap_or(DEFAULT_CDP_PORT)
  }

  pub fn workbuddy_cdp_port(&self) -> u16 {
    self
      .workbuddy_cdp_port
      .lock()
      .map(|g| *g)
      .unwrap_or(DEFAULT_WORKBUDDY_CDP_PORT)
  }

  pub fn set_cdp_port(&self, port: u16) {
    if let Ok(mut guard) = self.cdp_port.lock() {
      *guard = port;
    }
    if let Ok(mut status) = self.cdp.lock() {
      status.codex.port = port;
    }
  }

  pub fn set_workbuddy_cdp_port(&self, port: u16) {
    if let Ok(mut guard) = self.workbuddy_cdp_port.lock() {
      *guard = port;
    }
    if let Ok(mut status) = self.cdp.lock() {
      status.workbuddy.port = port;
    }
  }

  pub fn set_dual_cdp_status(&self, status: DualCdpStatus) {
    if let Ok(mut guard) = self.cdp.lock() {
      *guard = status;
    }
  }

  pub fn dual_cdp_status(&self) -> DualCdpStatus {
    self.cdp.lock().map(|g| g.clone()).unwrap_or_default()
  }

  /// Codex-only snapshot (back-compat for callers that only care about Codex).
  pub fn cdp_status(&self) -> CdpServerStatus {
    self.dual_cdp_status().codex
  }
}
