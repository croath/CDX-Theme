//! Theme Builder → Codex via the **Agent Client Protocol** (ACP).
//!
//! Spawns the Codex ACP adapter (`codex-acp`, or `npx @agentclientprotocol/codex-acp`)
//! and talks to it with the official [`agent-client-protocol`] Rust SDK:
//! `initialize` → `session/new` | `session/load` → `session/prompt`.
//!
//! Session list / transcript still fall back to `~/.codex` on disk when the
//! adapter does not implement `session/list` or load history.

use crate::launch::find_chatgpt_app;
use agent_client_protocol::schema::ProtocolVersion;
use agent_client_protocol::schema::v1::{
  ContentBlock, InitializeRequest, ListSessionsRequest, LoadSessionRequest, NewSessionRequest,
  PromptRequest, RequestPermissionOutcome, RequestPermissionRequest, RequestPermissionResponse,
  SelectedPermissionOutcome, SessionNotification, SessionUpdate, TextContent,
};
use agent_client_protocol::{AcpAgent, AcpAgentConfig, Agent, Client, ConnectionTo};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::timeout;

/// Result of a single Theme Builder → Codex (ACP) round-trip.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexChatResult {
  /// Whether the ACP prompt was submitted successfully.
  pub submitted: bool,
  /// Final assistant message (streamed agent text + tool summary).
  pub reply: String,
  /// Always `1` when a non-empty reply was captured, else `0`.
  pub assistant_count: usize,
  /// True when the turn completed within the wait budget.
  pub stable: bool,
  /// Human-readable status (for UI / logs).
  pub message: String,
  /// ACP agent command that was used (when known).
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub binary: Option<String>,
  /// ACP / Codex session id (for `session/load` on the next turn).
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub session_id: Option<String>,
  /// Stop reason from the agent when available.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub stop_reason: Option<String>,
  /// Absolute path to a `.cdxtheme` package found in the workspace after this turn.
  /// Host fills this when the agent packed a theme; apply is a separate user action.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub package_path: Option<String>,
  /// Theme id installed from workspace package (host fills after manual apply).
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub installed_theme_id: Option<String>,
  /// Display name of the installed theme (host fills after manual apply).
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub installed_theme_name: Option<String>,
  /// Whether the installed theme was also applied (host fills after manual apply).
  #[serde(default)]
  pub applied: bool,
}

/// One row in the Theme Builder session list.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexSessionSummary {
  pub id: String,
  pub title: String,
  pub updated_at: String,
  /// Absolute path to the rollout JSONL when known.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub path: Option<String>,
  /// Theme Builder workspace root (`app_data_dir/theme_builder/{id}`) when known.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub workspace_path: Option<String>,
}

/// A single message loaded from a Codex session transcript.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexSessionMessage {
  /// `user` | `assistant` | `system`
  pub role: String,
  pub content: String,
}

/// Full session payload for the chat view.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexSessionDetail {
  pub id: String,
  pub title: String,
  pub updated_at: String,
  pub messages: Vec<CodexSessionMessage>,
  /// Theme Builder workspace root when known.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub workspace_path: Option<String>,
}

/// Resolve the Codex CLI binary: ChatGPT-bundled first, then `PATH`.
/// Used to put Codex on PATH for the ACP adapter process.
pub fn find_codex_cli() -> Result<PathBuf, String> {
  if let Some(bundled) = find_bundled_codex_cli() {
    if bundled.is_file() {
      return Ok(bundled);
    }
  }
  if let Some(path) = which("codex") {
    return Ok(path);
  }
  Err(
    "Codex CLI not found. Install the ChatGPT desktop app (bundles `codex`) \
     or install the Codex CLI and ensure `codex` is on PATH."
      .into(),
  )
}

fn find_bundled_codex_cli() -> Option<PathBuf> {
  #[cfg(target_os = "macos")]
  {
    if let Some(exe) = find_chatgpt_app() {
      let resources = exe
        .parent()
        .and_then(|p| p.parent())
        .map(|contents| contents.join("Resources").join("codex"));
      if let Some(ref p) = resources {
        if p.is_file() {
          return Some(p.clone());
        }
      }
    }
    for app in [
      PathBuf::from("/Applications/ChatGPT.app"),
      PathBuf::from("/Applications/Codex.app"),
    ] {
      let p = app.join("Contents/Resources/codex");
      if p.is_file() {
        return Some(p);
      }
    }
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
      let p = home.join("Applications/ChatGPT.app/Contents/Resources/codex");
      if p.is_file() {
        return Some(p);
      }
    }
    return None;
  }

  #[cfg(target_os = "windows")]
  {
    if let Some(exe) = find_chatgpt_app() {
      let dir = exe.parent()?;
      for candidate in [
        dir.join("codex.exe"),
        dir.join("codex"),
        dir.join("resources").join("codex.exe"),
        dir.join("Resources").join("codex.exe"),
        dir.join("resources").join("codex"),
      ] {
        if candidate.is_file() {
          return Some(candidate);
        }
      }
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA").map(PathBuf::from) {
      for rel in [
        "Programs/ChatGPT/codex.exe",
        "Programs/ChatGPT/resources/codex.exe",
        "Programs/Codex/codex.exe",
        "Programs/Codex/resources/codex.exe",
      ] {
        let p = local.join(rel);
        if p.is_file() {
          return Some(p);
        }
      }
    }
    return None;
  }

  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  {
    let _ = find_chatgpt_app;
    None
  }
}

fn which(bin: &str) -> Option<PathBuf> {
  let path = std::env::var_os("PATH")?;
  for dir in std::env::split_paths(&path) {
    #[cfg(windows)]
    {
      let pathext = std::env::var("PATHEXT").unwrap_or_else(|_| ".EXE;.CMD;.BAT".into());
      for ext in pathext.split(';').filter(|e| !e.is_empty()) {
        let mut name = std::ffi::OsString::from(bin);
        name.push(ext);
        let candidate = dir.join(&name);
        if candidate.is_file() {
          return Some(candidate);
        }
      }
      let candidate = dir.join(bin);
      if candidate.is_file() {
        return Some(candidate);
      }
    }
    #[cfg(not(windows))]
    {
      let candidate = dir.join(bin);
      if candidate.is_file() {
        return Some(candidate);
      }
    }
  }
  None
}

fn codex_home() -> PathBuf {
  if let Ok(home) = std::env::var("CODEX_HOME") {
    let p = PathBuf::from(home);
    if !p.as_os_str().is_empty() {
      return p;
    }
  }
  dirs_home()
    .map(|h| h.join(".codex"))
    .unwrap_or_else(|| PathBuf::from(".codex"))
}

fn dirs_home() -> Option<PathBuf> {
  std::env::var_os("HOME")
    .or_else(|| std::env::var_os("USERPROFILE"))
    .map(PathBuf::from)
}

/// Fallback cwd only when the host does not pass a workspace path.
fn default_theme_builder_cwd() -> PathBuf {
  let dir = std::env::temp_dir().join("cdxtheme-theme-builder");
  let _ = std::fs::create_dir_all(&dir);
  dir
}

#[cfg(windows)]
const PATH_SEP: char = ';';
#[cfg(not(windows))]
const PATH_SEP: char = ':';

fn prepend_path_env(path_env: &mut String, dir: &Path) {
  let prepend = dir.display().to_string();
  if path_env
    .split(PATH_SEP)
    .any(|p| p == prepend || Path::new(p) == dir)
  {
    return;
  }
  if path_env.is_empty() {
    *path_env = prepend;
  } else {
    *path_env = format!("{prepend}{PATH_SEP}{path_env}");
  }
}

/// Build the ACP agent process config (Codex adapter).
///
/// Preference order:
/// 1. `codex-acp` on PATH
/// 2. `npx -y @agentclientprotocol/codex-acp@latest` (official SDK helper)
///
/// `path_prepend` directories (e.g. folder of bundled `cdxthemex`) are put first on PATH.
/// `extra_env` is merged into the agent process environment.
fn build_acp_agent(
  path_prepend: &[PathBuf],
  extra_env: &[(String, String)],
) -> Result<(AcpAgent, String), String> {
  let mut path_env = std::env::var("PATH").unwrap_or_default();
  // Host-provided dirs first (bundled cdxthemex).
  for dir in path_prepend.iter().rev() {
    prepend_path_env(&mut path_env, dir);
  }
  // Ensure ChatGPT-bundled `codex` is visible to the adapter.
  if let Ok(codex) = find_codex_cli() {
    if let Some(dir) = codex.parent() {
      prepend_path_env(&mut path_env, dir);
    }
  }

  let apply_env = |cfg: AcpAgentConfig| -> AcpAgentConfig {
    let mut cfg = cfg.env("PATH", &path_env);
    for (k, v) in extra_env {
      cfg = cfg.env(k, v);
    }
    cfg
  };

  if let Some(local) = which("codex-acp") {
    let label = local.display().to_string();
    let agent = AcpAgent::new(apply_env(AcpAgentConfig::new(&local)));
    return Ok((agent, label));
  }

  let label = "npx -y @agentclientprotocol/codex-acp@latest".to_string();
  let agent = AcpAgent::new(apply_env(
    AcpAgentConfig::new("npx").args(["-y", "@agentclientprotocol/codex-acp@latest"]),
  ));
  Ok((agent, label))
}

/// List saved sessions (ACP `session/list` when available, else `~/.codex`).
pub fn list_sessions(limit: Option<usize>) -> Result<Vec<CodexSessionSummary>, String> {
  let limit = limit.unwrap_or(50).clamp(1, 200);
  // Filesystem is reliable offline and does not require spawning npx.
  list_sessions_from_disk(limit)
}

/// Async list that prefers ACP `session/list`, falls back to disk.
pub async fn list_sessions_async(limit: Option<usize>) -> Result<Vec<CodexSessionSummary>, String> {
  let limit = limit.unwrap_or(50).clamp(1, 200);
  match list_sessions_via_acp(limit).await {
    Ok(list) if !list.is_empty() => Ok(list),
    Ok(_) => list_sessions_from_disk(limit),
    Err(e) => {
      tracing::debug!("ACP session/list unavailable ({e}); using ~/.codex index");
      list_sessions_from_disk(limit)
    }
  }
}

async fn list_sessions_via_acp(limit: usize) -> Result<Vec<CodexSessionSummary>, String> {
  let (agent, _) = build_acp_agent(&[], &[])?;
  // List without cwd filter so all Codex history is available for intersection.
  let result = Client
    .builder()
    .on_receive_request(
      async move |request: RequestPermissionRequest, responder, _| {
        auto_approve_permission(request, responder);
        Ok(())
      },
      agent_client_protocol::on_receive_request!(),
    )
    .connect_with(agent, |connection: ConnectionTo<Agent>| async move {
      connection
        .send_request(InitializeRequest::new(ProtocolVersion::V1))
        .block_task()
        .await?;

      let resp = connection
        .send_request(ListSessionsRequest::new())
        .block_task()
        .await?;

      let mut out = Vec::new();
      for s in resp.sessions.into_iter().take(limit) {
        out.push(CodexSessionSummary {
          id: s.session_id.to_string(),
          title: s
            .title
            .filter(|t| !t.trim().is_empty())
            .unwrap_or_else(|| "Untitled session".into()),
          updated_at: s.updated_at.unwrap_or_default(),
          path: None,
          workspace_path: None,
        });
      }
      Ok(out)
    })
    .await
    .map_err(|e| format!("ACP session/list: {e}"))?;

  Ok(result)
}

fn list_sessions_from_disk(limit: usize) -> Result<Vec<CodexSessionSummary>, String> {
  let home = codex_home();
  let index_path = home.join("session_index.jsonl");
  let mut sessions: Vec<CodexSessionSummary> = Vec::new();

  if index_path.is_file() {
    let text = std::fs::read_to_string(&index_path).map_err(|e| e.to_string())?;
    for line in text.lines() {
      let line = line.trim();
      if line.is_empty() {
        continue;
      }
      let Ok(v) = serde_json::from_str::<Value>(line) else {
        continue;
      };
      let id = v
        .get("id")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
      if id.is_empty() {
        continue;
      }
      let title = v
        .get("thread_name")
        .and_then(|x| x.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("Untitled session")
        .to_string();
      let updated_at = v
        .get("updated_at")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
      let path = find_rollout_path(&home, &id).map(|p| p.display().to_string());
      sessions.push(CodexSessionSummary {
        id,
        title,
        updated_at,
        path,
        workspace_path: None,
      });
    }
  }

  if sessions.is_empty() {
    for path in scan_rollout_files(&home) {
      if let Some(meta) = read_session_meta_from_rollout(&path) {
        sessions.push(meta);
      }
    }
  }

  sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
  sessions.truncate(limit);
  Ok(sessions)
}

/// Load transcript + meta for a session id (from `~/.codex` rollouts).
pub fn load_session(session_id: &str) -> Result<CodexSessionDetail, String> {
  let session_id = session_id.trim();
  if session_id.is_empty() {
    return Err("session id is empty".into());
  }
  let home = codex_home();

  let mut title = "Untitled session".to_string();
  let mut updated_at = String::new();
  if let Ok(list) = list_sessions_from_disk(200) {
    if let Some(s) = list.into_iter().find(|s| s.id == session_id) {
      title = s.title;
      updated_at = s.updated_at;
    }
  }

  let path = find_rollout_path(&home, session_id)
    .ok_or_else(|| format!("session rollout not found for id `{session_id}`"))?;

  let messages = parse_rollout_messages(&path)?;
  if updated_at.is_empty() {
    if let Ok(meta) = std::fs::metadata(&path) {
      if let Ok(mtime) = meta.modified() {
        updated_at = humantime_iso(mtime);
      }
    }
  }

  Ok(CodexSessionDetail {
    id: session_id.to_string(),
    title,
    updated_at,
    messages,
    workspace_path: None,
  })
}

fn scan_rollout_files(home: &Path) -> Vec<PathBuf> {
  let mut out = Vec::new();
  let roots = [home.join("sessions"), home.join("archived_sessions")];
  for root in roots {
    if !root.is_dir() {
      continue;
    }
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
      let Ok(rd) = std::fs::read_dir(&dir) else {
        continue;
      };
      for entry in rd.flatten() {
        let p = entry.path();
        if p.is_dir() {
          stack.push(p);
        } else if p
          .file_name()
          .and_then(|n| n.to_str())
          .is_some_and(|n| n.starts_with("rollout-") && n.ends_with(".jsonl"))
        {
          out.push(p);
        }
      }
    }
  }
  out.sort_by(|a, b| {
    let ma = a
      .metadata()
      .and_then(|m| m.modified())
      .ok()
      .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
    let mb = b
      .metadata()
      .and_then(|m| m.modified())
      .ok()
      .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
    mb.cmp(&ma)
  });
  out
}

fn find_rollout_path(home: &Path, session_id: &str) -> Option<PathBuf> {
  let needle = session_id.to_string();
  scan_rollout_files(home).into_iter().find(|p| {
    p.file_name()
      .and_then(|n| n.to_str())
      .is_some_and(|n| n.contains(&needle))
  })
}

fn read_session_meta_from_rollout(path: &Path) -> Option<CodexSessionSummary> {
  let text = std::fs::read_to_string(path).ok()?;
  let mut id = String::new();
  let mut title = String::new();
  for line in text.lines().take(40) {
    let Ok(v) = serde_json::from_str::<Value>(line) else {
      continue;
    };
    if v.get("type").and_then(|t| t.as_str()) == Some("session_meta") {
      if let Some(payload) = v.get("payload") {
        if let Some(s) = payload.get("id").and_then(|x| x.as_str()) {
          id = s.to_string();
        }
        if let Some(s) = payload
          .get("thread_name")
          .or_else(|| payload.get("title"))
          .and_then(|x| x.as_str())
        {
          title = s.to_string();
        }
      }
    }
  }
  if id.is_empty() {
    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
      let parts: Vec<&str> = name.split('-').collect();
      if parts.len() >= 5 {
        let candidate = parts[parts.len() - 5..].join("-");
        if candidate.len() >= 32 {
          id = candidate;
        }
      }
    }
  }
  if id.is_empty() {
    return None;
  }
  if title.is_empty() {
    title = "Untitled session".into();
  }
  let updated_at = path
    .metadata()
    .ok()
    .and_then(|m| m.modified().ok())
    .map(humantime_iso)
    .unwrap_or_default();
  Some(CodexSessionSummary {
    id,
    title,
    updated_at,
    path: Some(path.display().to_string()),
    workspace_path: None,
  })
}

fn parse_rollout_messages(path: &Path) -> Result<Vec<CodexSessionMessage>, String> {
  let text = std::fs::read_to_string(path).map_err(|e| format!("read rollout: {e}"))?;
  let mut messages: Vec<CodexSessionMessage> = Vec::new();

  for line in text.lines() {
    let line = line.trim();
    if line.is_empty() {
      continue;
    }
    let Ok(v) = serde_json::from_str::<Value>(line) else {
      continue;
    };
    let ty = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
    let payload = v.get("payload");

    match ty {
      "response_item" => {
        let Some(p) = payload else { continue };
        if p.get("type").and_then(|t| t.as_str()) != Some("message") {
          continue;
        }
        let role_raw = p.get("role").and_then(|r| r.as_str()).unwrap_or("");
        let role = match role_raw {
          "user" | "human" => "user",
          "assistant" | "agent" => "assistant",
          "system" | "developer" => continue,
          _ => continue,
        };
        let content = extract_message_text(p);
        if content.trim().is_empty() {
          continue;
        }
        if content.contains("<environment_context>")
          || content.contains("<permissions instructions>")
        {
          continue;
        }
        push_dedup(&mut messages, role, content);
      }
      "event_msg" => {
        let Some(p) = payload else { continue };
        let et = p.get("type").and_then(|t| t.as_str()).unwrap_or("");
        let (role, text) = match et {
          "user_message" => (
            "user",
            p.get("message")
              .or_else(|| p.get("text"))
              .and_then(|x| x.as_str())
              .unwrap_or("")
              .to_string(),
          ),
          "agent_message" | "assistant_message" => (
            "assistant",
            p.get("message")
              .or_else(|| p.get("text"))
              .and_then(|x| x.as_str())
              .unwrap_or("")
              .to_string(),
          ),
          _ => continue,
        };
        if text.trim().is_empty() {
          continue;
        }
        push_dedup(&mut messages, role, text);
      }
      _ => {}
    }
  }

  Ok(messages)
}

fn extract_message_text(payload: &Value) -> String {
  let content = match payload.get("content") {
    Some(c) => c,
    None => {
      return payload
        .get("text")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();
    }
  };
  if let Some(s) = content.as_str() {
    return s.to_string();
  }
  let mut out = String::new();
  if let Some(arr) = content.as_array() {
    for item in arr {
      if let Some(s) = item.as_str() {
        out.push_str(s);
        continue;
      }
      let ty = item.get("type").and_then(|t| t.as_str()).unwrap_or("");
      if matches!(ty, "input_text" | "output_text" | "text") {
        if let Some(t) = item.get("text").and_then(|t| t.as_str()) {
          if !out.is_empty() {
            out.push('\n');
          }
          out.push_str(t);
        }
      }
    }
  }
  out
}

fn push_dedup(messages: &mut Vec<CodexSessionMessage>, role: &str, content: String) {
  let content = content.trim().to_string();
  if content.is_empty() {
    return;
  }
  if let Some(last) = messages.last() {
    if last.role == role && last.content == content {
      return;
    }
  }
  messages.push(CodexSessionMessage {
    role: role.to_string(),
    content,
  });
}

fn humantime_iso(t: std::time::SystemTime) -> String {
  match t.duration_since(std::time::UNIX_EPOCH) {
    Ok(d) => format_unix_secs(d.as_secs() as i64),
    Err(_) => String::new(),
  }
}

fn format_unix_secs(secs: i64) -> String {
  let days = secs.div_euclid(86_400);
  let tod = secs.rem_euclid(86_400) as u32;
  let hour = tod / 3600;
  let min = (tod % 3600) / 60;
  let sec = tod % 60;
  let z = days + 719_468;
  let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
  let doe = (z - era * 146_097) as u64;
  let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
  let y = yoe as i64 + era * 400;
  let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
  let mp = (5 * doy + 2) / 153;
  let d = doy - (153 * mp + 2) / 5 + 1;
  let m = if mp < 10 { mp + 3 } else { mp - 9 };
  let y = if m <= 2 { y + 1 } else { y };
  format!("{y:04}-{m:02}-{d:02}T{hour:02}:{min:02}:{sec:02}Z")
}

fn auto_approve_permission(
  request: RequestPermissionRequest,
  responder: agent_client_protocol::Responder<RequestPermissionResponse>,
) {
  if let Some(opt) = request.options.first() {
    let _ = responder.respond(RequestPermissionResponse::new(
      RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(opt.option_id.clone())),
    ));
  } else {
    let _ = responder.respond(RequestPermissionResponse::new(
      RequestPermissionOutcome::Cancelled,
    ));
  }
}

/// Callback for live ACP transcript text (agent message + tool summary).
pub type CodexStreamCallback = Arc<dyn Fn(String) + Send + Sync + 'static>;

/// Options for a Theme Builder ACP turn.
#[derive(Clone, Default)]
pub struct CodexChatOptions {
  pub session_id: Option<String>,
  pub cwd: Option<PathBuf>,
  pub wait_ms: Option<u64>,
  /// Directories prepended to the agent process PATH (e.g. folder of bundled `cdxthemex`).
  pub path_prepend: Vec<PathBuf>,
  /// Extra env vars for the agent process (e.g. `CDXTHEME=/abs/path/cdxthemex`).
  pub extra_env: Vec<(String, String)>,
  /// Invoked whenever the turn transcript changes (for live UI streaming).
  pub on_stream: Option<CodexStreamCallback>,
}

impl std::fmt::Debug for CodexChatOptions {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CodexChatOptions")
      .field("session_id", &self.session_id)
      .field("cwd", &self.cwd)
      .field("wait_ms", &self.wait_ms)
      .field("path_prepend", &self.path_prepend)
      .field("extra_env", &self.extra_env)
      .field("on_stream", &self.on_stream.as_ref().map(|_| "<callback>"))
      .finish()
  }
}

/// Run one Theme Builder turn over ACP against Codex.
///
/// - No `session_id`: `session/new` then `session/prompt`
/// - With `session_id`: `session/load` then `session/prompt`
/// - `cwd` is the Theme Builder workspace root (skill + theme-dir); falls back to a temp dir
/// - Streams `agent_message_chunk` into the reply string
/// - Auto-approves permission requests
pub async fn send_and_wait(
  prompt: &str,
  session_id: Option<&str>,
  cwd: Option<&Path>,
  wait_ms: Option<u64>,
) -> Result<CodexChatResult, String> {
  send_and_wait_with(
    prompt,
    CodexChatOptions {
      session_id: session_id.map(|s| s.to_string()),
      cwd: cwd.map(Path::to_path_buf),
      wait_ms,
      ..Default::default()
    },
  )
  .await
}

/// Like [`send_and_wait`], with PATH/env for the bundled CDXTheme CLI.
pub async fn send_and_wait_with(
  prompt: &str,
  options: CodexChatOptions,
) -> Result<CodexChatResult, String> {
  let prompt = prompt.trim();
  if prompt.is_empty() {
    return Err("prompt is empty".into());
  }
  if prompt.len() > 32_000 {
    return Err("prompt is too long (max 32k characters)".into());
  }

  let wait_ms = options.wait_ms.unwrap_or(180_000).clamp(10_000, 600_000);
  let (agent, agent_label) = build_acp_agent(&options.path_prepend, &options.extra_env)?;
  let cwd = options
    .cwd
    .filter(|p| p.is_absolute())
    .unwrap_or_else(default_theme_builder_cwd);
  if !cwd.is_dir() {
    std::fs::create_dir_all(&cwd).map_err(|e| format!("create workspace cwd: {e}"))?;
  }
  let resume = options
    .session_id
    .as_deref()
    .map(str::trim)
    .filter(|s| !s.is_empty());

  tracing::info!(
    agent = %agent_label,
    chars = prompt.len(),
    wait_ms,
    resume = resume.unwrap_or(""),
    cwd = %cwd.display(),
    "theme builder → ACP session/prompt"
  );

  // Collect streamed agent text + tool activity for the chat UI.
  let transcript = Arc::new(Mutex::new(TurnTranscript::default()));
  let transcript_for_handler = transcript.clone();
  let on_stream = options.on_stream.clone();
  let last_streamed = Arc::new(Mutex::new(String::new()));
  let last_streamed_handler = last_streamed.clone();
  let prompt_owned = prompt.to_string();
  let resume_owned = resume.map(|s| s.to_string());

  let work = Client
    .builder()
    .on_receive_notification(
      async move |n: SessionNotification, _cx| {
        let rendered = {
          if let Ok(mut t) = transcript_for_handler.lock() {
            t.ingest_update(n.update);
            t.render()
          } else {
            String::new()
          }
        };
        if !rendered.is_empty() {
          if let Some(cb) = on_stream.as_ref() {
            let mut last = last_streamed_handler
              .lock()
              .unwrap_or_else(|e| e.into_inner());
            if *last != rendered {
              *last = rendered.clone();
              cb(rendered);
            }
          }
        }
        Ok(())
      },
      agent_client_protocol::on_receive_notification!(),
    )
    .on_receive_request(
      async move |request: RequestPermissionRequest, responder, _| {
        auto_approve_permission(request, responder);
        Ok(())
      },
      agent_client_protocol::on_receive_request!(),
    )
    .connect_with(agent, move |connection: ConnectionTo<Agent>| {
      let prompt_owned = prompt_owned.clone();
      let resume_owned = resume_owned.clone();
      let cwd = cwd.clone();
      async move {
        connection
          .send_request(InitializeRequest::new(ProtocolVersion::V1))
          .block_task()
          .await?;

        let session_id = if let Some(id) = resume_owned {
          connection
            .send_request(LoadSessionRequest::new(id.clone(), cwd.clone()))
            .block_task()
            .await?;
          id
        } else {
          let resp = connection
            .send_request(NewSessionRequest::new(cwd.clone()))
            .block_task()
            .await?;
          resp.session_id.to_string()
        };

        let prompt_resp = connection
          .send_request(PromptRequest::new(
            session_id.clone(),
            vec![ContentBlock::Text(TextContent::new(prompt_owned))],
          ))
          .block_task()
          .await?;

        let stop = format!("{:?}", prompt_resp.stop_reason);
        Ok((session_id, stop))
      }
    });

  let (session_id, stop_reason) = match timeout(Duration::from_millis(wait_ms), work).await {
    Ok(Ok(pair)) => pair,
    Ok(Err(e)) => {
      // Still surface any partial stream if we got one.
      let partial = transcript.lock().map(|t| t.render()).unwrap_or_default();
      if !partial.trim().is_empty() {
        return Ok(CodexChatResult {
          submitted: true,
          assistant_count: 1,
          stable: false,
          message: format!("ACP error (partial reply): {e}"),
          reply: partial,
          binary: Some(agent_label),
          session_id: None,
          stop_reason: None,
          package_path: None,
          installed_theme_id: None,
          installed_theme_name: None,
          applied: false,
        });
      }
      return Err(format!(
        "ACP error: {e}. Ensure Node/npm is available for codex-acp, or install `codex-acp` on PATH. \
         Sign in with `codex login` if needed."
      ));
    }
    Err(_) => {
      let partial = transcript.lock().map(|t| t.render()).unwrap_or_default();
      if !partial.trim().is_empty() {
        return Ok(CodexChatResult {
          submitted: true,
          assistant_count: 1,
          stable: false,
          message: format!(
            "Codex ACP turn timed out after {}s (partial reply)",
            wait_ms / 1000
          ),
          reply: partial,
          binary: Some(agent_label),
          session_id: None,
          stop_reason: Some("timeout".into()),
          package_path: None,
          installed_theme_id: None,
          installed_theme_name: None,
          applied: false,
        });
      }
      return Err(format!(
        "Codex ACP turn timed out after {}s.",
        wait_ms / 1000
      ));
    }
  };

  let mut reply = transcript.lock().map(|t| t.render()).unwrap_or_default();

  // Fallback: load latest assistant text from Codex rollout if stream was empty.
  if reply.trim().is_empty() {
    if let Ok(detail) = load_session(&session_id) {
      if let Some(last) = detail
        .messages
        .iter()
        .rev()
        .find(|m| m.role == "assistant" && !m.content.trim().is_empty())
      {
        reply = last.content.clone();
      }
    }
  }

  if reply.trim().is_empty() {
    reply = format!(
      "(No assistant text was streamed. Stop reason: {stop_reason}. \
       The agent may have only run tools — check the workspace files.)"
    );
  }

  Ok(CodexChatResult {
    submitted: true,
    assistant_count: 1,
    stable: true,
    message: "Codex reply ready".into(),
    reply,
    binary: Some(agent_label),
    session_id: Some(session_id),
    stop_reason: Some(stop_reason),
    package_path: None,
    installed_theme_id: None,
    installed_theme_name: None,
    applied: false,
  })
}

/// Accumulates ACP `session/update` traffic into a user-visible transcript.
#[derive(Default)]
struct TurnTranscript {
  agent_text: String,
  tools: Vec<String>,
}

impl TurnTranscript {
  fn ingest_update(&mut self, update: SessionUpdate) {
    match update {
      SessionUpdate::AgentMessageChunk(chunk) => {
        if let Some(text) = content_block_text(&chunk.content) {
          self.agent_text.push_str(&text);
        }
      }
      SessionUpdate::AgentThoughtChunk(chunk) => {
        // Keep thoughts out of the main reply body (often noisy).
        let _ = chunk;
      }
      SessionUpdate::ToolCall(call) => {
        let title = call.title.trim();
        if !title.is_empty() {
          self.tools.push(format!("• {title}"));
        }
      }
      SessionUpdate::ToolCallUpdate(update) => {
        // Prefer title from fields when present.
        let title = update
          .fields
          .title
          .as_ref()
          .map(|s| s.trim().to_string())
          .filter(|s| !s.is_empty());
        if let Some(t) = title {
          let line = format!("• {t}");
          if !self.tools.iter().any(|x| x == &line) {
            self.tools.push(line);
          }
        }
      }
      SessionUpdate::Plan(plan) => {
        // Summarize plan entries if present.
        for entry in plan.entries.iter().take(8) {
          let content = entry.content.trim();
          if !content.is_empty() {
            let line = format!("◇ {content}");
            if !self.tools.iter().any(|x| x == &line) {
              self.tools.push(line);
            }
          }
        }
      }
      _ => {}
    }
  }

  fn render(&self) -> String {
    let mut out = self.agent_text.trim().to_string();
    if !self.tools.is_empty() {
      if !out.is_empty() {
        out.push_str("\n\n");
      }
      out.push_str("**Actions**\n");
      out.push_str(&self.tools.join("\n"));
    }
    out
  }
}

fn content_block_text(block: &ContentBlock) -> Option<String> {
  match block {
    ContentBlock::Text(t) => {
      if t.text.is_empty() {
        None
      } else {
        Some(t.text.clone())
      }
    }
    ContentBlock::ResourceLink(link) => {
      let name = if link.name.trim().is_empty() {
        link.uri.clone()
      } else {
        link.name.clone()
      };
      Some(format!("[{name}]({})", link.uri))
    }
    ContentBlock::Resource(res) => {
      // Prefer embedded text resources when present.
      // Structure varies; best-effort via debug/json is too heavy — skip if no simple text.
      let _ = res;
      None
    }
    ContentBlock::Image(_) | ContentBlock::Audio(_) => None,
    _ => None,
  }
}
