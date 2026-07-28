+++
title = "Syntax highlighting"
description = "Highlighted at build time, as classes, in two themes"
weight = 10
toc = true
+++

Code blocks are highlighted while the site is built. No JavaScript runs in the
reader's browser to colour them, so the page is coloured on first paint.

```toml
[highlight]
enabled     = true
theme_light = "InspiredGitHub"
theme_dark  = "base16-ocean.dark"
```

## Classes, never inline styles

A highlighted block looks like this:

```html
<code class="language-rust"><span class="hl-source hl-rust">…</span></code>
```

Not `style="color:#268bd2"`. The distinction decides three things.

**Dark mode is possible at all.** Inline colours pin one theme into every
document you have ever generated. Changing it means rebuilding the site;
following the reader's preference means shipping both themes inside the markup.
With classes, the two themes are two blocks of CSS, and switching is a CSS
switch.

**A strict `style-src` stays available.** A Content-Security-Policy that
forbids inline styles is off the table the moment your HTML is full of them.

**The stylesheet is one file.** Change a colour and every page changes, without
rebuilding a single page of HTML.

## Two themes, one stylesheet

`theme_light` and `theme_dark` are both compiled into the generated stylesheet.
The dark rules are emitted twice — once under `prefers-color-scheme: dark` for
readers who never touch a toggle, and once under `[data-theme="dark"]` for
sites like this one that offer a switch.

Any theme name from syntect's default set works. A name that does not exist is
a build error, and the message lists the ones that do.

## The prefix

Classes are prefixed `hl-`. Without a prefix, syntect emits class names like
`source`, `keyword` and `string` — words general enough to collide with your
own stylesheet on a site about programming.

## Marking lines and naming files

Options go after the language, space-separated, as `key=value`:

````markdown
```rust hl_lines=2-3 name=src/main.rs
fn main() {
    let marked = 2;
    let also_marked = 3;
}
```
````

And rendered, on this site's stylesheet:

```rust hl_lines=2-3 name=src/main.rs
fn main() {
    let marked = 2;
    let also_marked = 3;
}
```

`hl_lines` takes 1-based line numbers and closed ranges, comma-separated:
`hl_lines=2-4,7`. Each listed line is wrapped in `<mark class="hl-line">` —
`<mark>` is visible with no CSS at all, because browsers style it out of the
box, and a site restyles it with `.highlight mark`. Highlighting continues
across the boundary: marking the middle line of a block comment keeps it a
comment.

`name` labels the block with a file name. It ships as an attribute, not as
markup — `<code … data-name="src/main.rs">` — and your CSS decides whether to
show it:

```css
.prose pre code { display: block; width: max-content; min-width: 100%; }
.prose pre code[data-name]::before {
  content: attr(data-name);
  display: block;
}
```

The first rule widens `code` to the longest line. It is what keeps the label —
and `hl_lines` marks — running to the end of a block that scrolls sideways,
instead of stopping at the visible width.

Why this syntax and not Zola's `rust,hl_lines=2-4`: the comma glues the options
onto the language token, which then fails syntax lookup and pollutes
`class="language-…"`. After the first space the info string is free, and that
is where comrak itself splits it.

A typo is a build error, not a silent no-op. `hl_line=3`, `linenos=true`,
`hl_lines=9` on a three-line block, a key given twice — each stops the build
and names the file; the alternative is a page that ships unhighlighted while
looking finished. So does a fence that starts with options instead of a
language: `=` never appears in a language name, so `` ```hl_lines=2 `` is an
option in the language slot, not an unknown language. The options belong to
the highlighter, so with `enabled = false` they are neither applied nor
checked.

## Line numbers and a copy button are yours

Neither is a configuration key, and `line_numbers` was deleted rather than
shipped inert — a setting that does nothing is worse than an absent one, because
someone sets it and waits.

Ordinary blocks carry no per-line markup to hang a CSS counter on — a wrapper
around every line of every block would tax all pages for a feature most blocks
never use. A gutter is a few lines of JS over the line count:

```js
document.querySelectorAll(".prose pre > code").forEach(function (code) {
  var lines = code.textContent.split("\n").length - 1;
  var gutter = document.createElement("span");
  gutter.className = "linenos";
  for (var i = 1; i <= lines; i++) gutter.textContent += i + "\n";
  code.parentElement.prepend(gutter);
});
```

```css
.prose pre { display: flex; gap: 1em; }
.prose pre .linenos { text-align: right; color: var(--ink-3); user-select: none; }
```

A copy button is about ten lines, and the language is already on the element:

```js
document.querySelectorAll(".prose pre").forEach(function (pre) {
  var b = document.createElement("button");
  b.textContent = "copy";
  b.addEventListener("click", function () {
    navigator.clipboard.writeText(pre.textContent);
  });
  pre.appendChild(b);
});
```

Both live in your `static/`, both are yours to restyle, and neither adds a key
to a configuration file that has to keep meaning the same thing forever.

## Turning it off

`enabled = false` skips highlighting and emits no stylesheet. Code blocks still
get `class="language-rust"`, so a client-side highlighter can pick them up.
