# Codex layout probe scripts

Reusable CDP expressions for `/path/to/cdxthemex probe --expr`. Used when polishing Chat/Work home layout (hero top, util+composer stack, composer `::before`, etc.) and when debugging **light-skin ink / surface contrast** (sidebar white text, Settings dark cards).

Codex must be running with remote debugging (default port **9335**).

## Quick use

Skill docs use bare `/path/to/run.sh <name>` (wrapper path is in repo `AGENTS.md`). From the themes repo root:

```bash
# wrapper (recommended) — resolve /path/to/run.sh from AGENTS.md
/path/to/run.sh work-layout
/path/to/run.sh go-work-home
/path/to/run.sh work-layout   # after nav settles
/path/to/run.sh chat-layout
/path/to/run.sh set-mode chat
/path/to/run.sh set-mode work
/path/to/run.sh discover-nav
/path/to/run.sh hero-row-tree

# light-skin contrast (Manchester City polish, etc.)
/path/to/run.sh token-bridge
/path/to/run.sh sidebar-ink
/path/to/run.sh contrast-scan
# open Settings in Codex first:
/path/to/run.sh settings-contrast

# or raw CLI
/path/to/cdxthemex probe --expr "$(cat work-layout.js)"  # probe dir; prefer /path/to/run.sh
```

Optional wrapper flags (passed through to `/path/to/cdxthemex probe`):

```bash
/path/to/run.sh work-layout --wait-ms 1200
/path/to/run.sh chat-layout --port 9335
```

## Scripts

### Layout / navigation

| File | Purpose |
| --- | --- |
| `work-layout.js` | Work home geometry: hero, missions, util/composer stack, `::before`, gaps |
| `chat-layout.js` | Chat home geometry: hero, shell, fixed composer, `::before` |
| `go-work-home.js` | Click “新建任务” / “New task” (etc.) to open Work home |
| `set-mode.js` | Switch header Composer mode — needs `MODE` (`chat` \| `work`) via wrapper |
| `discover-nav.js` | Dump mode group, header, useful buttons (locale-agnostic debug) |
| `hero-row-tree.js` | Why is hero mid-page? Parent grid rows + child boxes |

### Light-skin ink / surfaces (contrast)

| File | Purpose | When to run |
| --- | --- | --- |
| `token-bridge.js` | Dump host `--vscode-*` / `--color-token-*` / `--color-background-*` + suspect white ink / dark panels | After recolor or pack/apply; first stop for light skins |
| `sidebar-ink.js` | Left rail (`.app-shell-left-panel` aside **and** Settings `div`): whiteish labels, `.sidebar-foreground-muted` remap | “左侧文字全白 / 看不见” |
| `contrast-scan.js` | Full viewport: whiteish direct text + low-luma solid backgrounds | Broad “看不清” / mixed light+dark leftovers |
| `settings-contrast.js` | Settings rail + `.rounded-2xl` cards + `--color-background-panel` | Settings section cards dark while text is dark |

#### Light-skin debug loop

```bash
/path/to/cdxthemex theme pack theme-dir -o output/<id>.cdxtheme --force
/path/to/cdxthemex apply -t ./output/<id>.cdxtheme

/path/to/run.sh token-bridge
# expect: ok true, suspectWhiteInk / suspectDarkSurfaces empty

/path/to/run.sh sidebar-ink
# expect: whiteishCount 0 on every rail

/path/to/run.sh contrast-scan
# expect: whiteInkCount 0; darkSurfaceCount 0 (or only intentional dark chrome)

# manually open Settings, then:
/path/to/run.sh settings-contrast
# expect: darkCardCount 0; tokens.panel light; cards bg near white
```

#### Host traps these scripts catch

| Symptom | Host mechanism | Fix location (theme) |
| --- | --- | --- |
| Sidebar labels white on light rail | `.sidebar-foreground-muted { --color-token-foreground: color-mix(…, var(--vscode-foreground)) }` while `--vscode-foreground` stays white | `00-tokens.css` + `.app-shell-left-panel` ink overrides |
| Settings left nav unreadable | Settings uses **`div.app-shell-left-panel`** (not `aside`) with dark glass bg | Selectors on `.app-shell-left-panel` (not only `aside`) |
| Settings section dark + dark text | Inline `background-color: var(--color-background-panel, …)` defaults to `#373a45` | Bridge `--color-background-panel` / elevated surfaces to light |

## Typical Work polish loop

```bash
/path/to/cdxthemex theme pack theme-dir -o output/<id>.cdxtheme --force
/path/to/cdxthemex apply -t ./output/<id>.cdxtheme
/path/to/run.sh go-work-home
sleep 1
/path/to/run.sh work-layout
```

### Healthy Work snapshot (approx.)

| Field | Healthy |
| --- | --- |
| `gaps.heroFromHeader` | small (~6–12px), not ~60+ |
| `heroRow.alignItems` | `flex-start` |
| `shell.display` | `flex` (not equal `grid` rows) |
| `composerBefore.display` | `none` on Work if theme hides badge |
| `gaps.orderOK` | `true` (util above composer) |
| `gaps.missionsToUtil` | **> 0** (no overlap) |

## Notes

- Expressions must be a single JS **expression** (IIFE). No `import` / top-level `await`.
- Chinese UI: “新建任务”, `aria-label="选择项目"` / `"插件"` are covered where relevant.
- `--tab work` on the CLI only clicks the Chat/Work **mode** toggle; it does **not** open Work **home** if you are inside a task detail — use `go-work-home.js`.
- `theme.json` **`version` is an integer** (e.g. `4`), not a semver string.
