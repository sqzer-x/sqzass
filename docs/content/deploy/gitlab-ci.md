+++
title = "GitLab Pages"
description = "One .gitlab-ci.yml, and the subpath that trips people up"
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

`alpine` works because the release is statically linked — there is no glibc in
that image and nothing here needs one. The job is named `pages` and the artifact
directory is `public` because GitLab looks for exactly those two names.

## The subpath

Unless you have set a custom domain, a GitLab project site is served at
`https://<group>.gitlab.io/<project>/`. That path has to be in `base_url`:

```toml
base_url = "https://mygroup.gitlab.io/myproject"
```

sqzass then puts `/myproject` in front of every URL it generates — links,
stylesheet hrefs, the search index location, everything — while the output
directory stays flat, because that directory *is* the root GitLab serves.

Getting this wrong is quiet rather than loud. The site builds, the pages exist,
and every link and stylesheet resolves one level too high, so you get 404s and
an unstyled page from a build that reported success.

For a user or group site (`<group>.gitlab.io`) there is no path, and `base_url`
is just the domain.

## Merge request previews

```yaml
pages:
  # …
  script:
    - wget -qO- … | tar xz --strip-components=1
    - ./sqzass build --base-url "$CI_PAGES_URL"
```

`$CI_PAGES_URL` already contains the project path, so this is also the shortest
way to avoid writing the subpath twice.
