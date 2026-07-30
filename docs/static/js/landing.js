/* 랜딩: 테마 토글·복사 버튼·연출. 연출은 전부 점진적 향상 — Motion이 없거나
   reduced-motion이면 생략되고, 페이지는 CSS만으로 완성 상태다. */
(function () {
  "use strict";

  /* 테마 토글. docs와 같은 localStorage 키("theme")를 쓰므로 사이트 전체에서
     선택이 따라다닌다. */
  var toggle = document.querySelector(".theme-toggle");
  if (toggle) {
    toggle.addEventListener("click", function () {
      var root = document.documentElement;
      var next = root.dataset.theme === "dark" ? "light" : "dark";
      root.dataset.theme = next;
      try { localStorage.setItem("theme", next); } catch (e) {}
    });
  }

  /* 설치 한 줄 복사 */
  document.querySelectorAll(".inst-copy").forEach(function (btn) {
    var label = btn.getAttribute("aria-label");
    btn.addEventListener("click", function () {
      if (!navigator.clipboard) return;
      navigator.clipboard.writeText(btn.getAttribute("data-copy-text") || "").then(function () {
        btn.classList.add("is-done");
        btn.setAttribute("aria-label", btn.getAttribute("data-copied"));
        setTimeout(function () {
          btn.classList.remove("is-done");
          btn.setAttribute("aria-label", label);
        }, 1600);
      });
    });
  });

  var M = window.Motion;
  if (!M || !M.animate || !M.inView) return;
  if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;

  var EASE = [0.16, 1, 0.3, 1];

  /* 히어로 로드 시퀀스 */
  var heroEls = document.querySelectorAll(".hero [data-reveal]");
  heroEls.forEach(function (el) { el.style.opacity = "0"; });
  M.animate(
    heroEls,
    { opacity: [0, 1], transform: ["translateY(16px)", "none"] },
    { duration: 0.6, ease: EASE, delay: M.stagger(0.08) }
  );

  /* 이름값 하는 연출: 강조 줄이 옆으로 눌려 들어온다(squeeze). 스프링이
     안 되는 빌드면 조용히 생략 — 등장 자체는 위 시퀀스가 이미 보장한다. */
  var em = document.querySelector("[data-squeeze]");
  if (em) {
    try {
      M.animate(
        em,
        { transform: ["scaleX(1.35)", "scaleX(1)"] },
        { type: "spring", stiffness: 170, damping: 15, delay: 0.35 }
      );
    } catch (e) {}
  }

  /* 벤토 타일: 뷰포트에 들어올 때 한 번씩 */
  document.querySelectorAll(".bento-wrap [data-reveal], .foot [data-reveal]").forEach(function (el) {
    el.style.opacity = "0";
    M.inView(
      el,
      function () {
        M.animate(
          el,
          { opacity: [0, 1], transform: ["translateY(14px)", "none"] },
          { duration: 0.5, ease: EASE }
        );
      },
      { amount: 0.2 }
    );
  });

  /* 벤치 막대: 왼쪽에서 자라는 등장. 실척 폭은 SVG 속성이 이미 갖고 있다. */
  document.querySelectorAll(".row").forEach(function (row) {
    var bars = row.querySelectorAll("[data-bar]");
    bars.forEach(function (b) { b.style.transform = "scaleX(0)"; });
    M.inView(
      row,
      function () {
        M.animate(
          bars,
          { transform: ["scaleX(0)", "scaleX(1)"] },
          { duration: 0.8, ease: EASE, delay: M.stagger(0.1) }
        );
      },
      { amount: 0.5 }
    );
  });
})();
