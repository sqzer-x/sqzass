+++
title = "Vercel"
description = "vercel.json, and why none of the framework machinery applies"
weight = 30
toc = true
+++

```json
{
  "buildCommand": "curl -sSL https://github.com/sqzer-x/sqzass/releases/latest/download/sqzass-x86_64-unknown-linux-musl.tar.gz | tar xz --strip-components=1 && ./sqzass build",
  "outputDirectory": "public",
  "framework": null
}
```

`"framework": null` matters. Vercel's detection looks for a `package.json` and,
finding none, will otherwise guess — and a wrong guess produces a build that
fails in a way that reads as sqzass's fault.

## Preview deployments

Vercel exposes the preview host as `$VERCEL_URL`, without a scheme:

```json
{
  "buildCommand": "… && ./sqzass build --base-url \"https://$VERCEL_URL\""
}
```

Same reasoning as everywhere else: internal links are root-absolute and work
regardless, but canonical, the sitemap and social tags are absolute and would
otherwise announce production from a preview.

## Trailing slashes

sqzass writes `/start/index.html`, so `/start/` is the canonical form and every
link it generates uses it. Vercel's default `trailingSlash` behaviour redirects
between the two, which costs a hop on links written by hand as `/start`. Setting
`"trailingSlash": true` removes the hop.

This is a preference, not a requirement — the site is correct either way.

## What this does not need

No serverless functions, no ISR, no edge config, no image optimisation.
sqzass has nothing to run at request time, so the parts of Vercel that
distinguish it from a file server are all inert here.

That is worth saying plainly: if Vercel is where your team already deploys,
this works. If you are choosing a host for a sqzass site, the reason to pick one
over another is not going to be its framework support.
