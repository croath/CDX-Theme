/* Inspect why hero is mid-page: hero-row kids, parent grid rows, ancestor chain.
 * Prefer: /path/to/run.sh hero-row-tree
 */
(function () {
  var heroRow =
    document.querySelector(
      ".dream-home .flex.grow.basis-0.items-end.justify-center"
    ) ||
    document.querySelector(
      "main.main-surface .flex.grow.basis-0.items-end.justify-center"
    );
  if (!heroRow) {
    return {
      err: "no heroRow",
      dreamHome: !!document.querySelector(".dream-home"),
      isWork: !!document.querySelector("[data-feature=game-source]")
    };
  }

  var kids = Array.prototype.map.call(heroRow.children, function (c) {
    var r = c.getBoundingClientRect();
    var cs = getComputedStyle(c);
    return {
      cls: String(c.className).slice(0, 100),
      top: Math.round(r.top),
      bottom: Math.round(r.bottom),
      h: Math.round(r.height),
      w: Math.round(r.width),
      position: cs.position,
      display: cs.display,
      minH: cs.minHeight,
      height: cs.height,
      flex: cs.flex,
      margin: cs.margin,
      padding: cs.padding,
      hasGame: !!c.querySelector("[data-feature=game-source]"),
      hasH1: !!c.querySelector("h1"),
      childCount: c.children.length
    };
  });

  var parent = heroRow.parentElement;
  var p = null;
  if (parent) {
    var pcs = getComputedStyle(parent);
    p = {
      cls: String(parent.className).slice(0, 120),
      display: pcs.display,
      gridTemplateRows: pcs.gridTemplateRows,
      flexDir: pcs.flexDirection,
      height: pcs.height,
      minH: pcs.minHeight,
      gap: pcs.gap,
      paddingTop: pcs.paddingTop,
      paddingBottom: pcs.paddingBottom,
      top: Math.round(parent.getBoundingClientRect().top),
      h: Math.round(parent.getBoundingClientRect().height),
      kids: Array.prototype.map.call(parent.children, function (c) {
        return {
          cls: String(c.className).slice(0, 80),
          h: Math.round(c.getBoundingClientRect().height),
          top: Math.round(c.getBoundingClientRect().top),
          flex: getComputedStyle(c).flex,
          marginTop: getComputedStyle(c).marginTop
        };
      })
    };
  }

  var el = heroRow;
  var chain = [];
  for (var i = 0; i < 8 && el; i++) {
    var cs = getComputedStyle(el);
    chain.push({
      cls: String(el.className).slice(0, 90),
      display: cs.display,
      h: Math.round(el.getBoundingClientRect().height),
      minH: cs.minHeight,
      height: cs.height,
      flex: cs.flex,
      gridRows: cs.gridTemplateRows
    });
    el = el.parentElement;
  }

  return {
    isWork: !!document.querySelector("[data-feature=game-source]"),
    dreamHome: !!document.querySelector(".dream-home"),
    kids: kids,
    parent: p,
    chain: chain
  };
})()
