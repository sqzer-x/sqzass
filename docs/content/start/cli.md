+++
title = "Command line"
description = "Commands, flags, exit codes and machine-readable output"
weight = 40
toc = true
+++

```
sqzass init  [DIR]
sqzass build [-i DIR] [-o DIR] [--drafts] [--base-url URL]
sqzass serve [-i DIR] [-b ADDR] [-p PORT] [--drafts] [--base-url URL]
```

`--json` works on any of them.

## init

Writes a new site into `DIR` (default `.`), creating it if needed. Refuses to
run where a `sqzass.toml` already exists. See
[Your first site](@/start/first-site.md).

## build

| Flag | Default | |
|---|---|---|
| `-i`, `--input` | `.` | Site root — the directory holding `sqzass.toml`. |
| `-o`, `--output` | `<input>/public` | Resolved against your shell's directory, not the site root. |
| `--drafts` | | Include pages marked `draft = true`. |
| `--base-url` | from config | Useful for preview deployments. |

The output directory is emptied first, so a page you deleted does not linger as
a ghost in the built site.

## serve

Serves from memory with live reload on <http://127.0.0.1:3000>. See
[Development server](@/features/dev-server.md).

## Exit codes

A build either succeeds or tells you which part of your site it could not
accept. The code is the answer to "whose fault is it" — CI can branch on it
without parsing text.

| Code | Identifier | |
|---|---|---|
| `0` | | Success. |
| `1` | `SQZASS_E` | Something unclassified went wrong. |
| `2` | | Bad command line. This one is clap's, not ours. |
| `3` | `SQZASS_E_CONFIG` | `sqzass.toml` — unreadable, malformed, or a key that does not exist. |
| `4` | `SQZASS_E_CONTENT` | Something under `content/` — front matter, a missing title, two pages claiming one URL, an unresolved `@/` link. |
| `5` | `SQZASS_E_TEMPLATE` | `templates/`, `i18n/`, or an asset a template asked for and did not get. |
| `6` | `SQZASS_E_IO` | Reading or writing failed. |

The identifier is printed with the message, so it can be searched for:

```
error: [SQZASS_E_CONTENT] content/_index.md: 해석할 수 없는 내부 링크가 있습니다:
  @/nope.md
```

These numbers are a contract. They will not be reassigned to different meanings,
because a condition in someone's pipeline should not quietly start testing for
something else.

## --json

One JSON object on stdout, and nothing else, so a script reads a single pipe.

```console
$ sqzass build -i docs --json
{"ok":true,"output":"docs/public","pages":40}
```

Failures go to **stdout** too, rather than being split across two streams:

```console
$ sqzass build -i broken --json
{"code":4,"error":"[SQZASS_E_CONTENT] …","kind":"SQZASS_E_CONTENT","ok":false}
$ echo $?
4
```

Without `--json`, messages go to stderr and results to stdout, which is what a
person at a terminal expects.
