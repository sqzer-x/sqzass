//! UI 문자열 번역 — `i18n/<언어>.toml`.
//!
//! 템플릿의 `{{ t("skip_to_content") }}` 가 지금 렌더 중인 페이지의 언어로 찾는다.
//!
//! **없는 키도, 그 언어에 없는 번역도 에러다.** 기본 언어로 조용히 떨어뜨리면
//! 한국어 페이지에 영어 라벨이 섞여 나가고, 그건 아무도 보고하지 않는 종류의
//! 버그다 — 미번역 페이지를 복제하지 않고 감추는 것과 같은 이유다.

use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::path::Path;

pub const I18N_DIR: &str = "i18n";

/// 언어 코드 → (키 → 문자열)
pub type Catalog = BTreeMap<String, BTreeMap<String, String>>;

/// `<root>/i18n/*.toml`을 읽는다. 디렉터리가 없으면 빈 카탈로그다 —
/// `t`를 쓰지 않는 사이트는 이 기능의 존재를 알 필요가 없다.
pub fn load(root: &Path) -> Result<Catalog> {
    let dir = root.join(I18N_DIR);
    if !dir.is_dir() {
        return Ok(Catalog::new());
    }

    let mut out = Catalog::new();
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .with_context(|| format!("{}을(를) 읽을 수 없습니다", dir.display()))?
        .filter_map(std::result::Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("toml"))
        .collect();
    // 파일 순서가 빌드 결과에 영향을 주지 않도록 정렬한다.
    entries.sort();

    for path in entries {
        let Some(lang) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("{}을(를) 읽을 수 없습니다", path.display()))?;
        let table: BTreeMap<String, String> = toml::from_str(&raw).with_context(|| {
            format!(
                "{} 파싱 실패 (값은 전부 문자열이어야 합니다)",
                path.display()
            )
        })?;
        out.insert(lang.to_string(), table);
    }

    Ok(out)
}

/// 키를 찾는다. 실패 이유를 사람이 고칠 수 있는 문장으로 돌려준다.
pub fn lookup(catalog: &Catalog, language: &str, key: &str) -> std::result::Result<String, String> {
    let Some(table) = catalog.get(language) else {
        return Err(format!(
            "'{language}' 번역 파일이 없습니다. {I18N_DIR}/{language}.toml 을 만드세요."
        ));
    };
    table.get(key).cloned().ok_or_else(|| {
        // 다른 언어에는 있는데 이 언어에만 없는 경우가 압도적으로 흔하다.
        // 그때는 "파일을 만들라"가 아니라 "이 줄을 더하라"가 맞는 조언이다.
        let elsewhere: Vec<&str> = catalog
            .iter()
            .filter(|(l, t)| l.as_str() != language && t.contains_key(key))
            .map(|(l, _)| l.as_str())
            .collect();
        if elsewhere.is_empty() {
            format!("번역 키 '{key}' 가 어느 언어에도 없습니다")
        } else {
            format!(
                "번역 키 '{key}' 가 {I18N_DIR}/{language}.toml 에 없습니다 \
                 ({} 에는 있습니다)",
                elsewhere.join(", ")
            )
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog() -> Catalog {
        Catalog::from([
            (
                "en".to_string(),
                BTreeMap::from([
                    ("home".to_string(), "Home".to_string()),
                    ("only_en".to_string(), "Only".to_string()),
                ]),
            ),
            (
                "ko".to_string(),
                BTreeMap::from([("home".to_string(), "홈".to_string())]),
            ),
        ])
    }

    #[test]
    fn looks_up_in_the_page_language() {
        let c = catalog();
        assert_eq!(lookup(&c, "ko", "home").unwrap(), "홈");
        assert_eq!(lookup(&c, "en", "home").unwrap(), "Home");
    }

    #[test]
    fn a_key_missing_in_one_language_says_where_it_does_exist() {
        // 기본 언어로 떨어뜨리면 한국어 페이지에 영어가 섞여 나가고,
        // 그건 아무도 보고하지 않는다. 그래서 에러다.
        let err = lookup(&catalog(), "ko", "only_en").unwrap_err();
        assert!(err.contains("only_en"), "실제: {err}");
        assert!(err.contains("ko.toml"), "실제: {err}");
        assert!(err.contains("en"), "어디에 있는지 알려줘야 한다: {err}");
    }

    #[test]
    fn an_unknown_key_is_not_a_missing_translation() {
        let err = lookup(&catalog(), "ko", "typo").unwrap_err();
        assert!(err.contains("어느 언어에도 없습니다"), "실제: {err}");
    }

    #[test]
    fn a_language_with_no_file_says_so() {
        let err = lookup(&catalog(), "ja", "home").unwrap_err();
        assert!(err.contains("i18n/ja.toml"), "실제: {err}");
    }
}
