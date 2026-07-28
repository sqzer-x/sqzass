+++
title = "Search"
description = "A substring index, one file per language, and why it is not a word index"
weight = 20
toc = true
+++

Every build writes one index per language:

```
public/
├── search-en.json
└── search-ko.json
```

Each row is a page: its title, description, section, URL and body text. The
client fetches the file for the current language the first time someone opens
search, and scans it for substrings.

## Why substrings

The usual approach is a word index: split the text into words at build time,
look the query's words up at search time. It is smaller and it is faster, and
for Korean it is wrong.

Korean attaches particles to nouns, so `생성기` appears in running text as
`생성기는`, `생성기를`, `생성기가`. A word index can survive that with prefix
matching. What it cannot survive is that Korean writes compounds without
spaces: `최적화` inside `검색엔진최적화` is not a prefix of anything, and a word
index will never return it. Neither will it find `존성` inside `의존성이`.

The apparent fix is to run a morphological analyser at index time, and it makes
things worse. The dictionaries are general-purpose and do not know loanwords,
which is most of a technical vocabulary: `템플릿은` comes back as
`템플`+`릿`+`은`, and the word `템플릿` stops matching the pages that are about
it. Measured on a 2000-page Korean corpus, recall for `템플릿` fell from 1018
pages to 27.

Nor can the browser be taught to compensate. `Intl.Segmenter("ko")` returns
Korean 어절 whole — ICU ships dictionary-based breakers for Chinese, Japanese
and Thai, and none for Korean — so an index of morphemes has no way to agree
with the query typed against it.

Scanning the text for substrings has none of these problems, in any language,
and it costs a JSON file.

## What it costs

The index is the body text of every page. For this site that is a couple of
kilobytes per language, fetched once, on the first search. It grows linearly
with your documentation, and there is a size past which this is the wrong
design — but that size is far beyond a documentation site, and reaching it is a
better problem than shipping search that cannot find your own words.

A site that ships no search UI should not pay even that:

```toml
[search]
enabled = false
```

skips the index entirely.

## Ranking

Every term in the query must appear somewhere in a row — an AND, not an OR.
A hit in the title outranks one in the description, which outranks one in the
body, and a title that *starts* with the term outranks one that merely contains
it. Results are capped at twelve, which is as many as anyone reads.

Code blocks are indexed, because in documentation people search for the command
they half-remember.

## The row schema

The client is yours to build, so here is what it reads. One row per page:

| Key | | |
|---|---|---|
| `t` | title | always present |
| `d` | description | omitted when empty |
| `u` | URL | always present, and it carries the subpath if you have one |
| `s` | parent section title | omitted when empty, and on section index pages |
| `c` | body plain text, code blocks included | always present |

Keys are one character because they repeat on every row. `"title"` instead of
`"t"` costs a few tens of kilobytes across a few hundred pages, for a file
nobody reads by hand.

## The client

`search.js` on this site is about 150 lines and has no dependencies. The dialog
is a `<dialog>`, so Escape, the backdrop, the focus trap and returning focus to
the trigger are the platform's behaviour rather than code. Open it with
`Ctrl`/`⌘` + `/`.

It is example code, not a feature of the tool: sqzass writes the index, and
what you build on top of it is yours.
