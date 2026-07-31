+++
title = "GitHub Pages"
description = "Where this site is, and the host that forced the design"
weight = 10
toc = true
+++

This site is built and deployed by the workflow below, from the same repository
as the tool. CI builds `docs/` with the binary it just compiled, so the
documentation is a regression test for the generator.

## The workflow

```yaml
name: Deploy docs

on:
  push:
    branches: [main]
  workflow_dispatch:

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
      url: ${{ steps.deployment.outputs.page_url || steps.retry.outputs.page_url }}
    steps:
      - id: deployment
        uses: actions/deploy-pages@v5
        continue-on-error: true

      - name: Wait for the previous deployment to settle
        if: steps.deployment.outcome == 'failure'
        run: sleep 90

      - id: retry
        if: steps.deployment.outcome == 'failure'
        uses: actions/deploy-pages@v5
```

`cancel-in-progress: false` is worth keeping. Cancelling a deploy that is
halfway through leaves the site partly updated, which is worse than waiting.

The retry is not defensive padding — it is the fix for a failure this site hit.
`concurrency` serialises workflow *runs*, but a Pages deployment outlives its
run: GitHub can still be processing the previous one after the workflow that
started it has finished. Push two commits close together and the second lands
in that window and dies with a 400, "due to in progress deployment". Waiting and
trying once more is enough, and it is safe: the deploy fails before it changes
anything, so a retry cannot leave the site half updated.

## A custom domain

Put the domain in `static/CNAME`:

```
sqzass.sqzer.com
```

It is copied to the output with its name intact, and GitHub reads it from
there — so the domain survives every deploy without being configured in the
repository settings again.

Point DNS at `<user>.github.io` with a `CNAME` record. If your DNS provider
proxies traffic, **turn the proxy off for this record**. A proxy that
terminates TLS itself will stop GitHub from issuing and renewing the
certificate, and the failure shows up weeks later as an expired certificate
rather than immediately as a broken deploy.

## Checking it

```bash
curl -sI https://example.com/ | head -1          # 200, over HTTPS
curl -sI https://example.com/nonexistent/        # 404, not 200
```

The second one matters. A host that answers 200 for a missing page will have
search engines index your 404.

## What you give up

No custom headers — GitHub Pages serves everything with `max-age=600`. No
redirect rules. No preview deployments for pull requests.

The first two are the reason the output is built the way it is, and the third
is `sqzass serve`. If you later need real header control, the answer is moving
to a host that has it, not stacking a CDN in front of one that does not.
