+++
title = "sqzass"
description = "A static site generator written in Rust — fast and deterministic"
weight = 0
template = "landing.html"

[extra]
hero_kicker = "A static site generator, written in Rust"
hero_title_a = "A thousand pages,"
hero_title_b = "squeezed into 25 ms."
hero_sub = "The name is the promise: press a whole site out in one go. Measured against Hugo, Zola, Jekyll and Astro on the same machine, with the method published."
cta_start = "Get started"
cta_github = "View on GitHub"
install_alt = "With Rust instead:"
install_aur = "On Arch:"

bento_label = "Why sqzass"

bench_title = "Measured, not claimed"
bench_note = "vs Hugo, the fastest of the four we measured — same machine, cold builds, median of three. Bars are to scale, and the sqzass times include the search index, sitemap and llms.txt."
bench_more = "Full numbers and methodology →"
zero_label = "bytes of difference between two builds of the same input — checked in CI on every push"

foot_benchmark = "Benchmark"

[[extra.bench]]
label = "1,000 pages, minimal markdown"
us_ms = "18"
us_w = "19.4%"
them_ms = "93"
them_w = "100%"

[[extra.bench]]
label = "1,000-page blog corpus"
us_ms = "25"
us_w = "22.3%"
them_ms = "112"
them_w = "100%"

[[extra.bench]]
label = "Code-heavy, every block unique"
us_ms = "1,298"
us_w = "22.0%"
them_ms = "5,912"
them_w = "100%"

[[extra.features]]
title = "One small binary"
body = "A 7.3 MB static binary with a live-reload dev server built in. No runtime to install, no plugins to version."

[[extra.features]]
title = "Broken references stop the build"
body = "A dead link, a missing template, a typo in front matter — each one stops the build and names the file, the line and the fix. Every error has a stable identifier and exit code, so scripts can handle them too."
+++

Most generators lean on their host for pretty URLs, redirects and cache
headers. sqzass does that work itself, so the same output is correct on GitHub
Pages, Cloudflare Pages, or a plain directory served over HTTP.

Its speed claims are measurements with a published method, and its failures
are loud: a broken reference stops the build and names the file, the line and
the fix.
