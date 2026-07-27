//! Live Codex layout probe / verify / screenshot over CDP.
//!
//! Used by the CLI (`cdxtheme verify layout`, `probe`, `screenshot`) for the
//! theme authoring loop after pack + apply.

use crate::cdp::{CdpSession, wait_for_targets};
use crate::inject::{DEFAULT_CDP_PORT, InjectOptions};
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::Path;
use std::time::Duration;

/// Full layout probe JS (Chat home / Work home). Returns a JSON object.
const LAYOUT_PROBE_JS: &str = include_str!("../../assets/layout-probe.js");

/// Quick DOM snapshot JS. Returns a JSON object.
const SNAPSHOT_JS: &str = include_str!("../../assets/layout-snapshot.js");

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutVerifyReport {
  pub port: u16,
  pub ok: bool,
  pub issue_count: usize,
  pub issues: Vec<String>,
  pub contexts: serde_json::Map<String, Value>,
}

/// Open first `app://` page target.
async fn open_primary_session(options: &InjectOptions) -> Result<(CdpSession, String), String> {
  let targets = wait_for_targets(options.port, options.timeout_ms).await?;
  let target = targets
    .into_iter()
    .next()
    .ok_or_else(|| "no app:// page targets".to_string())?;
  let id = target.id.clone();
  let session = CdpSession::open(&target, options.timeout_ms).await?;
  let _ = session
    .send("Runtime.enable", json!({}))
    .await
    .or_else(|_| Ok::<Value, String>(Value::Null));
  Ok((session, id))
}

/// Click Codex Chat / Work header tab by exact visible text.
pub async fn click_tab(session: &CdpSession, label: &str) -> Result<bool, String> {
  let expr = format!(
    r#"(() => {{
  const want = {};
  const els = [...document.querySelectorAll('button, [role="tab"], a, span, div')];
  const hit = els.find(e => (e.textContent || '').trim() === want);
  const t = hit ? (hit.closest('button, [role="tab"], a') || hit) : null;
  if (t) {{ t.click(); return true; }}
  return false;
}})()"#,
    serde_json::to_string(label).unwrap_or_else(|_| "\"\"".into())
  );
  let v = session.evaluate(&expr).await?;
  Ok(v.as_bool().unwrap_or(false))
}

/// Probe current page layout (no tab switch).
pub async fn probe_layout(session: &CdpSession) -> Result<Value, String> {
  session.evaluate(LAYOUT_PROBE_JS).await
}

/// Quick DOM snapshot (or custom expression).
pub async fn probe(
  options: InjectOptions,
  tab: Option<&str>,
  expression: Option<&str>,
  wait_ms: u64,
) -> Result<Value, String> {
  let (session, _) = open_primary_session(&options).await?;
  if let Some(label) = tab {
    let _ = click_tab(&session, label).await?;
    tokio::time::sleep(Duration::from_millis(wait_ms)).await;
  }
  let expr = expression.unwrap_or(SNAPSHOT_JS);
  let value = session.evaluate(expr).await?;
  session.close().await;
  Ok(value)
}

/// Capture a JPEG screenshot of the current Codex page.
pub async fn screenshot(
  options: InjectOptions,
  output: &Path,
  tab: Option<&str>,
  quality: u8,
  wait_ms: u64,
) -> Result<(), String> {
  let (session, _) = open_primary_session(&options).await?;
  if let Some(label) = tab {
    let _ = click_tab(&session, label).await?;
    tokio::time::sleep(Duration::from_millis(wait_ms)).await;
  }
  let _ = session.send("Page.enable", json!({})).await;
  let q = quality.clamp(1, 100);
  let result = session
    .send(
      "Page.captureScreenshot",
      json!({ "format": "jpeg", "quality": q }),
    )
    .await?;
  let b64 = result
    .get("data")
    .and_then(|v| v.as_str())
    .ok_or_else(|| "screenshot missing data".to_string())?;
  let bytes = base64::engine::general_purpose::STANDARD
    .decode(b64)
    .map_err(|e| format!("decode screenshot base64: {e}"))?;
  if let Some(parent) = output.parent() {
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
  }
  std::fs::write(output, bytes).map_err(|e| e.to_string())?;
  session.close().await;
  Ok(())
}

/// Switch Chat/Work and run layout checks. Returns a report; `ok` is false if any issue.
pub async fn verify_layout(
  options: InjectOptions,
  contexts: &[&str],
  wait_ms: u64,
) -> Result<LayoutVerifyReport, String> {
  let (session, _) = open_primary_session(&options).await?;
  let mut contexts_map = serde_json::Map::new();
  let mut all_issues: Vec<String> = Vec::new();

  for ctx in contexts {
    let label = match ctx.to_ascii_lowercase().as_str() {
      "chat" => "Chat",
      "work" => "Work",
      other => {
        return Err(format!("unknown context `{other}` (use chat or work)"));
      }
    };
    let clicked = click_tab(&session, label).await?;
    if !clicked {
      all_issues.push(format!("{ctx}: failed to click tab `{label}`"));
    }
    tokio::time::sleep(Duration::from_millis(wait_ms)).await;
    let data = probe_layout(&session).await?;
    if let Some(arr) = data.get("issues").and_then(|v| v.as_array()) {
      for issue in arr {
        if let Some(s) = issue.as_str() {
          all_issues.push(format!("{ctx}: {s}"));
        }
      }
    }
    contexts_map.insert(ctx.to_string(), data);
  }

  // Leave UI on Chat when we visited Work.
  if contexts.iter().any(|c| c.eq_ignore_ascii_case("work")) {
    let _ = click_tab(&session, "Chat").await;
    tokio::time::sleep(Duration::from_millis(300)).await;
  }

  session.close().await;

  let issue_count = all_issues.len();
  Ok(LayoutVerifyReport {
    port: options.port,
    ok: issue_count == 0,
    issue_count,
    issues: all_issues,
    contexts: contexts_map,
  })
}

/// Default inject options for layout tooling.
pub fn default_options(port: Option<u16>, timeout_ms: Option<u64>) -> InjectOptions {
  InjectOptions {
    port: port.unwrap_or(DEFAULT_CDP_PORT),
    timeout_ms: timeout_ms.unwrap_or(30_000),
  }
}
