+++
title = "sqzass"
description = "Rust로 만든 정적 사이트 생성기 — 빠르고, 결정적이고, 한국어가 1급입니다"
weight = 0
template = "landing.html"

[extra]
hero_kicker = "Rust로 만든 정적 사이트 생성기"
hero_title_a = "천 페이지,"
hero_title_b = "25밀리초."
hero_sub = "sqzass는 빠르고, 그걸 증명합니다: 생성기 다섯을 같은 머신, 같은 코퍼스로 재고 방법을 공개했습니다. 출력물은 어느 호스트에서든 그대로 동작합니다."
cta_start = "시작하기"
cta_github = "GitHub에서 보기"

bench_title = "주장이 아니라 측정입니다"
bench_note = "각 줄은 우리가 잰 네 생성기 중 가장 빠른 Hugo와 비교합니다 — 같은 머신, 콜드 빌드, 3회 중앙값. 막대는 줄 안에서 실척이고, sqzass 시간에는 검색 색인·sitemap·llms.txt 생성이 포함돼 있습니다."
zero_label = "같은 입력을 두 번 빌드한 출력물의 바이트 차이 — CI가 push마다 다시 빌드해 비교합니다"
bench_more = "전체 수치와 방법론 →"

features_title = "기본으로 얻는 것"
features_more = "나머지는 문서에 있습니다 →"

quick_title = "빠른 시작"
quick_install = "설치 — 어느 줄이든 한 줄로 끝납니다"
quick_use = "사이트 만들기"
quick_note = "스크립트는 플랫폼을 판별하고 체크섬을 검증하며, cargo 줄은 소스에서 빌드합니다. 다른 방법은"
quick_note_link = "설치 안내에 있습니다."

foot_benchmark = "벤치마크"
built_line = "이 사이트도 sqzass가 빌드했습니다 — 문서가 곧 데모입니다."

[[extra.bench]]
label = "1,000페이지, 최소 마크다운"
us_ms = "18"
us_w = "19.4%"
them_ms = "93"
them_w = "100%"

[[extra.bench]]
label = "1,000페이지 블로그 코퍼스"
us_ms = "25"
us_w = "22.3%"
them_ms = "112"
them_w = "100%"

[[extra.bench]]
label = "코드 무거움, 전 블록 고유"
us_ms = "1,298"
us_w = "22.0%"
them_ms = "5,912"
them_w = "100%"

[[extra.features]]
title = "결정적 빌드"
body = "같은 입력은 스레드 수와 무관하게 같은 바이트를 냅니다. 렌더는 병렬이고, 결정성은 병합이 지킵니다."

[[extra.features]]
title = "깨진 참조는 빌드를 멈춥니다"
body = "죽은 링크, 없는 템플릿, front matter 오타 — 전부 안정적인 식별자와 exit code를 가진 에러입니다. 스크롤로 지나치는 경고가 아닙니다."

[[extra.features]]
title = "하이라이팅은 빌드 타임에"
body = "구문 강조는 인라인 style이 아니라 CSS 클래스를 냅니다 — 다크 모드가 싸게 유지되고, 엄격한 CSP도 가능합니다."

[[extra.features]]
title = "한국어를 읽는 검색"
body = "단어 색인 대신 부분 문자열 색인입니다. 조사와 합성어는 형태소 분석기를 이기기 때문이고, 2,000페이지 코퍼스 실측으로 고른 결정입니다."

[[extra.features]]
title = "두 언어를 나란히"
body = "영어와 한국어 페이지가 파일명으로 짝을 이룹니다. 링크는 언어별로 해석되고, 미번역 페이지는 죽은 링크를 남기지 않습니다."

[[extra.features]]
title = "작은 바이너리 하나"
body = "7.3 MB 정적 바이너리에 라이브 리로드 개발 서버까지 들어 있습니다. 설치할 런타임도, 버전 맞출 플러그인도 없습니다."
+++

대부분의 생성기는 예쁜 URL·리다이렉트·캐시 헤더를 호스트에 맡깁니다. sqzass는 그
일을 직접 합니다. 그래서 같은 출력물이 GitHub Pages든 Cloudflare Pages든, 그냥
디렉터리를 HTTP로 서빙하든 똑같이 동작합니다.

속도는 방법을 공개한 실측으로 말하고, 실패는 크게 말합니다: 깨진 참조는 빌드를
멈추고 파일과 줄과 고칠 방법을 짚어 줍니다.
