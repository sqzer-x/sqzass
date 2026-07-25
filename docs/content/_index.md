+++
title = "sqzass"
description = "A static site generator written in Rust"
weight = 0
+++

Most generators lean on their host for pretty URLs, redirects and cache
headers. sqzass does that work itself, so the same output is correct on GitHub
Pages, Vercel, Cloudflare Pages, or a plain directory served over HTTP.

```bash
sqzass build -i docs
sqzass serve -i docs
```

## What that buys you

**Markdown is transformed on the tree.** Link rewriting, heading anchors and the
table of contents are AST operations via [comrak]. The common shortcut — running
regexes over the finished HTML — silently skips any element whose attributes are
single-quoted or unquoted, and you find out in production.

**Templates cannot read a half-written file.** `templates/` is snapshotted once
per build and [minijinja]'s loader resolves `include` and `extends` from that
snapshot, so a rebuild triggered while you are saving sees one consistent state.

**Broken references stop the build.** An unresolved `@/` link, a template that
does not exist, two pages claiming the same URL, an unknown asset name — each is
an error, not a warning you scroll past.

**The same input produces the same bytes.** Two builds are byte-identical, and
CI checks it on every push.

**Bilingual from the start.** A page's translations are linked automatically, and
a language's navigation contains only pages that exist in it, so an untranslated
page cannot leave a dead link behind.

## Status

Early, and honest about it: syntax highlighting, navigation, the table of
contents, the asset pipeline and the dev server work. Search, feeds and themes
do not exist yet. Start at [Getting started](@/start/_index.md).

[comrak]: https://github.com/kivikakk/comrak
[minijinja]: https://github.com/mitsuhiko/minijinja
