/* Dump Composer mode group, header text, and useful nav buttons.
 * Locale-agnostic debug when --tab / go-work-home miss.
 * Prefer: /path/to/run.sh discover-nav
 */
(function () {
  var groups = Array.prototype.map.call(
    document.querySelectorAll("[role=group]"),
    function (g) {
      return {
        aria: g.getAttribute("aria-label"),
        text: (g.innerText || "").slice(0, 80),
        buttons: Array.prototype.map.call(
          g.querySelectorAll("button"),
          function (b) {
            return {
              text: (b.innerText || "").trim(),
              pressed: b.getAttribute("aria-pressed"),
              aria: b.getAttribute("aria-label")
            };
          }
        )
      };
    }
  );

  var interesting = Array.prototype.map
    .call(document.querySelectorAll("button, a, [role=button]"), function (b) {
      return {
        text: (b.innerText || "").trim().replace(/\s+/g, " ").slice(0, 50),
        aria: b.getAttribute("aria-label"),
        title: b.getAttribute("title"),
        testid: b.getAttribute("data-testid"),
        href: b.getAttribute("href")
      };
    })
    .filter(function (b) {
      var s = (
        (b.text || "") +
        (b.aria || "") +
        (b.title || "") +
        (b.testid || "")
      ).toLowerCase();
      return /home|work|chat|新|工作|对话|任务|mission|new|task|新建|主页|plus|filter|项目|plugin|插件/.test(
        s
      );
    })
    .slice(0, 40);

  var header = document.querySelector("header");
  var aside = document.querySelector("aside");

  return {
    href: location.href,
    dreamHome: !!document.querySelector(".dream-home"),
    gameSource: !!document.querySelector("[data-feature=game-source]"),
    heading: !!document.querySelector("h1.heading-xl"),
    headerText: header ? header.innerText.slice(0, 300) : null,
    sidebarText: aside ? aside.innerText.slice(0, 200) : null,
    groups: groups,
    interesting: interesting
  };
})()
