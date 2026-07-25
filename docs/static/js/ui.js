/* Chrome that needs the viewport or the scroll position to decide.
 *
 * Everything here degrades to a usable page when it does not run: the sidebar
 * stays open, and the table of contents stays a plain list of links.
 */
(function () {
  /* The sidebar ships open so that a page without JavaScript is complete. On a
     narrow screen that means a long list before the content, so close it —
     but only there, and only once, before anything is painted. */
  var drawer = document.getElementById("nav-drawer");
  if (drawer && matchMedia("(max-width: 60rem)").matches) {
    drawer.open = false;
  }

  /* Mark the heading the reader is currently under.
     rootMargin pins the trigger line near the top of the viewport: without it
     a heading counts as "visible" while it is still at the bottom of the
     screen, and the marker runs ahead of the reader. */
  var links = document.querySelectorAll(".toc a[href^='#']");
  if (!links.length || !("IntersectionObserver" in window)) return;

  var byId = {};
  var headings = [];
  links.forEach(function (a) {
    var id = decodeURIComponent(a.getAttribute("href").slice(1));
    var h = document.getElementById(id);
    if (!h) return;
    byId[id] = a;
    headings.push(h);
  });

  var current = null;
  var seen = {};

  function mark() {
    // 가장 위에 있는, 이미 지나온 제목이 지금 읽는 곳이다.
    var best = null;
    headings.forEach(function (h) {
      if (seen[h.id]) best = h;
    });
    if (!best || best === current) return;
    if (current && byId[current.id]) byId[current.id].removeAttribute("aria-current");
    byId[best.id].setAttribute("aria-current", "true");
    current = best;
  }

  var io = new IntersectionObserver(
    function (entries) {
      entries.forEach(function (e) {
        seen[e.target.id] = e.boundingClientRect.top < 0 || e.isIntersecting;
      });
      mark();
    },
    { rootMargin: "-10% 0px -70% 0px", threshold: 0 }
  );
  headings.forEach(function (h) { io.observe(h); });
})();

/* Copy buttons on code blocks.
 *
 * The documentation says line numbers and a copy button are the site's own CSS
 * and JavaScript rather than configuration keys. This is that claim, kept.
 */
(function () {
  var s = document.currentScript || document.querySelector("script[data-copy]");
  var COPY = (s && s.dataset.copy) || "Copy";
  var DONE = (s && s.dataset.copied) || "Copied";
  if (!navigator.clipboard) return;

  document.querySelectorAll(".prose pre.highlight").forEach(function (pre) {
    var code = pre.querySelector("code");
    if (!code) return;
    var b = document.createElement("button");
    b.type = "button";
    b.className = "copy";
    b.textContent = COPY;
    b.addEventListener("click", function () {
      // pre.textContent를 쓰면 버튼 자신의 라벨까지 복사된다.
      navigator.clipboard.writeText(code.textContent).then(function () {
        b.textContent = DONE;
        b.dataset.done = "";
        setTimeout(function () {
          b.textContent = COPY;
          delete b.dataset.done;
        }, 1200);
      });
    });
    pre.appendChild(b);
  });
})();
