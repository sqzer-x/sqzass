+++
title = "sqzass"
description = "A static site generator written in Rust"
weight = 0
+++

**sqzass** is a static site generator written in Rust. This site is built with it.

## Why another one?

Most generators lean on their host for pretty URLs, redirects and cache headers.
sqzass does that work itself, so the same output is correct on GitHub Pages,
Vercel, Cloudflare Pages or a plain directory served over HTTP.

- Markdown via [comrak](https://github.com/kivikakk/comrak), with a real AST — link
  rewriting and heading anchors happen on the tree, never as a regex pass over the
  final HTML.
- Templates via [minijinja](https://github.com/mitsuhiko/minijinja), autoescaped by
  default, resolved from a snapshot so a rebuild mid-save can never read a
  half-written partial.
- Bilingual from the start.

## Status

Early. See [Getting started](@/start/_index.md).
