# Codex target notes

Use app id `codex`. Scope CSS under **both**:

- `html.cdxtheme-host-codex` / `:root.cdxtheme-host-codex`
- `html.cdxtheme-codex-skin` / `:root.cdxtheme-codex-skin`

(Runtime inject adds both classes.)

Use **`cdxtheme-*` only** in package CSS.

## Surfaces (map before styling)

| Surface | How to detect | Notes |
| --- | --- | --- |
| **Chat home** | `.dream-home`, often `h1.heading-xl` welcome | Hero banner, optional grid suggestions |
| **Work home** | `.dream-home` + `[data-feature="game-source"]` and/or `button.group/home-suggestion-list-item` | Mission list rows, utility bar (project / plugins / run-location) |
| **Conversation detail** | No `.dream-home`; thread + sticky composer | **Different composer positioning** (see below) |
| **Chrome overlay** | `#cdxtheme-codex-skin-chrome` | Brand, signature, sparkles, ribbon, polaroid |

Chat/Work header toggle:

```text
[role="group"][aria-label="Composer mode"]
  button[aria-pressed="true|false"]   /* Chat | Work */
  span.relative                       /* sliding active pill */
```

## Injected chrome overlay

```html
<div id="cdxtheme-codex-skin-chrome" class="dream-home-shell?" aria-hidden="true">
  <div class="dream-brand">…</div>
  <div class="dream-signature"></div>
  <div class="dream-sparkles">…</div>
  <div class="dream-ribbon">…</div>
  <div class="dream-polaroid"></div>
</div>
```

Theme CSS should:

- Target **only** `#cdxtheme-codex-skin-chrome`
- `overflow: visible` so polaroid is not clipped
- Prefer `z-index: 40` (ribbon can be slightly higher, e.g. 45) and `pointer-events: none`
- Show decorations under `.dream-home-shell` with `display: … !important` when needed

Images:

```css
background-image: var(--cdxtheme-image-hero);
background-image: var(--cdxtheme-image-texture);
```

### Ribbon above / half-over composer

Recommended CSS vars (theme root or host):

```css
--theme-composer-bottom: 24px;
--theme-chat-composer-height: 60px; /* single-row Chat shell ≈ 60px */
```

**Work / tall shells** — sit clear above the bottom stack:

```css
#cdxtheme-codex-skin-chrome .dream-ribbon {
  position: absolute !important;
  left: 50% !important;
  bottom: calc(var(--theme-composer-bottom, 24px) + 118px + 14px) !important;
  transform: translateX(-50%) !important;
  z-index: 45 !important;
  pointer-events: none;
}
```

**Chat tab (half-cover the input)** — park ribbon on the composer top edge so ~half of it paints over the shell:

```css
/* Chat only: no Work game-source / mission list */
html.cdxtheme-host-codex:has(.dream-home):not(:has([data-feature="game-source"])):not(:has(.group\/home-suggestion-list-item))
  #cdxtheme-codex-skin-chrome .dream-ribbon {
  /* bottom = composer-bottom + chat-composer-height − half-ribbon (~20px when ribbon ≈ 40px) */
  bottom: calc(
    var(--theme-composer-bottom, 24px)
    + var(--theme-chat-composer-height, 60px)
    - 20px
  ) !important; /* ≈ 64px → 50% overlap on a 40px ribbon */
  z-index: 55 !important; /* above composer (50) so it covers the shell */
  transform: translateX(-50%) !important;
}
```

Rules:

- Any float animation **must keep** `translateX(-50%)` (e.g. `translateX(-50%) translateY(-6px)`). A float that sets only `rotate`/`translateY` un-centers the ribbon.
- Verify with CDP: `overlapPx ≈ ribbon.height / 2`, ribbon `z-index` > composer.
- Do not use a global narrow media query that resets `bottom` without the Chat formula.

## Home page column stack (critical)

Native home body is often:

```text
/* current */
div.min-h-0.w-full.flex-1.pt-6.grid.grid-rows-2
/* legacy */
div.min-h-full.w-full.pt-6.grid.grid-rows-2
  ├─ div.flex.grow…items-end…          /* hero / title row */
  └─ div.flex…flex-col.-mt-16…         /* suggestions + composer column (−64px pull-up) */
```

`grid-rows-2` + child `-mt-16` **intentionally overlaps** hero and the second column. If theme CSS only targets the **legacy** `min-h-full` shell, the new `min-h-0 flex-1` grid still splits the page into two equal rows and parks the hero mid-page (`align-items: center` / `items-end`). Match **both** shells (or use `.grid.grid-rows-2` under home-main-content).

```css
/* Force vertical stack — no hero/composer overlap (legacy + current shell) */
html.cdxtheme-host-codex [class*="home-main-content"] .min-h-full.w-full.pt-6,
html.cdxtheme-host-codex [class*="home-main-content"] .min-h-0.w-full.flex-1.pt-6,
html.cdxtheme-host-codex [class*="home-main-content"] .grid.grid-rows-2,
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
  box-sizing: border-box !important;
}

/* Hero row: content height at TOP, not grow / center into mid-page */
.dream-home .flex.grow.basis-0.items-end.justify-center {
  align-items: flex-start !important;
  justify-content: flex-start !important;
  flex: 0 0 auto !important;
  height: auto !important;
  padding-bottom: 12px !important; /* override native pb-24 */
  margin: 0 !important;
}

/* Kill native -mt-16 on the suggestions/composer column */
.dream-home .flex.min-w-0.shrink-0.grow.basis-0.flex-col {
  margin-top: 0 !important;
  flex: 0 0 auto !important;
  height: auto !important;
}
```

Room under home content for the fixed bottom shell:

```css
.dream-home .min-h-full.w-full.pt-6,
.dream-home .min-h-0.w-full.flex-1.pt-6,
.dream-home .grid.grid-rows-2 {
  padding-bottom: calc(var(--theme-composer-bottom, 24px) + 140px) !important;
}
/* Work (taller util+composer stack) needs more: +200px */
```

## Composer (critical)

Runtime sets `--cdxtheme-composer-left` / `--cdxtheme-composer-width` from `main.main-surface`. Prefer also defining `--theme-composer-bottom` (default `24px`).

### Home (Chat landing + Work landing)

Pin to the **viewport** bottom (not mid-column):

```css
html.cdxtheme-host-codex:has(.dream-home) .composer-surface-chrome,
html.cdxtheme-codex-skin:has(.dream-home) .composer-surface-chrome {
  position: fixed !important;
  left: var(--cdxtheme-composer-left, 24px) !important;
  width: var(--cdxtheme-composer-width, calc(100vw - 48px)) !important;
  bottom: var(--theme-composer-bottom, 24px) !important;
  top: auto !important;
  right: auto !important;
  z-index: 50 !important;
  transform: none !important;
  filter: none !important;
  animation: none !important;
}
```

Clear **transform / filter / animation** on the composer itself **and** on shells that wrap it (`.dream-home`, `[class*="home-main-content"]`, `.app-shell-main-content-frame`). Leave them set and fixed positioning drifts.

## Work tab layout (pit-wall)

Detect Work: `.dream-home` + `[data-feature="game-source"]` and/or `button.group/home-suggestion-list-item`.

### Bottom stack: util strip + composer

When the input is `position: fixed`, the Work utility strip often lives in:

```text
div.relative.flex.flex-col.z-10.gap-2.mb-10   /* height collapses to 0 once composer is fixed */
  ├─ div.absolute.inset-x-0.top-full            /* util host — lands MID-PAGE over missions */
  │    └─ div.select-none…:has([data-composer-navigation-target="workspace-project"])
  └─ div.relative
       └─ .composer-surface-chrome
```

**Bug:** util at mid-page overlaps mission rows. **Fix:** pin the whole wrapper as one fixed unit; put util **above** composer (DOM order is often reversed — use flex `order`):

```css
/* Fixed unit at viewport bottom */
html.cdxtheme-host-codex:has(.dream-home)
  .relative.flex.flex-col.z-10.gap-2.mb-10:has([data-composer-navigation-target="workspace-project"]) {
  position: fixed !important;
  left: var(--cdxtheme-composer-left, 24px) !important;
  width: var(--cdxtheme-composer-width, calc(100vw - 48px)) !important;
  bottom: var(--theme-composer-bottom, 24px) !important;
  top: auto !important;
  z-index: 50 !important;
  display: flex !important;
  flex-direction: column !important;
  gap: 0 !important;
  height: auto !important;
  margin: 0 !important;
  transform: none !important;
  filter: none !important;
  pointer-events: none !important; /* children re-enable */
}

/* Util host: in-flow first visual (was absolute top-full) */
… > .absolute.inset-x-0.top-full {
  position: relative !important;
  inset: auto !important;
  width: 100% !important;
  order: 1 !important;
  pointer-events: auto !important;
}

/* Composer wrapper: second visual */
… > .relative:has(.composer-surface-chrome) {
  order: 2 !important;
  width: 100% !important;
  pointer-events: auto !important;
}

/* Composer relative inside the fixed unit (not double-fixed) */
… .composer-surface-chrome {
  position: relative !important;
  left: auto !important;
  width: 100% !important;
  bottom: auto !important;
  border-top-left-radius: 0 !important;
  border-top-right-radius: 0 !important;
}
```

Style the util strip with top-only radius so it joins the composer shell (`border-radius: 14px 14px 0 0`, `border-bottom: 0`).

### Work hero / empty absolute rail

- Prefer `:has([data-feature="game-source"])` for the Work banner card (same pattern as Chat `h1.heading-xl`).
- **Pin hero at top** of the Work column (`align-items: flex-start`, compact row, shorter height than Chat — e.g. ~160–168px vs Chat ~220–240px). Do not leave `align-items: center` on the hero row for Work when the parent still allocates grid height.
- **Hide** the empty Chat-era absolute rail under Work hero:

```css
.dream-home div.mx-auto:has([data-feature="game-source"]) > .absolute {
  display: none !important;
}
```

- **Hide composer corner badge on Work** if the theme uses `.composer-surface-chrome::before` (clutters util + multi-tool stack):

```css
html.cdxtheme-host-codex:has([data-feature="game-source"]) .composer-surface-chrome::before,
html.cdxtheme-host-codex .composer-surface-chrome:has([data-composer-navigation-target="permissions"])::before {
  content: none !important;
  display: none !important;
}
```

- Chat-only absolute suggestion positioning — never apply on Work:

```css
.dream-home:not(:has([data-feature="game-source"]))
  > div:first-child > div:first-child > div:first-child > div:nth-child(2) { … }
```

- Work title copy is longer (“What should we work on?”) — use a **smaller** clamp than Chat welcome (`clamp(26px, 3.2vw, 40px)`), not Chat’s ~48–54px.
- Keep brand eyebrow/tagline on `::before` / `::after` of `[data-feature="game-source"]`; do **not** invent the Work question via CSS `content` (i18n is in the DOM).
- Theme-owned labels (`content`, `copy.*`) use the **target display language** (default **English**); see SKILL.md → Display language.

### Work chrome polish

| Element | Guidance |
| --- | --- |
| Polaroid | On Work, tuck top-right (smaller), not bottom over missions/composer |
| Mission list | Panel border/background; optional `::before` section label (target language, default EN); row min-height ~52–58px |
| Card min-height | Only `.group/home-suggestions button:not(.group/home-suggestion-list-item)` |
| i18n chips | Also match `aria-label="选择项目"`, `"插件"`, `"听写"` — match host locale; do not replace host strings |
| Theme copy language | `theme.json` `copy` + decorative `content: "…"` in user language; **default English** |
| Padding-bottom | Work stack (util ~50 + composer ~124) needs ~`composer-bottom + 200px` on home scroll body |

### Conversation detail

Thread shells often use Tailwind `[content-visibility:auto]`, which creates a **containing block for `position: fixed`**. Home offsets then apply *inside* the main column (~+sidebar width) and the input overflows off-screen.

**Do not** force fixed home geometry on detail. Prefer:

```css
/* Thread containers must not trap fixed descendants */
html.cdxtheme-host-codex .thread-scroll-container,
html.cdxtheme-host-codex .\[content-visibility\:auto\] {
  content-visibility: visible !important;
}

/* Detail: natural sticky column width */
html.cdxtheme-host-codex:not(:has(.dream-home)):not(:has([data-feature="game-source"])):not(:has(h1.heading-xl)) .composer-surface-chrome {
  position: relative !important;
  left: auto !important;
  width: 100% !important;
  bottom: auto !important;
  max-width: 100% !important;
}
```

### Home vs Work shell height

- **Work** often has multi-row tools (`data-composer-navigation-target="permissions"`, model, etc.) — taller min-height is OK.
- **Chat** home is often a **single row**. Forcing a tall `min-height` (e.g. 96px) creates an empty band that looks like a “line” above the input. Use a compact shell for Chat:

```css
.composer-surface-chrome:not(:has([data-composer-navigation-target="permissions"])) {
  min-height: 0 !important;
  padding: 8px 12px !important;
}
```

## Utility chips and model pickers

Stable attributes (prefer these over generated class hashes):

| Control | Selector |
| --- | --- |
| Choose project | `data-composer-navigation-target="workspace-project"`; also `aria-label="Choose project"` / `"选择项目"` |
| Plugins (utility bar) | `data-composer-navigation-target="plugins"`; also `aria-label="Plugins"` / `"插件"` |
| Run location | `data-composer-navigation-target="run-location"`; `aria-label="Choose where to run this chat"` |
| Work model | `data-composer-navigation-target="reasoning"` (+ often `data-codex-intelligence-trigger="true"`) |
| Chat model | `aria-label="Select ChatGPT model"` (may also have `data-codex-intelligence-trigger`) |
| Dictate / mic | `aria-label="Dictate"` / `"听写"` |
| Permissions | `data-composer-navigation-target="permissions"` |

Prefer **`data-composer-navigation-target`** over English-only `aria-label` so Chinese (and other) locales still match.

### Label chips are not icon buttons

Many chips look like:

```html
<button aria-label="Plugins" data-composer-navigation-target="plugins">
  <span class="…dropdownLabelText…">
    <span class="…dropdownLabelValue…">Plugins</span>
  </span>
</button>
```

**Do not** style `button > span:first-child` as a 20×20 icon badge unless that node is actually an icon. That pattern crushes labels (e.g. 43px-wide “PLUGINS” / “AUTO AUTO”).

Safe pattern:

1. Style the **button** (padding, border, gradient, spine).
2. Keep label spans as `inline-flex` / text; reset badge rules.
3. Scope Chat-only rules so they do not hit Work model:

```css
/* Chat model only */
button[aria-label="Select ChatGPT model"] { … }
button[data-codex-intelligence-trigger="true"]:not([data-composer-navigation-target="reasoning"]) { … }

/* Work model only */
button[data-composer-navigation-target="reasoning"] { … }
```

### Work model measurement ghost

Work model picker includes a hidden measurement node (`[class*="ModelPickerTriggerMeasurement"]`). If CSS sets `display: contents` or forces it into flow, wide kids inflate the chip.

Keep measurement out of layout:

```css
button[data-composer-navigation-target="reasoning"] > span[class*="Measurement"] {
  position: absolute !important;
  width: 0 !important;
  height: 0 !important;
  overflow: hidden !important;
  visibility: hidden !important;
  display: block !important; /* not contents */
  pointer-events: none !important;
}
```

## Animations (safe vs unsafe)

| Safe | Unsafe near composer |
| --- | --- |
| Opacity fades on shells | `transform` / `filter` on ancestors of `.composer-surface-chrome` |
| Transform on **leaf** nodes (title, mission rows, polaroid) | `animation-fill-mode: both` ending in `translateY(0)` on wrappers (still creates a fixed containing block) |
| Box-shadow glows | Animating `transform` on `.dream-home` / content frame |

Keyframes that end with transform should use **`transform: none`**, not `translateY(0)`.

Prefer opacity-only on:

- `.dream-home`
- `.app-shell-main-content-frame`
- any ancestor that also contains the composer

Always:

```css
.composer-surface-chrome {
  transform: none !important;
  filter: none !important;
  animation: none !important;
}
```

## Suggestions

| Mode | Selectors |
| --- | --- |
| Chat home cards | `.group/home-suggestions button` **without** list-item class |
| Work mission rows | `button.group/home-suggestion-list-item` |

Scope card min-heights so they do not force tall Work list rows:

```css
.group\/home-suggestions button:not(.group\/home-suggestion-list-item) { min-height: 126px; }
.group\/home-suggestions button.group\/home-suggestion-list-item { min-height: 52px; /* row */ }
```

## Stable shell selectors

- Sidebar: `aside.app-shell-left-panel`
- Main: `main.main-surface`
- Header: `header.app-header-tint`
- Composer: `.composer-surface-chrome`, `.ProseMirror`
- Home mark: `.dream-home` (runtime); also `[container-name:home-main-content]` / welcome copy — do **not** rely only on `[data-testid="home-icon"]`

## Brand tokens (recommended)

Mirror host tokens under `:root.cdxtheme-host-codex` / `:root.cdxtheme-codex-skin` so unstyled chrome inherits the theme:

`--color-token-bg-primary`, `--color-token-side-bar-background`, `--color-token-text-primary`, `--color-token-border`, `--color-token-list-hover-background`, `--color-token-dropdown-background`, etc.

## Live verify (CDP, port 9335)

After pack + apply, spot-check geometry (do not guess):

| Check | Expected |
| --- | --- |
| Home grid kids | `gapBetween >= 0`, `verticalOverlap === false` |
| Chat composer | `position: fixed`, `gapFromViewportBottom ≈ theme-composer-bottom` |
| Work util vs missions | no overlap; util bottom ≤ composer top |
| Work stack order | util **above** composer (`orderOK`) |
| Chat ribbon | `overlapRatio ≈ 50` on Chat; higher/clear on Work |
| Chat vs Work toggle | Chat fixed composer still works after Work stack rules |

```bash
# theme-dir = one theme tree (resolve via AGENTS.md)
/path/to/cdxthemex theme pack theme-dir -o output/<theme-id>.cdxtheme --force
/path/to/cdxthemex apply -t ./output/<theme-id>.cdxtheme
```

Pack is declaration-only from this skill’s POV; `apply` is the separate runtime CLI (see [cli.md](cli.md)). Always re-pack after CSS edits; verify **Chat home**, **Work home**, and **conversation detail** after composer or ribbon changes.
