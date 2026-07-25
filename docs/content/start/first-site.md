+++
title = "Your first site"
description = "From an empty directory to a page on screen"
weight = 20
toc = true
+++

```bash
sqzass init mysite
sqzass serve -i mysite
```

That is the whole thing. `init` writes three files, `serve` puts them on
<http://127.0.0.1:3000>, and there is no fourth step.

## What it wrote

```
mysite/
├── sqzass.toml
├── content/
│   └── _index.md
└── templates/
    └── page.html
```

Three files, because a scaffold you have to delete half of is not a head start.
There is no `.gitignore` you did not ask for, no example blog post, and no
directory sqzass expects to own.

`init` refuses to run in a directory that already has a `sqzass.toml`, so it
cannot half-overwrite a site you already have.

## sqzass.toml

Two keys. Everything else has a default, and a key you leave out behaves as if
it were set to that default.

```toml
title    = "mysite"
base_url = "https://example.com"
```

## content/_index.md

The front page. Front matter is TOML between `+++` fences, and `title` is the
only field you must supply.

```markdown
+++
title = "mysite"
+++

Hello.
```

## templates/page.html

`page.content` is already HTML, so it needs `| safe` — templates escape by
default, and that default is what keeps a stray `<` in your prose from becoming
markup.

```html
<!doctype html>
<html lang="{{ page.language }}">
<head>
<meta charset="utf-8">
<title>{{ page.title }}</title>
{%- if site.highlight_css %}
<link rel="stylesheet" href="{{ site.highlight_css }}">
{%- endif %}
</head>
<body>{{ page.content | safe }}</body>
</html>
```

The `site.highlight_css` line is the stylesheet the build generates from your
syntax themes. Leave it out and code blocks come out unstyled.

## Building it

```bash
sqzass build -i mysite
```

The output lands in `mysite/public`. Every page is written as
`<path>/index.html`, never `<path>.html`, so the URL is `/about/` and not
`/about.html` — see [Writing content](@/content/_index.md) for why that matters
on a host with no rewrite rules. A sitemap and a `robots.txt` come out too.

## Working on it

`sqzass serve` rebuilds when a file changes. Nothing is written to `public/`
while it runs, so a rebuilding site never serves a half-written file. The page
reloads itself; a change that only touched CSS swaps the stylesheet in place
and keeps your scroll position.

> [!NOTE]
> The dev server is a development tool. It does no caching, no compression and
> no access control. Serve the built directory with a real server in production.

## What to add next

A second page is a second file. `content/about.md` becomes `/about/`, and
`content/guide/_index.md` starts a section that collects everything beside it.
[Front matter](@/content/front-matter.md) lists the fields a page can carry.
