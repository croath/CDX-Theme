# cdxtheme CLI

Thin CLI over shared library **`cdx-theme-core`**.

Pack, unpack, apply, and **verify** multi-app portable theme packages (`.cdxtheme`) over CDP.

| Brand | `format` field | Extension |
|-------|----------------|-----------|
| **CDXTheme** (default) | `cdxtheme` | `.cdxtheme` |
| **CodeDrobe** | `codedrobe-theme` | `.codedrobe-theme` |

Same JSON schema (cloned from [CodeDrobe package.mjs](https://github.com/CodeDrobe/core/blob/main/src/theme/package.mjs)).  
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
cdxtheme apply --app codex --theme <PACKAGE> [OPTIONS]
cdxtheme verify inject [-t PACKAGE]
cdxtheme verify layout [--contexts chat,work]
cdxtheme probe [--tab chat|work] [--expr JS]
cdxtheme screenshot -o OUT.jpg [--tab chat|work]
```

### `theme pack`

```bash
cdxtheme theme pack themes/ferrari
# → ferrari-1.0.0.cdxtheme

cdxtheme theme pack themes/ferrari --format codedrobe-theme --pretty --force
# → ferrari-1.0.0.codedrobe-theme

cdxtheme theme pack themes/ferrari/theme.json -o dist/ferrari.cdxtheme
cdxtheme theme pack themes/ferrari/manifest.json -o dist/ferrari.cdxtheme
```

| Flag | Description |
|------|-------------|
| `-o`, `--output` | Output path (default `{id}-{version}.cdxtheme`) |
| `--format` | `cdxtheme` (default) or `codedrobe-theme` |
| `--pretty` | Pretty-print JSON |
| `--force` | Overwrite existing file |

**CSS brand rewrite (automatic):** when packing, every `codedrobe-` token in target CSS is rewritten to `cdxtheme-` (class names, ids, custom properties). Source CSS may still use CodeDrobe tokens; the package always gets CDXTheme-branded CSS.

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

Accepts both `.cdxtheme` and `.codedrobe-theme`.

### `theme convert`

Convert a **CodeDrobe** package to **CDXTheme** (`.cdxtheme`):

1. Sets `format` to `cdxtheme`
2. Rewrites every `codedrobe-` token in each target CSS to `cdxtheme-`
   (e.g. `.codedrobe-codex-skin` → `.cdxtheme-codex-skin`,
   `--codedrobe-image-hero` → `--cdxtheme-image-hero`)

```bash
cdxtheme theme convert ferrari-1.0.0.codedrobe-theme
# → ferrari-1.0.0.cdxtheme

cdxtheme theme convert ferrari.codedrobe-theme -o dist/ferrari.cdxtheme --pretty --force
```

| Flag | Description |
|------|-------------|
| `-o`, `--output` | Output path (default `{id}-{version}.cdxtheme`) |
| `--pretty` | Pretty-print JSON |
| `--force` | Overwrite existing file |

Also accepts an existing `.cdxtheme` (re-applies CSS brand rewrite and refreshes `exportedAt`).

### `apply`

Inject a theme package into a live host app over CDP.

1. Probe CDP on the remote-debugging port (default **9335**)
2. If not connected, launch (or restart) ChatGPT/Codex with `--remote-debugging-port`
3. Inject the package CSS/skin into all `app://` page targets

```bash
cdxtheme apply --app codex --theme ferrari-1.0.0.cdxtheme
cdxtheme apply -t themes/doll-sister.cdxtheme --port 9335
```

| Flag | Description |
|------|-------------|
| `--app` | Host app id (currently only `codex`) |
| `-t`, `--theme` | Path to `.cdxtheme` / `.codedrobe-theme` |
| `--port` | CDP port (default `9335`) |
| `--timeout-ms` | Wait / inject timeout (default `120000`) |


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
