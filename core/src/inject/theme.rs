//! Theme package loading for CDP injection.
//! Runtime themes are portable package files only (`.cdxtheme` / `.codedrobe-theme`).
//! Packages are plain JSON: load = read + parse; CSS/art stay in memory.
//!
//! Loaded theme types live in `cdx-theme-types` and are re-exported here.

use serde_json::Value;
use std::path::Path;

pub use cdx_theme_types::{
  APP_CODEX, APP_WORKBUDDY, BaseTheme, BaseThemeFonts, CodexLoadedTarget, CodexTargetOptions,
  CodexVerification, LoadedArt, LoadedTargets, LoadedTheme, PublicTheme, SelectorCheck,
  SemanticColors, ThemeCopy, VerificationContext, VerificationWhen, WorkBuddyLoadedTarget,
  WorkBuddyVerification,
};

const DEFAULT_ART_PNG_B64: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M/wHwAEAQH/2eN+WQAAAABJRU5ErkJggg==";

/// Load a portable theme package from a file path.
///
/// Validity is determined by **JSON content** (format / schema / targets), not the filename.
/// Reads and parses the file in place — no extract-to-disk.
pub fn load_theme_package(package_path: impl AsRef<Path>) -> Result<LoadedTheme, String> {
  let path = package_path.as_ref();
  if !path.is_file() {
    return Err(format!(
      "theme package path is not a file: {}",
      path.display()
    ));
  }
  if !crate::package::is_cdx_theme_file(path) {
    return Err(format!(
      "file is not a valid theme package (JSON with format cdxtheme|codedrobe-theme): {}",
      path.display()
    ));
  }

  crate::package::load_cdx_theme_file(path)
}

/// Build the Runtime.evaluate expression for the **Codex** host target.
///
/// Payload: `{ theme, cssText, imageDataUrls }` with **cdxtheme-only** branding.
/// Package CSS that still uses `codedrobe-*` tokens is rewritten to `cdxtheme-*`
/// before inject so multi-image vars / skin classes match the runtime.
pub fn build_inject_expression(theme: &LoadedTheme) -> Result<(String, PublicTheme), String> {
  let target = theme.codex()?;
  build_inject_from_css(theme, &target.css, cdx_theme_types::APP_CODEX)
}

/// Build inject expression for the WorkBuddy host (when wired up).
pub fn build_inject_expression_workbuddy(
  theme: &LoadedTheme,
) -> Result<(String, PublicTheme), String> {
  let target = theme.workbuddy().ok_or_else(|| {
    format!(
      "theme `{}` has no target `workbuddy` (available: {})",
      theme.id,
      theme.app_ids().join(", ")
    )
  })?;
  build_inject_from_css(theme, &target.css, cdx_theme_types::APP_WORKBUDDY)
}

/// Collect package images as data-URL map (`imageDataUrls` + optional art→hero).
fn build_image_data_urls(theme: &LoadedTheme) -> serde_json::Map<String, Value> {
  let mut image_map = serde_json::Map::new();
  for (name, art) in &theme.images {
    if art.base64.is_empty() {
      continue;
    }
    image_map.insert(name.clone(), Value::String(art.data_url()));
  }
  // If no images.hero but art exists, use art as hero.
  if !image_map.contains_key("hero") {
    if let Some(art) = &theme.art {
      if !art.base64.is_empty() {
        image_map.insert("hero".into(), Value::String(art.data_url()));
      }
    }
  }
  // Last-resort 1×1 so chrome CSS vars never go missing entirely.
  if image_map.is_empty() {
    image_map.insert(
      "hero".into(),
      Value::String(format!("data:image/png;base64,{DEFAULT_ART_PNG_B64}")),
    );
  }
  image_map
}

fn build_inject_from_css(
  theme: &LoadedTheme,
  css: &str,
  app_id: &str,
) -> Result<(String, PublicTheme), String> {
  if crate::util::css_has_remote_resources(css) {
    return Err(format!(
      "theme {} target `{app_id}` contains remote CSS resources; use local assets only",
      theme.id
    ));
  }

  // Prefer simple global rewrite (matches pack/convert).
  let css = crate::pack::rewrite_css_codedrobe_to_cdxtheme(css);
  let images = Value::Object(build_image_data_urls(theme));
  let public = theme.public();
  let template = include_str!("../../../assets/renderer-inject.js");
  // Placeholders must be valid JS literals (JSON-encoded). Raw CSS breaks the
  // Runtime.evaluate expression: `var cssText = :root { ... }` is a syntax error.
  let expression = template
    .replace(
      "__DREAM_CSS_JSON__",
      &serde_json::to_string(&css).map_err(|e| format!("encode css: {e}"))?,
    )
    .replace(
      "__DREAM_IMAGES_JSON__",
      &serde_json::to_string(&images).map_err(|e| format!("encode images: {e}"))?,
    )
    .replace(
      "__DREAM_THEME_JSON__",
      &serde_json::to_string(&public).map_err(|e| format!("encode theme: {e}"))?,
    );

  if expression.contains("__DREAM_") {
    return Err("inject template placeholders were not fully substituted".into());
  }

  Ok((expression, public))
}

#[cfg(test)]
mod tests {
  use super::*;
  use cdx_theme_types::{
    CodexLoadedTarget, CodexTargetOptions, LoadedArt, LoadedTargets, LoadedTheme, ThemeCopy,
  };
  use std::path::PathBuf;

  fn demo_theme(css: &str) -> LoadedTheme {
    LoadedTheme {
      id: "demo".into(),
      display_name: "Demo".into(),
      version: "1.0.0".into(),
      copy: ThemeCopy::default(),
      images: Default::default(),
      art: None,
      package_path: PathBuf::from("demo.cdxtheme"),
      targets: LoadedTargets {
        codex: Some(CodexLoadedTarget {
          css: css.into(),
          options: Some(CodexTargetOptions {
            renderer_profile: Some("codex-theme-v1".into()),
            base_theme: None,
          }),
          verification: None,
        }),
        workbuddy: None,
      },
    }
  }

  #[test]
  fn inject_expression_json_encodes_css() {
    let css = r#":root.cdxtheme-codex-skin { color: #f00; content: "x"; }"#;
    let theme = demo_theme(css);
    let (expression, public) = build_inject_expression(&theme).expect("build inject");
    assert_eq!(public.id, "demo");
    // Must be a JS string literal, not raw CSS.
    assert!(
      expression.contains(r#"var cssText = ":root.cdxtheme-codex-skin"#)
        || expression.contains("var cssText = \":root.cdxtheme-codex-skin"),
      "css must be JSON-string encoded:\n{}",
      &expression[..expression
        .find("var imageDataUrls")
        .unwrap_or(200)
        .min(expression.len())]
    );
    assert!(
      !expression.contains("var cssText = :root"),
      "raw CSS must not be spliced into the expression"
    );
    // Quotes inside CSS are escaped for JS.
    assert!(expression.contains(r#"\"x\""#) || expression.contains(r#"\\"x\\""#));
  }

  #[test]
  fn inject_rewrites_legacy_codedrobe_css_to_cdxtheme() {
    let theme = demo_theme(
      r#":root.codedrobe-codex-skin { background: var(--codedrobe-image-hero); }
#codedrobe-codex-skin-chrome .dream-polaroid { display: block; }"#,
    );
    let (expression, _) = build_inject_expression(&theme).expect("build inject");
    assert!(
      expression.contains("cdxtheme-codex-skin")
        && expression.contains("--cdxtheme-image-hero")
        && expression.contains("cdxtheme-codex-skin-chrome"),
      "legacy codedrobe CSS tokens must be rewritten to cdxtheme"
    );
    assert!(
      !expression.contains("codedrobe-codex-skin")
        && !expression.contains("--codedrobe-image-")
        && !expression.contains("codedrobe-codex-skin-chrome"),
      "codedrobe brand tokens must not remain in inject payload"
    );
  }

  #[test]
  fn inject_expression_includes_all_package_images() {
    let mut theme = demo_theme(":root.cdxtheme-codex-skin{}");
    theme.images.insert(
      "hero".into(),
      LoadedArt {
        mime_type: "image/png".into(),
        base64: "AAAHERO".into(),
      },
    );
    theme.images.insert(
      "texture".into(),
      LoadedArt {
        mime_type: "image/png".into(),
        base64: "AAATEXTURE".into(),
      },
    );
    theme.art = theme.images.get("hero").cloned();
    let (expression, _) = build_inject_expression(&theme).expect("build inject");
    assert!(
      expression.contains("data:image/png;base64,AAAHERO"),
      "hero image must be in inject payload"
    );
    assert!(
      expression.contains("data:image/png;base64,AAATEXTURE"),
      "texture image must be in inject payload"
    );
    assert!(
      expression.contains("cdxtheme-codex-skin-chrome"),
      "chrome id must be cdxtheme"
    );
    assert!(
      expression.contains("--cdxtheme-image-"),
      "inject must set --cdxtheme-image-*"
    );
    assert!(
      !expression.contains("--codedrobe-image-"),
      "inject must not set --codedrobe-image-*"
    );
  }
}
