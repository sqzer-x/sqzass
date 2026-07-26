+++
title = "Feeds"
description = "One Atom feed per language, from the dates you put in front matter"
weight = 25
toc = true
+++

Give a page a `date` and it enters that language's feed.

```toml
+++
title = "Release 0.2.0"
description = "What changed"
date = 2026-07-26
+++
```

```
public/
├── feed-en.xml
└── feed-ko.xml
```

## No dates, no feed

A language with no dated pages gets no file, and templates get no
`site.feed` to link. An empty feed is worse than an absent one: a subscriber who
sees nothing arriving reads it as broken rather than as empty on purpose.

This documentation has no dated pages, so this site publishes no feed. That is
the feature working.

## Autodiscovery

```html
{% if site.feed %}
<link rel="alternate" type="application/atom+xml" title="{{ site.title }}" href="{{ site.feed }}">
{% endif %}
```

`site.feed` is the current language's feed, or nothing. The `if` is not
defensive — it is the whole rule.

## Atom, not RSS 2.0

RSS dates are RFC 2822: `Tue, 26 Jul 2026 00:00:00 +0000`. That format needs a
day-of-week we would have to compute and English month names we would have to
embed — including in a Korean feed, where `Jul` is simply wrong.

Atom uses RFC 3339, which is the shape a TOML date already has. Less code, one
fewer thing to get subtly wrong, and every reader written this century handles
Atom.

## What is in it

The 20 most recent dated pages, newest first, each with a title, a permalink, an
`updated` timestamp, and the `description` as its summary. Twenty is a cap, not
a coincidence — a feed that grows without limit eventually becomes a download.

Pages within a day are ordered by title, so two posts dated the same day come
out in the same order on every build.

A date with no time becomes midnight UTC. Atom will not accept a bare date, and
a reader that cannot parse a timestamp drops the entry without telling anyone.

An offset is carried through as written — `2026-07-26T09:00:00+09:00` stays that,
because RFC 3339 accepts any offset and rewriting it as `Z` would publish a
morning in Seoul as an evening in Seoul. Ordering still compares instants rather
than the text, so a `+09:00` morning sorts before a `Z` afternoon of the same day.

A `date` that TOML reads as a time with no date — `10:30:00` — is an error rather
than a value quietly dropped. You wrote a date; the build should not disagree in
silence.

## Sorting by date

```toml
# content/posts/_index.md
+++
title = "Posts"
sort_by = "date"
+++
```

**Newest first**, unlike `weight` and `title`, which ascend. It is the order
anyone reading a dated list expects. Pages without a date go last rather than
first, which is what happens if you sort a missing date as zero.

## Showing the date

There is no date filter and no format string. `page.date` is the parts:

| | |
|---|---|
| `page.date.year` | `2026` |
| `page.date.month` | `7` |
| `page.date.day` | `26` |
| `page.date.date` | `2026-07-26`, for `<time datetime>` |
| `page.date.iso` | `2026-07-26T00:00:00Z` |

```html
{% if page.date %}
<time datetime="{{ page.date.date }}">{{ page.date.year }}년 {{ page.date.month }}월 {{ page.date.day }}일</time>
{% endif %}
```

A format-string filter would mean shipping a date-formatting mini-language and
then owning locale rules for every language someone writes in. The parts are
data, and a template already knows how to arrange data.
