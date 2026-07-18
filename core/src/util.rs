//! Shared helpers for package load and inject.

use cdx_theme_types::ThemeCopy;
use serde_json::Value;

pub fn is_named_theme(value: &str) -> bool {
  let mut chars = value.chars();
  match chars.next() {
    Some(c) if c.is_ascii_alphanumeric() => {
      chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    }
    _ => false,
  }
}

pub fn merge_copy(raw: Option<&Value>) -> ThemeCopy {
  let mut copy = ThemeCopy::default();
  let Some(Value::Object(map)) = raw else {
    return copy;
  };
  if let Some(Value::String(s)) = map.get("brandTitle") {
    copy.brand_title = s.clone();
  }
  if let Some(Value::String(s)) = map.get("brandSubtitle") {
    copy.brand_subtitle = s.clone();
  }
  if let Some(Value::String(s)) = map.get("signature") {
    copy.signature = s.clone();
  }
  if let Some(Value::String(s)) = map.get("tagline") {
    copy.tagline = s.clone();
  }
  if let Some(Value::String(s)) = map.get("projectPrefix") {
    copy.project_prefix = s.clone();
  }
  if let Some(Value::String(s)) = map.get("projectLabel") {
    copy.project_label = s.clone();
  }
  if let Some(Value::String(s)) = map.get("ribbon") {
    copy.ribbon = s.clone();
  }
  copy
}

/// Reject `@import` and remote `url(http...)`.
pub fn css_has_remote_resources(css: &str) -> bool {
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
