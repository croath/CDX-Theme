use serde::Serialize;
use std::sync::Mutex;

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
  pub connected: bool,
  pub port: u16,
  pub target_count: usize,
  pub targets: Vec<CdpTargetInfo>,
  pub message: String,
}

impl Default for CdpServerStatus {
  fn default() -> Self {
    Self {
      connected: false,
      port: crate::injector::DEFAULT_CDP_PORT,
      target_count: 0,
      targets: vec![],
      message: "Starting CDP monitor…".into(),
    }
  }
}

pub struct AppState {
  cdp: Mutex<CdpServerStatus>,
  cdp_port: Mutex<u16>,
}

impl Default for AppState {
  fn default() -> Self {
    Self::new(crate::injector::DEFAULT_CDP_PORT)
  }
}

impl AppState {
  pub fn new(cdp_port: u16) -> Self {
    let mut status = CdpServerStatus::default();
    status.port = cdp_port;
    Self {
      cdp: Mutex::new(status),
      cdp_port: Mutex::new(cdp_port),
    }
  }

  pub fn cdp_port(&self) -> u16 {
    self
      .cdp_port
      .lock()
      .map(|g| *g)
      .unwrap_or(crate::injector::DEFAULT_CDP_PORT)
  }

  pub fn set_cdp_port(&self, port: u16) {
    if let Ok(mut guard) = self.cdp_port.lock() {
      *guard = port;
    }
    if let Ok(mut status) = self.cdp.lock() {
      status.port = port;
    }
  }

  pub fn set_cdp_status(&self, status: CdpServerStatus) {
    if let Ok(mut guard) = self.cdp.lock() {
      *guard = status;
    }
  }

  pub fn cdp_status(&self) -> CdpServerStatus {
    self.cdp.lock().map(|g| g.clone()).unwrap_or_default()
  }
}
