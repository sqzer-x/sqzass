//! sqzass — Rust로 만든 정적 사이트 생성기.

pub mod config;
pub mod content;
pub mod markdown;
pub mod render;

use anyhow::{Context, Result};
use minijinja::context;
use std::path::{Path, PathBuf};

use config::Config;
use content::Page;
use render::{PageCtx, SiteCtx, Templates};

#[derive(Debug, Clone)]
pub struct BuildOptions {
    /// 사이트 루트 (`sqzass.toml`이 있는 디렉터리).
    pub input: PathBuf,
    /// 출력 디렉터리. `None`이면 `<input>/<build.output_dir>`.
    pub output: Option<PathBuf>,
    pub drafts: bool,
    /// 설정의 `base_url`을 덮어쓴다.
    pub base_url: Option<String>,
}

#[derive(Debug, Default)]
pub struct BuildStats {
    pub pages_written: usize,
    pub output_dir: PathBuf,
}

pub fn build(opts: &BuildOptions) -> Result<BuildStats> {
    let root = &opts.input;
    let mut cfg = Config::load(root)?;
    if let Some(base) = &opts.base_url {
        cfg.base_url = base.clone();
    }

    let drafts = opts.drafts || cfg.build.drafts;
    let out_dir = resolve_output_dir(opts, &cfg);

    let pages = content::discover(root, &cfg, drafts)?;
    let templates = Templates::load(root)?;
    let md = markdown::Renderer::new(&cfg.markdown);

    // 출력 디렉터리를 매번 새로 만든다. 지운 페이지가 유령으로 남는 걸 막는다.
    if out_dir.exists() {
        std::fs::remove_dir_all(&out_dir)
            .with_context(|| format!("{}을(를) 비울 수 없습니다", out_dir.display()))?;
    }

    let mut written = 0usize;
    for page in &pages {
        let html = render_page(page, &cfg, &templates, &md)?;
        let dest = out_dir.join(&page.out_path);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("{}을(를) 만들 수 없습니다", parent.display()))?;
        }
        std::fs::write(&dest, html)
            .with_context(|| format!("{}을(를) 쓸 수 없습니다", dest.display()))?;
        written += 1;
    }

    // GitHub Pages는 이 파일이 없으면 출력을 Jekyll로 한 번 더 굴려서
    // `_`로 시작하는 디렉터리를 통째로 삼킨다.
    std::fs::write(out_dir.join(".nojekyll"), "")
        .with_context(|| format!("{}에 .nojekyll을 쓸 수 없습니다", out_dir.display()))?;

    Ok(BuildStats {
        pages_written: written,
        output_dir: out_dir,
    })
}

/// `-o`가 없으면 `<input>/<output_dir>`, 있으면 **셸의 CWD 기준**으로 해석한다.
fn resolve_output_dir(opts: &BuildOptions, cfg: &Config) -> PathBuf {
    match &opts.output {
        Some(o) => o.clone(),
        None => opts.input.join(&cfg.build.output_dir),
    }
}

fn render_page(
    page: &Page,
    cfg: &Config,
    templates: &Templates,
    md: &markdown::Renderer,
) -> Result<String> {
    let template = select_template(page, templates)?;

    let site = SiteCtx {
        title: cfg.title.clone(),
        description: cfg.description.clone(),
        base_url: cfg.base_url_trimmed().to_string(),
        language: page.language.clone(),
    };

    let page_ctx = PageCtx {
        title: page.title.clone(),
        description: page.front.description.clone(),
        url: page.url.clone(),
        permalink: format!("{}{}", cfg.base_url_trimmed(), page.url),
        content: md.to_html(&page.body),
        weight: page.front.weight,
        draft: page.front.draft,
        toc: page.front.toc,
        language: page.language.clone(),
    };

    templates
        .render(&template, context! { site => site, page => page_ctx })
        .with_context(|| format!("{} 렌더 중", page.source.display()))
}

/// 템플릿 선택은 **명시적**이다: front matter `template` → 섹션이면 `section.html`
/// → `page.html`. Hugo의 20단계 lookup order 캐스케이드는 채택하지 않는다 —
/// 사용자가 가장 많이 헤매는 지점이기 때문이다.
///
/// (부모 섹션의 `page_template`을 보는 단계는 섹션 트리가 생기는 M1에서 들어간다.)
fn select_template(page: &Page, templates: &Templates) -> Result<String> {
    if let Some(explicit) = &page.front.template {
        if templates.has(explicit) {
            return Ok(explicit.clone());
        }
        anyhow::bail!(
            "{}: front matter가 template = \"{}\"를 가리키는데 그런 템플릿이 없습니다.\n\
             사용 가능한 템플릿: {}",
            page.source.display(),
            explicit,
            templates.names().join(", ")
        );
    }

    let candidate = if page.is_section {
        "section.html"
    } else {
        "page.html"
    };
    if templates.has(candidate) {
        return Ok(candidate.into());
    }
    if templates.has("page.html") {
        return Ok("page.html".into());
    }

    anyhow::bail!(
        "{}: 쓸 템플릿이 없습니다 ('{}'도 'page.html'도 없음).\n\
         사용 가능한 템플릿: {}",
        page.source.display(),
        candidate,
        templates.names().join(", ")
    )
}

/// 경로를 사람이 읽기 좋게 줄인다 (로그용).
pub fn display_path(path: &Path) -> String {
    std::env::current_dir()
        .ok()
        .and_then(|cwd| path.strip_prefix(&cwd).ok().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| path.to_path_buf())
        .display()
        .to_string()
}
