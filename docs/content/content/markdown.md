+++
title = "Markdown"
description = "The extensions that are on, and the ones you can turn off"
weight = 40
toc = true
+++

CommonMark, via [comrak], with a set of extensions on by default. Each is a key
under `[markdown]` in `sqzass.toml`.

```toml
[markdown]
footnotes             = true
tables                = true
tasklist              = true
strikethrough         = true
autolink              = true
alerts                = true
cjk_friendly_emphasis = true
heading_anchors       = "right"   # none | left | right
```

## Alerts

GitHub's callout syntax, built on blockquotes:

```markdown
> [!NOTE]
> Prebuilt binaries are planned but not published yet.
```

> [!NOTE]
> Prebuilt binaries are planned but not published yet.

`NOTE`, `TIP`, `IMPORTANT`, `WARNING` and `CAUTION` are recognised.

## Tables

```markdown
| Key | Default |
|---|---|
| `output_dir` | `public` |
```

| Key | Default |
|---|---|
| `output_dir` | `public` |

## Task lists

```markdown
- [x] Search
- [ ] Feeds
```

- [x] Search
- [ ] Feeds

## Footnotes

Text with a note[^1].

[^1]: The note itself.

```markdown
Text with a note[^1].

[^1]: The note itself.
```

## Headings

Every heading gets an `id`, whether or not anchors are shown, because the table
of contents and any deep link you hand out both depend on it. Repeated headings
get `-1`, `-2` suffixes, and the anchor and the contents entry are guaranteed to
agree — they come from the same counter, in one pass.

`heading_anchors` controls the visible `#` link: `"right"` (default), `"left"`,
or `"none"`.

## Raw HTML

Passed through. Content here is trusted — it is in your repository, written by
you, and reviewed in the same commit as the code. Sanitising it would be
theatre.

## Code

Highlighted at build time into CSS classes. See
[Syntax highlighting](@/features/highlighting.md).

[comrak]: https://github.com/kivikakk/comrak
