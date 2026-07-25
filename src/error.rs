//! 에러 분류와 exit code.
//!
//! 분류는 에러가 나는 자리마다 붙이지 않고 **파이프라인 단계 경계**에서 한 번씩만
//! 붙인다. 스무 군데에 종류를 다는 건 스무 번 틀릴 기회이고, 정작 사용자가 알고
//! 싶은 건 "설정이 문제냐, 콘텐츠가 문제냐, 템플릿이 문제냐"뿐이다.
//!
//! exit code는 **CI가 조건을 걸 수 있게** 하려고 나눈다. 사람이 읽는 건 메시지이고,
//! 코드는 스크립트가 읽는다.

use anyhow::Error;

/// 무엇이 잘못됐는지. 값은 exit code이기도 하므로 **번호를 재사용하지 말 것** —
/// 기존 스크립트의 조건이 조용히 다른 뜻이 된다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// `sqzass.toml`
    Config = 3,
    /// `content/` 아래의 마크다운
    Content = 4,
    /// `templates/`, `i18n/`, 에셋 조회
    Template = 5,
    /// 읽기/쓰기 실패
    Io = 6,
    /// 분류되지 않은 것. 새 단계를 추가하고 태그를 잊으면 여기로 온다.
    Other = 1,
}

impl Kind {
    /// 문서와 검색에 쓰는 안정된 식별자. 메시지에 같이 나간다.
    pub fn id(self) -> &'static str {
        match self {
            Self::Config => "SQZASS_E_CONFIG",
            Self::Content => "SQZASS_E_CONTENT",
            Self::Template => "SQZASS_E_TEMPLATE",
            Self::Io => "SQZASS_E_IO",
            Self::Other => "SQZASS_E",
        }
    }

    pub fn code(self) -> i32 {
        self as i32
    }

    /// `.map_err(Kind::Config.tag())` 형태로 단계 경계에 붙인다.
    pub fn tag(self) -> impl Fn(Error) -> Error {
        move |e| e.context(self)
    }
}

impl std::fmt::Display for Kind {
    /// anyhow의 컨텍스트 사슬에 그대로 찍히면 사용자에게 잡음이다. 식별자만 낸다.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.id())
    }
}

/// 사슬에서 `downcast_ref`로 다시 꺼내려면 이게 있어야 한다. anyhow의 사슬은
/// `&dyn std::error::Error`를 내주고, 그쪽 downcast만 타입을 되찾을 수 있다.
impl std::error::Error for Kind {}

/// 모든 분류. 메시지 정리와 테스트가 쓴다.
const ALL: &[Kind] = &[
    Kind::Config,
    Kind::Content,
    Kind::Template,
    Kind::Io,
    Kind::Other,
];

/// 에러에 붙은 분류를 꺼낸다. 없으면 `Other`.
///
/// `chain()`을 훑어 `downcast_ref`를 하면 안 된다 — 사슬이 내주는 건 anyhow가 컨텍스트를
/// 감싼 내부 타입이고, 거기서는 `Kind`를 되찾을 수 없다. `Error::downcast_ref`는 컨텍스트
/// 값까지 뒤져 주는 별개의 경로다.
pub fn kind_of(err: &Error) -> Kind {
    err.downcast_ref::<Kind>().copied().unwrap_or(Kind::Other)
}

/// 사람이 읽을 메시지. 분류는 맨 앞에 한 번만 보이고 사슬 중간에서는 지운다.
pub fn message(err: &Error) -> String {
    let kind = kind_of(err);
    let body = err
        .chain()
        .map(std::string::ToString::to_string)
        // 사슬에 낀 분류 계층은 자기 식별자를 Display로 낸다. 바깥 것만이 아니라
        // 어떤 분류든 걸러낸다 — 하나라도 새면 사용자는 그게 메시지의 일부라고 읽는다.
        .filter(|s| !ALL.iter().any(|k| k.id() == s))
        .collect::<Vec<_>>()
        .join(": ");
    format!("[{}] {body}", kind.id())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    #[test]
    fn tags_survive_further_context() {
        let e = anyhow!("파일이 없습니다");
        let e = Kind::Config.tag()(e);
        let e = e.context("설정을 읽는 중");
        assert_eq!(kind_of(&e), Kind::Config);
    }

    #[test]
    fn untagged_errors_are_other() {
        assert_eq!(kind_of(&anyhow!("무언가")), Kind::Other);
        assert_eq!(Kind::Other.code(), 1);
    }

    #[test]
    fn message_shows_the_id_once_and_keeps_the_chain() {
        let e = Kind::Content.tag()(anyhow!("front matter 파싱 실패"));
        let e = e.context("start/install.md");
        let msg = message(&e);
        assert!(msg.starts_with("[SQZASS_E_CONTENT] "), "실제: {msg}");
        assert!(msg.contains("start/install.md"), "실제: {msg}");
        assert!(msg.contains("front matter 파싱 실패"), "실제: {msg}");
        // 분류가 사슬 중간에 한 번 더 끼어들면 안 된다.
        assert_eq!(msg.matches("SQZASS_E_CONTENT").count(), 1, "실제: {msg}");
    }

    /// 두 번 태그하면 바깥이 이기고 안쪽은 메시지로 샌다. 실제로 한 번 그랬다:
    /// 깨진 `@/` 링크가 콘텐츠가 아니라 템플릿 에러로 보고됐다.
    #[test]
    fn a_nested_tag_never_leaks_into_the_message() {
        let e = Kind::Content.tag()(anyhow!("깨진 링크"));
        let e = Kind::Template.tag()(e);
        let msg = message(&e);
        assert!(!msg.contains("SQZASS_E_CONTENT"), "안쪽 분류가 샜다: {msg}");
        assert!(msg.contains("깨진 링크"), "실제: {msg}");
    }

    /// exit code는 계약이다. 바꾸면 남의 CI 조건이 조용히 뜻을 바꾼다.
    #[test]
    fn codes_are_stable_and_distinct() {
        assert_eq!(
            ALL.iter().map(|k| k.code()).collect::<Vec<_>>(),
            vec![3, 4, 5, 6, 1],
            "exit code가 바뀌었다"
        );
        // clap이 사용법 오류에 2를 쓰므로 우리는 2를 쓰지 않는다.
        assert!(!ALL.iter().any(|k| k.code() == 2));
        // 식별자도 계약이다. 문서와 사용자 스크립트가 문자열로 잡는다.
        let ids: Vec<&str> = ALL.iter().map(|k| k.id()).collect();
        assert_eq!(
            ids,
            vec![
                "SQZASS_E_CONFIG",
                "SQZASS_E_CONTENT",
                "SQZASS_E_TEMPLATE",
                "SQZASS_E_IO",
                "SQZASS_E"
            ]
        );
    }
}
