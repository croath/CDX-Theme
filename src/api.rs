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
