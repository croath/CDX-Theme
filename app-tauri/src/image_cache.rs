//! Local disk cache for remote preview / hero images.
//!
//! Files live under `{app_local_data_dir}/cache/preview-images/{sha256}`.
//! Callers receive a `data:` URL so the webview never needs the network after
//! the first successful fetch.

use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::{AppHandle, Manager};

/// Soft cap for a single cached preview (raw bytes). Larger remote images are rejected.
const MAX_PREVIEW_BYTES: u64 = 4 * 1024 * 1024;

/// `{app_local_data_dir}/cache/preview-images`
pub fn preview_images_dir(app: &AppHandle) -> Result<PathBuf, String> {
  let base = app
    .path()
    .app_local_data_dir()
    .map_err(|e| format!("app local data dir: {e}"))?;
  let dir = base.join("cache").join("preview-images");
  fs::create_dir_all(&dir)
    .map_err(|e| format!("create preview cache dir {}: {e}", dir.display()))?;
  Ok(dir)
}

fn is_remote_http_url(url: &str) -> bool {
  let u = url.trim();
  u.starts_with("https://") || u.starts_with("http://")
}

fn url_cache_key(url: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(url.trim().as_bytes());
  let digest = hasher.finalize();
  digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn cache_file_path(dir: &Path, url: &str) -> PathBuf {
  dir.join(url_cache_key(url))
}

fn sniff_mime(bytes: &[u8]) -> Option<&'static str> {
  if bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF {
    return Some("image/jpeg");
  }
  if bytes.len() >= 8 && bytes.starts_with(&[0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1A, b'\n']) {
    return Some("image/png");
  }
  if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
    return Some("image/webp");
  }
  if bytes.len() >= 6 && (bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a")) {
    return Some("image/gif");
  }
  None
}

fn mime_from_content_type(ct: &str) -> Option<&'static str> {
  let ct = ct.to_ascii_lowercase();
  let main = ct.split(';').next().unwrap_or("").trim();
  match main {
    "image/jpeg" | "image/jpg" => Some("image/jpeg"),
    "image/png" => Some("image/png"),
    "image/webp" => Some("image/webp"),
    "image/gif" => Some("image/gif"),
    _ => None,
  }
}

fn bytes_to_data_url(bytes: &[u8], mime: &str) -> String {
  format!("data:{mime};base64,{}", B64.encode(bytes))
}

fn read_cached_data_url(path: &Path) -> Result<String, String> {
  let bytes = fs::read(path).map_err(|e| format!("read cache {}: {e}", path.display()))?;
  if bytes.is_empty() {
    return Err("empty cached preview image".into());
  }
  let mime = sniff_mime(&bytes)
    .ok_or_else(|| format!("cached file is not a recognized image: {}", path.display()))?;
  Ok(bytes_to_data_url(&bytes, mime))
}

/// Resolve a preview image to a **local** `data:` URL.
///
/// - Already-`data:` URLs are returned unchanged.
/// - Non-HTTP strings (e.g. relative assets) are returned unchanged.
/// - HTTP(S) URLs are served from disk cache, downloading once when missing.
pub async fn resolve_to_data_url(app: &AppHandle, url: &str) -> Result<String, String> {
  let url = url.trim();
  if url.is_empty() {
    return Err("empty image url".into());
  }
  if url.starts_with("data:") {
    return Ok(url.to_string());
  }
  if !is_remote_http_url(url) {
    return Ok(url.to_string());
  }

  let dir = preview_images_dir(app)?;
  let path = cache_file_path(&dir, url);

  if path.is_file() {
    match read_cached_data_url(&path) {
      Ok(data_url) => {
        tracing::debug!(%url, path = %path.display(), "preview image cache hit");
        return Ok(data_url);
      }
      Err(e) => {
        tracing::warn!(
          path = %path.display(),
          error = %e,
          "preview image cache corrupt; re-downloading"
        );
        let _ = fs::remove_file(&path);
      }
    }
  }

  tracing::info!(%url, "downloading preview image into local cache");
  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(30))
    .build()
    .map_err(|e| format!("http client: {e}"))?;

  let response = client
    .get(url)
    .send()
    .await
    .map_err(|e| format!("preview download failed: {e}"))?;

  if !response.status().is_success() {
    return Err(format!(
      "preview download HTTP {} for {url}",
      response.status()
    ));
  }

  let header_mime = response
    .headers()
    .get(reqwest::header::CONTENT_TYPE)
    .and_then(|v| v.to_str().ok())
    .and_then(mime_from_content_type);

  if let Some(len) = response.content_length() {
    if len > MAX_PREVIEW_BYTES {
      return Err(format!(
        "preview image too large ({len} bytes, max {MAX_PREVIEW_BYTES})"
      ));
    }
  }

  let bytes = response
    .bytes()
    .await
    .map_err(|e| format!("preview body: {e}"))?;

  if bytes.len() as u64 > MAX_PREVIEW_BYTES {
    return Err(format!(
      "preview image too large ({} bytes, max {MAX_PREVIEW_BYTES})",
      bytes.len()
    ));
  }
  if bytes.is_empty() {
    return Err("preview image body empty".into());
  }

  let mime = sniff_mime(&bytes)
    .or(header_mime)
    .ok_or_else(|| format!("URL did not return a supported image type: {url}"))?;

  // Atomic-ish write: temp then rename.
  let tmp = path.with_extension("tmp");
  fs::write(&tmp, &bytes).map_err(|e| format!("write preview cache temp: {e}"))?;
  if path.exists() {
    let _ = fs::remove_file(&path);
  }
  fs::rename(&tmp, &path)
    .or_else(|_| {
      fs::copy(&tmp, &path)
        .and_then(|_| fs::remove_file(&tmp))
        .map(|_| ())
    })
    .map_err(|e| format!("commit preview cache {}: {e}", path.display()))?;

  tracing::info!(
    %url,
    path = %path.display(),
    bytes = bytes.len(),
    %mime,
    "preview image cached"
  );

  Ok(bytes_to_data_url(&bytes, mime))
}

/// Rewrite remote `preview_img` fields in place to local `data:` URLs (parallel downloads).
pub async fn localize_preview_images(app: &AppHandle, list: &mut [cdx_theme_types::ThemeMetadata]) {
  let mut set = tokio::task::JoinSet::new();

  for (i, item) in list.iter().enumerate() {
    let Some(url) = item.preview_img.clone() else {
      continue;
    };
    if !is_remote_http_url(&url) {
      continue;
    }
    let app = app.clone();
    set.spawn(async move {
      let result = resolve_to_data_url(&app, &url).await;
      (i, url, result)
    });
  }

  while let Some(joined) = set.join_next().await {
    match joined {
      Ok((i, _url, Ok(data_url))) => {
        if let Some(item) = list.get_mut(i) {
          item.preview_img = Some(data_url);
        }
      }
      Ok((_i, url, Err(e))) => {
        tracing::warn!(%url, error = %e, "failed to cache preview image; keeping remote url");
      }
      Err(e) => tracing::warn!("preview cache task join error: {e}"),
    }
  }
}
