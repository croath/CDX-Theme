//! Shared CDXTheme core: pack / unpack packages, load packages,
//! CDP inject, Codex launch, and high-level apply / restore.

pub mod appearance;
pub mod apply;
pub mod cdp;
pub mod codex_chat;
pub mod error;
pub mod inject;
pub mod launch;
pub mod layout;
pub mod pack;
pub mod package;
pub mod util;

pub use appearance::{
  APPEARANCE_THEME_KEY, AppearanceResult, AppearanceTheme, appearance_theme_value,
  codex_config_path, set_appearance_theme, upsert_appearance_theme,
};
pub use apply::{apply_theme, restore_theme};
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
  CSS_PARTIAL_TARGETS, CssMergeResult, EXT_CDXTHEME, FORMAT_CDXTHEME, MAX_THEME_PACKAGE_BYTES,
  PackThemeResult, THEME_SCHEMA_VERSION, ThemePackage, list_css_partials, maybe_merge_css_for_path,
  merge_css_for_path_in_memory, merge_css_partials_content, merge_css_target, merge_theme_css,
  merge_theme_css_optional, pack_theme_dir, pack_theme_dir_with_options, resolve_target_css,
  unpack_package,
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
  deserialize_version_u32, parse_version_u32,
};

pub use layout::{
  LayoutVerifyReport, click_tab, default_options as layout_default_options, probe, probe_layout,
  screenshot, verify_layout,
};

pub use codex_chat::{
  CodexChatOptions, CodexChatResult, CodexSessionDetail, CodexSessionMessage, CodexSessionSummary,
  CodexStreamCallback, ThemeBuilderRuntimeStatus, check_theme_builder_runtime,
  delete_codex_session, find_codex_cli, install_bun_for_theme_builder,
  list_sessions as list_codex_sessions, list_sessions_async as list_codex_sessions_async,
  load_session as load_codex_session, rename_codex_session,
  send_and_wait as codex_chat_send_and_wait, send_and_wait_with as codex_chat_send_and_wait_with,
};
