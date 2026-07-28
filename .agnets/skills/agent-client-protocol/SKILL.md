---
name: agent-client-protocol
description: >
  Summary of the Agent Client Protocol (ACP) and how CDXTheme Theme Builder
  uses the Rust SDK with Codex. Use when working on Theme Builder chat, ACP
  sessions, codex-acp, or agent-client-protocol integration.
---

# Agent Client Protocol (ACP)

## What it is

**Agent Client Protocol (ACP)** is an open standard for communication between:

| Role | Examples |
| --- | --- |
| **Client** | IDEs, editors, desktop apps (Zed, CDXTheme Theme Builder, …) |
| **Agent** | Coding agents (Codex, Claude Code adapters, Gemini CLI, …) |

It is to agent/editor integration what **LSP** is to language servers: one protocol, many pairings, instead of N×M custom bridges.

- Spec / site: https://agentclientprotocol.com  
- Official Rust SDK: https://github.com/agentclientprotocol/rust-sdk  
- Crate: [`agent-client-protocol`](https://crates.io/crates/agent-client-protocol) (protocol types + client/agent runtime)

Related but different:

| Protocol | Purpose |
| --- | --- |
| **ACP** | Client ↔ coding agent (sessions, prompts, streaming, permissions) |
| **MCP** | Agent/tools ↔ external tool servers (filesystem, APIs, …) |

ACP reuses some MCP-style JSON content shapes where useful, and adds agent UX types (e.g. diffs, tool-call progress).

---

## Why ACP

Without ACP:

- Every editor must integrate every agent separately.
- Agents implement editor-specific APIs.
- Developers are locked into agent+editor pairs.

With ACP:

- Agents that speak ACP work in any ACP client.
- Clients gain the whole agent ecosystem with one integration surface.
- Local and (evolving) remote agents share the same session/prompt model.

---

## Architecture

### Local (most common)

```text
┌─────────────┐     JSON-RPC over stdio      ┌──────────────────┐
│   Client    │ ◄──────────────────────────► │  Agent process   │
│ (IDE / app) │   spawn + stdin/stdout       │ (or ACP adapter) │
└─────────────┘                              └──────────────────┘
```

1. Client **spawns** the agent (or adapter) as a subprocess.
2. Messages are **JSON-RPC** on **stdio**.
3. On Unix, tearing down the connection typically kills the process group (including wrappers like `npx`).

### Remote (WIP)

Agents may also listen on HTTP / WebSocket. Full remote-agent support is still evolving; local stdio is the stable path.

### Codex-specific path (CDXTheme)

Codex CLI does not expose ACP natively as the primary UX. The usual stack is:

```text
CDXTheme (ACP Client)
    │  agent-client-protocol Rust SDK
    ▼
codex-acp  (ACP adapter: local binary or npx @agentclientprotocol/codex-acp)
    │
    ▼
Codex CLI  (ChatGPT-bundled `codex` and/or PATH)
```

Official SDK helper: `AcpAgent::codex()` → `npx -y @agentclientprotocol/codex-acp@latest`.  
Prefer a local `codex-acp` on `PATH` when available. Put ChatGPT’s bundled `codex` on `PATH` so the adapter can find it.

---

## Core concepts

### Connection lifecycle

1. **Spawn / connect** transport (stdio process, or stream pair).
2. **`initialize`** — exchange protocol version and capabilities.
3. **Session** — create or load a conversation.
4. **Prompt turns** — send user content; receive streamed updates + completion.
5. **Permissions** — agent may ask the client to approve tools / modes.
6. **Close** — drop connection; agent process exits.

### Session

A **session** is one conversation context (history, cwd, modes, optional MCP servers).

| Operation | Direction | Purpose |
| --- | --- | --- |
| `session/new` | Client → Agent | Start a new session (`cwd` required, absolute) |
| `session/load` | Client → Agent | Resume by `sessionId` (if agent supports it) |
| `session/list` | Client → Agent | List known sessions (optional capability) |
| `session/prompt` | Client → Agent | Send a user turn |
| `session/update` | Agent → Client | Stream progress (text chunks, tool calls, plan, …) |

User-facing text defaults to **Markdown**.

### Content blocks

Prompts and streamed chunks use **content blocks** (MCP-compatible where possible):

- `text` — required baseline  
- `image` / `audio` — optional capabilities  
- `resource` / `resource_link` — context references  

### Permissions

Agents can send **request permission** RPCs so the client can show UI or auto-approve. Theme Builder currently auto-selects the first option (automation-friendly, not production-secure for untrusted agents).

---

## Message cheat sheet

### Client → Agent (requests)

| Method (concept) | Role |
| --- | --- |
| `initialize` | Negotiate version / capabilities |
| `session/new` | Create session |
| `session/load` | Load existing session by id |
| `session/list` | List sessions (optional) |
| `session/prompt` | User message (content blocks) |
| mode / config helpers | Optional session configuration |

### Agent → Client

| Kind | Role |
| --- | --- |
| `session/update` notifications | Stream agent message chunks, thoughts, tool calls, plan |
| Permission requests | Ask client to approve an action |
| Prompt response | End of turn (`stopReason`, …) |

### Common `session/update` variants

- `user_message_chunk` / `agent_message_chunk` / `agent_thought_chunk`
- `tool_call` / `tool_call_update`
- `plan`

For chat UI, **`agent_message_chunk`** (text) is usually what you concatenate into the assistant bubble.

---

## Rust SDK (`agent-client-protocol`)

### Crates

| Crate | Use |
| --- | --- |
| `agent-client-protocol` | Client / Agent / Proxy builders, transports, handlers |
| `agent-client-protocol-schema` | Wire types (also re-exported as `schema`) |
| `agent-client-protocol-http` | HTTP/SSE, WebSocket transports |
| `agent-client-protocol-rmcp` | MCP (`rmcp`) integration |

Pinned in this monorepo (core): **`agent-client-protocol = "2.0.0"`**.

### Client roles

- **`Client`** — you are the editor/app; you spawn or connect to an agent.
- **`Agent`** — you implement the agent side.
- **`AcpAgent`** — launch config + stdio connection to an external agent binary.
- **`ActiveSession` / session builder** — higher-level session API (`send_prompt`, `read_to_string`, …).

### Minimal client pattern

```rust
use agent_client_protocol::schema::{ProtocolVersion, v1::*};
use agent_client_protocol::{AcpAgent, Agent, Client, ConnectionTo};

// 1) Agent process
let agent = AcpAgent::codex(); // or AcpAgentConfig / from_str

// 2) Handlers: stream updates + answer permissions
Client.builder()
  .on_receive_notification(/* SessionNotification → append agent_message_chunk */, …)
  .on_receive_request(/* RequestPermissionRequest → respond Selected/Cancelled */, …)
  .connect_with(agent, |connection: ConnectionTo<Agent>| async move {
      // 3) Initialize
      connection
        .send_request(InitializeRequest::new(ProtocolVersion::V1))
        .block_task()
        .await?;

      // 4) Session
      let new = connection
        .send_request(NewSessionRequest::new(cwd))
        .block_task()
        .await?;
      // or: LoadSessionRequest::new(session_id, cwd)

      // 5) Prompt
      connection
        .send_request(PromptRequest::new(
          new.session_id,
          vec![ContentBlock::Text(TextContent::new(prompt))],
        ))
        .block_task()
        .await?;

      Ok(())
  })
  .await?;
```

Higher-level alternative after connect:

```rust
// Conceptual: SessionBuilder / ActiveSession
session.send_prompt("…")?;
let text = session.read_to_string().await?; // ignores non-text updates
```

### Launching agents

```rust
// String command
AcpAgent::from_str("python my_agent.py --verbose")?;

// Structured
AcpAgent::new(
  AcpAgentConfig::new("npx")
    .args(["-y", "@agentclientprotocol/codex-acp@latest"])
    .env("PATH", custom_path),
);

// Built-in helpers
AcpAgent::codex();   // npx @agentclientprotocol/codex-acp
AcpAgent::claude_agent();
```

---

## How CDXTheme uses ACP

Implementation: `core/src/codex_chat.rs`  
UI: Theme Builder page (`app-ui` home list + chat)  
IPC: Tauri `codex_chat`, `list_codex_sessions`, `get_codex_session`

### Turn model

| UI action | ACP |
| --- | --- |
| Start theme build → first message | `session/new` + `session/prompt` |
| Follow-up message with `session_id` | `session/load` + `session/prompt` |
| List sessions | Prefer ACP `session/list` when available; else `~/.codex/session_index.jsonl` |
| Open saved session transcript | Load rollout JSONL under `~/.codex/sessions` (disk fallback) |

### Agent resolution (order)

1. `codex-acp` on `PATH`
2. `npx -y @agentclientprotocol/codex-acp@latest`
3. Prepend ChatGPT-bundled `codex` directory onto agent `PATH`

### Runtime requirements

- **Node/npm** if using the default npx adapter (or install `codex-acp` yourself).
- **Codex CLI** (bundled with ChatGPT app and/or on PATH).
- **Auth**: `codex login` (or existing `~/.codex/auth.json`) when the adapter/CLI needs it.
- Theme Builder workspace cwd: temp dir `…/cdxtheme-theme-builder` (absolute path for `session/new|load`).

### Security note

Theme Builder auto-approves permission requests for UX. Do not treat that as safe for untrusted agents or full-access sandboxes without an explicit product decision.

---

## Practical tips

1. **Always use absolute `cwd`** for `session/new` and `session/load`.
2. **Own session ids** returned by `session/new` for multi-turn UI; pass them into `session/load`.
3. **Stream UI from `agent_message_chunk`**, not only the final prompt RPC—text often arrives only via notifications.
4. **Timeouts** around the whole `connect_with` future; long agent turns need generous budgets (Theme Builder defaults ~3 minutes).
5. **Disk fallback** for list/transcript remains useful when the adapter omits `session/list` or history APIs.
6. Prefer **structured ACP** over scraping interactive TUI or brittle `codex exec` JSONL when you need multi-turn chat from a host app.

---

## References

| Resource | URL |
| --- | --- |
| ACP intro | https://agentclientprotocol.com/get-started/introduction |
| Agents list | https://agentclientprotocol.com/get-started/agents |
| Rust SDK | https://github.com/agentclientprotocol/rust-sdk |
| docs.rs | https://docs.rs/agent-client-protocol |
| Codex ACP adapter | https://github.com/agentclientprotocol/codex-acp (and/or `@agentclientprotocol/codex-acp`) |
| CDXTheme core | `core/src/codex_chat.rs` |

---

## When to use this skill

- Extending Theme Builder chat, sessions, or Codex connectivity  
- Debugging ACP initialize / session / prompt failures  
- Choosing between `codex exec`, app-server, and ACP  
- Adding streaming, permissions, or MCP attachment on the ACP path  
