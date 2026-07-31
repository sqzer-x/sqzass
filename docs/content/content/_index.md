+++
title = "Writing content"
description = "Pages, sections, front matter and the URLs they produce"
toc = true
weight = 20
sort_by = "weight"
+++

Everything under `content/` is a page. A directory with an `_index.md` in it is
a section, and a section collects the pages beside it.

## Pages and sections

```
content/
├── _index.md            →  /
├── about.md             →  /about/
└── guide/
    ├── _index.md        →  /guide/
    ├── install.md       →  /guide/install/
    └── deep/
        ├── _index.md    →  /guide/deep/
        └── dive.md      →  /guide/deep/dive/
```

A directory *without* an `_index.md` still becomes a section, titled after the
directory, so its pages are collected and it appears in navigation — forgetting
the file must not make pages vanish from the sidebar. What it does not get is an
index page: `/guide/` itself 404s until you add one. Add `_index.md` when the
section needs a title of its own, a description, or a body.

## Why every URL is a directory

Pages are written as `<path>/index.html`, never `<path>.html`. A host with
rewrite rules can serve `/about` from `about.html`, but a host without them
cannot, and sqzass is built to be correct on the host that gives you nothing.
The directory form works everywhere — GitHub Pages, Cloudflare Pages, S3, or
`python3 -m http.server` — because it asks for no cleverness from the server.

The cost is that a link to `/about` (no trailing slash) gets redirected by most
servers before it resolves. Link to `/about/`, or better, use
[`@/` links](@/content/links.md) and let sqzass write the URL.

## Slugs come from filenames

`install.md` becomes `/guide/install/`. The title has nothing to do with it.

This matters most in Korean. Transliterating a title would be the usual trick,
but `slug`-style crates map Hangul to ASCII one syllable at a time and without
context, so two different Korean titles can collapse onto the same path. A
filename you chose is unambiguous and it is already unique in its directory.
A Korean filename is percent-encoded UTF-8 and left alone.

Override it per page with `slug` in front matter. **Two pages claiming the same
URL is a build error**, not a last-writer-wins race.

## Ordering

Sections sort their pages by `weight` (ascending), and pages without one fall
back to their title. Set `sort_by` on a section's `_index.md`, or `[nav] sort_by`
in `sqzass.toml` to change the default for the whole site.

| | |
|---|---|
| `weight` | Ascending. The default. |
| `title` | Ascending. |
| `date` | **Descending** — newest first, undated pages last. See [Feeds](@/features/feeds.md). |

## Drafts

`draft = true` keeps a page out of the build. `--drafts` on the command line, or
`[build] drafts = true` in the config, puts them back.
