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
