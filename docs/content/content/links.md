+++
title = "Internal links"
description = "Link to a source file and let the build write the URL"
weight = 20
toc = true
+++

Write a link to the *file* and sqzass turns it into that page's URL:

```markdown
See [Installation](@/start/installation.md).
```

The path after `@/` is relative to `content/`, and it points at the markdown
file, not the URL. Move the file, rename it, or change its `slug`, and the link
follows.

## Broken links stop the build

An `@/` link that resolves to nothing is an error:

```
docs/content/start/first-site.md: 해석할 수 없는 내부 링크가 있습니다:
  @/start/setup.md
```

This is the point of the syntax. A plain `/start/setup/` link that goes nowhere
is indistinguishable from one that works until somebody clicks it in
production. A reference the build can check is a reference the build *does*
check.

## They follow the reader's language

`@/start/installation.md` resolves to `/start/installation/` for an English
reader and `/ko/start/installation/` for a Korean one — the same markdown, in
both language trees, without a single conditional.

If the target has no translation in the current language, the link falls back
to the default language rather than breaking. See
[Languages](@/content/languages.md).

Because of that fallback, you write the path once and never write it with a
language prefix. `@/ko/start/installation.md` is not a thing.

## It happens on the tree

Rewriting is done through comrak's URL rewriter, on the AST, before any HTML
exists. The shortcut — running a regex over the finished HTML — silently skips
any element whose attributes are single-quoted or unquoted, which produces a
class of bug you find in production rather than in the build.

Image sources go through the same path, so `![](@/images/x.png)` works too.

## Generated files are link targets too

`/sitemap.xml`, `/robots.txt`, `/llms.txt`, `/404.html`, `/feed-<lang>.xml` and
`/search-<lang>.json` are pages as far as the checker is concerned — the build
produces them, so a link to one resolves.

The exception is anything content-hashed. `/css/main.css` does not exist after a
build; `asset("css/main.css")` in a template is the way to reach it.

## Everything else is left alone

External links, anchors, `mailto:` — untouched. sqzass only claims the `@/`
prefix.
