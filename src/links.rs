//! 내부 링크 해석.
//!
//! 마크다운에서 `@/start/installation.md` 처럼 **소스 파일**을 가리키면, 렌더 시점에
//! 그 페이지의 실제 URL로 바뀐다. 파일을 옮기거나 슬러그를 바꿔도 링크가 따라온다.
//!
//! 해석은 comrak의 `link_url_rewriter`를 통해 **AST 단계에서** 일어난다. 최종 HTML에
//! 정규식을 돌리지 않는다 — 그 방식은 속성을 홑따옴표로 쓰거나 따옴표를 빼면 조용히
//! 처리에서 빠져서, 링크가 깨졌는지조차 알 수 없게 된다.

use comrak::options::URLRewriter;
use std::collections::BTreeMap;
use std::sync::Mutex;

pub const PREFIX: &str = "@/";

/// 언어별 URL 표. `translation_key` → (언어 → URL)
pub type LinkIndex = BTreeMap<String, BTreeMap<String, String>>;

pub struct Resolver {
    index: LinkIndex,
    language: String,
    default_language: String,
    /// 해석하지 못한 링크. 렌더가 끝난 뒤 빌드를 실패시키는 데 쓴다.
    unresolved: Mutex<Vec<String>>,
}

impl Resolver {
    pub fn new(index: LinkIndex, language: &str, default_language: &str) -> Self {
        Self {
            index,
            language: language.to_string(),
            default_language: default_language.to_string(),
            unresolved: Mutex::new(Vec::new()),
        }
    }

    /// 해석에 실패한 링크들. 비어 있지 않으면 빌드를 멈춰야 한다.
    pub fn take_unresolved(&self) -> Vec<String> {
        self.unresolved
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .split_off(0)
    }

    fn resolve(&self, url: &str) -> Option<String> {
        let (path, fragment) = match url.split_once('#') {
            Some((p, f)) => (p, Some(f)),
            None => (url, None),
        };
        let key = translation_key(path.trim_start_matches(PREFIX));
        let by_lang = self.index.get(&key)?;

        // 지금 페이지와 같은 언어판을 먼저 찾는다. 한국어 문서가
        // `@/start/installation.md` 라고 써도 한국어판으로 간다.
        let target = by_lang
            .get(&self.language)
            .or_else(|| by_lang.get(&self.default_language))?;

        Some(match fragment {
            Some(f) => format!("{target}#{f}"),
            None => target.clone(),
        })
    }
}

impl URLRewriter for Resolver {
    fn to_html(&self, url: &str) -> String {
        if !url.starts_with(PREFIX) {
            return url.to_string();
        }
        match self.resolve(url) {
            Some(resolved) => resolved,
            None => {
                self.unresolved
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(url.to_string());
                // 원문을 그대로 둔다. 어차피 빌드가 실패할 것이고, 실패 메시지에
                // 원래 쓴 문자열이 보이는 편이 찾기 쉽다.
                url.to_string()
            }
        }
    }
}

/// `start/installation.md`, `start/installation.ko.md` → `start/installation`
///
/// 언어 접미사를 떼는 이유: 같은 마크다운을 양쪽 언어판에서 그대로 쓰기 위해서다.
/// 저자는 언어를 신경 쓰지 않고 소스 경로만 적으면 된다.
fn translation_key(path: &str) -> String {
    let path = path.trim_start_matches('/');
    let stem = path.strip_suffix(".md").unwrap_or(path);
    match stem.rsplit_once('.') {
        // 마지막 조각이 짧으면 언어 코드로 본다. `my.file` 같은 이름을 언어로
        // 오인하지 않도록 길이로 거른다(ISO 639-1/2는 2~3자).
        Some((base, tail))
            if (2..=3).contains(&tail.len()) && tail.chars().all(|c| c.is_ascii_alphabetic()) =>
        {
            base.to_string()
        }
        _ => stem.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index() -> LinkIndex {
        let mut i = LinkIndex::new();
        i.insert(
            "start/installation".into(),
            [
                ("en".to_string(), "/start/installation/".to_string()),
                ("ko".to_string(), "/ko/start/installation/".to_string()),
            ]
            .into(),
        );
        i.insert(
            "start/_index".into(),
            [("en".to_string(), "/start/".to_string())].into(),
        );
        i
    }

    #[test]
    fn resolves_to_the_current_language() {
        let r = Resolver::new(index(), "ko", "en");
        assert_eq!(
            r.to_html("@/start/installation.md"),
            "/ko/start/installation/"
        );
        assert!(r.take_unresolved().is_empty());
    }

    #[test]
    fn falls_back_to_the_default_language() {
        // 한국어판이 없는 페이지를 한국어 문서에서 가리키면 영어판으로 간다.
        // 링크를 깨뜨리는 것보다 낫다.
        let r = Resolver::new(index(), "ko", "en");
        assert_eq!(r.to_html("@/start/_index.md"), "/start/");
    }

    #[test]
    fn keeps_the_fragment() {
        let r = Resolver::new(index(), "en", "en");
        assert_eq!(
            r.to_html("@/start/installation.md#requirements"),
            "/start/installation/#requirements"
        );
    }

    #[test]
    fn leaves_ordinary_urls_alone() {
        let r = Resolver::new(index(), "en", "en");
        for u in [
            "https://example.com",
            "/already/absolute/",
            "./relative",
            "#fragment",
            "mailto:a@b.c",
        ] {
            assert_eq!(r.to_html(u), u);
        }
        assert!(r.take_unresolved().is_empty());
    }

    #[test]
    fn records_unresolved_links_instead_of_shipping_them() {
        let r = Resolver::new(index(), "en", "en");
        r.to_html("@/does/not/exist.md");
        assert_eq!(r.take_unresolved(), vec!["@/does/not/exist.md".to_string()]);
    }

    #[test]
    fn language_suffixes_map_to_the_same_key() {
        assert_eq!(
            translation_key("start/installation.md"),
            "start/installation"
        );
        assert_eq!(
            translation_key("start/installation.ko.md"),
            "start/installation"
        );
        assert_eq!(translation_key("/start/_index.md"), "start/_index");
    }

    #[test]
    fn dotted_filenames_are_not_mistaken_for_languages() {
        // `notes.backup.md`의 "backup"은 언어 코드가 아니다.
        assert_eq!(translation_key("notes.backup.md"), "notes.backup");
        assert_eq!(translation_key("v1.2.md"), "v1.2");
    }
}
