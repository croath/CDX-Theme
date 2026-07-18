//! Pack / unpack portable multi-app theme packages.
//!
//! Supported brands (same schema):
//! - `cdxtheme` / `.cdxtheme` (default for this project)
//! - `codedrobe-theme` / `.codedrobe-theme` (CodeDrobe-compatible)
//!

use crate::error::{CoreError, Result};
use base64::Engine;
use chrono::Utc;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub const FORMAT_CDXTHEME: &str = "cdxtheme";
pub const FORMAT_CODEDROBE: &str = "codedrobe-theme";
pub const EXT_CDXTHEME: &str = "cdxtheme";
pub const EXT_CODEDROBE: &str = "codedrobe-theme";

pub const THEME_SCHEMA_VERSION: u64 = 1;
pub const MAX_THEME_PACKAGE_BYTES: u64 = 30 * 1024 * 1024;
pub const MAX_THEME_IMAGES: usize = 32;

const SAFE_IMAGE_TYPES: &[&str] = &["image/png", "image/jpeg", "image/webp", "image/gif"];

/// Package brand: same multi-app JSON schema, different `format` / file extension.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PackageFormat {
  /// CDXTheme brand (default): `format: "cdxtheme"`, extension `.cdxtheme`.
  #[default]
  Cdxtheme,
  /// CodeDrobe brand: `format: "codedrobe-theme"`, extension `.codedrobe-theme`.

  CodedrobeTheme,
}

impl PackageFormat {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Cdxtheme => FORMAT_CDXTHEME,
      Self::CodedrobeTheme => FORMAT_CODEDROBE,
    }
  }

  pub fn extension(self) -> &'static str {
    match self {
      Self::Cdxtheme => EXT_CDXTHEME,
      Self::CodedrobeTheme => EXT_CODEDROBE,
    }
  }

  pub fn parse(format: &str) -> Option<Self> {
    match format.trim().to_ascii_lowercase().as_str() {
      FORMAT_CDXTHEME => Some(Self::Cdxtheme),
      FORMAT_CODEDROBE => Some(Self::CodedrobeTheme),
      _ => None,
    }
  }
}

pub fn is_supported_package_format(format: &str) -> bool {
  PackageFormat::parse(format).is_some()
}

// ── Portable package ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeMeta {
  pub id: String,
  pub display_name: String,
  pub version: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub copy: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageAsset {
  pub filename: String,
  pub mime_type: String,
  pub base64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageAssets {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub images: Option<BTreeMap<String, ImageAsset>>,
  /// Alias for `images.hero` (CodeDrobe allows either, not both).
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub art: Option<ImageAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeTarget {
  pub css: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub options: Option<Value>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub verification: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemePackage {
  pub format: String,
  pub schema_version: u64,
  #[serde(default)]
  pub exported_at: String,
  pub theme: ThemeMeta,
  pub targets: BTreeMap<String, ThemeTarget>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub assets: Option<PackageAssets>,
}

impl ThemePackage {
  pub fn package_filename(&self) -> String {
    let ext = PackageFormat::parse(&self.format)
      .unwrap_or_default()
      .extension();
    format!("{}-{}.{}", self.theme.id, self.theme.version, ext)
  }
}

// ── Pack ────────────────────────────────────────────────────────────────────

/// Preferred source filenames inside a theme directory (first match wins).
pub const SOURCE_FILENAMES: &[&str] = &["theme.json", "manifest.json"];

/// Pack a theme directory (or path to `theme.json` / `manifest.json`) into a portable package.
///
/// Target CSS is automatically rewritten: `codedrobe-` → `cdxtheme-`.
pub fn pack_theme_dir(
  theme_dir_or_manifest: &Path,
  output: Option<&Path>,
  format: PackageFormat,
  pretty: bool,
  force: bool,
) -> Result<(PathBuf, u64)> {
  let (base, source_path) = resolve_source_paths(theme_dir_or_manifest)?;
  let package = build_package(&base, &source_path, format)?;
  let out = match output {
    Some(p) => {
      if p.is_dir() {
        p.join(package.package_filename())
      } else {
        let mut path = p.to_path_buf();
        if path.extension().is_none() {
          path.set_extension(format.extension());
        }
        path
      }
    }
    None => PathBuf::from(package.package_filename()),
  };

  if !force && out.is_file() {
    return Err(CoreError::msg(format!(
      "output already exists: {} (pass --force to overwrite)",
      out.display()
    )));
  }

  if let Some(parent) = out.parent() {
    if !parent.as_os_str().is_empty() {
      fs::create_dir_all(parent)?;
    }
  }

  let bytes = write_package(&package, &out, pretty)?;
  Ok((out, bytes))
}

/// Resolve theme base directory and source JSON path.
///
/// - Directory input: prefer `theme.json`, fall back to `manifest.json`.
/// - File input: use that file (any path ending in those names or a custom JSON).
fn resolve_source_paths(input: &Path) -> Result<(PathBuf, PathBuf)> {
  if input.is_file() {
    let base = input
      .parent()
      .ok_or_else(|| CoreError::msg("theme source path has no parent"))?
      .to_path_buf();
    return Ok((base, input.to_path_buf()));
  }
  if input.is_dir() {
    for name in SOURCE_FILENAMES {
      let candidate = input.join(name);
      if candidate.is_file() {
        return Ok((input.to_path_buf(), candidate));
      }
    }
    return Err(CoreError::msg(format!(
      "neither theme.json nor manifest.json found in {}",
      input.display()
    )));
  }
  Err(CoreError::msg(format!(
    "theme path not found: {}",
    input.display()
  )))
}

pub fn build_package(
  base: &Path,
  source_path: &Path,
  format: PackageFormat,
) -> Result<ThemePackage> {
  let raw = fs::read_to_string(source_path)?;
  let source: Value = serde_json::from_str(&raw)
    .map_err(|e| CoreError::msg(format!("failed to parse {}: {e}", source_path.display())))?;
  let obj = source.as_object().ok_or_else(|| {
    CoreError::msg(format!(
      "{} must be a JSON object",
      source_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("theme source")
    ))
  })?;

  let schema = obj
    .get("schemaVersion")
    .or_else(|| obj.get("schema_version"))
    .and_then(|v| v.as_u64())
    .unwrap_or(THEME_SCHEMA_VERSION);
  if schema != THEME_SCHEMA_VERSION {
    return Err(CoreError::msg(format!(
      "unsupported source manifest schemaVersion (expected {THEME_SCHEMA_VERSION})"
    )));
  }

  let id = require_str(obj, "id")?.to_string();
  let display_name = require_str(obj, "displayName")
    .or_else(|_| require_str(obj, "display_name"))?
    .to_string();
  let version = require_str(obj, "version")?.to_string();
  if !is_named_theme(&id) {
    return Err(CoreError::msg(format!(
      "invalid theme id `{id}` (use alphanumeric, `_`, `-`)"
    )));
  }
  if display_name.trim().is_empty() || version.trim().is_empty() {
    return Err(CoreError::msg(
      "manifest displayName and version must be non-empty",
    ));
  }

  let copy = obj.get("copy").cloned().filter(|v| !v.is_null());

  let targets_val = obj
    .get("targets")
    .ok_or_else(|| {
      CoreError::msg(
        "manifest requires a targets object (multi-app source format; see cli/README.md)",
      )
    })?
    .clone();
  let targets_map = targets_val
    .as_object()
    .ok_or_else(|| CoreError::msg("manifest.targets must be an object"))?;
  if targets_map.is_empty() {
    return Err(CoreError::msg(
      "manifest.targets must support at least one app target",
    ));
  }

  let mut targets = BTreeMap::new();
  for (app_id, target_val) in targets_map {
    if !is_named_theme(app_id) {
      return Err(CoreError::msg(format!("invalid target app id `{app_id}`")));
    }
    let t = target_val
      .as_object()
      .ok_or_else(|| CoreError::msg(format!("targets.{app_id} must be an object")))?;
    let css_rel = t
      .get("css")
      .and_then(|v| v.as_str())
      .map(str::trim)
      .filter(|s| !s.is_empty())
      .ok_or_else(|| CoreError::msg(format!("targets.{app_id}.css must be a non-empty string")))?;
    let css_path = base.join(css_rel);
    if !css_path.is_file() {
      return Err(CoreError::msg(format!(
        "theme css not found for target `{app_id}`: {}",
        css_path.display()
      )));
    }
    let css = fs::read_to_string(&css_path)?;
    if css.trim().is_empty() {
      return Err(CoreError::msg(format!(
        "theme css is empty for target `{app_id}`"
      )));
    }
    if css_has_remote_resources(&css) {
      return Err(CoreError::msg(format!(
        "target `{app_id}` contains an external CSS resource; only embedded data URLs are supported"
      )));
    }
    // Always normalize legacy CodeDrobe tokens → CDXTheme (`codedrobe-` → `cdxtheme-`).
    let css = rewrite_css_codedrobe_to_cdxtheme(&css);
    if css.trim().is_empty() {
      return Err(CoreError::msg(format!(
        "theme css became empty after brand rewrite for target `{app_id}`"
      )));
    }
    let options = t.get("options").cloned().filter(|v| v.is_object());
    let verification = t.get("verification").cloned().filter(|v| v.is_object());
    targets.insert(
      app_id.clone(),
      ThemeTarget {
        css,
        options,
        verification,
      },
    );
  }

  // images: { hero: "assets/art.png" } and/or art: "assets/art.png"
  if obj.get("art").is_some() && obj.get("images").and_then(|i| i.get("hero")).is_some() {
    return Err(CoreError::msg(
      "source manifest art cannot be combined with images.hero",
    ));
  }

  let mut source_images: BTreeMap<String, String> = BTreeMap::new();
  if let Some(art) = obj
    .get("art")
    .and_then(|v| v.as_str())
    .map(str::trim)
    .filter(|s| !s.is_empty() && *s != "null")
  {
    source_images.insert("hero".into(), art.to_string());
  }
  if let Some(images) = obj.get("images").and_then(|v| v.as_object()) {
    for (name, path_val) in images {
      if !is_named_theme(name) {
        return Err(CoreError::msg(format!(
          "source manifest contains invalid image id `{name}`"
        )));
      }
      let rel = path_val
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| CoreError::msg(format!("images.{name} must be a path string")))?;
      source_images.insert(name.clone(), rel.to_string());
    }
  }
  if source_images.len() > MAX_THEME_IMAGES {
    return Err(CoreError::msg(format!(
      "source manifest images exceeds {MAX_THEME_IMAGES} entries"
    )));
  }

  let mut images = BTreeMap::new();
  for (name, rel) in &source_images {
    let image_path = base.join(rel);
    if !image_path.is_file() {
      return Err(CoreError::msg(format!(
        "image `{name}` not found: {}",
        image_path.display()
      )));
    }
    let mime = mime_type_for(&image_path)
      .ok_or_else(|| CoreError::msg(format!("images.{name} uses an unsupported image file type")))?;
    let bytes = fs::read(&image_path)?;
    let filename = safe_asset_name(
      image_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("image.png"),
    );
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    images.insert(
      name.clone(),
      ImageAsset {
        filename,
        mime_type: mime.to_string(),
        base64: b64,
      },
    );
  }

  let assets = if images.is_empty() {
    None
  } else {
    Some(PackageAssets {
      images: Some(images),
      art: None,
    })
  };

  let package = ThemePackage {
    format: format.as_str().into(),
    schema_version: THEME_SCHEMA_VERSION,
    exported_at: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
    theme: ThemeMeta {
      id,
      display_name,
      version,
      copy,
    },
    targets,
    assets,
  };
  validate_package(&package)?;
  Ok(package)
}

pub fn write_package(package: &ThemePackage, out: &Path, pretty: bool) -> Result<u64> {
  let serialized = if pretty {
    serde_json::to_string_pretty(package)? + "\n"
  } else {
    serde_json::to_string(package)? + "\n"
  };
  let len = serialized.len() as u64;
  if len > MAX_THEME_PACKAGE_BYTES {
    return Err(CoreError::msg(format!(
      "package exceeds {}MB limit ({} bytes)",
      MAX_THEME_PACKAGE_BYTES / (1024 * 1024),
      len
    )));
  }
  fs::write(out, serialized)?;
  Ok(len)
}

// ── Convert ─────────────────────────────────────────────────────────────────

/// Convert a portable package to CDXTheme brand (`.cdxtheme`).
///
/// - Sets `format` to `cdxtheme`
/// - Rewrites every `codedrobe-` token in target CSS to `cdxtheme-`
///   (classes, ids, custom properties, etc.)
///
/// Accepts either brand as input. Output defaults to `{id}-{version}.cdxtheme`.
pub fn convert_package(
  package_path: &Path,
  output: Option<&Path>,
  pretty: bool,
  force: bool,
) -> Result<(PathBuf, u64)> {
  let mut package = read_package(package_path)?;
  validate_package(&package)?;

  for (app_id, target) in package.targets.iter_mut() {
    let rewritten = rewrite_css_codedrobe_to_cdxtheme(&target.css);
    if rewritten.trim().is_empty() {
      return Err(CoreError::msg(format!(
        "target `{app_id}` CSS became empty after brand rewrite"
      )));
    }
    target.css = rewritten;
  }

  package.format = FORMAT_CDXTHEME.into();
  package.exported_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
  validate_package(&package)?;

  let out = match output {
    Some(p) => {
      if p.is_dir() {
        p.join(package.package_filename())
      } else {
        let mut path = p.to_path_buf();
        if path.extension().is_none() {
          path.set_extension(EXT_CDXTHEME);
        }
        path
      }
    }
    None => PathBuf::from(package.package_filename()),
  };

  if !force && out.is_file() {
    return Err(CoreError::msg(format!(
      "output already exists: {} (pass --force to overwrite)",
      out.display()
    )));
  }

  if let Some(parent) = out.parent() {
    if !parent.as_os_str().is_empty() {
      fs::create_dir_all(parent)?;
    }
  }

  let bytes = write_package(&package, &out, pretty)?;
  Ok((out, bytes))
}

/// Rewrite CodeDrobe CSS brand tokens to CDXTheme: every `codedrobe-` → `cdxtheme-`.
///
/// Covers selectors (`.codedrobe-…`, `#codedrobe-…`, `html.codedrobe-…`),
/// custom properties (`--codedrobe-image-*`, `--codedrobe-art`), and related ids.
pub fn rewrite_css_codedrobe_to_cdxtheme(css: &str) -> String {
  css.replace("codedrobe-", "cdxtheme-")
}

// ── Unpack ──────────────────────────────────────────────────────────────────

/// Unpack a `.cdxtheme` / `.codedrobe-theme` package into a source theme directory.
pub fn unpack_package(package_path: &Path, output_dir: &Path) -> Result<PathBuf> {
  let package = read_package(package_path)?;
  validate_package(&package)?;

  fs::create_dir_all(output_dir)?;

  let mut source = Map::new();
  source.insert("schemaVersion".into(), Value::from(THEME_SCHEMA_VERSION));
  source.insert("id".into(), Value::String(package.theme.id.clone()));
  source.insert(
    "displayName".into(),
    Value::String(package.theme.display_name.clone()),
  );
  source.insert(
    "version".into(),
    Value::String(package.theme.version.clone()),
  );
  if let Some(copy) = &package.theme.copy {
    source.insert("copy".into(), copy.clone());
  }

  let mut targets_out = Map::new();
  for (app_id, target) in &package.targets {
    let css_rel = format!("{app_id}/theme.css");
    let css_path = output_dir.join(&css_rel);
    if let Some(parent) = css_path.parent() {
      fs::create_dir_all(parent)?;
    }
    fs::write(&css_path, &target.css)?;

    let mut t = Map::new();
    t.insert("css".into(), Value::String(css_rel));
    if let Some(options) = &target.options {
      t.insert("options".into(), options.clone());
    }
    if let Some(verification) = &target.verification {
      t.insert("verification".into(), verification.clone());
    }
    targets_out.insert(app_id.clone(), Value::Object(t));
  }
  source.insert("targets".into(), Value::Object(targets_out));

  let image_assets = resolved_image_assets(&package);
  if !image_assets.is_empty() {
    let mut images_out = Map::new();
    for (name, image) in &image_assets {
      let filename = safe_asset_name(&image.filename);
      let rel = format!("images/{name}-{filename}");
      let art_path = output_dir.join(safe_rel_path(&rel)?);
      if let Some(parent) = art_path.parent() {
        fs::create_dir_all(parent)?;
      }
      let bytes = base64::engine::general_purpose::STANDARD
        .decode(image.base64.trim())
        .map_err(|e| CoreError::msg(format!("decode assets.images.{name} base64: {e}")))?;
      fs::write(&art_path, bytes)?;
      images_out.insert(name.clone(), Value::String(rel));
    }
    source.insert("images".into(), Value::Object(images_out));
  }

  // Prefer theme.json so a subsequent pack picks it up first.
  let source_path = output_dir.join("theme.json");
  fs::write(
    &source_path,
    serde_json::to_string_pretty(&Value::Object(source))? + "\n",
  )?;

  Ok(output_dir.to_path_buf())
}

/// Read and parse a portable theme package file.
pub fn read_package(path: &Path) -> Result<ThemePackage> {
  if !path.is_file() {
    return Err(CoreError::msg(format!(
      "package file not found: {}",
      path.display()
    )));
  }
  let meta = fs::metadata(path)?;
  if meta.len() > MAX_THEME_PACKAGE_BYTES {
    return Err(CoreError::msg(format!(
      "package exceeds {}MB limit ({} bytes): {}",
      MAX_THEME_PACKAGE_BYTES / (1024 * 1024),
      meta.len(),
      path.display()
    )));
  }
  let raw = fs::read_to_string(path)?;
  let package: ThemePackage = serde_json::from_str(&raw)
    .map_err(|e| CoreError::msg(format!("failed to parse package {}: {e}", path.display())))?;
  if !is_supported_package_format(&package.format) {
    return Err(CoreError::msg(format!(
      "unsupported package format `{}` in {} (expected {FORMAT_CDXTHEME} or {FORMAT_CODEDROBE})",
      package.format,
      path.display()
    )));
  }
  Ok(package)
}

// ── Validation helpers ──────────────────────────────────────────────────────

fn validate_package(package: &ThemePackage) -> Result<()> {
  if !is_supported_package_format(&package.format) {
    return Err(CoreError::msg(format!(
      "unsupported theme format `{}` (expected {FORMAT_CDXTHEME} or {FORMAT_CODEDROBE})",
      package.format
    )));
  }
  if package.schema_version != THEME_SCHEMA_VERSION {
    return Err(CoreError::msg(format!(
      "unsupported schemaVersion {}",
      package.schema_version
    )));
  }
  if !is_named_theme(&package.theme.id) {
    return Err(CoreError::msg(format!(
      "invalid theme id `{}`",
      package.theme.id
    )));
  }
  if package.theme.display_name.trim().is_empty() {
    return Err(CoreError::msg("theme.displayName must be non-empty"));
  }
  if package.theme.version.trim().is_empty() {
    return Err(CoreError::msg("theme.version must be non-empty"));
  }
  if package.targets.is_empty() {
    return Err(CoreError::msg(
      "theme package must support at least one app target",
    ));
  }
  for (app_id, target) in &package.targets {
    if !is_named_theme(app_id) {
      return Err(CoreError::msg(format!("invalid target app id `{app_id}`")));
    }
    if target.css.trim().is_empty() {
      return Err(CoreError::msg(format!(
        "targets.{app_id}.css must be non-empty"
      )));
    }
    if css_has_remote_resources(&target.css) {
      return Err(CoreError::msg(format!(
        "target `{app_id}` contains an external CSS resource"
      )));
    }
  }

  if let Some(assets) = &package.assets {
    if assets.art.is_some()
      && assets
        .images
        .as_ref()
        .is_some_and(|m| m.contains_key("hero"))
    {
      return Err(CoreError::msg(
        "assets.art cannot be combined with assets.images.hero",
      ));
    }
    if let Some(images) = &assets.images {
      if images.is_empty() {
        return Err(CoreError::msg(
          "assets.images must not be empty when provided",
        ));
      }
      if images.len() > MAX_THEME_IMAGES {
        return Err(CoreError::msg(format!(
          "assets.images exceeds {MAX_THEME_IMAGES} entries"
        )));
      }
      for (name, image) in images {
        if !is_named_theme(name) {
          return Err(CoreError::msg(format!(
            "assets.images contains invalid image id `{name}`"
          )));
        }
        validate_image_asset(image, &format!("assets.images.{name}"))?;
      }
    }
    if let Some(art) = &assets.art {
      validate_image_asset(art, "assets.art")?;
    }
  }
  Ok(())
}

fn validate_image_asset(image: &ImageAsset, label: &str) -> Result<()> {
  if image.filename.trim().is_empty() {
    return Err(CoreError::msg(format!("{label}.filename must be non-empty")));
  }
  if Path::new(&image.filename)
    .file_name()
    .and_then(|n| n.to_str())
    != Some(image.filename.as_str())
  {
    return Err(CoreError::msg(format!(
      "{label}.filename must be a safe basename"
    )));
  }
  if !SAFE_IMAGE_TYPES.contains(&image.mime_type.as_str()) {
    return Err(CoreError::msg(format!(
      "{label}.mimeType '{}' is not supported",
      image.mime_type
    )));
  }
  if image.base64.trim().is_empty() {
    return Err(CoreError::msg(format!("{label}.base64 must be non-empty")));
  }
  Ok(())
}

fn resolved_image_assets(package: &ThemePackage) -> BTreeMap<String, ImageAsset> {
  let mut images = BTreeMap::new();
  if let Some(assets) = &package.assets {
    if let Some(map) = &assets.images {
      images.extend(map.clone());
    }
    if let Some(art) = &assets.art {
      if !images.contains_key("hero") {
        images.insert("hero".into(), art.clone());
      }
    }
  }
  images
}

fn require_str<'a>(obj: &'a Map<String, Value>, key: &str) -> Result<&'a str> {
  obj
    .get(key)
    .and_then(|v| v.as_str())
    .ok_or_else(|| CoreError::msg(format!("manifest missing string field `{key}`")))
}

fn is_named_theme(value: &str) -> bool {
  let mut chars = value.chars();
  match chars.next() {
    Some(c) if c.is_ascii_alphanumeric() => {
      chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    }
    _ => false,
  }
}

fn css_has_remote_resources(css: &str) -> bool {
  let lower = css.to_ascii_lowercase();
  if lower.contains("@import") {
    return true;
  }
  for needle in [
    "url(http://",
    "url(https://",
    "url(\"http://",
    "url(\"https://",
    "url('http://",
    "url('https://",
    "url(//",
    "url(\"//",
    "url('//",
  ] {
    if lower.contains(needle) {
      return true;
    }
  }
  false
}

fn safe_asset_name(filename: &str) -> String {
  let base = Path::new(filename)
    .file_name()
    .and_then(|n| n.to_str())
    .unwrap_or("art.png");
  let cleaned: String = base
    .chars()
    .map(|c| {
      if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' {
        c
      } else {
        '-'
      }
    })
    .collect();
  if cleaned.is_empty() {
    "art.png".into()
  } else {
    cleaned
  }
}

fn mime_type_for(path: &Path) -> Option<&'static str> {
  let name = path
    .extension()
    .and_then(|e| e.to_str())
    .unwrap_or("")
    .to_ascii_lowercase();
  match name.as_str() {
    "jpg" | "jpeg" => Some("image/jpeg"),
    "webp" => Some("image/webp"),
    "gif" => Some("image/gif"),
    "png" => Some("image/png"),
    _ => None,
  }
}

fn safe_rel_path(rel: &str) -> Result<PathBuf> {
  let rel = rel.trim();
  if rel.is_empty() {
    return Err(CoreError::msg("empty relative path in package"));
  }
  let path = Path::new(rel);
  if path.is_absolute() {
    return Err(CoreError::msg(format!(
      "refusing absolute path in package: {rel}"
    )));
  }
  for c in path.components() {
    match c {
      Component::Normal(_) | Component::CurDir => {}
      _ => {
        return Err(CoreError::msg(format!(
          "refusing unsafe path in package: {rel}"
        )));
      }
    }
  }
  Ok(path.to_path_buf())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rewrite_css_replaces_all_codedrobe_prefixes() {
    let css = r#":root.codedrobe-codex-skin {
  background: var(--codedrobe-image-hero);
  --codedrobe-art: none;
}
#codedrobe-codex-skin-chrome .dream-polaroid { display: block; }
html.codedrobe-host-codex .codedrobe-theme { opacity: 1; }
"#;
    let out = rewrite_css_codedrobe_to_cdxtheme(css);
    assert!(out.contains("cdxtheme-codex-skin"));
    assert!(out.contains("--cdxtheme-image-hero"));
    assert!(out.contains("--cdxtheme-art"));
    assert!(out.contains("#cdxtheme-codex-skin-chrome"));
    assert!(out.contains("html.cdxtheme-host-codex"));
    assert!(out.contains(".cdxtheme-theme"));
    assert!(!out.contains("codedrobe-"));
  }

  #[test]
  fn pack_auto_rewrites_css_to_cdxtheme() {
    let dir = std::env::temp_dir().join(format!("cdxtheme-pack-brand-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("assets")).unwrap();

    fs::write(
      dir.join("theme.json"),
      r#"{
  "schemaVersion": 1,
  "id": "brand-demo",
  "displayName": "Brand Demo",
  "version": "1.0.0",
  "targets": { "codex": { "css": "style.css" } }
}
"#,
    )
    .unwrap();
    fs::write(
      dir.join("style.css"),
      ":root.codedrobe-codex-skin { color: #f00; background: var(--codedrobe-image-hero); }\n",
    )
    .unwrap();

    let package = build_package(&dir, &dir.join("theme.json"), PackageFormat::Cdxtheme).unwrap();
    assert_eq!(package.format, FORMAT_CDXTHEME);
    let css = &package.targets["codex"].css;
    assert!(css.contains("cdxtheme-codex-skin"));
    assert!(css.contains("--cdxtheme-image-hero"));
    assert!(!css.contains("codedrobe-"));

    // Same CSS rewrite runs regardless of package format field.
    let package_cd =
      build_package(&dir, &dir.join("theme.json"), PackageFormat::CodedrobeTheme).unwrap();
    assert_eq!(package_cd.format, FORMAT_CODEDROBE);
    assert!(
      package_cd.targets["codex"]
        .css
        .contains("cdxtheme-codex-skin")
    );
    assert!(!package_cd.targets["codex"].css.contains("codedrobe-"));

    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn convert_package_sets_format_and_rewrites_css() {
    let dir = std::env::temp_dir().join(format!("cdxtheme-convert-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let input = dir.join("demo.codedrobe-theme");
    let package = ThemePackage {
      format: FORMAT_CODEDROBE.into(),
      schema_version: THEME_SCHEMA_VERSION,
      exported_at: "2020-01-01T00:00:00.000Z".into(),
      theme: ThemeMeta {
        id: "demo".into(),
        display_name: "Demo".into(),
        version: "1.0.0".into(),
        copy: None,
      },
      targets: BTreeMap::from([(
        "codex".into(),
        ThemeTarget {
          css: ":root.codedrobe-codex-skin { color: red; }".into(),
          options: None,
          verification: None,
        },
      )]),
      assets: None,
    };
    write_package(&package, &input, true).unwrap();

    let out = dir.join("out.cdxtheme");
    let (path, _) = convert_package(&input, Some(&out), true, true).unwrap();
    assert_eq!(path, out);

    let converted = read_package(&out).unwrap();
    assert_eq!(converted.format, FORMAT_CDXTHEME);
    assert!(
      converted.targets["codex"]
        .css
        .contains("cdxtheme-codex-skin")
    );
    assert!(!converted.targets["codex"].css.contains("codedrobe-"));

    let _ = fs::remove_dir_all(&dir);
  }
}
