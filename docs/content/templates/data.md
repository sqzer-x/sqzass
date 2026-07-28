+++
title = "Template data"
description = "Everything a template can read"
weight = 20
toc = true
+++

Two objects are in scope on every page: `site` and `page`.

## site

| | |
|---|---|
| `site.title` | From `sqzass.toml`. |
| `site.description` | From `sqzass.toml`. |
| `site.base_url` | Trailing slash removed, so `{{ site.base_url }}{{ page.url }}` is always right. |
| `site.language` | The language of the page being rendered. |
| `site.sections` | Top-level sections **in this language**. |
| `site.highlight_css` | URL of the generated highlight stylesheet, or nothing if highlighting is off. |
| `site.search` | URL of this language's search index, or nothing with `[search] enabled = false`. |

`site.sections` contains only the current language's tree, which is what makes
navigation safe: an untranslated page is not in it, so a link to it cannot be
drawn. Each section carries `title`, `description`, `url`, `weight`, `pages`
and `subsections`, and each entry in `pages` has `title`, `description`, `url`
and `weight`.

## page

| | |
|---|---|
| `page.title` | |
| `page.description` | |
| `page.url` | `/ko/start/installation/` |
| `page.permalink` | `base_url` + `url`. |
| `page.content` | Rendered HTML. **Needs `\| safe`.** |
| `page.weight`, `page.draft`, `page.language` | Front matter, as given. |
| `page.toc` | Whether the author asked for a contents list. |
| `page.toc_entries` | The contents themselves — `{level, id, title, children}`, nested. |
| `page.translations` | Only languages this page exists in. Empty means no switcher. |
| `page.section` | The section this page belongs to, or nothing at the top level. |
| `page.prev`, `page.next` | The neighbouring pages **within this section**. |
| `page.children` | A section's own pages. Empty on ordinary pages. |
| `page.is_section` | |
| `page.extra` | Your `[extra]` table. |

## page.children is two things

On the root `_index.md` it holds the top-level sections. On any other section it
holds that section's own pages, followed by its subsections. Both are what a
listing template needs, and neither is obvious from the name.

```html
{% for child in page.children %}
<a href="{{ child.url }}">{{ child.title }}</a>
{%- if child.description %}<p>{{ child.description }}</p>{% endif %}
{% endfor %}
```

Ordinary pages have an empty list.

## page.toc_entries

`{level, id, title, children}`, nested by relative depth — an h2 followed by an
h4 nests, without assuming the levels are consecutive. It is collected for every
page, whether or not `toc = true`; the front matter field is the author's
intent, and the data is there either way so a template can decide.

Rendering it needs a recursive macro:

```html
{% macro toc_list(entries) %}
<ul>
  {%- for e in entries %}
  <li><a href="#{{ e.id }}">{{ e.title }}</a>
    {%- if e.children %}{{ toc_list(e.children) }}{% endif %}
  </li>
  {%- endfor %}
</ul>
{% endmacro %}

{% if page.toc and page.toc_entries %}{{ toc_list(page.toc_entries) }}{% endif %}
```

## asset()

`asset("css/main.css")` returns the hashed URL that file was written to:

```html
<link rel="stylesheet" href="{{ asset("css/main.css") }}">
```

Asking for a file that was not collected is an error, so a renamed stylesheet
fails the build instead of silently 404ing for every visitor.

## Slashes are not escaped

Jinja2 escapes five characters. Some ports escape `/` as well, which turns
every URL on every page into `href="https:&#x2f;&#x2f;…"`. sqzass restores
Jinja2's own behaviour, so URLs come out as URLs.

## Missing keys stop the build

```
undefined value: page.descriptoin
```

Rather than an empty string where your description was. See
[Templates](@/templates/_index.md).
