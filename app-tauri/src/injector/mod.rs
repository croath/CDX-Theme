//! CDP theme injector — re-exports from `cdx-theme-core`.

pub use cdx_theme_core::inject::{
  DEFAULT_CDP_PORT, DEFAULT_WORKBUDDY_CDP_PORT, InjectOptions, InjectRunResult, TargetResult,
  apply_loaded_theme, apply_loaded_theme_for_app, apply_theme_package, apply_theme_package_for_app,
  build_inject_expression, build_inject_expression_workbuddy, load_theme_package,
  restore_default_theme, restore_default_theme_for_app, verify_theme,
};
pub use cdx_theme_core::{CdpTarget, TargetUrlKind, wait_for_targets, wait_for_targets_with};
pub use cdx_theme_types::{
  APP_CODEX, APP_WORKBUDDY, BaseTheme, BaseThemeFonts, CodexLoadedTarget, CodexTargetOptions,
  CodexVerification, LoadedTargets, LoadedTheme, PublicTheme, SelectorCheck, SemanticColors,
  ThemeCopy, VerificationContext, VerificationWhen, WorkBuddyLoadedTarget, WorkBuddyVerification,
};

// Back-compat submodule path used by older call sites.
pub mod theme {
  pub use cdx_theme_core::inject::theme::{
    build_inject_expression, build_inject_expression_workbuddy, load_theme_package,
  };
  pub use cdx_theme_types::*;
}
