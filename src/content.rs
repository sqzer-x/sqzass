//! 콘텐츠 발견과 front matter 파싱.
//!
//! front matter는 TOML(`+++`)이 정본이다. `serde_yaml`이 아카이브된 상태라
//! YAML을 정본으로 삼으면 살아있는 파서를 고르는 문제부터 떠안게 된다.

use crate::config::Config;
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub const CONTENT_DIR: &str = "content";
const FENCE: &str = "+++";

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FrontMatter {
    /// 유일한 필수 필드. 없으면 좋은 에러를 내기 위해 Option으로 받는다.
    pub title: Option<String>,
    pub description: String,
    pub weight: i64,
    pub draft: bool,
    /// 기본은 파일명 stem.
    pub slug: Option<String>,
    pub template: Option<String>,
    pub toc: bool,
    /// 기본은 파일명 stem 추론.
    pub translation_key: Option<String>,
    /// 이 페이지로 보낼 예전 URL들. 루트 절대 경로여야 한다.
    pub aliases: Vec<String>,

    // --- 섹션(`_index.md`) 전용 ---
    pub sort_by: Option<crate::config::SortBy>,
    pub page_template: Option<String>,

    #[serde(default)]
    pub extra: toml::Table,
}

#[derive(Debug, Clone)]
pub struct Page {
    /// 소스 파일 절대 경로.
    pub source: PathBuf,
    /// `content/` 기준 상대 경로.
    pub rel: PathBuf,
    pub front: FrontMatter,
    pub title: String,
    /// front matter를 제거한 마크다운 본문.
    pub body: String,
    /// `body`의 첫 줄이 원본 파일에서 몇 번째 줄인지(1-based). 에러 보고에 쓴다.
    pub body_line_offset: usize,
    pub language: String,
    /// 번역 연결 키. 기본은 파일명 stem 추론.
    pub translation_key: String,
    /// `/`, `/about/`, `/ko/start/installation/`
    pub url: String,
    /// 출력 디렉터리 기준 상대 경로. `index.html`, `about/index.html`
    pub out_path: PathBuf,
    /// `_index.md` 여부 (섹션 인덱스).
    pub is_section: bool,
}

/// `content/` 아래의 모든 마크다운을 읽어 `Page`로 만든다.
///
/// 드래프트는 `drafts`가 false면 제외한다.
pub fn discover(root: &Path, cfg: &Config, drafts: bool) -> Result<Vec<Page>> {
    let content_root = root.join(CONTENT_DIR);
    if !content_root.is_dir() {
        bail!("{} 디렉터리가 없습니다", content_root.display());
    }

    let mut pages = Vec::new();
    for entry in WalkDir::new(&content_root).sort_by_file_name() {
        let entry = entry.context("content/ 순회 실패")?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let page = parse_page(path, &content_root, cfg)?;
        if page.front.draft && !drafts {
            continue;
        }
        pages.push(page);
    }

    check_url_collisions(&pages)?;
    Ok(pages)
}

fn parse_page(path: &Path, content_root: &Path, cfg: &Config) -> Result<Page> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("{}을(를) 읽을 수 없습니다", path.display()))?;

    let (fm_text, body, body_line_offset) = split_front_matter(&raw, path)?;

    // 빈 줄 하나를 앞에 붙여 toml이 세는 줄 번호를 원본 파일의 줄 번호에 맞춘다.
    // 여는 `+++`가 항상 1번 줄이므로 front matter 본문은 2번 줄부터 시작한다.
    // 이 한 줄이 없으면 에러가 가리키는 줄이 항상 하나씩 위를 가리킨다.
    let front: FrontMatter = toml::from_str(&format!("\n{fm_text}"))
        .with_context(|| format!("{}: front matter 파싱 실패", path.display()))?;

    let title = front.title.clone().ok_or_else(|| {
        anyhow::anyhow!(
            "{}: front matter에 필수 필드 `title`이 없습니다",
            path.display()
        )
    })?;

    let rel = path
        .strip_prefix(content_root)
        .unwrap_or(path)
        .to_path_buf();

    let name = NameParts::parse(&rel, cfg);
    let slug = front.slug.clone().unwrap_or_else(|| name.stem.clone());
    let translation_key = front
        .translation_key
        .clone()
        .unwrap_or_else(|| name.translation_key.clone());

    let (url, out_path) = build_url(&name, &slug, cfg);

    Ok(Page {
        source: path.to_path_buf(),
        rel,
        front,
        title,
        body,
        body_line_offset,
        language: name.language,
        translation_key,
        url,
        out_path,
        is_section: name.is_section,
    })
}

/// 파일 경로에서 stem / 언어 / 섹션 여부를 뽑는다.
///
/// `start/installation.md`    → stem "installation", lang = default
/// `start/installation.ko.md` → stem "installation", lang = "ko"
/// `start/_index.md`          → 섹션, stem "start"
struct NameParts {
    /// URL의 마지막 세그먼트가 될 이름.
    stem: String,
    /// `content/` 기준 부모 디렉터리 세그먼트들.
    dirs: Vec<String>,
    language: String,
    is_section: bool,
    translation_key: String,
}

impl NameParts {
    fn parse(rel: &Path, cfg: &Config) -> Self {
        let dirs: Vec<String> = rel
            .parent()
            .map(|p| {
                p.components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();

        // `foo.ko.md` → file_stem은 "foo.ko" 이므로 한 번 더 벗긴다.
        let full_stem = rel
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();

        let (base, language) = match full_stem.rsplit_once('.') {
            // 마지막 조각이 설정에 선언된 언어 코드일 때만 언어로 인정한다.
            // 그렇지 않으면 `my.file.md` 같은 이름이 언어로 오인된다.
            Some((base, maybe_lang)) if cfg.languages.contains_key(maybe_lang) => {
                (base.to_string(), maybe_lang.to_string())
            }
            _ => (full_stem.clone(), cfg.default_language.clone()),
        };

        let is_section = base == "_index";
        // 섹션의 이름은 자기 디렉터리 이름이다. 루트 섹션이면 빈 문자열.
        let stem = if is_section {
            dirs.last().cloned().unwrap_or_default()
        } else {
            base.clone()
        };

        // 번역 키는 언어를 뗀 경로. `start/installation`, `start/_index`
        let mut key_parts = dirs.clone();
        key_parts.push(base);
        let translation_key = key_parts.join("/");

        Self {
            stem,
            dirs,
            language,
            is_section,
            translation_key,
        }
    }
}

/// URL과 출력 경로를 만든다. 항상 `path/index.html` 형태로 뽑는다 —
/// GitHub Pages처럼 rewrite 규칙이 없는 호스트에서 예쁜 URL을 내는 유일한 방법이다.
fn build_url(name: &NameParts, slug: &str, cfg: &Config) -> (String, PathBuf) {
    let mut segments: Vec<String> = Vec::new();

    // 기본 언어는 루트, 나머지는 `/<code>/` 아래.
    if name.language != cfg.default_language {
        segments.push(name.language.clone());
    }

    if name.is_section {
        // 섹션의 URL은 자기 디렉터리 경로 그 자체다.
        segments.extend(name.dirs.iter().cloned());
    } else {
        segments.extend(name.dirs.iter().cloned());
        segments.push(slug.to_string());
    }

    // URL에는 서브경로가 들어가고 출력 경로에는 들어가지 않는다. 호스트가 서빙하는
    // 루트가 곧 출력 디렉터리이고, 서브경로는 그 루트가 도메인 어디에 놓이는지의
    // 문제이기 때문이다.
    let base = cfg.base_path();
    let url = if segments.is_empty() {
        format!("{base}/")
    } else {
        format!("{base}/{}/", segments.join("/"))
    };

    let mut out = PathBuf::new();
    for s in &segments {
        out.push(s);
    }
    out.push("index.html");

    (url, out)
}

/// 두 페이지가 같은 URL을 주장하면 하드 에러. 조용히 덮어쓰면 어느 쪽이 남는지가
/// 파일 순회 순서에 의존하게 되고, 그건 재현 불가능한 버그가 된다.
fn check_url_collisions(pages: &[Page]) -> Result<()> {
    use std::collections::HashMap;
    let mut seen: HashMap<&str, &Page> = HashMap::new();
    for page in pages {
        if let Some(prev) = seen.insert(&page.url, page) {
            bail!(
                "URL 충돌: '{}' 을(를) 두 파일이 주장합니다\n  {}\n  {}\n\
                 front matter의 `slug`로 한쪽을 바꾸세요.",
                page.url,
                prev.source.display(),
                page.source.display()
            );
        }
    }
    Ok(())
}

/// `+++` 펜스로 front matter를 분리한다.
///
/// 직접 자르는 이유: 파싱 에러를 **원본 파일의 실제 줄 번호**로 보고하려면 front matter가
/// 몇 줄을 먹었는지 알아야 하는데, 대부분의 크레이트가 그 정보를 잃는다.
fn split_front_matter(raw: &str, path: &Path) -> Result<(String, String, usize)> {
    let text = raw.strip_prefix('\u{feff}').unwrap_or(raw);
    let mut lines = text.lines();

    let Some(first) = lines.next() else {
        bail!("{}: 파일이 비어 있습니다", path.display());
    };

    if first.trim_end() != FENCE {
        if first.trim_end() == "---" {
            bail!(
                "{}: front matter가 `---`로 시작합니다. sqzass는 TOML front matter를 쓰므로 \
                 `+++`로 감싸주세요.",
                path.display()
            );
        }
        bail!(
            "{}: 파일이 `{}` front matter로 시작해야 합니다",
            path.display(),
            FENCE
        );
    }

    let mut fm = String::new();
    let mut closed_at = None;
    // 1번 줄은 여는 펜스이므로 본문 후보는 2번 줄부터.
    for (idx, line) in lines.enumerate() {
        if line.trim_end() == FENCE {
            closed_at = Some(idx + 2); // 닫는 펜스의 1-based 줄 번호
            break;
        }
        fm.push_str(line);
        fm.push('\n');
    }

    let Some(closed_at) = closed_at else {
        bail!(
            "{}: front matter를 닫는 `{}`를 찾지 못했습니다",
            path.display(),
            FENCE
        );
    };

    // 닫는 펜스 다음 줄부터가 본문.
    let body: String = text.lines().skip(closed_at).collect::<Vec<_>>().join("\n");

    Ok((fm, body, closed_at + 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> Config {
        toml::from_str(
            r#"
            title = "t"
            base_url = "https://example.com"
            default_language = "en"
            [languages.en]
            name = "English"
            [languages.ko]
            name = "한국어"
            "#,
        )
        .unwrap()
    }

    /// front matter 오타도 조용히 무시하지 않는다. 그리고 에러가 가리키는 줄은
    /// **원본 파일의** 줄이어야 한다 — 펜스를 손으로 자른 이유가 그것이다.
    /// URL에는 서브경로가 들어가고 출력 경로에는 들어가지 않는다. 호스트가 서빙하는
    /// 루트가 곧 출력 디렉터리이기 때문이다.
    #[test]
    fn base_path_reaches_urls_but_not_output_paths() {
        let cfg: Config =
            toml::from_str("title = \"t\"\nbase_url = \"https://user.github.io/repo\"\n").unwrap();

        let parts = NameParts::parse(Path::new("start/install.md"), &cfg);
        let (url, out) = build_url(&parts, &parts.stem, &cfg);
        assert_eq!(url, "/repo/start/install/");
        assert_eq!(out, Path::new("start/install/index.html"));

        let root = NameParts::parse(Path::new("_index.md"), &cfg);
        let (url, out) = build_url(&root, &root.stem, &cfg);
        assert_eq!(url, "/repo/");
        assert_eq!(out, Path::new("index.html"));
    }

    #[test]
    fn unknown_front_matter_key_errors_at_the_real_line() {
        let dir = std::env::temp_dir().join(format!("sqzass-fm-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("content")).unwrap();
        let path = dir.join("content/_index.md");
        std::fs::write(&path, "+++\ntitle = \"x\"\nweigth = 10\n+++\n\nbody\n").unwrap();

        let cfg: Config =
            toml::from_str("title = \"t\"\nbase_url = \"https://example.com\"\n").unwrap();
        // anyhow는 `{}`에 최상위 컨텍스트만 낸다. 원인 사슬은 `{:#}`이라야 보인다.
        let err = format!(
            "{:#}",
            parse_page(&path, &dir.join("content"), &cfg).unwrap_err()
        );
        let _ = std::fs::remove_dir_all(&dir);

        assert!(err.contains("weigth"), "실제: {err}");
        assert!(err.contains("weight"), "후보를 제시해야 한다: {err}");
        // `weigth`는 파일의 3번 줄이다. 펜스를 세지 않으면 2번 줄이라고 말한다.
        assert!(err.contains("line 3"), "원본 파일 줄 번호가 아니다: {err}");
    }

    #[test]
    fn splits_front_matter_and_tracks_line_offset() {
        let raw = "+++\ntitle = \"Hi\"\n+++\n\n# Body\n";
        let (fm, body, offset) = split_front_matter(raw, Path::new("x.md")).unwrap();
        assert_eq!(fm.trim(), "title = \"Hi\"");
        assert_eq!(body, "\n# Body");
        // `+++`(1) `title`(2) `+++`(3) → 본문은 4번째 줄부터
        assert_eq!(offset, 4);
    }

    #[test]
    fn rejects_yaml_fence_with_a_useful_message() {
        let raw = "---\ntitle: Hi\n---\n";
        let err = split_front_matter(raw, Path::new("x.md"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("+++"), "실제 메시지: {err}");
    }

    #[test]
    fn rejects_unterminated_front_matter() {
        let raw = "+++\ntitle = \"Hi\"\n\n# Body\n";
        assert!(split_front_matter(raw, Path::new("x.md")).is_err());
    }

    #[test]
    fn url_for_root_index() {
        let cfg = cfg();
        let parts = NameParts::parse(Path::new("_index.md"), &cfg);
        let (url, out) = build_url(&parts, &parts.stem, &cfg);
        assert_eq!(url, "/");
        assert_eq!(out, PathBuf::from("index.html"));
    }

    #[test]
    fn url_for_nested_page() {
        let cfg = cfg();
        let parts = NameParts::parse(Path::new("start/installation.md"), &cfg);
        let (url, out) = build_url(&parts, &parts.stem, &cfg);
        assert_eq!(url, "/start/installation/");
        assert_eq!(out, PathBuf::from("start/installation/index.html"));
    }

    #[test]
    fn url_for_section_index() {
        let cfg = cfg();
        let parts = NameParts::parse(Path::new("start/_index.md"), &cfg);
        assert!(parts.is_section);
        let (url, out) = build_url(&parts, &parts.stem, &cfg);
        assert_eq!(url, "/start/");
        assert_eq!(out, PathBuf::from("start/index.html"));
    }

    #[test]
    fn korean_pages_go_under_language_prefix() {
        let cfg = cfg();
        let parts = NameParts::parse(Path::new("start/installation.ko.md"), &cfg);
        assert_eq!(parts.language, "ko");
        assert_eq!(parts.stem, "installation");
        // 번역 키는 언어를 뗀 경로여서 영어판과 짝이 맞는다
        assert_eq!(parts.translation_key, "start/installation");
        let (url, out) = build_url(&parts, &parts.stem, &cfg);
        assert_eq!(url, "/ko/start/installation/");
        assert_eq!(out, PathBuf::from("ko/start/installation/index.html"));
    }

    #[test]
    fn translation_keys_match_across_languages() {
        let cfg = cfg();
        let en = NameParts::parse(Path::new("start/installation.md"), &cfg);
        let ko = NameParts::parse(Path::new("start/installation.ko.md"), &cfg);
        assert_eq!(en.translation_key, ko.translation_key);
    }

    #[test]
    fn unknown_dotted_suffix_is_not_treated_as_a_language() {
        let cfg = cfg();
        // "backup"은 선언된 언어가 아니므로 파일명 일부로 남아야 한다
        let parts = NameParts::parse(Path::new("notes.backup.md"), &cfg);
        assert_eq!(parts.language, "en");
        assert_eq!(parts.stem, "notes.backup");
    }
}
