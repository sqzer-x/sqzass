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

By the path under `content/` with the language suffix removed — so
`start/installation.md` and `start/installation.ko.md` both key on
`start/installation` and are the same page in two languages.

The path matters, not just the name: `a/notes.md` and `b/notes.md` are two
different pages, not translations of each other. Only a suffix that is a
language you declared is stripped, so a file named `notes.ab.md` keys on
`notes.ab` unless `ab` is in your `[languages]`. When the filenames have to differ — a localised slug, say —
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

## UI strings

Page text lives in `content/`. The words the *template* supplies — "Skip to
content", "On this page", "Previous" — live in `i18n/<code>.toml`.

```toml
# i18n/en.toml
home         = "Home"
on_this_page = "On this page"
```

```toml
# i18n/ko.toml
home         = "홈"
on_this_page = "이 페이지"
```

```html
<a href="{{ site.base_path }}/">{{ t("home") }}</a>
```

`t` reads the language from the page being rendered, so a template never asks
which language it is in. It never has to be told, and there is no line where
someone can forget to tell it.

**A key missing from one language is an error**, and the message says which
languages do have it:

```
번역 키 'next' 가 i18n/ko.toml 에 없습니다 (en 에는 있습니다) (in page.html:22)
```

Falling back to the default language would put English labels inside a Korean
page — visible to every Korean reader and invisible to whoever is maintaining
the site, which is the same reason untranslated pages are hidden rather than
duplicated.

Sites with no `i18n/` directory work fine. `t` is only needed by templates that
call it.

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
