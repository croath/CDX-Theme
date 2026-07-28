/* Left-rail ink check (Chat aside + Settings div.app-shell-left-panel).
 *
 * Catches white text on light rails and dark-on-dark after light recolor.
 * Also reports .sidebar-foreground-muted local --color-token-foreground remap
 * (host sets it from --vscode-foreground, often still white).
 *
 * Prefer: /path/to/run.sh sidebar-ink
 */
(function () {
  function sample(el) {
    if (!el) return null;
    var s = getComputedStyle(el);
    return {
      tag: el.tagName,
      text: (el.textContent || "").trim().slice(0, 40),
      color: s.color,
      fill: s.webkitTextFillColor || s.getPropertyValue("-webkit-text-fill-color"),
      bg: s.backgroundColor,
      opacity: s.opacity,
      cls: (el.className + "").toString().slice(0, 80)
    };
  }

  function isWhiteish(color) {
    if (!color) return false;
    return (
      /255,\s*255,\s*255/.test(color) ||
      /oklab\(\s*0\.99/.test(color) ||
      /rgba\(\s*255/.test(color)
    );
  }

  function lumFromColor(color) {
    var m = color && color.match(/rgba?\(([\d.]+),\s*([\d.]+),\s*([\d.]+)/);
    if (!m) return null;
    return (0.2126 * +m[1] + 0.7152 * +m[2] + 0.0722 * +m[3]) / 255;
  }

  var rails = Array.prototype.slice
    .call(document.querySelectorAll(".app-shell-left-panel, aside.app-shell-left-panel"))
    .map(function (rail) {
      var rs = getComputedStyle(rail);
      var muted = rail.querySelector(".sidebar-foreground-muted");
      var whiteish = [];
      var samples = [];
      var nodes = rail.querySelectorAll(
        "button, a, span, div.text-token-foreground, .text-fade-truncate, [class*='text-token-']"
      );
      for (var i = 0; i < nodes.length && samples.length < 24; i++) {
        var el = nodes[i];
        var t = (el.textContent || "").trim();
        if (!t && el.tagName !== "SVG") continue;
        // prefer leaf-ish text
        if (
          !Array.prototype.some.call(el.childNodes, function (n) {
            return n.nodeType === 3 && n.textContent.trim();
          })
        ) {
          continue;
        }
        var row = sample(el);
        samples.push(row);
        if (isWhiteish(row.color) || isWhiteish(row.fill)) whiteish.push(row);
      }

      var bg = rs.backgroundColor;
      if (rs.backgroundImage && rs.backgroundImage !== "none") {
        bg = bg + " | " + rs.backgroundImage.slice(0, 100);
      }

      return {
        tag: rail.tagName,
        cls: (rail.className + "").toString().slice(0, 90),
        color: rs.color,
        bg: bg,
        localForeground: muted
          ? getComputedStyle(muted).getPropertyValue("--color-token-foreground").trim()
          : rs.getPropertyValue("--color-token-foreground").trim(),
        vscodeForeground: rs.getPropertyValue("--vscode-foreground").trim() ||
          getComputedStyle(document.documentElement)
            .getPropertyValue("--vscode-foreground")
            .trim(),
        whiteishCount: whiteish.length,
        whiteish: whiteish.slice(0, 10),
        samples: samples.slice(0, 12)
      };
    });

  return {
    railCount: rails.length,
    rails: rails,
    ok: rails.every(function (r) {
      return r.whiteishCount === 0;
    })
  };
})()
