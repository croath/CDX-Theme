/* Work home geometry: hero top, missions, util+composer stack, ::before, gaps.
 * Usage: /path/to/cdxthemex probe --expr "$(cat …/work-layout.js)"
 * Prefer: /path/to/run.sh work-layout
 */
(function () {
  function box(el) {
    if (!el) return null;
    var r = el.getBoundingClientRect();
    var cs = getComputedStyle(el);
    return {
      top: Math.round(r.top),
      bottom: Math.round(r.bottom),
      h: Math.round(r.height),
      left: Math.round(r.left),
      w: Math.round(r.width),
      position: cs.position,
      display: cs.display,
      alignItems: cs.alignItems,
      justifyContent: cs.justifyContent,
      flex: cs.flex,
      minH: cs.minHeight,
      height: cs.height,
      paddingTop: cs.paddingTop,
      paddingBottom: cs.paddingBottom,
      marginTop: cs.marginTop,
      gap: cs.gap,
      gridRows: cs.gridTemplateRows
    };
  }

  var hero =
    document.querySelector(
      "main.main-surface div.mx-auto:has([data-feature=game-source])"
    ) ||
    document.querySelector(
      ".dream-home div.mx-auto:has([data-feature=game-source])"
    );
  var heroRow =
    document.querySelector(
      ".dream-home .flex.grow.basis-0.items-end.justify-center"
    ) ||
    document.querySelector(
      "main.main-surface .flex.grow.basis-0.items-end.justify-center"
    );
  var shell =
    document.querySelector(".dream-home .min-h-0.w-full.flex-1.pt-6") ||
    document.querySelector(".dream-home .min-h-full.w-full.pt-6") ||
    document.querySelector(".dream-home .grid.grid-rows-2") ||
    document.querySelector(
      "[class*=home-main-content] .min-h-0.w-full.flex-1.pt-6"
    ) ||
    document.querySelector(
      "[class*=home-main-content] .grid.grid-rows-2"
    );
  var missions = document.querySelector(".group\\/home-suggestions");
  var utilBtn = document.querySelector(
    "[data-composer-navigation-target=workspace-project]"
  );
  var util = utilBtn
    ? utilBtn.closest("div.select-none.flex.flex-nowrap.items-center")
    : null;
  var wrap = document.querySelector(
    ".relative.flex.flex-col.z-10.gap-2.mb-10:has([data-composer-navigation-target=workspace-project])"
  );
  var comp = document.querySelector(".composer-surface-chrome");
  var before = comp ? getComputedStyle(comp, "::before") : null;
  var header = document.querySelector("header.app-header-tint");

  var orderOK = null;
  if (wrap) {
    var kids = Array.prototype.slice.call(wrap.children);
    var utilHost = kids.find(function (c) {
      return (
        c.classList.contains("absolute") ||
        c.querySelector(
          "[data-composer-navigation-target=workspace-project]"
        )
      );
    });
    var compHost = kids.find(function (c) {
      return c.querySelector(".composer-surface-chrome");
    });
    if (utilHost && compHost) {
      orderOK =
        utilHost.getBoundingClientRect().top <=
        compHost.getBoundingClientRect().top;
    }
  }

  return {
    isWork: !!document.querySelector("[data-feature=game-source]"),
    dreamHome: !!document.querySelector(".dream-home"),
    headerBottom: header
      ? Math.round(header.getBoundingClientRect().bottom)
      : null,
    shell: box(shell),
    heroRow: box(heroRow),
    hero: box(hero),
    missions: box(missions),
    wrap: box(wrap),
    util: box(util),
    composer: box(comp),
    composerBefore: before
      ? {
          content: before.content,
          display: before.display,
          w: before.width,
          h: before.height,
          top: before.top,
          left: before.left
        }
      : null,
    gaps: {
      heroFromHeader:
        hero && header
          ? Math.round(
              hero.getBoundingClientRect().top -
                header.getBoundingClientRect().bottom
            )
          : null,
      heroToMissions:
        hero && missions
          ? Math.round(
              missions.getBoundingClientRect().top -
                hero.getBoundingClientRect().bottom
            )
          : null,
      missionsToUtil:
        missions && util
          ? Math.round(
              util.getBoundingClientRect().top -
                missions.getBoundingClientRect().bottom
            )
          : null,
      utilToComposer:
        util && comp
          ? Math.round(
              comp.getBoundingClientRect().top -
                util.getBoundingClientRect().bottom
            )
          : null,
      orderOK: orderOK
    },
    wrapKids: wrap
      ? Array.prototype.map.call(wrap.children, function (c) {
          return {
            cls: String(c.className).slice(0, 60),
            order: getComputedStyle(c).order,
            top: Math.round(c.getBoundingClientRect().top)
          };
        })
      : null
  };
})()
