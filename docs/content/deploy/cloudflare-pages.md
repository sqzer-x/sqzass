+++
title = "Cloudflare Pages"
description = "Two settings in the dashboard, and the headers GitHub Pages would not give you"
weight = 40
toc = true
+++

In the project's build settings:

| | |
|---|---|
| Framework preset | None |
| Build command | `curl -sSL https://github.com/sqzer-x/sqzass/releases/latest/download/sqzass-x86_64-unknown-linux-musl.tar.gz \| tar xz --strip-components=1 && ./sqzass build` |
| Build output directory | `public` |

There is no configuration file to write.

## This is the host to move to when you need headers

sqzass is built so nothing depends on host configuration, and this site is on
GitHub Pages specifically to keep that honest. But GitHub Pages serves
everything with `cache-control: max-age=600` and there is no way to change it.

Cloudflare Pages reads a `_headers` file, so the content-hashed filenames can
finally mean what they are for:

```
# static/_headers
/css/*
  Cache-Control: public, max-age=31536000, immutable
/js/*
  Cache-Control: public, max-age=31536000, immutable
/*
  Cache-Control: public, max-age=600
```

A year is safe for those two directories precisely because their filenames
change when their contents change. That is the payoff of hashing the name
rather than appending a query string, and it is unavailable on a host that will
not let you set the header.

Put the file in `static/` and it is copied through untouched.

## Redirects

`_redirects` works the same way, but reach for `aliases` in front matter first:
it lives next to the page that moved, it is checked at build time, and it
follows the page to whatever host you use next.

## Preview deployments

`$CF_PAGES_URL` holds the preview address:

```bash
./sqzass build --base-url "$CF_PAGES_URL"
```
