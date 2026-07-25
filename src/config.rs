//! `sqzass.toml` 파싱.
//!
//! 설계 원칙: 작고 직교하게. 아무 일도 하지 않는 설정 키는 넣지 않는다.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

pub const CONFIG_FILE: &str = "sqzass.toml";

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

    /// `base_url`에서 뒤쪽 슬래시를 제거한 형태.
    pub fn base_url_trimmed(&self) -> &str {
        self.base_url.trim_end_matches('/')
    }

    /// 스킴과 호스트만. `https://user.github.io/repo` → `https://user.github.io`
    ///
    /// 절대 URL(canonical, sitemap, OpenGraph)을 조립할 때 쓴다. 페이지 URL이 이미
    /// 서브경로를 품고 있으므로 여기에 경로가 또 들어가면 두 번 붙는다.
    pub fn origin(&self) -> &str {
        let trimmed = self.base_url_trimmed();
        // `://` 다음의 첫 슬래시가 경로의 시작이다. 스킴이 없으면 경로도 없다고 본다.
        match trimmed.find("://") {
            Some(i) => match trimmed[i + 3..].find('/') {
                Some(j) => &trimmed[..i + 3 + j],
                None => trimmed,
            },
            None => trimmed,
        }
    }

    /// 사이트가 도메인 루트가 아닌 곳에 놓일 때의 경로. 루트면 빈 문자열.
    ///
    /// `https://user.github.io/repo` → `/repo`
    ///
    /// GitHub·GitLab·Codeberg의 **프로젝트 페이지가 기본으로 이 모양**이다. 도메인을
    /// 따로 사지 않은 사람이 가장 먼저 만나는 형태이므로, 여기서 URL이 어긋나면
    /// 사이트 전체가 404가 된다 — 빌드는 성공한 채로.
    pub fn base_path(&self) -> &str {
        let trimmed = self.base_url_trimmed();
        let origin = self.origin();
        trimmed.strip_prefix(origin).unwrap_or("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 프로젝트 페이지(`user.github.io/repo`)는 도메인을 사지 않은 사람의 기본
    /// 상황이다. 여기서 origin과 경로를 잘못 나누면 사이트 전체가 404가 된다.
    #[test]
    fn splits_origin_from_base_path() {
        let cfg = |url: &str| -> Config {
            toml::from_str(&format!("title = \"t\"\nbase_url = \"{url}\"\n")).unwrap()
        };

        let root = cfg("https://sqzass.sqzer.com");
        assert_eq!(root.origin(), "https://sqzass.sqzer.com");
        assert_eq!(root.base_path(), "", "루트 사이트에는 접두사가 없다");

        let trailing = cfg("https://sqzass.sqzer.com/");
        assert_eq!(trailing.origin(), "https://sqzass.sqzer.com");
        assert_eq!(trailing.base_path(), "");

        let project = cfg("https://user.github.io/repo");
        assert_eq!(project.origin(), "https://user.github.io");
        assert_eq!(project.base_path(), "/repo");

        let nested = cfg("https://example.com/a/b/");
        assert_eq!(nested.origin(), "https://example.com");
        assert_eq!(nested.base_path(), "/a/b");

        let port = cfg("http://localhost:3000/site");
        assert_eq!(port.origin(), "http://localhost:3000");
        assert_eq!(port.base_path(), "/site");
    }

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
