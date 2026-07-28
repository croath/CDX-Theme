/* Chat home geometry: hero top, shell flex vs grid, fixed composer, ::before.
 * Usage: /path/to/cdxthemex probe --expr "$(cat …/chat-layout.js)"
 * Prefer: /path/to/run.sh chat-layout
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
      flex: cs.flex,
      minH: cs.minHeight,
      height: cs.height,
      paddingBottom: cs.paddingBottom,
      gridRows: cs.gridTemplateRows
    };
  }

  var hero =
    document.querySelector(
      "main.main-surface div.mx-auto:has(h1.heading-xl)"
    ) ||
    document.querySelector(".dream-home div.mx-auto:has(h1.heading-xl)");
  var heroRow = document.querySelector(
    ".dream-home .flex.grow.basis-0.items-end.justify-center"
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
  var comp = document.querySelector(".composer-surface-chrome");
  var before = comp ? getComputedStyle(comp, "::before") : null;
  var header = document.querySelector("header.app-header-tint");
  var cards = document.querySelectorAll(
    ".group\\/home-suggestions button:not(.group\\/home-suggestion-list-item)"
  );

  return {
    isWork: !!document.querySelector("[data-feature=game-source]"),
    dreamHome: !!document.querySelector(".dream-home"),
    hasHeading: !!document.querySelector("h1.heading-xl"),
    headerBottom: header
      ? Math.round(header.getBoundingClientRect().bottom)
      : null,
    shell: box(shell),
    heroRow: box(heroRow),
    hero: box(hero),
    suggestionCards: cards.length,
    composer: box(comp),
    composerBefore: before
      ? { content: before.content, display: before.display }
      : null,
    gaps: {
      heroFromHeader:
        hero && header
          ? Math.round(
              hero.getBoundingClientRect().top -
                header.getBoundingClientRect().bottom
            )
          : null,
      composerFromViewportBottom: comp
        ? Math.round(
            window.innerHeight - comp.getBoundingClientRect().bottom
          )
        : null
    }
  };
})()
