//! sqzass — Rust로 만든 정적 사이트 생성기.

pub mod assets;
pub mod config;
pub mod content;
pub mod doctor;
pub mod emit;
pub mod error;
pub mod feed;
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
    /// 실제 콘텐츠 페이지 수.
    ///
    /// `.html`을 세지 않는다. 리다이렉트 스텁과 404도 `.html`이지만 사람이 쓴
    /// 페이지가 아니고, 그걸 세면 "5 pages"라고 보고하면서 콘텐츠는 두 개인
    /// 상태가 된다.
    pub pages: usize,
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
    let mut assets =
        assets::Assets::collect(root, &cfg.assets, cfg.base_path()).map_err(Kind::Io.tag())?;

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

    // 피드를 먼저 만든다. 페이지가 <head>에 자동 발견 링크를 넣으려면 자기 언어에
    // 피드가 있는지를 렌더 시점에 알아야 한다.
    let mut feeds: BTreeMap<String, String> = BTreeMap::new();
    {
        let mut by_lang: BTreeMap<&str, Vec<&Page>> = BTreeMap::new();
        for p in &pages {
            by_lang.entry(p.language.as_str()).or_default().push(p);
        }
        for (lang, langs_pages) in by_lang {
            let home = if lang == cfg.default_language {
                format!("{}/", cfg.base_path())
            } else {
                format!("{}/{lang}/", cfg.base_path())
            };
            let self_url = format!("{}/{}", cfg.base_path(), feed::path(lang));
            if let Some(xml) = feed::atom(
                &langs_pages,
                lang,
                &cfg.title,
                cfg.origin(),
                &home,
                &self_url,
            ) {
                feeds.insert(lang.to_string(), xml);
            }
        }
    }

    // 사이트가 실제로 내보내는 URL 전체. 이게 있어야 `[설치](/start/install/)` 같은
    // 평범한 절대 경로 링크도 빌드가 검증할 수 있다 — 지금까지는 `@/`만 봤다.
    let base = cfg.base_path();
    let generated = [
        seo::SITEMAP_PATH,
        seo::ROBOTS_PATH,
        emit::LLMS_PATH,
        emit::NOT_FOUND_PATH,
        assets::MANIFEST_PATH,
    ];
    let known = links::KnownUrls {
        urls: pages
            .iter()
            .map(|p| p.url.clone())
            .chain(assets.manifest.values().cloned())
            // 빌드가 만드는 파일도 존재하는 URL이다. 빼 두면 우리 사이트의
            // 피드나 sitemap을 가리키는 링크가 "어디도 가리키지 않는다"며
            // 빌드를 깨뜨린다 — 실제로 그랬다.
            .chain(generated.iter().map(|p| format!("{base}/{p}")))
            .chain(
                feeds
                    .keys()
                    .map(|lang| format!("{base}/{}", feed::path(lang))),
            )
            // 설정의 [languages]가 아니라 **실제 페이지의 언어**로 만든다.
            // 언어를 선언하지 않은 사이트도 기본 언어로 색인을 내보내므로,
            // 설정을 기준으로 삼으면 그 사이트에서 색인 링크가 죽는다.
            .chain(
                pages
                    .iter()
                    .map(|p| p.language.as_str())
                    .collect::<std::collections::BTreeSet<_>>()
                    .into_iter()
                    .map(|lang| format!("{base}/{}", search::path(lang))),
            )
            .collect(),
    };
    md = md.with_known_urls(known);

    assets.write_manifest();
    let templates = Templates::load(root)
        .map_err(Kind::Template.tag())?
        .with_assets(assets.manifest.clone())
        .with_i18n(i18n::load(root).map_err(Kind::Template.tag())?);

    // 사이드바 컨텍스트는 언어당 한 번만 만들어 미리 minijinja 값으로 직렬화한다.
    // 페이지마다 전 섹션·전 페이지를 재조립해 다시 직렬화하면 페이지당 O(n),
    // 빌드 전체 O(n²)다 — 5000페이지 실측에서 링크 표 복사와 함께 지배 항이었다.
    // `Value` 복사는 참조 카운트라 페이지마다 나눠 주는 비용은 포인터 복사다.
    let sections_by_lang: BTreeMap<&str, minijinja::Value> = pages
        .iter()
        .map(|p| p.language.as_str())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .map(|lang| {
            let ctx = site::section_ctx(site.sections(lang), site.pages);
            (lang, minijinja::Value::from_serialize(&ctx))
        })
        .collect();

    let mut out = BuildOutput {
        pages: pages.len(),
        ..BuildOutput::default()
    };
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
            sections_by_lang
                .get(page.language.as_str())
                .cloned()
                .expect("언어별 섹션 컨텍스트는 모든 페이지 언어에 대해 만들었다"),
            highlight_css_url.as_deref(),
            feeds
                .contains_key(&page.language)
                .then(|| format!("{}/{}", cfg.base_path(), feed::path(&page.language)))
                .as_deref(),
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

    // 옛 URL을 새 URL로 보내는 스텁. GitHub Pages에는 리다이렉트 규칙이 없으므로
    // 이걸 빌드가 만들지 않으면 아무도 만들어 주지 않는다.
    for page in &pages {
        for alias in &page.front.aliases {
            let out_path = emit::alias_output_path(alias, page).map_err(Kind::Content.tag())?;
            let stub = emit::redirect_stub(&page.url, &format!("{}{}", cfg.origin(), page.url));
            if let Some(prev) = out.files.insert(out_path.clone(), stub.into_bytes()) {
                let _ = prev;
                return Err(Kind::Content.tag()(anyhow::anyhow!(
                    "{}: alias `{alias}` 가 이미 존재하는 출력 `{out_path}` 를 덮어씁니다.\n\
                     다른 페이지나 다른 alias가 같은 URL을 주장하고 있습니다.",
                    page.source.display()
                )));
            }
        }
    }

    // 호스트들이 이름으로 찾는 404. 템플릿이 있을 때만 만든다 — 하이라이트
    // 스타일시트와 같은 계약이다.
    if templates.has(emit::NOT_FOUND_PATH) {
        let html = render_standalone(
            emit::NOT_FOUND_PATH,
            &cfg,
            &templates,
            // 기본 언어에 페이지가 하나도 없으면 섹션도 없다 — 빈 목록이 옳다.
            sections_by_lang
                .get(cfg.default_language.as_str())
                .cloned()
                .unwrap_or_else(
                    || minijinja::Value::from_serialize(Vec::<site::SectionCtx>::new()),
                ),
            highlight_css_url.as_deref(),
        )
        .map_err(Kind::Template.tag())?;
        out.files
            .insert(emit::NOT_FOUND_PATH.into(), html.into_bytes());
    }

    for (lang, xml) in feeds {
        // `static/`에 같은 이름이 있으면 그쪽이 이긴다. sitemap·robots과 같은
        // 규칙이다 — 직접 넣은 파일이 조용히 덮이는 건 어느 쪽이든 나쁘다.
        out.files
            .entry(feed::path(&lang))
            .or_insert_with(|| xml.into_bytes());
    }

    out.files.entry(emit::LLMS_PATH.into()).or_insert_with(|| {
        emit::llms_txt(&cfg.title, &cfg.description, cfg.origin(), &pages).into_bytes()
    });

    // `static/`에 같은 이름이 있으면 그쪽이 이긴다. 직접 넣은 robots.txt가
    // 조용히 무시당하는 것보다는 생성을 건너뛰는 쪽이 낫다.
    let origin = cfg.origin();
    out.files
        .entry(seo::SITEMAP_PATH.into())
        .or_insert_with(|| seo::sitemap(&pages, &site, origin).into_bytes());
    out.files
        .entry(seo::ROBOTS_PATH.into())
        .or_insert_with(|| seo::robots(origin, cfg.base_path()).into_bytes());

    // GitHub Pages는 이 파일이 없으면 출력을 Jekyll로 한 번 더 굴려서
    // `_`로 시작하는 디렉터리를 통째로 삼킨다.
    out.files.insert(".nojekyll".into(), Vec::new());

    check_output_collisions(&out).map_err(Kind::Content.tag())?;

    Ok(out)
}

/// 사이트를 읽고 지적할 것들을 모은다. 아무것도 쓰지 않는다.
pub fn diagnose(opts: &BuildOptions) -> Result<Vec<doctor::Finding>> {
    let cfg = load_config(opts)?;
    let pages = content::discover(&opts.input, &cfg, opts.drafts || cfg.build.drafts)
        .map_err(Kind::Content.tag())?;
    let site = Site::build(&pages, &cfg);
    doctor::run(&opts.input, &cfg, &pages, &site).map_err(Kind::Other.tag())
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
        pages_written: output.pages,
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

/// 한 경로가 파일이면서 동시에 디렉터리일 수는 없다.
///
/// `feed-en.xml`이라는 이름의 페이지를 만들면 출력에 `feed-en.xml`(파일)과
/// `feed-en.xml/index.html`(디렉터리)이 함께 생긴다. 쓰는 단계에서야
/// `File exists (os error 17)`로 죽는데, 그때는 이미 절반이 디스크에 나가 있고
/// 메시지는 원인을 하나도 말해 주지 않는다. URL 충돌과 같은 부류이므로 같은
/// 자리에서, 같은 방식으로 막는다.
fn check_output_collisions(out: &BuildOutput) -> Result<()> {
    for key in out.files.keys() {
        let prefix = format!("{key}/");
        // BTreeMap이 정렬돼 있으므로 접두사를 공유하는 이웃만 보면 된다.
        if let Some(other) = out.files.range(prefix.clone()..).next()
            && other.0.starts_with(&prefix)
        {
            anyhow::bail!(
                "출력 경로 충돌: `{key}` 가 파일이면서 동시에 `{}` 의 디렉터리입니다.\n\
                 빌드가 만드는 파일과 같은 이름의 페이지가 있는지 확인하세요 \
                 (sitemap.xml, robots.txt, llms.txt, 404.html, feed-<언어>.xml, search-<언어>.json).",
                other.0
            );
        }
    }
    Ok(())
}

/// 페이지 HTML과 그 페이지의 검색 색인 행을 함께 낸다. 본문 평문은 렌더 과정에서
/// 이미 나오므로, 색인을 위해 문서를 다시 파싱하지 않는다.
// 렌더 루프 한 곳에서만 부르는 내부 함수라 인자를 묶는 구조체가 이름값을 못 한다.
#[allow(clippy::too_many_arguments)]
fn render_page(
    page: &Page,
    site: &Site,
    cfg: &Config,
    templates: &Templates,
    md: &markdown::Renderer,
    sections: minijinja::Value,
    highlight_css: Option<&str>,
    feed_url: Option<&str>,
) -> Result<(String, search::Entry)> {
    let template = select_template(page, site, templates).map_err(Kind::Template.tag())?;

    let site_ctx = SiteCtx {
        title: cfg.title.clone(),
        description: cfg.description.clone(),
        origin: cfg.origin().to_string(),
        base_path: cfg.base_path().to_string(),
        language: page.language.clone(),
        sections,
        feed: feed_url.map(str::to_string),
        highlight_css: highlight_css.map(str::to_string),
    };

    let rendered = md.render_in(&page.language, &page.body);
    if !rendered.unresolved.is_empty() {
        return Err(Kind::Content.tag()(anyhow::anyhow!(
            "{}: 어디도 가리키지 않는 링크가 있습니다:\n  {}\n\n\
             `@/`로 시작하는 링크는 content/ 기준 소스 경로여야 합니다 \
             (예: `@/start/installation.md`).\n\
             `/`로 시작하는 링크는 이 사이트가 실제로 내보내는 URL이어야 합니다.\n\
             CSS와 JS는 파일명에 해시가 붙으므로 마크다운에서 `/css/main.css`로 \
             가리킬 수 없습니다 — 템플릿에서 `asset(\"css/main.css\")`를 쓰세요.",
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
        // page.url이 서브경로를 품고 있으므로 origin만 붙인다.
        permalink: format!("{}{}", cfg.origin(), page.url),
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
        extra: page.front.extra.clone(),
        date: page.front.date.as_ref().and_then(feed::PageDate::from_toml),
    };

    let html = templates
        .render(&template, context! { site => site_ctx, page => page_ctx })
        .with_context(|| format!("{} 렌더 중", page.source.display()))
        .map_err(Kind::Template.tag())?;

    Ok((html, entry))
}

/// 페이지에 매이지 않은 템플릿 하나를 렌더한다. 지금은 404뿐이다.
///
/// 일부러 이름이 붙은 단일 분기로 둔다. "아무 템플릿이나 아무 경로로 렌더한다"는
/// 일반 기구를 만들면 그 순간 출력 URL 소유권을 누가 갖는지가 흐려진다.
fn render_standalone(
    name: &str,
    cfg: &Config,
    templates: &Templates,
    sections: minijinja::Value,
    highlight_css: Option<&str>,
) -> Result<String> {
    let language = cfg.default_language.clone();
    let url = format!("{}/{}", cfg.base_path(), name);

    let site_ctx = SiteCtx {
        title: cfg.title.clone(),
        description: cfg.description.clone(),
        origin: cfg.origin().to_string(),
        base_path: cfg.base_path().to_string(),
        language: language.clone(),
        sections,
        feed: None,
        highlight_css: highlight_css.map(str::to_string),
    };

    let page_ctx = PageCtx {
        title: "404".into(),
        description: String::new(),
        permalink: format!("{}{url}", cfg.origin()),
        url,
        content: String::new(),
        weight: 0,
        draft: false,
        toc: false,
        toc_entries: Vec::new(),
        language,
        translations: Vec::new(),
        children: Vec::new(),
        section: None,
        prev: None,
        next: None,
        is_section: false,
        extra: toml::Table::new(),
        date: None,
    };

    templates
        .render(name, context! { site => site_ctx, page => page_ctx })
        .with_context(|| format!("{name} 렌더 중"))
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
pub(crate) fn select_template(page: &Page, site: &Site, templates: &Templates) -> Result<String> {
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
