//! comrak 기반 마크다운 → HTML 변환.
//!
//! M1에서 syntect 하이라이터와 heading anchor는 `Options.render`의
//! `codefence_syntax_highlighter` / `heading_adapter` 자리에 붙는다.
//! 최종 HTML에 정규식을 돌리는 방식은 쓰지 않는다 — 속성을 홑따옴표로 쓰거나
//! 따옴표를 빼면 조용히 처리에서 빠지는, 버그를 양산하는 구조다.

use crate::config::Markdown as MarkdownConfig;
use comrak::{Options, markdown_to_html};

pub struct Renderer<'a> {
    options: Options<'a>,
}

impl<'a> Renderer<'a> {
    pub fn new(cfg: &MarkdownConfig) -> Self {
        let mut o = Options::default();

        o.extension.footnotes = cfg.footnotes;
        o.extension.table = cfg.tables;
        o.extension.tasklist = cfg.tasklist;
        o.extension.strikethrough = cfg.strikethrough;
        o.extension.autolink = cfg.autolink;
        o.extension.alerts = cfg.alerts;
        // ⚠️ 한국어에 필수. 끄면 `**강조**한다` 같은 구성이 강조로 파싱되지 않는다.
        o.extension.cjk_friendly_emphasis = cfg.cjk_friendly_emphasis;

        // 우리 콘텐츠는 git에 들어있는 신뢰된 소스이므로 raw HTML을 허용한다.
        // (`<details>`, 임베드 등이 문서 사이트에 필요하다.)
        // 신뢰되지 않은 소스를 다루게 되면 소스별 신뢰 등급으로 나눌 것.
        o.render.r#unsafe = true;
        // false여야 `<code class="language-rust">`가 나온다. true면 `<pre lang="rust">`다.
        // 전자가 Prism/highlight.js/Pagefind가 기대하는 관례이고, M1에서 syntect가
        // 붙기 전까지의 기본 출력도 이쪽이어야 CSS를 다시 안 쓴다.
        o.render.github_pre_lang = false;

        Self { options: o }
    }

    pub fn to_html(&self, body: &str) -> String {
        markdown_to_html(body, &self.options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn renderer() -> Renderer<'static> {
        Renderer::new(&MarkdownConfig::default())
    }

    #[test]
    fn renders_basic_markdown() {
        let html = renderer().to_html("# Hello\n\nWorld.");
        assert!(html.contains("<h1>Hello</h1>"), "실제: {html}");
        assert!(html.contains("<p>World.</p>"), "실제: {html}");
    }

    #[test]
    fn korean_emphasis_attached_to_particle_parses() {
        // cjk_friendly_emphasis가 꺼져 있으면 이게 <em>으로 파싱되지 않는다.
        // 한국어 문서에서 매우 흔한 형태라 회귀하면 바로 알아야 한다.
        let html = renderer().to_html("**정적 사이트 생성기**를 만들었다.");
        assert!(
            html.contains("<strong>정적 사이트 생성기</strong>"),
            "실제: {html}"
        );
    }

    #[test]
    fn code_fence_carries_language_class() {
        let html = renderer().to_html("```rust\nfn main() {}\n```");
        assert!(html.contains("language-rust"), "실제: {html}");
    }

    #[test]
    fn gfm_table_renders() {
        let html = renderer().to_html("| a | b |\n|---|---|\n| 1 | 2 |");
        assert!(html.contains("<table>"), "실제: {html}");
    }

    #[test]
    fn raw_html_passes_through() {
        let html = renderer().to_html("<details><summary>x</summary>y</details>");
        assert!(html.contains("<details>"), "실제: {html}");
    }
}
