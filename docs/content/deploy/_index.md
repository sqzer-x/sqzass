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

## What else is written

`sitemap.xml` lists every page, with `<xhtml:link>` alternates for pages that
exist in more than one language — which is the form Google asks for when a site
is bilingual. `robots.txt` allows everything and points at the sitemap.

Neither carries `priority` or `changefreq`, because Google confirmed in 2023
that it ignores both. Neither carries `lastmod` either, and that one is a
choice: the honest sources for it are all unreliable here. A file's mtime is
its checkout time, so in CI every page would claim to have changed this
morning, and it would break the guarantee that two builds produce the same
bytes. Git commit times are accurate but need full history, and
`actions/checkout` clones shallow by default — so it would quietly stamp every
page with the same date. Google ignores a site's `lastmod` entirely once it
finds it untrustworthy, which makes a wrong one worse than none.

Put your own `sitemap.xml` or `robots.txt` in `static/` and sqzass will not
generate that file at all. It does not overwrite yours, and it does not
silently ignore it either.

## Two files worth knowing about

`.nojekyll` is written into every build. Without it GitHub Pages runs the
output through Jekyll, which swallows directories beginning with `_`.

`CNAME`, if you need one, goes in `static/` and is copied through with its name
intact — a hashed `CNAME` is a file GitHub will never look for.

## Determinism

Two builds of the same input produce byte-identical output. That makes the
build safe to run in CI as a check, and it means a deploy that changes nothing
uploads nothing.
