/* 랜딩 연출. 전부 점진적 향상이다 — Motion이 없거나 reduced-motion이면
   아무것도 하지 않고, 페이지는 CSS만으로 완성 상태다. */
(function () {
  "use strict";
  var M = window.Motion;
  if (!M || !M.animate || !M.inView) return;
  if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;

  var EASE = [0.16, 1, 0.3, 1];

  /* 히어로: 로드 시퀀스. 위에서 아래로 한 호흡씩. */
  var heroEls = document.querySelectorAll(".hero [data-reveal]");
  heroEls.forEach(function (el) { el.style.opacity = "0"; });
  M.animate(
    heroEls,
    { opacity: [0, 1], transform: ["translateY(14px)", "none"] },
    { duration: 0.6, ease: EASE, delay: M.stagger(0.09) }
  );

  /* 나머지 섹션: 뷰포트에 들어올 때 한 번씩. */
  var rest = document.querySelectorAll("main > :not(.hero) [data-reveal]");
  rest.forEach(function (el) {
    el.style.opacity = "0";
    M.inView(
      el,
      function () {
        M.animate(
          el,
          { opacity: [0, 1], transform: ["translateY(14px)", "none"] },
          { duration: 0.55, ease: EASE }
        );
      },
      { amount: 0.3 }
    );
  });

  /* 벤치마크 막대: 실척 폭은 SVG 속성이 이미 갖고 있고, 여기서는 왼쪽에서
     자라는 등장만 준다. 값이 아니라 등장이 연출이다. */
  document.querySelectorAll(".row").forEach(function (row) {
    var bars = row.querySelectorAll("[data-bar]");
    bars.forEach(function (b) { b.style.transform = "scaleX(0)"; });
    M.inView(
      row,
      function () {
        M.animate(
          bars,
          { transform: ["scaleX(0)", "scaleX(1)"] },
          { duration: 0.8, ease: EASE, delay: M.stagger(0.12) }
        );
      },
      { amount: 0.5 }
    );
  });
})();
