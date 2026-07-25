//! `sqzass doctor` — 빌드가 통과시키지만 사람이 알아야 하는 것들.
//!
//! **빌드가 이미 잡는 건 여기서 다시 잡지 않는다.** 설정 오타, front matter 오타,
//! URL 충돌, 해석 안 되는 `@/` 링크, 없는 템플릿·에셋·번역 키는 전부 빌드가 멈추는
//! 사유다. 그걸 doctor가 또 보고하면 두 명령이 같은 말을 하게 되고, 그중 하나는
//! 언젠가 낡는다.
//!
//! 남는 건 **틀리지는 않았지만 의도한 게 아닐 가능성이 있는 것들**이다. 그래서
//! 경고이고, 그래서 게이팅 기준을 고를 수 있어야 한다.

use crate::config::Config;
use crate::content::Page;
use crate::render::Templates;
use crate::site::Site;
use anyhow::Result;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// 찾은 게 있을 때의 exit code. `Kind`의 번호들과 겹치지 않는다 — doctor의 지적은
/// 빌드 실패와 다른 종류의 사건이고, CI가 둘을 구분할 수 있어야 한다.
pub const FINDINGS_EXIT_CODE: i32 = 7;

/// `init`이 넣어 주는 자리표시자. 이대로 배포하면 canonical URL과 sitemap이
/// 남의 도메인을 가리킨다.
const PLACEHOLDER_BASE_URL: &str = "https://example.com";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// 알아 두면 좋은 것. 기본 게이트는 여기서 걸리지 않는다.
    Note,
    /// 의도한 게 아닐 가능성이 높은 것.
    Warn,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Note => "note",
            Self::Warn => "warn",
        })
    }
}

impl std::str::FromStr for Severity {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, String> {
        match s {
            "note" => Ok(Self::Note),
            "warn" => Ok(Self::Warn),
            other => Err(format!("알 수 없는 심각도 '{other}' (note | warn)")),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub severity: Severity,
    /// 검사 이름. 안정된 문자열이라 스크립트가 잡아도 된다.
    pub check: &'static str,
    pub message: String,
    /// 관련된 소스 파일이 있으면.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
}

impl std::fmt::Display for Finding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: [{}] {}", self.severity, self.check, self.message)?;
        if let Some(file) = &self.file {
            write!(f, " ({file})")?;
        }
        Ok(())
    }
}

/// 사이트를 읽고 지적할 것들을 모은다. 아무것도 쓰지 않는다.
pub fn run(root: &Path, cfg: &Config, pages: &[Page], site: &Site) -> Result<Vec<Finding>> {
    let mut out = Vec::new();

    placeholder_base_url(cfg, &mut out);
    untranslated_pages(cfg, pages, site, &mut out);
    missing_descriptions(pages, &mut out);
    drafts(root, cfg, &mut out)?;
    empty_sections(cfg, site, &mut out);
    unused_templates(root, pages, site, &mut out)?;

    // 심각도가 높은 것부터, 같으면 검사 이름순. 출력 순서가 빌드처럼 결정적이어야
    // CI 로그를 diff 할 수 있다.
    out.sort_by(|a, b| {
        b.severity
            .cmp(&a.severity)
            .then_with(|| a.check.cmp(b.check))
            .then_with(|| a.message.cmp(&b.message))
    });
    Ok(out)
}

fn placeholder_base_url(cfg: &Config, out: &mut Vec<Finding>) {
    if cfg.base_url_trimmed() == PLACEHOLDER_BASE_URL {
        out.push(Finding {
            severity: Severity::Warn,
            check: "base-url",
            message: format!(
                "base_url이 아직 `{PLACEHOLDER_BASE_URL}` 입니다. \
                 canonical URL과 sitemap이 이 주소로 나갑니다."
            ),
            file: Some("sqzass.toml".into()),
        });
    }
}

/// 어떤 언어에는 있고 어떤 언어에는 없는 페이지. 빌드는 이걸 통과시킨다 —
/// 미번역 페이지를 감추는 건 의도된 동작이다. 다만 감췄다는 사실은 알아야 한다.
fn untranslated_pages(cfg: &Config, pages: &[Page], site: &Site, out: &mut Vec<Finding>) {
    if cfg.languages.len() < 2 {
        return;
    }
    let declared: BTreeSet<&str> = cfg.languages.keys().map(String::as_str).collect();

    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for page in pages {
        if !seen.insert(&page.translation_key) {
            continue;
        }
        let have: BTreeSet<String> = site
            .language_set_of(page)
            .into_iter()
            .map(|(code, _)| code)
            .collect();
        let missing: Vec<&str> = declared
            .iter()
            .filter(|code| !have.contains(**code))
            .copied()
            .collect();
        if !missing.is_empty() {
            out.push(Finding {
                severity: Severity::Warn,
                check: "untranslated",
                message: format!("{} 번역이 없습니다", missing.join(", ")),
                file: Some(page.rel.to_string_lossy().into_owned()),
            });
        }
    }
}

fn missing_descriptions(pages: &[Page], out: &mut Vec<Finding>) {
    for page in pages {
        if page.front.description.trim().is_empty() {
            out.push(Finding {
                severity: Severity::Note,
                check: "description",
                message: "description이 없습니다. 검색 결과와 <meta>에 쓰입니다.".into(),
                file: Some(page.rel.to_string_lossy().into_owned()),
            });
        }
    }
}

/// 드래프트는 기본 빌드에서 빠지므로, 사이트에 없는 이유가 여기 있다.
fn drafts(root: &Path, cfg: &Config, out: &mut Vec<Finding>) -> Result<()> {
    let all = crate::content::discover(root, cfg, true)?;
    for page in all.iter().filter(|p| p.front.draft) {
        out.push(Finding {
            severity: Severity::Note,
            check: "draft",
            message: "draft = true 이므로 빌드에 포함되지 않습니다".into(),
            file: Some(page.rel.to_string_lossy().into_owned()),
        });
    }
    Ok(())
}

/// 자식이 하나도 없는 섹션. 내비게이션에 들어가지만 눌러도 목록이 비어 있다.
fn empty_sections(cfg: &Config, site: &Site, out: &mut Vec<Finding>) {
    for lang in cfg.languages.keys() {
        let mut stack: Vec<&crate::site::Section> = site.sections(lang).iter().collect();
        while let Some(section) = stack.pop() {
            if section.pages.is_empty() && section.subsections.is_empty() {
                out.push(Finding {
                    severity: Severity::Warn,
                    check: "empty-section",
                    message: format!("'{}' 섹션에 페이지가 없습니다", section.title),
                    file: Some(section.url.clone()),
                });
            }
            stack.extend(section.subsections.iter());
        }
    }
}

/// 어떤 페이지도 고르지 않은 템플릿. 이름을 바꾸고 참조를 안 고쳤을 때 남는다.
fn unused_templates(
    root: &Path,
    pages: &[Page],
    site: &Site,
    out: &mut Vec<Finding>,
) -> Result<()> {
    let templates = Templates::load(root)?;
    let mut used: BTreeSet<String> = BTreeSet::new();
    for page in pages {
        if let Ok(name) = crate::select_template(page, site, &templates) {
            used.insert(name);
        }
    }

    // `include`/`extends`로 끌려 들어가는 것들은 페이지가 직접 고르지 않는다.
    // 그것까지 미사용이라고 하면 이 검사는 매번 틀린 말을 하게 된다.
    let referenced = referenced_templates(root, &templates)?;

    for name in templates.names() {
        if !used.contains(name) && !referenced.contains(name) {
            out.push(Finding {
                severity: Severity::Note,
                check: "unused-template",
                message: "어떤 페이지도 이 템플릿을 고르지 않았습니다".into(),
                file: Some(format!("templates/{name}")),
            });
        }
    }
    Ok(())
}

/// 다른 템플릿이 이름으로 언급하는 템플릿들.
///
/// 문자열 포함 검사다. 정확한 파서가 아니지만, 여기서 틀리는 방향은 **미사용이라고
/// 말하지 않는 쪽**이다 — 지우라고 잘못 권하는 것보다 조용한 게 낫다.
fn referenced_templates(root: &Path, templates: &Templates) -> Result<BTreeSet<String>> {
    let dir = root.join(crate::render::TEMPLATE_DIR);
    let mut sources = BTreeMap::new();
    for name in templates.names() {
        if let Ok(text) = std::fs::read_to_string(dir.join(name)) {
            sources.insert(name.clone(), text);
        }
    }

    let mut out = BTreeSet::new();
    for name in templates.names() {
        let quoted = format!("\"{name}\"");
        if sources
            .iter()
            .any(|(owner, text)| owner != name && text.contains(&quoted))
        {
            out.insert(name.clone());
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_orders_warn_above_note() {
        assert!(Severity::Warn > Severity::Note);
    }

    #[test]
    fn severity_parses_and_rejects_nonsense() {
        assert_eq!("warn".parse::<Severity>().unwrap(), Severity::Warn);
        assert!("loud".parse::<Severity>().is_err());
    }

    #[test]
    fn placeholder_base_url_is_reported() {
        let cfg: Config =
            toml::from_str("title = \"t\"\nbase_url = \"https://example.com/\"\n").unwrap();
        let mut out = Vec::new();
        placeholder_base_url(&cfg, &mut out);
        assert_eq!(out.len(), 1, "{out:?}");
        assert_eq!(out[0].check, "base-url");
    }

    #[test]
    fn a_real_base_url_is_not_reported() {
        let cfg: Config =
            toml::from_str("title = \"t\"\nbase_url = \"https://sqzass.sqzer.com\"\n").unwrap();
        let mut out = Vec::new();
        placeholder_base_url(&cfg, &mut out);
        assert!(out.is_empty(), "{out:?}");
    }

    #[test]
    fn findings_render_with_check_and_file() {
        let f = Finding {
            severity: Severity::Warn,
            check: "untranslated",
            message: "ko 번역이 없습니다".into(),
            file: Some("start/x.md".into()),
        };
        assert_eq!(
            f.to_string(),
            "warn: [untranslated] ko 번역이 없습니다 (start/x.md)"
        );
    }

    /// doctor의 exit code는 빌드 실패의 코드들과 겹치면 안 된다. 겹치면 CI가
    /// "지적이 있었다"와 "빌드가 깨졌다"를 구분할 수 없다.
    #[test]
    fn findings_exit_code_is_distinct_from_error_kinds() {
        use crate::error::Kind;
        for k in [
            Kind::Config,
            Kind::Content,
            Kind::Template,
            Kind::Io,
            Kind::Other,
        ] {
            assert_ne!(k.code(), FINDINGS_EXIT_CODE);
        }
    }
}
