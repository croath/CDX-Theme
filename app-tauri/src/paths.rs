//! Theme search paths: bundled resources + user local data.
//! Path helpers are written to behave correctly on Windows (UNC `\\?\` prefixes, case).

use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, path::BaseDirectory};

/// Bundled / dev themes directory (`…/themes` resource or repo `themes/`).
pub fn builtin_themes_dir(app: &AppHandle) -> Option<PathBuf> {
  if let Ok(path) = app.path().resolve("themes", BaseDirectory::Resource) {
    if path.is_dir() {
      return Some(path);
    }
  }
  // `CARGO_MANIFEST_DIR/../themes` works on Windows too (PathBuf normalizes separators).
  let dev = Path::new(env!("CARGO_MANIFEST_DIR")).join("../themes");
  if dev.is_dir() {
    return Some(dev);
  }
  None
}

/// User-writable themes: `{app_local_data_dir}/themes`.
pub fn user_themes_dir(app: &AppHandle) -> Result<PathBuf, String> {
  let base = app
    .path()
    .app_local_data_dir()
    .map_err(|e| format!("app local data dir: {e}"))?;
  let dir = base.join("themes");
  fs::create_dir_all(&dir).map_err(|e| format!("create user themes dir {}: {e}", dir.display()))?;
  Ok(dir)
}

/// Roots to scan for themes (builtin first, then user). Missing builtin is ok.
pub fn theme_search_dirs(app: &AppHandle) -> Result<Vec<PathBuf>, String> {
  let mut dirs = Vec::new();
  if let Some(builtin) = builtin_themes_dir(app) {
    dirs.push(builtin);
  }
  dirs.push(user_themes_dir(app)?);
  if dirs.is_empty() {
    return Err("no theme directories available".into());
  }
  Ok(dirs)
}

/// Resolve a catalog `location` (absolute path preferred; relative searched in theme dirs).
pub fn resolve_theme_location(app: &AppHandle, location: &str) -> Result<PathBuf, String> {
  let loc = location.trim();
  if loc.is_empty() {
    return Err("empty theme location".into());
  }

  let as_path = PathBuf::from(loc);
  // Themes are package files only (.cdxtheme).
  if (as_path.is_absolute() || looks_absolute_windows(loc)) && as_path.is_file() {
    return Ok(as_path);
  }

  let rel = loc.trim_start_matches("./").trim_start_matches(".\\");
  for root in theme_search_dirs(app)? {
    let candidate = root.join(rel);
    if candidate.is_file() {
      return Ok(candidate);
    }
  }

  Err(format!(
    "theme package not found for location `{loc}` (expected a .cdxtheme file under builtin or local_data/themes)"
  ))
}

// Back-compat alias used by older call sites.
pub fn themes_dir(app: &AppHandle) -> Result<PathBuf, String> {
  builtin_themes_dir(app).ok_or_else(|| "builtin themes directory not found".to_string())
}

/// Strip Windows `\\?\` / `//?/` extended path prefixes for stable comparisons.
pub fn strip_verbatim_prefix(path: &Path) -> PathBuf {
  let s = path.to_string_lossy();
  if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
    PathBuf::from(format!(r"\\{rest}"))
  } else if let Some(rest) = s.strip_prefix(r"\\?\") {
    PathBuf::from(rest)
  } else if let Some(rest) = s.strip_prefix("//?/") {
    PathBuf::from(rest)
  } else {
    path.to_path_buf()
  }
}

/// Canonical absolute path as a string suitable for catalog storage.
pub fn abs_location_string(path: &Path) -> String {
  let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
  strip_verbatim_prefix(&abs).to_string_lossy().into_owned()
}

/// True if `child` is the same as or nested under `parent` (Windows-safe).
pub fn path_is_under(child: &Path, parent: &Path) -> bool {
  let child = strip_verbatim_prefix(&child.canonicalize().unwrap_or_else(|_| child.to_path_buf()));
  let parent = strip_verbatim_prefix(
    &parent
      .canonicalize()
      .unwrap_or_else(|_| parent.to_path_buf()),
  );

  #[cfg(windows)]
  {
    let c = child
      .to_string_lossy()
      .replace('/', "\\")
      .to_ascii_lowercase();
    let p = parent
      .to_string_lossy()
      .replace('/', "\\")
      .to_ascii_lowercase();
    let p = p.trim_end_matches('\\');
    c == p || c.starts_with(&format!("{p}\\"))
  }
  #[cfg(not(windows))]
  {
    child.starts_with(&parent)
  }
}

fn looks_absolute_windows(s: &str) -> bool {
  // C:\… or \\server\share or \\?\…
  let bytes = s.as_bytes();
  if bytes.len() >= 3
    && bytes[0].is_ascii_alphabetic()
    && bytes[1] == b':'
    && (bytes[2] == b'\\' || bytes[2] == b'/')
  {
    return true;
  }
  s.starts_with(r"\\") || s.starts_with("//")
}

/// User home directory (USERPROFILE / HOME / HOMEDRIVE+HOMEPATH).
pub fn user_home_dir() -> Option<PathBuf> {
  #[cfg(windows)]
  {
    if let Some(p) = std::env::var_os("USERPROFILE").map(PathBuf::from) {
      if !p.as_os_str().is_empty() {
        return Some(p);
      }
    }
    // HOMEDRIVE=C: + HOMEPATH=\Users\name
    if let (Some(drive), Some(path)) = (std::env::var_os("HOMEDRIVE"), std::env::var_os("HOMEPATH"))
    {
      let mut combined = PathBuf::from(drive);
      // HOMEPATH is usually like `\Users\foo` — join carefully.
      let path = PathBuf::from(path);
      if path.is_absolute() {
        // On Windows, `\Users\foo` is absolute-relative to current drive.
        let s = format!(
          "{}{}",
          combined.to_string_lossy().trim_end_matches(['\\', '/']),
          path.to_string_lossy()
        );
        return Some(PathBuf::from(s));
      }
      combined.push(path);
      return Some(combined);
    }
    std::env::var_os("HOME").map(PathBuf::from)
  }
  #[cfg(not(windows))]
  {
    std::env::var_os("HOME")
      .or_else(|| std::env::var_os("USERPROFILE"))
      .map(PathBuf::from)
  }
}
