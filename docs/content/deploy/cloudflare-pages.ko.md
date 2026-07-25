+++
title = "Cloudflare Pages"
description = "대시보드 설정 두 개, 그리고 GitHub Pages가 못 주던 헤더"
weight = 40
toc = true
+++

프로젝트 빌드 설정에서:

| | |
|---|---|
| 프레임워크 프리셋 | 없음 |
| 빌드 명령 | `curl -sSL https://github.com/sqzer-x/sqzass/releases/latest/download/sqzass-x86_64-unknown-linux-musl.tar.gz \| tar xz --strip-components=1 && ./sqzass build` |
| 빌드 출력 디렉터리 | `public` |

작성할 설정 파일이 없습니다.

## 헤더가 필요해지면 옮겨 갈 곳이 여기입니다

sqzass는 호스트 설정에 아무것도 기대지 않도록 만들었고, 이 사이트를 GitHub Pages에
둔 것도 그 약속을 정직하게 지키기 위해서입니다. 다만 GitHub Pages는 전부
`cache-control: max-age=600`으로 내보내고 그걸 바꿀 방법이 없습니다.

Cloudflare Pages는 `_headers` 파일을 읽으므로, 콘텐츠 해시가 붙은 파일명이 드디어
제 뜻대로 쓰입니다.

```
# static/_headers
/css/*
  Cache-Control: public, max-age=31536000, immutable
/js/*
  Cache-Control: public, max-age=31536000, immutable
/*
  Cache-Control: public, max-age=600
```

이 두 디렉터리에 1년을 걸어도 안전한 이유는, 내용이 바뀌면 파일명이 바뀌기
때문입니다. 쿼리 문자열이 아니라 이름에 해시를 넣은 것의 보상이고, 헤더를 못 바꾸는
호스트에서는 쓸 수 없는 보상입니다.

파일을 `static/`에 두면 그대로 복사됩니다.

## 리다이렉트

`_redirects`도 같은 방식으로 동작하지만, front matter의 `aliases`를 먼저 보세요.
옮겨진 페이지 바로 옆에 있고, 빌드가 검사하며, 다음에 어떤 호스트로 가든 따라옵니다.

## 프리뷰 배포

`$CF_PAGES_URL`에 프리뷰 주소가 들어 있습니다.

```bash
./sqzass build --base-url "$CF_PAGES_URL"
```
