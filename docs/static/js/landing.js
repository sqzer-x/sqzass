/* 랜딩 연출·복사 버튼. 전부 점진적 향상이다 — Motion이 없거나
   reduced-motion이면 연출은 생략되고, 페이지는 CSS만으로 완성 상태다. */
(function () {
  "use strict";

  /* 설치 한 줄 복사. 연출과 무관하므로 reduced-motion과 상관없이 동작한다. */
  document.querySelectorAll(".inst-copy").forEach(function (btn) {
    var label = btn.getAttribute("aria-label");
    btn.addEventListener("click", function () {
      var text = btn.getAttribute("data-copy-text") || "";
      if (!navigator.clipboard) return;
      navigator.clipboard.writeText(text).then(function () {
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

  /* 히어로: 로드 시퀀스. 위에서 아래로 한 호흡씩. */
  var heroEls = document.querySelectorAll(".hero [data-reveal]");
  heroEls.forEach(function (el) { el.style.opacity = "0"; });
  M.animate(
    heroEls,
    { opacity: [0, 1], transform: ["translateY(16px)", "none"] },
    { duration: 0.65, ease: EASE, delay: M.stagger(0.09) }
  );

  /* 아트워크: 등장 후 느린 부유. 물속이라는 뜻이지 장식 경련이 아니다. */
  var art = document.querySelector("[data-float]");
  if (art) {
    M.animate(art, { opacity: [0, 1], transform: ["scale(0.96)", "scale(1)"] },
      { duration: 1.1, ease: EASE });
    M.animate(art,
      { transform: ["translateY(0)", "translateY(-12px)", "translateY(0)"] },
      { duration: 9, repeat: Infinity, ease: [0.45, 0, 0.55, 1], delay: 1.1 });
  }

  /* 나머지 섹션: 뷰포트에 들어올 때 한 번씩. */
  var rest = document.querySelectorAll("main > :not(.hero) [data-reveal]");
  rest.forEach(function (el) {
    el.style.opacity = "0";
    M.inView(
      el,
      function () {
        M.animate(
          el,
          { opacity: [0, 1], transform: ["translateY(16px)", "none"] },
          { duration: 0.55, ease: EASE }
        );
      },
      { amount: 0.25 }
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
          { duration: 0.85, ease: EASE, delay: M.stagger(0.12) }
        );
      },
      { amount: 0.5 }
    );
  });
})();
