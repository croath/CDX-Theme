# CDXTheme

Desktop theme manager for the **Codex / ChatGPT** app.

Browse built-in themes, install portable packages, apply appearance + live CSS skins, and restore defaults — all from a native desktop UI.

| Platform | Status |
|----------|--------|
| macOS | Supported |
| Windows | Supported |
| Linux | Not targeted |

---

## Features

- **Recommend** — list built-in and installed themes; apply in one click  
- **Install** — import `.cdxtheme` / `.codedrobe-theme` multi-app packages  
- **Restore** — roll back managed appearance keys and remove injected skins  
- **Settings** — language (EN / 简中 / 繁中 / JP), light·dark UI, CDP port  
- **CDP inject** — live CSS/chrome skin via Chrome DevTools Protocol  
- **Config apply** — write Codex `~/.codex/config.toml` appearance (restarts only when values change)  
- **macOS overlay chrome** — seamless title bar without private API  
- **Window drag** — drag non-interactive areas; double-click to maximize  

---

## How it works

```
┌─────────────┐     config.toml      ┌──────────────────┐
│  CDXTheme   │ ───────────────────► │ Codex appearance │
│             │                      │ (startup load)   │
│             │   CDP inject CSS     │                  │
│             │ ───────────────────► │ Renderer skin    │
└─────────────┘                      └──────────────────┘
```

1. **Appearance** — managed keys under `[desktop]` in `~/.codex/config.toml`  
   (`appearanceTheme`, `appearanceLightCodeThemeId`, `appearanceLightChromeTheme`).  
   Codex loads these at startup; CDXTheme restarts Codex only when they actually change.

2. **Skin** — package CSS + art injected into the Codex renderer over CDP  
   (`--remote-debugging-port`, default `9335`).

3. **Restore** — reverts managed keys from a one-time backup and strips injected DOM.

---

## Requirements

### Runtime

- **Codex / ChatGPT desktop app** installed  
  - macOS: `ChatGPT.app` / bundle id `com.openai.codex`  
  - Windows: desktop install or Microsoft Store `OpenAI.Codex`
- Network loopback for CDP (`127.0.0.1`)

### Development

- [Rust](https://rustup.rs/) (see `rust-toolchain.toml`, channel `1.96.0`)
- [Trunk](https://trunkrs.dev/) — WASM frontend bundler  
- [Tauri CLI 2](https://v2.tauri.app/)  
- Node (for Tailwind via Trunk / optional `npx @tauri-apps/cli`)  
- macOS: Xcode CLT · Windows: WebView2 (embedBootstrapper bundled, supports Windows 7+)

```bash
# examples
cargo install trunk
cargo install tauri-cli --version "^2"
rustup target add wasm32-unknown-unknown
```

---

## Develop

```bash
# from repo root
cargo tauri dev
# or
npm exec --package @tauri-apps/cli@2 -- tauri dev
```

Frontend is served by Trunk on **http://localhost:1420** (`Trunk.toml`).

### Debug logging (`tauri dev`)

Debug builds enable **`log::Debug`**, print to the terminal, write rotating files under the app log dir, and open **Web Inspector** automatically.

| Output | Where |
|--------|--------|
| Terminal | stdout (from `cargo tauri dev`) |
| Log file | macOS: `~/Library/Logs/com.cdxtheme.cdx/cdxtheme.log` |
| Webview | DevTools console (Webview target + auto `open_devtools`) |

Release builds use **`Info`** level and do not open DevTools.

Useful checks:

```bash
cargo check --manifest-path app-tauri/Cargo.toml
cargo check --target wasm32-unknown-unknown
cargo test --manifest-path app-tauri/Cargo.toml --lib
```

---

## Build

Use the project build scripts (installs missing toolchain pieces when possible):

```bash
# macOS / Linux
./scripts/build.sh              # release app bundle
./scripts/build.sh --debug      # debug build
./scripts/build.sh --clean      # clean then release
./scripts/build.sh --check      # typecheck only
./scripts/build.sh --frontend   # trunk build only
```

```powershell
# Windows (PowerShell)
.\scripts\build.ps1
.\scripts\build.ps1 -Debug
.\scripts\build.ps1 -Clean
.\scripts\build.ps1 -Check
```

Or invoke Tauri directly:

```bash
cargo tauri build --manifest-path app-tauri/Cargo.toml
```

Outputs under `target/release/bundle/` (workspace target; `.app` / `.dmg` on macOS, NSIS .exe on Windows).

Regenerate app icons from source art:

```bash
npm exec --package @tauri-apps/cli@2 -- tauri icon app-tauri/app-icon-source.png
```

### GitHub Release CI

Workflow: [`.github/workflows/release.yml`](.github/workflows/release.yml).

| Trigger | Behavior |
|---------|----------|
| **Publish a GitHub Release** | Builds macOS (Apple Silicon / arm64) and Windows (NSIS), uploads installers + updater `.sig` / `latest.json` to that release |
| **workflow_dispatch** | Manual run; optional `release_tag` to attach assets to an existing release |

**Repo setting:** Settings → Actions → General → Workflow permissions → **Read and write**.

**Secrets** (Settings → Secrets and variables → Actions):

| Secret | Required | Purpose |
|--------|----------|---------|
| `TAURI_SIGNING_PRIVATE_KEY` | **Yes** | Updater minisign private key (`cargo tauri signer generate`) |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | **Yes** | Key password (use empty secret if none) |
| `APPLE_CERTIFICATE` | Recommended | Base64 `.p12` Developer ID Application cert |
| `APPLE_CERTIFICATE_PASSWORD` | Recommended | Password for the `.p12` |
| `APPLE_SIGNING_IDENTITY` | Optional | e.g. `Developer ID Application: Name (TEAMID)` |
| `APPLE_API_ISSUER` + `APPLE_API_KEY` + `APPLE_API_KEY_CONTENT` | Notarization | App Store Connect API key (only method supported in release workflow) |
| `WINDOWS_CERTIFICATE` + `WINDOWS_CERTIFICATE_PASSWORD` | Optional | Base64 `.pfx` for Windows signing (NSIS) |

Generate updater keys (public key must match `plugins.updater.pubkey` in `app-tauri/tauri.conf.json`):

```bash
cargo tauri signer generate -w ~/.tauri/cdxtheme.key
```

**Windows Authenticode (self-signed, local):** materials live under [`.tauri/`](.tauri/) (see [`.tauri/README.md`](.tauri/README.md)).

```bash
# Regenerate if needed
./.tauri/generate-windows-codesign.sh
# Export env for local/CI-style bundling
source .tauri/export-windows-signing.sh
```

`app-tauri/tauri.conf.json` → `bundle.windows` sets `digestAlgorithm` + `timestampUrl` + `webviewInstallMode` (NSIS).  
PFX password is **not** stored in config (use env / secrets only).

Export Apple cert for CI:

```bash
openssl base64 -A -in certificate.p12 -out certificate-base64.txt
```

---

## Theme packages

### Portable formats

Multi-app schema (same shape for both brands; see [CodeDrobe package.mjs](https://github.com/CodeDrobe/core/blob/main/src/theme/package.mjs)):

| Extension | `format` field |
|-----------|----------------|
| `.cdxtheme` | `cdxtheme` (CDXTheme default) |
| `.codedrobe-theme` | `codedrobe-theme` |

Schema version `1`. Max size **30MB**. CSS must not load remote `@import` / `url(http…)`.  

**Runtime loading:** the app catalogs and applies **only** `.cdxtheme` / `.codedrobe-theme` package files (not source directories).  
Packages may include multiple `targets` (`codex`, `workbuddy`, …); **only `targets.codex` is read and applied today**.

Example package (`tmp/doll-sister.cdxtheme` shape):

```json
{
  "format": "cdxtheme",
  "schemaVersion": 1,
  "exportedAt": "…",
  "theme": {
    "id": "doll-sister",
    "displayName": "Doll Sister",
    "version": "1.0.0",
    "copy": { "brandTitle": "…", "tagline": "…" }
  },
  "targets": {
    "codex": {
      "css": "/* inlined stylesheet */",
      "options": {
        "rendererProfile": "codex-theme-v1",
        "baseTheme": { "mode": "light", "accent": "…" }
      }
    }
  },
  "assets": {
    "images": {
      "hero": { "filename": "art.png", "mimeType": "image/png", "base64": "…" }
    }
  }
}
```

- **Built-in:** package files under repo `themes/` (bundled as app resources), e.g. `themes/doll-sister.cdxtheme`
- **Installed:** same extensions under app local data `themes/`

### Core library

Shared host logic lives in **`core/`** (`cdx-theme-core`):

| Module | Role |
|--------|------|
| `pack` | `pack_theme_dir` / `unpack_package` / `convert_package` |
| `package` | Load portable packages → `LoadedTheme` |
| `inject` | CDP apply / restore / verify |
| `launch` | Ensure Codex remote-debugging |
| `apply` | High-level: ensure CDP → inject |

**CLI** and **app-tauri** both depend on this crate (no duplicated inject/pack paths).

### CLI (authoring source → package)

Rust CLI (`cli/`, binary `cdxtheme`). Full usage: **[cli/README.md](cli/README.md)**.

```bash
cargo install --path cli
# or
cargo run -p cdx-theme-cli -- <command>

# Source dir uses theme.json (preferred) or manifest.json
cdxtheme theme pack path/to/theme-source
# → {id}-{version}.cdxtheme
cdxtheme theme unpack doll-sister.cdxtheme path/to/out
# CodeDrobe → CDXTheme (format + CSS codedrobe- → cdxtheme-)
cdxtheme theme convert path/to/theme.codedrobe-theme
# → {id}-{version}.cdxtheme

# Ensure Codex CDP, then inject a package
cdxtheme apply --app codex --theme path/to/theme.cdxtheme
```

### Node helpers (optional)

`scripts/` still contains older Node tooling:

| Script | Role |
|--------|------|
| `theme-package.mjs` / `export-theme.mjs` | Build portable packages |
| `injector.mjs` | CDP inject from CLI |
| `theme-tool.mjs` | Apply / restore config |
| `start-codedrobe.sh` / `.ps1` | Launch Codex with debug port |

---

## Settings

| Setting | Default | Notes |
|---------|---------|--------|
| CDP port | `9335` | Used as `--remote-debugging-port` when launching Codex |
| Language | system / EN | 简体中文 · 繁體中文 · English · 日本語 |
| UI theme | system | Light / dark for CDXTheme itself |

Config backup (first apply): app data `config.before.toml`.  
Codex config path: `~/.codex/config.toml` (Windows: `%USERPROFILE%\.codex\config.toml`).

---

## Project layout

```
├── src/                 # Leptos CSR frontend (WASM)
├── app-tauri/           # Tauri 2 backend (CDP, catalog, launch, config)
├── types/               # Shared ThemeMetadata / ThemeSource
├── themes/              # Built-in theme packages
├── assets/              # Injected renderer script (renderer-inject.js)
├── scripts/             # Optional Node/PowerShell helpers
├── public/              # Static assets (logo.png)
├── style/               # Tailwind entry
└── Trunk.toml
```

Stack: **Tauri 2 · Rust · Leptos 0.8 · Trunk · Tailwind 4**.

---

## Troubleshooting

**Codex not found**  
Install ChatGPT / Codex desktop. On Windows, Store package `OpenAI.Codex` is detected via Appx.

**CDP disconnected**  
Open Settings → confirm port → Save & relaunch. Codex must expose remote debugging on that port.

**Appearance didn’t update**  
Appearance only loads at Codex startup. Apply restarts Codex when config appearance keys change; re-apply if needed.

**Skin didn’t apply**  
Confirm CDP status is connected, then apply again (inject is best-effort after config write).

**macOS name wrong in menu / Cmd+Tab**  
Rebuild/reinstall the app; `productName` / `CFBundleName` are set to **CDXTheme**. Log out or clear Launch Services cache if an old name sticks.

---

## License

Proprietary / as provided by the project author unless otherwise stated.
