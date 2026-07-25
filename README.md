# sqzass

A static site generator written in Rust.

Most generators lean on their host for pretty URLs, redirects and cache headers.
sqzass does that work itself, so the same output is correct on GitHub Pages,
Vercel, Cloudflare Pages, or a plain directory served over HTTP.

> **Status: early.** The build pipeline works end to end. Syntax highlighting,
> table of contents, the asset pipeline, search and the dev server are not built
> yet. Not usable for a real site.

## Design

- **Markdown on the AST, not on the output.** Link rewriting, heading anchors and
  the table of contents are tree operations via [comrak]. Running regexes over the
  final HTML — the common shortcut — silently skips any element whose attributes
  are single-quoted or unquoted.
- **Templates resolved from a snapshot.** `templates/` is read into memory once per
  build and [minijinja]'s loader serves `include`/`extends` from that snapshot, so a
  rebuild triggered mid-save can never ingest a half-written partial.
- **Autoescaped by default.** `| safe` is opt-in, and undefined variable access is
  an error rather than an empty string.
- **Reproducible.** Two builds of the same input are byte-identical. Two pages
  claiming the same URL is a hard error, not a race.
- **Bilingual from the start.** `page.md` and `page.ko.md` sit side by side; the
  default language lives at the root and others under `/<code>/`.
- **One binary, no runtime.** No Node, no Python, no system libraries.

## Usage

```bash
sqzass build -i <site-dir>
```

A site is a directory containing `sqzass.toml`, `content/` and `templates/`.
Front matter is TOML, fenced with `+++`:

```markdown
+++
title = "Installation"
weight = 10
+++

Body goes here.
```

Output goes to `public/` as `path/index.html`, plus a `.nojekyll` marker.

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
