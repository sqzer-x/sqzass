+++
title = "Codeberg Pages"
description = "pages 브랜치, .domains 파일, 그리고 Forgejo Actions"
weight = 60
toc = true
+++

Codeberg는 저장소의 `pages` 브랜치를 `https://<사용자>.codeberg.page/<저장소>/`에
서빙합니다. 저장소 이름이 `pages`면 통째로 `https://<사용자>.codeberg.page/`가
됩니다.

## CI에서 빌드하기

```yaml
# .forgejo/workflows/deploy.yml
on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: docker
    container:
      image: alpine:latest
    steps:
      - run: apk add --no-cache git nodejs
      - uses: actions/checkout@v4
      - run: wget -qO- https://github.com/sqzer-x/sqzass/releases/latest/download/sqzass-x86_64-unknown-linux-musl.tar.gz | tar xz --strip-components=1
      - run: ./sqzass build
      - name: pages 브랜치로 발행
        run: |
          cd public
          git init -q && git add -A
          git -c user.email=ci -c user.name=ci commit -qm "Deploy"
          git push -f "https://$GITHUB_ACTOR:${{ secrets.PAGES_TOKEN }}@codeberg.org/$GITHUB_REPOSITORY.git" HEAD:pages
```

`nodejs`는 Forgejo Actions 러너에 필요한 것이지 sqzass에 필요한 게 아닙니다.

## 여기도 서브경로

저장소 이름이 `pages`가 아니라면 사이트는 `/<저장소>/` 아래에 놓입니다.

```toml
base_url = "https://myuser.codeberg.page/myrepo"
```

실패 방식은 GitLab과 같습니다. 빌드는 성공하고 모든 링크가 한 단계 위를 가리킵니다.
자세한 설명은 [GitLab Pages](@/deploy/gitlab-ci.md)에 적어 두었습니다.

## 커스텀 도메인

Codeberg는 서빙되는 디렉터리 루트의 `.domains` 파일을 읽습니다. `static/`에 두면
GitHub Pages의 `CNAME`과 마찬가지로 이름이 유지된 채 그대로 나갑니다.

```
# static/.domains
example.com
www.example.com
```

커스텀 도메인을 쓰면 서브경로가 없으므로 `base_url`은 도메인만 남습니다.

> [!NOTE]
> `Path::extension()`은 이름 전체가 확장자처럼 생긴 파일에 대해 아무것도 돌려주지
> 않습니다. `.domains`나 `.nojekyll` 같은 파일이 `main.css`와 다른 취급을 받아야
> 하는 이유입니다. sqzass는 이런 파일을 그대로 통과시키고, 이름 전체가 확장자인
> 것에 해시를 붙이려 하지 않습니다.
