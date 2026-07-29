+++
title = "벤치마크"
description = "생성기 다섯, 머신 하나, 코퍼스 하나 — 그리고 정의는 전부 공개"
weight = 60
toc = true
+++

같은 1,000페이지 사이트를 같은 머신에서 다섯 생성기로 콜드 빌드했습니다.
짧을수록 좋습니다. sqzass 수치에는 기본으로 내보내는 모든 것 — 검색 색인,
sitemap, robots.txt, llms.txt — 이 포함돼 있고, 다른 도구는 이걸 만들지
않습니다.

수치는 특정 머신 한 대에서 잰 콜드 3회의 중앙값입니다. 밀리초보다 비율이
더 멀리 갑니다. 생성기와 하네스가
[`bench/`](https://github.com/sqzer-x/sqzass/tree/main/bench)에 있으니 직접
돌려 보세요.

## minimal — 제목 하나와 문단 하나

<div class="bench">
<span class="b-name">sqzass</span><span class="b-row"><svg><rect class="b-us" width="0.6%" height="100%" rx="3"></rect></svg><span class="b-val">18 ms</span></span>
<span class="b-name">Hugo</span><span class="b-row"><svg><rect width="2%" height="100%" rx="3"></rect></svg><span class="b-val">93 ms</span></span>
<span class="b-name">Zola</span><span class="b-row"><svg><rect width="3%" height="100%" rx="3"></rect></svg><span class="b-val">141 ms</span></span>
<span class="b-name">Jekyll</span><span class="b-row"><svg><rect width="9.5%" height="100%" rx="3"></rect></svg><span class="b-val">442 ms</span></span>
<span class="b-name">Astro</span><span class="b-row"><svg><rect width="100%" height="100%" rx="3"></rect></svg><span class="b-val">4,672 ms</span></span>
</div>

## blog — 문단 여섯, 목록, 인용, 링크

<div class="bench">
<span class="b-name">sqzass</span><span class="b-row"><svg><rect class="b-us" width="0.6%" height="100%" rx="3"></rect></svg><span class="b-val">25 ms</span></span>
<span class="b-name">Hugo</span><span class="b-row"><svg><rect width="1.7%" height="100%" rx="3"></rect></svg><span class="b-val">112 ms</span></span>
<span class="b-name">Zola</span><span class="b-row"><svg><rect width="2.6%" height="100%" rx="3"></rect></svg><span class="b-val">172 ms</span></span>
<span class="b-name">Jekyll</span><span class="b-row"><svg><rect width="11.2%" height="100%" rx="3"></rect></svg><span class="b-val">752 ms</span></span>
<span class="b-name">Astro</span><span class="b-row"><svg><rect width="100%" height="100%" rx="3"></rect></svg><span class="b-val">6,699 ms</span></span>
</div>

## heavy·반복 — 페이지마다 동일한 20줄 Rust 블록 5개

<div class="bench">
<span class="b-name">sqzass</span><span class="b-row"><svg><rect class="b-us" width="0.85%" height="100%" rx="3"></rect></svg><span class="b-val">187 ms</span></span>
<span class="b-name">Hugo</span><span class="b-row"><svg><rect width="16.1%" height="100%" rx="3"></rect></svg><span class="b-val">3,559 ms</span></span>
<span class="b-name">Zola</span><span class="b-row"><svg><rect width="23%" height="100%" rx="3"></rect></svg><span class="b-val">5,084 ms</span></span>
<span class="b-name">Jekyll</span><span class="b-row"><svg><rect width="49.1%" height="100%" rx="3"></rect></svg><span class="b-val">10,850 ms</span></span>
<span class="b-name">Astro</span><span class="b-row"><svg><rect width="100%" height="100%" rx="3"></rect></svg><span class="b-val">22,090 ms</span></span>
</div>

## heavy·고유 — 페이지마다 전부 다른 20줄 Rust 블록 5개

<div class="bench">
<span class="b-name">sqzass</span><span class="b-row"><svg><rect class="b-us" width="5.4%" height="100%" rx="3"></rect></svg><span class="b-val">1,298 ms</span></span>
<span class="b-name">Hugo</span><span class="b-row"><svg><rect width="24.7%" height="100%" rx="3"></rect></svg><span class="b-val">5,912 ms</span></span>
<span class="b-name">Zola</span><span class="b-row"><svg><rect width="28.3%" height="100%" rx="3"></rect></svg><span class="b-val">6,789 ms</span></span>
<span class="b-name">Jekyll</span><span class="b-row"><svg><rect width="47.8%" height="100%" rx="3"></rect></svg><span class="b-val">11,456 ms</span></span>
<span class="b-name">Astro</span><span class="b-row"><svg><rect width="100%" height="100%" rx="3"></rect></svg><span class="b-val">23,958 ms</span></span>
</div>

## 전체 수치

벽시계 시간(콜드 3회 중앙값)과 peak RSS입니다.

| | minimal | blog | heavy·반복 | heavy·고유 |
|---|---|---|---|---|
| **sqzass 0.1.0** | **18 ms** · 15 MB | **25 ms** · 24 MB | **187 ms** · 229 MB | **1,298 ms** · 439 MB |
| Hugo 0.164.0 | 93 ms · 123 MB | 112 ms · 140 MB | 3,559 ms · 411 MB | 5,912 ms · 421 MB |
| Zola 0.22.1 | 141 ms · 136 MB | 172 ms · 142 MB | 5,084 ms · 269 MB | 6,789 ms · 279 MB |
| Jekyll 4.4.1 | 442 ms · 56 MB | 752 ms · 62 MB | 10,850 ms · 174 MB | 11,456 ms · 182 MB |
| Astro 5.18.2 | 4,672 ms · 555 MB | 6,699 ms · 604 MB | 22,090 ms · 2,578 MB | 23,958 ms · 2,607 MB |

## heavy가 둘인 이유

코드 반복도가 순위를 가르는데, 대부분의 벤치마크는 자기 코퍼스의 반복도를
말하지 않기 때문입니다. sqzass는 서로 다른 블록만 빌드에서 한 번씩
하이라이트하므로, 같은 블록을 반복하는 생성 코퍼스와 전부 다른 블록인
코퍼스는 서로 다른 측정입니다 — 187 ms와 1,298 ms의 간격이 바로 그
변수입니다. 반복도를 밝히지 않은 단일 "heavy" 수치는 결과를 만든 변수를
숨긴 것입니다.

## 방법

- **콜드.** 매 회 출력 디렉터리와 도구별 캐시 — `.jekyll-cache`, Astro의
  `cacheDir`와 `dist`, Hugo의 `resources`와 빌드 락 — 를 지웠습니다. 벽시계
  시간과 peak RSS는 GNU time으로 쟀습니다.
- **코퍼스.** 1,000페이지 + 섹션 인덱스. 마크다운 *본문*은 도구 간 바이트
  동일이고 front matter 문법만 다릅니다. 문단은 76자 문장 하나를 네 번
  반복한 고정 텍스트입니다.
- **하이라이팅은 다섯 도구 모두 빌드 타임에**, 각자 싣는 그대로 켰습니다:
  syntect+Oniguruma(sqzass), Chroma(Hugo), Giallo(Zola), Rouge(Jekyll),
  Shiki(Astro). 마크업 세분도는 다릅니다 — 같은 heavy 페이지의 span 수:
  sqzass 3,205 · Jekyll 1,800 · Zola·Astro 1,700 · Hugo 600. sqzass는 다섯 중
  가장 세밀한 마크업을 내면서 위의 수치를 냅니다.
- **템플릿.** 다섯 도구 모두 테마 없는 동일한 최소 레이아웃 — 렌더된 본문을
  감싸는 HTML 셸 — 을 씁니다.
- **Astro의 시간에는 Node 기동(~1초)이 포함**됩니다. Astro는 마크다운→HTML
  이상의 일을 하는 컴포넌트 프레임워크이기도 합니다. 그게 정직한 단서이고,
  이 페이지의 대상은 순수 마크다운 워크로드입니다.

## 이 페이지가 주장하지 않는 것

당신의 머신은 다른 밀리초를 냅니다. 증분 빌드와 개발 서버는 재지 않았습니다.
기능 범위도 서로 다릅니다 — 여기서 잰 건 하나입니다: 마크다운 디렉터리를
HTML 디렉터리로, 콜드로, 각 도구의 기본값으로 바꾸는 일. 위의 정의와
[`bench/`](https://github.com/sqzer-x/sqzass/tree/main/bench)의 스크립트가
방법론의 전부입니다. 수치가 이상해 보이면 직접 돌려 보세요.
