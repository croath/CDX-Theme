use cdx_theme_types::ThemeMetadata;
use serde::Serialize;
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
  async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

fn js_err_to_string(err: JsValue) -> String {
  err
    .as_string()
    .or_else(|| {
      js_sys::JSON::stringify(&err)
        .ok()
        .and_then(|s| s.as_string())
    })
    .unwrap_or_else(|| "unknown error".into())
}

fn empty_args() -> JsValue {
  JsValue::from(js_sys::Object::new())
}

async fn invoke_cmd_with_args<T>(cmd: &str, args: JsValue) -> Result<T, String>
where
  T: for<'de> serde::Deserialize<'de>,
{
  let value = invoke(cmd, args).await.map_err(js_err_to_string)?;
  from_value(value).map_err(|e| e.to_string())
}

async fn invoke_unit_with_args(cmd: &str, args: JsValue) -> Result<(), String> {
  invoke(cmd, args).await.map_err(js_err_to_string)?;
  Ok(())
}

pub async fn retrieve_local_theme_list() -> Result<Vec<ThemeMetadata>, String> {
  match invoke_cmd_with_args::<Vec<ThemeMetadata>>("retrieve_local_theme_list", empty_args()).await
  {
    Ok(list) => Ok(list),
    Err(e) => Err(e),
  }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct FetchRemoteThemeCatalogArgs {
  force: bool,
}

/// Remote recommend catalog (`https://s3.cdxtheme.com/themes/index.json`).
/// When `force` is true, the backend clears its cache and re-fetches.
pub async fn fetch_remote_theme_catalog(force: bool) -> Result<Vec<ThemeMetadata>, String> {
  let args = to_value(&FetchRemoteThemeCatalogArgs { force }).map_err(|e| e.to_string())?;
  invoke_cmd_with_args::<Vec<ThemeMetadata>>("fetch_remote_theme_catalog", args).await
}

#[derive(Clone, Debug, Default, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CdpTargetInfo {
  pub id: String,
  pub title: String,
  pub url: String,
}

#[derive(Clone, Debug, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CdpServerStatus {
  pub connected: bool,
  pub port: u16,
  pub target_count: usize,
  pub targets: Vec<CdpTargetInfo>,
  pub message: String,
}

impl Default for CdpServerStatus {
  fn default() -> Self {
    Self {
      connected: false,
      port: 9335,
      target_count: 0,
      targets: vec![],
      message: "…".into(),
    }
  }
}

pub async fn cdp_status() -> Result<CdpServerStatus, String> {
  invoke_cmd_with_args::<CdpServerStatus>("cdp_status", empty_args()).await
}

#[derive(Serialize)]
struct SetWindowAppearanceArgs {
  dark: bool,
}

/// Match native window background to light/dark UI (opaque window, no private API).
pub async fn set_window_appearance(dark: bool) -> Result<(), String> {
  let args = to_value(&SetWindowAppearanceArgs { dark }).map_err(|e| e.to_string())?;
  match invoke_unit_with_args("set_window_appearance", args).await {
    Ok(()) => Ok(()),
    Err(e) if e.contains("__TAURI__") || e.contains("undefined") => Ok(()),
    Err(e) => Err(e),
  }
}

pub async fn get_cdp_port() -> Result<u16, String> {
  invoke_cmd_with_args::<u16>("get_cdp_port", empty_args()).await
}

#[derive(Serialize)]
struct SetCdpPortArgs {
  port: u16,
}

pub async fn set_cdp_port(port: u16) -> Result<u16, String> {
  let args = to_value(&SetCdpPortArgs { port }).map_err(|e| e.to_string())?;
  invoke_cmd_with_args::<u16>("set_cdp_port", args).await
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct ApplyThemeArgs {
  theme_id: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  theme_url: Option<String>,
}

/// Apply a theme by id. For remote catalog entries, pass `theme_url` so the package
/// is downloaded into the library first.
pub async fn apply_theme(
  theme_id: impl Into<String>,
  theme_url: Option<String>,
) -> Result<bool, String> {
  let args = to_value(&ApplyThemeArgs {
    theme_id: theme_id.into(),
    theme_url,
  })
  .map_err(|e| e.to_string())?;
  match invoke_cmd_with_args::<bool>("apply_theme", args).await {
    Ok(ok) => Ok(ok),
    Err(e) if e.contains("__TAURI__") || e.contains("undefined") => Ok(true),
    Err(e) => Err(e),
  }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct DownloadThemeArgs {
  theme_url: String,
}

/// Download a remote package into the local library only (no apply).
pub async fn download_theme(theme_url: impl Into<String>) -> Result<ThemeMetadata, String> {
  let args = to_value(&DownloadThemeArgs {
    theme_url: theme_url.into(),
  })
  .map_err(|e| e.to_string())?;
  invoke_cmd_with_args::<ThemeMetadata>("download_theme", args).await
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct ResolveCachedImageArgs {
  url: String,
}

/// Resolve an image URL to a local `data:` URL (HTTP(S) images are disk-cached).
/// `data:` and other non-remote values are returned unchanged by the backend.
pub async fn resolve_cached_image(url: impl Into<String>) -> Result<String, String> {
  let url = url.into();
  let trimmed = url.trim();
  if trimmed.is_empty() {
    return Err("empty image url".into());
  }
  // Already local — skip the IPC round-trip.
  if trimmed.starts_with("data:") {
    return Ok(trimmed.to_string());
  }
  let args = to_value(&ResolveCachedImageArgs {
    url: trimmed.to_string(),
  })
  .map_err(|e| e.to_string())?;
  match invoke_cmd_with_args::<String>("resolve_cached_image", args).await {
    Ok(local) => Ok(local),
    Err(e) if e.contains("__TAURI__") || e.contains("undefined") => Ok(url),
    Err(e) => Err(e),
  }
}

pub async fn restore_theme() -> Result<(), String> {
  match invoke_unit_with_args("restore_theme", empty_args()).await {
    Ok(()) => Ok(()),
    Err(e) if e.contains("__TAURI__") || e.contains("undefined") => Ok(()),
    Err(e) => Err(e),
  }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct InstallThemeArgs {
  file_name: String,
  content: String,
}

/// Install a portable `.cdxtheme` package (raw JSON text).
pub async fn install_theme(
  file_name: impl Into<String>,
  content: impl Into<String>,
) -> Result<ThemeMetadata, String> {
  let args = to_value(&InstallThemeArgs {
    file_name: file_name.into(),
    content: content.into(),
  })
  .map_err(|e| e.to_string())?;
  invoke_cmd_with_args::<ThemeMetadata>("install_theme", args).await
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct DeleteThemeArgs {
  theme_id: String,
}

/// Remove a user-installed theme package from the local library.
pub async fn delete_theme(theme_id: impl Into<String>) -> Result<bool, String> {
  let args = to_value(&DeleteThemeArgs {
    theme_id: theme_id.into(),
  })
  .map_err(|e| e.to_string())?;
  invoke_cmd_with_args::<bool>("delete_theme", args).await
}

pub async fn get_analytics_enabled() -> Result<bool, String> {
  invoke_cmd_with_args::<bool>("get_analytics_enabled", empty_args()).await
}

#[derive(Clone, Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsState {
  pub enabled: bool,
  pub distinct_id: String,
  #[allow(dead_code)]
  pub configured: bool,
}

pub async fn get_analytics_state() -> Result<AnalyticsState, String> {
  invoke_cmd_with_args::<AnalyticsState>("get_analytics_state", empty_args()).await
}

#[derive(Serialize)]
struct SetAnalyticsEnabledArgs {
  enabled: bool,
}

pub async fn set_analytics_enabled(enabled: bool) -> Result<bool, String> {
  let args = to_value(&SetAnalyticsEnabledArgs { enabled }).map_err(|e| e.to_string())?;
  let result = invoke_cmd_with_args::<bool>("set_analytics_enabled", args).await;
  // Keep the HTML PostHog snippet in sync with the persisted preference.
  if let Ok(saved) = result.as_ref() {
    if let Ok(state) = get_analytics_state().await {
      crate::posthog::apply_state(*saved, &state.distinct_id);
    } else {
      crate::posthog::set_enabled(*saved);
    }
    // After opt-in, send a standard `$pageview` so PostHog install check can pass.
    if *saved {
      crate::posthog::capture_pageview("settings");
    }
  }
  result
}

/// Pull install analytics state and sync posthog-js (identify + opt-in).
/// Returns whether capturing is enabled after sync.
pub async fn sync_posthog_js() -> bool {
  match get_analytics_state().await {
    Ok(state) => crate::posthog::apply_state(state.enabled, &state.distinct_id),
    Err(_) => {
      if let Ok(enabled) = get_analytics_enabled().await {
        crate::posthog::set_enabled(enabled);
        enabled
      } else {
        false
      }
    }
  }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct TrackEventArgs {
  name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  properties: Option<serde_json::Map<String, serde_json::Value>>,
}

/// Fire a allow-listed product analytics event via the native SDK
/// (no-op if analytics is off / not configured). Prefer this for non-page UI events.
#[allow(dead_code)]
pub async fn track_event(
  name: impl Into<String>,
  properties: Option<serde_json::Map<String, serde_json::Value>>,
) -> Result<(), String> {
  let args = to_value(&TrackEventArgs {
    name: name.into(),
    properties,
  })
  .map_err(|e| e.to_string())?;
  match invoke_unit_with_args("track_event", args).await {
    Ok(()) => Ok(()),
    Err(e) if e.contains("__TAURI__") || e.contains("undefined") => Ok(()),
    Err(e) => Err(e),
  }
}

pub async fn track_page_viewed(page: &str) {
  // PostHog standard `$pageview` (+ automatic `$pageleave` for the previous page).
  // No-op while opted out or when POSTHOG_API_KEY was not baked into the build.
  crate::posthog::capture_pageview(page);
}

/// Explicit `$pageleave` (e.g. app hide). Usually handled inside `capture_pageview`.
#[allow(dead_code)]
pub async fn track_page_leave(page: Option<&str>) {
  crate::posthog::capture_pageleave(page);
}
