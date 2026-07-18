//! Portable multi-app theme package format.
//!
//! Supported brands (identical schema):
//! - `cdxtheme` / `.cdxtheme` (CDXTheme default)
//! - `codedrobe-theme` / `.codedrobe-theme` (CodeDrobe-compatible)
//!
//! Shape: `theme` + `targets.{appId}` + optional `assets.images`
//! See https://github.com/CodeDrobe/core/blob/main/src/theme/package.mjs
//!
//! Packages are **plain JSON files**. Load = read + parse into memory
//! (CSS / art stay inline — no extract-to-disk).
//!
//! # App targets
//! Packages may declare multiple host apps under `targets` (e.g. `codex`, `workbuddy`).
//! **Runtime currently only reads and applies `targets.codex`.**
//! `workbuddy` is reserved for a future host adapter.

use crate::util::{is_named_theme, merge_copy};
use cdx_theme_types::{
  BaseTheme, CodexLoadedTarget, CodexTargetOptions, CodexVerification, LoadedArt, LoadedTargets,
  LoadedTheme, WorkBuddyLoadedTarget, WorkBuddyVerification,
};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub const MAX_THEME_PACKAGE_BYTES: u64 = 30 * 1024 * 1024;
pub const THEME_SCHEMA_VERSION: u64 = 1;

pub const FORMAT_CDXTHEME: &str = "cdxtheme";
pub const FORMAT_CODEDROBE: &str = "codedrobe-theme";
pub const EXT_CDXTHEME: &str = "cdxtheme";
pub const EXT_CODEDROBE: &str = "codedrobe-theme";

/// Default store / install extension (CDXTheme brand).
pub const THEME_EXTENSION: &str = EXT_CDXTHEME;

// ── Host app ids (package `targets` keys) ───────────────────────────────────

// Re-export app ids from shared types.
pub use cdx_theme_types::{APP_CODEX, APP_WORKBUDDY};

/// Host app id used for inject + host settings **today**.
/// Switch / parameterize this when WorkBuddy support lands.
pub const ACTIVE_APP_ID: &str = cdx_theme_types::APP_CODEX;

/// Back-compat alias for call sites.
pub const DEFAULT_APP_ID: &str = ACTIVE_APP_ID;

const CODEX_THEME_V1_PROFILE: &str = "codex-theme-v1";

/// Accepted package file extensions.
pub const THEME_PACKAGE_EXTENSIONS: &[&str] = &[EXT_CDXTHEME, EXT_CODEDROBE];

/// Soft cap for list-response preview data URLs (~3MB base64 payload).
const PREVIEW_BASE64_MAX_LEN: usize = 3_500_000;

// ── Package shape ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ThemeMetaIn {
  id: String,
  display_name: String,
  version: String,
  #[serde(default)]
  copy: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImageAssetIn {
  filename: String,
  #[serde(default)]
  mime_type: Option<String>,
  base64: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PackageAssetsIn {
  #[serde(default)]
  images: Option<BTreeMap<String, ImageAssetIn>>,
  #[serde(default)]
  art: Option<ImageAssetIn>,
}

/// Raw package target before mapping to Codex / WorkBuddy structs.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ThemeTargetIn {
  css: String,
  #[serde(default)]
  options: Option<CodexTargetOptions>,
  #[serde(default)]
  verification: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ThemePackageFile {
  format: String,
  schema_version: u64,
  theme: ThemeMetaIn,
  targets: BTreeMap<String, ThemeTargetIn>,
  #[serde(default)]
  assets: Option<PackageAssetsIn>,
}

// ── Public API ──────────────────────────────────────────────────────────────

pub fn is_supported_package_format(format: &str) -> bool {
  matches!(
    format.trim().to_ascii_lowercase().as_str(),
    FORMAT_CDXTHEME | FORMAT_CODEDROBE
  )
}

/// Optional filename hint for file pickers / UI filters (not used for validation).
pub fn is_theme_package_filename(name: &str) -> bool {
  let lower = name.trim().to_ascii_lowercase();
  THEME_PACKAGE_EXTENSIONS
    .iter()
    .any(|ext| lower.ends_with(&format!(".{ext}")))
    || lower.ends_with(".json")
}

/// True if `path` is a file whose **JSON content** is a valid multi-app theme package.
/// Filename / extension is ignored — detection is by deserialize + format/schema checks.
pub fn is_cdx_theme_file(path: &Path) -> bool {
  match fs::metadata(path) {
    Ok(meta) if meta.is_file() && meta.len() <= MAX_THEME_PACKAGE_BYTES => {}
    _ => return false,
  }
  match fs::read_to_string(path) {
    Ok(raw) => is_theme_package_content(&raw),
    Err(_) => false,
  }
}

/// True if `content` deserializes as a valid multi-app theme package
/// (`format` + `schemaVersion` + `theme` + `targets` with the active host).
pub fn is_theme_package_content(content: &str) -> bool {
  parse_bundle_str(content).is_ok()
}

#[derive(Debug, Clone)]
pub struct CodexThemePeek {
  pub id: String,
  pub display_name: String,
  pub version: String,
  pub preview_img: Option<String>,
  pub preview_colors: Vec<String>,
}

/// Peek package metadata for catalog listing (uses **active** app target only).
pub fn peek_codex_theme_meta(path: &Path) -> Result<CodexThemePeek, String> {
  let bundle = read_bundle(path)?;
  // Catalog only lists packages that support the currently active host.
  require_target(&bundle.targets, ACTIVE_APP_ID, path)?;
  let base = resolve_target_base_theme(&bundle.targets, ACTIVE_APP_ID);
  let preview_img = resolve_preview_img(bundle.assets.as_ref(), path);
  Ok(CodexThemePeek {
    id: bundle.theme.id,
    display_name: bundle.theme.display_name,
    version: bundle.theme.version,
    preview_img,
    preview_colors: colors_from_base_theme(base.as_ref()),
  })
}

/// Read + parse a portable theme package into memory.
///
/// CSS and art stay inline on [`LoadedTheme`] — nothing is written under
/// `.extracted/` or elsewhere on disk.
pub fn load_cdx_theme_file(path: &Path) -> Result<LoadedTheme, String> {
  let bundle = read_bundle(path)?;
  validate_bundle(&bundle, path)?;
  // Active host must exist so apply works today.
  require_target(&bundle.targets, ACTIVE_APP_ID, path)?;

  let mut loaded_targets = LoadedTargets::default();

  for (app_id, target) in &bundle.targets {
    if crate::util::css_has_remote_resources(&target.css) {
      return Err(format!(
        "targets.{app_id}.css contains an external resource; only embedded data URLs are supported"
      ));
    }

    match app_id.as_str() {
      APP_CODEX => {
        let mut options = target.options.clone().unwrap_or_default();
        if options.renderer_profile.is_none() {
          options.renderer_profile = Some(CODEX_THEME_V1_PROFILE.into());
        }
        let options = if options.renderer_profile.is_none() && options.base_theme.is_none() {
          None
        } else {
          Some(options)
        };

        let verification = match &target.verification {
          Some(v) => Some(
            serde_json::from_value::<CodexVerification>(v.clone())
              .map_err(|e| format!("targets.codex.verification: {e}"))?,
          ),
          None => None,
        };

        loaded_targets.codex = Some(CodexLoadedTarget {
          css: target.css.clone(),
          options,
          verification,
        });
      }
      APP_WORKBUDDY => {
        let verification = match &target.verification {
          Some(v) => Some(
            serde_json::from_value::<WorkBuddyVerification>(v.clone())
              .map_err(|e| format!("targets.workbuddy.verification: {e}"))?,
          ),
          None => None,
        };

        loaded_targets.workbuddy = Some(WorkBuddyLoadedTarget {
          css: target.css.clone(),
          verification,
        });
      }
      other => {
        tracing::warn!(
          "skipping unsupported package target `{other}` in {}",
          path.display()
        );
      }
    }
  }

  let images = resolve_all_images(bundle.assets.as_ref());
  let art = images.get("hero").cloned();

  Ok(LoadedTheme {
    id: bundle.theme.id,
    display_name: bundle.theme.display_name,
    version: bundle.theme.version,
    copy: merge_copy(bundle.theme.copy.as_ref()),
    images,
    art,
    package_path: path.to_path_buf(),
    targets: loaded_targets,
  })
}

/// Reject `@import` and remote `url(http...)`.
pub use crate::util::css_has_remote_resources;

// ── Internals ───────────────────────────────────────────────────────────────

fn read_bundle(path: &Path) -> Result<ThemePackageFile, String> {
  let meta = fs::metadata(path).map_err(|e| format!("stat {}: {e}", path.display()))?;
  if meta.len() > MAX_THEME_PACKAGE_BYTES {
    return Err(format!(
      "theme package exceeds {}MB limit: {}",
      MAX_THEME_PACKAGE_BYTES / (1024 * 1024),
      path.display()
    ));
  }
  let raw = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
  parse_bundle_str(&raw).map_err(|e| format!("{e} ({})", path.display()))
}

/// Deserialize + structural checks (format, schema, theme, targets, active app).
fn parse_bundle_str(raw: &str) -> Result<ThemePackageFile, String> {
  let bundle: ThemePackageFile =
    serde_json::from_str(raw.trim()).map_err(|e| format!("invalid theme JSON: {e}"))?;
  if !is_supported_package_format(&bundle.format) {
    return Err(format!(
      "unsupported package format {:?} (expected {FORMAT_CDXTHEME} or {FORMAT_CODEDROBE})",
      bundle.format
    ));
  }
  if bundle.schema_version != THEME_SCHEMA_VERSION {
    return Err(format!(
      "unsupported schemaVersion {} (expected {THEME_SCHEMA_VERSION})",
      bundle.schema_version
    ));
  }
  if !is_named_theme(&bundle.theme.id) {
    return Err(format!("invalid theme id: {}", bundle.theme.id));
  }
  if bundle.theme.display_name.trim().is_empty() {
    return Err("theme.displayName must be a non-empty string".into());
  }
  if bundle.theme.version.trim().is_empty() {
    return Err("theme.version must be a non-empty string".into());
  }
  if bundle.targets.is_empty() {
    return Err("theme package must support at least one app target".into());
  }
  if !bundle.targets.contains_key(ACTIVE_APP_ID) {
    return Err(format!(
      "theme package missing required target `{ACTIVE_APP_ID}` (found: {})",
      bundle
        .targets
        .keys()
        .cloned()
        .collect::<Vec<_>>()
        .join(", ")
    ));
  }
  for (app_id, target) in &bundle.targets {
    if !is_named_theme(app_id) {
      return Err(format!("invalid target app id: {app_id}"));
    }
    if target.css.trim().is_empty() {
      return Err(format!("targets.{app_id}.css must be non-empty"));
    }
  }
  Ok(bundle)
}

fn validate_bundle(bundle: &ThemePackageFile, path: &Path) -> Result<(), String> {
  // Structural checks already ran in parse_bundle_str; re-check active target only.
  if !bundle.targets.contains_key(ACTIVE_APP_ID) {
    return Err(format!(
      "theme package missing required target `{ACTIVE_APP_ID}` in {}",
      path.display()
    ));
  }
  Ok(())
}

/// Resolve a specific app target. No fallback to other targets.
fn require_target<'a>(
  targets: &'a BTreeMap<String, ThemeTargetIn>,
  app_id: &str,
  path: &Path,
) -> Result<&'a ThemeTargetIn, String> {
  targets.get(app_id).ok_or_else(|| {
    let found = targets.keys().cloned().collect::<Vec<_>>().join(", ");
    format!(
      "theme '{}' does not support app '{app_id}' (targets: {found})",
      path.display()
    )
  })
}

fn resolve_target_base_theme(
  targets: &BTreeMap<String, ThemeTargetIn>,
  app_id: &str,
) -> Option<BaseTheme> {
  targets
    .get(app_id)
    .and_then(|t| t.options.as_ref())
    .and_then(|o| o.base_theme.clone())
}

fn resolved_hero(assets: Option<&PackageAssetsIn>) -> Option<ImageAssetIn> {
  let assets = assets?;
  if let Some(images) = &assets.images {
    if let Some(hero) = images.get("hero") {
      return Some(ImageAssetIn {
        filename: hero.filename.clone(),
        mime_type: hero.mime_type.clone(),
        base64: hero.base64.clone(),
      });
    }
  }
  assets.art.as_ref().map(|a| ImageAssetIn {
    filename: a.filename.clone(),
    mime_type: a.mime_type.clone(),
    base64: a.base64.clone(),
  })
}

/// Load every `assets.images.*` entry (and legacy `assets.art` as `hero`).
/// Inject maps each name to `--cdxtheme-image-{name}`.
fn resolve_all_images(assets: Option<&PackageAssetsIn>) -> BTreeMap<String, LoadedArt> {
  let mut out = BTreeMap::new();
  let Some(assets) = assets else {
    return out;
  };

  if let Some(images) = &assets.images {
    for (name, img) in images {
      if !is_named_theme(name) {
        tracing::warn!("skipping invalid package image id `{name}`");
        continue;
      }
      if let Some(loaded) = image_asset_to_loaded(img) {
        out.insert(name.clone(), loaded);
      }
    }
  }

  // Legacy single-art field becomes `hero` when not already present.
  if !out.contains_key("hero") {
    if let Some(art) = &assets.art {
      if let Some(loaded) = image_asset_to_loaded(art) {
        out.insert("hero".into(), loaded);
      }
    }
  }

  out
}

fn image_asset_to_loaded(img: &ImageAssetIn) -> Option<LoadedArt> {
  let b64 = img.base64.trim();
  if b64.is_empty() {
    return None;
  }
  let mime = img
    .mime_type
    .as_deref()
    .filter(|s| !s.is_empty())
    .unwrap_or_else(|| mime_from_filename(&img.filename))
    .to_string();
  // Accept either raw base64 or a full data URL from the package.
  let base64 = if let Some(rest) = b64.strip_prefix("data:") {
    rest
      .split_once("base64,")
      .map(|(_, payload)| payload.to_string())
      .unwrap_or_else(|| b64.to_string())
  } else {
    b64.to_string()
  };
  if base64.is_empty() {
    return None;
  }
  Some(LoadedArt {
    mime_type: mime,
    base64,
  })
}

fn resolve_preview_img(assets: Option<&PackageAssetsIn>, path: &Path) -> Option<String> {
  let hero = resolved_hero(assets)?;
  let b64 = hero.base64.trim();
  if b64.is_empty() {
    return None;
  }
  let mime = hero
    .mime_type
    .as_deref()
    .filter(|s| !s.is_empty())
    .unwrap_or_else(|| mime_from_filename(&hero.filename));
  let url = if b64.starts_with("data:image/") {
    b64.to_string()
  } else {
    format!("data:{mime};base64,{b64}")
  };
  clamp_preview_data_url(&url, path)
}

fn colors_from_base_theme(base: Option<&BaseTheme>) -> Vec<String> {
  let accent = base.and_then(|b| b.accent.as_deref()).unwrap_or("#84CC16");
  let surface = base.and_then(|b| b.surface.as_deref()).unwrap_or("#F7FEE7");
  let ink = base.and_then(|b| b.ink.as_deref()).unwrap_or("#1A2E05");
  vec![accent.into(), surface.into(), ink.into()]
}

fn clamp_preview_data_url(url: &str, path: &Path) -> Option<String> {
  let payload_len = url
    .split_once("base64,")
    .map(|(_, b)| b.len())
    .unwrap_or(url.len());
  if payload_len > PREVIEW_BASE64_MAX_LEN {
    tracing::warn!(
      "preview art too large ({} base64 chars) in {}, using color fallback",
      payload_len,
      path.display()
    );
    return None;
  }
  Some(url.to_string())
}

fn mime_from_filename(filename: &str) -> &'static str {
  let name = filename.to_ascii_lowercase();
  if name.ends_with(".jpg") || name.ends_with(".jpeg") {
    "image/jpeg"
  } else if name.ends_with(".webp") {
    "image/webp"
  } else if name.ends_with(".gif") {
    "image/gif"
  } else {
    "image/png"
  }
}
