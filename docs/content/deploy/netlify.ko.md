+++
title = "Netlify"
description = "netlify.toml, 그리고 base_url이 맞는 배포 프리뷰"
weight = 20
toc = true
+++

```toml
# netlify.toml
[build]
  command   = "curl -sSL https://github.com/sqzer-x/sqzass/releases/latest/download/sqzass-x86_64-unknown-linux-musl.tar.gz | tar xz --strip-components=1 && ./sqzass build"
  publish   = "public"

[context.deploy-preview]
  command   = "curl -sSL https://github.com/sqzer-x/sqzass/releases/latest/download/sqzass-x86_64-unknown-linux-musl.tar.gz | tar xz --strip-components=1 && ./sqzass build --base-url $DEPLOY_PRIME_URL"
```

설정은 이게 전부입니다. 플러그인도, 빌드 이미지도, 런타임도 없습니다.

## 왜 설치가 아니라 바이너리를 받나

Netlify 빌드 이미지에는 Rust 툴체인이 기본으로 없고, 넣으면 빌드마다 몇 분이
더 듭니다. 리눅스 릴리스는 정적 링크라 glibc 버전을 맞추지 않고도 어떤 이미지에서든
돕니다. Alpine에서도, scratch 컨테이너에서도, 빌드한 머신보다 오래된 머신에서도
도는 것과 같은 성질입니다.

소스에서 빌드하고 싶으면 `cargo install --git
https://github.com/sqzer-x/sqzass` 도 됩니다. 다만 느립니다.

## 배포 프리뷰에는 자기 base_url이 필요합니다

프리뷰는 당신의 도메인이 아니라 `deploy-preview-42--yoursite.netlify.app`에서
돕니다. Netlify가 그 주소를 `$DEPLOY_PRIME_URL`에 넣어 주고, `--base-url`이
바로 이 경우를 위해 설정을 덮어씁니다. 그래야 canonical과 sitemap, OpenGraph 태그가
프로덕션이 아니라 프리뷰를 가리킵니다.

이걸 안 해도 프리뷰는 **동작합니다.** 내부 링크가 루트 절대 경로라서 호스트 이름을
신경 쓰지 않기 때문입니다. 깨지는 건 절대 URL 쪽입니다 — `page.permalink`,
`sitemap.xml`, 소셜 태그.

## 예쁜 URL·리다이렉트·헤더

설정할 게 없습니다. 모든 페이지가 `index.html`을 담은 디렉터리라서 Netlify가
rewrite 규칙 없이 `/start/`를 서빙하고, `templates/404.html`이 있으면 `/404.html`을
이름으로 찾아 씁니다.

`_redirects`와 `_headers`는 Netlify 고유 파일이고 둘 다 필수가 아닙니다. 쓰고 싶으면
`static/`에 두면 그대로 복사됩니다 — `static/`은 통과 경로이고, 이름 자체가 계약인
파일은 이름이 유지됩니다.

```
static/
├── _headers
└── _redirects
```

## 필요 없는 것

`netlify-plugin-*`도, `NODE_VERSION`도, `functions` 디렉터리도 없습니다. 결과물은
파일이 든 디렉터리입니다.
