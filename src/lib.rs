//! sqzass — Rust로 만든 정적 사이트 생성기.

pub mod assets;
pub mod config;
pub mod content;
pub mod error;
pub mod highlight;
pub mod i18n;
pub mod init;
pub mod links;
pub mod markdown;
pub mod render;
pub mod search;
pub mod seo;
pub mod serve;
pub mod site;

use anyhow::{Context, Result};
use minijinja::context;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use error::Kind;

use config::Config;
use content::Page;
use render::{PageCtx, SiteCtx, Templates};
use site::Site;

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

/// 생성된 하이라이트 스타일시트의 논리 경로. 다른 에셋과 같은 규칙으로 해시가 붙는다.
const HIGHLIGHT_ASSET: &str = "assets/highlight.css";

#[derive(Debug, Default)]
pub struct BuildStats {
    pub pages_written: usize,
    pub output_dir: PathBuf,
}

/// 빌드 산출물을 메모리에 담는다.
///
/// 개발 서버가 디스크를 거치지 않고 이걸 그대로 서빙한다. 그래서 브라우저가 반쯤
/// 쓰인 파일을 읽는 일이 없고, 저장할 때마다 디스크에 쓰기가 증폭되지도 않는다.
#[derive(Debug, Default)]
pub struct BuildOutput {
    /// 출력 디렉터리 기준 상대 경로 → 내용
    pub files: BTreeMap<String, Vec<u8>>,
}

impl BuildOutput {
    pub fn pages(&self) -> usize {
        self.files.keys().filter(|k| k.ends_with(".html")).count()
    }
}

/// 사이트를 빌드해 메모리에 담는다. 디스크는 건드리지 않는다.
pub fn build_to_memory(opts: &BuildOptions) -> Result<BuildOutput> {
    render_site(opts, load_config(opts)?)
}

/// 설정을 읽고 명령줄 덮어쓰기를 적용한다.
///
/// 빌드 경로가 이걸 두 번 부르지 않도록 따로 뺐다. 두 번 부르면 설정 진단이 두 번
/// 찍히고, 사용자는 자기 파일에 문제가 두 개 있다고 읽는다.
fn load_config(opts: &BuildOptions) -> Result<Config> {
    let mut cfg = Config::load(&opts.input).map_err(Kind::Config.tag())?;
    if let Some(base) = &opts.base_url {
        cfg.base_url = base.clone();
    }
    Ok(cfg)
}

fn render_site(opts: &BuildOptions, cfg: Config) -> Result<BuildOutput> {
    let root = &opts.input;
    let drafts = opts.drafts || cfg.build.drafts;

    let pages = content::discover(root, &cfg, drafts).map_err(Kind::Content.tag())?;
    let site = Site::build(&pages, &cfg);

    // 에셋을 먼저 처리해야 템플릿이 해시가 붙은 최종 URL을 조회할 수 있다.
    let mut assets = assets::Assets::collect(root, &cfg.assets).map_err(Kind::Io.tag())?;

    let mut md =
        markdown::Renderer::new(&cfg.markdown).with_links(site.link_index(), &cfg.default_language);
    let highlight_css_url = if cfg.highlight.enabled {
        md = md.with_highlighter(highlight::Highlighter::new());
        // 하이라이트 스타일시트는 테마에서 생성한다. HTML은 클래스만 담고 있으므로
        // 이 파일 하나를 바꾸면 사이트 전체의 코드 색이 바뀐다.
        let css = highlight::stylesheet(&cfg.highlight).map_err(Kind::Config.tag())?;
        assets.insert(HIGHLIGHT_ASSET, css.into_bytes(), cfg.assets.fingerprint);
        assets.url(HIGHLIGHT_ASSET).map(str::to_string)
    } else {
        None
    };

    assets.write_manifest();
    let templates = Templates::load(root)
        .map_err(Kind::Template.tag())?
        .with_assets(assets.manifest.clone())
        .with_i18n(i18n::load(root).map_err(Kind::Template.tag())?);

    let mut out = BuildOutput::default();
    out.files.extend(assets.files);

    // 언어 코드 → 그 언어의 검색 색인 행들.
    let mut index: BTreeMap<String, Vec<search::Entry>> = BTreeMap::new();

    for page in &pages {
        let (html, entry) = render_page(
            page,
            &site,
            &cfg,
            &templates,
            &md,
            highlight_css_url.as_deref(),
        )?;
        out.files.insert(
            page.out_path.to_string_lossy().replace('\\', "/"),
            html.into_bytes(),
        );
        index.entry(page.language.clone()).or_default().push(entry);
    }

    for (language, entries) in index {
        let json = serde_json::to_vec(&entries)
            .with_context(|| format!("{language} 검색 색인을 만들 수 없습니다"))?;
        out.files.insert(search::path(&language), json);
    }

    // `static/`에 같은 이름이 있으면 그쪽이 이긴다. 직접 넣은 robots.txt가
    // 조용히 무시당하는 것보다는 생성을 건너뛰는 쪽이 낫다.
    let base = cfg.base_url_trimmed();
    out.files
        .entry(seo::SITEMAP_PATH.into())
        .or_insert_with(|| seo::sitemap(&pages, &site, base).into_bytes());
    out.files
        .entry(seo::ROBOTS_PATH.into())
        .or_insert_with(|| seo::robots(base).into_bytes());

    // GitHub Pages는 이 파일이 없으면 출력을 Jekyll로 한 번 더 굴려서
    // `_`로 시작하는 디렉터리를 통째로 삼킨다.
    out.files.insert(".nojekyll".into(), Vec::new());

    Ok(out)
}

/// 사이트를 빌드해 디스크에 쓴다.
pub fn build(opts: &BuildOptions) -> Result<BuildStats> {
    let cfg = load_config(opts)?;
    let out_dir = resolve_output_dir(opts, &cfg);
    let output = render_site(opts, cfg)?;

    // 출력 디렉터리를 매번 새로 만든다. 지운 페이지가 유령으로 남는 걸 막는다.
    if out_dir.exists() {
        std::fs::remove_dir_all(&out_dir)
            .with_context(|| format!("{}을(를) 비울 수 없습니다", out_dir.display()))
            .map_err(Kind::Io.tag())?;
    }

    for (rel, bytes) in &output.files {
        let dest = out_dir.join(rel);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("{}을(를) 만들 수 없습니다", parent.display()))?;
        }
        std::fs::write(&dest, bytes)
            .with_context(|| format!("{}을(를) 쓸 수 없습니다", dest.display()))
            .map_err(Kind::Io.tag())?;
    }

    Ok(BuildStats {
        pages_written: output.pages(),
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

/// 페이지 HTML과 그 페이지의 검색 색인 행을 함께 낸다. 본문 평문은 렌더 과정에서
/// 이미 나오므로, 색인을 위해 문서를 다시 파싱하지 않는다.
fn render_page(
    page: &Page,
    site: &Site,
    cfg: &Config,
    templates: &Templates,
    md: &markdown::Renderer,
    highlight_css: Option<&str>,
) -> Result<(String, search::Entry)> {
    let template = select_template(page, site, templates).map_err(Kind::Template.tag())?;

    let site_ctx = SiteCtx {
        title: cfg.title.clone(),
        description: cfg.description.clone(),
        base_url: cfg.base_url_trimmed().to_string(),
        language: page.language.clone(),
        sections: site::section_ctx(site.sections(&page.language), site.pages),
        highlight_css: highlight_css.map(str::to_string),
    };

    let rendered = md.render_in(&page.language, &page.body);
    if !rendered.unresolved.is_empty() {
        return Err(Kind::Content.tag()(anyhow::anyhow!(
            "{}: 해석할 수 없는 내부 링크가 있습니다:\n  {}\n\
             `@/` 는 content/ 기준 소스 경로를 가리켜야 합니다 (예: `@/start/installation.md`).",
            page.source.display(),
            rendered.unresolved.join("\n  ")
        )));
    }

    // 섹션 인덱스는 자기 자신이 섹션이라 라벨과 제목이 겹친다. 그때는 비워 둔다.
    let section = if page.is_section {
        None
    } else {
        site.section_ref_of(page)
    };

    let (prev, next) = site.neighbours_of(page);

    let entry = search::Entry {
        t: page.title.clone(),
        d: page.front.description.clone(),
        u: page.url.clone(),
        s: section
            .as_ref()
            .map(|s| s.title.clone())
            .unwrap_or_default(),
        c: rendered.text.clone(),
    };

    let page_ctx = PageCtx {
        title: page.title.clone(),
        description: page.front.description.clone(),
        url: page.url.clone(),
        permalink: format!("{}{}", cfg.base_url_trimmed(), page.url),
        content: rendered.html,
        weight: page.front.weight,
        draft: page.front.draft,
        // `toc`는 "목차를 보여줄지"라는 저자의 의사, `toc_entries`는 실제 데이터다.
        toc: page.front.toc,
        toc_entries: rendered.toc,
        language: page.language.clone(),
        // 번역이 실제로 있는 언어만 담긴다 — 템플릿은 이게 비었는지로
        // 언어 전환 UI를 보일지 결정하면 된다.
        translations: site.translations_of(page),
        children: site.children_of(page),
        section,
        prev,
        next,
        is_section: page.is_section,
    };

    let html = templates
        .render(&template, context! { site => site_ctx, page => page_ctx })
        .with_context(|| format!("{} 렌더 중", page.source.display()))
        .map_err(Kind::Template.tag())?;

    Ok((html, entry))
}

/// 템플릿 선택은 **명시적**이다. 순서는 딱 넷:
///
/// 1. front matter `template`
/// 2. 섹션이면 `section.html`
/// 3. 부모 섹션의 `page_template`
/// 4. `page.html`
///
/// Hugo의 20단계 lookup order 캐스케이드는 채택하지 않는다 — kind × section × type ×
/// layout × language × output format을 곱해 만든 순서표는 사용자가 가장 많이 헤매는
/// 지점이고, "해석 순서를 출력이라도 해달라"는 요청이 10년째 열려 있다.
fn select_template(page: &Page, site: &Site, templates: &Templates) -> Result<String> {
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

    if page.is_section && templates.has("section.html") {
        return Ok("section.html".into());
    }

    if !page.is_section
        && let Some(pt) = site.page_template_for(page)
        && templates.has(pt)
    {
        return Ok(pt.to_string());
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
