+++
title = "콘텐츠 작성"
description = "페이지와 섹션, front matter, 그리고 거기서 나오는 URL"
weight = 20
sort_by = "weight"
+++

`content/` 아래의 모든 파일이 페이지입니다. `_index.md`가 들어 있는 디렉터리는
섹션이 되고, 섹션은 그 옆의 페이지들을 묶습니다.

## 페이지와 섹션

```
content/
├── _index.md            →  /
├── about.md             →  /about/
└── guide/
    ├── _index.md        →  /guide/
    ├── install.md       →  /guide/install/
    └── deep/
        ├── _index.md    →  /guide/deep/
        └── dive.md      →  /guide/deep/dive/
```

`_index.md`가 **없는** 디렉터리도 그 안의 페이지 URL은 그대로 만들어집니다.
다만 섹션은 아닙니다. 그 페이지들을 묶어 주는 것이 없고, 디렉터리 자체를 가리키는
내비게이션 항목도 생기지 않습니다.

## 모든 URL이 디렉터리인 이유

페이지는 `<경로>.html`이 아니라 `<경로>/index.html`로 쓰입니다. rewrite 규칙이
있는 호스트라면 `about.html`을 `/about`으로 내줄 수 있지만, 규칙이 없는 호스트는
못 합니다. sqzass는 아무것도 해 주지 않는 호스트에서 옳게 도는 것을 기준으로
만들었습니다. 디렉터리 형태는 서버에게 아무 재주도 요구하지 않기 때문에 GitHub
Pages든 Cloudflare Pages든 S3든 `python3 -m http.server`든 똑같이 동작합니다.

대가는 있습니다. 슬래시 없는 `/about` 링크는 대부분의 서버가 리다이렉트를 한 번
거친 뒤에야 도착합니다. `/about/`으로 쓰거나, 더 나은 방법으로는
[`@/` 링크](@/content/links.md)를 써서 URL을 sqzass가 쓰게 하세요.

## 슬러그는 제목이 아니라 파일명에서 나옵니다

`install.md`는 `/guide/install/`이 됩니다. 제목은 여기에 관여하지 않습니다.

이 규칙이 가장 중요해지는 곳이 한국어입니다. 흔한 방법은 제목을 로마자로 옮기는
것인데, `slug` 계열 크레이트는 한글을 문맥 없이 한 음절씩 ASCII로 매핑하기 때문에
서로 다른 한국어 제목이 같은 경로로 겹칠 수 있습니다. 직접 정한 파일명은 모호하지
않고, 같은 디렉터리 안에서 이미 유일합니다. 한국어 파일명은 퍼센트 인코딩된
UTF-8 그대로 둡니다.

페이지마다 바꾸고 싶으면 front matter의 `slug`로 덮어쓰면 됩니다. **두 페이지가
같은 URL을 주장하면 빌드 에러입니다.** 나중에 쓴 쪽이 이긴다는 식으로 처리하지
않습니다.

## 정렬

섹션은 자식 페이지를 `weight` 오름차순으로 정렬하고, `weight`가 없으면 제목으로
넘어갑니다. 섹션의 `_index.md`에 `sort_by`를 주거나, `sqzass.toml`의
`[nav] sort_by`로 사이트 전체 기본값을 바꿀 수 있습니다.

| | |
|---|---|
| `weight` | 오름차순. 기본값. |
| `title` | 오름차순. |
| `date` | **내림차순** — 최신이 먼저, 날짜 없는 페이지는 뒤. [피드](@/features/feeds.md) 참고. |

## 드래프트

`draft = true`인 페이지는 빌드에서 빠집니다. 명령줄의 `--drafts`나 설정의
`[build] drafts = true`로 다시 넣을 수 있습니다.
