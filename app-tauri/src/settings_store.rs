//! Persist app settings under Tauri app data dir.

use crate::injector::{APP_CODEX, APP_WORKBUDDY, DEFAULT_CDP_PORT, DEFAULT_WORKBUDDY_CDP_PORT};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const SETTINGS_FILE: &str = "settings.json";

/// Per-host last successfully applied theme id.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppliedThemes {
  #[serde(default)]
  pub codex: Option<String>,
  #[serde(default)]
  pub workbuddy: Option<String>,
}

impl AppliedThemes {
  pub fn get(&self, host: &str) -> Option<&str> {
    match host.trim().to_ascii_lowercase().as_str() {
      "workbuddy" | "work-buddy" | "wb" => self.workbuddy.as_deref(),
      _ => self.codex.as_deref(),
    }
  }

  pub fn set(&mut self, host: &str, theme_id: Option<String>) {
    match host.trim().to_ascii_lowercase().as_str() {
      "workbuddy" | "work-buddy" | "wb" => self.workbuddy = theme_id,
      _ => self.codex = theme_id,
    }
  }

  pub fn clear_all(&mut self) {
    self.codex = None;
    self.workbuddy = None;
  }

  /// Any applied theme id (prefer codex, then workbuddy) — UI back-compat.
  pub fn any_theme_id(&self) -> Option<&str> {
    self.codex.as_deref().or(self.workbuddy.as_deref())
  }

  pub fn contains_theme(&self, theme_id: &str) -> bool {
    self.codex.as_deref() == Some(theme_id) || self.workbuddy.as_deref() == Some(theme_id)
  }

  pub fn is_empty(&self) -> bool {
    self.codex.is_none() && self.workbuddy.is_none()
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
  pub cdp_port: u16,
  /// WorkBuddy desktop remote-debugging port (default 9336).
  #[serde(default = "default_workbuddy_cdp_port")]
  pub workbuddy_cdp_port: u16,
  /// Legacy single applied theme id (pre multi-host). Migrated into [`Self::applied_themes`].
  #[serde(default)]
  pub applied_theme_id: Option<String>,
  /// Last applied theme id per host app (`codex` / `workbuddy`).
  #[serde(default)]
  pub applied_themes: AppliedThemes,
  /// When true, anonymous product analytics may be sent to PostHog.
  /// Defaults to **true** (opt-out); users can disable it in Settings.
  #[serde(default = "default_analytics_enabled")]
  pub analytics_enabled: bool,
  /// Stable anonymous id for PostHog (`distinct_id`). Generated once per install.
  #[serde(default)]
  pub analytics_distinct_id: Option<String>,
}

fn default_analytics_enabled() -> bool {
  true
}

fn default_workbuddy_cdp_port() -> u16 {
  DEFAULT_WORKBUDDY_CDP_PORT
}

impl Default for AppSettings {
  fn default() -> Self {
    Self {
      cdp_port: DEFAULT_CDP_PORT,
      workbuddy_cdp_port: DEFAULT_WORKBUDDY_CDP_PORT,
      applied_theme_id: None,
      applied_themes: AppliedThemes::default(),
      analytics_enabled: true,
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

/// Normalize legacy `applied_theme_id` into `applied_themes` and keep both in sync.
fn migrate_applied(settings: &mut AppSettings) {
  if settings.applied_themes.is_empty()
    && let Some(id) = settings
      .applied_theme_id
      .clone()
      .filter(|s| !s.trim().is_empty())
  {
    // Pre multi-host: treat as Codex apply.
    settings.applied_themes.codex = Some(id);
  }
  // Keep legacy field as "any" for older readers of settings.json.
  settings.applied_theme_id = settings
    .applied_themes
    .any_theme_id()
    .map(|s| s.to_string());
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
  if !is_valid_port(settings.workbuddy_cdp_port) {
    settings.workbuddy_cdp_port = DEFAULT_WORKBUDDY_CDP_PORT;
  }
  migrate_applied(&mut settings);
  settings
}

pub fn save(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
  if !is_valid_port(settings.cdp_port) {
    return Err(format!(
      "invalid CDP port {} (allowed 1024–65535)",
      settings.cdp_port
    ));
  }
  if !is_valid_port(settings.workbuddy_cdp_port) {
    return Err(format!(
      "invalid WorkBuddy CDP port {} (allowed 1024–65535)",
      settings.workbuddy_cdp_port
    ));
  }
  let mut to_write = settings.clone();
  migrate_applied(&mut to_write);
  let path = settings_path(app)?;
  let raw = serde_json::to_string_pretty(&to_write).map_err(|e| e.to_string())?;
  fs::write(&path, raw).map_err(|e| format!("write settings {}: {e}", path.display()))
}

pub fn is_valid_port(port: u16) -> bool {
  (1024..=65535).contains(&port)
}

/// Record (or clear) the applied theme for a specific host (`codex` / `workbuddy`).
pub fn set_applied_theme(
  app: &AppHandle,
  host: &str,
  theme_id: Option<String>,
) -> Result<(), String> {
  let mut settings = load(app);
  settings.applied_themes.set(host, theme_id);
  settings.applied_theme_id = settings
    .applied_themes
    .any_theme_id()
    .map(|s| s.to_string());
  save(app, &settings)
}

/// Clear applied markers for all hosts.
pub fn clear_applied_themes(app: &AppHandle) -> Result<(), String> {
  let mut settings = load(app);
  settings.applied_themes.clear_all();
  settings.applied_theme_id = None;
  save(app, &settings)
}

/// Applied theme id for a host app.
pub fn applied_theme_for(app: &AppHandle, host: &str) -> Option<String> {
  load(app).applied_themes.get(host).map(|s| s.to_string())
}

/// Full per-host applied map.
pub fn applied_themes(app: &AppHandle) -> AppliedThemes {
  load(app).applied_themes
}

/// Legacy: any applied theme id (Codex preferred). Prefer [`applied_theme_for`].
pub fn set_applied_theme_id(app: &AppHandle, theme_id: Option<String>) -> Result<(), String> {
  // Back-compat callers treated this as the global applied theme → Codex.
  set_applied_theme(app, APP_CODEX, theme_id)
}

pub fn applied_theme_id(app: &AppHandle) -> Option<String> {
  load(app)
    .applied_themes
    .any_theme_id()
    .map(|s| s.to_string())
}

/// Whether `theme_id` is currently applied on any host.
pub fn theme_is_applied(app: &AppHandle, theme_id: &str) -> bool {
  load(app).applied_themes.contains_theme(theme_id)
}

#[allow(dead_code)]
pub fn host_ids() -> [&'static str; 2] {
  [APP_CODEX, APP_WORKBUDDY]
}
