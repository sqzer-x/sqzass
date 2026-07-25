/* sqzass documentation search.
 *
 * Substring matching over the page text, not a word index. Korean attaches
 * particles to nouns and writes compounds without spaces, so a word index
 * cannot find 최적화 inside 검색엔진최적화. Running a morphological analyser
 * instead trades that for a worse problem: the dictionary does not know
 * loanwords, and technical documentation is made of them.
 *
 * The dialog is a <dialog>. Escape, the backdrop and the focus trap are the
 * platform's job, not ours.
 */
(function () {
  var lang = (document.documentElement.lang || "en").split("-")[0];

  var STRINGS = {
    en: {
      placeholder: "Search the documentation",
      empty: "Type to search",
      none: "No results for",
      loading: "Loading index…",
      count: function (n) { return n + (n === 1 ? " result" : " results"); },
      /* Arrows survive every fallback stack; ↵ does not, and renders as a box
         in the mono face this footer uses. Same reason the theme button says
         "Theme" instead of a moon. */
      hint: "↑↓ move · enter open · esc close",
    },
    ko: {
      placeholder: "문서 검색",
      empty: "검색어를 입력하세요",
      none: "결과가 없습니다:",
      loading: "색인을 불러오는 중…",
      count: function (n) { return n + "건"; },
      hint: "↑↓ 이동 · enter 열기 · esc 닫기",
    },
  };
  var T = STRINGS[lang] || STRINGS.en;

  var dialog = document.getElementById("search-dialog");
  var trigger = document.getElementById("search-trigger");
  if (!dialog || !trigger) return;

  var input = dialog.querySelector("input");
  var list = dialog.querySelector(".search-results");
  var status = dialog.querySelector(".search-status");
  input.placeholder = T.placeholder;
  dialog.querySelector(".search-hint").textContent = T.hint;

  var rows = null;
  var pending = false;
  var active = 0;

  /* The index is the whole body text of every page in this language. It is
     only worth downloading once someone actually opens the palette. */
  function load() {
    if (rows || pending) return;
    pending = true;
    status.textContent = T.loading;
    fetch(trigger.dataset.index)
      .then(function (r) { return r.json(); })
      .then(function (data) { rows = data; pending = false; render(); })
      .catch(function () { pending = false; status.textContent = "!"; });
  }

  function esc(s) {
    return s.replace(/[&<>"]/g, function (c) {
      return { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c];
    });
  }

  /* Every term has to appear somewhere in the row — an AND, not an OR. Two
     words that each match a different page are not a match. */
  function score(row, terms) {
    var title = row.t.toLowerCase();
    var desc = (row.d || "").toLowerCase();
    var body = row.c.toLowerCase();
    var total = 0;
    for (var i = 0; i < terms.length; i++) {
      var q = terms[i];
      var inTitle = title.indexOf(q);
      var inDesc = desc.indexOf(q);
      var inBody = body.indexOf(q);
      if (inTitle < 0 && inDesc < 0 && inBody < 0) return 0;
      if (inTitle === 0) total += 200;
      else if (inTitle > 0) total += 120;
      if (inDesc >= 0) total += 40;
      if (inBody >= 0) total += 20;
    }
    return total;
  }

  /* A window of body text around the first hit, so the result shows why it
     matched rather than just that it did. */
  function snippet(text, term) {
    var at = text.toLowerCase().indexOf(term);
    if (at < 0) return "";
    var from = Math.max(0, at - 60);
    var to = Math.min(text.length, at + term.length + 90);
    var cut = text.slice(from, to);
    return (from > 0 ? "…" : "") + cut + (to < text.length ? "…" : "");
  }

  function mark(text, terms) {
    var out = esc(text);
    for (var i = 0; i < terms.length; i++) {
      var q = terms[i];
      if (!q) continue;
      var lower = out.toLowerCase();
      var built = "";
      var at;
      var rest = out;
      var restLower = lower;
      while ((at = restLower.indexOf(q)) >= 0) {
        built += rest.slice(0, at) + "<mark>" + rest.slice(at, at + q.length) + "</mark>";
        rest = rest.slice(at + q.length);
        restLower = rest.toLowerCase();
      }
      out = built + rest;
    }
    return out;
  }

  function render() {
    var raw = input.value.trim();
    if (!raw) {
      list.innerHTML = "";
      status.textContent = T.empty;
      return;
    }
    if (!rows) { load(); return; }

    var terms = raw.toLowerCase().split(/\s+/);
    var hits = [];
    for (var i = 0; i < rows.length; i++) {
      var s = score(rows[i], terms);
      if (s > 0) hits.push({ row: rows[i], score: s });
    }
    hits.sort(function (a, b) { return b.score - a.score; });
    hits = hits.slice(0, 12);

    status.textContent = hits.length ? T.count(hits.length) : T.none + ' "' + raw + '"';
    active = 0;
    list.innerHTML = hits
      .map(function (h, i) {
        var r = h.row;
        var body = snippet(r.c, terms[0]) || r.d || "";
        return (
          '<li><a href="' + esc(r.u) + '"' + (i === 0 ? ' aria-selected="true"' : "") + ">" +
          (r.s ? '<span class="search-crumb">' + esc(r.s) + "</span>" : "") +
          '<span class="search-title">' + mark(r.t, terms) + "</span>" +
          '<span class="search-snippet">' + mark(body, terms) + "</span>" +
          "</a></li>"
        );
      })
      .join("");
  }

  function move(delta) {
    var items = list.querySelectorAll("a");
    if (!items.length) return;
    items[active].removeAttribute("aria-selected");
    active = (active + delta + items.length) % items.length;
    items[active].setAttribute("aria-selected", "true");
    items[active].scrollIntoView({ block: "nearest" });
  }

  function open() {
    if (dialog.open) return;
    dialog.showModal();
    input.value = "";
    render();
    load();
    input.focus();
  }

  trigger.addEventListener("click", open);

  document.addEventListener("keydown", function (e) {
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
      e.preventDefault();
      dialog.open ? dialog.close() : open();
    }
  });

  input.addEventListener("input", render);

  dialog.addEventListener("keydown", function (e) {
    if (e.key === "ArrowDown") { e.preventDefault(); move(1); }
    else if (e.key === "ArrowUp") { e.preventDefault(); move(-1); }
    else if (e.key === "Enter") {
      var items = list.querySelectorAll("a");
      if (items.length) { e.preventDefault(); location.href = items[active].href; }
    }
  });

  /* Clicking the backdrop closes. A <dialog>'s backdrop is part of the dialog
     element, so the target being the dialog itself means the click missed the
     panel inside it. */
  dialog.addEventListener("click", function (e) {
    if (e.target === dialog) dialog.close();
  });

  /* The shortcut hint is written here, not in the template, because it is a
     property of the reader's keyboard rather than of the page. */
  var mac = /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);
  trigger.querySelector("kbd").textContent = mac ? "⌘K" : "Ctrl K";
})();
