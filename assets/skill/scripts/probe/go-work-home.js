/* Navigate to Work home by clicking New task / 新建任务 (locale-aware).
 * Does not only flip Composer mode — opens the Work landing surface.
 * Prefer: /path/to/run.sh go-work-home
 * Then sleep ~1s and run work-layout.
 */
(function () {
  var labels = [
    "新建任务",
    "New task",
    "New Task",
    "Create task",
    "新任务"
  ];
  var buttons = Array.prototype.slice.call(
    document.querySelectorAll("button, a, [role=button]")
  );

  function match(el) {
    var text = (el.innerText || "").trim().replace(/\s+/g, " ");
    var aria = el.getAttribute("aria-label") || "";
    var title = el.getAttribute("title") || "";
    var blob = text + " " + aria + " " + title;
    for (var i = 0; i < labels.length; i++) {
      if (blob.indexOf(labels[i]) !== -1) return labels[i];
    }
    return null;
  }

  var hit = null;
  for (var i = 0; i < buttons.length; i++) {
    var m = match(buttons[i]);
    if (m) {
      hit = { el: buttons[i], via: m };
      break;
    }
  }

  if (!hit) {
    return {
      clicked: false,
      reason: "no New task / 新建任务 control found",
      isWork: !!document.querySelector("[data-feature=game-source]"),
      dreamHome: !!document.querySelector(".dream-home")
    };
  }

  hit.el.click();
  return {
    clicked: true,
    via: hit.via,
    text: (hit.el.innerText || "").trim().slice(0, 40),
    aria: hit.el.getAttribute("aria-label")
  };
})()
