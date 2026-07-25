+++
title = "Social cards and structured data"
description = "Markup you own, from values that already exist"
weight = 50
toc = true
+++

sqzass injects nothing into `<head>`. There is no injection point and there will
not be one — a generator that quietly adds tags is a generator you cannot fully
read the output of. Everything below is markup you put in your own `base.html`,
built from values already in the template context.

## OpenGraph and Twitter

Without these, every page of your site renders as a bare URL in Slack, Discord
and KakaoTalk.

```html
<meta property="og:type" content="{{ "website" if page.is_section else "article" }}">
<meta property="og:site_name" content="{{ site.title }}">
<meta property="og:title" content="{{ page.title }}">
<meta property="og:url" content="{{ page.permalink }}">
<meta property="og:locale" content="{{ "ko_KR" if page.language == "ko" else "en_US" }}">
{%- if page.description %}
<meta property="og:description" content="{{ page.description }}">
{%- endif %}
<meta name="twitter:card" content="summary">
```

`summary`, not `summary_large_image` — the large variant needs an image, and
declaring it without one produces an empty box rather than a nicer card. If you
have a per-page image, put it in front matter and switch:

```toml
+++
title = "Installation"
[extra]
image = "/images/install.png"
+++
```

```html
{%- if page.extra.image %}
<meta property="og:image" content="{{ site.origin }}{{ page.extra.image }}">
<meta name="twitter:card" content="summary_large_image">
{%- else %}
<meta name="twitter:card" content="summary">
{%- endif %}
```

`og:image` must be absolute, which is what `site.origin` is for.

## Breadcrumbs as JSON-LD

This is the one piece of structured data that visibly changes a Google result
for a documentation site: the result shows `Home › Writing content › Front matter`
instead of a bare URL.

```html
{%- if page.section %}
<script type="application/ld+json">
{
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  "itemListElement": [
    {"@type": "ListItem", "position": 1, "name": "{{ site.title }}", "item": "{{ site.origin }}{{ site.base_path }}/"},
    {"@type": "ListItem", "position": 2, "name": "{{ page.section.title }}", "item": "{{ site.origin }}{{ page.section.url }}"},
    {"@type": "ListItem", "position": 3, "name": "{{ page.title }}", "item": "{{ page.permalink }}"}
  ]
}
</script>
{%- endif %}
```

Note the `{%- if page.section %}`: top-level pages have no section, and a
breadcrumb with a missing rung is worse than none.

> [!WARNING]
> This block is inside `<script>`, where HTML escaping is wrong — a title
> containing `"` produces invalid JSON. Keep titles free of quotes, or drop the
> structured data rather than shipping JSON that silently fails to parse. This
> is the one place in a template where our escaping does not protect you.

## Site-level

```html
<script type="application/ld+json">
{
  "@context": "https://schema.org",
  "@type": "WebSite",
  "name": "{{ site.title }}",
  "url": "{{ site.origin }}{{ site.base_path }}/"
}
</script>
```

One block, on every page, is enough. Search engines read it once.
