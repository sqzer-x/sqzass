+++
title = "sqzass"
description = "Rust로 만든 정적 사이트 생성기 — 빠르고 결정적입니다"
weight = 0
template = "landing.html"

[extra]
hero_kicker = "Rust로 만든 정적 사이트 생성기"
hero_title_a = "천 페이지를"
hero_title_b = "25밀리초로 쥐어짭니다."
hero_sub = "이름 그대로입니다. 사이트 전체를 한 번에 짜냅니다 — Hugo·Zola·Jekyll·Astro와 같은 머신에서 직접 재고, 측정 방법도 전부 공개해 뒀습니다."
cta_start = "시작하기"
cta_github = "GitHub에서 보기"
install_alt = "Rust가 있다면 이렇게도 됩니다:"
install_aur = "Arch라면:"

bento_label = "왜 sqzass인가"

bench_title = "주장이 아니라 측정입니다"
bench_note = "비교 대상은 넷 중 가장 빨랐던 Hugo입니다. 같은 머신에서 캐시 없이 세 번씩 재서 중앙값을 적었고, 막대 길이는 실제 비율입니다. sqzass 쪽 시간에는 검색 색인·sitemap·llms.txt 생성까지 포함돼 있습니다."
bench_more = "전체 수치와 측정 방법 →"
zero_label = "같은 입력을 두 번 빌드했을 때 생기는 바이트 차이입니다. push마다 CI가 두 번 빌드해서 비교합니다."

foot_benchmark = "벤치마크"

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
label = "코드 많은 코퍼스, 모든 블록이 서로 다름"
us_ms = "1,298"
us_w = "22.0%"
them_ms = "5,912"
them_w = "100%"

[[extra.features]]
title = "바이너리 하나면 됩니다"
body = "7.3 MB 정적 바이너리 하나에 라이브 리로드 개발 서버까지 들어 있습니다. 따로 설치할 런타임도, 버전을 맞춰야 할 플러그인도 없습니다."

[[extra.features]]
title = "깨진 참조는 빌드를 멈춥니다"
body = "깨진 링크, 없는 템플릿, front matter 오타를 만나면 경고만 띄우고 지나가지 않습니다. 빌드를 멈추고 어느 파일 몇째 줄이 문제인지, 어떻게 고치면 되는지 알려 줍니다. 에러마다 고정된 식별자와 exit code가 있어서 스크립트로 다루기도 쉽습니다."
+++

대부분의 생성기는 예쁜 URL, 리다이렉트, 캐시 헤더 같은 일을 호스트에 맡깁니다.
sqzass는 이걸 직접 처리합니다. 그래서 GitHub Pages에 올리든, Cloudflare Pages에
올리든, 그냥 디렉터리째 서빙하든 같은 결과물이 그대로 동작합니다.

속도는 형용사가 아니라 측정값으로 말합니다. 문제가 생기면 조용히 넘어가는 대신
빌드를 멈추고, 어느 파일 몇째 줄이 왜 문제인지 짚어 줍니다.
