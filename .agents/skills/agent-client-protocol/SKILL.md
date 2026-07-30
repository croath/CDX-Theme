---
name: agent-client-protocol
description: >
  Summary of the Agent Client Protocol (ACP) and how CDXTheme Theme Builder
  uses the Rust SDK with Codex — including sessions, streaming, permissions,
  and model selection via session config options. Use when working on Theme
  Builder chat, ACP sessions, codex-acp, model select menus, or
  agent-client-protocol integration.
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
3. On Unix, tearing down the connection typically kills the process group (including child runners like bundled Bun `x`).

### Remote (WIP)

Agents may also listen on HTTP / WebSocket. Full remote-agent support is still evolving; local stdio is the stable path.

### Codex-specific path (CDXTheme)

Codex CLI does not expose ACP natively as the primary UX. The usual stack is:

```text
CDXTheme (ACP Client)
    │  agent-client-protocol Rust SDK
    ▼
codex-acp  (PATH binary, or app-bundled Bun: `bun x @agentclientprotocol/codex-acp`)
    │
    ▼
Codex CLI  (ChatGPT-bundled `codex` and/or PATH)
```

CDXTheme prefers a local `codex-acp` on `PATH`, else the **app-bundled Bun** sidecar
(`bun x @agentclientprotocol/codex-acp@latest`). Put ChatGPT’s bundled `codex` on `PATH`
so the adapter can find it.

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
| `session/set_config_option` | Client → Agent | Set session config (model, reasoning, …) |
| `session/set_mode` | Client → Agent | Switch agent mode (ask / code / …) when advertised |
| `session/update` | Agent → Client | Stream progress (text chunks, tool calls, plan, …) |

User-facing text defaults to **Markdown**.

### Session config options (model, reasoning, …)

ACP does **not** have a dedicated `session/set_model` method in the modern
protocol. Model (and related knobs) use the generic **session config option**
mechanism:

1. On `session/new` / `session/load` / `session/resume`, the agent may return
   `configOptions: SessionConfigOption[]`.
2. The client renders selectors (model, thought level, …) from those options.
3. The client calls `session/set_config_option` with `configId` + value.
4. The agent may also push `config_option_update` notifications when options
   change mid-session.

#### `SessionConfigOption` shape (conceptual)

| Field | Role |
| --- | --- |
| `id` | Stable option id (e.g. `"model"`, `"reasoning_effort"`) |
| `name` / `description` | UI labels |
| `category` | UX hint only — not required for correctness |
| `type` | `"select"` (value id) or `"boolean"` |
| `currentValue` | Current selection |
| `options` | Select choices (`value` + `name` + optional description) |

#### Categories (UX only)

| `category` | Typical use |
| --- | --- |
| `model` | Primary model selector |
| `model_config` | Model-adjacent params (speed/quality, context, …) |
| `thought_level` | Reasoning / thought effort |
| `mode` | Session mode (overlaps with `session/set_mode` for some agents) |

Clients **must** handle missing or unknown categories gracefully. Do not hard-code
agent-specific option ids without a fallback; prefer matching
`category == "model"` when available, then fall back to known ids.

#### `session/set_config_option` value wire shape

```json
// Select / value-id (default when type omitted)
{ "sessionId": "…", "configId": "model", "value": "gpt-5.6-luna" }

// Explicit value_id form used by some SDKs
{ "sessionId": "…", "configId": "model", "type": "value_id", "value": "gpt-5.6-luna" }

// Boolean
{ "sessionId": "…", "configId": "fast_mode", "type": "boolean", "value": true }
```

Rust SDK types (schema v1): `SetSessionConfigOptionRequest`,
`SessionConfigOptionValue::value_id(...)` / `::boolean(...)`.

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
| `session/set_config_option` | Set model / reasoning / other session options |
| `session/set_mode` | Set agent operating mode |
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
      // new.config_options may list model / reasoning selectors

      // 4b) Optional: set model before the first prompt
      // connection.send_request(SetSessionConfigOptionRequest::new(
      //   new.session_id.clone(),
      //   "model",
      //   SessionConfigOptionValue::value_id("gpt-5.6-luna"),
      // )).block_task().await?;

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
UI: Theme Builder (`app-ui/src/pages/theme_builder/`)  
IPC: Tauri `codex_chat`, `list_codex_sessions`, `list_codex_models`, `get_codex_session`

### Turn model

| UI action | ACP / host |
| --- | --- |
| Start theme build → first message | `session/new` → optional `session/set_config_option(model)` → `session/prompt` |
| Follow-up message with `session_id` | `session/load` → optional set model → `session/prompt` |
| List sessions | Prefer ACP `session/list` when available; else `~/.codex/session_index.jsonl` |
| Open saved session transcript | Load rollout JSONL under `~/.codex/sessions` (disk fallback) |
| Populate model menu | Disk: `list_codex_models` (no ACP spawn); apply model on next turn via ACP |

**Important:** Theme Builder currently spawns a **fresh ACP connection per turn**
(connect → initialize → new/load → set model → prompt → drop). Model must be
re-applied after every `session/new` or `session/load`, not only on first open.

### Agent resolution (order)

1. `codex-acp` on `PATH`
2. App-bundled Bun sidecar: `bun x @agentclientprotocol/codex-acp@latest`
3. Prepend ChatGPT-bundled `codex` directory onto agent `PATH`

### Runtime requirements

- **App-bundled Bun** sidecar (staged by `prepare-bun-sidecar`), or `codex-acp` on PATH.
- **Codex CLI** (bundled with ChatGPT app and/or on PATH).
- **Auth**: `codex login` (or existing `~/.codex/auth.json`) when the adapter/CLI needs it.
- Theme Builder workspace cwd: app data `…/theme_builder/{id}` (absolute path for `session/new|load`).
- No host bunx/npx detect and no Install Bun UI — the app ships Bun.

### Security note

Theme Builder auto-approves permission requests for UX. Do not treat that as safe for untrusted agents or full-access sandboxes without an explicit product decision.

---

## Codex-acp model selection (CDXTheme)

### How codex-acp exposes models

`@agentclientprotocol/codex-acp` advertises session config options after
`session/new` / `session/load`, including:

| Option `id` | Category | Meaning |
| --- | --- | --- |
| `model` | `model` | Base model slug (e.g. `gpt-5.6-luna`) |
| `reasoning_effort` | `thought_level` | low / medium / high / … (when model supports it) |
| `fast_mode` | (boolean) | Fast mode when supported |
| (+ mode / collaboration options) | — | Agent-specific |

Internal constants in the adapter (for orientation):

- `MODEL_CONFIG_ID = "model"`
- `REASONING_EFFORT_CONFIG_ID = "reasoning_effort"`

Legacy `session/set_model` may still exist in older adapters; prefer
`session/set_config_option` with id `"model"`.

### Listing models without spawning ACP

Spawning the agent only to fetch the model list is slow and fragile. CDXTheme
reads local Codex files instead:

| File | Role |
| --- | --- |
| `~/.codex/models_cache.json` | Cached catalog (`models[]` with `slug`, `display_name`, `description`, `visibility`, …). Written by Codex CLI. |
| `~/.codex/config.toml` | Preferred default: top-level `model = "…"` |
| `CODEX_HOME` | Override for the Codex home directory (else `~/.codex`) |

Filter cache entries with `visibility == "list"` (or include when visibility is
absent). Ensure the configured `model` appears even if missing from the cache.
Keep a small hard-coded fallback list so the UI still renders offline.

Core API: `cdx_theme_core::list_codex_models()` → `CodexModelsList { models, current }`.  
IPC: `list_codex_models`. UI: `api::list_codex_models()`.

### Applying the selected model on a turn

In `send_and_wait_with` / `CodexChatOptions.model`:

```text
initialize
  → session/new | session/load
  → session/set_config_option { configId: "model", value: <id> }   // best-effort
  → session/prompt
```

Rules:

1. Treat set-model failure as **non-fatal** (log + continue with agent default).
2. Pass model from UI → `api::codex_chat(..., model)` → Tauri `codex_chat` →
   `CodexChatOptions.model`.
3. Show the model menu only on surfaces that **send prompts** (New Build + Chat),
   not on the session list Home page.
4. Share one `selected_model` signal across New Build and Chat so switching views
   keeps the same choice for the Theme Builder session.
5. Disable the menu while generate/send/apply is in flight.

### UI map

| Piece | Path |
| --- | --- |
| Shared state + `BuilderModelSelect` | `app-ui/src/pages/theme_builder/mod.rs` |
| New build (generate) | `builder_new_build.rs` — header model menu + `codex_chat` model arg |
| Resume chat | `builder_chat.rs` — header model menu + `codex_chat` model arg |
| i18n label | `builder.model.label` in `app-ui/src/i18n.rs` |
| Set model in ACP turn | `core/src/codex_chat.rs` (`SetSessionConfigOptionRequest`) |
| List models from disk | `core/src/codex_chat.rs` (`list_models`) |
| IPC | `list_codex_models`, `codex_chat` (`model` optional) in `app-tauri` + `api.rs` |

### codex-acp env notes (optional)

The adapter also accepts env such as:

- `CODEX_CONFIG` — JSON merged into Codex session config  
- `MODEL_PROVIDER` — provider for new sessions  

Theme Builder does **not** rely on these for the primary model picker; ACP
`set_config_option` is the session-scoped path.

### Common pitfalls

| Pitfall | Fix |
| --- | --- |
| Expect a dedicated `session/set_model` | Use `session/set_config_option` + id `model` |
| Only set model on first UI open | Re-set after every `session/new` / `session/load` (per-turn connect) |
| Spawn ACP just to fill the dropdown | Prefer `models_cache.json` + `config.toml` |
| Fail the whole turn if set-model errors | Best-effort; log and prompt anyway |
| Hard-code only agent option ids | Prefer `category == "model"` when parsing live `configOptions` |
| Show model menu on session list | Only on pages that call `codex_chat` |

---

## Practical tips

1. **Always use absolute `cwd`** for `session/new` and `session/load`.
2. **Own session ids** returned by `session/new` for multi-turn UI; pass them into `session/load`.
3. **Stream UI from `agent_message_chunk`**, not only the final prompt RPC—text often arrives only via notifications.
4. **Timeouts** around the whole `connect_with` future; long agent turns need generous budgets (Theme Builder defaults ~3 minutes).
5. **Disk fallback** for list/transcript remains useful when the adapter omits `session/list` or history APIs.
6. Prefer **structured ACP** over scraping interactive TUI or brittle `codex exec` JSONL when you need multi-turn chat from a host app.
7. **Model select:** list from disk, apply via `session/set_config_option` after session setup, non-fatal on failure, re-apply every turn when connections are short-lived.

---

## References

| Resource | URL / path |
| --- | --- |
| ACP intro | https://agentclientprotocol.com/get-started/introduction |
| ACP schema (incl. set_config_option) | https://agentclientprotocol.com/protocol/v1/schema |
| Session config / model category | https://agentclientprotocol.com/protocol/v1/session-config-options (and announcements on model_config category) |
| Agents list | https://agentclientprotocol.com/get-started/agents |
| Rust SDK | https://github.com/agentclientprotocol/rust-sdk |
| docs.rs | https://docs.rs/agent-client-protocol |
| Codex ACP adapter | https://github.com/agentclientprotocol/codex-acp / `@agentclientprotocol/codex-acp` |
| CDXTheme core | `core/src/codex_chat.rs` |
| Theme Builder UI | `app-ui/src/pages/theme_builder/` |
| Local model cache | `~/.codex/models_cache.json` |
| Local Codex config | `~/.codex/config.toml` |

---

## When to use this skill

- Extending Theme Builder chat, sessions, or Codex connectivity  
- Debugging ACP initialize / session / prompt failures  
- Adding or changing the **model select** menu / `list_codex_models` / set-model path  
- Choosing between `codex exec`, app-server, and ACP  
- Adding streaming, permissions, session config options, or MCP attachment on the ACP path  
