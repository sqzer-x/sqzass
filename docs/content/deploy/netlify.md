+++
title = "Netlify"
description = "netlify.toml, and deploy previews that build with the right base_url"
weight = 20
toc = true
+++

```toml
# netlify.toml
[build]
  command   = "curl -sSL https://github.com/sqzer-x/sqzass/releases/latest/download/sqzass-x86_64-unknown-linux-musl.tar.gz | tar xz --strip-components=1 && ./sqzass build"
  publish   = "public"

[context.deploy-preview]
  command   = "curl -sSL https://github.com/sqzer-x/sqzass/releases/latest/download/sqzass-x86_64-unknown-linux-musl.tar.gz | tar xz --strip-components=1 && ./sqzass build --base-url $DEPLOY_PRIME_URL"
```

That is the whole configuration. No plugin, no build image, no runtime.

## Why the binary is downloaded rather than installed

Netlify's build image has no Rust toolchain by default, and adding one costs
minutes on every build. The Linux release is statically linked, so it runs in
any image without matching a glibc version — which is the same property that
makes it work in Alpine, in a scratch container, and on a machine older than the
one that built it.

If you would rather build from source, `cargo install --git
https://github.com/sqzer-x/sqzass` works too and is slower.

## Deploy previews need their own base_url

A preview runs at `deploy-preview-42--yoursite.netlify.app`, not at your domain.
Netlify puts that address in `$DEPLOY_PRIME_URL`, and `--base-url` overrides the
config for exactly this case — so canonical links, the sitemap and OpenGraph
tags describe the preview rather than pointing every crawler at production.

Without it the preview still *works*, because internal links are root-absolute
and do not care what the host is called. What breaks is everything absolute:
`page.permalink`, `sitemap.xml`, and the social tags.

## Pretty URLs, redirects and headers

Nothing to configure. Every page is a directory containing `index.html`, so
Netlify serves `/start/` without a rewrite rule, and `/404.html` is picked up by
name if you have a `templates/404.html`.

`_redirects` and `_headers` are Netlify's own files and neither is required.
If you want them, put them in `static/` and they are copied through untouched —
`static/` is a passthrough, and files whose name is the contract keep their name.

```
static/
├── _headers
└── _redirects
```

## What this does not need

No `netlify-plugin-*`, no `NODE_VERSION`, no `functions` directory. The output
is a directory of files.
