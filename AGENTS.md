# AGENTS.md — CDXTheme

Project rules for AI agents and contributors working in this repository.

## What this project is

**CDXTheme** is a native desktop theme manager for the **Codex / ChatGPT** desktop app (OpenAI). It is an independent community project, not affiliated with OpenAI.

It:

1. Manages selected appearance keys under `[desktop]` in `~/.codex/config.toml` (Windows: `%USERPROFILE%\.codex\config.toml`).
2. Injects live CSS/chrome skins into the host renderer over **Chrome DevTools Protocol (CDP)** on `127.0.0.1` (default port **9335**).
3. Packs/unpacks portable **`.cdxtheme`** packages (schema v1, max ~30 MB; no remote `@import` / `url(http…)`).
4. Restores managed keys from a one-time backup (`config.before.toml`) and removes injected DOM.
5. Ships auto-updates via Tauri updater metadata on `s3.cdxtheme.com`.
6. **Theme Builder**: chat UI that designs themes by talking to **Codex over ACP** (Agent Client Protocol), not CDP.

Primary targets today: **macOS 12+ (Apple Silicon)** and **Windows x64**. Linux is not a focus.

Product / user docs: root `README.md` (and locale variants). CLI: `cli/README.md`. Site: https://cdxtheme.com

## Workspace layout

Cargo workspace (`edition = "2024"`, version `0.1.3`, Rust **1.96.0** via `rust-toolchain.toml`):

| Path | Crate / role |
| --- | --- |
| `app-ui/` | Leptos **CSR** frontend (`cdx-theme`) → WASM via Trunk |
| `app-tauri/` | Tauri 2 shell, commands, plugins, bundling (`cdx-theme-app`, binary `CDXTheme`) |
| `core/` | Shared lib `cdx-theme-core`: pack/unpack, CDP inject, launch, apply, restore, **ACP Theme Builder** |
| `types/` | Shared types `cdx-theme-types` (theme metadata, loaded theme, verification) |
| `cli/` | `cdxtheme` CLI over core (pack/unpack/apply/**verify layout**/probe/screenshot) |
| `assets/renderer-inject.js` | Script injected into the host renderer |
| `public/` | Marketing assets / screenshots (not the WASM public dir) |
| `scripts/build.sh`, `scripts/build.ps1` | Release/debug/check builds |
| `skills/rust/` | Project skill: Rust / Tauri / Leptos notes (`SKILLS.md`) |
| `.agnets/skills/agent-client-protocol/` | Project skill: ACP + Theme Builder (`SKILL.md`) |

**Do not** put app UI under a root `src/` — frontend lives in `app-ui/`, backend in `app-tauri/`. Shared logic belongs in `core/` or `types/`, not duplicated in both hosts.

```text
CDXTheme (Tauri)
  ├── app-ui (Leptos WASM)  ──invoke──►  app-tauri commands
  └── app-tauri  ──uses──►  cdx-theme-core
                              ├── CDP ──► Codex/ChatGPT (theme inject / apply / restore)
                              ├── config.toml appearance + backup/restore
                              └── ACP ──► codex-acp ──► Codex CLI (Theme Builder chat)
```

## Runtime model (important)

### Appearance vs skin

- **Appearance**: keys written into Codex `config.toml` under `[desktop]`. Host restart is required only when those startup-loaded values change.
- **Skin**: CSS + embedded art injected live over CDP into `app://` renderer targets. Needs CDP connected.
- **Restore**: rewrites managed keys from `config.before.toml` and strips injected theme elements.

### Themes

- Local list: scan **builtin** + **user** package files only (`.cdxtheme` / recognized content). Directory-style themes are ignored at discover time.
- User install dir: app local data → `themes/`.
- Remote recommend catalog: `https://s3.cdxtheme.com/themes/index.json` (in-memory/disk cache TTL ~2 minutes).
- A package may declare multiple app targets; **runtime apply currently focuses on `targets.codex`**. Core also has WorkBuddy-related types/paths — do not assume multi-app UI parity without checking callers.
- Packages: schema version `1`. Prefer `theme.json` (else `manifest.json`) when packing from a source directory.

### CDP / host launch

- Default port `9335` (valid range for settings: 1024–65535).
- Background monitor updates `cdp_status`; do not auto-launch ChatGPT on every status poll without an explicit user action path.
- Changing CDP port persists settings and attempts to ensure Codex is relaunched with `--remote-debugging-port`.
- Inject timeout for large themes can be long (e.g. 120s) because multi-MB art goes through CDP WebSocket + base64→blob.

### Window chrome

- Opaque window, **no** transparent window / macOS private API.
- Overlay titlebar + solid native background colors synced with light/dark UI (`set_window_appearance`).
- Drag via pointer-down handlers / startDragging — `data-tauri-drag-region` does not bubble as one might expect.

## Frontend (`app-ui`)

- **Leptos 0.8 CSR**, Trunk serve on **http://localhost:1420**, Tailwind **4** (`style/tailwind.css`, Trunk tool pin `4.3.3`).
- UI deps via Bun/npm: Tailwind CLI only (`package.json`); install with `bun install` from `app-ui/` (or root scripts).
- Pages under `app-ui/src/pages/`: Recommend, Install, Library, Restore, Settings, Theme Builder.
- Theme Builder UI is a module (`app-ui/src/pages/theme_builder/`): `mod.rs` (`ThemeBuilderPage` + shared types/helpers), `builder_runtime_setup.rs` (bunx/npx gate + Install Bun), `builder_home.rs`, `builder_new_build.rs`, `builder_chat.rs`. On open, probes host for `codex-acp` / `bunx` / `npx`; if missing, shows setup UI and can install Bun via multi-mirror download (official, GitHub, jsDelivr).
- Shared state: `AppCtx` in `state.rs` (page, dark mode, locale) via `provide_context` / `use_context`.
- **All Tauri calls** go through `app-ui/src/api.rs` (`window.__TAURI__.core.invoke`). Keep invoke arg shapes in sync with backend command `rename_all` (many use `snake_case`).
- i18n: English, Simplified Chinese, Traditional Chinese, Japanese (`i18n.rs`). Prefer adding strings there rather than hardcoding copy in pages.
- Toasts: Sonner-style component under `components/ui/`.
- PostHog JS: config generated by `app-ui/build.rs` → `public/posthog-config.js` (gitignored). Trunk **must ignore** that file in watch (already in `Trunk.toml`) to avoid rebuild loops.

## Backend (`app-tauri`)

Notable modules:

| Module | Responsibility |
| --- | --- |
| `commands.rs` | Tauri IPC surface |
| `theme_catalog.rs` | Discover local themes, remote catalog, download/install/delete |
| `theme_tool.rs` / `injector/` | Apply/restore orchestration (wraps core) |
| `codex_launch.rs` | Find/relaunch host with debugging port |
| `cdp_monitor.rs` | Background CDP connectivity |
| `settings_store.rs` | CDP port, analytics, applied theme id, etc. |
| `image_cache.rs` | HTTP(S) preview → disk cache → `data:` URLs |
| `analytics.rs` | PostHog (posthog-rs) + opt-out state (default on) |
| `paths.rs` | App data / themes / cache locations |

### IPC commands (keep UI + backend aligned)

`retrieve_local_theme_list`, `fetch_remote_theme_catalog`, `resolve_cached_image`, `cdp_status`, `set_window_appearance`, `get_cdp_port`, `set_cdp_port`, `apply_theme`, `restore_theme`, `download_theme`, `install_theme`, `delete_theme`, `get_analytics_enabled`, `get_analytics_state`, `set_analytics_enabled`, `track_event`, **Theme Builder:** `check_theme_builder_runtime`, `install_bun_for_theme_builder`, `codex_chat`, `list_codex_sessions`, `list_codex_models`, `get_codex_session`.

Capabilities: `app-tauri/capabilities/default.json` (window drag/minimize/close/set-background-color, opener, log, updater). New privileged APIs need capability + command registration.

Logging: `tracing` + `tauri-plugin-log`; respect `RUST_LOG` filter syntax (default `info`).

## Core & CLI

- Prefer implementing pack/load/CDP/apply/restore once in **`cdx-theme-core`**, then call from Tauri and CLI.
- CLI binary name: **`cdxtheme`** (`cargo run -p cdx-theme-cli -- …` or `cargo install --path cli`).
- The desktop app **bundles** helpers via Tauri `bundle.externalBin` as **`cdxthemex`** and **`bun`** (`app-tauri/binaries/{name}-<triple>`). Staging: `scripts/prepare-cli-sidecar.*` / `prepare-bun-sidecar.*` (also run from `beforeBuildCommand` / `beforeDevCommand`). On macOS they land in **`CDXTheme.app/Contents/MacOS/`** next to `CDXTheme`. CLI cargo/install name stays `cdxtheme`; staged name is always **`cdxthemex`** (must not case-collide with `CDXTheme`). Bun pin: `BUN_VERSION` (default `latest`), re-download with `BUN_SIDECAR_FORCE=1`. Runtime prefers app-bundled Bun over system install.
- Supported portable formats include `cdxtheme` (`.cdxtheme`).
- Legacy single-file `.codex-theme` layouts are **not** supported.

## Toolchain & commands

Prereqs: Rust **1.96.0**, `wasm32-unknown-unknown`, Trunk, Tauri CLI 2, Bun or Node. macOS: Xcode CLT. Windows: WebView2.

```bash
# One-time
rustup target add wasm32-unknown-unknown
cargo install trunk
cargo install tauri-cli --version "^2"
# frontend CSS toolchain
(cd app-ui && bun install)

# Dev (Trunk + Tauri)
cargo tauri dev --manifest-path app-tauri/Cargo.toml
# or from repo root if configured: cargo tauri dev

# Typecheck
cargo check --manifest-path app-tauri/Cargo.toml
cargo check -p cdx-theme --target wasm32-unknown-unknown
./scripts/build.sh --check

# Tests (primarily Tauri/lib as available)
cargo test --manifest-path app-tauri/Cargo.toml --lib

# Release / debug bundles
./scripts/build.sh
./scripts/build.sh --debug
# Windows: .\scripts\build.ps1
```

Bundles land under `target/release/bundle/`. Release CI: `.github/workflows/release.yml`.

`rustfmt.toml`: **2-space** indent, `max_width = 100`, reorder imports. Prefer matching existing style over personal defaults.

## Environment & secrets

- Copy `.env.example` → `.env` (gitignored) for local analytics builds.
- `POSTHOG_API_KEY` / optional `POSTHOG_HOST` injected at build time into native (`app-tauri/build.rs`) and webview (`app-ui/build.rs`).
- Never commit `.env`, code-signing keys under `.tauri/`, or generated `app-ui/public/posthog-config.js`.
- PostHog project keys are public client tokens; still treat personal secrets and signing material carefully.

## Coding conventions

1. **Rust edition 2024**, workspace dependency versions only in root `Cargo.toml`; enable crate features in member manifests.
2. Shared types and package schema changes go in **`types/`** (and pack/load in **`core/`**), then update UI/CLI.
3. Tauri command args that use `rename_all = "snake_case"` must match `api.rs` serde args.
4. Prefer `Result<T, String>` at the IPC boundary (user-visible errors); use `thiserror` / structured errors inside core.
5. Do not introduce remote CSS loading in packages; security model forbids remote `@import` / `url(http…)`.
6. Do not switch the main window to transparent/private-API chrome without an explicit product decision.
7. Keep WASM size and inject payload size in mind — large assets are already a CDP bottleneck.
8. Analytics is **on by default** (opt-out); do not track when disabled, and respect settings / identify flows.
9. When adding UI strings, wire **i18n** for supported locales.
10. Prefer minimal, focused diffs; do not drive-by reformat unrelated modules.

## What not to break

- CDP default port and settings persistence.
- Apply pipeline order: ensure package on disk → write appearance → restart host only if needed → CDP inject.
- Remote catalog URL and package schema v1 compatibility for existing `.cdxtheme` files.
- Trunk public_url `./` (required for Tauri webview asset loading).
- Updater endpoint / signed artifacts assumptions in release packaging.

## Quick orientation for common tasks

| Task | Start here |
| --- | --- |
| New UI page or chrome | `app-ui/src/pages/`, `components/`, `app.rs` |
| New backend capability | `app-tauri/src/commands.rs` + `api.rs` + capabilities |
| Package format / inject / apply | `core/src/` |
| Theme list / remote catalog | `app-tauri/src/theme_catalog.rs` |
| Host process launch | `core/src/launch.rs`, `app-tauri/src/codex_launch.rs` |
| Injected DOM/CSS runtime | `assets/renderer-inject.js`, `core/src/inject/` |
| Theme Builder (ACP / Codex chat) | `core/src/codex_chat.rs`, `app-ui/src/pages/theme_builder/` |
| CLI authoring | `cli/`, `cli/README.md` |
| Build / CI | `scripts/`, `.github/workflows/release.yml` |
| Project agent skills | See **[Agent skills](#agent-skills-english)** below |

---

## Agent skills (English)

Project-local skills for AI agents. Prefer these over inventing ad-hoc workflows.

| Skill | Path | When to use |
| --- | --- | --- |
| **Rust / Tauri / Leptos** | `skills/rust/SKILLS.md` | Frontend/backend Rust changes, WASM, Tauri IPC, formatting |
| **Agent Client Protocol** | `.agnets/skills/agent-client-protocol/SKILL.md` | Theme Builder, Codex chat, ACP sessions, model select, `codex-acp` |

All skill documentation in this repository is written in **English**.

### Skill: Rust / Tauri / Leptos

**Path:** `skills/rust/SKILLS.md`

#### Stack

- Workspace: Rust **edition 2024**, pin via `rust-toolchain.toml` (see root).
- Frontend: **Leptos 0.8 CSR**, Trunk (`app-ui/`), Tailwind 4.
- Backend: **Tauri 2** (`app-tauri/`), shared logic in **`cdx-theme-core`**.
- Types: **`cdx-theme-types`** — package schema and UI-facing metadata.

#### Conventions (must follow)

1. **2-space indent**, `max_width = 100` (`rustfmt.toml`); do not reformat unrelated files.
2. Workspace dependency **versions** only in root `Cargo.toml`; enable features in member crates.
3. Tauri command args with `rename_all = "snake_case"` must match `app-ui/src/api.rs`.
4. IPC boundary: `Result<T, String>` for user-visible errors; structured errors inside core.
5. New UI strings go through **`app-ui/src/i18n.rs`** (all supported locales).
6. Prefer implementing pack/load/CDP/apply/ACP once in **`core/`**, then call from Tauri and CLI.
7. `cargo check` targets: native for Tauri/core; `wasm32-unknown-unknown` for `cdx-theme` UI.

#### Useful commands

```bash
cargo check --manifest-path app-tauri/Cargo.toml
cargo check -p cdx-theme --target wasm32-unknown-unknown
cargo check -p cdx-theme-core
./scripts/build.sh --check
```

#### Layout reminders

- Pages: `app-ui/src/pages/` · components: `app-ui/src/components/`
- Commands: `app-tauri/src/commands.rs` · capabilities: `app-tauri/capabilities/`
- Shared: `core/src/`, `types/`

---

### Skill: Agent Client Protocol (ACP)

**Path:** `.agnets/skills/agent-client-protocol/SKILL.md` (full detail)  
**Implementation:** `core/src/codex_chat.rs`  
**UI:** Theme Builder (`app-ui/src/pages/theme_builder/`: `mod.rs`, `builder_home.rs`, `builder_new_build.rs`, `builder_chat.rs`)  
**Crate:** `agent-client-protocol = "2.0.0"` (in `core/`)

#### What ACP is

ACP standardizes **client ↔ coding agent** communication (sessions, prompts, streaming, permissions, **session config options** such as model). It is to agents what **LSP** is to language servers.

| Protocol | Purpose |
| --- | --- |
| **ACP** | Client ↔ coding agent |
| **MCP** | Agent ↔ external tools/servers |
| **CDP** | CDXTheme ↔ ChatGPT renderer (theme **inject only**; not Theme Builder chat) |

- Spec: https://agentclientprotocol.com  
- Rust SDK: https://github.com/agentclientprotocol/rust-sdk  
- docs.rs: https://docs.rs/agent-client-protocol  

#### Local architecture

```text
Client (CDXTheme)  ──JSON-RPC over stdio──►  Agent process (codex-acp)
                                                    │
                                                    ▼
                                              Codex CLI
```

Client spawns the agent; messages go over **stdio**. Dropping the connection tears down the process group (including `npx` / `bunx` wrappers on Unix).

#### CDXTheme Theme Builder path

```text
Theme Builder UI
  ──invoke──►  codex_chat / list_codex_sessions / list_codex_models / get_codex_session
  ──core──►  agent-client-protocol Client
  ──stdio──►  codex-acp  (PATH binary, or bunx/npx @agentclientprotocol/codex-acp)
  ──►  Codex CLI (ChatGPT-bundled `…/Resources/codex` preferred on PATH)
```

**Not CDP.** Theme inject/apply still uses CDP; chat uses ACP only.

#### Lifecycle

1. Spawn / connect agent  
2. `initialize` (protocol version + capabilities)  
3. `session/new` or `session/load` (`cwd` must be absolute)  
4. Optional: `session/set_config_option` (e.g. model)  
5. `session/prompt` (content blocks; text baseline)  
6. Stream `session/update` notifications (`agent_message_chunk` → assistant text)  
7. Answer **permission** requests if the agent asks (Theme Builder auto-approves first option)  
8. Close connection  

Theme Builder currently uses a **fresh ACP connection per turn**, so model must be re-applied after every `session/new` / `session/load`.

#### Session ops

| Op | Direction | Use |
| --- | --- | --- |
| `session/new` | Client → Agent | First message of a new Theme Builder chat |
| `session/load` | Client → Agent | Continue with stored `session_id` |
| `session/list` | Client → Agent | Optional; fall back to `~/.codex/session_index.jsonl` |
| `session/set_config_option` | Client → Agent | Set model / reasoning / other session options |
| `session/prompt` | Client → Agent | User turn |
| `session/update` | Agent → Client | Streaming chunks, tool calls, plan |

#### Model selection (summary)

There is **no** dedicated modern `session/set_model`. Use generic config options:

| Concern | CDXTheme approach |
| --- | --- |
| List models for UI | Disk: `~/.codex/models_cache.json` + default from `config.toml` (`list_codex_models`) — do **not** spawn ACP just for the menu |
| Apply model on turn | After new/load: `session/set_config_option` with `configId: "model"` (codex-acp); **best-effort**, non-fatal |
| UI surfaces | Model menu on **New Build** + **Chat** only (`BuilderModelSelect`); shared `selected_model` signal |
| IPC | `list_codex_models`; `codex_chat(..., model: Option<String>)` → `CodexChatOptions.model` |

codex-acp option ids of interest: `model` (category `model`), `reasoning_effort` (category `thought_level`). Prefer matching `category == "model"` when parsing live `configOptions`.

#### Agent resolution (order)

1. `codex-acp` on `PATH`  
2. `bunx` / `npx -y @agentclientprotocol/codex-acp@latest`  
3. Prepend ChatGPT-bundled `codex` directory onto the agent’s `PATH`  

#### Runtime requirements

- **Bun/Node** if using the default bunx/npx adapter (or install `codex-acp` yourself).  
- **Codex CLI** (bundled with ChatGPT and/or on PATH).  
- **Auth:** `codex login` when needed (`~/.codex/auth.json`).  
- Theme Builder workspace cwd: app data `…/theme_builder/{id}` (absolute).  
- Prompt timeout budget: long (e.g. ~180s); inject/CDP is separate.

#### IPC (Theme Builder)

| Command | Role |
| --- | --- |
| `codex_chat` | `prompt` + optional `session_id` + optional `wait_ms` + optional `model` → ACP turn |
| `list_codex_sessions` | ACP list when available, else disk index |
| `list_codex_models` | Models from `~/.codex/models_cache.json` + current from `config.toml` |
| `get_codex_session` | Load transcript from `~/.codex` rollout JSONL |

#### Practical rules

1. Always pass **absolute** `cwd` into `session/new` and `session/load`.  
2. Persist **session ids** from `session/new` for multi-turn UI.  
3. Build the assistant bubble from **`agent_message_chunk`**, not only the prompt RPC result.  
4. Prefer ACP over scraping Codex TUI or brittle `codex exec` for host-app chat.  
5. Keep disk fallbacks for list/transcript **and models** when the adapter lacks APIs.  
6. Auto-approve permissions is a product shortcut — do not treat as safe for untrusted agents.  
7. **Re-apply model every turn** when using short-lived ACP connections; do not fail the turn if set-model errors.

#### Minimal Rust client sketch

```rust
use agent_client_protocol::schema::{ProtocolVersion, v1::*};
use agent_client_protocol::{AcpAgent, Agent, Client, ConnectionTo};

Client.builder()
  .on_receive_notification(/* SessionNotification: append AgentMessageChunk text */, …)
  .on_receive_request(/* RequestPermissionRequest: respond Selected/Cancelled */, …)
  .connect_with(AcpAgent::codex() /* or codex-acp config */, |cx: ConnectionTo<Agent>| async move {
    cx.send_request(InitializeRequest::new(ProtocolVersion::V1)).block_task().await?;
    let s = cx.send_request(NewSessionRequest::new(cwd)).block_task().await?;
    // or LoadSessionRequest::new(session_id, cwd)
    // optional: SetSessionConfigOptionRequest::new(s.session_id, "model", SessionConfigOptionValue::value_id(model))
    cx.send_request(PromptRequest::new(
      s.session_id,
      vec![ContentBlock::Text(TextContent::new(prompt))],
    )).block_task().await?;
    Ok(())
  })
  .await?;
```

#### References

| Resource | URL / path |
| --- | --- |
| ACP intro | https://agentclientprotocol.com/get-started/introduction |
| Rust SDK | https://github.com/agentclientprotocol/rust-sdk |
| Codex ACP adapter | `@agentclientprotocol/codex-acp` / agentclientprotocol/codex-acp |
| Core implementation | `core/src/codex_chat.rs` |
| Theme Builder UI | `app-ui/src/pages/theme_builder/` |
| Full skill file | `.agnets/skills/agent-client-protocol/SKILL.md` |
