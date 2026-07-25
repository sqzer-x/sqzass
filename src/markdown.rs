//! comrak 기반 마크다운 → HTML 변환, heading anchor, 목차 수집.
//!
//! 링크 재작성·anchor·TOC는 전부 comrak의 어댑터/AST 경로로 처리한다.
//! 최종 HTML에 정규식을 돌리는 방식은 쓰지 않는다 — 속성을 홑따옴표로 쓰거나
//! 따옴표를 빼면 조용히 처리에서 빠지는, 버그를 양산하는 구조다.

use crate::config::{HeadingAnchors, Markdown as MarkdownConfig};
use comrak::adapters::{HeadingAdapter, HeadingMeta};
use comrak::nodes::Sourcepos;
use comrak::options::Plugins;
use comrak::{Anchorizer, Options, markdown_to_html_with_plugins};
use serde::Serialize;
use std::fmt;
use std::sync::Mutex;

/// 목차 항목. 템플릿에는 중첩된 형태로 나간다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TocEntry {
    pub level: u8,
    pub id: String,
    pub title: String,
    pub children: Vec<TocEntry>,
}

pub struct Rendered {
    pub html: String,
    pub toc: Vec<TocEntry>,
}

pub struct Renderer {
    cfg: MarkdownConfig,
    highlighter: Option<crate::highlight::Highlighter>,
}

impl Renderer {
    pub fn new(cfg: &MarkdownConfig) -> Self {
        Self {
            cfg: cfg.clone(),
            highlighter: None,
        }
    }

    /// 구문 강조를 켠다. 없으면 코드 블록은 `<code class="language-x">`로만 나간다.
    pub fn with_highlighter(mut self, h: crate::highlight::Highlighter) -> Self {
        self.highlighter = Some(h);
        self
    }

    fn options<'p>(&self) -> Options<'p> {
        let mut o = Options::default();
        let c = &self.cfg;

        o.extension.footnotes = c.footnotes;
        o.extension.table = c.tables;
        o.extension.tasklist = c.tasklist;
        o.extension.strikethrough = c.strikethrough;
        o.extension.autolink = c.autolink;
        o.extension.alerts = c.alerts;
        // ⚠️ 한국어에 필수. 끄면 `**강조**한다` 같은 구성이 강조로 파싱되지 않는다.
        o.extension.cjk_friendly_emphasis = c.cjk_friendly_emphasis;

        // 우리 콘텐츠는 git에 들어있는 신뢰된 소스이므로 raw HTML을 허용한다.
        // 신뢰되지 않은 소스를 다루게 되면 소스별 신뢰 등급으로 나눌 것.
        o.render.r#unsafe = true;
        // false여야 `<code class="language-rust">`가 나온다. true면 `<pre lang="rust">`다.
        o.render.github_pre_lang = false;
        o
    }

    pub fn render(&self, body: &str) -> Rendered {
        // 어댑터는 `Options`가 아니라 `Plugins`에 붙는다. 페이지마다 새 collector를
        // 쓰는 게 중요하다 — heading id는 사이트가 아니라 **페이지 안에서** 유일해야
        // 하므로 anchorizer의 dedupe 카운터도 페이지마다 초기화되어야 한다.
        let headings = HeadingCollector::new(self.cfg.heading_anchors);
        let options = self.options();

        let html = {
            let mut plugins = Plugins::default();
            plugins.render.heading_adapter = Some(&headings);
            if let Some(h) = &self.highlighter {
                plugins.render.codefence_syntax_highlighter = Some(h);
            }
            markdown_to_html_with_plugins(body, &options, &plugins)
        };

        Rendered {
            html,
            toc: headings.into_toc(),
        }
    }
}

/// heading에 id를 붙이고 anchor 링크를 그리면서, 동시에 목차를 모은다.
///
/// **Anchorizer 인스턴스를 하나만 쓰는 게 핵심이다.** 같은 제목이 두 번 나오면
/// `설치`, `설치-1`처럼 접미사가 붙는데, TOC와 heading이 서로 다른 인스턴스를 쓰면
/// 두 쪽의 id가 어긋나 목차 링크가 죽는다.
struct HeadingCollector {
    anchors: HeadingAnchors,
    state: Mutex<CollectorState>,
}

#[derive(Default)]
struct CollectorState {
    anchorizer: Anchorizer,
    flat: Vec<(u8, String, String)>, // (level, id, title)
    /// `enter`에서 만든 id를 `exit`가 다시 써야 한다. 다시 anchorize하면
    /// dedupe 카운터가 올라가 서로 다른 id가 나온다.
    current_id: Option<String>,
}

impl HeadingCollector {
    fn new(anchors: HeadingAnchors) -> Self {
        Self {
            anchors,
            state: Mutex::new(CollectorState::default()),
        }
    }

    fn into_toc(self) -> Vec<TocEntry> {
        let flat = self
            .state
            .into_inner()
            .unwrap_or_else(|e| e.into_inner())
            .flat;
        nest(&flat)
    }
}

impl HeadingAdapter for HeadingCollector {
    fn enter(
        &self,
        out: &mut dyn fmt::Write,
        heading: &HeadingMeta,
        _sourcepos: Option<Sourcepos>,
    ) -> fmt::Result {
        let id = {
            let mut st = self.state.lock().unwrap_or_else(|e| e.into_inner());
            let id = st.anchorizer.anchorize(&heading.content);
            st.flat
                .push((heading.level, id.clone(), heading.content.clone()));
            st.current_id = Some(id.clone());
            id
        };

        // anchor 표시가 꺼져 있어도 id는 항상 넣는다 — 목차와 딥링크가 이걸 쓴다.
        write!(out, "<h{} id=\"{}\">", heading.level, attr_escape(&id))?;
        if self.anchors == HeadingAnchors::Left {
            write_anchor(out, &id)?;
        }
        Ok(())
    }

    fn exit(&self, out: &mut dyn fmt::Write, heading: &HeadingMeta) -> fmt::Result {
        if self.anchors == HeadingAnchors::Right {
            let id = self
                .state
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .current_id
                .clone()
                .unwrap_or_default();
            write_anchor(out, &id)?;
        }
        write!(out, "</h{}>", heading.level)
    }
}

fn write_anchor(out: &mut dyn fmt::Write, id: &str) -> fmt::Result {
    // aria-hidden + tabindex=-1: 스크린리더와 키보드 탐색에 잡음을 더하지 않는다.
    write!(
        out,
        "<a class=\"anchor\" href=\"#{}\" aria-hidden=\"true\" tabindex=\"-1\">#</a>",
        attr_escape(id)
    )
}

/// 속성값 이스케이프. id는 Anchorizer가 만들지만 방어적으로 처리한다.
fn attr_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(ch),
        }
    }
    out
}

/// 평면 heading 목록을 레벨에 따라 중첩시킨다.
///
/// 문서가 h2에서 h4로 건너뛰는 건 흔한 일이므로, 레벨이 연속이라고 가정하지 않고
/// "직전보다 깊으면 자식"으로만 판단한다.
fn nest(flat: &[(u8, String, String)]) -> Vec<TocEntry> {
    let mut roots: Vec<TocEntry> = Vec::new();
    // 현재 열려 있는 조상들의 경로 (인덱스 스택)
    let mut stack: Vec<u8> = Vec::new();

    for (level, id, title) in flat {
        let entry = TocEntry {
            level: *level,
            id: id.clone(),
            title: title.clone(),
            children: Vec::new(),
        };

        while stack.last().is_some_and(|&l| l >= *level) {
            stack.pop();
        }

        if stack.is_empty() {
            roots.push(entry);
        } else {
            // 스택 깊이만큼 내려가 마지막 자식에 붙인다
            let mut cur = roots.last_mut().expect("스택이 비지 않았으면 루트가 있다");
            for _ in 1..stack.len() {
                cur = cur.children.last_mut().expect("경로가 유효해야 한다");
            }
            cur.children.push(entry);
        }
        stack.push(*level);
    }

    roots
}

#[cfg(test)]
mod tests {
    use super::*;

    fn renderer() -> Renderer {
        Renderer::new(&MarkdownConfig::default())
    }

    #[test]
    fn renders_basic_markdown() {
        let r = renderer().render("# Hello\n\nWorld.");
        assert!(r.html.contains("Hello"), "실제: {}", r.html);
        assert!(r.html.contains("<p>World.</p>"), "실제: {}", r.html);
    }

    #[test]
    fn korean_emphasis_attached_to_particle_parses() {
        // cjk_friendly_emphasis가 꺼져 있으면 이게 <em>으로 파싱되지 않는다.
        // 한국어 문서에서 매우 흔한 형태라 회귀하면 바로 알아야 한다.
        let r = renderer().render("**정적 사이트 생성기**를 만들었다.");
        assert!(
            r.html.contains("<strong>정적 사이트 생성기</strong>"),
            "실제: {}",
            r.html
        );
    }

    #[test]
    fn code_fence_carries_language_class() {
        let r = renderer().render("```rust\nfn main() {}\n```");
        assert!(r.html.contains("language-rust"), "실제: {}", r.html);
    }

    #[test]
    fn gfm_table_renders() {
        let r = renderer().render("| a | b |\n|---|---|\n| 1 | 2 |");
        assert!(r.html.contains("<table>"), "실제: {}", r.html);
    }

    #[test]
    fn raw_html_passes_through() {
        let r = renderer().render("<details><summary>x</summary>y</details>");
        assert!(r.html.contains("<details>"), "실제: {}", r.html);
    }

    #[test]
    fn headings_get_ids_and_anchors() {
        let r = renderer().render("## Getting started");
        assert!(
            r.html.contains(r#"<h2 id="getting-started">"#),
            "실제: {}",
            r.html
        );
        assert!(
            r.html.contains(r##"href="#getting-started""##),
            "실제: {}",
            r.html
        );
    }

    #[test]
    fn korean_headings_keep_hangul_ids() {
        let r = renderer().render("## 시작하기\n\n### 설치");
        assert!(r.html.contains(r#"<h2 id="시작하기">"#), "실제: {}", r.html);
        assert!(r.html.contains(r#"<h3 id="설치">"#), "실제: {}", r.html);
    }

    #[test]
    fn duplicate_headings_get_distinct_ids_shared_with_toc() {
        // heading과 TOC가 서로 다른 Anchorizer를 쓰면 여기서 어긋난다.
        let r = renderer().render("## 설치\n\n## 설치");
        assert!(r.html.contains(r#"id="설치""#), "실제: {}", r.html);
        assert!(r.html.contains(r#"id="설치-1""#), "실제: {}", r.html);
        let ids: Vec<&str> = r.toc.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, vec!["설치", "설치-1"]);
    }

    #[test]
    fn toc_nests_by_level() {
        let r = renderer().render("# A\n\n## A1\n\n### A1a\n\n## A2\n\n# B\n");
        assert_eq!(r.toc.len(), 2, "최상위는 A와 B: {:?}", r.toc);
        assert_eq!(r.toc[0].title, "A");
        assert_eq!(r.toc[0].children.len(), 2);
        assert_eq!(r.toc[0].children[0].title, "A1");
        assert_eq!(r.toc[0].children[0].children[0].title, "A1a");
        assert_eq!(r.toc[0].children[1].title, "A2");
        assert_eq!(r.toc[1].title, "B");
    }

    #[test]
    fn toc_survives_skipped_levels() {
        // h2 다음에 h4로 뛰는 문서는 흔하다. 레벨이 연속이라고 가정하면 여기서 깨진다.
        let r = renderer().render("## Top\n\n#### Deep\n");
        assert_eq!(r.toc.len(), 1);
        assert_eq!(r.toc[0].children.len(), 1);
        assert_eq!(r.toc[0].children[0].title, "Deep");
    }

    #[test]
    fn heading_content_is_flattened_for_the_toc() {
        let r = renderer().render("## This is **bold**");
        assert_eq!(r.toc[0].title, "This is bold");
    }

    #[test]
    fn anchors_can_be_disabled_but_ids_remain() {
        let cfg = MarkdownConfig {
            heading_anchors: HeadingAnchors::None,
            ..MarkdownConfig::default()
        };
        let r = Renderer::new(&cfg).render("## Title");
        assert!(r.html.contains(r#"<h2 id="title">"#), "실제: {}", r.html);
        assert!(
            !r.html.contains("class=\"anchor\""),
            "anchor를 껐는데 나왔다: {}",
            r.html
        );
        // 목차는 여전히 동작해야 한다
        assert_eq!(r.toc[0].id, "title");
    }
}
