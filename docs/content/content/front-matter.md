+++
title = "Front matter"
description = "Every field a page can carry, and what it does"
weight = 10
toc = true
+++

Front matter is TOML between `+++` fences at the top of the file. TOML is the
only format — YAML would mean picking a parser, and the obvious Rust one has
been archived, so the choice was between an unmaintained dependency and a
format that needs none.

```toml
+++
title = "Installation"
+++
```

`title` is the only required field.

## Fields

| Field | Type | Default | |
|---|---|---|---|
| `title` | string | — | **Required.** |
| `description` | string | `""` | Used by templates for `<meta name="description">` and in search results. |
| `weight` | integer | `0` | Sort order within a section. Lower comes first. |
| `draft` | bool | `false` | Excluded from the build unless `--drafts`. |
| `date` | TOML date | — | Publication date. Feeds and `sort_by = "date"` use it. |
| `slug` | string | filename stem | The last URL segment. |
| `template` | string | — | Render with this template instead of the usual one. |
| `toc` | bool | `false` | Whether a table of contents should be shown. |
| `translation_key` | string | the path minus the language suffix | Links this page to its translations. See [Languages](@/content/languages.md). |
| `aliases` | array of strings | `[]` | Old URLs that should land here. Each one must be a root-absolute path, and each gets a redirect stub in the output. |
| `extra` | table | `{}` | Anything you want. Reaches templates as `page.extra`. |

`toc` is the author's intent, not the data: the table of contents is collected
for every page regardless, and templates get it as `page.toc_entries`. That
split lets a template show a contents list on long pages only, without you
having to strip the data out.

## Section-only fields

These do nothing on an ordinary page, and belong in an `_index.md`.

| Field | Type | Default | |
|---|---|---|---|
| `sort_by` | `"weight"` \| `"title"` \| `"date"` | site default | How this section orders its pages. |
| `page_template` | string | — | Default template for pages in this section. |

## Extra

`[extra]` is an open table. Nothing in sqzass reads it; templates do.

```toml
+++
title = "Release notes"
[extra]
version = "0.2.0"
badge = "beta"
+++
```

```html
{% if page.extra.badge %}<span class="badge">{{ page.extra.badge }}</span>{% endif %}
```

Reading a key that does not exist is an error, not an empty string — see
[Template data](@/templates/data.md) for how strict that is and why.

## Errors point at your file

A malformed value is reported with the line number in the source file, counted
from the top of the file rather than from the end of the front matter, because
the second one sends you to the wrong line.
