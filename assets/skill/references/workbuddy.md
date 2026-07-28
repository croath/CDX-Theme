# WorkBuddy target

Use app id `workbuddy`. Scope CSS under `html.cdxtheme-host-workbuddy` / `:root.cdxtheme-host-workbuddy`.

## Generate notes

- Start from skill `assets/theme-starter/workbuddy.css` (copy into `theme-dir/`) and the `targets.workbuddy` entry in `theme.json`.
- Prefer live semantic classes over stale template selectors.
- Stable cross-route landmarks often include:
  - root / shell: `.teams-container`, `#root`
  - sidebar: `.conversation-sidebar`, `.conversation-list`
  - workspace: `.teams-main-content`, `.main-content`, `.chat-container`, `.wb-cb-chat`
  - composer: `[role="textbox"][contenteditable="true"]`
- Keep home layout rules (`.wb-home-page`, `.wb-home-header`, `.quick-actions`, `.wb-home-composer`) in the theme package.
- Reference shared artwork with `var(--cdxtheme-image-hero)` and `var(--cdxtheme-image-texture)`.

## Home vs conversation

When styling both contexts:

1. Home header/hero, scene tabs, quick actions, and home composer.
2. Conversation with long text, tables or code, scrolling, and the conversation composer shell.
3. Sidebar selection, hover states, menus, and send controls remain usable.
4. No horizontal overflow or hidden native actions.

## Branding

- Scope only under `html.cdxtheme-host-workbuddy` / `:root.cdxtheme-host-workbuddy`.
- Prefer `cdxtheme-*` variables (`--cdxtheme-image-hero`).
- Keep home and conversation composers usable; avoid transform animations on wrappers that own the input shell.

## Pack

Include `targets.workbuddy` only when shipping WorkBuddy CSS:

```json
"workbuddy": {
  "css": "workbuddy.css"
}
```

Then pack from repo root: `/path/to/cdxthemex theme pack theme-dir -o output/<theme-id>.cdxtheme --force` (see [cli.md](cli.md)).
