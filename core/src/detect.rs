//! Detect host app install / process / CDP status for CLI `cdxtheme detect`.

use crate::cdp::{TargetUrlKind, wait_for_targets_with};
use crate::inject::{DEFAULT_CDP_PORT, DEFAULT_WORKBUDDY_CDP_PORT};
use crate::launch::{
  find_chatgpt_app, find_workbuddy_app, find_workbuddy_install_path, is_chatgpt_running,
  is_workbuddy_running,
};
use cdx_theme_types::{APP_CODEX, APP_WORKBUDDY};
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Install + runtime status for one host app.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostDetectStatus {
  /// Package / CLI app id (`codex`, `workbuddy`).
  pub app_id: String,
  /// Human label for display.
  pub display_name: String,
  /// Whether a desktop install was found.
  pub installed: bool,
  /// User-facing install path (`.app` bundle on macOS when available, else executable).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub path: Option<PathBuf>,
  /// Resolved executable used for launch / process checks.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub executable: Option<PathBuf>,
  /// Whether the host process appears to be running.
  pub running: bool,
  /// Default CDP remote-debugging port for this host.
  pub default_cdp_port: u16,
  /// Whether CDP responded with matching page targets on the default port.
  pub cdp_reachable: bool,
  /// Number of matching CDP page targets when reachable.
  pub cdp_targets: usize,
}

/// Report for all known host apps.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectReport {
  pub hosts: Vec<HostDetectStatus>,
}

impl DetectReport {
  /// True when every host is installed.
  pub fn all_installed(&self) -> bool {
    !self.hosts.is_empty() && self.hosts.iter().all(|h| h.installed)
  }

  /// True when at least one host is installed.
  pub fn any_installed(&self) -> bool {
    self.hosts.iter().any(|h| h.installed)
  }
}

/// Detect Codex and WorkBuddy install, process, and default-port CDP status.
pub async fn detect_hosts() -> DetectReport {
  DetectReport {
    hosts: vec![detect_codex().await, detect_workbuddy().await],
  }
}

async fn detect_codex() -> HostDetectStatus {
  let executable = find_chatgpt_app();
  let path = executable.as_ref().map(|p| display_install_path(p));
  let installed =
    executable.as_ref().is_some_and(|p| p.is_file()) || path.as_ref().is_some_and(|p| p.exists());
  let running = is_chatgpt_running();
  let port = DEFAULT_CDP_PORT;
  let (cdp_reachable, cdp_targets) = probe_cdp(port, TargetUrlKind::App).await;

  HostDetectStatus {
    app_id: APP_CODEX.into(),
    display_name: "Codex / ChatGPT".into(),
    installed,
    path,
    executable,
    running,
    default_cdp_port: port,
    cdp_reachable,
    cdp_targets,
  }
}

async fn detect_workbuddy() -> HostDetectStatus {
  let launch_path = find_workbuddy_install_path();
  let executable = find_workbuddy_app();
  let path = launch_path.or_else(|| executable.as_ref().map(|p| display_install_path(p)));
  let installed =
    path.as_ref().is_some_and(|p| p.exists()) || executable.as_ref().is_some_and(|p| p.is_file());
  let running = is_workbuddy_running();
  let port = DEFAULT_WORKBUDDY_CDP_PORT;
  let (cdp_reachable, cdp_targets) = probe_cdp(port, TargetUrlKind::File).await;

  HostDetectStatus {
    app_id: APP_WORKBUDDY.into(),
    display_name: "WorkBuddy AI".into(),
    installed,
    path,
    executable,
    running,
    default_cdp_port: port,
    cdp_reachable,
    cdp_targets,
  }
}

async fn probe_cdp(port: u16, kind: TargetUrlKind) -> (bool, usize) {
  match wait_for_targets_with(port, 800, kind).await {
    Ok(targets) => (true, targets.len()),
    Err(_) => (false, 0),
  }
}

/// Prefer the `.app` bundle on macOS for display; otherwise return the executable path.
fn display_install_path(exe: &Path) -> PathBuf {
  let mut cur = exe.to_path_buf();
  // Walk up a few levels looking for `Something.app`.
  for _ in 0..5 {
    if cur
      .file_name()
      .and_then(|n| n.to_str())
      .is_some_and(|n| n.ends_with(".app"))
    {
      return cur;
    }
    match cur.parent() {
      Some(parent) if !parent.as_os_str().is_empty() => cur = parent.to_path_buf(),
      _ => break,
    }
  }
  exe.to_path_buf()
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::path::PathBuf;

  #[test]
  fn display_path_prefers_app_bundle() {
    let exe = PathBuf::from("/Applications/ChatGPT.app/Contents/MacOS/ChatGPT");
    assert_eq!(
      display_install_path(&exe),
      PathBuf::from("/Applications/ChatGPT.app")
    );
  }

  #[test]
  fn display_path_keeps_plain_exe() {
    let exe = PathBuf::from(r"C:\Programs\ChatGPT\ChatGPT.exe");
    assert_eq!(display_install_path(&exe), exe);
  }
}
