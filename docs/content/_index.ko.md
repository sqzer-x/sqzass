+++
title = "sqzass"
description = "Rust로 만든 정적 사이트 생성기"
weight = 0
+++

**sqzass**는 Rust로 만든 정적 사이트 생성기다. 이 사이트가 sqzass로 만들어졌다.

## 왜 또 만드나

대부분의 생성기는 예쁜 URL·리다이렉트·캐시 헤더를 호스트에 맡긴다.
sqzass는 그걸 직접 한다. 그래서 같은 출력물이 GitHub Pages든 Vercel이든
Cloudflare Pages든, 그냥 디렉터리를 HTTP로 서빙하든 똑같이 동작한다.

- 마크다운은 [comrak](https://github.com/kivikakk/comrak). 진짜 AST를 쓴다 —
  링크 재작성과 heading anchor를 트리에서 처리하지, 최종 HTML에 정규식을
  돌리지 않는다.
- 템플릿은 [minijinja](https://github.com/mitsuhiko/minijinja). 기본으로
  이스케이프하고, 스냅샷에서 해석하므로 저장 도중 리빌드가 걸려도 반쯤 쓰인
  파셜을 읽는 일이 없다.
- 처음부터 이중 언어.

## 상태

초기 단계. [시작하기](@/start/_index.md)를 보라.
