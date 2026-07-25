+++
title = "Static assets"
description = "Copied, hashed, and looked up by their original name"
weight = 30
toc = true
+++

Everything under `static/` is copied into the output, keeping its path.

```
static/
├── css/main.css   →  /css/main.a1b2c3d4.css
├── js/search.js   →  /js/search.e5f6a7b8.js
├── images/x.png   →  /images/x.png
└── CNAME          →  /CNAME
```

CSS and JavaScript get a content hash in the filename. Look them up by the
name you wrote:

```html
<link rel="stylesheet" href="{{ asset("css/main.css") }}">
<script src="{{ asset("js/search.js") }}" defer></script>
```

## In the filename, not a query string

`main.css?v=123` has two problems. Some CDNs and proxies drop the query from
the cache key, so the bust does not happen. And the common shape of it — one
build stamp on every file — invalidates the whole site when you fix one line of
CSS.

A per-file content hash invalidates exactly the file that changed, and works
the same on every host, including hosts with no cache configuration at all.

## Why only CSS and JS

Images are usually referenced by a literal path — from markdown, from CSS,
sometimes from a template you did not write. Renaming them breaks those
references without a way to fix them up.

And some filenames *are* the contract. `CNAME` tells GitHub Pages your domain;
`robots.txt` is looked for by name. `CNAME.9f8e7d6c` is a file nothing will
ever read.

## Generated assets go through the same path

The highlight stylesheet is built from your themes rather than copied, but it
is hashed and served identically. Templates reach it as
`site.highlight_css`.

## Turning it off

```toml
[assets]
source_dir  = "static"
fingerprint = false
```

With `fingerprint = false`, files keep their names and `asset()` still works —
so you can flip it without editing a template.

## asset-manifest.json

Every build writes one, at the output root, mapping logical names to the URLs
they were written to:

```json
{
  "css/main.css": "/css/main.a1b2c3d4.css",
  "CNAME": "/CNAME"
}
```

It contains every static file, not only the hashed ones, and it is written even
with `fingerprint = false`. Nothing in sqzass reads it back — it is there so
that something outside the build can answer the same question `asset()` answers
inside it: a service worker, a deploy script, a cache warmer.

It is safe to ignore. It is not safe to delete in a cleanup step and then wonder
why a tool that depended on it stopped working, which is the situation this
paragraph exists to prevent.

## The search index is not hashed

`/search-en.json` and `/search-ko.json` keep fixed names. They are fetched by
script rather than linked from `<head>`, and their contents come from the
rendered pages — which are rendered after assets are resolved, so hashing them
would be circular. Ten minutes of a stale index costs a few search hits. See
[Search](@/features/search.md).
