# WorkBuddy target

Use app id `workbuddy`. Scope CSS under `html.cdxtheme-host-workbuddy` / `:root.cdxtheme-host-workbuddy`.

## Generate notes

- Start from skill `assets/theme-starter/workbuddy/` (copy into `theme-dir/workbuddy/`) and the `targets.workbuddy` entry in `theme.json`.
- Prefer live semantic classes over stale template selectors.
- Stable cross-route landmarks often include:
  - root / shell: `.teams-container`, `#root`, `body.light|dark`
  - sidebar: `.conversation-sidebar`, `.conversation-list`, `.user-menu-popover`
  - workspace: `.teams-content-wrapper`, `.teams-main-content`, `.main-content`, `.chat-container`, `.wb-cb-chat`
  - topbar: `.workbuddy-topbar` / `.workbuddy-topbar--mac`
  - home: `.wb-home-page`, `.wb-home-header`, `.wb-scene-tabs`, `.wb-home-composer`
  - composer: `.wb-home-composer__input-slot`, host `[class*="_container_"]`, `[class*="_mainArea_"]`, `.wb-input-footer`, `[role="textbox"][contenteditable="true"]`
- Reference shared artwork with `var(--cdxtheme-image-hero)` and `var(--cdxtheme-image-texture)`.

## Starter layout contracts

Source of truth: `assets/theme-starter/workbuddy/`. Recolor tokens; keep these contracts unless fixing a proven bug.

Production reference polish (Red Bull WorkBuddy, 2026-08): dual light/dark, dark hero glass, host mainArea rebind, user-menu popover.

| Contract | Detail |
| --- | --- |
| **Dual light/dark** | Host theme classes live on **`body` and/or `html`**. Palette must use `html:has(body.light\|dark)`, `html.light\|dark`, `cb-*`, `vscode-*`, `electron-*`, `data-color-scheme`. Prefer `!important` on primary tokens. |
| Shell fill | `html` / `body` / `#root` / `.teams-container` height 100%; opaque `background-color` + wash |
| Dark shell field | Dark wash from `bg-elevated` → `bg` → panel mix (pairs with dark hero) |
| Topbar | `.workbuddy-topbar` height = `--theme-workbuddy-topbar` (default **24px**) |
| Main pane | `.teams-content-wrapper` full-bleed; `.chat-container` **horizontally centered** (`max-width: --theme-home-max`, margin auto) |
| Home stack | hero top · free mid · composer bottom (`margin-top: auto` on composer) |
| **Dark hero** | **No milky white scrim** (`white 97%`). Use **bg / bg-elevated / panel glass** over hero image + soft accent glow so banner matches page field |
| Light hero | Soft white scrim OK for ink on photo |
| Sizes (tokens) | header ~228 · tabs ~52 · composer = chips 36 + gap ≥24 + slot ~200 |
| Scene tabs | **flush left**, vertically centered in row |
| Chips gap | chips → input-slot **≥ 24px** (`--theme-composer-chips-gap`) |
| Host input shell | target `_container_*` **and** `section`; height 100% of slot |
| **Host mainArea** | Host often paints **white** on `[class*="_mainArea_"]` even under dark skin. **Always** rebind to `color-mix(bg, panel)` + `color: var(--theme-text)` |
| Menus | `overflow: visible` on composer container so `_menu_*` are not clipped; menu bg = `--theme-panel` |
| Footer | Opaque strip (not translucent); min-height ~44px for Select workspace |
| **User menu popover** | `.user-menu-popover` / `--portal` under **`body`**. Style with high specificity + `--theme-panel` / `--theme-text` (host fixed gray/white otherwise) |
| Polaroid | **hidden** on all WorkBuddy pages; corner badge (`--theme-badge`) only |
| Brand title | inject does **not** set `--dream-brand-title` — define fallback in `00-tokens.css` |

### Token knobs (`workbuddy/00-tokens.css`)

| Token | Default | Role |
| --- | --- | --- |
| `--theme-composer-chips-gap` | `24px` | Min space chips → input-slot |
| `--theme-composer-height` | `260px` | Full home composer (chips + gap + slot) |
| `--theme-input-slot-height` | `200px` | Input card + footer |
| `--theme-header-height` | `228px` | Hero billboard |
| `--theme-tabs-height` | `52px` | Scene tabs row |
| `--theme-home-max` | `900px` | Centered `.chat-container` max width |
| `--theme-workbuddy-topbar` | `24px` | Host topbar height |
| `--theme-home-gap` / `--theme-home-pad-bottom` | `16px` / `14px` | Home column spacing |
| `--dream-brand-title` / `--theme-eyebrow` / `--theme-badge` | starter EN | Header + badge copy |

### Partial map

| File | Owns |
| --- | --- |
| `00-palette.css` | Light + dark primary `--theme-*` (body/html host classes) |
| `00-tokens.css` | Aliases, layout knobs, badge/title fallbacks |
| `01-shell.css` | Shell wash, sidebar, topbar, **user-menu-popover** |
| `02-home.css` | Centered chat-container, hero (light + dark glass), tabs left |
| `03-composer.css` | Composer stack, **mainArea rebind**, footer, menus, polaroid hide |

## Dark-mode traps (from production)

| Symptom | Cause | Fix |
| --- | --- | --- |
| Light text on white input | Host `_mainArea_*` stays white | Force `background-color: color-mix(var(--theme-bg), var(--theme-panel))` + theme text |
| Hero looks milky / clashes with bg | Light-only white 97% scrim in dark | Dark hero: bg/elevated glass, not white wash |
| Theme switch doesn’t recolor | Tokens only on bare `:root` / ignore `body.light` | Palette via `html:has(body.light\|dark)` + html classes |
| User menu gray/white blob | Portal under `body`; host fixed bg | High-specificity `.user-menu-popover--portal` → `--theme-panel` |
| Footer strip wrong | Translucent mid on host surface | Opaque footer using panel/bg mix |
| Select workspace clipped | `overflow:hidden` on `_container_*` | `overflow: visible` + slot height ≥ footer |

## Home vs conversation

When styling both contexts:

1. Home header/hero, scene tabs (left), quick actions, and home composer.
2. Conversation with long text, tables or code, scrolling, and the conversation composer shell.
3. Sidebar selection, hover states, **user menu**, menus, and send controls remain usable.
4. No horizontal overflow; do not clip native menus with `overflow: hidden` on the input shell.

## Branding

- Scope only under `html.cdxtheme-host-workbuddy` / `:root.cdxtheme-host-workbuddy`.
- Prefer `cdxtheme-*` variables (`--cdxtheme-image-hero`).
- Keep home and conversation composers usable; avoid transform animations on wrappers that own the input shell.
- Theme-owned strings default **English** (`theme.json` `copy` + token fallbacks).
- Recolor: edit **only** `00-palette.css` (and mirror codex `00-palette.css`); keep layout contracts in other partials.

## Pack / apply

Include `targets.workbuddy` only when shipping WorkBuddy CSS:

```json
"workbuddy": {
  "css": "workbuddy.css"
}
```

```bash
# pack
/path/to/cdxthemex theme pack theme-dir -o output/<theme-id>.cdxtheme --force

# apply to WorkBuddy (default CDP 9336)
/path/to/cdxthemex apply --app workbuddy -t ./output/<theme-id>.cdxtheme

# optional inject check / restore
/path/to/cdxthemex verify inject -t ./output/<theme-id>.cdxtheme --port 9336
/path/to/cdxthemex restore --app workbuddy
```

Runtime inject is CSS-only (`enableChrome: false`). Sets `--cdxtheme-image-*`, `--dream-tagline`, `--dream-project-prefix`, `--dream-project-label`. Does **not** set `--dream-brand-title` — define a fallback in theme CSS/tokens if the header uses it.

**CDP note:** WorkBuddy uses `file://` targets on port **9336**. Built-in `probe` / `screenshot` default to Codex `app://` on 9335 — use apply + raw CDP websocket evaluate for WB layout measures when needed.

After dual-mode changes, verify **both** Light and Dark via user-menu theme switcher: hero, composer `_mainArea_`, footer, user-menu-popover.

See [cli.md](cli.md) for hosts, ports, and full command reference.
