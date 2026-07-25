+++
title = "Deploying"
description = "The output is a directory. Any host that serves files will do."
weight = 50
sort_by = "weight"
+++

```bash
sqzass build -i mysite
```

`mysite/public` is the site. Copy it somewhere that serves files and you are
done — there is no runtime, no server component and no build step left to run.

## What sqzass does so the host does not have to

Every URL is a directory containing `index.html`, so pretty URLs need no
rewrite rules. Cache busting is in the filename, so it needs no cache headers.
Nothing in the output depends on host configuration.

This is deliberate, and the docs site is hosted on
[GitHub Pages](@/deploy/github-pages.md) to keep it honest: Pages offers no
custom headers, no redirect rules and no rewrites, so anything that needed them
would break here first.

The payoff is that moving hosts is a copy. The same directory is correct on
Cloudflare Pages, Netlify, S3 behind CloudFront, or nginx.

## What else is written

`sitemap.xml` lists every page, with `<xhtml:link>` alternates for pages that
exist in more than one language — which is the form Google asks for when a site
is bilingual. `robots.txt` allows everything and points at the sitemap.

Neither carries `priority` or `changefreq`, because Google confirmed in 2023
that it ignores both. Neither carries `lastmod` either, and that one is a
choice: the honest sources for it are all unreliable here. A file's mtime is
its checkout time, so in CI every page would claim to have changed this
morning, and it would break the guarantee that two builds produce the same
bytes. Git commit times are accurate but need full history, and
`actions/checkout` clones shallow by default — so it would quietly stamp every
page with the same date. Google ignores a site's `lastmod` entirely once it
finds it untrustworthy, which makes a wrong one worse than none.

`llms.txt` is a flat list of every page — title, URL and description — in the
[format proposed at llmstxt.org](https://llmstxt.org). A language model asked
about your site can read one file instead of crawling the whole thing. It costs
nothing to emit because the title, URL and description already exist.

Put your own `sitemap.xml`, `robots.txt` or `llms.txt` in `static/` and sqzass will not
generate that file at all. It does not overwrite yours, and it does not
silently ignore it either.

## Where these guides go

| | |
|---|---|
| [GitHub Pages](@/deploy/github-pages.md) | Where this site is, and the host that forced the design |
| [Netlify](@/deploy/netlify.md) | `netlify.toml`, deploy previews |
| [Vercel](@/deploy/vercel.md) | `vercel.json`, and why the framework machinery is inert |
| [Cloudflare Pages](@/deploy/cloudflare-pages.md) | Two dashboard fields, and `_headers` — the host to move to when you need cache control |
| [GitLab Pages](@/deploy/gitlab-ci.md) | `.gitlab-ci.yml`, and the project subpath |
| [Codeberg Pages](@/deploy/codeberg-pages.md) | A `pages` branch, `.domains`, Forgejo Actions |

Every one of them is the same two facts: run `sqzass build`, publish `public/`.
The pages differ only in where those facts are written down and what each host
calls its preview URL.

## Sites served under a path

`https://user.github.io/repo`, `https://group.gitlab.io/project` and
`https://user.codeberg.page/repo` are all project sites, and all of them serve
your output under a path rather than at a domain root. Put the whole thing in
`base_url`:

```toml
base_url = "https://user.github.io/repo"
```

sqzass then prefixes every URL it generates — page links, stylesheet hrefs, the
search index — while the output directory stays flat, because that directory is
the root the host serves.

Leaving the path out is the one mistake here that is silent. The build succeeds,
the pages are all there, and every link and stylesheet resolves one level too
high.

## static/ is a passthrough

Anything in `static/` lands in the output with its path and name intact, which
is how host-specific files work without sqzass knowing about any host:

| | |
|---|---|
| `CNAME` | GitHub Pages custom domain |
| `.domains` | Codeberg Pages custom domain |
| `_headers`, `_redirects` | Netlify, Cloudflare Pages |
| `.well-known/*` | Domain verification, `security.txt` |

Only CSS and JavaScript get a content hash. A name that is the contract keeps
its name.

## Two files worth knowing about

`.nojekyll` is written into every build. Without it GitHub Pages runs the
output through Jekyll, which swallows directories beginning with `_`.

`CNAME`, if you need one, goes in `static/` and is copied through with its name
intact — a hashed `CNAME` is a file GitHub will never look for.

## What a build emits

The complete list, so that "moving hosts is a copy" is something you can check
rather than take on trust:

| | |
|---|---|
| everything in `static/` | paths and names intact; CSS and JS get a content hash |
| `assets/highlight.<hash>.css` | unless `[highlight] enabled = false` |
| `asset-manifest.json` | logical name → written URL, always |
| `<path>/index.html` | one per page |
| `search-<lang>.json` | one per language |
| `sitemap.xml`, `robots.txt` | unless `static/` supplied one with that name |
| `llms.txt` | same terms |
| `404.html` | when `templates/404.html` exists |
| alias stubs | one per `aliases` entry |
| `.nojekyll` | always, and it wins over a `static/.nojekyll` |

Nothing else, and nothing outside the output directory.

## Determinism

Two builds of the same input produce byte-identical output. That makes the
build safe to run in CI as a check, and it means a deploy that changes nothing
uploads nothing.
