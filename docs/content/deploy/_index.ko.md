+++
title = "배포"
description = "결과물은 디렉터리입니다. 파일을 서빙하는 호스트면 어디든 됩니다."
weight = 50
sort_by = "weight"
+++

```bash
sqzass build -i mysite
```

`mysite/public`이 사이트입니다. 파일을 서빙하는 곳에 복사하면 끝입니다. 런타임도,
서버 컴포넌트도, 남아 있는 빌드 단계도 없습니다.

## 호스트가 하지 않아도 되도록 sqzass가 하는 일

모든 URL이 `index.html`을 담은 디렉터리라서 예쁜 URL에 rewrite 규칙이 필요 없습니다.
캐시 무효화가 파일명에 들어 있어서 캐시 헤더가 필요 없습니다. 출력물 중 어느 것도
호스트 설정에 기대지 않습니다.

의도한 것이고, 이 문서 사이트를 [GitHub Pages](@/deploy/github-pages.md)에 올려 둔
것도 그걸 정직하게 지키기 위해서입니다. Pages에는 커스텀 헤더도, 리다이렉트 규칙도,
rewrite도 없으므로, 그런 것에 기대는 게 생기면 여기서 가장 먼저 깨집니다.

보상은 호스트를 옮기는 일이 복사가 된다는 점입니다. 같은 디렉터리가 Cloudflare Pages
에서도, Netlify에서도, CloudFront 뒤의 S3에서도, nginx에서도 그대로 맞습니다.

## 그 밖에 만들어지는 것

`sitemap.xml`에 모든 페이지가 들어가고, 두 개 이상의 언어에 존재하는 페이지에는
`<xhtml:link>` 대체 링크가 붙습니다. 이중 언어 사이트에 대해 구글이 요구하는
형태입니다. `robots.txt`는 전부 허용하고 sitemap을 가리킵니다.

둘 다 `priority`와 `changefreq`를 담지 않습니다. 구글이 2023년에 둘 다 무시한다고
확인했기 때문입니다. `lastmod`도 담지 않는데, 이건 선택입니다. 여기서 쓸 수 있는
출처가 전부 믿을 만하지 않습니다. 파일 mtime은 체크아웃 시각이라 CI에서는 모든
페이지가 "오늘 아침에 바뀌었다"고 주장하게 되고, 두 번 빌드하면 같은 바이트가
나온다는 보장도 깨집니다. git 커밋 시각은 정확하지만 전체 이력이 필요한데
`actions/checkout`은 기본이 얕은 클론이라, 조용히 모든 페이지에 같은 날짜를 찍게
됩니다. 구글은 한 번 믿을 수 없다고 판단하면 그 사이트의 `lastmod`를 통째로
무시합니다. 틀린 값이 없는 것보다 나쁘다는 뜻입니다.

`llms.txt`는 모든 페이지의 제목·URL·설명을 담은 평평한 목록으로,
[llmstxt.org가 제안한 형식](https://llmstxt.org)입니다. 이 사이트에 대해 질문받은
언어 모델이 전체를 훑는 대신 파일 하나를 읽으면 됩니다. 제목·URL·설명이 이미 있으니
만드는 데 드는 게 없습니다.

`sitemap.xml`이나 `robots.txt`, `llms.txt`를 직접 `static/`에 넣으면 sqzass는 그 파일을 아예
만들지 않습니다. 당신 것을 덮어쓰지도, 조용히 무시하지도 않습니다.

## 각 안내서

| | |
|---|---|
| [GitHub Pages](@/deploy/github-pages.md) | 이 사이트가 있는 곳이자, 설계를 강제한 호스트 |
| [Netlify](@/deploy/netlify.md) | `netlify.toml`, 배포 프리뷰 |
| [Vercel](@/deploy/vercel.md) | `vercel.json`, 프레임워크 기능이 무의미한 이유 |
| [Cloudflare Pages](@/deploy/cloudflare-pages.md) | 대시보드 두 칸과 `_headers` — 캐시 제어가 필요해지면 옮겨 갈 곳 |
| [GitLab Pages](@/deploy/gitlab-ci.md) | `.gitlab-ci.yml`, 프로젝트 서브경로 |
| [Codeberg Pages](@/deploy/codeberg-pages.md) | `pages` 브랜치, `.domains`, Forgejo Actions |

전부 같은 사실 두 개입니다. `sqzass build`를 돌리고 `public/`을 발행한다. 문서가
갈리는 건 그 두 사실을 어디에 적느냐와, 각 호스트가 프리뷰 URL을 뭐라고 부르느냐뿐입니다.

## 경로 아래에 놓이는 사이트

`https://user.github.io/repo`, `https://group.gitlab.io/project`,
`https://user.codeberg.page/repo` — 전부 프로젝트 사이트이고, 전부 결과물을 도메인
루트가 아니라 경로 아래에 서빙합니다. `base_url`에 그 전체를 적으세요.

```toml
base_url = "https://user.github.io/repo"
```

그러면 sqzass가 생성하는 모든 URL에 접두사가 붙습니다 — 페이지 링크도, 스타일시트
href도, 검색 색인도. 출력 디렉터리는 평평하게 남는데, 그 디렉터리가 곧 호스트가
서빙하는 루트이기 때문입니다.

경로를 빠뜨리는 건 여기서 유일하게 **조용한** 실수입니다. 빌드는 성공하고 페이지도
다 있는데, 모든 링크와 스타일시트가 한 단계 위를 가리킵니다.

## static/ 은 통과 경로입니다

`static/` 안의 것은 경로와 이름이 그대로 유지된 채 출력에 놓입니다. sqzass가 어떤
호스트도 알지 못한 채로 호스트별 파일이 동작하는 방식입니다.

| | |
|---|---|
| `CNAME` | GitHub Pages 커스텀 도메인 |
| `.domains` | Codeberg Pages 커스텀 도메인 |
| `_headers`, `_redirects` | Netlify, Cloudflare Pages |
| `.well-known/*` | 도메인 소유 검증, `security.txt` |

콘텐츠 해시가 붙는 건 CSS와 JavaScript뿐입니다. 이름 자체가 계약인 파일은 이름을
유지합니다.

## 알아 둘 파일 둘

`.nojekyll`은 빌드마다 출력에 들어갑니다. 없으면 GitHub Pages가 출력을 Jekyll로 한 번
더 굴려서 `_`로 시작하는 디렉터리를 통째로 삼킵니다.

`CNAME`이 필요하면 `static/`에 두면 이름 그대로 복사됩니다. 해시가 붙은 `CNAME`은
GitHub이 영영 찾지 않을 파일입니다.

## 빌드가 내보내는 것

전체 목록입니다. "호스트를 옮기는 일이 복사"라는 말을 믿는 대신 확인할 수 있으라고
적어 둡니다.

| | |
|---|---|
| `static/`의 전부 | 경로와 이름 그대로. CSS와 JS에만 콘텐츠 해시 |
| `assets/highlight.<해시>.css` | `[highlight] enabled = false`가 아니면 |
| `asset-manifest.json` | 논리 이름 → 실제 URL, 항상 |
| `<경로>/index.html` | 페이지마다 하나 |
| `search-<언어>.json` | 언어마다 하나 |
| `feed-<언어>.xml` | 날짜 있는 페이지가 있는 언어마다 하나 |
| `sitemap.xml`, `robots.txt` | 같은 이름을 `static/`에 두지 않았다면 |
| `llms.txt` | 같은 조건 |
| `404.html` | `templates/404.html`이 있을 때 |
| alias 스텁 | `aliases` 항목마다 하나 |
| `.nojekyll` | 항상. `static/.nojekyll`보다 이쪽이 이깁니다 |

그 밖엔 없고, 출력 디렉터리 밖으로는 아무것도 나가지 않습니다.

## 결정성

같은 입력을 두 번 빌드하면 바이트까지 같은 출력이 나옵니다. 그래서 이 빌드를 CI에서
검사로 돌려도 되고, 바뀐 게 없는 배포는 아무것도 올리지 않습니다.
