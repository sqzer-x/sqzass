+++
title = "Functions and filters"
description = "Everything a template can call, and the closing line that nothing else exists"
weight = 40
toc = true
+++

## Provided by sqzass

Two functions. That is the whole list.

### asset(path)

Returns the URL a static file was written to, hash and subpath included.

```html
<link rel="stylesheet" href="{{ asset("css/main.css") }}">
<script src="{{ asset("js/search.js") }}" defer></script>
```

The argument is the logical path under `static/`, with or without a leading
slash. A name that was not collected is a build error listing every name that
was — a renamed stylesheet fails the build instead of 404ing for every visitor.

### t(key)

Looks a UI string up in `i18n/<language>.toml`, for the language of the page
being rendered.

```html
<a class="skip" href="#content">{{ t("skip_to_content") }}</a>
```

One argument. The language is never passed in — see
[Languages](@/content/languages.md) for why that is deliberate. A key missing
from the current language is a build error.

> [!WARNING]
> Do not name a loop variable `t`. `{% for t in page.translations %}` shadows
> the function inside that block, and the first person to add a label there gets
> an error a long way from its cause.

## From minijinja

Standard Jinja2 syntax works: `{% if %}`, `{% for %}`, `{% extends %}`,
`{% block %}`, `{% include %}`, `{% macro %}`, `{% from … import … %}`, `{% set %}`,
and the usual filters — `safe`, `escape`, `length`, `join`, `default`, `upper`,
`lower`, `replace`, `trim`, `first`, `last`, `reverse`, `sort`, `map`,
`select`, `selectattr`, `batch`, `slice`, `int`, `float`, `abs`, `round`,
`urlencode`, `striptags`, `indent`.

Our own templates use `{% macro %}` and `{% from "partials/sidebar.html" import nav %}`,
so those two are exercised on every build.

`tojson` is **not** available: minijinja's `json` feature is off, and turning it
on to serialise a value into a template is a dependency for something a
`{% for %}` loop already does.

## Nothing else exists

There is no `url_for`, no `markdownify`, no `date`, no `now()`, no `env()`, no
custom tests. Some of those are absences with reasons rather than gaps:

`now()` and `env()` would break the guarantee that two builds of the same input
produce identical bytes, which CI checks on every push. A template that can read
the clock cannot be reproducible.

`url_for` would wrap a comparison that is already one line. Page and section
URLs arrive ready to use, and they already carry the subpath.

Reading a name that does not exist is a build error, not an empty string — so
if you call something from another generator by habit, you find out at build
time rather than from a page with a hole in it.

## Marking the current page

There is no helper for this, and none is needed. Exact match for a page:

```html
<a href="{{ p.url }}"{% if p.url == page.url %} aria-current="page"{% endif %}>{{ p.title }}</a>
```

Ancestor match for a section, using `page.section`:

```html
<a href="{{ s.url }}"{% if page.section and page.section.url == s.url %} aria-current="true"{% endif %}>{{ s.title }}</a>
```

For a prefix test use `startingwith`, but be careful with the root: every URL
starts with `/`, so the home link must be compared exactly and never as an
ancestor.
