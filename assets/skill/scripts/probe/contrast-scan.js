/* Visible-UI contrast scan: light text + dark section surfaces.
 *
 * Use on light skins when “text is dark but cards are dark” or ink vanishes.
 * Scans elements with direct text nodes; reports whiteish ink and low-luma bgs.
 *
 * Prefer: /path/to/run.sh contrast-scan
 * Optional: /path/to/run.sh contrast-scan --wait-ms 500
 */
(function () {
  function isWhiteish(color) {
    if (!color) return false;
    return (
      /255,\s*255,\s*255/.test(color) ||
      /oklab\(\s*0\.99/.test(color) ||
      /rgba\(\s*255/.test(color)
    );
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

  var whiteInk = [];
  var darkSurfaces = [];
  var seenInk = {};
  var seenBg = {};

  var nodes = document.querySelectorAll("body *");
  for (var i = 0; i < nodes.length; i++) {
    var el = nodes[i];
    var r = el.getBoundingClientRect();
    if (r.width < 2 || r.height < 2 || r.bottom < 0 || r.top > innerHeight) continue;

    var s = getComputedStyle(el);
    if (s.display === "none" || s.visibility === "hidden" || Number(s.opacity) < 0.05) {
      continue;
    }

    // Text with direct text node
    var direct = "";
    for (var c = 0; c < el.childNodes.length; c++) {
      var n = el.childNodes[c];
      if (n.nodeType === 3) {
        var tt = n.textContent.trim();
        if (tt) direct += (direct ? " " : "") + tt;
      }
    }
    if (direct && direct.length <= 80) {
      if (isWhiteish(s.color) || isWhiteish(s.webkitTextFillColor)) {
        var ik =
          direct.slice(0, 30) +
          "|" +
          s.color +
          "|" +
          Math.round(r.left) +
          "|" +
          Math.round(r.top);
        if (!seenInk[ik]) {
          seenInk[ik] = 1;
          whiteInk.push({
            t: direct.slice(0, 40),
            color: s.color,
            fill: s.webkitTextFillColor,
            left: Math.round(r.left),
            top: Math.round(r.top),
            cls: (el.className + "").toString().slice(0, 70),
            tag: el.tagName
          });
        }
      }
    }

    // Dark solid backgrounds (cards / rails / chips)
    if (r.width >= 40 && r.height >= 16) {
      var lum = parseLum(s.backgroundColor);
      if (lum !== null && lum < 0.45) {
        var bk =
          (el.className + "").toString().slice(0, 50) +
          "|" +
          s.backgroundColor +
          "|" +
          Math.round(r.width);
        if (!seenBg[bk]) {
          seenBg[bk] = 1;
          darkSurfaces.push({
            tag: el.tagName,
            cls: (el.className + "").toString().slice(0, 90),
            bg: s.backgroundColor,
            lum: Math.round(lum * 100) / 100,
            w: Math.round(r.width),
            h: Math.round(r.height),
            left: Math.round(r.left),
            top: Math.round(r.top),
            text: (el.textContent || "").trim().slice(0, 36),
            style: (el.getAttribute("style") || "").slice(0, 100)
          });
        }
      }
    }
  }

  whiteInk.sort(function (a, b) {
    return a.top - b.top || a.left - b.left;
  });
  darkSurfaces.sort(function (a, b) {
    return a.top - b.top || a.left - b.left;
  });

  return {
    whiteInkCount: whiteInk.length,
    darkSurfaceCount: darkSurfaces.length,
    whiteInk: whiteInk.slice(0, 25),
    darkSurfaces: darkSurfaces.slice(0, 25),
    onSettings: !!document.querySelector("[data-settings-panel-slug]"),
    ok: whiteInk.length === 0 && darkSurfaces.length === 0
  };
})()
