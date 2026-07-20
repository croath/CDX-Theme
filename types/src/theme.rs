use serde::{Deserialize, Serialize};

/// Where a catalog theme came from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThemeSource {
  /// Bundled with the app under resource `themes/`.
  #[default]
  Builtin,
  /// User-installed / downloaded package under local data `themes/`.
  Installed,
  /// Listed in the remote recommend catalog (not yet on disk).
  Remote,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeMetadata {
  /// Theme package id from `manifest.id` / remote `theme_id`.
  pub id: String,
  pub name: String,
  /// Absolute path to the local package file (empty for remote-only entries).
  #[serde(default)]
  pub location: String,
  /// Preview image: local `data:` URL preferred (remote HTTP(S) is disk-cached by the app).
  pub preview_img: Option<String>,
  /// Colors derived from target `baseTheme` (accent, surface, ink) for gradient fallback.
  #[serde(default)]
  pub preview_colors: Vec<String>,
  /// Whether this theme is the currently recorded applied theme.
  #[serde(default)]
  pub is_applied: bool,
  /// Catalog source — used for UI tags.
  #[serde(default)]
  pub source: ThemeSource,
  /// Local installed / package version as `u32` when known.
  #[serde(default)]
  pub version: Option<u32>,
  /// Remote catalog version (`u32`) when known from the recommend index.
  #[serde(default)]
  pub remote_version: Option<u32>,
  /// True when `remote_version` is greater than installed `version`.
  #[serde(default)]
  pub update_available: bool,
  /// Remote package URL (recommend catalog). Used to download before apply.
  #[serde(default)]
  pub theme_url: Option<String>,
}
