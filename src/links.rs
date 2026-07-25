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

/// 이 사이트가 실제로 내보내는 URL들. 루트 절대 경로 링크를 검증하는 데 쓴다.
#[derive(Debug, Clone, Default)]
pub struct KnownUrls {
    /// 페이지 URL (`/start/install/`)과 에셋 URL (`/css/main.abc.css`)
    pub urls: std::collections::BTreeSet<String>,
}

pub struct Resolver {
    index: LinkIndex,
    language: String,
    default_language: String,
    /// 해석하지 못한 링크. 렌더가 끝난 뒤 빌드를 실패시키는 데 쓴다.
    unresolved: Mutex<Vec<String>>,
    /// 이 사이트가 내보내는 URL 전체. 비어 있으면 절대 경로 검사를 건너뛴다.
    known: KnownUrls,
}

impl Resolver {
    pub fn new(index: LinkIndex, language: &str, default_language: &str) -> Self {
        Self {
            index,
            language: language.to_string(),
            default_language: default_language.to_string(),
            unresolved: Mutex::new(Vec::new()),
            known: KnownUrls::default(),
        }
    }

    /// 루트 절대 경로 링크를 검증할 수 있게 사이트가 내보내는 URL 목록을 준다.
    ///
    /// `@/`만 검사하면 "깨진 참조는 빌드를 멈춘다"가 반쪽이 된다. 사람이 가장
    /// 자연스럽게 쓰는 건 `[설치](/start/install/)`인데, 그건 지금까지 아무도
    /// 검사하지 않았고 프로덕션에서 404가 됐다.
    pub fn with_known_urls(mut self, known: KnownUrls) -> Self {
        self.known = known;
        self
    }

    /// 사이트 안을 가리키는 루트 절대 경로인지. 스킴·프로토콜 상대·`#`·`?`는 제외.
    fn is_local_absolute(url: &str) -> bool {
        url.starts_with('/') && !url.starts_with("//")
    }

    /// 링크가 실제로 존재하는 곳을 가리키는지.
    fn known_target(&self, url: &str) -> bool {
        // 프래그먼트와 쿼리를 떼고 본다. `/start/#설치`는 `/start/`를 가리킨다.
        let path = url.split(['#', '?']).next().unwrap_or(url);
        if path.is_empty() {
            return true; // `#anchor` 단독 — 같은 페이지 안이다.
        }
        if self.known.urls.contains(path) {
            return true;
        }
        // 디렉터리 URL은 슬래시가 있으나 없으나 같은 곳을 가리킨다. 호스트가
        // 리다이렉트해 주므로 죽은 링크는 아니다.
        let with_slash = format!("{path}/");
        self.known.urls.contains(&with_slash)
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
            // `@/`가 아니어도 사이트 안을 가리키는 절대 경로라면 존재를 확인한다.
            if !self.known.urls.is_empty()
                && Self::is_local_absolute(url)
                && !self.known_target(url)
            {
                self.unresolved
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(url.to_string());
            }
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
mod absolute_link_tests {
    use super::*;
    use comrak::options::URLRewriter;
    use std::collections::BTreeSet;

    fn resolver(urls: &[&str]) -> Resolver {
        Resolver::new(LinkIndex::new(), "en", "en").with_known_urls(KnownUrls {
            urls: urls
                .iter()
                .map(|s| (*s).to_string())
                .collect::<BTreeSet<_>>(),
        })
    }

    /// `@/`만 검사하면 "깨진 참조는 빌드를 멈춘다"가 반쪽이다. 사람이 가장
    /// 자연스럽게 쓰는 건 `[설치](/start/install/)`이고, 그건 검사된 적이 없었다.
    #[test]
    fn a_dead_absolute_path_is_unresolved() {
        let r = resolver(&["/about/"]);
        r.to_html("/nope/");
        assert_eq!(r.take_unresolved(), vec!["/nope/".to_string()]);
    }

    #[test]
    fn live_paths_anchors_and_external_urls_pass() {
        let r = resolver(&["/about/", "/images/x.png"]);
        for url in [
            "/about/",
            "/about/#section",
            "/about/?q=1",
            "/images/x.png",
            "#same-page",
            "https://example.com/x",
            // 프로토콜 상대 URL은 남의 호스트다. 우리 사이트가 아니다.
            "//cdn.example.com/x",
            "mailto:a@b.c",
        ] {
            r.to_html(url);
        }
        assert!(r.take_unresolved().is_empty());
    }

    /// 알려진 URL이 없으면(에셋 수집 전 등) 검사를 아예 하지 않는다.
    /// 반쯤 아는 상태로 판정하면 멀쩡한 링크를 죽었다고 말하게 된다.
    #[test]
    fn no_known_urls_means_no_checking() {
        let r = Resolver::new(LinkIndex::new(), "en", "en");
        r.to_html("/anything/");
        assert!(r.take_unresolved().is_empty());
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
