//! 검색 색인 — 언어마다 JSON 한 벌.
//!
//! **부분 문자열로 찾는다. 형태소 분석도, 단어 색인도 쓰지 않는다.**
//! 한국어는 명사에 조사가 붙어 어절이 되고 합성어를 붙여 쓰기 때문에, 단어 단위로
//! 쪼갠 색인은 `검색엔진최적화` 안의 `최적화`를 영영 못 찾는다. 형태소 분석기를
//! 붙이면 이번엔 사전에 없는 외래어가 뭉개진다(`템플릿` → `템플`+`릿`) — 기술 문서엔
//! 치명적이다. 원문을 그대로 두고 부분 문자열로 훑으면 둘 다 걸린다.
//!
//! 대가는 색인이 본문 전체라는 것이다. 문서 사이트 규모에서 이건 문제가 아니고,
//! 브라우저는 첫 검색 때 한 번만 받는다.
//!
//! 파일명에 해시를 붙이지 않는다. 색인은 `<link>`나 `<script>`가 아니라 JS가 런타임에
//! 가져가는 데이터고, 게다가 색인 내용은 렌더된 페이지에서 나오는데 템플릿은 렌더
//! 전에 에셋 URL을 조회한다 — 해시를 붙이면 순환이 된다. 잠깐 옛 색인이 뜨는 건
//! 검색 결과 몇 줄의 문제라 그 대가를 치를 이유가 없다.

use serde::Serialize;

/// 색인 한 행. 키를 한 글자로 줄인 건 취향이 아니라 크기다 — 행마다 반복되므로
/// `"title"` 대신 `"t"`를 쓰면 수백 페이지에서 수십 KB가 빠진다.
#[derive(Debug, Clone, Serialize)]
pub struct Entry {
    /// 제목
    pub t: String,
    /// 설명. 없으면 통째로 뺀다.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub d: String,
    /// URL
    pub u: String,
    /// 소속 섹션 제목. 최상위 페이지엔 없다.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub s: String,
    /// 본문 평문
    pub c: String,
}

/// 해당 언어 색인의 출력 경로. 클라이언트가 `<html lang>`으로 같은 이름을 만든다.
pub fn path(language: &str) -> String {
    format!("search-{language}.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_is_per_language() {
        assert_eq!(path("ko"), "search-ko.json");
        assert_eq!(path("en"), "search-en.json");
    }

    #[test]
    fn empty_optional_fields_are_omitted() {
        let e = Entry {
            t: "설치".into(),
            d: String::new(),
            u: "/ko/start/installation/".into(),
            s: String::new(),
            c: "소스에서 빌드합니다".into(),
        };
        let json = serde_json::to_string(&e).unwrap();
        assert!(!json.contains("\"d\""), "빈 설명이 나갔다: {json}");
        assert!(!json.contains("\"s\""), "빈 섹션이 나갔다: {json}");
        assert!(json.contains("\"t\":\"설치\""), "실제: {json}");
    }
}
