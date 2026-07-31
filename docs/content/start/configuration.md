+++
title = "Configuration"
description = "Every key in sqzass.toml, and what happens when you misspell one"
weight = 30
toc = true
+++

`sqzass.toml` sits at the site root. Two keys are required; everything else has
a default and behaves as if you had written the default.

```toml
title    = "My site"
base_url = "https://example.com"
```

## A typo is an error

```
error: [SQZASS_E_CONFIG] sqzass.toml 파싱 실패: TOML parse error at line 8, column 1
  |
8 | theme_ligth = "InspiredGitHub"
  | ^^^^^^^^^^^
unknown field `theme_ligth`, expected one of `enabled`, `theme_light`, `theme_dark`
```

A key sqzass does not read is a key that does nothing, and a setting that
silently does nothing is the same failure as a broken link: you asked for
something, the tool agreed, and nothing happened. You would spend the afternoon
wondering why your theme did not change.

The same applies in front matter, and the line number points into your file
rather than into the front matter block.

## Site

| Key | Default | |
|---|---|---|
| `title` | — | **Required.** |
| `base_url` | — | **Required.** Used for canonical URLs, the sitemap and `robots.txt`. |
| `description` | `""` | |
| `default_language` | `"en"` | This language lives at the root; others get a URL prefix. |

## `[languages.<code>]`

```toml
[languages.en]
name   = "English"
weight = 1

[languages.ko]
name   = "한국어"
weight = 2
```

`name` is what a language switcher shows. `weight` orders them. Declaring no
languages at all is fine — the site is then single-language under
`default_language`. See [Languages](@/content/languages.md).

## `[build]`

| Key | Default | |
|---|---|---|
| `output_dir` | `"public"` | Relative to the site root. |
| `drafts` | `false` | `--drafts` sets this from the command line. |

A draft is excluded from the build entirely, not hidden by CSS. See
[Writing content](@/content/_index.md).

## `[markdown]`

| Key | Default | |
|---|---|---|
| `footnotes` | `true` | |
| `tables` | `true` | |
| `tasklist` | `true` | |
| `strikethrough` | `true` | |
| `autolink` | `true` | |
| `alerts` | `true` | GitHub's `> [!NOTE]` callouts. |
| `cjk_friendly_emphasis` | `true` | **Leave this on for Korean.** |
| `heading_anchors` | `"right"` | `none`, `left` or `right`. |

`cjk_friendly_emphasis` is what makes `**강조**한다` parse as emphasis.
CommonMark's flanking rules assume spaces around words, and without this a bold
run followed immediately by a Korean particle is not emphasis at all. Turning it
off breaks Korean text in a way that reads like your markdown is wrong. See
[Markdown](@/content/markdown.md).

## `[highlight]`

| Key | Default | |
|---|---|---|
| `enabled` | `true` | |
| `theme_light` | `"InspiredGitHub"` | |
| `theme_dark` | `"base16-ocean.dark"` | |

A theme name that does not exist is an error, and the message lists the ones
that do. See [Syntax highlighting](@/features/highlighting.md).

## `[assets]`

| Key | Default | |
|---|---|---|
| `source_dir` | `"static"` | |
| `fingerprint` | `true` | Content hash in the filename, for CSS and JS. |

See [Static assets](@/templates/assets.md).

## `[nav]`

| Key | Default | |
|---|---|---|
| `sort_by` | `"weight"` | `weight`, `title` or `date`. A section can override it. |

How a section overrides it, and why `date` alone sorts newest first, are in
[Writing content](@/content/_index.md).

## `[search]`

| Key | Default | |
|---|---|---|
| `enabled` | `true` | Emit `search-<lang>.json`, one per language. |

The index carries the body text of every page, so it grows with your content —
a site with no search UI has no reason to pay for it. Disabled, no index is
written and no index URL exists for links to point at. See
[Search](@/features/search.md).

## Overriding from the command line

`--base-url` and `--drafts` win over the file, which is what makes one
configuration serve both a preview deployment and production.

```bash
sqzass build -i mysite --base-url https://preview.example.com --drafts
```
