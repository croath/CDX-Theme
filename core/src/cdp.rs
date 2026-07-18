//! Chrome DevTools Protocol client (ported from `scripts/injector.mjs` CdpSession).

use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::{Mutex, oneshot};
use tokio::time::timeout;
use tokio_tungstenite::{
  connect_async_with_config,
  tungstenite::{Message, protocol::WebSocketConfig},
};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CdpTarget {
  pub id: String,
  #[serde(default)]
  pub title: String,
  #[serde(default)]
  pub url: String,
  #[serde(default)]
  pub r#type: String,
  pub web_socket_debugger_url: String,
}

type PendingMap = HashMap<u64, oneshot::Sender<Result<Value, String>>>;

pub struct CdpSession {
  write: Mutex<
    futures_util::stream::SplitSink<
      tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
      Message,
    >,
  >,
  pending: std::sync::Arc<Mutex<PendingMap>>,
  next_id: AtomicU64,
  timeout: Duration,
  closed: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl CdpSession {
  pub async fn open(target: &CdpTarget, timeout_ms: u64) -> Result<Self, String> {
    let timeout_dur = Duration::from_millis(timeout_ms);
    // Theme inject can embed multi-MB image data URLs (~5–30MB JSON frames).
    let mut ws_config = WebSocketConfig::default();
    ws_config.max_message_size = Some(64 * 1024 * 1024);
    ws_config.max_frame_size = Some(64 * 1024 * 1024);
    let (ws, _) = timeout(
      timeout_dur,
      connect_async_with_config(&target.web_socket_debugger_url, Some(ws_config), false),
    )
    .await
    .map_err(|_| "CDP socket open timed out".to_string())?
    .map_err(|e| format!("CDP connect failed: {e}"))?;

    let (write, mut read) = ws.split();
    let pending: std::sync::Arc<Mutex<PendingMap>> =
      std::sync::Arc::new(Mutex::new(HashMap::new()));
    let closed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

    // Reader task resolves pending request futures.
    {
      let pending = pending.clone();
      let closed = closed.clone();
      tokio::spawn(async move {
        while let Some(msg) = read.next().await {
          match msg {
            Ok(Message::Text(text)) => {
              if let Ok(value) = serde_json::from_str::<Value>(&text) {
                if let Some(id) = value.get("id").and_then(|v| v.as_u64()) {
                  let result = if let Some(err) = value.get("error") {
                    let message = err
                      .get("message")
                      .and_then(|m| m.as_str())
                      .unwrap_or("CDP error");
                    let code = err.get("code").and_then(|c| c.as_i64()).unwrap_or(0);
                    Err(format!("{message} ({code})"))
                  } else {
                    Ok(value.get("result").cloned().unwrap_or(Value::Null))
                  };
                  if let Some(tx) = pending.lock().await.remove(&id) {
                    let _ = tx.send(result);
                  }
                }
              }
            }
            Ok(Message::Close(_)) | Err(_) => break,
            _ => {}
          }
        }
        closed.store(true, Ordering::SeqCst);
        let mut pending = pending.lock().await;
        for (_, tx) in pending.drain() {
          let _ = tx.send(Err("CDP socket closed".into()));
        }
      });
    }

    let session = Self {
      write: Mutex::new(write),
      pending,
      next_id: AtomicU64::new(1),
      timeout: timeout_dur,
      closed,
    };

    session.send("Runtime.enable", json!({})).await?;
    session.send("Page.enable", json!({})).await?;
    Ok(session)
  }

  pub fn is_closed(&self) -> bool {
    self.closed.load(Ordering::SeqCst)
  }

  pub async fn send(&self, method: &str, params: Value) -> Result<Value, String> {
    self.send_with_timeout(method, params, self.timeout).await
  }

  pub async fn send_with_timeout(
    &self,
    method: &str,
    params: Value,
    request_timeout: Duration,
  ) -> Result<Value, String> {
    if self.is_closed() {
      return Err("CDP session is closed".into());
    }

    let id = self.next_id.fetch_add(1, Ordering::SeqCst);
    let (tx, rx) = oneshot::channel();
    self.pending.lock().await.insert(id, tx);

    let payload = json!({
      "id": id,
      "method": method,
      "params": params,
    });

    self
      .write
      .lock()
      .await
      .send(Message::Text(payload.to_string().into()))
      .await
      .map_err(|e| {
        // Drop pending entry on send failure
        let pending = self.pending.clone();
        let id = id;
        tokio::spawn(async move {
          pending.lock().await.remove(&id);
        });
        format!("CDP send failed: {e}")
      })?;

    match timeout(request_timeout, rx).await {
      Ok(Ok(result)) => result,
      Ok(Err(_)) => Err("CDP response channel closed".into()),
      Err(_) => {
        self.pending.lock().await.remove(&id);
        Err(format!("CDP request timed out: {method}"))
      }
    }
  }

  pub async fn evaluate(&self, expression: &str) -> Result<Value, String> {
    // Theme inject can embed multi-MB hero/texture data URLs; use a longer
    // timeout than generic CDP methods.
    let eval_timeout = if expression.len() > 500_000 {
      Duration::from_millis(self.timeout.as_millis().max(120_000) as u64)
    } else {
      self.timeout
    };
    let result = self
      .send_with_timeout(
        "Runtime.evaluate",
        json!({
          "expression": expression,
          "awaitPromise": true,
          "returnByValue": true,
          "userGesture": false,
        }),
        eval_timeout,
      )
      .await?;

    if let Some(details) = result.get("exceptionDetails") {
      let detail = details
        .pointer("/exception/description")
        .and_then(|v| v.as_str())
        .or_else(|| details.get("text").and_then(|v| v.as_str()))
        .unwrap_or("unknown evaluation error");
      return Err(format!("Renderer evaluation failed: {detail}"));
    }

    Ok(
      result
        .get("result")
        .and_then(|r| r.get("value"))
        .cloned()
        .unwrap_or(Value::Null),
    )
  }

  /// Register a script that runs on every new document (SPA reloads).
  /// Mirrors Codex-Dream-Skin `Page.addScriptToEvaluateOnNewDocument`.
  pub async fn add_script_on_new_document(&self, source: &str) -> Result<Option<String>, String> {
    let timeout = if source.len() > 500_000 {
      Duration::from_millis(self.timeout.as_millis().max(120_000) as u64)
    } else {
      self.timeout
    };
    let result = self
      .send_with_timeout(
        "Page.addScriptToEvaluateOnNewDocument",
        json!({ "source": source }),
        timeout,
      )
      .await?;
    Ok(
      result
        .get("identifier")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string()),
    )
  }

  /// Remove a previously registered early-document script.
  pub async fn remove_script_on_new_document(&self, identifier: &str) -> Result<(), String> {
    self
      .send(
        "Page.removeScriptToEvaluateOnNewDocument",
        json!({ "identifier": identifier }),
      )
      .await
      .map(|_| ())
  }

  pub async fn close(&self) {
    self.closed.store(true, Ordering::SeqCst);
    let _ = self.write.lock().await.close().await;
    let mut pending = self.pending.lock().await;
    for (_, tx) in pending.drain() {
      let _ = tx.send(Err("CDP session closed".into()));
    }
  }
}

/// Poll Codex CDP `/json/list` until page targets with `app://` URLs appear.
pub async fn wait_for_targets(port: u16, timeout_ms: u64) -> Result<Vec<CdpTarget>, String> {
  let deadline = tokio::time::Instant::now() + Duration::from_millis(timeout_ms);
  let client = reqwest::Client::builder()
    .timeout(Duration::from_millis(1500))
    .build()
    .map_err(|e| e.to_string())?;
  let url = format!("http://127.0.0.1:{port}/json/list");

  let mut last_error = String::from("timed out");
  while tokio::time::Instant::now() < deadline {
    match client.get(&url).send().await {
      Ok(response) if response.status().is_success() => {
        match response.json::<Vec<CdpTarget>>().await {
          Ok(targets) => {
            let pages: Vec<CdpTarget> = targets
              .into_iter()
              .filter(|t| t.r#type == "page" && t.url.starts_with("app://"))
              .collect();
            if !pages.is_empty() {
              return Ok(pages);
            }
            last_error = "no app:// page targets".into();
          }
          Err(e) => last_error = e.to_string(),
        }
      }
      Ok(response) => last_error = format!("HTTP {}", response.status()),
      Err(e) => last_error = e.to_string(),
    }
    tokio::time::sleep(Duration::from_millis(350)).await;
  }

  Err(format!(
    "No Codex renderer target on 127.0.0.1:{port}: {last_error}"
  ))
}
