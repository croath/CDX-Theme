/* Switch header Composer mode (Chat | Work).
 * MODE is injected by /path/to/run.sh as a leading assignment, e.g.:
 *   var __CDXTHEME_MODE__ = "work";
 * Prefer: /path/to/run.sh set-mode chat|work
 */
(function () {
  var mode = (
    typeof __CDXTHEME_MODE__ !== "undefined" ? __CDXTHEME_MODE__ : "work"
  )
    .toString()
    .toLowerCase();
  var wantWork = mode === "work" || mode === "w";
  var group = document.querySelector(
    '[role="group"][aria-label="Composer mode"]'
  );
  if (!group) {
    return { ok: false, reason: "Composer mode group not found", mode: mode };
  }
  var buttons = Array.prototype.slice.call(group.querySelectorAll("button"));
  var target = buttons.find(function (b) {
    var t = (b.innerText || "").trim().toLowerCase();
    if (wantWork) return t.indexOf("work") !== -1;
    return t.indexOf("chat") !== -1;
  });
  if (!target) {
    return {
      ok: false,
      reason: "mode button not found",
      mode: mode,
      buttons: buttons.map(function (b) {
        return {
          text: b.innerText.trim(),
          pressed: b.getAttribute("aria-pressed")
        };
      })
    };
  }
  target.click();
  return {
    ok: true,
    mode: wantWork ? "work" : "chat",
    text: target.innerText.trim(),
    pressed: target.getAttribute("aria-pressed"),
    all: buttons.map(function (b) {
      return {
        text: b.innerText.trim(),
        pressed: b.getAttribute("aria-pressed")
      };
    })
  };
})()
