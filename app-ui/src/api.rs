use cdx_theme_types::ThemeMetadata;
use serde::Serialize;
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
  async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

  #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"], catch)]
  async fn listen(event: &str, handler: &js_sys::Function) -> Result<JsValue, JsValue>;
}

/// Payload of `theme-builder-acp-stream` events from the host.
#[derive(Clone, Debug, Default, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AcpStreamEvent {
  #[serde(default)]
  pub text: String,
  #[serde(default)]
  pub done: bool,
}

/// Handle returned by [`listen_theme_builder_acp_stream`]; drops to stop listening.
pub struct EventUnlisten {
  unlisten: Option<js_sys::Function>,
  _handler: Option<Closure<dyn FnMut(JsValue)>>,
}

impl EventUnlisten {
  pub fn unlisten(mut self) {
    if let Some(f) = self.unlisten.take() {
      let _ = f.call0(&JsValue::NULL);
    }
    self._handler = None;
  }
}

impl Drop for EventUnlisten {
  fn drop(&mut self) {
    if let Some(f) = self.unlisten.take() {
      let _ = f.call0(&JsValue::NULL);
    }
  }
}

/// Subscribe to live ACP transcript chunks while Theme Builder generates.
///
/// `on_event` is invoked on the browser event loop for each partial/final update.
pub async fn listen_theme_builder_acp_stream(
  on_event: impl Fn(AcpStreamEvent) + 'static,
) -> Result<EventUnlisten, String> {
  let cb = Closure::wrap(Box::new(move |event: JsValue| {
    // Tauri event shape: { event, id, payload }
    let payload = js_sys::Reflect::get(&event, &JsValue::from_str("payload")).unwrap_or(event);
    match from_value::<AcpStreamEvent>(payload.clone()) {
      Ok(ev) => on_event(ev),
      Err(_) => {
        if let Some(text) = payload.as_string() {
          on_event(AcpStreamEvent { text, done: false });
        }
      }
    }
  }) as Box<dyn FnMut(JsValue)>);

  let unlisten_val = listen("theme-builder-acp-stream", cb.as_ref().unchecked_ref())
    .await
    .map_err(js_err_to_string)?;

  let unlisten = unlisten_val.dyn_into::<js_sys::Function>().ok();

  Ok(EventUnlisten {
    unlisten,
    _handler: Some(cb),
  })
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

#[derive(Clone, Debug, Default, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodexChatResult {
  pub submitted: bool,
  pub reply: String,
  pub assistant_count: usize,
  pub stable: bool,
  pub message: String,
  #[serde(default)]
  pub binary: Option<String>,
  #[serde(default)]
  pub session_id: Option<String>,
  #[serde(default)]
  pub stop_reason: Option<String>,
  /// Packed `.cdxtheme` path in the workspace (ready for Apply).
  #[serde(default)]
  pub package_path: Option<String>,
  #[serde(default)]
  pub installed_theme_id: Option<String>,
  #[serde(default)]
  pub installed_theme_name: Option<String>,
  #[serde(default)]
  pub applied: bool,
}

#[derive(Clone, Debug, Default, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ApplyBuiltThemeResult {
  pub theme_id: String,
  pub theme_name: String,
  pub package_path: String,
  pub applied: bool,
}

#[derive(Clone, Debug, Default, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodexSessionSummary {
  pub id: String,
  pub title: String,
  pub updated_at: String,
  #[serde(default)]
  pub path: Option<String>,
  #[serde(default)]
  pub workspace_path: Option<String>,
}

#[derive(Clone, Debug, Default, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodexSessionMessage {
  pub role: String,
  pub content: String,
}

#[derive(Clone, Debug, Default, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodexSessionDetail {
  pub id: String,
  pub title: String,
  pub updated_at: String,
  pub messages: Vec<CodexSessionMessage>,
  #[serde(default)]
  pub workspace_path: Option<String>,
}

#[derive(Clone, Debug, Default, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PreparedWorkspace {
  pub workspace_id: String,
  pub workspace_path: String,
}

/// Create `{app_data_dir}/theme_builder/{id}` with bundled skill + theme scaffold.
pub async fn start_theme_build() -> Result<PreparedWorkspace, String> {
  invoke_cmd_with_args::<PreparedWorkspace>("start_theme_build", empty_args()).await
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct CodexChatArgs {
  prompt: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  session_id: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  workspace_path: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  workspace_id: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  wait_ms: Option<u64>,
}

/// Theme Builder: prompt Codex over ACP (`session/new|load` + `session/prompt`).
pub async fn codex_chat(
  prompt: impl Into<String>,
  session_id: Option<String>,
  workspace_path: Option<String>,
  workspace_id: Option<String>,
  wait_ms: Option<u64>,
) -> Result<CodexChatResult, String> {
  let args = to_value(&CodexChatArgs {
    prompt: prompt.into(),
    session_id,
    workspace_path,
    workspace_id,
    wait_ms,
  })
  .map_err(|e| e.to_string())?;
  invoke_cmd_with_args::<CodexChatResult>("codex_chat", args).await
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct ApplyBuiltThemeArgs {
  workspace_path: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  package_path: Option<String>,
}

/// Install the workspace `.cdxtheme` into `app_data_dir/themes` and apply it.
pub async fn apply_built_theme(
  workspace_path: impl Into<String>,
  package_path: Option<String>,
) -> Result<ApplyBuiltThemeResult, String> {
  let args = to_value(&ApplyBuiltThemeArgs {
    workspace_path: workspace_path.into(),
    package_path,
  })
  .map_err(|e| e.to_string())?;
  invoke_cmd_with_args::<ApplyBuiltThemeResult>("apply_built_theme", args).await
}

#[derive(Clone, Debug, Default, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SavedHeroImage {
  pub relative_path: String,
  pub theme_asset_path: String,
  pub file_name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct SaveThemeBuilderHeroArgs {
  workspace_path: String,
  file_name: String,
  content_base64: String,
}

/// Write a hero image into `{workspace}/theme/assets/hero.*` for Theme Builder.
pub async fn save_theme_builder_hero(
  workspace_path: impl Into<String>,
  file_name: impl Into<String>,
  content_base64: impl Into<String>,
) -> Result<SavedHeroImage, String> {
  let args = to_value(&SaveThemeBuilderHeroArgs {
    workspace_path: workspace_path.into(),
    file_name: file_name.into(),
    content_base64: content_base64.into(),
  })
  .map_err(|e| e.to_string())?;
  invoke_cmd_with_args::<SavedHeroImage>("save_theme_builder_hero", args).await
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct ListCodexSessionsArgs {
  #[serde(skip_serializing_if = "Option::is_none")]
  limit: Option<usize>,
}

/// Theme Builder: list saved Codex CLI sessions.
pub async fn list_codex_sessions(limit: Option<usize>) -> Result<Vec<CodexSessionSummary>, String> {
  let args = to_value(&ListCodexSessionsArgs { limit }).map_err(|e| e.to_string())?;
  invoke_cmd_with_args::<Vec<CodexSessionSummary>>("list_codex_sessions", args).await
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct DeleteThemeBuilderSessionArgs {
  session_id: String,
}

/// Theme Builder: delete a tracked session and its workspace.
pub async fn delete_theme_builder_session(session_id: impl Into<String>) -> Result<bool, String> {
  let args = to_value(&DeleteThemeBuilderSessionArgs {
    session_id: session_id.into(),
  })
  .map_err(|e| e.to_string())?;
  // Accept bool or null/empty payload from Tauri.
  match invoke_cmd_with_args::<bool>("delete_theme_builder_session", args).await {
    Ok(v) => Ok(v),
    Err(e) if e.contains("invalid type") || e.contains("null") || e.contains("undefined") => {
      Ok(true)
    }
    Err(e) => Err(e),
  }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct GetCodexSessionArgs {
  session_id: String,
}

/// Theme Builder: load a session transcript for the chat view.
pub async fn get_codex_session(
  session_id: impl Into<String>,
) -> Result<CodexSessionDetail, String> {
  let args = to_value(&GetCodexSessionArgs {
    session_id: session_id.into(),
  })
  .map_err(|e| e.to_string())?;
  invoke_cmd_with_args::<CodexSessionDetail>("get_codex_session", args).await
}

/// Host runtime probe for Theme Builder (`codex-acp` / bunx / npx).
#[derive(Clone, Debug, Default, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ThemeBuilderRuntimeStatus {
  pub ready: bool,
  #[serde(default)]
  pub has_codex_acp: bool,
  #[serde(default)]
  pub has_bun: bool,
  #[serde(default)]
  pub has_bunx: bool,
  #[serde(default)]
  pub has_npx: bool,
  #[serde(default)]
  pub runner: Option<String>,
  #[serde(default)]
  pub runner_path: Option<String>,
  #[serde(default)]
  pub message: String,
}

/// Theme Builder: check whether bunx/npx/codex-acp is available on the host.
pub async fn check_theme_builder_runtime() -> Result<ThemeBuilderRuntimeStatus, String> {
  invoke_cmd_with_args::<ThemeBuilderRuntimeStatus>("check_theme_builder_runtime", empty_args())
    .await
}

/// Theme Builder: install Bun via multi-mirror download (official / GitHub / jsDelivr).
pub async fn install_bun_for_theme_builder() -> Result<ThemeBuilderRuntimeStatus, String> {
  invoke_cmd_with_args::<ThemeBuilderRuntimeStatus>("install_bun_for_theme_builder", empty_args())
    .await
}

/// Explicit `$pageleave` (e.g. app hide). Usually handled inside `capture_pageview`.
#[allow(dead_code)]
pub async fn track_page_leave(page: Option<&str>) {
  crate::posthog::capture_pageleave(page);
}
