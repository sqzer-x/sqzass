+++
title = "Codeberg Pages"
description = "A pages branch, a .domains file, and Forgejo Actions"
weight = 60
toc = true
+++

Codeberg serves the `pages` branch of a repository at
`https://<user>.codeberg.page/<repo>/`, or the whole of a repository named
`pages` at `https://<user>.codeberg.page/`.

## Building it in CI

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
      - name: Publish to the pages branch
        run: |
          cd public
          git init -q && git add -A
          git -c user.email=ci -c user.name=ci commit -qm "Deploy"
          git push -f "https://$GITHUB_ACTOR:${{ secrets.PAGES_TOKEN }}@codeberg.org/$GITHUB_REPOSITORY.git" HEAD:pages
```

`nodejs` is there for the Forgejo Actions runner, not for sqzass.

## The subpath, again

Unless the repository is named `pages`, the site lives under `/<repo>/`:

```toml
base_url = "https://myuser.codeberg.page/myrepo"
```

The failure mode is the same one GitLab has — a build that succeeds and a site
where every link is one level too high. See
[GitLab Pages](@/deploy/gitlab-ci.md) for the longer explanation.

## A custom domain

Codeberg reads a `.domains` file from the root of the served directory. Put it
in `static/` and it comes through with its name intact, the same way `CNAME`
does for GitHub Pages:

```
# static/.domains
example.com
www.example.com
```

With a custom domain there is no subpath, so `base_url` becomes the domain.

> [!NOTE]
> `Path::extension()` returns nothing for a name that is all suffix, which is
> why files like `.domains` and `.nojekyll` need handling that files like
> `main.css` do not. sqzass copies them through; it does not try to fingerprint
> a name that is entirely an extension.
