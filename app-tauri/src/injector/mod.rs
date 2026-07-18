//! CDP theme injector — re-exports from `cdx-theme-core`.

pub use cdx_theme_core::inject::{
  InjectOptions, InjectRunResult, TargetResult, apply_loaded_theme, apply_theme_package,
  build_inject_expression, build_inject_expression_workbuddy, load_theme_package,
  restore_default_theme, verify_theme, DEFAULT_CDP_PORT,
};
pub use cdx_theme_core::{CdpTarget, wait_for_targets};
pub use cdx_theme_types::{
  BaseTheme, BaseThemeFonts, CodexLoadedTarget, CodexTargetOptions, CodexVerification,
  LoadedTargets, LoadedTheme, PublicTheme, SelectorCheck, SemanticColors, ThemeCopy,
  VerificationContext, VerificationWhen, WorkBuddyLoadedTarget, WorkBuddyVerification,
};

// Back-compat submodule path used by older call sites.
pub mod theme {
  pub use cdx_theme_core::inject::theme::{
    build_inject_expression, build_inject_expression_workbuddy, load_theme_package,
  };
  pub use cdx_theme_types::*;
}
