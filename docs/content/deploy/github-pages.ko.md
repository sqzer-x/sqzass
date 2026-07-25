+++
title = "GitHub Pages"
description = "이 사이트가 배포되는 워크플로"
weight = 10
toc = true
+++

이 사이트는 도구와 같은 저장소에서 아래 워크플로로 빌드·배포됩니다. CI가 방금
컴파일한 바이너리로 `docs/`를 빌드하므로, 이 문서가 곧 생성기의 회귀 테스트가 됩니다.

## 워크플로

```yaml
name: Deploy docs

on:
  push:
    branches: [main]

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: pages
  cancel-in-progress: false

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: actions/configure-pages@v6
      - run: cargo run --quiet -- build -i docs
      - uses: actions/upload-pages-artifact@v5
        with:
          path: docs/public

  deploy:
    needs: build
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - id: deployment
        uses: actions/deploy-pages@v5
```

`cancel-in-progress: false`는 그대로 두는 게 좋습니다. 진행 중인 배포를 취소하면
사이트가 반쯤 갱신된 상태로 남는데, 그건 기다리는 것보다 나쁩니다.

## 커스텀 도메인

도메인을 `static/CNAME`에 넣습니다.

```
sqzass.sqzer.com
```

이름 그대로 출력에 복사되고 GitHub이 거기서 읽습니다. 그래서 배포할 때마다 저장소
설정에서 도메인을 다시 넣지 않아도 살아남습니다.

DNS는 `CNAME` 레코드로 `<사용자>.github.io`를 가리키게 합니다. DNS 제공자가 트래픽을
프록시한다면 **이 레코드에서는 프록시를 끄세요.** 프록시가 TLS를 자기가 종료하면
GitHub이 인증서를 발급·갱신하지 못하는데, 이 실패는 배포가 깨지는 형태로 바로
드러나지 않고 몇 주 뒤 인증서 만료로 나타납니다.

## 확인하기

```bash
curl -sI https://example.com/ | head -1          # HTTPS로 200
curl -sI https://example.com/nonexistent/        # 200이 아니라 404
```

두 번째가 중요합니다. 없는 페이지에 200을 돌려주는 호스트에서는 검색 엔진이 당신의
404 페이지를 색인하게 됩니다.

## 감수하는 것

커스텀 헤더가 없습니다. GitHub Pages는 전부 `max-age=600`으로 내보냅니다. 리다이렉트
규칙도 없습니다. 풀 리퀘스트 프리뷰 배포도 없습니다.

앞의 둘은 출력물을 지금처럼 만든 이유 그 자체이고, 세 번째는 `sqzass serve`가
대신합니다. 나중에 정말로 헤더 제어가 필요해지면, 답은 헤더가 없는 호스트 앞에 CDN을
얹는 게 아니라 헤더가 있는 호스트로 옮기는 것입니다.
