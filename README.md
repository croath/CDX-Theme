<p align="center">
  <img src="public/logo.png" width="128" alt="CDXTheme logo">
</p>

<h1 align="center">CDXTheme</h1>

<p align="center">
  A native desktop theme manager that gives Codex and ChatGPT a look of their own.
</p>

<p align="center">
  <a href="https://cdxtheme.com"><strong>cdxtheme.com</strong></a>
</p>

<p align="center">
  <strong>English</strong> ·
  <a href="README.zh-CN.md">简体中文</a> ·
  <a href="README.ja.md">日本語</a> ·
  <a href="README.ko.md">한국어</a>
</p>

<p align="center">
  <a href="https://github.com/croath/CDX-Theme/releases/latest"><img src="https://img.shields.io/github/v/release/croath/CDX-Theme?style=flat-square&logo=github&label=release" alt="Latest release"></a>
  <a href="https://github.com/croath/CDX-Theme/releases"><img src="https://img.shields.io/github/downloads/croath/CDX-Theme/total?style=flat-square&logo=github" alt="Downloads"></a>
  <a href="https://github.com/croath/CDX-Theme/actions/workflows/release.yml"><img src="https://img.shields.io/github/actions/workflow/status/croath/CDX-Theme/release.yml?style=flat-square&logo=githubactions&logoColor=white&label=release" alt="Release build"></a>
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows-555?style=flat-square&logo=apple" alt="macOS and Windows">
  <img src="https://img.shields.io/badge/Rust-1.96-orange?style=flat-square&logo=rust" alt="Rust 1.96">
  <img src="https://img.shields.io/badge/Tauri-2-24C8D8?style=flat-square&logo=tauri&logoColor=white" alt="Tauri 2">
  <a href="#license"><img src="https://img.shields.io/badge/license-proprietary-lightgrey?style=flat-square" alt="Proprietary license"></a>
</p>

> [!NOTE]
> CDXTheme is an independent community project and is not affiliated with or endorsed by OpenAI.

## Sponsor CDXTheme

[Want to appear in the sponsor list?](mailto:business@cdxtheme.com)

<table>
  <tbody>
    <tr>
      <td width="300">
        <img src="public/sponsors/yylx-logo.jpg" width="80" align="center" alt="Yylx logo">&nbsp;&nbsp;
        <a href="https://yylx.io"><strong>鱼鱼连线中转站</strong></a>
      </td>
      <td>Yylx provides a unified AI model API gateway optimized for Claude Code workflows. Switch between Claude and OpenAI GPT models by changing a single configuration line. It gives developers a convenient and reliable way to connect to leading AI models.</td>
    </tr>
  </tbody>
</table>

## Use CDXTheme

### 1. Download

Visit the [official website](https://cdxtheme.com) or get the newest installer directly from [GitHub Releases](https://github.com/croath/CDX-Theme/releases/latest).

| Platform | Package | Status |
| --- | --- | --- |
| macOS 12+ (Apple Silicon) | `.dmg` | Supported |
| Windows x64 | NSIS `.exe` | Supported |
| Linux | — | Not currently targeted |

CDXTheme expects the Codex / ChatGPT desktop app to be installed. It communicates with the app locally through Chrome DevTools Protocol (CDP) on `127.0.0.1`; the default port is `9335`.

### 2. Choose and apply a theme

1. Open **Recommend** to browse available and installed themes.
2. Select a theme and apply it with one click.
3. If requested, allow CDXTheme to relaunch Codex / ChatGPT with the CDP port enabled.

CDXTheme updates supported appearance keys in `~/.codex/config.toml` and injects the live CSS skin into the desktop renderer. Codex is restarted only when startup-loaded appearance values actually change.

### 3. Install your own package

Open **Install** and import either supported portable format:

| Extension | Package `format` |
| --- | --- |
| `.cdxtheme` | `cdxtheme` |
| `.codedrobe-theme` | `codedrobe-theme` |

Packages use schema version `1`, may be up to **30 MB**, and cannot load remote CSS through `@import` or `url(http…)`. A package can describe multiple app targets, but CDXTheme currently applies only `targets.codex`.

### 4. Restore the default appearance

Choose **Restore** to revert the managed appearance values from the one-time backup and remove injected theme elements from the live renderer.

### What you can do

- Browse built-in, remote, and locally installed themes.
- Install and remove portable theme packages.
- Apply appearance settings and live CSS/chrome skins together.
- Restore Codex / ChatGPT to its previous managed appearance.
- Switch CDXTheme between light, dark, and system appearance.
- Use English, Simplified Chinese, Traditional Chinese, or Japanese in the app.
- Configure the CDP port and relaunch the host app when needed.

## Theme authoring CLI

The Rust CLI is a thin interface over the shared `cdx-theme-core` library. See the [complete CLI guide](cli/README.md) for every option.

```bash
cargo install --path cli

# Pack a source directory into a portable package
cdxtheme theme pack path/to/theme-source

# Unpack or convert a package
cdxtheme theme unpack theme.cdxtheme path/to/output
cdxtheme theme convert theme.codedrobe-theme

# Apply a package directly through CDP
cdxtheme apply --app codex --theme theme.cdxtheme
```

A source directory uses `theme.json` (preferred) or `manifest.json`, plus CSS and optional image assets.

## Technical overview

### How it works

```text
                         ~/.codex/config.toml
                    ┌──────────────────────────► startup appearance
                    │
┌──────────────┐    │    CDP on 127.0.0.1:9335
│   CDXTheme   │────┼──────────────────────────► live renderer skin
│  Tauri app   │    │
└──────────────┘    └──────────────────────────► backup / restore
```

1. **Appearance** — manages selected keys under `[desktop]` in the Codex config.
2. **Skin** — injects package CSS and embedded art into `app://` renderer targets over CDP.
3. **Restore** — recovers managed keys from `config.before.toml` and removes injected DOM.
4. **Updates** — checks signed Tauri updater metadata and installs available releases.

### Stack and architecture

| Layer | Technology | Responsibility |
| --- | --- | --- |
| Desktop shell | Tauri 2 | Native windows, commands, updater, bundling |
| Frontend | Rust · Leptos 0.8 · WASM | Client-side UI and state |
| Styling | Tailwind CSS 4 | Application UI styles |
| Host integration | Rust · CDP | Launch, inject, verify, and restore |
| Build | Cargo · Trunk · Bun | Workspace, WASM bundle, frontend dependencies |

```text
├── src/          # Leptos CSR frontend
├── app-tauri/    # Tauri backend and desktop bundle
├── core/         # Shared package, launch, apply, and injection logic
├── cli/          # cdxtheme authoring CLI
├── types/        # Shared theme types
├── assets/       # Renderer injection script
├── public/       # Static assets
├── style/        # Tailwind entry point
└── scripts/      # Build and optional helper scripts
```

### Development

You need [Rust](https://rustup.rs/) `1.96.0`, the `wasm32-unknown-unknown` target, [Trunk](https://trunkrs.dev/), Tauri CLI 2, and Bun or Node. macOS development also requires Xcode Command Line Tools; Windows requires WebView2.

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
cargo install tauri-cli --version "^2"
bun install
cargo tauri dev
```

Trunk serves the frontend at `http://localhost:1420`. Debug builds log to the terminal and the platform app-log directory and automatically open Web Inspector.

Useful checks:

```bash
cargo check --manifest-path app-tauri/Cargo.toml
cargo check --target wasm32-unknown-unknown
cargo test --manifest-path app-tauri/Cargo.toml --lib
```

### Build

```bash
# macOS / Linux host
./scripts/build.sh
./scripts/build.sh --debug
./scripts/build.sh --check

# Direct Tauri build
cargo tauri build --manifest-path app-tauri/Cargo.toml
```

```powershell
# Windows PowerShell
.\scripts\build.ps1
.\scripts\build.ps1 -Debug
.\scripts\build.ps1 -Check
```

Bundles are written beneath `target/release/bundle/`. Publishing a GitHub Release triggers the release workflow for Apple Silicon macOS and Windows x64 artifacts.

### Defaults and paths

| Item | Default / path |
| --- | --- |
| CDP endpoint | `127.0.0.1:9335` |
| Codex config | `~/.codex/config.toml` |
| Windows Codex config | `%USERPROFILE%\.codex\config.toml` |
| First-apply backup | app data directory → `config.before.toml` |
| User themes | app local data directory → `themes/` |

## Troubleshooting

<details>
<summary><strong>Codex / ChatGPT is not found</strong></summary>

Install the desktop app first. On Windows, CDXTheme also detects the Microsoft Store package named `OpenAI.Codex`.
</details>

<details>
<summary><strong>CDP is disconnected</strong></summary>

Open **Settings**, confirm the port, then save and relaunch. The same port must be available to both CDXTheme and the host app.
</details>

<details>
<summary><strong>The appearance or skin did not update</strong></summary>

Startup appearance values require a host restart; live CSS requires a CDP connection. Reapply the theme after confirming the connection status.
</details>

## License

Proprietary / as provided by the project author unless otherwise stated. Third-party components remain subject to their respective licenses.

---

<p align="center">
  <a href="https://cdxtheme.com">Website</a> ·
  <a href="https://github.com/croath/CDX-Theme/releases/latest">Download</a> ·
  <a href="https://github.com/croath/CDX-Theme/issues">Issues</a> ·
  <a href="cli/README.md">CLI documentation</a> ·
  <a href="mailto:business@cdxtheme.com">Sponsor inquiry</a>
</p>
