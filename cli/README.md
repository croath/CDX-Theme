# cdxtheme CLI

Thin CLI over shared library **`cdx-theme-core`**.

Pack, unpack, apply, restore, adjust appearance, and **verify** multi-app portable theme packages (`.cdxtheme`) over CDP.

| Brand | `format` field | Extension |
|-------|----------------|-----------|
| **CDXTheme** (default) | `cdxtheme` | `.cdxtheme` |

Legacy single-file formats (`.codex-theme` with top-level `manifest` + `css`) are **not** supported.

## Install

```bash
cargo install --path cli
# or
cargo run -p cdx-theme-cli -- <command>
```

Binary: **`cdxtheme`**.

## Usage

```text
cdxtheme theme pack <SOURCE> [OPTIONS]
cdxtheme theme unpack <INPUT> <OUTPUT>
cdxtheme theme merge-css <SOURCE> [-t codex|workbuddy]
cdxtheme apply --app codex|workbuddy --theme <PACKAGE> [OPTIONS]
cdxtheme restore [--app codex|workbuddy] [OPTIONS]
cdxtheme detect [--json]
cdxtheme appearance <dark|light|system> [OPTIONS]
cdxtheme verify inject [-t PACKAGE]
cdxtheme verify layout [--contexts chat,work]
cdxtheme probe [--tab chat|work] [--expr JS]
cdxtheme screenshot -o OUT.jpg [--tab chat|work]
```

### `theme pack`

```bash
cdxtheme theme pack themes/ferrari
# → (default) merge partials in memory into package (no root codex.css written)
# → ferrari-1.0.0.cdxtheme

cdxtheme theme pack themes/ferrari --pretty --force
cdxtheme theme pack themes/ferrari --no-merge-css   # skip partial merge
cdxtheme theme pack themes/ferrari/theme.json -o dist/ferrari.cdxtheme
cdxtheme theme pack themes/ferrari/manifest.json -o dist/ferrari.cdxtheme
```

| Flag | Description |
|------|-------------|
| `-o`, `--output` | Output path (default `{id}-{version}.cdxtheme`) |
| `--pretty` | Pretty-print JSON |
| `--force` | Overwrite existing file |
| `--no-merge-css` | Skip in-memory partial merge into the package |


**Source layout**

When packing a directory, the CLI looks for source JSON in this order:

1. `theme.json`
2. `manifest.json` (if `theme.json` is missing)

```text
themes/ferrari/
  theme.json         # preferred (or manifest.json)
  style.css
  assets/art.png
```

```json
{
  "schemaVersion": 1,
  "id": "ferrari",
  "displayName": "Ferrari",
  "version": "1.0.0",
  "copy": { "brandTitle": "…" },
  "targets": {
    "codex": {
      "css": "style.css",
      "options": {
        "rendererProfile": "codex-theme-v1",
        "baseTheme": { "mode": "dark", "accent": "#DC0000" }
      }
    }
  },
  "images": { "hero": "assets/art.png" }
}
```

### `theme unpack`

```bash
cdxtheme theme unpack ferrari-1.0.0.cdxtheme /tmp/ferrari
# → theme.json + codex/theme.css + images/…
```

Accepts `.cdxtheme` .

### `theme merge-css`

Merge split authoring partials into the root CSS files that `theme.json` → `targets.*.css` points at.

```bash
cdxtheme theme merge-css themes/my-theme
# → themes/my-theme/codex.css from themes/my-theme/codex/*.css
# → themes/my-theme/workbuddy.css from themes/my-theme/workbuddy/*.css

cdxtheme theme merge-css themes/my-theme -t codex
```

| Convention | Meaning |
| --- | --- |
| `{theme}/{target}/*.css` | Partials (alphabetical order; use `00-`, `01-` prefixes) |
| `{theme}/{target}.css` | Merged output (generated — do not hand-edit) |

**`theme pack` merges partials in memory** into the `.cdxtheme` (does not write root `codex.css` / `workbuddy.css`). Use `merge-css` only when you want those merged files on disk for inspection or editing.

Typical starter layout:

```text
src/my-theme/
  theme.json          # "css": "codex.css"
  codex/
    00-tokens.css     # brand colors only
    01-shell.css
    02-chrome.css
    03-home.css
    04-composer.css
    05-work.css
    06-settings.css
  workbuddy/
    00-tokens.css
    01-shell.css
    02-home.css
    03-composer.css
  codex.css           # generated
  workbuddy.css       # generated
```

### `apply`

Inject a theme package into a live host app over CDP.

1. Probe CDP on the remote-debugging port
2. If not connected, launch (or restart) the host with `--remote-debugging-port`
3. Inject the package CSS/skin into live page targets

| Host | Default CDP | Targets | Notes |
|------|-------------|---------|--------|
| `codex` | **9335** | `app://` | ChatGPT / Codex desktop |
| `workbuddy` | **9336** | `file://` (asar renderer) | WorkBuddy AI Electron app |

The package must declare the matching `targets.codex` / `targets.workbuddy` CSS.

```bash
cdxtheme apply --app codex --theme ferrari-1.0.0.cdxtheme
cdxtheme apply -t themes/doll-sister.cdxtheme --port 9335
cdxtheme apply --app workbuddy -t ferrari-1.0.0.cdxtheme
cdxtheme apply --app workbuddy -t ferrari-1.0.0.cdxtheme --port 9336
```

| Flag | Description |
|------|-------------|
| `--app` | Host app id: `codex` (default) or `workbuddy` |
| `-t`, `--theme` | Path to `.cdxtheme` |
| `--port` | CDP port (default `9335` codex / `9336` workbuddy) |
| `--timeout-ms` | Wait / inject timeout (default `120000`) |

### `restore`

Remove the injected CSS/skin from live renderer targets (inverse of `apply` inject).

1. Probe CDP on the remote-debugging port
2. If not connected, launch (or restart) the host with remote debugging
3. Strip injected theme DOM / early scripts from matching page targets

Does **not** rewrite `~/.codex/config.toml` appearance keys (use `appearance` for mode).

```bash
cdxtheme restore
cdxtheme restore --port 9335 --timeout-ms 120000
cdxtheme restore --app workbuddy
```

| Flag | Description |
|------|-------------|
| `--app` | Host app id: `codex` (default) or `workbuddy` |
| `--port` | CDP port (default `9335` codex / `9336` workbuddy) |
| `--timeout-ms` | Wait / restore timeout (default `120000`) |

### `detect`

Probe whether host apps are installed, running, and exposing CDP on their default ports.

| Host | Default CDP port |
|------|------------------|
| Codex / ChatGPT | `9335` |
| WorkBuddy AI | `9336` |

```bash
cdxtheme detect
cdxtheme detect --json
```

Human output shows install path, process state, and CDP reachability per host. `--json` prints the full report.

### `appearance`

Set ChatGPT / Codex host appearance mode by writing `[desktop].appearanceTheme` in `~/.codex/config.toml`.

Supported modes: **`dark`**, **`light`**, **`system`**.

When the value changes, Codex is restarted with remote debugging so the mode takes effect (unless `--no-restart`).

```bash
cdxtheme appearance dark
cdxtheme appearance light
cdxtheme appearance system
cdxtheme appearance dark --no-restart
cdxtheme appearance light --config /path/to/config.toml
```

| Flag / arg | Description |
|------------|-------------|
| `MODE` | `dark`, `light`, or `system` |
| `--config` | Config path (default `~/.codex/config.toml`) |
| `--port` | CDP port used when restarting Codex (default `9335`) |
| `--no-restart` | Only write config; skip Codex restart |

### `verify`

Check live Codex over CDP (default port **9335**). Requires Codex with remote debugging.

```bash
# Inject state: host classes, style tag, theme id/version, image vars
cdxtheme verify inject
cdxtheme verify inject -t output/mclaren-f1.cdxtheme

# Layout contracts: Chat fixed composer, Work util+composer stack, ribbon, hero
cdxtheme verify layout
cdxtheme verify layout --contexts chat,work --json-out /tmp/layout.json
```

| Subcommand | Exit 0 | Exit 1 |
| --- | --- | --- |
| `inject` | all targets `pass: true` | any fail |
| `layout` | no layout issues | one or more issues |

### `probe`

Quick DOM snapshot or custom JS:

```bash
cdxtheme probe
cdxtheme probe --tab work
cdxtheme probe --expr 'getComputedStyle(document.querySelector(".composer-surface-chrome"),"::before").content'
```

### `screenshot`

```bash
cdxtheme screenshot -o /tmp/chat.jpg --tab chat
cdxtheme screenshot -o /tmp/work.jpg --tab work --quality 80
```

### Authoring loop

```bash
cdxtheme theme pack src/mclaren-f1 -o output/mclaren-f1.cdxtheme --force
cdxtheme apply -t output/mclaren-f1.cdxtheme
cdxtheme verify layout
```

## Package JSON

```json
{
  "format": "cdxtheme",
  "schemaVersion": 1,
  "exportedAt": "…",
  "theme": { "id": "…", "displayName": "…", "version": "…", "copy": { } },
  "targets": {
    "codex": {
      "css": "/* inlined */",
      "options": { "rendererProfile": "codex-theme-v1", "baseTheme": { } }
    }
  },
  "assets": {
    "images": {
      "hero": { "filename": "art.png", "mimeType": "image/png", "base64": "…" }
    }
  }
}
```

Max size **30 MB**. Max images **32**. No remote CSS `@import` / `url(http…)`.
