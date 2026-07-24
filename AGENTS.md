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

Primary targets today: **macOS 12+ (Apple Silicon)** and **Windows x64**. Linux is not a focus.

Product / user docs: root `README.md` (and locale variants). CLI: `cli/README.md`. Site: https://cdxtheme.com

## Workspace layout

Cargo workspace (`edition = "2024"`, version `0.1.3`, Rust **1.96.0** via `rust-toolchain.toml`):

| Path | Crate / role |
| --- | --- |
| `app-ui/` | Leptos **CSR** frontend (`cdx-theme`) → WASM via Trunk |
| `app-tauri/` | Tauri 2 shell, commands, plugins, bundling (`cdx-theme-app`, binary `CDXTheme`) |
| `core/` | Shared lib `cdx-theme-core`: pack/unpack, CDP inject, launch, apply, restore |
| `types/` | Shared types `cdx-theme-types` (theme metadata, loaded theme, verification) |
| `cli/` | `cdxtheme` CLI over core |
| `assets/renderer-inject.js` | Script injected into the host renderer |
| `public/` | Marketing assets / screenshots (not the WASM public dir) |
| `scripts/build.sh`, `scripts/build.ps1` | Release/debug/check builds |
| `skills/rust/` | Optional Rust agent skill notes |

**Do not** put app UI under a root `src/` — frontend lives in `app-ui/`, backend in `app-tauri/`. Shared logic belongs in `core/` or `types/`, not duplicated in both hosts.

```text
CDXTheme (Tauri)
  ├── app-ui (Leptos WASM)  ──invoke──►  app-tauri commands
  └── app-tauri  ──uses──►  cdx-theme-core  ──CDP──►  Codex/ChatGPT
                              │
                              └── config.toml appearance + backup/restore
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
- Pages under `app-ui/src/pages/`: Recommend, Install, Library, Restore, Settings.
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
| `analytics.rs` | PostHog (posthog-rs) + opt-in state |
| `paths.rs` | App data / themes / cache locations |

### IPC commands (keep UI + backend aligned)

`retrieve_local_theme_list`, `fetch_remote_theme_catalog`, `resolve_cached_image`, `cdp_status`, `set_window_appearance`, `get_cdp_port`, `set_cdp_port`, `apply_theme`, `restore_theme`, `download_theme`, `install_theme`, `delete_theme`, `get_analytics_enabled`, `get_analytics_state`, `set_analytics_enabled`, `track_event`.

Capabilities: `app-tauri/capabilities/default.json` (window drag/minimize/close/set-background-color, opener, log, updater). New privileged APIs need capability + command registration.

Logging: `tracing` + `tauri-plugin-log`; respect `RUST_LOG` filter syntax (default `info`).

## Core & CLI

- Prefer implementing pack/load/CDP/apply/restore once in **`cdx-theme-core`**, then call from Tauri and CLI.
- CLI binary name: **`cdxtheme`** (`cargo run -p cdx-theme-cli -- …` or `cargo install --path cli`).
- Supported portable formats include `cdxtheme` (`.cdxtheme`) and CodeDrobe-compatible packing; packing rewrites `codedrobe-` CSS tokens to `cdxtheme-`.
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
8. Analytics is **opt-in**; do not track without respecting settings / identify flows.
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
| CLI authoring | `cli/`, `cli/README.md` |
| Build / CI | `scripts/`, `.github/workflows/release.yml` |
