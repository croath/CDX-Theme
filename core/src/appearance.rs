//! Codex / ChatGPT desktop appearance settings under `~/.codex/config.toml`.
//!
//! Manages `[desktop].appearanceTheme` (`dark` / `light` / `system`).

use crate::error::{CoreError, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Key under `[desktop]` for the host light/dark/system mode.
pub const APPEARANCE_THEME_KEY: &str = "appearanceTheme";

/// Host appearance mode written to `appearanceTheme`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppearanceTheme {
  Dark,
  Light,
  System,
}

impl AppearanceTheme {
  pub fn as_str(self) -> &'static str {
    match self {
      AppearanceTheme::Dark => "dark",
      AppearanceTheme::Light => "light",
      AppearanceTheme::System => "system",
    }
  }

  pub fn parse(s: &str) -> Result<Self> {
    match s.trim().to_ascii_lowercase().as_str() {
      "dark" => Ok(AppearanceTheme::Dark),
      "light" => Ok(AppearanceTheme::Light),
      "system" => Ok(AppearanceTheme::System),
      other => Err(CoreError::msg(format!(
        "invalid appearance mode `{other}` (supported: dark, light, system)"
      ))),
    }
  }
}

impl std::fmt::Display for AppearanceTheme {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.as_str())
  }
}

/// Result of writing `[desktop].appearanceTheme`.
#[derive(Debug, Clone)]
pub struct AppearanceResult {
  pub mode: AppearanceTheme,
  pub previous: Option<String>,
  /// True when the config file content changed.
  pub changed: bool,
  pub config: PathBuf,
}

/// Default Codex config location (`~/.codex/config.toml` / `%USERPROFILE%\.codex\config.toml`).
pub fn codex_config_path() -> PathBuf {
  user_home_dir()
    .map(|h| h.join(".codex").join("config.toml"))
    .unwrap_or_else(|| PathBuf::from(".codex").join("config.toml"))
}

fn user_home_dir() -> Option<PathBuf> {
  #[cfg(windows)]
  {
    if let Some(p) = std::env::var_os("USERPROFILE").map(PathBuf::from) {
      if !p.as_os_str().is_empty() {
        return Some(p);
      }
    }
    if let (Some(drive), Some(path)) = (std::env::var_os("HOMEDRIVE"), std::env::var_os("HOMEPATH"))
    {
      let mut combined = PathBuf::from(drive);
      let path = PathBuf::from(path);
      if path.is_absolute() {
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
  }
  std::env::var_os("HOME")
    .or_else(|| std::env::var_os("USERPROFILE"))
    .map(PathBuf::from)
    .filter(|p| !p.as_os_str().is_empty())
}

/// Set ChatGPT / Codex `[desktop].appearanceTheme` to `dark`, `light`, or `system`.
///
/// Writes `config.toml` when the value changes. Does **not** restart the host;
/// callers that need the mode to apply immediately should restart Codex.
pub fn set_appearance_theme(
  mode: AppearanceTheme,
  config_path: Option<&Path>,
) -> Result<AppearanceResult> {
  let config = config_path
    .map(Path::to_path_buf)
    .unwrap_or_else(codex_config_path);

  if !config.is_file() {
    return Err(CoreError::msg(format!(
      "Codex config not found: {}",
      config.display()
    )));
  }

  let content = fs::read_to_string(&config)
    .map_err(|e| CoreError::msg(format!("failed to read config {}: {e}", config.display())))?;
  let previous = appearance_theme_value(&content);
  let updated = upsert_appearance_theme(&content, mode.as_str());
  let changed = content != updated;

  if changed {
    fs::write(&config, &updated)
      .map_err(|e| CoreError::msg(format!("failed to write config {}: {e}", config.display())))?;
  }

  Ok(AppearanceResult {
    mode,
    previous,
    changed,
    config,
  })
}

/// Read the normalized `appearanceTheme` value from a Codex config.toml string.
///
/// Looks in the `[desktop]` section only. Returns the bare value without quotes,
/// e.g. `Some("dark")` for `appearanceTheme = "dark"`.
pub fn appearance_theme_value(content: &str) -> Option<String> {
  let desktop_body = desktop_section_body(content)?;
  for line in desktop_body.lines() {
    let t = line.trim();
    if t.is_empty() || t.starts_with('#') {
      continue;
    }
    let Some((key, rhs)) = t.split_once('=') else {
      continue;
    };
    if key.trim() != APPEARANCE_THEME_KEY {
      continue;
    }
    let rhs = rhs.trim();
    if rhs.is_empty() {
      return None;
    }
    let unquoted = rhs
      .strip_prefix('"')
      .and_then(|s| s.strip_suffix('"'))
      .or_else(|| rhs.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
      .unwrap_or(rhs);
    return Some(unquoted.trim().to_string());
  }
  None
}

/// Upsert `appearanceTheme = "…"` under `[desktop]` (creates the section if missing).
pub fn upsert_appearance_theme(content: &str, mode: &str) -> String {
  let line = format!("{APPEARANCE_THEME_KEY} = \"{mode}\"");
  let content = ensure_desktop_section(content);
  map_desktop_body(&content, |body| {
    replace_unique_setting(body, APPEARANCE_THEME_KEY, &line)
  })
}

fn is_table_header(line: &str) -> bool {
  let t = line.trim();
  t.starts_with('[') && t.ends_with(']')
}

fn normalize_table_path(header: &str) -> Option<String> {
  let t = header.trim();
  let inner = t.strip_prefix('[')?.strip_suffix(']')?;
  let path = inner
    .split('.')
    .map(str::trim)
    .filter(|s| !s.is_empty())
    .collect::<Vec<_>>()
    .join(".");
  if path.is_empty() { None } else { Some(path) }
}

fn is_desktop_header(line: &str) -> bool {
  normalize_table_path(line).as_deref() == Some("desktop")
}

fn split_sections(content: &str) -> Vec<(Option<String>, String)> {
  let mut sections = Vec::new();
  let mut current_header: Option<String> = None;
  let mut current_body = String::new();

  for line in content.lines() {
    if is_table_header(line) {
      if current_header.is_some() || !current_body.is_empty() {
        sections.push((current_header.take(), std::mem::take(&mut current_body)));
      }
      current_header = Some(line.trim().to_string());
    } else {
      current_body.push_str(line);
      current_body.push('\n');
    }
  }
  if current_header.is_some() || !current_body.is_empty() {
    sections.push((current_header, current_body));
  }
  sections
}

fn join_sections(sections: &[(Option<String>, String)]) -> String {
  let mut out = String::new();
  for (header, body) in sections {
    if let Some(h) = header {
      out.push_str(h);
      out.push('\n');
    }
    out.push_str(body);
    if !out.ends_with('\n') && !body.is_empty() {
      out.push('\n');
    }
  }
  out
}

fn desktop_section_body(content: &str) -> Option<String> {
  split_sections(content)
    .into_iter()
    .find(|(h, _)| h.as_deref().is_some_and(is_desktop_header))
    .map(|(_, b)| b)
}

fn ensure_desktop_section(content: &str) -> String {
  let sections = split_sections(content);
  if sections
    .iter()
    .any(|(h, _)| h.as_deref().is_some_and(is_desktop_header))
  {
    return content.to_string();
  }
  let mut next = content.trim_end().to_string();
  if !next.is_empty() {
    next.push_str("\n\n");
  }
  next.push_str("[desktop]\n");
  next
}

fn map_desktop_body<F>(content: &str, mut f: F) -> String
where
  F: FnMut(&str) -> String,
{
  let content = ensure_desktop_section(content);
  let sections = split_sections(&content);
  let mut out = Vec::new();
  let mut mapped = false;
  for (header, body) in sections {
    if !mapped && header.as_deref().is_some_and(is_desktop_header) {
      out.push((header, f(&body)));
      mapped = true;
    } else {
      out.push((header, body));
    }
  }
  join_sections(&out)
}

fn replace_unique_setting(body: &str, key: &str, line: &str) -> String {
  let mut kept = String::new();
  for raw in body.lines() {
    let t = raw.trim();
    let is_key = t
      .split_once('=')
      .map(|(k, _)| k.trim() == key)
      .unwrap_or(false);
    if is_key {
      continue;
    }
    kept.push_str(raw);
    kept.push('\n');
  }
  let without = kept.trim_end();
  if without.is_empty() {
    format!("{line}\n")
  } else {
    format!("{without}\n{line}\n")
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn appearance_theme_value_reads_desktop_key() {
    let cfg = "[desktop]\nappearanceTheme = \"dark\"\nfoo = 1\n";
    assert_eq!(appearance_theme_value(cfg).as_deref(), Some("dark"));
  }

  #[test]
  fn upsert_replaces_and_creates() {
    let cfg = "[desktop]\nappearanceTheme = \"dark\"\nfoo = 1\n";
    let next = upsert_appearance_theme(cfg, "light");
    assert!(next.contains("appearanceTheme = \"light\""));
    assert!(!next.contains("appearanceTheme = \"dark\""));
    assert!(next.contains("foo = 1"));

    let bare = "model = \"gpt-5\"\n";
    let with = upsert_appearance_theme(bare, "system");
    assert!(with.contains("[desktop]"));
    assert!(with.contains("appearanceTheme = \"system\""));
    assert!(with.contains("model = \"gpt-5\""));
  }

  #[test]
  fn parse_modes() {
    assert_eq!(
      AppearanceTheme::parse("Dark").unwrap(),
      AppearanceTheme::Dark
    );
    assert_eq!(
      AppearanceTheme::parse("light").unwrap(),
      AppearanceTheme::Light
    );
    assert_eq!(
      AppearanceTheme::parse("SYSTEM").unwrap(),
      AppearanceTheme::System
    );
    assert!(AppearanceTheme::parse("auto").is_err());
  }
}
