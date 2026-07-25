+++
title = "Choosing a template"
description = "Four steps, in order, and no cascade"
weight = 10
toc = true
+++

A page is rendered with the first of these that exists:

1. the `template` named in the page's front matter
2. `section.html`, if the page is a section index
3. the `page_template` set on the parent section's `_index.md`
4. `page.html`

That is the whole rule. Naming a `template` that does not exist is an error,
and the message lists the templates you do have.

## Why it is four steps and not twenty

Hugo resolves templates through a lookup order built from kind × section × type
× layout × language × output format. It is more powerful, and it is the single
thing Hugo users get lost in most often — there is a decade-old request open
asking the tool to at least *print* which template it picked.

Four steps you can hold in your head need no such command. If you cannot tell
which template rendered a page, the rule is too complicated.

## Setting a section's default

```toml
# content/blog/_index.md
+++
title = "Blog"
page_template = "post.html"
+++
```

Every page in `content/blog/` now renders with `post.html` unless it names its
own. Note that this reaches direct children only; a subsection sets its own.

## Extending a base

The usual arrangement is one skeleton and thin templates on top:

```html
{# templates/base.html #}
<!doctype html>
<html lang="{{ page.language }}">
<head><title>{{ page.title }}</title></head>
<body>{% block content %}{% endblock %}</body>
</html>
```

```html
{# templates/page.html #}
{% extends "base.html" %}
{% block content %}<article class="prose">{{ page.content | safe }}</article>{% endblock %}
```
