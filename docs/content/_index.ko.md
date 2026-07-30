+++
title = "sqzass"
description = "Rust로 만든 정적 사이트 생성기 — 빠르고, 결정적이고, 한국어가 1급입니다"
weight = 0
template = "landing.html"

[extra]
hero_kicker = "Rust로 만든 정적 사이트 생성기"
hero_title_a = "천 페이지를"
hero_title_b = "25밀리초로 쥐어짭니다."
hero_sub = "이름이 곧 약속입니다: 사이트 하나를 한 번에 눌러 짜냅니다. Hugo·Zola·Jekyll·Astro와 같은 머신에서 재고, 방법을 전부 공개했습니다."
cta_start = "시작하기"
cta_github = "GitHub에서 보기"

bento_label = "왜 sqzass인가"

bench_title = "주장이 아니라 측정입니다"
bench_note = "우리가 잰 네 생성기 중 가장 빠른 Hugo와 비교합니다 — 같은 머신, 콜드 빌드, 3회 중앙값. 막대는 실척이고, sqzass 시간에는 검색 색인·sitemap·llms.txt 생성이 포함돼 있습니다."
bench_more = "전체 수치와 방법론 →"
zero_label = "같은 입력을 두 번 빌드한 출력물의 바이트 차이 — CI가 push마다 확인합니다"

install_title = "설치는 한 줄"
install_alt = "Rust가 있다면:"

demo_title = "문서가 곧 데모"
demo_body = "이 사이트도 저장소의 docs/를 sqzass가 빌드한 것입니다 — 모든 페이지, 두 언어, 검색 색인과 피드까지요."

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
title = "한국어가 1급입니다"
body = "영어와 한국어 페이지가 파일명으로 짝을 이루고, 링크는 언어별로 해석되며, 검색은 부분 문자열로 찾습니다 — 검색엔진최적화 안의 최적화가 걸리는 유일한 방법입니다."

[[extra.features]]
title = "깨진 참조는 빌드를 멈춥니다"
body = "죽은 링크, 없는 템플릿, front matter 오타 — 전부 안정적인 식별자와 exit code를 가진 에러입니다. 스크롤로 지나치는 경고가 아닙니다."

[[extra.features]]
title = "작은 바이너리 하나"
body = "7.3 MB 정적 바이너리에 라이브 리로드 개발 서버까지 들어 있습니다. 설치할 런타임도, 버전 맞출 플러그인도 없습니다."
+++

대부분의 생성기는 예쁜 URL·리다이렉트·캐시 헤더를 호스트에 맡깁니다. sqzass는 그
일을 직접 합니다. 그래서 같은 출력물이 GitHub Pages든 Cloudflare Pages든, 그냥
디렉터리를 HTTP로 서빙하든 똑같이 동작합니다.

속도는 방법을 공개한 실측으로 말하고, 실패는 크게 말합니다: 깨진 참조는 빌드를
멈추고 파일과 줄과 고칠 방법을 짚어 줍니다.
