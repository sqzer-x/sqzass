+++
title = "Deploying"
description = "The output is a directory. Any host that serves files will do."
weight = 50
sort_by = "weight"
+++

```bash
sqzass build -i mysite
```

`mysite/public` is the site. Copy it somewhere that serves files and you are
done — there is no runtime, no server component and no build step left to run.

## What sqzass does so the host does not have to

Every URL is a directory containing `index.html`, so pretty URLs need no
rewrite rules. Cache busting is in the filename, so it needs no cache headers.
Nothing in the output depends on host configuration.

This is deliberate, and the docs site is hosted on
[GitHub Pages](@/deploy/github-pages.md) to keep it honest: Pages offers no
custom headers, no redirect rules and no rewrites, so anything that needed them
would break here first.

The payoff is that moving hosts is a copy. The same directory is correct on
Cloudflare Pages, Netlify, S3 behind CloudFront, or nginx.

## Two files worth knowing about

`.nojekyll` is written into every build. Without it GitHub Pages runs the
output through Jekyll, which swallows directories beginning with `_`.

`CNAME`, if you need one, goes in `static/` and is copied through with its name
intact — a hashed `CNAME` is a file GitHub will never look for.

## Determinism

Two builds of the same input produce byte-identical output. That makes the
build safe to run in CI as a check, and it means a deploy that changes nothing
uploads nothing.
