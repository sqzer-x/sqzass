# sqzass

A static site generator written in Rust.

Most generators lean on their host for pretty URLs, redirects and cache headers.
sqzass does that work itself, so the same output is correct on GitHub Pages,
Vercel, Cloudflare Pages, or a plain directory served over HTTP.

> **Status: early, and honest about it.** The pipeline works end to end —
> markdown, sections, templates, syntax highlighting, table of contents, the
> asset pipeline, search, a dev server with live reload, sitemap and robots.txt.
> The documentation site at <https://sqzass.sqzer.com> is built with it, from
> `docs/` in this repository, and CI builds that site with the binary it just
> compiled. There are no prebuilt binaries yet, and no theme system.

## Quick start

```bash
sqzass init mysite
sqzass serve -i mysite
```

`init` writes three files — `sqzass.toml`, `content/_index.md` and
`templates/page.html` — and `serve` puts them on <http://127.0.0.1:3000> with
live reload. `sqzass build -i mysite` writes the site to `mysite/public`.

Front matter is TOML, fenced with `+++`. `title` is the only required field.

```markdown
+++
title = "Installation"
weight = 10
+++

Body goes here.
```

## Design

- **Markdown on the AST, not on the output.** Link rewriting, heading anchors and
  the table of contents are tree operations via [comrak]. Running regexes over the
  final HTML — the common shortcut — silently skips any element whose attributes
  are single-quoted or unquoted.
- **Broken references stop the build.** An unresolved `@/` link, a missing
  template, two pages claiming the same URL, an unknown asset name, a misspelled
  configuration key — each is an error, not a warning you scroll past.
- **Templates resolved from a snapshot.** `templates/` is read into memory once per
  build and [minijinja]'s loader serves `include`/`extends` from that snapshot, so a
  rebuild triggered mid-save can never ingest a half-written partial.
- **Autoescaped by default.** `| safe` is opt-in, and undefined variable access is
  an error rather than an empty string.
- **Highlighting emits classes, never inline styles.** Inline colours pin one
  theme into every document forever: dark mode becomes impossible without
  rebuilding, and a strict `style-src` is off the table.
- **Reproducible.** Two builds of the same input are byte-identical, and CI
  checks it on every push.
- **Korean was not bolted on afterwards.** `page.md` and `page.ko.md` sit side by
  side, `@/` links resolve to the reader's language, and untranslated pages are
  absent from that language's navigation rather than duplicated or 404ing.
  Search matches substrings rather than words, which is the only way `최적화`
  inside `검색엔진최적화` is ever found.
- **One binary, no runtime.** No Node, no Python, no system libraries.

Runtime messages — `--help`, errors, `doctor` findings, the dev server log — are
in Korean, with no locale switch. Error identifiers (`SQZASS_E_CONTENT`) and
`doctor` check names are ASCII and stable, so scripts have something to match on.

## Building

```bash
cargo build --release
```

Requires Rust 1.97 or newer. Nothing else.

<details>
<summary>Optional: faster linking with mold</summary>

`.cargo/config.toml` is gitignored, because a linker choice is a property of your
machine and not of this project — committing one forces it on everyone who clones
and on CI. If you have [mold] installed, create the file yourself:

```toml
[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

Note `-fuse-ld=mold` is the gcc spelling; `--ld-path=` is clang-only and gcc
rejects it.

[mold]: https://github.com/rui314/mold
</details>

## Documentation

<https://sqzass.sqzer.com> — built with sqzass, from `docs/` in this repository.

## License

MIT

[comrak]: https://github.com/kivikakk/comrak
[minijinja]: https://github.com/mitsuhiko/minijinja
