+++
title = "Templates"
description = "Jinja2-compatible templates, a strict data model, and explicit selection"
weight = 30
sort_by = "weight"
+++

Templates live in `templates/` and are [minijinja] — Jinja2 syntax, so
`{% extends %}`, `{% block %}`, `{% include %}`, `{% macro %}` and filters all
work the way you expect.

```
templates/
├── base.html
├── page.html
├── section.html
└── partials/
    ├── sidebar.html
    └── toc.html
```

## Two things are stricter than you may be used to

**Undefined access is an error.** Rename a front matter key and the build stops
with the name of the template and the key, instead of rendering a page with a
hole in it. It is not the strictest setting available — `{% if optional %}` on
something that was never defined still works, because a template that cannot
ask "is this here?" is not usable for a site where pages differ.

**Everything is escaped by default.** `page.content` is already HTML, so it
needs `| safe`. That is one deliberate `| safe` in exchange for every other
value being safe without you thinking about it.

## A snapshot per build

`templates/` is read once at the start of a build, and `include`/`extends`
resolve against that snapshot. Save a partial while the dev server is watching
and the rebuild it triggers sees one consistent set of files — never a
half-written one.

[minijinja]: https://github.com/mitsuhiko/minijinja
