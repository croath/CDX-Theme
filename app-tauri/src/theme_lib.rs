//! Theme library helpers (ported from `scripts/theme-lib.mjs`).
//! Handles Codex `config.toml` `[desktop]` appearance settings.

use cdx_theme_types::{BaseTheme, BaseThemeFonts, LoadedTheme};
use regex::Regex;
use std::collections::BTreeMap;
use std::sync::LazyLock;

static SETTING_LINE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^(?P<key>[A-Za-z0-9_.]+)\s*=").expect("setting line regex"));

fn toml_string(value: &str) -> String {
  serde_json::to_string(value).unwrap_or_else(|_| format!("\"{value}\""))
}

/// Resolve UI/code fonts from typed theme fonts for the current compile target.
#[cfg(target_os = "macos")]
fn platform_fonts(fonts: Option<&BaseThemeFonts>) -> (&str, &str) {
  (
    fonts
      .and_then(|f| f.mac_code.as_deref())
      .unwrap_or("SF Mono"),
    fonts
      .and_then(|f| f.mac_ui.as_deref())
      .unwrap_or("PingFang SC"),
  )
}

#[cfg(target_os = "windows")]
fn platform_fonts(fonts: Option<&BaseThemeFonts>) -> (&str, &str) {
  (
    fonts
      .and_then(|f| f.windows_code.as_deref())
      .unwrap_or("Cascadia Code"),
    fonts
      .and_then(|f| f.windows_ui.as_deref())
      .unwrap_or("Microsoft YaHei UI"),
  )
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn platform_fonts(fonts: Option<&BaseThemeFonts>) -> (&str, &str) {
  (
    fonts
      .and_then(|f| f.windows_code.as_deref())
      .unwrap_or("Cascadia Code"),
    fonts
      .and_then(|f| f.windows_ui.as_deref())
      .unwrap_or("sans-serif"),
  )
}

/// Build the three managed `[desktop]` appearance lines for a host target.
///
/// Currently only `codex` has `options.baseTheme`. Other targets fall back to defaults.
pub fn build_base_theme_settings(theme: &LoadedTheme, target: &str) -> BTreeMap<String, String> {
  let empty = BaseTheme::default();
  let base = match target {
    cdx_theme_types::APP_CODEX => theme
      .targets
      .codex
      .as_ref()
      .and_then(|t| t.base_theme())
      .unwrap_or(&empty),
    _ => &empty,
  };
  let fonts = base.fonts.as_ref();
  let semantic = base.semantic_colors.as_ref();

  let (code_font, ui_font) = platform_fonts(fonts);

  let accent = base.accent.as_deref().unwrap_or("#B65CFF");
  let contrast = base.contrast.unwrap_or(64.0);
  let ink = base.ink.as_deref().unwrap_or("#4A235F");
  let surface = base.surface.as_deref().unwrap_or("#FFF4FA");
  let opaque = base.opaque_windows.unwrap_or(true);
  let mode = base.mode.as_deref().unwrap_or("light");
  let code_theme = base.code_theme.as_deref().unwrap_or("codex");
  let diff_added = semantic
    .and_then(|s| s.diff_added.as_deref())
    .unwrap_or("#BCE8CF");
  let diff_removed = semantic
    .and_then(|s| s.diff_removed.as_deref())
    .unwrap_or("#F7B8CE");
  let skill = semantic
    .and_then(|s| s.skill.as_deref())
    .unwrap_or("#C47BFF");

  let contrast_txt = if contrast.fract() == 0.0 {
    format!("{}", contrast as i64)
  } else {
    contrast.to_string()
  };

  let chrome = format!(
    "appearanceLightChromeTheme = {{ accent = {}, contrast = {}, fonts = {{ code = {}, ui = {} }}, ink = {}, opaqueWindows = {}, semanticColors = {{ diffAdded = {}, diffRemoved = {}, skill = {} }}, surface = {} }}",
    toml_string(accent),
    contrast_txt,
    toml_string(code_font),
    toml_string(ui_font),
    toml_string(ink),
    if opaque { "true" } else { "false" },
    toml_string(diff_added),
    toml_string(diff_removed),
    toml_string(skill),
    toml_string(surface),
  );

  let mut settings = BTreeMap::new();
  settings.insert(
    "appearanceTheme".into(),
    format!("appearanceTheme = {}", toml_string(mode)),
  );
  settings.insert(
    "appearanceLightCodeThemeId".into(),
    format!("appearanceLightCodeThemeId = {}", toml_string(code_theme)),
  );
  settings.insert("appearanceLightChromeTheme".into(), chrome);
  settings
}

fn is_table_header(line: &str) -> bool {
  let t = line.trim();
  t.starts_with('[') && t.ends_with(']')
}

/// Normalize a TOML table header path, trimming whitespace around segments.
/// e.g. `[desktop.appearanceLightChromeTheme ]` → `desktop.appearanceLightChromeTheme`
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

fn is_managed_chrome_header(line: &str) -> bool {
  let Some(path) = normalize_table_path(line) else {
    return false;
  };
  path == "desktop.appearanceLightChromeTheme"
    || path.starts_with("desktop.appearanceLightChromeTheme.")
}

fn has_managed_chrome_tables(content: &str) -> bool {
  split_sections(content)
    .iter()
    .any(|(h, _)| h.as_deref().is_some_and(is_managed_chrome_header))
}

/// Last managed setting line under `[desktop]`, if any.
fn desktop_setting_line(content: &str, key: &str) -> Option<String> {
  let desktop_body = split_sections(content)
    .into_iter()
    .find(|(h, _)| h.as_deref().is_some_and(is_desktop_header))
    .map(|(_, b)| b)?;
  setting_pattern(key)
    .find_iter(&desktop_body)
    .last()
    .map(|m| m.as_str().trim().to_string())
}

/// Split content into non-overlapping sections: either a TOML table or a preamble block.
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

fn setting_pattern(key: &str) -> Regex {
  let escaped = regex::escape(key);
  Regex::new(&format!(r"(?m)^{escaped}\s*=.*(?:\r?\n|$)")).expect("setting pattern")
}

fn replace_unique_setting(body: &str, key: &str, line: &str) -> String {
  let without = setting_pattern(key).replace_all(body, "").to_string();
  let without = without.trim_end();
  if without.is_empty() {
    format!("{line}\n")
  } else {
    format!("{without}\n{line}\n")
  }
}

fn remove_managed_chrome_tables(content: &str) -> String {
  let sections = split_sections(content);
  let filtered: Vec<_> = sections
    .into_iter()
    .filter(|(header, _)| {
      header
        .as_deref()
        .map(|h| !is_managed_chrome_header(h))
        .unwrap_or(true)
    })
    .collect();
  join_sections(&filtered)
}

/// Capture nested `[desktop.appearanceLightChromeTheme…]` tables for restore.
/// Matches CodeDrobe `extractManagedChromeTables`.
fn extract_managed_chrome_tables(content: &str) -> String {
  split_sections(content)
    .into_iter()
    .filter(|(header, _)| header.as_deref().is_some_and(is_managed_chrome_header))
    .map(|(header, body)| {
      let mut block = String::new();
      if let Some(h) = header {
        block.push_str(&h);
        block.push('\n');
      }
      block.push_str(&body);
      block.trim_end().to_string()
    })
    .filter(|s| !s.is_empty())
    .collect::<Vec<_>>()
    .join("\n\n")
}

/// Remove managed keys only from the **root** (before the first `[table]`).
/// Matches CodeDrobe `removeMisplacedRootSettings` — does **not** strip keys
/// already living under `[desktop]` (those are replaced in-place later).
fn remove_misplaced_root_settings(content: &str, keys: &[&str]) -> String {
  static FIRST_TABLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\[").expect("first table regex"));
  let root_end = FIRST_TABLE
    .find(content)
    .map(|m| m.start())
    .unwrap_or(content.len());

  let mut root = content[..root_end].to_string();
  for key in keys {
    root = setting_pattern(key).replace_all(&root, "").to_string();
  }
  format!("{root}{}", &content[root_end..])
}

fn merge_desktop_sections(content: &str) -> String {
  let sections = split_sections(content);
  let mut desktop_bodies = Vec::new();
  let mut others = Vec::new();
  let mut first_desktop_idx: Option<usize> = None;

  for (header, body) in sections {
    if header.as_deref().is_some_and(is_desktop_header) {
      if first_desktop_idx.is_none() {
        first_desktop_idx = Some(others.len());
        others.push((header, String::new()));
      }
      let trimmed = body.trim();
      if !trimmed.is_empty() {
        desktop_bodies.push(trimmed.to_string());
      }
    } else {
      others.push((header, body));
    }
  }

  if let Some(idx) = first_desktop_idx {
    let merged = desktop_bodies.join("\n");
    let body = if merged.is_empty() {
      String::new()
    } else {
      format!("{merged}\n")
    };
    others[idx].1 = body;
  }

  join_sections(&others)
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

/// Apply managed appearance settings into a Codex config.toml string.
/// Pipeline matches CodeDrobe `applyCodexSettings`:
/// remove nested chrome tables → strip root misplaced keys → merge `[desktop]` → upsert lines.
pub fn apply_settings(content: &str, settings: &BTreeMap<String, String>) -> String {
  let keys: Vec<&str> = settings.keys().map(|s| s.as_str()).collect();
  let cleaned = merge_desktop_sections(&remove_misplaced_root_settings(
    &remove_managed_chrome_tables(content),
    &keys,
  ));
  map_desktop_body(&cleaned, |body| {
    let mut next = body.to_string();
    for (key, line) in settings {
      next = replace_unique_setting(&next, key, line);
    }
    next
  })
}

/// Restore managed appearance keys from a pre-apply backup into the current config.
/// Matches CodeDrobe `restoreCodexSettings` (including nested chrome tables from backup).
pub fn restore_settings(current: &str, backup: &str, keys: &[&str]) -> String {
  let saved_chrome_tables = extract_managed_chrome_tables(backup);
  let cleaned = merge_desktop_sections(&remove_misplaced_root_settings(
    &remove_managed_chrome_tables(current),
    keys,
  ));

  let backup_body = split_sections(backup)
    .into_iter()
    .find(|(h, _)| h.as_deref().is_some_and(is_desktop_header))
    .map(|(_, b)| b)
    .unwrap_or_default();

  let restored = map_desktop_body(&cleaned, |body| {
    let mut next = body.to_string();
    for key in keys {
      let pattern = setting_pattern(key);
      let saved = pattern
        .find_iter(&backup_body)
        .last()
        .map(|m| m.as_str().trim_end().to_string());
      next = pattern.replace_all(&next, "").to_string();
      next = next.trim_end().to_string();
      // If backup stored chrome as nested tables, prefer those over the inline key.
      if *key == "appearanceLightChromeTheme" && !saved_chrome_tables.is_empty() {
        if !next.is_empty() && !next.ends_with('\n') {
          next.push('\n');
        }
        continue;
      }
      if let Some(saved) = saved {
        if next.is_empty() {
          next = format!("{saved}\n");
        } else {
          next = format!("{next}\n{saved}\n");
        }
      } else if !next.is_empty() && !next.ends_with('\n') {
        next.push('\n');
      }
    }
    next
  });

  if saved_chrome_tables.is_empty() {
    restored
  } else {
    format!("{}\n\n{}\n", restored.trim_end(), saved_chrome_tables)
  }
}

pub const MANAGED_SETTINGS_KEYS: &[&str] = &[
  "appearanceTheme",
  "appearanceLightCodeThemeId",
  "appearanceLightChromeTheme",
];

/// Key used for light/dark mode under `[desktop]`.
pub const APPEARANCE_THEME_KEY: &str = "appearanceTheme";

/// Read the normalized `appearanceTheme` value from a Codex config.toml string.
///
/// Looks in the `[desktop]` section only. Returns the bare value without quotes,
/// e.g. `Some("dark")` for `appearanceTheme = "dark"`.
pub fn appearance_theme_value(content: &str) -> Option<String> {
  let line = desktop_setting_line(content, APPEARANCE_THEME_KEY)?;

  // `appearanceTheme = "dark"` or `appearanceTheme = dark`
  let rhs = line
    .split_once('=')
    .map(|(_, v)| v.trim())
    .filter(|v| !v.is_empty())?;
  let unquoted = rhs
    .strip_prefix('"')
    .and_then(|s| s.strip_suffix('"'))
    .or_else(|| rhs.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
    .unwrap_or(rhs);
  Some(unquoted.trim().to_string())
}

/// True when `appearanceTheme` (light/dark) differs between two config contents.
pub fn appearance_theme_changed(before: &str, after: &str) -> bool {
  appearance_theme_value(before) != appearance_theme_value(after)
}

/// True when any managed appearance setting differs (or a nested chrome override
/// table is present only on one side). Codex loads these at startup, so callers
/// should restart when this is true.
pub fn managed_appearance_changed(before: &str, after: &str) -> bool {
  if has_managed_chrome_tables(before) != has_managed_chrome_tables(after) {
    return true;
  }
  MANAGED_SETTINGS_KEYS
    .iter()
    .any(|key| desktop_setting_line(before, key) != desktop_setting_line(after, key))
}

// silence unused warning for SETTING_LINE if we don't use it
#[allow(dead_code)]
fn _setting_key(line: &str) -> Option<&str> {
  SETTING_LINE
    .captures(line)
    .and_then(|c| c.name("key").map(|m| m.as_str()))
}

#[cfg(test)]
mod tests {
  use super::*;
  use cdx_theme_types::{
    APP_CODEX, BaseTheme, CodexLoadedTarget, CodexTargetOptions, LoadedTargets, LoadedTheme,
    SemanticColors, ThemeCopy,
  };
  use std::path::PathBuf;

  fn sample_theme() -> LoadedTheme {
    let base = BaseTheme {
      mode: Some("light".into()),
      code_theme: Some("codex".into()),
      accent: Some("#B85F6C".into()),
      contrast: Some(78.0),
      ink: Some("#38292C".into()),
      surface: Some("#FBF7F3".into()),
      opaque_windows: Some(true),
      fonts: Some(BaseThemeFonts {
        mac_code: Some("SF Mono".into()),
        mac_ui: Some("PingFang SC".into()),
        windows_code: Some("Cascadia Code".into()),
        windows_ui: Some("Microsoft YaHei UI".into()),
      }),
      semantic_colors: Some(SemanticColors {
        diff_added: Some("#CFE7D7".into()),
        diff_removed: Some("#F1C7CB".into()),
        skill: Some("#C87582".into()),
      }),
    };
    LoadedTheme {
      id: "demo".into(),
      display_name: "Demo".into(),
      version: 1,
      copy: ThemeCopy::default(),
      images: Default::default(),
      art: None,
      package_path: PathBuf::from("demo.cdxtheme"),
      targets: LoadedTargets {
        codex: Some(CodexLoadedTarget {
          css: ":root { --demo: 1; }".into(),
          options: Some(CodexTargetOptions {
            renderer_profile: Some("codex-theme-v1".into()),
            base_theme: Some(base),
          }),
          verification: None,
        }),
        workbuddy: None,
      },
    }
  }

  #[test]
  fn appearance_theme_value_reads_desktop_key() {
    let cfg = "[desktop]\nappearanceTheme = \"dark\"\nfoo = 1\n";
    assert_eq!(appearance_theme_value(cfg).as_deref(), Some("dark"));
    assert!(!appearance_theme_changed(cfg, cfg));
    let light = "[desktop]\nappearanceTheme = \"light\"\n";
    assert!(appearance_theme_changed(cfg, light));
  }

  #[test]
  fn apply_writes_unique_keys() {
    let theme = sample_theme();
    let settings = build_base_theme_settings(&theme, APP_CODEX);
    let updated = apply_settings("[desktop]\nfoo = 1\n", &settings);
    assert_eq!(updated.matches("appearanceTheme =").count(), 1);
    assert_eq!(updated.matches("appearanceLightCodeThemeId =").count(), 1);
    assert_eq!(updated.matches("appearanceLightChromeTheme =").count(), 1);
    assert!(updated.contains("foo = 1"));
    assert!(updated.contains("[desktop]"));
  }

  #[test]
  fn apply_strips_nested_chrome_tables_with_odd_whitespace() {
    let theme = sample_theme();
    let settings = build_base_theme_settings(&theme, APP_CODEX);
    // Real-world Codex configs sometimes keep a nested override table (with odd
    // spacing inside the header) that wins over the inline chrome inline table.
    let original = r##"[desktop]
conversationDetailMode = "STEPS_PROSE"
appearanceLightChromeTheme = { accent = "#74ACDF", contrast = 90, fonts = { code = "SF Mono", ui = "SF Pro Text" }, ink = "#FFFFFF", opaqueWindows = true, semanticColors = { diffAdded = "#0A3022", diffRemoved = "#4E0D14", skill = "#E5A93B" }, surface = "#0A1120" }
appearanceLightCodeThemeId = "codex"
appearanceTheme = "dark"
[desktop.appearanceLightChromeTheme ]
accent = "#0169cc"
contrast = 78
ink = "#0d0d0d"
opaqueWindows = true
surface = "#ffffff"

[desktop.open-in-target-preferences]
global = "systemDefault"
"##;
    let updated = apply_settings(original, &settings);
    assert!(
      !updated.contains("[desktop.appearanceLightChromeTheme"),
      "nested chrome override must be removed:\n{updated}"
    );
    assert_eq!(updated.matches("appearanceLightChromeTheme =").count(), 1);
    assert!(updated.contains("#B85F6C"), "theme accent must be written");
    assert!(updated.contains("appearanceTheme = \"light\""));
    assert!(updated.contains("[desktop.open-in-target-preferences]"));
    assert!(managed_appearance_changed(original, &updated));
  }

  #[test]
  fn managed_appearance_changed_detects_chrome_only_edits() {
    let before = r##"[desktop]
appearanceTheme = "light"
appearanceLightCodeThemeId = "codex"
appearanceLightChromeTheme = { accent = "#000000", contrast = 1, fonts = { code = "SF Mono", ui = "PingFang SC" }, ink = "#111111", opaqueWindows = true, semanticColors = { diffAdded = "#a", diffRemoved = "#b", skill = "#c" }, surface = "#ffffff" }
"##;
    let after = r##"[desktop]
appearanceTheme = "light"
appearanceLightCodeThemeId = "codex"
appearanceLightChromeTheme = { accent = "#B85F6C", contrast = 78, fonts = { code = "SF Mono", ui = "PingFang SC" }, ink = "#38292C", opaqueWindows = true, semanticColors = { diffAdded = "#CFE7D7", diffRemoved = "#F1C7CB", skill = "#C87582" }, surface = "#FBF7F3" }
"##;
    assert!(!appearance_theme_changed(before, after));
    assert!(managed_appearance_changed(before, after));
    assert!(!managed_appearance_changed(before, before));
  }

  #[test]
  fn restore_reverts_managed_keys() {
    let theme = sample_theme();
    let settings = build_base_theme_settings(&theme, APP_CODEX);
    let original = "[desktop]\nappearanceTheme = \"dark\"\nfoo = 1\n";
    let applied = apply_settings(original, &settings);
    let restored = restore_settings(&applied, original, MANAGED_SETTINGS_KEYS);
    assert!(restored.contains("appearanceTheme = \"dark\""));
    assert!(restored.contains("foo = 1"));
    assert!(!restored.contains("#B85F6C"));
  }

  #[test]
  fn remove_misplaced_only_touches_root_not_desktop() {
    // CodeDrobe removeMisplacedRootSettings: root key gone, desktop key kept for upsert.
    let content = r##"appearanceTheme = "dark"
foo = 1
[desktop]
appearanceTheme = "light"
bar = 2
"##;
    let cleaned = remove_misplaced_root_settings(content, &["appearanceTheme"]);
    assert!(
      !cleaned
        .lines()
        .next()
        .unwrap_or("")
        .starts_with("appearanceTheme"),
      "root key must be stripped:\n{cleaned}"
    );
    assert!(
      cleaned.contains("appearanceTheme = \"light\""),
      "desktop key must remain until replace:\n{cleaned}"
    );
  }

  #[test]
  fn restore_prefers_nested_chrome_tables_from_backup() {
    let current = r##"[desktop]
appearanceTheme = "light"
appearanceLightChromeTheme = { accent = "#B85F6C", contrast = 78, fonts = { code = "SF Mono", ui = "PingFang SC" }, ink = "#38292C", opaqueWindows = true, semanticColors = { diffAdded = "#a", diffRemoved = "#b", skill = "#c" }, surface = "#FBF7F3" }
"##;
    let backup = r##"[desktop]
appearanceTheme = "dark"
[desktop.appearanceLightChromeTheme]
accent = "#0169cc"
contrast = 78
surface = "#ffffff"
"##;
    let restored = restore_settings(current, backup, MANAGED_SETTINGS_KEYS);
    assert!(restored.contains("appearanceTheme = \"dark\""));
    assert!(
      restored.contains("[desktop.appearanceLightChromeTheme]"),
      "nested chrome from backup must be restored:\n{restored}"
    );
    assert!(
      !restored.contains("appearanceLightChromeTheme = {"),
      "inline chrome must not be re-inserted when nested tables exist:\n{restored}"
    );
    assert!(restored.contains("#0169cc"));
  }
}
