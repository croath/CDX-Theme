# CSS templates

## Neutral starter

**Required path for new themes:** copy from `assets/theme-starter/` only. default generation is always the starter.

Copy the complete multi-app starter from `assets/theme-starter/` into a new writable project directory. It contains:

- `theme.json`: `codex` and `workbuddy` targets with home / work-home / conversation verification contexts.
- `codex/*.css` partials (**source of truth** — edit these; merge → `codex.css`):
  | File | Contents |
  | --- | --- |
  | `00-tokens.css` | **Only place to recolor** (`--theme-*`) |
  | `01-shell.css` | body / texture / stripe / sidebar / main |
  | `02-chrome.css` | brand, sparkles, polaroid, ribbon |
  | `03-home.css` | home flex, Chat/Work hero, suggestions |
  | `04-composer.css` | home fixed / detail relative |
  | `05-work.css` | missions, util stack, chips |
  | `06-settings.css` | hide dream chrome on settings |
- `workbuddy/*.css` partials → merge → `workbuddy.css` (`00-tokens`, `01-shell`, `02-home`, `03-composer`). **Mirror** codex primary tokens.
- Generated `codex.css` / `workbuddy.css`: do not hand-edit; produced by `/path/to/cdxthemex theme merge-css` (or auto on `theme pack`).

Body rules already consume `var(--theme-*)` and `color-mix(..., var(--theme-*), ...)`. Full partial map: [SKILL.md](../SKILL.md) → **CSS partials**.

### Token recolor (copy into new theme `:root`)

Edit these in **both** `codex/00-tokens.css` and `workbuddy/00-tokens.css`:

```css
:root.cdxtheme-host-codex,
:root.cdxtheme-codex-skin {
  color-scheme: dark !important; /* or light */

  --theme-accent: #00d2c4;
  --theme-accent-hot: #00ffe6;
  --theme-accent-deep: #008f86;
  --theme-accent-2: #a6aeb5;
  --theme-accent-2-hot: #d2d8dd;
  --theme-accent-3: #e10600;
  --theme-text: #ffffff;
  --theme-muted: #75818c;
  --theme-bg: #0b0c0e;
  --theme-bg-elevated: #121417;
  --theme-panel: #1c1e23;
  --theme-panel-mid: #16181c;
  --theme-on-accent: #0b0c0e;
  --theme-white: #ffffff;
  --theme-black: #000000;

  /* semantic aliases can stay as in starter (color-mix from primaries) */
  --theme-composer-bottom: 24px;
  --theme-chat-composer-height: 60px;

  --theme-eyebrow: "BRAND";
  --theme-eyebrow-work: "BRAND · WORK";
  --theme-panel-label: "MISSIONS";
  --theme-badge: "✦";
}
```

Do **not** paste raw brand hex into polaroid/composer/chip rules — those already reference the tokens.

| Do | Don’t |
| --- | --- |
| Change `--theme-accent` once | Search-replace `#00d2c4` through the file |
| Sync workbuddy `:root` palette with codex | Leave codex teal / workbuddy purple |
| Keep layout numbers (142px polaroid, 240px hero, fixed stack) | Fork sizes from an old livery |
| Author copy in `theme.json` + eyebrow tokens | Hardcode brand strings only mid-file |

Add named images to the copied manifest when needed:

```json
"images": {
  "hero": "assets/hero.png",
  "texture": "assets/texture.png"
}
```

The CSS uses optional `--cdxtheme-image-hero` and `--cdxtheme-image-texture` variables. Missing optional images fall back to gradients or no texture.

## Codex inject contract (runtime)

CDXTheme injects a decorative overlay into Codex. Theme CSS **must** match this contract:

| Runtime | CSS should use |
|---------|----------------|
| `html.cdxtheme-codex-skin` + `html.cdxtheme-host-codex` | Scope under **both** classes |
| `#cdxtheme-codex-skin-chrome` | Only this chrome id (not `cdxtheme-host-codex-chrome`) |
| `.dream-home-shell` on chrome when home | Show brand / polaroid / ribbon under `#cdxtheme-codex-skin-chrome.dream-home-shell …` |
| `--cdxtheme-image-hero` / `--cdxtheme-image-texture` | `var(--cdxtheme-image-hero)` etc. |
| `--cdxtheme-composer-left` / `--cdxtheme-composer-width` | Home fixed composer geometry |

Chrome layout tips that keep polaroid visible:

```css
#cdxtheme-codex-skin-chrome {
  position: fixed;
  z-index: 40;
  pointer-events: none;
  overflow: visible; /* not hidden — avoids clipping rotated polaroid */
}

.dream-polaroid {
  display: none;
  background-image: var(--cdxtheme-image-hero);
  /* …size / border / transform… */
}

#cdxtheme-codex-skin-chrome.dream-home-shell .dream-polaroid {
  display: block !important; /* beat narrow media queries */
}
```

Do **not** use:

```css
@media (max-width: 1120px) {
  .dream-polaroid { display: none !important; } /* hides polaroid on common window sizes */
}
```

Prefer:

```css
@media (max-width: 900px) {
  #cdxtheme-codex-skin-chrome:not(.dream-home-shell) .dream-polaroid {
    display: none !important;
  }
}
```

## Home column stack (kill native overlap)

Native home uses `grid.grid-rows-2` + second child `-mt-16` (pulls composer column over hero). Current shell is often `min-h-0 w-full flex-1 pt-6 grid grid-rows-2` (legacy: `min-h-full w-full pt-6`). Match **both** or use `.grid.grid-rows-2` so the hero is not mid-grid.

```css
html.cdxtheme-host-codex [class*="home-main-content"] .min-h-full.w-full.pt-6,
html.cdxtheme-host-codex [class*="home-main-content"] .min-h-0.w-full.flex-1.pt-6,
html.cdxtheme-host-codex [class*="home-main-content"] .grid.grid-rows-2,
html.cdxtheme-codex-skin [class*="home-main-content"] .min-h-full.w-full.pt-6,
html.cdxtheme-codex-skin [class*="home-main-content"] .min-h-0.w-full.flex-1.pt-6,
html.cdxtheme-codex-skin [class*="home-main-content"] .grid.grid-rows-2,
.dream-home .min-h-full.w-full.pt-6,
.dream-home .min-h-0.w-full.flex-1.pt-6,
.dream-home .grid.grid-rows-2 {
  display: flex !important;
  flex-direction: column !important;
  grid-template-rows: none !important;
  gap: 12px !important;
  height: auto !important;
  min-height: 0 !important;
  flex: 0 0 auto !important;
  padding-top: 8px !important;
  padding-bottom: calc(var(--theme-composer-bottom, 24px) + 140px) !important;
  box-sizing: border-box !important;
}

/* Hero banner at top of column */
.dream-home .flex.grow.basis-0.items-end.justify-center {
  align-items: flex-start !important;
  justify-content: flex-start !important;
  flex: 0 0 auto !important;
  height: auto !important;
  padding-bottom: 12px !important;
  margin: 0 !important;
}

.dream-home .flex.min-w-0.shrink-0.grow.basis-0.flex-col {
  margin-top: 0 !important; /* kill -mt-16 */
  flex: 0 0 auto !important;
  height: auto !important;
}
```

## Composer templates

### Home: fixed pin (Chat / Work landing)

```css
html.cdxtheme-host-codex:has(.dream-home) .composer-surface-chrome,
html.cdxtheme-codex-skin:has(.dream-home) .composer-surface-chrome {
  position: fixed !important;
  left: var(--cdxtheme-composer-left, 24px) !important;
  width: var(--cdxtheme-composer-width, calc(100vw - 48px)) !important;
  bottom: var(--theme-composer-bottom, 24px) !important;
  top: auto !important;
  z-index: 50 !important;
  transform: none !important;
  filter: none !important;
  animation: none !important;
}
```

### Conversation detail: relative in sticky footer

```css
html.cdxtheme-host-codex .thread-scroll-container,
html.cdxtheme-host-codex .\[content-visibility\:auto\] {
  content-visibility: visible !important;
}

html.cdxtheme-host-codex:not(:has(.dream-home)):not(:has([data-feature="game-source"])):not(:has(h1.heading-xl)) .composer-surface-chrome {
  position: relative !important;
  left: auto !important;
  width: 100% !important;
  bottom: auto !important;
  max-width: 100% !important;
}
```

### Never trap fixed positioning

Do **not** leave `transform` or `filter` (including animated end-state `translateY(0)`) on:

- `.dream-home`
- `.app-shell-main-content-frame`
- `[class*="home-main-content"]`
- any ancestor that also contains `.composer-surface-chrome`

Shell enter animations should be **opacity-only**.

### Work: fixed util + composer stack

When composer is fixed alone, Work’s util strip (`absolute.inset-x-0.top-full` under a zero-height wrapper) lands **mid-page over missions**. Pin the wrapper as one unit; flex-order util above composer:

```css
html.cdxtheme-host-codex:has(.dream-home)
  .relative.flex.flex-col.z-10.gap-2.mb-10:has([data-composer-navigation-target="workspace-project"]) {
  position: fixed !important;
  left: var(--cdxtheme-composer-left, 24px) !important;
  width: var(--cdxtheme-composer-width, calc(100vw - 48px)) !important;
  bottom: var(--theme-composer-bottom, 24px) !important;
  z-index: 50 !important;
  display: flex !important;
  flex-direction: column !important;
  gap: 0 !important;
  height: auto !important;
  margin: 0 !important;
  transform: none !important;
  filter: none !important;
  pointer-events: none !important;
}

/* Util host often AFTER composer in DOM — force first visual */
… > .absolute.inset-x-0.top-full {
  position: relative !important;
  inset: auto !important;
  width: 100% !important;
  order: 1 !important;
  pointer-events: auto !important;
}

… > .relative:has(.composer-surface-chrome) {
  order: 2 !important;
  width: 100% !important;
  pointer-events: auto !important;
}

… .composer-surface-chrome {
  position: relative !important;
  left: auto !important;
  width: 100% !important;
  bottom: auto !important;
  border-top-left-radius: 0 !important;
  border-top-right-radius: 0 !important;
}

/* Util strip joins shell */
div.select-none.flex.flex-nowrap.items-center:has([data-composer-navigation-target="workspace-project"]) {
  width: 100% !important;
  border-radius: 14px 14px 0 0 !important;
  border-bottom: 0 !important;
}
```

Also hide Work’s empty Chat-era absolute rail, pin a shorter Work hero at top, hide optional composer `::before` corner badge on Work, and scope Chat-only `nth-child(2)` rules:

```css
.dream-home div.mx-auto:has([data-feature="game-source"]) > .absolute {
  display: none !important;
}

/* Optional: Work-only shorter hero + hide corner badge if theme uses ::before */
.dream-home div.mx-auto:has([data-feature="game-source"]) {
  height: 160px !important;
  min-height: 160px !important;
}
html.cdxtheme-host-codex:has([data-feature="game-source"]) .composer-surface-chrome::before,
html.cdxtheme-host-codex .composer-surface-chrome:has([data-composer-navigation-target="permissions"])::before {
  content: none !important;
  display: none !important;
}
```

### Utility label chips

Style the **button**, not the first child as a fake icon. Prefer `data-composer-navigation-target` (works in Chinese UI):

```css
/* Good */
button[data-composer-navigation-target="plugins"] {
  padding: 0 12px;
  /* border, gradient, spine… */
}

button[data-composer-navigation-target="plugins"] > span {
  display: inline-flex;
  background: none;
  width: auto;
  height: auto;
}

/* Bad — crushes “Plugins” / “Auto Auto” into a 20px square */
button[aria-label="Plugins"] > span:first-child {
  width: 20px;
  height: 20px;
  display: grid;
  background: linear-gradient(…);
}
```

Also match localized host `aria-label` values when needed (prefer `data-composer-navigation-target` over language-specific labels).

Scope Chat vs Work model pickers separately:

```css
button[aria-label="Select ChatGPT model"] { /* Chat */ }
button[data-composer-navigation-target="reasoning"] { /* Work */ }
button[data-codex-intelligence-trigger="true"]:not([data-composer-navigation-target="reasoning"]) { /* Chat-only shared attr */ }
```

Work model measurement ghost:

```css
button[data-composer-navigation-target="reasoning"] > span[class*="Measurement"] {
  position: absolute !important;
  width: 0 !important;
  height: 0 !important;
  overflow: hidden !important;
  visibility: hidden !important;
  display: block !important; /* not contents */
}
```

### Chat vs Work suggestions

```css
/* Chat cards — never apply min-height to Work rows */
.group\/home-suggestions button:not(.group\/home-suggestion-list-item) { min-height: 110px; … }

/* Work list rows */
.group\/home-suggestions button.group\/home-suggestion-list-item { min-height: 52px; … }
```

### Ribbon: Work clear / Chat half-cover

```css
/* Default / Work — clear above stack */
#cdxtheme-codex-skin-chrome .dream-ribbon {
  left: 50%;
  bottom: calc(var(--theme-composer-bottom, 24px) + 118px + 14px);
  transform: translateX(-50%);
  z-index: 45;
  pointer-events: none;
}

/* Chat — half of ribbon covers the input shell */
html.cdxtheme-host-codex:has(.dream-home):not(:has([data-feature="game-source"])):not(:has(.group\/home-suggestion-list-item))
  #cdxtheme-codex-skin-chrome .dream-ribbon {
  bottom: calc(
    var(--theme-composer-bottom, 24px)
    + var(--theme-chat-composer-height, 60px)
    - 20px
  ); /* ≈ 64px when shell ≈ 60 and half-ribbon ≈ 20 */
  z-index: 55; /* above composer 50 */
  transform: translateX(-50%);
}

@keyframes ribbon-float {
  0%, 100% { transform: translateX(-50%) translateY(0); }
  50% { transform: translateX(-50%) translateY(-6px); }
}
```

## CSS order

Matches starter partial prefixes (alphabetical merge):

1. `00-tokens` — host root  
2. `01-shell` — body + sidebar + main  
3. `02-chrome` — `#cdxtheme-codex-skin-chrome` overlays  
4. `03-home` — column stack, Chat cards vs Work rows  
5. `04-composer` — home fixed + detail relative  
6. `05-work` — Work stack + chips + mode toggle  
7. `06-settings` — settings hide chrome  

Prefer live semantic selectors; recolor via `--theme-*`. Full narrative: [codex.md](codex.md).
