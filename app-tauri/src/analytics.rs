//! Product analytics via PostHog (anonymous usage events).
//!
//! Configuration (compile-time, public project API key is safe to embed):
//! - `POSTHOG_API_KEY` — PostHog project API key (`phc_…`)
//! - `POSTHOG_HOST` — optional host, default `https://us.i.posthog.com`
//!   (use `https://eu.i.posthog.com` for EU cloud)
//!
//! Collection is **opt-in** (default off). Users enable it in Settings.
//! When disabled, or when no API key was baked in, all capture calls are no-ops.

use crate::settings_store::{self, AppSettings};
use posthog_rs::{Client, ClientOptionsBuilder, Event};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use tauri::AppHandle;
use uuid::Uuid;

/// Public snapshot for the Leptos UI (shared with posthog-js identify).
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsState {
  pub enabled: bool,
  pub distinct_id: String,
  /// True when this build was compiled with `POSTHOG_API_KEY`.
  pub configured: bool,
}

/// Compile-time project token. Empty → analytics disabled for this build.
const API_KEY: Option<&str> = option_env!("POSTHOG_API_KEY");
/// Optional override; empty falls back to PostHog US ingestion.
const HOST: Option<&str> = option_env!("POSTHOG_HOST");

static ANALYTICS: OnceLock<Analytics> = OnceLock::new();

pub struct Analytics {
  client: Option<Client>,
  distinct_id: Mutex<String>,
  enabled: AtomicBool,
  app_version: String,
}

impl Analytics {
  fn configured() -> bool {
    API_KEY.map(|k| !k.trim().is_empty()).unwrap_or(false)
  }

  /// Initialize once after settings are loaded. Safe to call only from setup.
  pub async fn init(app: &AppHandle) {
    if ANALYTICS.get().is_some() {
      return;
    }

    let settings = settings_store::load(app);
    let distinct_id = ensure_distinct_id(app, &settings);
    let enabled = settings.analytics_enabled;
    let app_version = app.package_info().version.to_string();

    let client = if Self::configured() {
      let api_key = API_KEY.unwrap_or("").trim().to_string();
      let mut builder = ClientOptionsBuilder::default();
      builder.api_key(api_key);
      // Desktop client — allow OS attribution on the person, not "server".
      builder.is_server(false);
      if let Some(host) = HOST.map(str::trim).filter(|h| !h.is_empty()) {
        builder.host(host.to_string());
      }
      builder.flush_interval_ms(3_000);
      builder.flush_at(10);
      builder.on_error(|err| {
        tracing::debug!(error = ?err, "posthog capture error");
      });

      match builder.build() {
        Ok(opts) => {
          let client = posthog_rs::client(opts).await;
          tracing::info!(
            host = HOST.unwrap_or("default-us"),
            enabled,
            "PostHog analytics ready"
          );
          Some(client)
        }
        Err(e) => {
          tracing::warn!("PostHog options invalid: {e}");
          None
        }
      }
    } else {
      tracing::info!("PostHog not configured (set POSTHOG_API_KEY at build time)");
      None
    };

    let analytics = Analytics {
      client,
      distinct_id: Mutex::new(distinct_id),
      enabled: AtomicBool::new(enabled),
      app_version,
    };

    if ANALYTICS.set(analytics).is_err() {
      return;
    }

    // Session start
    capture("app_opened", HashMap::new());
  }

  fn get() -> Option<&'static Analytics> {
    ANALYTICS.get()
  }

  pub fn is_enabled() -> bool {
    Self::get()
      .map(|a| a.enabled.load(Ordering::Relaxed))
      .unwrap_or(false)
  }

  pub fn is_configured() -> bool {
    Self::configured()
  }

  pub fn distinct_id() -> Option<String> {
    Self::get().and_then(|a| a.distinct_id.lock().ok().map(|g| g.clone()))
  }

  /// Snapshot for the web UI (opt-in flag + shared anonymous person id).
  pub fn state(app: &AppHandle) -> AnalyticsState {
    let settings = settings_store::load(app);
    let distinct_id = Self::distinct_id().unwrap_or_else(|| {
      // Init may not have finished yet; still surface a stable id from settings.
      ensure_distinct_id(app, &settings)
    });
    AnalyticsState {
      enabled: settings.analytics_enabled,
      distinct_id,
      configured: Self::configured(),
    }
  }

  pub fn set_enabled(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let mut settings = settings_store::load(app);
    settings.analytics_enabled = enabled;
    settings_store::save(app, &settings)?;
    if let Some(a) = Self::get() {
      a.enabled.store(enabled, Ordering::Relaxed);
    }
    if enabled {
      capture(
        "analytics_enabled",
        HashMap::from([("source".into(), Value::String("settings".into()))]),
      );
    } else {
      // Best-effort final event before opt-out sticks for subsequent calls.
      capture_force(
        "analytics_disabled",
        HashMap::from([("source".into(), Value::String("settings".into()))]),
      );
    }
    Ok(())
  }

  pub async fn shutdown() {
    if let Some(a) = Self::get() {
      if let Some(client) = a.client.as_ref() {
        client.shutdown().await;
      }
    }
  }
}

fn ensure_distinct_id(app: &AppHandle, settings: &AppSettings) -> String {
  if let Some(id) = settings
    .analytics_distinct_id
    .as_ref()
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty())
  {
    return id;
  }
  let id = Uuid::new_v4().to_string();
  let mut next = settings.clone();
  next.analytics_distinct_id = Some(id.clone());
  if let Err(e) = settings_store::save(app, &next) {
    tracing::warn!("persist analytics distinct_id: {e}");
  }
  id
}

fn common_props(app_version: &str) -> HashMap<String, Value> {
  let mut props = HashMap::new();
  props.insert("app_version".into(), Value::String(app_version.to_string()));
  props.insert("os".into(), Value::String(std::env::consts::OS.to_string()));
  props.insert(
    "arch".into(),
    Value::String(std::env::consts::ARCH.to_string()),
  );
  props.insert("$lib".into(), Value::String("cdxtheme-rust".into()));
  props
}

/// Capture a product event if analytics is enabled and configured.
pub fn capture(event: &str, props: HashMap<String, Value>) {
  capture_inner(event, props, false);
}

/// Capture even when the user is turning analytics off (one last opt-out event).
fn capture_force(event: &str, props: HashMap<String, Value>) {
  capture_inner(event, props, true);
}

fn capture_inner(event: &str, props: HashMap<String, Value>, force: bool) {
  let Some(analytics) = Analytics::get() else {
    return;
  };
  if !force && !analytics.enabled.load(Ordering::Relaxed) {
    return;
  }
  let Some(client) = analytics.client.as_ref() else {
    return;
  };
  let distinct_id = analytics
    .distinct_id
    .lock()
    .map(|g| g.clone())
    .unwrap_or_else(|_| "unknown".into());

  let mut event = Event::new(event, &distinct_id);
  for (k, v) in common_props(&analytics.app_version) {
    if let Err(e) = event.insert_prop(k, v) {
      tracing::debug!("posthog prop: {e}");
    }
  }
  for (k, v) in props {
    if let Err(e) = event.insert_prop(k, v) {
      tracing::debug!("posthog prop: {e}");
    }
  }
  client.capture(event);
}

/// Convenience helpers for command handlers.
pub fn track_theme_applied(theme_id: &str, from_remote: bool, success: bool) {
  capture(
    "theme_applied",
    HashMap::from([
      ("theme_id".into(), Value::String(theme_id.to_string())),
      ("from_remote".into(), Value::Bool(from_remote)),
      ("success".into(), Value::Bool(success)),
    ]),
  );
}

pub fn track_theme_restored(success: bool) {
  capture(
    "theme_restored",
    HashMap::from([("success".into(), Value::Bool(success))]),
  );
}

pub fn track_theme_downloaded(theme_id: Option<&str>, success: bool) {
  let mut props = HashMap::from([("success".into(), Value::Bool(success))]);
  if let Some(id) = theme_id {
    props.insert("theme_id".into(), Value::String(id.to_string()));
  }
  capture("theme_downloaded", props);
}

pub fn track_theme_installed(theme_id: &str, success: bool) {
  capture(
    "theme_installed",
    HashMap::from([
      ("theme_id".into(), Value::String(theme_id.to_string())),
      ("success".into(), Value::Bool(success)),
    ]),
  );
}

pub fn track_theme_deleted(theme_id: &str, success: bool) {
  capture(
    "theme_deleted",
    HashMap::from([
      ("theme_id".into(), Value::String(theme_id.to_string())),
      ("success".into(), Value::Bool(success)),
    ]),
  );
}

pub fn track_cdp_port_changed(port: u16) {
  capture(
    "cdp_port_changed",
    HashMap::from([("port".into(), Value::Number(port.into()))]),
  );
}

pub fn track_page_viewed(page: &str) {
  capture(
    "page_viewed",
    HashMap::from([("page".into(), Value::String(page.to_string()))]),
  );
}

/// Shared handle for optional future use (tests / extensions).
#[allow(dead_code)]
pub type SharedAnalytics = Arc<Analytics>;
