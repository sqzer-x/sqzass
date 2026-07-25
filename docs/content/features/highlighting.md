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

## Turning it off

`enabled = false` skips highlighting and emits no stylesheet. Code blocks still
get `class="language-rust"`, so a client-side highlighter can pick them up.
