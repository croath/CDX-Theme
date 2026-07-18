//! Shared CDXTheme core: pack / unpack / convert packages, load packages,
//! CDP inject, Codex launch, and high-level apply.

pub mod apply;
pub mod cdp;
pub mod error;
pub mod inject;
pub mod launch;
pub mod pack;
pub mod package;
pub mod util;

pub use apply::apply_theme;
pub use cdp::{CdpTarget, wait_for_targets};
pub use error::{CoreError, Result};
pub use inject::{
  DEFAULT_CDP_PORT, InjectOptions, InjectRunResult, TargetResult, apply_loaded_theme,
  apply_theme_package, build_inject_expression, build_inject_expression_workbuddy,
  load_theme_package, restore_default_theme, verify_theme,
};
pub use launch::{
  ensure_codex_debugging, ensure_codex_debugging_with_log, find_chatgpt_app,
  restart_codex_debugging, restart_codex_debugging_with_log,
};
pub use pack::{
  EXT_CDXTHEME, EXT_CODEDROBE, FORMAT_CDXTHEME, FORMAT_CODEDROBE, MAX_THEME_PACKAGE_BYTES,
  PackageFormat, THEME_SCHEMA_VERSION, ThemePackage, convert_package, pack_theme_dir,
  rewrite_css_codedrobe_to_cdxtheme, unpack_package,
};
pub use package::{
  ACTIVE_APP_ID, APP_CODEX, APP_WORKBUDDY, CodexThemePeek, THEME_EXTENSION,
  THEME_PACKAGE_EXTENSIONS, is_cdx_theme_file, is_supported_package_format,
  is_theme_package_content, is_theme_package_filename, load_cdx_theme_file, peek_codex_theme_meta,
};

// Re-export loaded types commonly needed by hosts.
pub use cdx_theme_types::{
  BaseTheme, BaseThemeFonts, CodexLoadedTarget, CodexTargetOptions, CodexVerification, LoadedArt,
  LoadedTargets, LoadedTheme, PublicTheme, SelectorCheck, SemanticColors, ThemeCopy,
  VerificationContext, VerificationWhen, WorkBuddyLoadedTarget, WorkBuddyVerification,
};
