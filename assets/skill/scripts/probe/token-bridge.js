/* Dump host surface / foreground tokens light skins must re-bridge.
 *
 * Codex keeps many dark-theme defaults (--vscode-foreground white,
 * --color-background-panel dark, etc.). Use this after pack/apply to see
 * which bridges are still wrong.
 *
 * Prefer: /path/to/run.sh token-bridge
 */
(function () {
  var root = getComputedStyle(document.documentElement);
  var keys = [
    "--vscode-foreground",
    "--vscode-sideBar-foreground",
    "--vscode-icon-foreground",
    "--vscode-input-foreground",
    "--vscode-input-placeholderForeground",
    "--color-text-foreground",
    "--color-text-foreground-secondary",
    "--color-text-foreground-tertiary",
    "--color-token-foreground",
    "--color-token-text-primary",
    "--color-token-text-secondary",
    "--color-token-text-tertiary",
    "--color-token-icon-foreground",
    "--color-token-description-foreground",
    "--color-token-input-foreground",
    "--color-token-input-placeholder-foreground",
    "--color-token-bg-primary",
    "--color-token-bg-secondary",
    "--color-token-bg-tertiary",
    "--color-token-main-surface-primary",
    "--color-token-side-bar-background",
    "--color-token-dropdown-background",
    "--color-token-dropdown-foreground",
    "--color-token-list-active-selection-foreground",
    "--color-token-text-link-foreground",
    "--color-background-panel",
    "--color-background-surface",
    "--color-background-surface-under",
    "--color-background-elevated-primary",
    "--color-background-elevated-primary-opaque",
    "--color-background-elevated-secondary",
    "--color-background-elevated-secondary-opaque",
    "--color-background-button-secondary",
    "--theme-text",
    "--theme-muted",
    "--theme-bg",
    "--theme-panel",
    "--theme-panel-mid",
    "--theme-accent"
  ];

  var tokens = {};
  var missing = [];
  for (var i = 0; i < keys.length; i++) {
    var k = keys[i];
    var v = root.getPropertyValue(k).trim();
    tokens[k] = v || null;
    if (!v) missing.push(k);
  }

  // Heuristic: values that still look like dark-theme white ink / dark panels
  function isWhiteish(v) {
    if (!v) return false;
    return (
      /#fff(fff)?\b/i.test(v) ||
      /255,\s*255,\s*255/.test(v) ||
      /oklab\(\s*0\.99/.test(v) ||
      /rgba?\(\s*255/.test(v)
    );
  }
  function isDarkHex(v) {
    if (!v) return false;
    var m = v.match(/#([0-9a-f]{6})\b/i);
    if (!m) return false;
    var n = parseInt(m[1], 16);
    var r = (n >> 16) & 255;
    var g = (n >> 8) & 255;
    var b = n & 255;
    return (0.2126 * r + 0.7152 * g + 0.0722 * b) / 255 < 0.4;
  }

  var suspectWhite = [];
  var suspectDarkPanel = [];
  for (var j = 0; j < keys.length; j++) {
    var key = keys[j];
    var val = tokens[key];
    if (!val) continue;
    if (/foreground|text|icon|placeholder/i.test(key) && isWhiteish(val)) {
      suspectWhite.push({ key: key, val: val });
    }
    if (/background|panel|surface|elevated|bg-/i.test(key) && isDarkHex(val)) {
      suspectDarkPanel.push({ key: key, val: val });
    }
  }

  return {
    host: document.documentElement.className,
    colorScheme: root.colorScheme || root.getPropertyValue("color-scheme").trim(),
    tokens: tokens,
    missing: missing,
    suspectWhiteInk: suspectWhite,
    suspectDarkSurfaces: suspectDarkPanel,
    ok:
      suspectWhite.length === 0 &&
      suspectDarkPanel.length === 0 &&
      missing.indexOf("--theme-text") === -1
  };
})()
