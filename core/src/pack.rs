//! Pack / unpack portable multi-app theme packages (`.cdxtheme`).

use crate::error::{CoreError, Result};
use base64::Engine;
use cdx_theme_types::{deserialize_version_u32, parse_version_u32};
use chrono::Utc;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub const FORMAT_CDXTHEME: &str = "cdxtheme";
pub const EXT_CDXTHEME: &str = "cdxtheme";

pub const THEME_SCHEMA_VERSION: u64 = 1;
pub const MAX_THEME_PACKAGE_BYTES: u64 = 30 * 1024 * 1024;
pub const MAX_THEME_IMAGES: usize = 32;

const SAFE_IMAGE_TYPES: &[&str] = &["image/png", "image/jpeg", "image/webp", "image/gif"];

pub fn is_supported_package_format(format: &str) -> bool {
  format.trim().eq_ignore_ascii_case(FORMAT_CDXTHEME)
}

// ── Portable package ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeMeta {
  pub id: String,
  pub display_name: String,
  /// Integer package version (JSON number; legacy string versions are accepted on read).
  #[serde(deserialize_with = "deserialize_version_u32")]
  pub version: u32,
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
  /// Alias for `images.hero` (either `art` or `images.hero`, not both).
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
    format!("{}-{}.{}", self.theme.id, self.theme.version, EXT_CDXTHEME)
  }
}

// ── Pack ────────────────────────────────────────────────────────────────────

/// Preferred source filenames inside a theme directory (first match wins).
pub const SOURCE_FILENAMES: &[&str] = &["theme.json", "manifest.json"];

// ── Split CSS partials → merged target CSS ──────────────────────────────────

/// Default targets that map partial dirs to root CSS files.
/// Convention: `{theme_dir}/codex/*.css` → `{theme_dir}/codex.css` (alphabetical order).
pub const CSS_PARTIAL_TARGETS: &[(&str, &str)] =
  &[("codex", "codex.css"), ("workbuddy", "workbuddy.css")];

/// Result of merging one target's partials.
#[derive(Debug, Clone)]
pub struct CssMergeResult {
  /// Target id (e.g. `codex`).
  pub target: String,
  /// Directory that held the partials.
  pub partials_dir: PathBuf,
  /// Written merged file.
  pub output: PathBuf,
  /// Partial filenames in merge order.
  pub parts: Vec<String>,
  /// Byte length of the merged CSS (without requiring a re-read).
  pub bytes: u64,
}

/// List `*.css` files in `partials_dir` in deterministic alphabetical order.
pub fn list_css_partials(partials_dir: &Path) -> Result<Vec<PathBuf>> {
  if !partials_dir.is_dir() {
    return Err(CoreError::msg(format!(
      "css partials directory not found: {}",
      partials_dir.display()
    )));
  }
  let mut parts: Vec<PathBuf> = fs::read_dir(partials_dir)?
    .filter_map(|e| e.ok())
    .map(|e| e.path())
    .filter(|p| {
      p.is_file()
        && p
          .extension()
          .and_then(|e| e.to_str())
          .map(|e| e.eq_ignore_ascii_case("css"))
          .unwrap_or(false)
    })
    .collect();
  parts.sort_by(|a, b| {
    a.file_name()
      .unwrap_or_default()
      .cmp(b.file_name().unwrap_or_default())
  });
  if parts.is_empty() {
    return Err(CoreError::msg(format!(
      "no *.css partials in {}",
      partials_dir.display()
    )));
  }
  Ok(parts)
}

/// Concatenate sorted CSS partials into a single string (with a generated header).
pub fn merge_css_partials_content(
  partials_dir: &Path,
  target: &str,
) -> Result<(String, Vec<String>)> {
  let parts = list_css_partials(partials_dir)?;
  let names: Vec<String> = parts
    .iter()
    .filter_map(|p| {
      p.file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
    })
    .collect();

  let mut out = format!(
    "/* Generated by `cdxtheme theme merge-css` — do not edit by hand.\n * Target: {target}\n * Sources ({n}): {list}\n * Edit partials under `{target}/` then re-run merge (or `theme pack`, which auto-merges).\n */\n\n",
    target = target,
    n = names.len(),
    list = names.join(", "),
  );

  for (i, path) in parts.iter().enumerate() {
    let name = path
      .file_name()
      .and_then(|n| n.to_str())
      .unwrap_or("partial.css");
    let body = fs::read_to_string(path)?;
    if body.trim().is_empty() {
      return Err(CoreError::msg(format!(
        "css partial is empty: {}",
        path.display()
      )));
    }
    if i > 0 {
      out.push('\n');
    }
    out.push_str(&format!("/* ── begin {target}/{name} ── */\n"));
    out.push_str(body.trim_end());
    out.push('\n');
    out.push_str(&format!("/* ── end {target}/{name} ── */\n"));
  }
  if !out.ends_with('\n') {
    out.push('\n');
  }
  Ok((out, names))
}

/// Merge `{theme_dir}/{target}/*.css` → `{theme_dir}/{output_name}`.
pub fn merge_css_target(
  theme_dir: &Path,
  target: &str,
  output_name: &str,
) -> Result<CssMergeResult> {
  let partials_dir = theme_dir.join(target);
  let output = theme_dir.join(output_name);
  let (content, parts) = merge_css_partials_content(&partials_dir, target)?;
  if let Some(parent) = output.parent()
    && !parent.as_os_str().is_empty() {
      fs::create_dir_all(parent)?;
    }
  fs::write(&output, &content)?;
  Ok(CssMergeResult {
    target: target.to_string(),
    partials_dir,
    output,
    parts,
    bytes: content.len() as u64,
  })
}

/// Merge CSS partials for a theme directory.
///
/// - `target_filter`: `None` merges every known target dir that exists (`codex/`, `workbuddy/`).
///   `Some("codex")` merges only that target (directory must exist).
/// - Also accepts a custom target name: merges `{dir}/{name}/*.css` → `{dir}/{name}.css`.
pub fn merge_theme_css(
  theme_dir: &Path,
  target_filter: Option<&str>,
) -> Result<Vec<CssMergeResult>> {
  if !theme_dir.is_dir() {
    return Err(CoreError::msg(format!(
      "theme directory not found: {}",
      theme_dir.display()
    )));
  }

  let mut jobs: Vec<(String, String)> = Vec::new();
  if let Some(t) = target_filter.map(str::trim).filter(|s| !s.is_empty()) {
    // Prefer known mapping; else {t}/{t}.css
    let out = CSS_PARTIAL_TARGETS
      .iter()
      .find(|(name, _)| *name == t)
      .map(|(_, o)| (*o).to_string())
      .unwrap_or_else(|| format!("{t}.css"));
    jobs.push((t.to_string(), out));
  } else {
    for (name, out) in CSS_PARTIAL_TARGETS {
      if theme_dir.join(name).is_dir() {
        jobs.push(((*name).to_string(), (*out).to_string()));
      }
    }
    if jobs.is_empty() {
      return Err(CoreError::msg(format!(
        "no css partial directories found under {} (expected codex/ and/or workbuddy/)",
        theme_dir.display()
      )));
    }
  }

  let mut results = Vec::new();
  for (target, out_name) in jobs {
    let partials = theme_dir.join(&target);
    if !partials.is_dir() {
      return Err(CoreError::msg(format!(
        "css partials directory not found: {}",
        partials.display()
      )));
    }
    results.push(merge_css_target(theme_dir, &target, &out_name)?);
  }
  Ok(results)
}

/// Merge CSS partials when present; returns an empty list if none (never errors for missing dirs).
///
/// Used by `theme pack` (default) so packing a monolithic-only theme stays a no-op for merge.
pub fn merge_theme_css_optional(theme_dir: &Path) -> Result<Vec<CssMergeResult>> {
  if !theme_dir.is_dir() {
    return Ok(Vec::new());
  }
  let mut results = Vec::new();
  for (name, out_name) in CSS_PARTIAL_TARGETS {
    let partials = theme_dir.join(name);
    if !partials.is_dir() {
      continue;
    }
    match list_css_partials(&partials) {
      Ok(_) => results.push(merge_css_target(theme_dir, name, out_name)?),
      Err(_) => continue,
    }
  }
  Ok(results)
}

/// Locate partials for a target CSS path (`codex.css` → `codex/*.css`).
fn partials_dir_for_css(base: &Path, css_rel: &str) -> Option<(PathBuf, String)> {
  let stem = Path::new(css_rel)
    .file_stem()
    .and_then(|s| s.to_str())
    .filter(|s| !s.is_empty())?
    .to_string();
  let parent = Path::new(css_rel)
    .parent()
    .filter(|p| !p.as_os_str().is_empty())
    .map(|p| base.join(p))
    .unwrap_or_else(|| base.to_path_buf());
  let partials_dir = parent.join(&stem);
  if partials_dir.is_dir() {
    Some((partials_dir, stem))
  } else {
    None
  }
}

/// In-memory merge of CSS partials for a target path. Does **not** write disk files.
///
/// Returns `None` when no partials directory / `*.css` parts exist.
pub fn merge_css_for_path_in_memory(
  base: &Path,
  css_rel: &str,
) -> Result<Option<(String, CssMergeResult)>> {
  let Some((partials_dir, stem)) = partials_dir_for_css(base, css_rel) else {
    return Ok(None);
  };
  match list_css_partials(&partials_dir) {
    Ok(_) => {
      let (content, parts) = merge_css_partials_content(&partials_dir, &stem)?;
      let bytes = content.len() as u64;
      Ok(Some((
        content,
        CssMergeResult {
          target: stem,
          partials_dir,
          // Logical path only — pack embeds CSS without writing this file.
          output: base.join(css_rel),
          parts,
          bytes,
        },
      )))
    }
    Err(_) => Ok(None),
  }
}

/// Resolve target CSS for packing: merge partials in memory when present, else read the file.
///
/// Never writes `codex.css` / `workbuddy.css` — merged CSS is returned for embedding in the package.
pub fn resolve_target_css(
  base: &Path,
  css_rel: &str,
  merge_css: bool,
) -> Result<(String, Option<CssMergeResult>)> {
  if merge_css
    && let Some((content, merged)) = merge_css_for_path_in_memory(base, css_rel)? {
      if content.trim().is_empty() {
        return Err(CoreError::msg(format!(
          "merged css is empty for `{css_rel}` (partials under {})",
          merged.partials_dir.display()
        )));
      }
      return Ok((content, Some(merged)));
    }
  let css_path = base.join(css_rel);
  if !css_path.is_file() {
    return Err(CoreError::msg(format!(
      "theme css not found: {} (and no partials dir to merge)",
      css_path.display()
    )));
  }
  let css = fs::read_to_string(&css_path)?;
  if css.trim().is_empty() {
    return Err(CoreError::msg(format!(
      "theme css is empty: {}",
      css_path.display()
    )));
  }
  Ok((css, None))
}

/// Merge partials and write the root CSS file (used by `theme merge-css` only).
pub fn maybe_merge_css_for_path(base: &Path, css_rel: &str) -> Result<Option<CssMergeResult>> {
  let Some((content, mut result)) = merge_css_for_path_in_memory(base, css_rel)? else {
    return Ok(None);
  };
  let css_path = base.join(css_rel);
  if let Some(p) = css_path.parent()
    && !p.as_os_str().is_empty() {
      fs::create_dir_all(p)?;
    }
  fs::write(&css_path, &content)?;
  result.output = css_path;
  result.bytes = content.len() as u64;
  Ok(Some(result))
}

/// Result of packing a theme directory.
#[derive(Debug, Clone)]
pub struct PackThemeResult {
  pub path: PathBuf,
  pub bytes: u64,
  /// CSS partial merges performed before packing (empty if none / disabled).
  pub merges: Vec<CssMergeResult>,
}

/// Pack a theme directory (or path to `theme.json` / `manifest.json`) into a `.cdxtheme` package.
///
/// By default (`merge_css = true`), merges `codex/*.css` / `workbuddy/*.css` **in memory**
/// into the package. Does **not** write root `codex.css` / `workbuddy.css`
/// (use `theme merge-css` to materialize those files on disk).
pub fn pack_theme_dir(
  theme_dir_or_manifest: &Path,
  output: Option<&Path>,
  pretty: bool,
  force: bool,
) -> Result<PackThemeResult> {
  pack_theme_dir_with_options(theme_dir_or_manifest, output, pretty, force, true)
}

/// Pack with explicit control over the default CSS partial merge step.
pub fn pack_theme_dir_with_options(
  theme_dir_or_manifest: &Path,
  output: Option<&Path>,
  pretty: bool,
  force: bool,
  merge_css: bool,
) -> Result<PackThemeResult> {
  let (base, source_path) = resolve_source_paths(theme_dir_or_manifest)?;

  // Merge partials in memory into package CSS only — do not write codex.css / workbuddy.css.
  let (package, merges) = build_package_with_options(&base, &source_path, merge_css)?;
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

  if let Some(parent) = out.parent()
    && !parent.as_os_str().is_empty() {
      fs::create_dir_all(parent)?;
    }

  let bytes = write_package(&package, &out, pretty)?;
  Ok(PackThemeResult {
    path: out,
    bytes,
    merges,
  })
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

pub fn build_package(base: &Path, source_path: &Path) -> Result<ThemePackage> {
  Ok(build_package_with_options(base, source_path, true)?.0)
}

/// Build a package; when `merge_css`, partials are merged **in memory** (no root CSS files written).
pub fn build_package_with_options(
  base: &Path,
  source_path: &Path,
  merge_css: bool,
) -> Result<(ThemePackage, Vec<CssMergeResult>)> {
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
  let version = require_version(obj)?;
  if !is_named_theme(&id) {
    return Err(CoreError::msg(format!(
      "invalid theme id `{id}` (use alphanumeric, `_`, `-`)"
    )));
  }
  if display_name.trim().is_empty() {
    return Err(CoreError::msg("manifest displayName must be non-empty"));
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

  let mut merges: Vec<CssMergeResult> = Vec::new();
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
    // Merge partials in memory when present; never write root CSS during pack.
    let (css, merge_info) = resolve_target_css(base, css_rel, merge_css)
      .map_err(|e| CoreError::msg(format!("target `{app_id}`: {e}")))?;
    if let Some(m) = merge_info {
      merges.push(m);
    }
    if css_has_remote_resources(&css) {
      return Err(CoreError::msg(format!(
        "target `{app_id}` contains an external CSS resource; only embedded data URLs are supported"
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
    let mime = mime_type_for(&image_path).ok_or_else(|| {
      CoreError::msg(format!("images.{name} uses an unsupported image file type"))
    })?;
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
    format: FORMAT_CDXTHEME.into(),
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
  Ok((package, merges))
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

// ── Unpack ──────────────────────────────────────────────────────────────────

/// Unpack a `.cdxtheme` package into a source theme directory.
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
  source.insert("version".into(), Value::from(package.theme.version));
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
      "unsupported package format `{}` in {} (expected {FORMAT_CDXTHEME})",
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
      "unsupported theme format `{}` (expected {FORMAT_CDXTHEME})",
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
    return Err(CoreError::msg(format!(
      "{label}.filename must be non-empty"
    )));
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
    if let Some(art) = &assets.art
      && !images.contains_key("hero") {
        images.insert("hero".into(), art.clone());
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

/// Source `version` as integer — accepts JSON number or legacy string (`"1"`, `"1.2.3"`).
fn require_version(obj: &Map<String, Value>) -> Result<u32> {
  let v = obj
    .get("version")
    .ok_or_else(|| CoreError::msg("manifest missing field `version`"))?;
  match v {
    Value::Number(n) => {
      let n = n
        .as_u64()
        .or_else(|| n.as_i64().filter(|i| *i >= 0).map(|i| i as u64))
        .ok_or_else(|| CoreError::msg("manifest version must be a non-negative integer"))?;
      u32::try_from(n).map_err(|_| CoreError::msg(format!("manifest version {n} out of u32 range")))
    }
    Value::String(s) => parse_version_u32(s).ok_or_else(|| {
      CoreError::msg(format!(
        "manifest version must be a non-negative integer (or dotted major.minor…), got {s:?}"
      ))
    }),
    _ => Err(CoreError::msg(
      "manifest version must be a number or string",
    )),
  }
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
  fn pack_sets_cdxtheme_format() {
    let dir =
      std::env::temp_dir().join(format!("cdxtheme-pack-format-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    fs::write(
      dir.join("theme.json"),
      r#"{
  "schemaVersion": 1,
  "id": "brand-demo",
  "displayName": "Brand Demo",
  "version": 2,
  "targets": { "codex": { "css": "style.css" } }
}
"#,
    )
    .unwrap();
    fs::write(
      dir.join("style.css"),
      ":root.cdxtheme-codex-skin { color: #f00; background: var(--cdxtheme-image-hero); }\n",
    )
    .unwrap();

    let package = build_package(&dir, &dir.join("theme.json")).unwrap();
    assert_eq!(package.format, FORMAT_CDXTHEME);
    assert_eq!(package.theme.version, 2);
    assert_eq!(package.package_filename(), "brand-demo-2.cdxtheme");
    // Packed JSON stores version as a number.
    let out = dir.join("out.cdxtheme");
    write_package(&package, &out, true).unwrap();
    let raw = fs::read_to_string(&out).unwrap();
    assert!(
      raw.contains("\"version\": 2") || raw.contains("\"version\":2"),
      "packed version must be a JSON number: {raw}"
    );
    let css = &package.targets["codex"].css;
    assert!(css.contains("cdxtheme-codex-skin"));
    assert!(css.contains("--cdxtheme-image-hero"));

    // Legacy string versions still pack (major component).
    fs::write(
      dir.join("theme.json"),
      r#"{
  "schemaVersion": 1,
  "id": "brand-demo",
  "displayName": "Brand Demo",
  "version": "1.2.3",
  "targets": { "codex": { "css": "style.css" } }
}
"#,
    )
    .unwrap();
    let legacy = build_package(&dir, &dir.join("theme.json")).unwrap();
    assert_eq!(legacy.theme.version, 1);

    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn rejects_non_cdxtheme_format() {
    let dir = std::env::temp_dir().join(format!("cdxtheme-reject-format-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let input = dir.join("demo.cdxtheme");
    let package = ThemePackage {
      format: "codedrobe-theme".into(),
      schema_version: THEME_SCHEMA_VERSION,
      exported_at: "2020-01-01T00:00:00.000Z".into(),
      theme: ThemeMeta {
        id: "demo".into(),
        display_name: "Demo".into(),
        version: 1,
        copy: None,
      },
      targets: BTreeMap::from([(
        "codex".into(),
        ThemeTarget {
          css: ":root.cdxtheme-codex-skin { color: red; }".into(),
          options: None,
          verification: None,
        },
      )]),
      assets: None,
    };
    // write_package does not re-validate format; read_package does.
    write_package(&package, &input, true).unwrap();
    let err = read_package(&input).unwrap_err().to_string();
    assert!(
      err.contains("unsupported package format") && err.contains(FORMAT_CDXTHEME),
      "unexpected error: {err}"
    );

    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn merge_css_partials_alphabetical() {
    let dir = std::env::temp_dir().join(format!("cdxtheme-merge-css-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let codex = dir.join("codex");
    fs::create_dir_all(&codex).unwrap();
    fs::write(codex.join("01-b.css"), "/* b */\n.b { color: blue; }\n").unwrap();
    fs::write(codex.join("00-a.css"), "/* a */\n.a { color: red; }\n").unwrap();
    fs::write(codex.join("02-c.css"), "/* c */\n.c { color: green; }\n").unwrap();

    let results = merge_theme_css(&dir, None).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].parts, vec!["00-a.css", "01-b.css", "02-c.css"]);
    let merged = fs::read_to_string(dir.join("codex.css")).unwrap();
    assert!(merged.contains("begin codex/00-a.css"));
    assert!(merged.contains(".a { color: red; }"));
    assert!(merged.contains(".b { color: blue; }"));
    assert!(merged.contains(".c { color: green; }"));
    // order: a before b before c
    let ia = merged.find(".a {").unwrap();
    let ib = merged.find(".b {").unwrap();
    let ic = merged.find(".c {").unwrap();
    assert!(ia < ib && ib < ic);

    // pack merges partials in memory (does not write root codex.css)
    fs::write(
      dir.join("theme.json"),
      r#"{
  "schemaVersion": 1,
  "id": "merge-demo",
  "displayName": "Merge Demo",
  "version": 1,
  "targets": { "codex": { "css": "codex.css" } }
}
"#,
    )
    .unwrap();
    // delete any root css if present — pack must still succeed from partials only
    let _ = fs::remove_file(dir.join("codex.css"));
    let (package, merges) =
      build_package_with_options(&dir, &dir.join("theme.json"), true).unwrap();
    assert!(package.targets["codex"].css.contains(".a { color: red; }"));
    assert_eq!(merges.len(), 1);
    assert!(
      !dir.join("codex.css").is_file(),
      "pack must not materialize codex.css"
    );

    let _ = fs::remove_dir_all(&dir);
  }
}
