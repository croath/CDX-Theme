//! Build the theme list at runtime by scanning portable package files only.
//! Also fetch the remote recommend catalog and download packages into the user library.

use crate::paths::{builtin_themes_dir, user_themes_dir};
use crate::settings_store;
use crate::theme_package::{self, is_cdx_theme_file};
use cdx_theme_types::{ThemeMetadata, ThemeSource};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};

/// Remote recommend catalog index.
pub const REMOTE_THEME_INDEX_URL: &str = "https://s3.cdxtheme.com/themes/index.json";

/// Request `sig` and cache validity window (2 minutes).
pub const REMOTE_CATALOG_SIG_WINDOW: Duration = Duration::from_secs(120);

/// Disk cache file under `{app_local_data_dir}/cache/remote-theme-index.json`.
const REMOTE_CATALOG_CACHE_FILE: &str = "remote-theme-index.json";

/// Discover themes from builtin + user-installed **package files** only
/// (`.cdxtheme` / `.codedrobe-theme`). Directory themes are ignored.
pub fn discover_themes(app: &AppHandle) -> Result<Vec<ThemeMetadata>, String> {
  let mut by_id: std::collections::HashMap<String, ThemeMetadata> =
    std::collections::HashMap::new();

  if let Some(root) = builtin_themes_dir(app) {
    scan_root(&root, ThemeSource::Builtin, &mut by_id);
  }

  match user_themes_dir(app) {
    Ok(root) => scan_root(&root, ThemeSource::Installed, &mut by_id),
    Err(e) => tracing::warn!("user themes dir unavailable: {e}"),
  }

  let applied = settings_store::applied_theme_id(app);
  let mut list: Vec<ThemeMetadata> = by_id.into_values().collect();
  for item in &mut list {
    item.is_applied = applied.as_ref().is_some_and(|id| id == &item.id);
  }

  list.sort_by(|a, b| match (a.source, b.source) {
    (ThemeSource::Installed, ThemeSource::Builtin) => std::cmp::Ordering::Less,
    (ThemeSource::Builtin, ThemeSource::Installed) => std::cmp::Ordering::Greater,
    _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
  });
  Ok(list)
}

fn scan_root(
  root: &Path,
  source: ThemeSource,
  by_id: &mut std::collections::HashMap<String, ThemeMetadata>,
) {
  let Ok(entries) = fs::read_dir(root) else {
    return;
  };

  for entry in entries.flatten() {
    let path = entry.path();
    let name = path
      .file_name()
      .and_then(|n| n.to_str())
      .unwrap_or_default();

    if name.starts_with('.') {
      continue;
    }

    // Package files only — validity is JSON content, not filename extension.
    if path.is_file() && is_cdx_theme_file(&path) {
      match theme_package::peek_codex_theme_meta(&path) {
        Ok(peek) => {
          by_id.insert(
            peek.id.clone(),
            ThemeMetadata {
              id: peek.id,
              name: peek.display_name,
              location: abs_location(&path),
              preview_img: peek.preview_img,
              preview_colors: peek.preview_colors,
              is_applied: false,
              source,
              version: parse_version_u32(&peek.version),
              remote_version: None,
              update_available: false,
              theme_url: None,
            },
          );
        }
        Err(e) => tracing::warn!("skip package {}: {e}", path.display()),
      }
    }
  }
}

pub fn colors_from_base_theme(base: Option<&cdx_theme_types::BaseTheme>) -> Vec<String> {
  let accent = base.and_then(|b| b.accent.as_deref()).unwrap_or("#84CC16");
  let surface = base.and_then(|b| b.surface.as_deref()).unwrap_or("#F7FEE7");
  let ink = base.and_then(|b| b.ink.as_deref()).unwrap_or("#1A2E05");
  vec![accent.into(), surface.into(), ink.into()]
}

fn abs_location(path: &Path) -> String {
  crate::paths::abs_location_string(path)
}

pub fn ensure_user_themes_dir(app: &AppHandle) -> Result<PathBuf, String> {
  crate::paths::user_themes_dir(app)
}

/// Validate and persist a portable theme package into the user themes dir.
/// Accepts `.cdxtheme` / `.codedrobe-theme`; stores as `.cdxtheme`.
pub fn import_codex_theme_content(
  app: &AppHandle,
  file_name: &str,
  content: &str,
) -> Result<ThemeMetadata, String> {
  if content.len() as u64 > theme_package::MAX_THEME_PACKAGE_BYTES {
    return Err(format!(
      "theme package exceeds {}MB limit",
      theme_package::MAX_THEME_PACKAGE_BYTES / (1024 * 1024)
    ));
  }

  // Validate JSON content (filename is only a hint for the final store name).
  if !theme_package::is_theme_package_content(content) {
    return Err(
      "file is not a valid theme package (expected JSON with format cdxtheme, theme, and targets.codex)"
        .into(),
    );
  }

  let dest_dir = user_themes_dir(app)?;
  let temp_name = format!(
    ".install-{}.{}",
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .map(|d| d.as_millis())
      .unwrap_or(0),
    theme_package::THEME_EXTENSION
  );
  let temp_path = dest_dir.join(&temp_name);
  fs::write(&temp_path, content).map_err(|e| format!("failed to stage install: {e}"))?;

  // Full parse validates CSS (no remote resources) and target shape; stays in memory.
  let loaded = match theme_package::load_cdx_theme_file(&temp_path) {
    Ok(t) => t,
    Err(e) => {
      let _ = fs::remove_file(&temp_path);
      return Err(e);
    }
  };
  // Preview image/colors from the same file (cheap re-read of JSON metadata).
  let peek = match theme_package::peek_codex_theme_meta(&temp_path) {
    Ok(p) => p,
    Err(e) => {
      let _ = fs::remove_file(&temp_path);
      return Err(e);
    }
  };

  let _ = file_name;
  let final_path = dest_dir.join(format!(
    "{}-{}.{}",
    loaded.id,
    loaded.version,
    theme_package::THEME_EXTENSION
  ));
  if final_path.exists() {
    let _ = fs::remove_file(&final_path);
  }
  fs::rename(&temp_path, &final_path)
    .or_else(|_| fs::copy(&temp_path, &final_path).and_then(|_| fs::remove_file(&temp_path)))
    .map_err(|e| format!("failed to save theme package: {e}"))?;

  Ok(ThemeMetadata {
    id: loaded.id,
    name: loaded.display_name,
    location: abs_location(&final_path),
    preview_img: peek.preview_img,
    preview_colors: peek.preview_colors,
    is_applied: false,
    source: ThemeSource::Installed,
    version: parse_version_u32(&loaded.version),
    remote_version: None,
    update_available: false,
    theme_url: None,
  })
}

// ── Remote catalog / download ───────────────────────────────────────────────

/// Matches the S3 index payload field names (`theme_id`, `display_name`, …).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RemoteThemeIndexEntry {
  theme_id: String,
  display_name: String,
  #[serde(default)]
  version: serde_json::Value,
  #[serde(default)]
  hero: Option<String>,
  theme_url: String,
  #[serde(default)]
  created_at: Option<i64>,
  #[serde(default)]
  updated_at: Option<i64>,
}

/// Cached remote index payload (memory + disk).
///
/// Disk path: `{app_local_data_dir}/cache/remote-theme-index.json`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemoteCatalogCache {
  /// Floor-timestamp sig for the window this payload was fetched under.
  sig: String,
  /// Unix seconds when the cache was written.
  #[serde(default)]
  fetched_at: u64,
  entries: Vec<RemoteThemeIndexEntry>,
}

/// Process-local L1 cache (avoids re-reading disk within the same sig window).
static REMOTE_CATALOG_MEMORY: LazyLock<Mutex<Option<RemoteCatalogCache>>> =
  LazyLock::new(|| Mutex::new(None));

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

/// Remote index `version` field is a JSON number (u32) or string.
fn version_to_u32(v: &serde_json::Value) -> u32 {
  match v {
    serde_json::Value::Number(n) => n
      .as_u64()
      .or_else(|| n.as_i64().map(|i| i.max(0) as u64))
      .unwrap_or(0) as u32,
    serde_json::Value::String(s) => parse_version_u32(s).unwrap_or(0),
    _ => 0,
  }
}

/// Unix timestamp floored to a 2-minute window (changes every 120s).
///
/// Example: `t=1784381254` → window start `1784381200` → sig `"1784381200"`.
pub fn remote_catalog_sig(now: SystemTime) -> String {
  let secs = now
    .duration_since(UNIX_EPOCH)
    .map(|d| d.as_secs())
    .unwrap_or(0);
  let window = REMOTE_CATALOG_SIG_WINDOW.as_secs().max(1);
  let floored = (secs / window) * window;
  floored.to_string()
}

fn unix_now_secs() -> u64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|d| d.as_secs())
    .unwrap_or(0)
}

/// Build the recommend index URL with the current time-based `sig` query param.
pub fn remote_catalog_url_with_sig(sig: &str) -> String {
  // Avoid double-`?` if the base URL ever gains query params.
  let sep = if REMOTE_THEME_INDEX_URL.contains('?') {
    '&'
  } else {
    '?'
  };
  format!("{REMOTE_THEME_INDEX_URL}{sep}sig={sig}")
}

/// `{app_local_data_dir}/cache/remote-theme-index.json`
pub fn remote_catalog_cache_path(app: &AppHandle) -> Result<PathBuf, String> {
  let base = app
    .path()
    .app_local_data_dir()
    .map_err(|e| format!("app local data dir: {e}"))?;
  let dir = base.join("cache");
  fs::create_dir_all(&dir).map_err(|e| format!("create cache dir {}: {e}", dir.display()))?;
  Ok(dir.join(REMOTE_CATALOG_CACHE_FILE))
}

fn load_remote_catalog_memory(sig: &str) -> Option<Vec<RemoteThemeIndexEntry>> {
  let guard = REMOTE_CATALOG_MEMORY.lock().ok()?;
  let cache = guard.as_ref()?;
  if cache.sig == sig {
    tracing::debug!(
      sig,
      count = cache.entries.len(),
      "remote catalog memory cache hit"
    );
    Some(cache.entries.clone())
  } else {
    None
  }
}

fn store_remote_catalog_memory(cache: &RemoteCatalogCache) {
  if let Ok(mut guard) = REMOTE_CATALOG_MEMORY.lock() {
    *guard = Some(cache.clone());
    tracing::debug!(
      sig = %cache.sig,
      count = cache.entries.len(),
      "remote catalog memory cache stored"
    );
  }
}

fn clear_remote_catalog_memory() {
  if let Ok(mut guard) = REMOTE_CATALOG_MEMORY.lock() {
    *guard = None;
  }
}

fn load_remote_catalog_disk_cache(app: &AppHandle) -> Option<RemoteCatalogCache> {
  let path = remote_catalog_cache_path(app).ok()?;
  let raw = fs::read_to_string(&path).ok()?;
  match serde_json::from_str::<RemoteCatalogCache>(&raw) {
    Ok(cache) => Some(cache),
    Err(e) => {
      tracing::warn!(
        path = %path.display(),
        error = %e,
        "remote catalog disk cache unreadable; ignoring"
      );
      None
    }
  }
}

fn save_remote_catalog_disk_cache(
  app: &AppHandle,
  cache: &RemoteCatalogCache,
) -> Result<(), String> {
  let path = remote_catalog_cache_path(app)?;
  let raw = serde_json::to_string_pretty(cache).map_err(|e| format!("serialize cache: {e}"))?;
  fs::write(&path, raw).map_err(|e| format!("write cache {}: {e}", path.display()))?;
  tracing::debug!(
    path = %path.display(),
    sig = %cache.sig,
    count = cache.entries.len(),
    "remote catalog disk cache stored"
  );
  Ok(())
}

/// Clear memory + disk remote catalog caches (force refresh).
pub fn clear_remote_catalog_cache(app: &AppHandle) {
  clear_remote_catalog_memory();
  match remote_catalog_cache_path(app) {
    Ok(path) => {
      if path.is_file() {
        match fs::remove_file(&path) {
          Ok(()) => tracing::info!(path = %path.display(), "remote catalog disk cache cleared"),
          Err(e) => tracing::warn!(
            path = %path.display(),
            error = %e,
            "failed to clear remote catalog disk cache"
          ),
        }
      }
    }
    Err(e) => tracing::warn!("remote catalog cache path: {e}"),
  }
}

async fn fetch_remote_index_entries(
  app: &AppHandle,
  sig: &str,
) -> Result<Vec<RemoteThemeIndexEntry>, String> {
  // L1: process memory (same 2-minute sig window).
  if let Some(entries) = load_remote_catalog_memory(sig) {
    return Ok(entries);
  }

  // L2: disk under app local data.
  if let Some(cache) = load_remote_catalog_disk_cache(app) {
    if cache.sig == sig {
      tracing::debug!(
        sig,
        count = cache.entries.len(),
        fetched_at = cache.fetched_at,
        "remote catalog disk cache hit"
      );
      store_remote_catalog_memory(&cache);
      return Ok(cache.entries);
    }
    tracing::debug!(
      cached_sig = %cache.sig,
      current_sig = %sig,
      "remote catalog disk cache stale"
    );
  }

  // L3: network.
  let url = remote_catalog_url_with_sig(sig);
  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(30))
    .build()
    .map_err(|e| format!("http client: {e}"))?;

  tracing::info!(%url, sig, "fetching remote theme catalog");
  let response = client
    .get(&url)
    .send()
    .await
    .map_err(|e| format!("failed to fetch theme catalog: {e}"))?;
  let status = response.status();
  let body = response
    .bytes()
    .await
    .map_err(|e| format!("failed to read theme catalog body: {e}"))?;
  if !status.is_success() {
    let preview = String::from_utf8_lossy(&body);
    let preview = preview.chars().take(200).collect::<String>();
    return Err(format!("theme catalog HTTP {status}: {url} body={preview}"));
  }

  let entries: Vec<RemoteThemeIndexEntry> = serde_json::from_slice(&body).map_err(|e| {
    let preview = String::from_utf8_lossy(&body);
    let preview = preview.chars().take(200).collect::<String>();
    format!("invalid theme catalog JSON: {e}; body starts with: {preview}")
  })?;

  let cache = RemoteCatalogCache {
    sig: sig.to_string(),
    fetched_at: unix_now_secs(),
    entries: entries.clone(),
  };
  store_remote_catalog_memory(&cache);
  if let Err(e) = save_remote_catalog_disk_cache(app, &cache) {
    // Non-fatal: still return network result if disk write fails.
    tracing::warn!("remote catalog disk cache write failed: {e}");
  }

  Ok(entries)
}

fn map_remote_entries_to_metadata(
  app: &AppHandle,
  entries: Vec<RemoteThemeIndexEntry>,
) -> Vec<ThemeMetadata> {
  let applied = settings_store::applied_theme_id(app);
  let local = discover_themes(app).unwrap_or_default();
  let local_by_id: std::collections::HashMap<_, _> =
    local.into_iter().map(|t| (t.id.clone(), t)).collect();

  let mut list = Vec::with_capacity(entries.len());
  for entry in entries {
    let remote_ver = version_to_u32(&entry.version);
    let id = entry.theme_id;
    // Prefer local installed/builtin entry when the user already has this theme.
    if let Some(local) = local_by_id.get(&id) {
      let mut meta = local.clone();
      meta.theme_url = Some(entry.theme_url);
      if meta.preview_img.is_none() {
        meta.preview_img = entry.hero.filter(|s| !s.is_empty());
      }
      let local_ver = meta.version.unwrap_or(0);
      meta.remote_version = Some(remote_ver);
      // Notify when cloud index is newer than the installed package version.
      meta.update_available = remote_ver > local_ver;
      if meta.name.trim().is_empty() {
        meta.name = entry.display_name;
      } else if !entry.display_name.is_empty() {
        // Keep display name fresh from catalog.
        meta.name = entry.display_name;
      }
      list.push(meta);
      continue;
    }

    let _ = (entry.created_at, entry.updated_at);
    list.push(ThemeMetadata {
      id: id.clone(),
      name: entry.display_name,
      location: String::new(),
      preview_img: entry.hero.filter(|s| !s.is_empty()),
      preview_colors: vec![],
      is_applied: applied.as_ref().is_some_and(|a| a == &id),
      source: ThemeSource::Remote,
      version: None,
      remote_version: Some(remote_ver),
      update_available: false,
      theme_url: Some(entry.theme_url),
    });
  }

  list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
  list
}

/// Fetch the public recommend index and map to UI metadata (`source = Remote`).
///
/// - Appends `?sig=<floor_unix_2min>` so the request token rotates every 2 minutes.
/// - Caches the raw index in process memory (L1) and under
///   `{app_local_data}/cache/remote-theme-index.json` (L2) for the current sig window.
///   Local install state is re-merged each call.
/// - When `force` is true, clears memory + disk caches and re-fetches from the network.
pub async fn fetch_remote_theme_catalog(
  app: &AppHandle,
  force: bool,
) -> Result<Vec<ThemeMetadata>, String> {
  if force {
    clear_remote_catalog_cache(app);
  }
  let sig = remote_catalog_sig(SystemTime::now());
  let entries = fetch_remote_index_entries(app, &sig).await?;
  Ok(map_remote_entries_to_metadata(app, entries))
}

/// Download a remote `.cdxtheme` package into the user themes library.
pub async fn download_theme_to_library(
  app: &AppHandle,
  theme_url: &str,
) -> Result<ThemeMetadata, String> {
  let url = theme_url.trim();
  if url.is_empty() {
    return Err("theme_url is empty".into());
  }
  if !(url.starts_with("https://") || url.starts_with("http://")) {
    return Err(format!("unsupported theme_url scheme: {url}"));
  }

  tracing::info!(url, "downloading theme package");
  let client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(120))
    .build()
    .map_err(|e| format!("http client: {e}"))?;

  let response = client
    .get(url)
    .send()
    .await
    .map_err(|e| format!("download failed: {e}"))?;
  if !response.status().is_success() {
    return Err(format!("download HTTP {} for {url}", response.status()));
  }

  let bytes = response
    .bytes()
    .await
    .map_err(|e| format!("download body: {e}"))?;
  if bytes.len() as u64 > theme_package::MAX_THEME_PACKAGE_BYTES {
    return Err(format!(
      "theme package exceeds {}MB limit",
      theme_package::MAX_THEME_PACKAGE_BYTES / (1024 * 1024)
    ));
  }

  let content = String::from_utf8(bytes.to_vec())
    .map_err(|e| format!("theme package is not UTF-8 JSON: {e}"))?;

  let file_hint = url
    .rsplit('/')
    .next()
    .filter(|s| !s.is_empty())
    .unwrap_or("theme.cdxtheme");

  let meta = import_codex_theme_content(app, file_hint, &content)?;
  tracing::info!(
    id = %meta.id,
    location = %meta.location,
    "theme downloaded into library"
  );
  Ok(meta)
}

/// Ensure a theme is available as a local package file, downloading if needed.
///
/// Returns the absolute package path.
pub async fn ensure_theme_package_path(
  app: &AppHandle,
  theme_id: &str,
  theme_url: Option<&str>,
) -> Result<PathBuf, String> {
  // Prefer an existing local package (builtin or installed).
  if let Ok(list) = discover_themes(app) {
    if let Some(meta) = list.iter().find(|t| t.id == theme_id) {
      if !meta.location.is_empty() {
        let path = PathBuf::from(&meta.location);
        if path.is_file() {
          return Ok(path);
        }
      }
    }
  }

  let url = theme_url
    .map(str::trim)
    .filter(|s| !s.is_empty())
    .ok_or_else(|| {
      format!("theme `{theme_id}` is not installed and no theme_url was provided to download it")
    })?;

  let meta = download_theme_to_library(app, url).await?;
  if meta.id != theme_id {
    tracing::warn!(
      expected = theme_id,
      got = %meta.id,
      "downloaded package id differs from requested theme_id"
    );
  }
  Ok(PathBuf::from(meta.location))
}

/// Delete a user-installed theme package. Built-in themes cannot be deleted.
pub fn delete_installed_theme(app: &AppHandle, theme_id: &str) -> Result<(), String> {
  let list = discover_themes(app)?;
  let meta = list
    .into_iter()
    .find(|t| t.id == theme_id)
    .ok_or_else(|| format!("theme id `{theme_id}` not found"))?;

  if meta.source != ThemeSource::Installed {
    return Err("only installed themes can be deleted (built-in themes are protected)".into());
  }

  let path = PathBuf::from(&meta.location);
  let user_root = user_themes_dir(app)?;
  if !crate::paths::path_is_under(&path, &user_root) {
    return Err("refusing to delete theme outside user themes directory".into());
  }

  let path_canon = path
    .canonicalize()
    .map(|p| crate::paths::strip_verbatim_prefix(&p))
    .unwrap_or_else(|_| path.clone());

  if path_canon.is_file() {
    fs::remove_file(&path_canon)
      .map_err(|e| format!("failed to delete {}: {e}", path_canon.display()))?;
  } else {
    return Err(format!(
      "theme package missing or not a file: {}",
      path_canon.display()
    ));
  }

  // Best-effort cleanup of legacy extract caches from older builds.
  let extract_root = user_root.join(".extracted");
  if extract_root.is_dir() {
    let _ = fs::remove_dir_all(&extract_root);
  }

  if settings_store::applied_theme_id(app).as_deref() == Some(theme_id) {
    settings_store::set_applied_theme_id(app, None)?;
  }

  Ok(())
}
