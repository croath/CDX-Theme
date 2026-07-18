//! Portable multi-app theme package format — re-exports from `cdx-theme-core`.

pub use cdx_theme_core::package::{
  ACTIVE_APP_ID, APP_CODEX, APP_WORKBUDDY, CodexThemePeek, DEFAULT_APP_ID, EXT_CDXTHEME,
  EXT_CODEDROBE, FORMAT_CDXTHEME, FORMAT_CODEDROBE, MAX_THEME_PACKAGE_BYTES, THEME_EXTENSION,
  THEME_PACKAGE_EXTENSIONS, THEME_SCHEMA_VERSION, css_has_remote_resources, is_cdx_theme_file,
  is_supported_package_format, is_theme_package_content, is_theme_package_filename,
  load_cdx_theme_file, peek_codex_theme_meta,
};
