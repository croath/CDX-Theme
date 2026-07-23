(() => {
  // CDXTheme apply payload for the Codex host.
  // Placeholders (JSON literals): theme, cssText, imageDataUrls
  var host = { id: "codex", className: "cdxtheme-host-codex" };
  var theme = __DREAM_THEME_JSON__;
  var cssText = __DREAM_CSS_JSON__;
  var imageDataUrls = __DREAM_IMAGES_JSON__;
  var profileId = "codex-theme-v1";

  var STYLE_ID = "cdxtheme-theme-style-" + host.id;
  var CHROME_ID = "cdxtheme-codex-skin-chrome";
  var CLASS_HOST = "cdxtheme-host-codex";
  var CLASS_SKIN = "cdxtheme-codex-skin";

  if (!window.__CDXTHEME__) window.__CDXTHEME__ = { hosts: {} };
  if (!window.__CDXTHEME__.hosts) window.__CDXTHEME__.hosts = {};
  var rootState = window.__CDXTHEME__;

  try {
    if (rootState.hosts[host.id] && rootState.hosts[host.id].cleanup) {
      rootState.hosts[host.id].cleanup();
    }
  } catch (e) {}

  // ── Image resolution (all package images: hero, texture, …) ──────────────
  // Always convert data: URLs → blob: URLs. Electron/Chromium silently drops
  // multi-MB data:image values from CSS custom properties (setProperty "succeeds"
  // but getPropertyValue returns empty). blob: URLs work for large heroes/textures.
  var imageUrls = {};
  var ownedImageUrls = [];
  function resolveImageUrl(dataUrl) {
    if (!dataUrl || typeof dataUrl !== "string" || dataUrl.indexOf("data:") !== 0) return null;
    try {
      var comma = dataUrl.indexOf(",");
      if (comma < 0) return null;
      var header = dataUrl.slice(0, comma);
      var mimeMatch = /^data:([^;,]+)/.exec(header);
      var mimeType = (mimeMatch && mimeMatch[1]) || "application/octet-stream";
      var b64 = dataUrl.slice(comma + 1);
      var binary = atob(b64);
      var bytes = new Uint8Array(binary.length);
      for (var index = 0; index < binary.length; index += 1) {
        bytes[index] = binary.charCodeAt(index);
      }
      var objectUrl = URL.createObjectURL(new Blob([bytes], { type: mimeType }));
      ownedImageUrls.push(objectUrl);
      return objectUrl;
    } catch (e) {
      // Last resort: only small data URLs are usable in CSS on Electron.
      return dataUrl.length < 100000 ? dataUrl : null;
    }
  }
  if (imageDataUrls && typeof imageDataUrls === "object") {
    for (var name in imageDataUrls) {
      if (!Object.prototype.hasOwnProperty.call(imageDataUrls, name)) continue;
      if (!/^[a-z0-9][a-z0-9_-]*$/i.test(name)) continue;
      var resolved = resolveImageUrl(imageDataUrls[name]);
      if (resolved) imageUrls[name] = resolved;
    }
  }
  var artDataUrl = (imageDataUrls && imageDataUrls.hero) || null;
  var artUrl = imageUrls.hero || null;

  // ── CSS custom properties ────────────────────────────────────────────────
  //   --cdxtheme-image-{name}
  //   --cdxtheme-art  /  --dream-art
  function setImageCssVars(root, imageName, imageUrl) {
    root.style.setProperty(
      "--cdxtheme-image-" + imageName,
      'url("' + imageUrl + '")'
    );
  }
  function clearImageCssVars(root) {
    if (!root || !root.style) return;
    for (var i = root.style.length - 1; i >= 0; i -= 1) {
      var prop = root.style.item(i);
      if (prop.indexOf("--cdxtheme-image-") === 0) {
        root.style.removeProperty(prop);
      }
    }
  }
  function setArtCssVars(root, url) {
    if (url) {
      var value = 'url("' + url + '")';
      root.style.setProperty("--dream-art", value);
      root.style.setProperty("--cdxtheme-art", value);
    } else {
      root.style.removeProperty("--dream-art");
      root.style.removeProperty("--cdxtheme-art");
    }
  }

  // ── codex-theme-v1 profile (chrome overlay) ──────────────────────────────
  var copy = {
    brandTitle: (theme && (theme.displayName || theme.id)) || "CDXTheme",
    brandSubtitle: "",
    signature: "",
    tagline: "",
    projectPrefix: "",
    projectLabel: "",
    ribbon: "✦",
  };
  if (theme && theme.copy && typeof theme.copy === "object") {
    for (var ck in theme.copy) {
      if (Object.prototype.hasOwnProperty.call(theme.copy, ck)) copy[ck] = theme.copy[ck];
    }
  }
  function cssString(value) {
    return JSON.stringify(String(value == null ? "" : value));
  }

  function updateChromeCopy(chrome) {
    function set(sel, text) {
      var node = chrome.querySelector(sel);
      if (node) node.textContent = String(text == null ? "" : text);
    }
    set(".dream-brand-title", copy.brandTitle);
    set(".dream-brand-subtitle", copy.brandSubtitle);
    set(".dream-signature", copy.signature);
    set(".dream-ribbon-emoji", copy.ribbon);
  }

  /**
   * Detect Codex home route. Package CSS only shows polaroid/ribbon/brand when
   * chrome has `.dream-home-shell` (e.g. `#…chrome.dream-home-shell .dream-polaroid`).
   * Codex DOM changed: older builds used [data-testid="home-icon"]; current builds
   * use container-name `home-main-content` and welcome copy instead.
   */
  function detectHomeMain() {
    try {
      var byIcon = document.querySelector(
        '[role="main"]:has([data-testid="home-icon"])'
      );
      if (byIcon) return byIcon;
    } catch (e) {}

    try {
      var byFeature = document.querySelector(
        '[role="main"]:has([data-feature="game-source"])'
      );
      if (byFeature && byFeature.querySelector('[class*="home-suggestions"]')) {
        return byFeature;
      }
    } catch (e) {}

    var mains = document.querySelectorAll('[role="main"]');
    for (var i = 0; i < mains.length; i += 1) {
      var main = mains[i];
      var cls = String(main.className || "");
      // Tailwind arbitrary: [container-name:home-main-content]
      if (
        cls.indexOf("home-main-content") >= 0 ||
        cls.indexOf("home-main") >= 0 ||
        cls.indexOf("container-name:home") >= 0
      ) {
        return main;
      }
      if (
        main.querySelector('[data-feature="game-source"]') &&
        main.querySelector('[class*="home-suggestions"]')
      ) {
        return main;
      }
      // Welcome surface without thread messages
      if (
        !main.querySelector("[data-message-author-role]") &&
        /what.?s on your mind|今天想|有什么.*想|on your mind/i.test(
          (main.innerText || "").slice(0, 240)
        )
      ) {
        return main;
      }
    }
    return null;
  }

  /** Build a clean chrome overlay node (valid decorative HTML, no interactive controls). */
  function createChromeNode() {
    var chrome = document.createElement("div");
    chrome.id = CHROME_ID;
    chrome.setAttribute("aria-hidden", "true");
    chrome.setAttribute("data-cdxtheme-chrome", "1");
    chrome.setAttribute("role", "presentation");

    var brand = document.createElement("div");
    brand.className = "dream-brand";
    brand.innerHTML =
      '<span class="dream-note" aria-hidden="true">✦</span>' +
      '<span class="dream-brand-text">' +
      '<b class="dream-brand-title"></b>' +
      '<small class="dream-brand-subtitle"></small>' +
      "</span>";

    var signature = document.createElement("div");
    signature.className = "dream-signature";

    var sparkles = document.createElement("div");
    sparkles.className = "dream-sparkles";
    sparkles.setAttribute("aria-hidden", "true");
    for (var s = 0; s < 8; s += 1) {
      sparkles.appendChild(document.createElement("i"));
    }

    var ribbon = document.createElement("div");
    ribbon.className = "dream-ribbon";
    ribbon.setAttribute("aria-hidden", "true");
    ribbon.innerHTML =
      '<span>✦</span><b class="dream-ribbon-emoji"></b><span>✦</span>';

    var polaroid = document.createElement("div");
    polaroid.className = "dream-polaroid";
    polaroid.setAttribute("aria-hidden", "true");

    chrome.appendChild(brand);
    chrome.appendChild(signature);
    chrome.appendChild(sparkles);
    chrome.appendChild(ribbon);
    chrome.appendChild(polaroid);
    return chrome;
  }

  /** Keep only one chrome node as last child of body. */
  function ensureChromeNode() {
    var nodes = document.querySelectorAll(
      "#" + CHROME_ID + ', [data-cdxtheme-chrome="1"]'
    );
    var chrome = null;
    for (var i = 0; i < nodes.length; i += 1) {
      if (!chrome && nodes[i].id === CHROME_ID) chrome = nodes[i];
      else nodes[i].parentNode && nodes[i].parentNode.removeChild(nodes[i]);
    }
    if (
      !chrome ||
      !chrome.querySelector(".dream-brand-title") ||
      !chrome.querySelector(".dream-polaroid")
    ) {
      if (chrome && chrome.parentNode) chrome.parentNode.removeChild(chrome);
      chrome = createChromeNode();
    }
    if (chrome.parentElement !== document.body) {
      document.body.appendChild(chrome);
    } else if (document.body.lastElementChild !== chrome) {
      // Keep above #root content for stacking; pointer-events:none so clicks pass through.
      document.body.appendChild(chrome);
    }
    return chrome;
  }

  /** Layout + stacking so decorations aren't clipped / buried under app chrome. */
  function layoutChrome(chrome, shellMain) {
    var shellBox = shellMain.getBoundingClientRect();
    chrome.style.setProperty("position", "fixed", "important");
    chrome.style.setProperty("pointer-events", "none", "important");
    // Package CSS uses overflow:hidden which clips rotated polaroid / ribbon.
    chrome.style.setProperty("overflow", "visible", "important");
    // App UI often uses z-[55]; stay above main surface but under true modals.
    chrome.style.setProperty("z-index", "40", "important");
    chrome.style.setProperty("left", Math.round(shellBox.left) + "px", "important");
    chrome.style.setProperty("top", Math.round(shellBox.top) + "px", "important");
    chrome.style.setProperty("width", Math.round(shellBox.width) + "px", "important");
    chrome.style.setProperty("height", Math.round(shellBox.height) + "px", "important");
    chrome.style.setProperty("margin", "0", "important");
    chrome.style.setProperty("box-sizing", "border-box", "important");
  }

  /**
   * Theme CSS pins .composer-surface-chrome with position:fixed + bottom.
   * Publish main-surface geometry so left/width track the sidebar resize.
   */
  function layoutComposerPin(shellMain) {
    var root = document.documentElement;
    if (!root || !shellMain) return;
    var box = shellMain.getBoundingClientRect();
    var pad = 24;
    var left = Math.max(0, Math.round(box.left + pad));
    var width = Math.max(0, Math.round(box.width - pad * 2));
    root.style.setProperty("--cdxtheme-composer-left", left + "px");
    root.style.setProperty("--cdxtheme-composer-width", width + "px");
  }

  /**
   * Force all dream-* chrome decorations visible on home.
   * Theme CSS may still default to display:none; runtime wins with !important.
   */
  function syncHomeChromeDecor(chrome, isHome) {
    if (!chrome) return;
    chrome.classList.toggle("dream-home-shell", Boolean(isHome));

    var polaroid = chrome.querySelector(".dream-polaroid");
    var ribbon = chrome.querySelector(".dream-ribbon");
    var brand = chrome.querySelector(".dream-brand");
    var signature = chrome.querySelector(".dream-signature");
    var sparkles = chrome.querySelector(".dream-sparkles");

    function show(el, display) {
      if (!el) return;
      if (isHome) el.style.setProperty("display", display, "important");
      else el.style.removeProperty("display");
      el.style.setProperty("visibility", "visible", "important");
      el.style.setProperty("opacity", el === sparkles ? "0.85" : "1", "important");
    }

    show(brand, "flex");
    show(signature, "block");
    show(sparkles, "block");
    show(ribbon, "flex");
    show(polaroid, "block");

    if (polaroid) {
      if (isHome && artUrl) {
        polaroid.style.setProperty(
          "background-image",
          'url("' + artUrl + '")',
          "important"
        );
      } else if (!isHome) {
        polaroid.style.removeProperty("background-image");
      }
    }
  }

  function profileEnsure() {
    var root = document.documentElement;
    if (!root) return false;

    root.classList.add(CLASS_SKIN);
    root.dataset.codexSkinTheme = theme.id;
    root.dataset.codexSkinBrand = "cdxtheme";

    setArtCssVars(root, artUrl);
    root.style.setProperty("--dream-tagline", cssString(copy.tagline));
    root.style.setProperty("--dream-project-prefix", cssString(copy.projectPrefix));
    root.style.setProperty("--dream-project-label", cssString(copy.projectLabel));

    var shellMain =
      document.querySelector("main.main-surface") || document.querySelector("main");
    var home = detectHomeMain();
    var homes = document.querySelectorAll('[role="main"].dream-home');
    for (var h = 0; h < homes.length; h += 1) {
      if (homes[h] !== home) homes[h].classList.remove("dream-home");
    }
    if (home) home.classList.add("dream-home");
    if (!shellMain || !document.body) return true;

    if (home) shellMain.classList.add("dream-home-shell");
    else shellMain.classList.remove("dream-home-shell");

    var chrome = ensureChromeNode();
    layoutChrome(chrome, shellMain);
    layoutComposerPin(shellMain);
    updateChromeCopy(chrome);
    syncHomeChromeDecor(chrome, Boolean(home));
    return true;
  }

  function profileCleanup() {
    var root = document.documentElement;
    if (root) {
      root.classList.remove(CLASS_SKIN);
      delete root.dataset.codexSkinTheme;
      delete root.dataset.codexSkinBrand;
      setArtCssVars(root, null);
      root.style.removeProperty("--dream-tagline");
      root.style.removeProperty("--dream-project-prefix");
      root.style.removeProperty("--dream-project-label");
      root.style.removeProperty("--cdxtheme-composer-left");
      root.style.removeProperty("--cdxtheme-composer-width");
    }
    var dreamHomes = document.querySelectorAll(".dream-home");
    for (var dh = 0; dh < dreamHomes.length; dh += 1) {
      dreamHomes[dh].classList.remove("dream-home");
    }
    var shells = document.querySelectorAll(".dream-home-shell");
    for (var sh = 0; sh < shells.length; sh += 1) {
      shells[sh].classList.remove("dream-home-shell");
    }
    var chrome = document.getElementById(CHROME_ID);
    if (chrome) chrome.remove();
  }

  function profileVerify() {
    var root = document.documentElement;
    var chrome = document.getElementById(CHROME_ID);
    var missing = [];
    if (!root || !root.classList.contains(CLASS_SKIN)) {
      missing.push({
        name: "root-class",
        selectors: ["html.cdxtheme-codex-skin"],
      });
    }
    if (!chrome) missing.push({ name: "chrome", selectors: ["#" + CHROME_ID] });
    if (artDataUrl && root) {
      var hasArt =
        root.style.getPropertyValue("--dream-art") ||
        root.style.getPropertyValue("--cdxtheme-art") ||
        root.style.getPropertyValue("--cdxtheme-image-hero");
      if (!hasArt) {
        missing.push({
          name: "art-variable",
          selectors: ["--dream-art", "--cdxtheme-art", "--cdxtheme-image-hero"],
        });
      }
    }
    return {
      id: profileId,
      pass: missing.length === 0,
      missing: missing,
      rootClassPresent: Boolean(root && root.classList.contains(CLASS_SKIN)),
      chromePresent: Boolean(chrome),
    };
  }

  // ── Core ensure ──────────────────────────────────────────────────────────
  function ensure() {
    var root = document.documentElement;
    if (!root) return false;

    root.classList.add("cdxtheme-theme", CLASS_HOST, host.className);
    root.dataset.cdxthemeHost = host.id;
    root.dataset.cdxthemeTheme = theme.id;
    // dataset values are always strings; keep state.version as a number.
    root.dataset.cdxthemeThemeVersion = String(theme.version);

    for (var imgName in imageUrls) {
      if (!Object.prototype.hasOwnProperty.call(imageUrls, imgName)) continue;
      setImageCssVars(root, imgName, imageUrls[imgName]);
    }
    setArtCssVars(root, artUrl);

    var style = document.getElementById(STYLE_ID);
    if (!style) {
      style = document.createElement("style");
      style.id = STYLE_ID;
      (document.head || root).appendChild(style);
    }
    if (style.dataset.themeVersion !== theme.id + "@" + theme.version) {
      style.textContent = cssText;
      style.dataset.themeVersion = theme.id + "@" + theme.version;
    }

    profileEnsure();
    return true;
  }

  var timer = null;
  var observer = new MutationObserver(function () {
    if (timer) clearTimeout(timer);
    timer = setTimeout(ensure, 120);
  });
  observer.observe(document.documentElement, { childList: true, subtree: true });
  var interval = setInterval(ensure, 5000);

  function cleanup() {
    observer.disconnect();
    if (timer) clearTimeout(timer);
    clearInterval(interval);
    profileCleanup();
    for (var u = 0; u < ownedImageUrls.length; u += 1) {
      try {
        URL.revokeObjectURL(ownedImageUrls[u]);
      } catch (e) {}
    }
    ownedImageUrls.length = 0;

    var styleNode = document.getElementById(STYLE_ID);
    if (styleNode) styleNode.remove();
    var chromeNode = document.getElementById(CHROME_ID);
    if (chromeNode) chromeNode.remove();

    var root = document.documentElement;
    if (root) {
      root.classList.remove(host.className, CLASS_HOST, CLASS_SKIN, "cdxtheme-theme");
      setArtCssVars(root, null);
      clearImageCssVars(root);
      root.style.removeProperty("--dream-tagline");
      root.style.removeProperty("--dream-project-prefix");
      root.style.removeProperty("--dream-project-label");
      root.style.removeProperty("--cdxtheme-composer-left");
      root.style.removeProperty("--cdxtheme-composer-width");
      if (root.dataset.cdxthemeHost === host.id) {
        delete root.dataset.cdxthemeHost;
        delete root.dataset.cdxthemeTheme;
        delete root.dataset.cdxthemeThemeVersion;
      }
      delete root.dataset.codexSkinTheme;
      delete root.dataset.codexSkinBrand;
    }
    delete rootState.hosts[host.id];
    return true;
  }

  function verifyProfile() {
    var root = document.documentElement;
    var missing = [];
    var profile = profileVerify();
    if (profile.missing && profile.missing.length) {
      for (var i = 0; i < profile.missing.length; i += 1) missing.push(profile.missing[i]);
    }
    for (var imgCheck in imageUrls) {
      if (!Object.prototype.hasOwnProperty.call(imageUrls, imgCheck)) continue;
      if (!root) break;
      if (!root.style.getPropertyValue("--cdxtheme-image-" + imgCheck)) {
        missing.push({
          name: "image-" + imgCheck,
          selectors: ["--cdxtheme-image-" + imgCheck],
        });
      }
    }
    if (!document.getElementById(STYLE_ID)) {
      missing.push({ name: "style", selectors: ["#" + STYLE_ID] });
    }
    return {
      id: profileId,
      pass: missing.length === 0,
      missing: missing,
      rootClassPresent: profile.rootClassPresent,
      chromePresent: profile.chromePresent,
      imageNames: Object.keys(imageUrls),
    };
  }

  var hostState = {
    cleanup: cleanup,
    ensure: ensure,
    observer: observer,
    interval: interval,
    themeId: theme.id,
    version: theme.version,
    imageNames: Object.keys(imageUrls),
    profileId: profileId,
    verifyProfile: verifyProfile,
  };
  rootState.hosts[host.id] = hostState;
  ensure();
  return {
    installed: true,
    appId: host.id,
    themeId: theme.id,
    version: theme.version,
    images: Object.keys(imageUrls),
  };
})()
