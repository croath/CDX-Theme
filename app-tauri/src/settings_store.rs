//! Persist app settings under Tauri app data dir.

use crate::injector::DEFAULT_CDP_PORT;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const SETTINGS_FILE: &str = "settings.json";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
  pub cdp_port: u16,
  /// Last successfully applied theme `manifest.id`, if any.
  #[serde(default)]
  pub applied_theme_id: Option<String>,
  /// When true, anonymous product analytics may be sent to PostHog.
  /// Defaults to **false** (opt-in); users enable it in Settings.
  #[serde(default)]
  pub analytics_enabled: bool,
  /// Stable anonymous id for PostHog (`distinct_id`). Generated once per install.
  #[serde(default)]
  pub analytics_distinct_id: Option<String>,
}

impl Default for AppSettings {
  fn default() -> Self {
    Self {
      cdp_port: DEFAULT_CDP_PORT,
      applied_theme_id: None,
      analytics_enabled: false,
      analytics_distinct_id: None,
    }
  }
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
  let dir = app
    .path()
    .app_data_dir()
    .map_err(|e| format!("app data dir: {e}"))?;
  fs::create_dir_all(&dir).map_err(|e| format!("create app data dir: {e}"))?;
  Ok(dir.join(SETTINGS_FILE))
}

pub fn load(app: &AppHandle) -> AppSettings {
  let Ok(path) = settings_path(app) else {
    return AppSettings::default();
  };
  let Ok(raw) = fs::read_to_string(&path) else {
    return AppSettings::default();
  };
  let mut settings: AppSettings = serde_json::from_str(&raw).unwrap_or_default();
  if !is_valid_port(settings.cdp_port) {
    settings.cdp_port = DEFAULT_CDP_PORT;
  }
  settings
}

pub fn save(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
  if !is_valid_port(settings.cdp_port) {
    return Err(format!(
      "invalid CDP port {} (allowed 1024–65535)",
      settings.cdp_port
    ));
  }
  let path = settings_path(app)?;
  let raw = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
  fs::write(&path, raw).map_err(|e| format!("write settings {}: {e}", path.display()))
}

pub fn is_valid_port(port: u16) -> bool {
  (1024..=65535).contains(&port)
}

pub fn set_applied_theme_id(app: &AppHandle, theme_id: Option<String>) -> Result<(), String> {
  let mut settings = load(app);
  settings.applied_theme_id = theme_id;
  save(app, &settings)
}

pub fn applied_theme_id(app: &AppHandle) -> Option<String> {
  load(app).applied_theme_id
}
