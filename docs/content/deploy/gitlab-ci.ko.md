+++
title = "GitLab Pages"
description = ".gitlab-ci.yml 하나, 그리고 사람들이 걸려 넘어지는 서브경로"
weight = 50
toc = true
+++

```yaml
# .gitlab-ci.yml
pages:
  image: alpine:latest
  script:
    - wget -qO- https://github.com/sqzer-x/sqzass/releases/latest/download/sqzass-x86_64-unknown-linux-musl.tar.gz | tar xz --strip-components=1
    - ./sqzass build
  artifacts:
    paths: [public]
  rules:
    - if: $CI_COMMIT_BRANCH == $CI_DEFAULT_BRANCH
```

`alpine`이 되는 건 릴리스가 정적 링크이기 때문입니다. 그 이미지엔 glibc가 없고,
여기엔 glibc가 필요한 게 없습니다. 잡 이름이 `pages`이고 아티팩트 디렉터리가
`public`인 건 GitLab이 정확히 그 두 이름을 찾기 때문입니다.

## 서브경로

커스텀 도메인을 걸지 않았다면 GitLab 프로젝트 사이트는
`https://<그룹>.gitlab.io/<프로젝트>/`에서 서빙됩니다. 그 경로가 `base_url`에
들어가야 합니다.

```toml
base_url = "https://mygroup.gitlab.io/myproject"
```

그러면 sqzass가 생성하는 모든 URL 앞에 `/myproject`가 붙습니다 — 링크도,
스타일시트 href도, 검색 색인 위치도 전부. 출력 디렉터리는 그대로 평평하게 남는데,
그 디렉터리가 곧 GitLab이 서빙하는 루트이기 때문입니다.

이걸 틀리면 요란하지 않고 조용합니다. 빌드는 되고 페이지도 생기는데, 모든 링크와
스타일시트가 한 단계 위를 가리켜서 404와 스타일 없는 페이지가 나옵니다. 빌드는
성공했다고 보고한 채로요.

사용자/그룹 사이트(`<그룹>.gitlab.io`)에는 경로가 없으므로 `base_url`은 도메인뿐입니다.

## 머지 리퀘스트 프리뷰

```yaml
pages:
  # …
  script:
    - wget -qO- … | tar xz --strip-components=1
    - ./sqzass build --base-url "$CI_PAGES_URL"
```

`$CI_PAGES_URL`에 이미 프로젝트 경로가 들어 있으므로, 서브경로를 두 번 적지 않는
가장 짧은 방법이기도 합니다.
