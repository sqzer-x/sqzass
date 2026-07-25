//! `sqzass.toml` 파싱.
//!
//! 설계 원칙: 작고 직교하게. 아무 일도 하지 않는 설정 키는 넣지 않는다.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

pub const CONFIG_FILE: &str = "sqzass.toml";

/// `sqzass.toml`이 가질 수 있는 최상위 키. 오타 감지에 쓴다.
const KNOWN_TOP_LEVEL: &[&str] = &[
    "title",
    "description",
    "base_url",
    "default_language",
    "languages",
    "build",
    "markdown",
    "highlight",
    "assets",
    "nav",
];

#[derive(Debug, Clone, Deserialize)]
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
pub struct Language {
    pub name: String,
    #[serde(default)]
    pub weight: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
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
#[serde(default)]
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
#[serde(default)]
pub struct Highlight {
    pub theme_light: String,
    pub theme_dark: String,
    pub line_numbers: bool,
}

impl Default for Highlight {
    fn default() -> Self {
        Self {
            theme_light: "InspiredGitHub".into(),
            theme_dark: "base16-ocean.dark".into(),
            line_numbers: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
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
#[serde(default)]
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
    /// 알 수 없는 최상위 키는 **경고**하고 계속 진행한다 — 오타 하나로 빌드를 막는 것보다
    /// "이 키는 읽히지 않는다"고 알려주는 쪽이 낫다. 설정 오타는 모든 SSG의 1순위
    /// 지원 부담이고, 고치는 비용이 거의 0이다.
    pub fn load(root: &Path) -> Result<Self> {
        let path = root.join(CONFIG_FILE);
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("{}을(를) 읽을 수 없습니다", path.display()))?;

        warn_unknown_keys(&raw, &path);

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

fn warn_unknown_keys(raw: &str, path: &Path) {
    let Ok(table) = raw.parse::<toml::Table>() else {
        return; // 진짜 파싱 에러는 아래에서 제대로 보고된다
    };
    for key in table.keys() {
        if KNOWN_TOP_LEVEL.contains(&key.as_str()) {
            continue;
        }
        let hint = closest(key, KNOWN_TOP_LEVEL)
            .map(|s| format!(" '{s}'을(를) 의도하셨나요?"))
            .unwrap_or_default();
        eprintln!(
            "warning: {}의 '{}' 키는 sqzass가 읽지 않습니다.{}",
            path.display(),
            key,
            hint
        );
    }
}

/// 편집 거리가 충분히 가까운 후보를 고른다.
fn closest<'a>(needle: &str, haystack: &[&'a str]) -> Option<&'a str> {
    let max = (needle.len() / 3).max(1);
    haystack
        .iter()
        .map(|c| (*c, levenshtein(needle, c)))
        .filter(|(_, d)| *d <= max)
        .min_by_key(|(_, d)| *d)
        .map(|(c, _)| c)
}

fn levenshtein(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];

    for (i, ca) in a.chars().enumerate() {
        cur[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != *cb);
            cur[j + 1] = (prev[j + 1] + 1).min(cur[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levenshtein_basics() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("markdown", "markdown"), 0);
        assert_eq!(levenshtein("markdonw", "markdown"), 2);
        assert_eq!(levenshtein("title", "titel"), 2);
    }

    #[test]
    fn suggests_close_key() {
        assert_eq!(closest("markdonw", KNOWN_TOP_LEVEL), Some("markdown"));
        assert_eq!(closest("titl", KNOWN_TOP_LEVEL), Some("title"));
        // 전혀 다른 키에는 엉뚱한 제안을 하지 않는다
        assert_eq!(closest("completely_unrelated", KNOWN_TOP_LEVEL), None);
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
