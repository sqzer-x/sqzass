+++
title = "Development server"
description = "Serving from memory, rebuilds on change, and reloads that keep your place"
weight = 40
toc = true
+++

```bash
sqzass serve -i docs
```

<http://127.0.0.1:3000>, rebuilding whenever anything in `content/`, `templates/`,
`static/` or `i18n/` changes, or `sqzass.toml` itself. The output directory is
deliberately not watched: a build that triggered on its own output would never
stop rebuilding.

| Flag | | |
|---|---|---|
| `-i`, `--input` | `.` | Site root. |
| `-b`, `--bind` | `127.0.0.1` | Bind address. |
| `-p`, `--port` | `3000` | Port. |
| `--drafts` | | Include draft pages. |
| `--base-url` | | Override `base_url`. |

## Nothing is written to disk

The build goes into memory and is served from there. `public/` is not touched
while the server runs.

That is not an optimisation. A browser that requests a file mid-rebuild would
otherwise get whatever bytes had been written so far, and the resulting
half-page is the kind of bug you chase for an hour before realising it was not
your code. Serving a build that is complete or not served at all removes the
window entirely.

## Reloading

The reload script is injected as the page is served, not written into the
build, so the output stays byte-identical to what a production build produces.

A change that touched only CSS swaps the stylesheet's `href` in place instead
of reloading — the page does not move, and you keep your scroll position while
you nudge a margin. Anything else reloads the page.

## When a build fails

The error is shown in the browser rather than only in the terminal you may not
be looking at, and the last good version stays served underneath. Fix the file,
save, and the overlay goes away.

## It is not a production server

No caching, no compression, no access control, no TLS. It binds to localhost by
default for that reason. `--bind 0.0.0.0` will let a phone on your network see
the site, which is useful and is as far as it should go.
