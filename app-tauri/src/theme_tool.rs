//! Theme tool operations (ported from `scripts/theme-tool.mjs`).
//! `apply` / `restore` mutate Codex `~/.codex/config.toml`.

use crate::injector::theme::load_theme_package;
use crate::theme_lib::{
  MANAGED_SETTINGS_KEYS, appearance_theme_changed, apply_settings, build_base_theme_settings,
  restore_settings,
};
use cdx_theme_types::LoadedTheme;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

/// Default Codex config location (`~/.codex/config.toml` / `%USERPROFILE%\.codex\config.toml`).
pub fn codex_config_path() -> PathBuf {
  crate::paths::user_home_dir()
    .map(|h| h.join(".codex").join("config.toml"))
    .unwrap_or_else(|| PathBuf::from(".codex").join("config.toml"))
}

/// App state root — Tauri app data directory (`app_data_dir`).
pub fn state_root(app: &AppHandle) -> Result<PathBuf, String> {
  let dir = app
    .path()
    .local_data_dir()
    .map_err(|e| format!("failed to resolve app data dir: {e}"))?;
  fs::create_dir_all(&dir)
    .map_err(|e| format!("failed to create app data dir {}: {e}", dir.display()))?;
  Ok(dir)
}

pub fn config_backup_path(app: &AppHandle) -> Result<PathBuf, String> {
  Ok(state_root(app)?.join("config.before.toml"))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyResult {
  /// False when the package has no `targets.codex.options.baseTheme`
  /// (CodeDrobe `applyCodexBaseTheme` no-op).
  pub applied: bool,
  /// True when `[desktop].appearanceTheme` (light/dark) changed.
  /// Only this key requires a ChatGPT restart; chrome/code theme can update live.
  pub appearance_changed: bool,
  /// True when any managed config lines were written (even if mode did not change).
  pub config_changed: bool,
  pub theme: String,
  pub config: String,
  pub backup: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResult {
  pub restored: bool,
  /// True when config content changed vs pre-restore.
  pub appearance_changed: bool,
  pub config: String,
  pub backup: String,
}

/// `theme-tool apply` — backup config once, then write appearance settings from the theme package.
pub fn apply(app: &AppHandle, theme_package: impl AsRef<Path>) -> Result<ApplyResult, String> {
  let theme = load_theme_package(theme_package)?;
  apply_loaded(&theme, &codex_config_path(), &config_backup_path(app)?)
}

/// Port of CodeDrobe [`applyCodexBaseTheme`](https://github.com/CodeDrobe/core/blob/main/src/host/codex-settings.mjs):
///
/// ```text
/// const baseTheme = targetTheme?.options?.baseTheme;
/// if (!baseTheme) return { applied: false, changed: false, restartRequired: false };
/// // backup once (COPYFILE_EXCL), applyCodexSettings, write if changed
/// // restartRequired = updated !== before
/// ```
pub fn apply_loaded(
  theme: &LoadedTheme,
  config_path: &Path,
  backup_path: &Path,
) -> Result<ApplyResult, String> {
  // CodeDrobe L165–166: no baseTheme → skip host settings (CSS inject may still run).
  if theme.active_base_theme().is_none() {
    return Ok(ApplyResult {
      applied: false,
      appearance_changed: false,
      config_changed: false,
      theme: theme.id.clone(),
      config: config_path.display().to_string(),
      backup: backup_path.display().to_string(),
    });
  }

  if !config_path.is_file() {
    return Err(format!("Codex config not found: {}", config_path.display()));
  }

  if let Some(parent) = backup_path.parent() {
    fs::create_dir_all(parent)
      .map_err(|e| format!("failed to create state dir {}: {e}", parent.display()))?;
  }

  // First-apply only backup (never overwrite — CodeDrobe COPYFILE_EXCL).
  if !backup_path.exists() {
    fs::copy(config_path, backup_path)
      .map_err(|e| format!("failed to backup config to {}: {e}", backup_path.display()))?;
  }

  let content = fs::read_to_string(config_path)
    .map_err(|e| format!("failed to read config {}: {e}", config_path.display()))?;
  let settings = build_base_theme_settings(theme, crate::theme_package::ACTIVE_APP_ID);
  let updated = apply_settings(&content, &settings);
  // Always persist managed keys when they changed (chrome, code theme, mode, …).
  let config_changed = content != updated;
  if config_changed {
    fs::write(config_path, &updated)
      .map_err(|e| format!("failed to write config {}: {e}", config_path.display()))?;
  }
  // Restart only when light/dark mode (`appearanceTheme`) actually changed.
  let appearance_changed = appearance_theme_changed(&content, &updated);

  Ok(ApplyResult {
    applied: true,
    appearance_changed,
    config_changed,
    theme: theme.id.clone(),
    config: config_path.display().to_string(),
    backup: backup_path.display().to_string(),
  })
}

/// `theme-tool restore` — restore managed appearance keys from backup into config.
pub fn restore(app: &AppHandle) -> Result<RestoreResult, String> {
  restore_paths(&codex_config_path(), &config_backup_path(app)?)
}

pub fn restore_paths(config_path: &Path, backup_path: &Path) -> Result<RestoreResult, String> {
  if !config_path.is_file() {
    return Err(format!("Codex config not found: {}", config_path.display()));
  }
  if !backup_path.is_file() {
    return Err(format!(
      "No pre-install config backup is available: {}",
      backup_path.display()
    ));
  }

  let current = fs::read_to_string(config_path)
    .map_err(|e| format!("failed to read config {}: {e}", config_path.display()))?;
  let backup = fs::read_to_string(backup_path)
    .map_err(|e| format!("failed to read backup {}: {e}", backup_path.display()))?;
  let updated = restore_settings(&current, &backup, MANAGED_SETTINGS_KEYS);
  // CodeDrobe: changed = restored !== current
  let appearance_changed = current != updated;
  if appearance_changed {
    fs::write(config_path, &updated)
      .map_err(|e| format!("failed to write config {}: {e}", config_path.display()))?;
  }

  Ok(RestoreResult {
    restored: true,
    appearance_changed,
    config: config_path.display().to_string(),
    backup: backup_path.display().to_string(),
  })
}
