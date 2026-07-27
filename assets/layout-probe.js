(() => {
  const style = getComputedStyle(document.documentElement);
  const composer = document.querySelector('.composer-surface-chrome');
  const composerCs = composer ? getComputedStyle(composer) : null;
  const dreamHome = !!document.querySelector('.dream-home');
  const work = document.querySelector('[data-feature="game-source"]');
  const missions = [...document.querySelectorAll('button.group\\/home-suggestion-list-item')];
  const chatCards = [...document.querySelectorAll(
    '.group\\/home-suggestions button:not(.group\\/home-suggestion-list-item)'
  )];
  const polaroid = document.querySelector('.dream-polaroid');
  const ribbon = document.querySelector('.dream-ribbon');
  const util = document.querySelector('[data-composer-navigation-target="workspace-project"]');
  const utilStrip = util
    ? (util.closest('div.select-none.flex.flex-nowrap.items-center') || util)
    : null;
  const wrap = util ? util.closest('.relative.flex.flex-col.z-10') : null;
  const heroShell =
    document.querySelector('main.main-surface div.mx-auto:has(h1.heading-xl)') ||
    document.querySelector('main.main-surface div.mx-auto:has([data-feature="game-source"])');
  const thread = document.querySelector('.thread-scroll-container');
  const msg = document.querySelector('[data-message-author-role]');
  const h1 = document.querySelector('h1.heading-xl');
  const chrome = document.querySelector('#cdxtheme-codex-skin-chrome');

  let badTransform = 0;
  if (composer) {
    let el = composer.parentElement;
    for (let i = 0; i < 12 && el; i++, el = el.parentElement) {
      const s = getComputedStyle(el);
      if (s.transform && s.transform !== 'none') badTransform++;
      if (s.filter && s.filter !== 'none') badTransform++;
    }
  }

  const rect = (el) => {
    if (!el) return null;
    const r = el.getBoundingClientRect();
    return {
      top: Math.round(r.top),
      bottom: Math.round(r.bottom),
      left: Math.round(r.left),
      width: Math.round(r.width),
      height: Math.round(r.height),
    };
  };

  const cg = rect(composer);
  const ug = rect(utilStrip);
  const hr = rect(heroShell);
  const firstM = missions[0] ? missions[0].getBoundingClientRect() : null;
  const lastM = missions.length
    ? missions[missions.length - 1].getBoundingClientRect()
    : null;
  const rr = ribbon ? ribbon.getBoundingClientRect() : null;
  const before = composer ? getComputedStyle(composer, '::before') : null;

  const hostOk = document.documentElement.classList.contains('cdxtheme-host-codex');
  const accent = (
    style.getPropertyValue('--color-token-primary') ||
    ''
  ).trim();

  const context = work
    ? 'work-home'
    : ((thread || msg) && !dreamHome)
      ? 'conversation'
      : dreamHome
        ? 'chat-home'
        : 'other';

  const issues = [];
  if (!hostOk) issues.push('missing cdxtheme-host-codex on <html>');
  if (!chrome) issues.push('missing #cdxtheme-codex-skin-chrome');
  if (!composer) issues.push('missing .composer-surface-chrome');

  if (dreamHome && !work) {
    if (composerCs && composerCs.position !== 'fixed') {
      issues.push('chat home composer not fixed: ' + (composerCs && composerCs.position));
    }
    if (cg) {
      const gap = Math.round(innerHeight - cg.bottom);
      if (gap > 80) issues.push('chat composer far from bottom gap=' + gap);
      if (gap < 0) issues.push('chat composer below viewport');
    }
    if (badTransform) issues.push('transform/filter on composer ancestors: ' + badTransform);
  }

  if (work) {
    if (wrap && getComputedStyle(wrap).position !== 'fixed') {
      issues.push('work stack wrap not fixed: ' + getComputedStyle(wrap).position);
    }
    if (composerCs && composerCs.position === 'fixed') {
      issues.push('work composer still fixed alone (want relative inside stack)');
    }
    if (ug && cg) {
      if (ug.top < innerHeight * 0.5) {
        issues.push('work util strip high (mid-page?) top=' + ug.top);
      }
      if (ug.bottom > cg.top + 4) {
        issues.push('work util overlaps composer');
      }
    }
    if (lastM && ug && lastM.bottom > ug.top - 2) {
      issues.push('missions crushed by util/composer');
    }
    if (missions.some((b) => b.getBoundingClientRect().height > 110)) {
      issues.push('work mission rows too tall');
    }
    if (rr && rr.width > 2 && rr.left > -500 && ug) {
      if (rr.bottom > ug.top && rr.top < ug.bottom) {
        issues.push('ribbon overlaps util strip');
      }
    }
  }

  if ((thread || msg) && !dreamHome && composerCs && composerCs.position === 'fixed') {
    issues.push('detail composer still fixed (should be relative)');
  }

  return {
    context,
    href: location.href,
    hostOk,
    chrome: !!chrome,
    brand: (document.querySelector('.dream-brand') || {}).textContent?.trim()?.slice(0, 80) || null,
    accent,
    dreamHome,
    isWork: !!work,
    isConversation: !!(thread || msg) && !dreamHome,
    h1: h1 ? h1.textContent.trim().slice(0, 60) : null,
    missions: missions.length,
    chatCards: chatCards.length,
    missionHeights: missions.slice(0, 4).map((b) => Math.round(b.getBoundingClientRect().height)),
    hero: hr,
    composer: composerCs
      ? {
          position: composerCs.position,
          bottom: composerCs.bottom,
          width: composerCs.width,
          zIndex: composerCs.zIndex,
        }
      : null,
    composerBox: cg,
    utilBox: ug,
    wrapPos: wrap ? getComputedStyle(wrap).position : null,
    gapHeroMissions: firstM && hr ? Math.round(firstM.top - hr.bottom) : null,
    gapMissionsUtil: lastM && ug ? Math.round(ug.top - lastM.bottom) : null,
    polaroid: polaroid
      ? {
          display: getComputedStyle(polaroid).display,
          top: getComputedStyle(polaroid).top,
          bottom: getComputedStyle(polaroid).bottom,
          width: getComputedStyle(polaroid).width,
        }
      : null,
    ribbon: ribbon
      ? {
          display: getComputedStyle(ribbon).display,
          bottom: getComputedStyle(ribbon).bottom,
          box: {
            left: Math.round(rr.left),
            top: Math.round(rr.top),
            width: Math.round(rr.width),
            height: Math.round(rr.height),
          },
        }
      : null,
    composerBefore: before
      ? { content: before.content, display: before.display, width: before.width }
      : null,
    badTransform,
    issues,
  };
})()
