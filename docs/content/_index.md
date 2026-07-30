+++
title = "sqzass"
description = "A static site generator written in Rust — fast, deterministic, and Korean-first"
weight = 0
template = "landing.html"

[extra]
hero_kicker = "A static site generator, written in Rust"
hero_title_a = "One thousand pages."
hero_title_b = "Twenty-five milliseconds."
hero_sub = "sqzass builds fast and proves it: five generators, one machine, one corpus, method published. The output is correct on any host."
cta_start = "Get started"
cta_github = "View on GitHub"

bench_title = "Measured, not claimed"
bench_note = "Each row compares sqzass with Hugo, the fastest of the four generators we measured — same machine, cold builds, median of three. Bars are to scale within each row, and the sqzass times include building the search index, sitemap and llms.txt."
zero_label = "bytes of difference between two builds of the same input — rebuilt and diffed in CI on every push"
bench_more = "Full numbers and methodology →"

features_title = "What you get"
features_more = "Everything else is in the documentation →"

quick_title = "Quick start"
quick_install = "Install — either line works"
quick_use = "Make a site"
quick_note = "The script detects your platform and verifies checksums; the cargo line builds from source. More options are in"
quick_note_link = "the install guide."

foot_benchmark = "Benchmark"
built_line = "This site is built by sqzass itself — the docs are the demo."

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
title = "Deterministic builds"
body = "The same input produces the same bytes, on any thread count. Rendering is parallel; determinism comes from the merge."

[[extra.features]]
title = "Broken references stop the build"
body = "A dead link, a missing template, a typo in front matter — each is an error with a stable identifier and exit code, not a warning you scroll past."

[[extra.features]]
title = "Highlighting at build time"
body = "Syntax highlighting emits CSS classes, never inline styles — so dark mode stays cheap and a strict CSP stays possible."

[[extra.features]]
title = "Search that can read Korean"
body = "A substring index instead of a word index, because particles and compounds defeat stemmers. Measured on a 2,000-page corpus before it was chosen."

[[extra.features]]
title = "Two languages, side by side"
body = "English and Korean pages pair by filename. Links resolve per language, and an untranslated page never leaves a dead link behind."

[[extra.features]]
title = "One small binary"
body = "A 7.3 MB static binary with a live-reload dev server built in. No runtime to install, no plugins to version."
+++

Most generators lean on their host for pretty URLs, redirects and cache
headers. sqzass does that work itself, so the same output is correct on GitHub
Pages, Cloudflare Pages, or a plain directory served over HTTP.

Its speed claims are measurements with a published method, and its failures
are loud: a broken reference stops the build and names the file, the line and
the fix.
