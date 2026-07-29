#!/usr/bin/env python3
"""ssg 벤치마크 코퍼스 생성기. 본문 마크다운은 도구 간 바이트 동일, front matter 형식만 다르다."""
import os, shutil, subprocess, sys

S = os.path.dirname(os.path.abspath(__file__))
N = 1000
PARA = "The quick brown fox jumps over the lazy dog while the build system watches. " * 4

def code_block(page, blk, unique):
    tag = f"page{page}_blk{blk}_" if unique else ""
    lines = "\n".join(
        f"fn {tag}handler_{j}(req: Request) -> Response {{ Response::ok(req.body_{tag}{j}()) }}"
        for j in range(20)
    )
    return f"```rust\n{lines}\n```"

def body(scenario, i):
    if scenario == "minimal":
        return f"# Post {i}\n\n{PARA}\n"
    if scenario == "blog":
        nxt = f"/posts/post-{(i + 1) % N}/"
        return (f"# Post {i}\n\n" + "\n\n".join([PARA] * 6)
                + f"\n\n- one\n- two\n- three\n\n> quote block\n\n[next]({nxt})\n")
    unique = scenario == "heavyu"
    parts = [f"# Post {i}"]
    for b in range(5):
        parts.append(PARA)
        parts.append(code_block(i, b, unique))
    return "\n\n".join(parts) + "\n"

def w(path, content):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        f.write(content)

SCENARIOS = ["minimal", "blog", "heavyr", "heavyu"]
SQZASS = os.environ.get("SQZASS", "target/release/sqzass")

for sc in SCENARIOS:
    # --- sqzass: init 스캐폴드 + TOML +++ front matter
    d = f"{S}/sqzass/{sc}"
    if not os.path.exists(d):
        subprocess.run([SQZASS, "init", d], check=True, capture_output=True)
    w(f"{d}/content/posts/_index.md", '+++\ntitle = "posts"\n+++\n')
    for i in range(N):
        w(f"{d}/content/posts/post-{i}.md", f'+++\ntitle = "Post {i}"\n+++\n\n{body(sc, i)}')

    # --- hugo: TOML +++ front matter (동일 형식)
    d = f"{S}/hugo/{sc}"
    w(f"{d}/hugo.toml", 'baseURL = "https://example.com/"\ntitle = "bench"\n')
    w(f"{d}/layouts/_default/single.html",
      "<!doctype html><html><head><meta charset=\"utf-8\"><title>{{ .Title }}</title></head>"
      "<body><main>{{ .Content }}</main></body></html>\n")
    w(f"{d}/layouts/_default/list.html",
      "<!doctype html><html><head><meta charset=\"utf-8\"><title>{{ .Title }}</title></head>"
      "<body><main>{{ range .Pages }}<a href=\"{{ .RelPermalink }}\">{{ .Title }}</a> {{ end }}</main></body></html>\n")
    for i in range(N):
        w(f"{d}/content/posts/post-{i}.md", f'+++\ntitle = "Post {i}"\n+++\n\n{body(sc, i)}')

    # --- zola: TOML +++ front matter, highlight_code 명시적 활성화(기본 꺼짐)
    d = f"{S}/zola/{sc}"
    w(f"{d}/config.toml",
      'base_url = "https://example.com"\ntitle = "bench"\ncompile_sass = false\n'
      'build_search_index = false\n\n[markdown.highlighting]\ntheme = "github-dark"\n')
    w(f"{d}/templates/index.html",
      "<!doctype html><html><head><meta charset=\"utf-8\"><title>bench</title></head><body><main>index</main></body></html>\n")
    w(f"{d}/templates/page.html",
      "<!doctype html><html><head><meta charset=\"utf-8\"><title>{{ page.title }}</title></head>"
      "<body><main>{{ page.content | safe }}</main></body></html>\n")
    w(f"{d}/templates/section.html",
      "<!doctype html><html><head><meta charset=\"utf-8\"><title>{{ section.title }}</title></head>"
      "<body><main>{% for p in section.pages %}<a href=\"{{ p.permalink }}\">{{ p.title }}</a> {% endfor %}</main></body></html>\n")
    w(f"{d}/content/posts/_index.md", '+++\ntitle = "posts"\n+++\n')
    for i in range(N):
        w(f"{d}/content/posts/post-{i}.md", f'+++\ntitle = "Post {i}"\n+++\n\n{body(sc, i)}')

    # --- jekyll: YAML --- front matter
    d = f"{S}/jekyll/{sc}"
    w(f"{d}/_config.yml",
      'title: bench\ndefaults:\n  - scope: { path: "" }\n    values: { layout: default }\n')
    w(f"{d}/_layouts/default.html",
      "<!doctype html><html><head><meta charset=\"utf-8\"><title>{{ page.title }}</title></head>"
      "<body><main>{{ content }}</main></body></html>\n")
    for i in range(N):
        w(f"{d}/posts/post-{i}.md", f'---\ntitle: "Post {i}"\n---\n\n{body(sc, i)}')

    # --- astro: YAML front matter + layout 지정, node_modules는 공용 심링크
    d = f"{S}/astro/{sc}"
    os.makedirs(d, exist_ok=True)
    shutil.copy(f"{S}/astro-base/package.json", f"{d}/package.json")
    w(f"{d}/astro.config.mjs", "export default { cacheDir: './.astro-cache' };\n")
    if not os.path.islink(f"{d}/node_modules"):
        os.symlink(f"{S}/astro-base/node_modules", f"{d}/node_modules")
    w(f"{d}/src/layouts/Base.astro",
      "---\nconst { frontmatter } = Astro.props;\n---\n"
      "<!doctype html><html><head><meta charset=\"utf-8\"><title>{frontmatter.title}</title></head>"
      "<body><main><slot /></main></body></html>\n")
    for i in range(N):
        w(f"{d}/src/pages/posts/post-{i}.md",
          f'---\nlayout: ../../layouts/Base.astro\ntitle: "Post {i}"\n---\n\n{body(sc, i)}')

print("corpora ready:", ", ".join(SCENARIOS))
