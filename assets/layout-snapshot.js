(() => {
  const html = document.documentElement;
  const style = getComputedStyle(html);
  const composer = document.querySelector('.composer-surface-chrome');
  const cs = composer ? getComputedStyle(composer) : null;
  const work = document.querySelector('[data-feature="game-source"]');
  const util = document.querySelector('[data-composer-navigation-target="workspace-project"]');
  const wrap = util ? util.closest('.relative.flex.flex-col.z-10') : null;
  const hero =
    document.querySelector('main.main-surface div.mx-auto:has(h1.heading-xl)') ||
    document.querySelector('main.main-surface div.mx-auto:has([data-feature="game-source"])');
  const before = composer ? getComputedStyle(composer, '::before') : null;
  return {
    href: location.href,
    hostClasses: [...html.classList].filter((c) => c.includes('cdxtheme')),
    brand: (document.querySelector('.dream-brand') || {}).textContent?.trim()?.slice(0, 100) || null,
    accent: (style.getPropertyValue('--color-token-primary') || '').trim(),
    dreamHome: !!document.querySelector('.dream-home'),
    isWork: !!work,
    missions: document.querySelectorAll('button.group\\/home-suggestion-list-item').length,
    hero: hero
      ? {
          h: Math.round(hero.getBoundingClientRect().height),
          cssH: getComputedStyle(hero).height,
        }
      : null,
    composer: cs
      ? {
          position: cs.position,
          bottom: cs.bottom,
          width: cs.width,
          box: (() => {
            const r = composer.getBoundingClientRect();
            return {
              top: Math.round(r.top),
              bottom: Math.round(r.bottom),
              h: Math.round(r.height),
            };
          })(),
        }
      : null,
    wrapPos: wrap ? getComputedStyle(wrap).position : null,
    utilPresent: !!util,
    chrome: !!document.querySelector('#cdxtheme-codex-skin-chrome'),
    composerBefore: before
      ? { content: before.content, display: before.display }
      : null,
  };
})()
