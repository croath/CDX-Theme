---
name: rust
description: >
  Rust, Tauri 2, and Leptos 0.8 conventions for the CDXTheme monorepo.
  Use when editing app-ui, app-tauri, core, types, or cli Rust code.
---

# Rust / Tauri / Leptos (CDXTheme)

English skill notes for agents working in this repository.

## Stack

| Layer | Tech |
| --- | --- |
| Workspace | Cargo, edition **2024**, pin in `rust-toolchain.toml` |
| Frontend | Leptos **0.8 CSR**, Trunk, Tailwind **4** (`app-ui/`) |
| Shell | Tauri **2** (`app-tauri/`, binary `CDXTheme`) |
| Shared | `cdx-theme-core`, `cdx-theme-types` |
| CLI | `cdx-theme-cli` → `cdxtheme` / bundled sidecar `cdxthemex` |

## Where code lives

| Concern | Path |
| --- | --- |
| UI pages | `app-ui/src/pages/` |
| UI components | `app-ui/src/components/` |
| Tauri invoke API | `app-ui/src/api.rs` |
| i18n | `app-ui/src/i18n.rs` |
| Tauri commands | `app-tauri/src/commands.rs` |
| Capabilities | `app-tauri/capabilities/` |
| Pack / inject / ACP / launch | `core/src/` |
| Package schema types | `types/` |

Do **not** put app UI under a root `src/`. Do **not** duplicate core logic in both Tauri and CLI.

## Conventions

1. **rustfmt:** 2-space indent, `max_width = 100`; minimal, focused diffs.
2. Dependency **versions** only in root `Cargo.toml`; features in member manifests.
3. Tauri `rename_all = "snake_case"` args must match `api.rs` serde shapes.
4. IPC: `Result<T, String>` at the boundary; structured errors inside core.
5. New copy: add strings for all locales in `i18n.rs`.
6. No remote CSS in theme packages (`@import` / `url(http…)` forbidden).
7. Analytics is opt-in only.
8. Prefer `core/` for pack, CDP inject, launch, apply, restore, and ACP chat.

## Commands

```bash
# Typecheck native app
cargo check --manifest-path app-tauri/Cargo.toml

# Typecheck WASM UI
cargo check -p cdx-theme --target wasm32-unknown-unknown

# Core only
cargo check -p cdx-theme-core

# Workspace check script
./scripts/build.sh --check

# Dev
cargo tauri dev --manifest-path app-tauri/Cargo.toml
```

## Theme Builder note

Theme Builder chat uses **ACP** (`core/src/codex_chat.rs`), not CDP.  
CDP remains for **live skin inject** into ChatGPT. See `.agnets/skills/agent-client-protocol/SKILL.md` and root `AGENTS.md` § Agent skills.

## Related

- Project rules: root `AGENTS.md`
- ACP skill: `.agnets/skills/agent-client-protocol/SKILL.md`
