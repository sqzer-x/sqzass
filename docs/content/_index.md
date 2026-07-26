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

**Korean was not bolted on afterwards.** A page's translations are linked
automatically, and a language's navigation contains only pages that exist in it,
so an untranslated page cannot leave a dead link behind. [Search](@/features/search.md)
matches substrings rather than words, which is the only way `최적화` inside
`검색엔진최적화` is ever found.

## Status

Early, and honest about it: syntax highlighting, navigation, the table of
contents, the asset pipeline, the dev server, search and Atom feeds all work.
A theme system does not exist yet. Start at [Getting started](@/start/_index.md).

[comrak]: https://github.com/kivikakk/comrak
[minijinja]: https://github.com/mitsuhiko/minijinja
