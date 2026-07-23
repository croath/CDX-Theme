//! Loaded multi-app theme package types (shared by app backend).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// OpenAI Codex / ChatGPT desktop target key.
pub const APP_CODEX: &str = "codex";
/// WorkBuddy host target key.
pub const APP_WORKBUDDY: &str = "workbuddy";

// ── Theme identity / chrome copy ────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeCopy {
  pub brand_title: String,
  pub brand_subtitle: String,
  pub signature: String,
  pub tagline: String,
  pub project_prefix: String,
  pub project_label: String,
  pub ribbon: String,
}

impl Default for ThemeCopy {
  fn default() -> Self {
    Self {
      brand_title: "Codex 自定义皮肤".into(),
      brand_subtitle: "AI Crafted Theme ✦".into(),
      signature: "Codex ♡".into(),
      tagline: "把灵感写进每一天 ♡".into(),
      project_prefix: "选择项目 · ".into(),
      project_label: "♡  选择项目".into(),
      ribbon: "🎀".into(),
    }
  }
}

/// Public theme identity sent into the renderer inject payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicTheme {
  pub id: String,
  pub display_name: String,
  /// Package version (integer).
  pub version: u32,
  pub copy: ThemeCopy,
}

// ── Shared verification types ───────────────────────────────────────────────

/// A named check with one or more CSS selectors (`any` = match if any selector hits).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorCheck {
  pub name: String,
  pub any: Vec<String>,
}

/// Context activation: when any of these selectors match.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationWhen {
  pub any: Vec<String>,
}

/// Contextual verification block (`contexts[]` entry).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationContext {
  pub name: String,
  pub when: VerificationWhen,
  #[serde(default)]
  pub required: Vec<SelectorCheck>,
  #[serde(default)]
  pub recommended: Vec<SelectorCheck>,
}

// ── Codex target ────────────────────────────────────────────────────────────

/// `targets.codex.options`
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexTargetOptions {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub renderer_profile: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub base_theme: Option<BaseTheme>,
}

/// Host appearance / chrome theme (`options.baseTheme`).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseTheme {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub mode: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub code_theme: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub accent: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub contrast: Option<f64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub ink: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub surface: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub opaque_windows: Option<bool>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub fonts: Option<BaseThemeFonts>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub semantic_colors: Option<SemanticColors>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseThemeFonts {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub mac_code: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub mac_ui: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub windows_code: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub windows_ui: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticColors {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub diff_added: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub diff_removed: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub skill: Option<String>,
}

/// `targets.codex.verification`
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexVerification {
  #[serde(default)]
  pub contexts: Vec<VerificationContext>,
  #[serde(default)]
  pub required: Vec<SelectorCheck>,
  #[serde(default)]
  pub recommended: Vec<SelectorCheck>,
}

/// Loaded `targets.codex` entry (CSS kept in memory from package JSON).
#[derive(Clone, Debug)]
pub struct CodexLoadedTarget {
  /// Inline CSS from `targets.codex.css`.
  pub css: String,
  pub options: Option<CodexTargetOptions>,
  pub verification: Option<CodexVerification>,
}

impl CodexLoadedTarget {
  pub fn base_theme(&self) -> Option<&BaseTheme> {
    self.options.as_ref().and_then(|o| o.base_theme.as_ref())
  }

  pub fn renderer_profile(&self) -> Option<&str> {
    self
      .options
      .as_ref()
      .and_then(|o| o.renderer_profile.as_deref())
  }
}

// ── WorkBuddy target ────────────────────────────────────────────────────────

/// `targets.workbuddy.verification`
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkBuddyVerification {
  #[serde(default)]
  pub required: Vec<SelectorCheck>,
  #[serde(default)]
  pub recommended: Vec<SelectorCheck>,
  #[serde(default)]
  pub contexts: Vec<VerificationContext>,
}

/// Loaded `targets.workbuddy` entry (no options/baseTheme today).
#[derive(Clone, Debug)]
pub struct WorkBuddyLoadedTarget {
  /// Inline CSS from `targets.workbuddy.css`.
  pub css: String,
  pub verification: Option<WorkBuddyVerification>,
}

/// Package image asset kept in memory (from `assets.images.*` / `assets.art`).
#[derive(Clone, Debug)]
pub struct LoadedArt {
  pub mime_type: String,
  /// Raw base64 payload (no `data:` prefix).
  pub base64: String,
}

impl LoadedArt {
  /// `data:{mime};base64,{payload}` for CDP inject / CSS custom properties.
  pub fn data_url(&self) -> String {
    format!("data:{};base64,{}", self.mime_type, self.base64)
  }
}

// ── Targets container ───────────────────────────────────────────────────────

/// Known host targets from a multi-app package.
#[derive(Clone, Debug, Default)]
pub struct LoadedTargets {
  pub codex: Option<CodexLoadedTarget>,
  pub workbuddy: Option<WorkBuddyLoadedTarget>,
}

impl LoadedTargets {
  pub fn app_ids(&self) -> Vec<&'static str> {
    let mut ids = Vec::new();
    if self.codex.is_some() {
      ids.push(APP_CODEX);
    }
    if self.workbuddy.is_some() {
      ids.push(APP_WORKBUDDY);
    }
    ids
  }
}

// ── Loaded package ──────────────────────────────────────────────────────────

/// Fully loaded package ready for apply / inject.
///
/// Parsed directly from the theme JSON file — no extract-to-disk step.
#[derive(Clone, Debug)]
pub struct LoadedTheme {
  pub id: String,
  pub display_name: String,
  /// Package version (integer).
  pub version: u32,
  pub copy: ThemeCopy,
  /// All package images (`assets.images`), e.g. `hero`, `texture`.
  /// Inject sets `--cdxtheme-image-{name}` for each entry.
  pub images: BTreeMap<String, LoadedArt>,
  /// Convenience alias for `images["hero"]` (catalog preview / polaroid).
  pub art: Option<LoadedArt>,
  /// Absolute path of the source package file.
  pub package_path: PathBuf,
  /// Known host targets (`codex`, `workbuddy`).
  pub targets: LoadedTargets,
  // pub assets:
}

impl LoadedTheme {
  pub fn public(&self) -> PublicTheme {
    PublicTheme {
      id: self.id.clone(),
      display_name: self.display_name.clone(),
      version: self.version,
      copy: self.copy.clone(),
    }
  }

  pub fn app_ids(&self) -> Vec<&'static str> {
    self.targets.app_ids()
  }

  /// Codex target (required for apply today).
  pub fn codex(&self) -> Result<&CodexLoadedTarget, String> {
    self.targets.codex.as_ref().ok_or_else(|| {
      format!(
        "theme `{}` has no target `codex` (available: {})",
        self.id,
        self.app_ids().join(", ")
      )
    })
  }

  pub fn workbuddy(&self) -> Option<&WorkBuddyLoadedTarget> {
    self.targets.workbuddy.as_ref()
  }

  /// Target for the currently active host (today: `codex`).
  pub fn active_target(&self) -> Result<&CodexLoadedTarget, String> {
    self.codex()
  }

  pub fn active_css(&self) -> Result<&str, String> {
    Ok(self.codex()?.css.as_str())
  }

  pub fn active_base_theme(&self) -> Option<&BaseTheme> {
    self.targets.codex.as_ref().and_then(|t| t.base_theme())
  }
}
