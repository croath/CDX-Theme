/* Settings page: rail + content card surfaces vs ink.
 *
 * Settings uses div.app-shell-left-panel (not aside) and section cards with
 * inline style: background-color: var(--color-background-panel, …).
 * Light skins often leave panel dark while forcing dark text.
 *
 * Prefer: open Settings first, then
 *   /path/to/run.sh settings-contrast
 */
(function () {
  function sample(el) {
    if (!el) return null;
    var s = getComputedStyle(el);
    var r = el.getBoundingClientRect();
    return {
      tag: el.tagName,
      cls: (el.className + "").toString().slice(0, 90),
      color: s.color,
      bg: s.backgroundColor,
      bgImg:
        s.backgroundImage && s.backgroundImage !== "none"
          ? s.backgroundImage.slice(0, 100)
          : null,
      style: (el.getAttribute("style") || "").slice(0, 120),
      w: Math.round(r.width),
      h: Math.round(r.height),
      left: Math.round(r.left),
      top: Math.round(r.top),
      text: (el.textContent || "").trim().slice(0, 40)
    };
  }

  function parseLum(bg) {
    var m = bg && bg.match(/rgba?\(([\d.]+),\s*([\d.]+),\s*([\d.]+)(?:,\s*([\d.]+))?\)/);
    var m2 = bg && bg.match(/color\(srgb\s+([\d.]+)\s+([\d.]+)\s+([\d.]+)/);
    if (m) {
      var a = m[4] !== undefined ? +m[4] : 1;
      if (a < 0.05) return null;
      return (0.2126 * +m[1] + 0.7152 * +m[2] + 0.0722 * +m[3]) / 255;
    }
    if (m2) return 0.2126 * +m2[1] + 0.7152 * +m2[2] + 0.0722 * +m2[3];
    return null;
  }

  var onSettings = !!document.querySelector("[data-settings-panel-slug]");
  var root = getComputedStyle(document.documentElement);
  var tokens = {
    panel: root.getPropertyValue("--color-background-panel").trim(),
    elevatedPrimary: root.getPropertyValue("--color-background-elevated-primary").trim(),
    elevatedSecondary: root
      .getPropertyValue("--color-background-elevated-secondary")
      .trim(),
    surface: root.getPropertyValue("--color-background-surface").trim(),
    tokenFg: root.getPropertyValue("--color-token-foreground").trim(),
    textPrimary: root.getPropertyValue("--color-token-text-primary").trim(),
    themePanel: root.getPropertyValue("--theme-panel").trim(),
    themeText: root.getPropertyValue("--theme-text").trim()
  };

  var rail = document.querySelector(".app-shell-left-panel");
  var railInfo = rail ? sample(rail) : null;
  if (railInfo && rail) {
    var rs = getComputedStyle(rail);
    railInfo.localPanel = rs.getPropertyValue("--color-background-panel").trim();
    railInfo.localFg = rs.getPropertyValue("--color-token-foreground").trim();
  }

  var cards = Array.prototype.slice
    .call(document.querySelectorAll(".rounded-2xl.border, .rounded-2xl"))
    .filter(function (el) {
      var r = el.getBoundingClientRect();
      return r.width > 200 && r.left > 240 && r.height > 40;
    })
    .slice(0, 8)
    .map(function (el) {
      var row = sample(el);
      row.lum = parseLum(row.bg);
      return row;
    });

  var darkCards = cards.filter(function (c) {
    return c.lum !== null && c.lum < 0.5;
  });

  // Content text samples (right of rail)
  var contentTexts = [];
  var els = document.querySelectorAll("h1, h2, label, span, p, a, button");
  for (var i = 0; i < els.length && contentTexts.length < 20; i++) {
    var el = els[i];
    var r = el.getBoundingClientRect();
    if (r.left < 280 || r.width < 2 || r.top > innerHeight) continue;
    var direct = "";
    for (var c = 0; c < el.childNodes.length; c++) {
      var n = el.childNodes[c];
      if (n.nodeType === 3 && n.textContent.trim()) {
        direct += (direct ? " " : "") + n.textContent.trim();
      }
    }
    if (!direct) continue;
    var s = getComputedStyle(el);
    contentTexts.push({
      t: direct.slice(0, 40),
      color: s.color,
      left: Math.round(r.left),
      top: Math.round(r.top),
      cls: (el.className + "").toString().slice(0, 55)
    });
  }
  contentTexts.sort(function (a, b) {
    return a.top - b.top || a.left - b.left;
  });

  var chips = Array.prototype.slice
    .call(
      document.querySelectorAll(
        "main button.border-token-border, .main-surface button.border-token-border"
      )
    )
    .slice(0, 6)
    .map(function (el) {
      var row = sample(el);
      row.lum = parseLum(row.bg);
      return row;
    });

  return {
    onSettings: onSettings,
    tokens: tokens,
    rail: railInfo,
    cards: cards,
    darkCardCount: darkCards.length,
    darkCards: darkCards,
    chips: chips,
    contentTexts: contentTexts.slice(0, 16),
    ok: onSettings && darkCards.length === 0 && !!railInfo
  };
})()
