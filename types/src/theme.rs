use serde::de::{self, Deserializer, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Parse a package / catalog version into `u32`.
///
/// Accepts plain integers (`1`, `"12"`) or dotted strings (`"1.2.3"` → major `1`).
pub fn parse_version_u32(s: &str) -> Option<u32> {
  let s = s.trim();
  if s.is_empty() {
    return None;
  }
  if let Ok(n) = s.parse::<u32>() {
    return Some(n);
  }
  s.split(|c: char| !c.is_ascii_digit())
    .find(|p| !p.is_empty())
    .and_then(|p| p.parse().ok())
}

/// Serde helper: accept JSON number **or** legacy string versions into `u32`.
///
/// Serializes as a JSON number when used on a `u32` field (default).
pub fn deserialize_version_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
  D: Deserializer<'de>,
{
  struct VersionVisitor;

  impl<'de> Visitor<'de> for VersionVisitor {
    type Value = u32;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
      f.write_str("a non-negative integer or version string")
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<u32, E> {
      u32::try_from(v).map_err(|_| E::custom(format!("version {v} out of u32 range")))
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<u32, E> {
      if v < 0 {
        return Err(E::custom("version must be non-negative"));
      }
      u32::try_from(v as u64).map_err(|_| E::custom(format!("version {v} out of u32 range")))
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<u32, E> {
      parse_version_u32(v).ok_or_else(|| E::custom(format!("invalid theme version {v:?}")))
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<u32, E> {
      self.visit_str(&v)
    }
  }

  deserializer.deserialize_any(VersionVisitor)
}

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
