//! `sqzass.toml` 파싱.
//!
//! 설계 원칙: 작고 직교하게. 아무 일도 하지 않는 설정 키는 넣지 않는다.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

pub const CONFIG_FILE: &str = "sqzass.toml";

/// `sqzass.toml`이 가질 수 있는 최상위 키. 오타 감지에 쓴다.

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub base_url: String,
    #[serde(default = "default_language")]
    pub default_language: String,

    #[serde(default)]
    pub languages: BTreeMap<String, Language>,
    #[serde(default)]
    pub build: Build,
    #[serde(default)]
    pub markdown: Markdown,
    #[serde(default)]
    pub highlight: Highlight,
    #[serde(default)]
    pub assets: Assets,
    #[serde(default)]
    pub nav: Nav,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Language {
    pub name: String,
    #[serde(default)]
    pub weight: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Build {
    pub output_dir: String,
    pub drafts: bool,
}

impl Default for Build {
    fn default() -> Self {
        Self {
            output_dir: "public".into(),
            drafts: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Markdown {
    pub footnotes: bool,
    pub tables: bool,
    pub tasklist: bool,
    pub strikethrough: bool,
    pub autolink: bool,
    /// GitHub `> [!NOTE]` 스타일 admonition.
    pub alerts: bool,
    /// ⚠️ 한국어에 필수. 끄면 `**강조**한다` 같은 구성이 파싱되지 않는다.
    pub cjk_friendly_emphasis: bool,
    /// `none` | `left` | `right`
    pub heading_anchors: HeadingAnchors,
}

impl Default for Markdown {
    fn default() -> Self {
        Self {
            footnotes: true,
            tables: true,
            tasklist: true,
            strikethrough: true,
            autolink: true,
            alerts: true,
            cjk_friendly_emphasis: true,
            heading_anchors: HeadingAnchors::Right,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HeadingAnchors {
    None,
    Left,
    Right,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Highlight {
    pub enabled: bool,
    pub theme_light: String,
    pub theme_dark: String,
}

impl Default for Highlight {
    fn default() -> Self {
        Self {
            enabled: true,
            theme_light: "InspiredGitHub".into(),
            theme_dark: "base16-ocean.dark".into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Assets {
    pub source_dir: String,
    pub fingerprint: bool,
}

impl Default for Assets {
    fn default() -> Self {
        Self {
            source_dir: "static".into(),
            fingerprint: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Nav {
    /// `weight` | `title`
    pub sort_by: SortBy,
}

impl Default for Nav {
    fn default() -> Self {
        Self {
            sort_by: SortBy::Weight,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortBy {
    Weight,
    Title,
}

fn default_language() -> String {
    "en".into()
}

impl Config {
    /// 사이트 루트에서 `sqzass.toml`을 읽는다.
    ///
    /// 모르는 키는 **에러**다. 조용히 무시되는 설정은 "깨진 참조는 빌드를 멈춘다"는
    /// 원칙의 정확한 위반이다 — `theme_ligth`라고 쓴 사람은 테마를 바꾼 줄 알고 있고,
    /// 바뀌지 않은 이유를 찾느라 시간을 쓴다.
    ///
    /// 후보 목록과 줄 번호는 serde와 toml이 만들어 준다. 유효한 키 목록을 손으로
    /// 관리하면 필드를 추가할 때마다 같이 고쳐야 하고, 언젠가 잊는다.
    pub fn load(root: &Path) -> Result<Self> {
        let path = root.join(CONFIG_FILE);
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("{}을(를) 읽을 수 없습니다", path.display()))?;

        let cfg: Config =
            toml::from_str(&raw).with_context(|| format!("{} 파싱 실패", path.display()))?;
        cfg.validate()?;
        Ok(cfg)
    }

    fn validate(&self) -> Result<()> {
        if !self.languages.is_empty() && !self.languages.contains_key(&self.default_language) {
            anyhow::bail!(
                "default_language = \"{}\" 인데 [languages.{}] 가 정의되어 있지 않습니다",
                self.default_language,
                self.default_language
            );
        }
        Ok(())
    }

    /// `base_url`에서 뒤쪽 슬래시를 제거한 형태. URL 조립에 쓴다.
    pub fn base_url_trimmed(&self) -> &str {
        self.base_url.trim_end_matches('/')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 조용히 무시되는 설정은 "깨진 참조는 빌드를 멈춘다"의 정확한 위반이다.
    #[test]
    fn unknown_keys_are_an_error() {
        let err = toml::from_str::<Config>(
            r#"
            title = "t"
            base_url = "https://example.com"
            [highlight]
            theme_ligth = "InspiredGitHub"
            "#,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("theme_ligth"), "실제: {err}");
        // 후보 목록은 serde가 만들어 준다. 손으로 관리하는 목록은 언젠가 어긋난다.
        assert!(err.contains("theme_light"), "실제: {err}");
    }

    #[test]
    fn unknown_top_level_keys_are_an_error() {
        let err = toml::from_str::<Config>(
            r#"
            title = "t"
            base_url = "https://example.com"
            [markdwon]
            tables = false
            "#,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("markdwon"), "실제: {err}");
        assert!(err.contains("markdown"), "실제: {err}");
    }

    #[test]
    fn defaults_apply() {
        let cfg: Config = toml::from_str(
            r#"
            title = "t"
            base_url = "https://example.com"
            "#,
        )
        .unwrap();
        assert_eq!(cfg.default_language, "en");
        assert_eq!(cfg.build.output_dir, "public");
        // 한국어 필수 항목이 기본으로 켜져 있어야 한다
        assert!(cfg.markdown.cjk_friendly_emphasis);
        assert_eq!(cfg.markdown.heading_anchors, HeadingAnchors::Right);
    }

    #[test]
    fn rejects_default_language_without_definition() {
        let cfg: Result<Config> = toml::from_str::<Config>(
            r#"
            title = "t"
            base_url = "https://example.com"
            default_language = "en"
            [languages.ko]
            name = "한국어"
            "#,
        )
        .map_err(Into::into);
        assert!(cfg.unwrap().validate().is_err());
    }
}
