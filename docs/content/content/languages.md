+++
title = "Languages"
description = "Two languages from one content tree, with untranslated pages hidden"
weight = 30
toc = true
+++

Declare the languages in `sqzass.toml`. The default one lives at the root;
every other one gets a prefix.

```toml
default_language = "en"

[languages.en]
name   = "English"
weight = 1

[languages.ko]
name   = "한국어"
weight = 2
```

`/start/` is English. `/ko/start/` is Korean.

## A suffix on the filename

```
content/start/
├── installation.md        →  /start/installation/
└── installation.ko.md     →  /ko/start/installation/
```

The two files sit next to each other, which means `ls` tells you what is not
translated yet. A parallel `content.ko/` tree would hide that behind a diff.

## Untranslated pages are hidden, not duplicated

A page with no Korean version does not appear in the Korean navigation at all.
The two alternatives are both worse: rendering the English text at a Korean URL
creates duplicate content for search engines, and emitting a 404 sends a reader
to a dead end from a link the site itself drew.

Templates see `page.translations`, which contains only the languages this page
actually exists in — so a language switcher can render exactly the choices that
work:

```html
{% for t in page.translations %}
<a href="{{ t.url }}" hreflang="{{ t.code }}">{{ t.name }}</a>
{% endfor %}
```

Empty list, no switcher. There is no state where the button lies.

## How translations are matched

By filename stem: `installation.md` and `installation.ko.md` are the same page
in two languages. When the filenames have to differ — a localised slug, say —
set `translation_key` in both files to the same value.

```toml
# content/start/installation.md
+++
title = "Installation"
translation_key = "install"
+++
```

```toml
# content/start/설치.ko.md
+++
title = "설치"
translation_key = "install"
+++
```

## Korean specifics

Two things are handled for you and are worth knowing about, because both are
silent when they go wrong.

**`**강조**한다` parses as emphasis.** CommonMark's flanking rules were written
for languages that put spaces around words, and under them a `**bold**` run
immediately followed by a Korean particle is not emphasis at all. sqzass turns
on comrak's `cjk_friendly_emphasis`, which is why the markdown you would
naturally write works. It is a `[markdown]` key, and turning it off breaks
Korean text in a way that looks like your markdown is wrong.

**Korean headings keep their Hangul ids.** `## 설치` becomes `id="설치"`, so
anchors and the table of contents work without transliteration.
