+++
title = "Benchmark"
description = "Five generators, one machine, one corpus — and the definitions published"
weight = 60
toc = true
+++

## minimal — a heading and a paragraph

<div class="bench">
<span class="b-name">sqzass</span><span class="b-row"><svg><rect class="b-us" width="0.6%" height="100%" rx="3"></rect></svg><span class="b-val">18 ms</span></span>
<span class="b-name">Hugo</span><span class="b-row"><svg><rect width="2%" height="100%" rx="3"></rect></svg><span class="b-val">93 ms</span></span>
<span class="b-name">Zola</span><span class="b-row"><svg><rect width="3%" height="100%" rx="3"></rect></svg><span class="b-val">141 ms</span></span>
<span class="b-name">Jekyll</span><span class="b-row"><svg><rect width="9.5%" height="100%" rx="3"></rect></svg><span class="b-val">442 ms</span></span>
<span class="b-name">Astro</span><span class="b-row"><svg><rect width="100%" height="100%" rx="3"></rect></svg><span class="b-val">4,672 ms</span></span>
</div>

## blog — six paragraphs, a list, a quote, a link

<div class="bench">
<span class="b-name">sqzass</span><span class="b-row"><svg><rect class="b-us" width="0.6%" height="100%" rx="3"></rect></svg><span class="b-val">25 ms</span></span>
<span class="b-name">Hugo</span><span class="b-row"><svg><rect width="1.7%" height="100%" rx="3"></rect></svg><span class="b-val">112 ms</span></span>
<span class="b-name">Zola</span><span class="b-row"><svg><rect width="2.6%" height="100%" rx="3"></rect></svg><span class="b-val">172 ms</span></span>
<span class="b-name">Jekyll</span><span class="b-row"><svg><rect width="11.2%" height="100%" rx="3"></rect></svg><span class="b-val">752 ms</span></span>
<span class="b-name">Astro</span><span class="b-row"><svg><rect width="100%" height="100%" rx="3"></rect></svg><span class="b-val">6,699 ms</span></span>
</div>

## heavy, repeated code — five identical 20-line Rust blocks per page

<div class="bench">
<span class="b-name">sqzass</span><span class="b-row"><svg><rect class="b-us" width="0.85%" height="100%" rx="3"></rect></svg><span class="b-val">187 ms</span></span>
<span class="b-name">Hugo</span><span class="b-row"><svg><rect width="16.1%" height="100%" rx="3"></rect></svg><span class="b-val">3,559 ms</span></span>
<span class="b-name">Zola</span><span class="b-row"><svg><rect width="23%" height="100%" rx="3"></rect></svg><span class="b-val">5,084 ms</span></span>
<span class="b-name">Jekyll</span><span class="b-row"><svg><rect width="49.1%" height="100%" rx="3"></rect></svg><span class="b-val">10,850 ms</span></span>
<span class="b-name">Astro</span><span class="b-row"><svg><rect width="100%" height="100%" rx="3"></rect></svg><span class="b-val">22,090 ms</span></span>
</div>

## heavy, unique code — five distinct 20-line Rust blocks per page

<div class="bench">
<span class="b-name">sqzass</span><span class="b-row"><svg><rect class="b-us" width="5.4%" height="100%" rx="3"></rect></svg><span class="b-val">1,298 ms</span></span>
<span class="b-name">Hugo</span><span class="b-row"><svg><rect width="24.7%" height="100%" rx="3"></rect></svg><span class="b-val">5,912 ms</span></span>
<span class="b-name">Zola</span><span class="b-row"><svg><rect width="28.3%" height="100%" rx="3"></rect></svg><span class="b-val">6,789 ms</span></span>
<span class="b-name">Jekyll</span><span class="b-row"><svg><rect width="47.8%" height="100%" rx="3"></rect></svg><span class="b-val">11,456 ms</span></span>
<span class="b-name">Astro</span><span class="b-row"><svg><rect width="100%" height="100%" rx="3"></rect></svg><span class="b-val">23,958 ms</span></span>
</div>

## Every number

Wall clock, median of three cold runs, and peak RSS.

| | minimal | blog | heavy · repeat | heavy · unique |
|---|---|---|---|---|
| **sqzass 0.1.0** | **18 ms** · 15 MB | **25 ms** · 24 MB | **187 ms** · 229 MB | **1,298 ms** · 439 MB |
| Hugo 0.164.0 | 93 ms · 123 MB | 112 ms · 140 MB | 3,559 ms · 411 MB | 5,912 ms · 421 MB |
| Zola 0.22.1 | 141 ms · 136 MB | 172 ms · 142 MB | 5,084 ms · 269 MB | 6,789 ms · 279 MB |
| Jekyll 4.4.1 | 442 ms · 56 MB | 752 ms · 62 MB | 10,850 ms · 174 MB | 11,456 ms · 182 MB |
| Astro 5.18.2 | 4,672 ms · 555 MB | 6,699 ms · 604 MB | 22,090 ms · 2,578 MB | 23,958 ms · 2,607 MB |

## Why heavy comes twice

Because code repetition decides rankings, and most benchmarks do not say what
theirs is. sqzass highlights each distinct block once per build, so a corpus
whose generator repeats the same block is a different measurement from one
where every block is unique — the gap between 187 ms and 1,298 ms *is* that
variable. A single "heavy" number that does not state its repetition has
hidden the thing that produced it.

## Method

- **Cold.** Before every run the output directory and each tool's caches are
  removed — `.jekyll-cache`, Astro's `cacheDir` and `dist`, Hugo's `resources`
  and build lock. Wall clock and peak RSS come from GNU time.
- **Corpus.** 1,000 pages plus a section index. The markdown *bodies* are
  byte-identical across tools; only the front matter syntax differs.
  A paragraph is one fixed 76-character sentence repeated four times.
- **Highlighting is on everywhere**, at build time, as each tool ships it:
  syntect + Oniguruma (sqzass), Chroma (Hugo), Giallo (Zola), Rouge (Jekyll),
  Shiki (Astro). Markup granularity differs — spans on the same heavy page:
  sqzass 3,205 · Jekyll 1,800 · Zola and Astro 1,700 · Hugo 600. sqzass emits
  the most detailed markup of the five *and* the numbers above.
- **Templates.** Each tool gets the same minimal no-theme layout: an HTML
  shell around the rendered content.
- **Astro's time includes Node startup** (~1 s). It is also doing more than
  markdown-to-HTML — it is a component framework. That is the honest caveat;
  the corpus is plain markdown, which is the workload this page is about.
