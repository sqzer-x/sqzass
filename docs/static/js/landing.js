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
  /* 복사 버튼은 클래스가 아니라 data-copy-text로 찾는다. 클래스로 찾으면 새
     버튼마다 아이콘 전용 스타일(.inst-copy는 2rem 정사각형)을 같이 받게 된다. */
  document.querySelectorAll("[data-copy-text]").forEach(function (btn) {
    var label = btn.getAttribute("aria-label");
    /* 글자를 달고 있는 버튼은 그 글자가 바뀌어야 눌린 걸 안다. 아이콘만 있는
       버튼은 aria-label만 바꾼다 — 볼 게 없으니 들릴 것이라도 바뀌어야 한다. */
    var text = btn.querySelector(".copy-label");
    var was = text ? text.textContent : null;
    btn.addEventListener("click", function () {
      if (!navigator.clipboard) return;
      navigator.clipboard.writeText(btn.getAttribute("data-copy-text") || "").then(function () {
        btn.classList.add("is-done");
        if (label) btn.setAttribute("aria-label", btn.getAttribute("data-copied"));
        if (text) text.textContent = btn.getAttribute("data-copied");
        setTimeout(function () {
          btn.classList.remove("is-done");
          if (label) btn.setAttribute("aria-label", label);
          if (text) text.textContent = was;
        }, 1600);
      });
    });
  });

  var M = window.Motion;
  var REDUCED = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  /* 한 번만 판정한다. 도중에 바뀌면 Motion이 관리하던 transform과 우리가 직접
     쓴 transform이 한 엘리먼트에서 섞이는데, 그 상태는 새로고침이 고친다. */
  var CAN_ANIMATE = !!(M && M.animate) && !REDUCED;
  var EASE = [0.16, 1, 0.3, 1];

  /* 설치 채널 전환.

     선택 자체는 radio + CSS가 이미 하고 있다. 표시자는 그 위에 얹는 것이지만
     **연출이 아니라 상태 표시**다 — 어느 탭이 선택됐는지를 말한다. 그래서
     reduced-motion이거나 Motion이 아예 없어도 제자리는 찾아가야 하고, 빠지는
     것은 미끄러짐과 squeeze뿐이다.

     squeeze는 장식이 아니라 이 제품의 동작이다. 히어로의 강조 줄이 옆으로
     눌려 들어오는 것과 같은 스프링을 쓴다. */
  var install = document.querySelector(".install");
  if (install) {
    var marker = install.querySelector(".pick-marker");
    var tabs = [].slice.call(install.querySelectorAll(".pick-tab"));
    var radios = [].slice.call(install.querySelectorAll(".pick-in"));
    var panels = [].slice.call(install.querySelectorAll(".pick-panel"));

    /* 위치를 잡는 두 경로. 한 엘리먼트에 둘을 섞지 않는다 — Motion은 transform을
       자기 방식으로 조립하므로, 직접 쓴 transform과 번갈아 쓰면 어긋난다. */
    var put = function (x, w, animated) {
      if (!CAN_ANIMATE) {
        marker.style.width = w + "px";
        marker.style.transform = "translateX(" + x + "px)";
        return;
      }
      /* x와 scaleX를 따로 준다. transform 문자열을 두 번 애니메이션하면 뒤엣것이
         앞엣것을 덮어 이동이 사라진다. */
      if (!animated) {
        M.animate(marker, { width: w + "px", x: x, scaleX: 1 }, { duration: 0 });
        return;
      }
      /* 이동하는 동안 눌렸다 펴진다. */
      M.animate(marker, { width: w + "px", x: x }, { duration: 0.34, ease: EASE });
      M.animate(marker, { scaleX: [0.72, 1] },
        { type: "spring", stiffness: 260, damping: 18 });
    };

    var place = function (animated) {
      var i = radios.findIndex(function (r) { return r.checked; });
      if (i < 0) return;
      put(tabs[i].offsetLeft, tabs[i].offsetWidth, animated);
      install.classList.add("is-enhanced");
      if (animated && CAN_ANIMATE) {
        M.animate(panels[i], { opacity: [0, 1], scaleX: [1.05, 1] },
          { type: "spring", stiffness: 240, damping: 22 });
      }
    };

    place(false);
    radios.forEach(function (r) {
      r.addEventListener("change", function () { place(true); });
    });
    /* 폰트가 늦게 붙으면 탭 폭이 달라진다. 그때 표시자가 어긋난 채로 남는다. */
    if (document.fonts && document.fonts.ready) {
      document.fonts.ready.then(function () { place(false); });
    }
    window.addEventListener("resize", function () { place(false); });
  }

  /* 여기부터는 순수한 연출이다. 없어도 페이지는 완성 상태이므로, Motion이
     없거나 사용자가 모션을 줄여 달라고 했으면 여기서 끝낸다. */
  if (!CAN_ANIMATE || !M.inView) return;

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
  document.querySelectorAll(".bento-wrap [data-reveal]").forEach(function (el) {
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
