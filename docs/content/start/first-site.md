+++
title = "Your first site"
description = "The smallest site sqzass will build, one file at a time"
weight = 20
toc = true
+++

There is no `sqzass init` yet, so a site starts as three files you make
yourself. That is the whole scaffold — sqzass has no hidden state and no
generated directory it expects to own.

## The three files

```
mysite/
├── sqzass.toml
├── content/
│   └── _index.md
└── templates/
    └── page.html
```

`sqzass.toml` needs two keys. Everything else has a default.

```toml
title    = "My site"
base_url = "https://example.com"
```

`content/_index.md` is the front page. Front matter is TOML between `+++`
fences, and `title` is the only field you must supply.

```markdown
+++
title = "My site"
+++

Hello.
```

`templates/page.html` renders it. `page.content` is already HTML, so it needs
`| safe` — templates escape by default, and that default is what keeps a stray
`<` in your prose from becoming markup.

```html
<!doctype html>
<html lang="{{ page.language }}">
<head><meta charset="utf-8"><title>{{ page.title }}</title></head>
<body>{{ page.content | safe }}</body>
</html>
```

## Build it

```bash
sqzass build -i mysite
```

The output lands in `mysite/public`. Every page is written as
`<path>/index.html`, never `<path>.html`, so the URL is `/about/` and not
`/about.html` — see [Pages and sections](@/content/_index.md) for why that
matters on a host with no rewrite rules.

## Work on it

```bash
sqzass serve -i mysite
```

This serves the site from memory on <http://127.0.0.1:3000> and rebuilds when
a file changes. Nothing is written to `public/` while it runs, so a rebuilding
site never serves a half-written file. The page reloads itself; a change that
only touched CSS swaps the stylesheet in place and keeps your scroll position.

> [!NOTE]
> The dev server is a development tool. It does no caching, no compression and
> no access control. Serve the built directory with a real server in production.

## What to add next

A second page is a second file. `content/about.md` becomes `/about/`, and
`content/guide/_index.md` starts a section that collects everything beside it.
[Front matter](@/content/front-matter.md) lists the fields a page can carry.
