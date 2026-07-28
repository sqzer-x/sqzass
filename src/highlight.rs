//! 빌드 타임 구문 강조.
//!
//! **인라인 `style=`을 쓰지 않는다.** syntect의 편의 함수(`highlighted_html_for_string`)는
//! 색을 HTML에 직접 박는데, 그러면 테마 하나가 문서에 영구히 고정되어 다크 모드가
//! 불가능해지고, 페이지마다 색 문자열이 중복되며, CSP의 `style-src` 강화도 막힌다.
//! 대신 의미론적 CSS 클래스를 뽑고 테마는 스타일시트로 분리한다 — 그래서 같은 HTML이
//! 라이트/다크 양쪽에서 동작한다.

use crate::config::Highlight as HighlightConfig;
use anyhow::{Context, Result};
use comrak::adapters::SyntaxHighlighterAdapter;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Mutex, OnceLock};
use syntect::highlighting::ThemeSet;
use syntect::html::{
    ClassStyle, ClassedHTMLGenerator, css_for_theme_with_class_style, line_tokens_to_classed_spans,
};
use syntect::parsing::{ParseState, Scope, ScopeStack, SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

/// 클래스 접두사. 접두사가 없으면 syntect가 `source`, `keyword`, `string` 같은
/// 일반적인 단어를 그대로 클래스로 쓰기 때문에 사이트 CSS와 충돌한다.
/// syntect API가 `&'static str`을 요구하므로 상수여야 한다.
const CLASS_PREFIX: &str = "hl-";
const CLASS_STYLE: ClassStyle = ClassStyle::SpacedPrefixed {
    prefix: CLASS_PREFIX,
};

/// 문법 덤프는 프로세스당 한 번만 역직렬화한다.
///
/// two-face의 bincode 덤프 역직렬화는 수십 ms급 고정비다. 단일 `build`에서는 어차피
/// 한 번이지만, `serve`는 저장할 때마다 빌드를 새로 돌므로 이게 저장→반영 지연에
/// 매번 얹혔다. 문법 집합은 콘텐츠와 무관하니 재사용해도 결정성에 영향이 없다.
static SYNTAXES: OnceLock<SyntaxSet> = OnceLock::new();

pub struct Highlighter {
    syntaxes: &'static SyntaxSet,
    /// 파싱된 펜스 옵션을 `write_code_tag` → `write_highlighted` 사이에 건네는
    /// 보관함. comrak은 한 블록을 pre → code → highlighted 순서로 부르고 렌더는
    /// 단일 스레드다. 병렬 렌더를 도입하면 이 상태는 렌더 단위별로 분리해야 한다.
    pending: Mutex<FenceOpts>,
    /// 잘못된 펜스 옵션. 렌더가 끝난 뒤 빌드를 실패시키는 데 쓴다 — 모르는 키를
    /// 조용히 무시하면 `hl_line=` 오타가 강조 없는 채로 배포된다.
    errors: Mutex<Vec<String>>,
}

impl Highlighter {
    pub fn new() -> Self {
        // `extra_newlines` 여야 한다. ClassedHTMLGenerator는 줄 끝의 개행을 요구하고,
        // no-newlines 덤프와 섞으면 조용히 잘못된 결과가 나온다.
        Self {
            syntaxes: SYNTAXES.get_or_init(two_face::syntax::extra_newlines),
            pending: Mutex::default(),
            errors: Mutex::default(),
        }
    }

    /// 잘못된 펜스 옵션들. 비어 있지 않으면 빌드를 멈춰야 한다.
    pub fn take_errors(&self) -> Vec<String> {
        self.errors
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .split_off(0)
    }

    fn record_error(&self, message: String) {
        self.errors
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(message);
    }
}

/// 코드 펜스 옵션: ```` ```rust hl_lines=2-4,7 name=main.rs ```` 처럼 **언어 뒤에
/// 공백으로 구분한 key=value**로 적는다.
///
/// 이 문법을 고른 근거: comrak이 info 문자열을 첫 공백에서 언어/메타로 가르므로
/// 공백 뒤 key=value가 자연스럽고, Zola식 `rust,hl_lines=2-4`는 콤마까지 언어
/// 토큰에 붙어 문법 조회와 `language-` 클래스를 오염시킨다. hwaro식 `{}`는 같은
/// 위치에서 괄호만 더한다. 값에 공백은 못 쓴다 — 필요해지면 따옴표를 그때 더한다.
#[derive(Debug, Default, PartialEq)]
struct FenceOpts {
    /// 강조할 줄들, 1부터 시작하는 닫힌 구간.
    hl_lines: Vec<(usize, usize)>,
    /// 파일명 라벨. `data-name` 속성으로만 나가고, 보여줄지는 사이트 CSS가 정한다.
    name: Option<String>,
}

impl FenceOpts {
    fn parse(meta: &str) -> Result<Self, String> {
        let mut opts = Self::default();
        for token in meta.split_whitespace() {
            let Some((key, value)) = token.split_once('=') else {
                return Err(format!(
                    "펜스 옵션 `{token}`: key=value 꼴이어야 합니다 (예: hl_lines=2-4,7 name=main.rs)"
                ));
            };
            match key {
                // 같은 키가 두 번 오면 앞의 것이 소리 없이 버려진다 — 편집
                // 잔재가 살아남는 정확히 그 경로라서 에러다.
                "hl_lines" if !opts.hl_lines.is_empty() => {
                    return Err(format!("펜스 옵션 `{key}` 가 두 번 왔습니다"));
                }
                "name" if opts.name.is_some() => {
                    return Err(format!("펜스 옵션 `{key}` 가 두 번 왔습니다"));
                }
                "hl_lines" => opts.hl_lines = parse_ranges(value)?,
                "name" => opts.name = Some(value.to_string()),
                _ => {
                    return Err(format!(
                        "펜스 옵션 `{key}` 는 없습니다. 사용 가능: hl_lines, name"
                    ));
                }
            }
        }
        Ok(opts)
    }
}

/// `2-4,7` → `[(2,4), (7,7)]`
fn parse_ranges(value: &str) -> Result<Vec<(usize, usize)>, String> {
    let mut out = Vec::new();
    for part in value.split(',') {
        let (a, b) = part.split_once('-').unwrap_or((part, part));
        let (a, b) = (parse_line_no(a, part)?, parse_line_no(b, part)?);
        if a > b {
            return Err(format!("hl_lines `{part}`: 시작이 끝보다 큽니다"));
        }
        out.push((a, b));
    }
    Ok(out)
}

fn parse_line_no(s: &str, part: &str) -> Result<usize, String> {
    match s.parse::<usize>() {
        Ok(n) if n >= 1 => Ok(n),
        _ => Err(format!(
            "hl_lines `{part}`: 줄 번호는 1부터 시작하는 정수여야 합니다"
        )),
    }
}

/// 줄 시작 시점에 열려 있는 스코프를 그 줄 안에서 다시 연다.
///
/// syntect의 `scope_to_classes`는 비공개라, 공개 API인 `Scope::build_string()`의
/// atom들로 같은 클래스 목록을 재구성한다. atom에는 `.`이 들어갈 수 없으므로
/// split이 정확히 역연산이다. 재구성이 어긋나면 아래 여러 줄 주석 테스트가 깨진다.
fn open_scope_span(out: &mut String, scope: Scope) {
    out.push_str("<span class=\"");
    let dotted = scope.build_string();
    for (i, atom) in dotted.split('.').enumerate() {
        if i != 0 {
            out.push(' ');
        }
        out.push_str(CLASS_PREFIX);
        out.push_str(atom);
    }
    out.push_str("\">");
}

impl Default for Highlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl Highlighter {
    /// 옵션 없는 펜스의 경로. 기존 출력과 바이트 단위로 같아야 한다.
    fn write_block(
        &self,
        out: &mut dyn fmt::Write,
        syntax: &SyntaxReference,
        code: &str,
    ) -> fmt::Result {
        let mut generator =
            ClassedHTMLGenerator::new_with_class_style(syntax, self.syntaxes, CLASS_STYLE);
        for line in LinesWithEndings::from(code) {
            // 강조에 실패해도 문서 빌드를 통째로 죽이지 않는다. 이 블록만 포기하고
            // 이스케이프된 원문을 내보낸다.
            if generator
                .parse_html_for_line_which_includes_newline(line)
                .is_err()
            {
                return write_plain(out, code);
            }
        }
        out.write_str(&generator.finalize())
    }

    /// `hl_lines`가 있을 때의 경로. 표시할 줄을 `<mark>`로 감싸려면 각 줄이 span
    /// 균형이 맞아야 하는데, `ClassedHTMLGenerator`는 span을 줄 경계 너머로 열어
    /// 둔다(마지막 `finalize`에서만 닫는다). 그래서 파스 상태는 줄을 넘겨 잇되
    /// (여러 줄 주석·문자열이 올바르게 칠해진다), 각 줄은 시작에 현재 열려 있는
    /// 스코프를 다시 열고 끝에서 전부 닫아 자립시킨다.
    ///
    /// `<mark>`인 이유: CSS 없이도 브라우저 기본 스타일로 표시가 보이고, 사이트는
    /// `.highlight mark`로 덮어쓰면 된다.
    fn write_lines(
        &self,
        out: &mut dyn fmt::Write,
        syntax: &SyntaxReference,
        code: &str,
        hl: &[(usize, usize)],
    ) -> fmt::Result {
        let total = LinesWithEndings::from(code).count();
        if let Some(&(a, b)) = hl.iter().find(|&&(_, b)| b > total) {
            self.record_error(format!(
                "hl_lines {a}-{b}: 이 코드 블록은 {total}줄뿐입니다"
            ));
            return self.write_block(out, syntax, code);
        }

        let mut parse_state = ParseState::new(syntax);
        let mut stack = ScopeStack::new();
        let mut html = String::new();
        for (i, line) in LinesWithEndings::from(code).enumerate() {
            let n = i + 1;
            let Ok(ops) = parse_state.parse_line(line, self.syntaxes) else {
                return write_plain(out, code);
            };
            let marked = hl.iter().any(|&(a, b)| (a..=b).contains(&n));
            if marked {
                html.push_str("<mark class=\"hl-line\">");
            }
            for &scope in stack.as_slice() {
                open_scope_span(&mut html, scope);
            }
            let Ok((body, _)) = line_tokens_to_classed_spans(line, &ops, CLASS_STYLE, &mut stack)
            else {
                return write_plain(out, code);
            };
            html.push_str(&body);
            // 이 줄에서 열린 span 수 = 줄 시작에 다시 연 것 + Push − Pop = 스택 크기.
            for _ in 0..stack.len() {
                html.push_str("</span>");
            }
            if marked {
                html.push_str("</mark>");
            }
        }
        out.write_str(&html)
    }
}

impl SyntaxHighlighterAdapter for Highlighter {
    fn write_highlighted(
        &self,
        out: &mut dyn fmt::Write,
        lang: Option<&str>,
        code: &str,
    ) -> fmt::Result {
        let opts = std::mem::take(&mut *self.pending.lock().unwrap_or_else(|e| e.into_inner()));
        let syntax = lang
            .and_then(|l| {
                let l = l.trim();
                self.syntaxes
                    .find_syntax_by_token(l)
                    .or_else(|| self.syntaxes.find_syntax_by_extension(l))
            })
            .unwrap_or_else(|| self.syntaxes.find_syntax_plain_text());

        if opts.hl_lines.is_empty() {
            self.write_block(out, syntax, code)
        } else {
            self.write_lines(out, syntax, code, &opts.hl_lines)
        }
    }

    fn write_pre_tag(
        &self,
        out: &mut dyn fmt::Write,
        _attributes: HashMap<&'static str, Cow<'_, str>>,
    ) -> fmt::Result {
        out.write_str("<pre class=\"highlight\">")
    }

    fn write_code_tag(
        &self,
        out: &mut dyn fmt::Write,
        attributes: HashMap<&'static str, Cow<'_, str>>,
    ) -> fmt::Result {
        // comrak이 `class="language-rust"`를 넘겨준다. 언어를 data 속성으로도 복제해
        // 복사 버튼이나 라벨이 클래스 문자열을 파싱하지 않아도 되게 한다.
        let class = attributes.get("class").map(|c| c.as_ref()).unwrap_or("");
        let lang = class.strip_prefix("language-").unwrap_or("");

        // 언어 이름에 `=`는 없다 — 언어 자리의 `=`는 언어를 빼먹고 옵션부터
        // 적었다는 확실한 증거다. comrak이 첫 토큰을 언어로 삼아 버려 아래
        // 옵션 파서에는 도달하지 않으므로, 여기서 잡지 않으면 옵션이 조용히
        // 증발한 페이지가 배포된다.
        if lang.contains('=') {
            self.record_error(format!(
                "펜스 언어 자리에 `{lang}`: 옵션은 언어 뒤에 옵니다 (언어가 없으면 ```text {lang})"
            ));
        }

        // 펜스 옵션은 comrak이 `data-meta`로 넘겨준다(언어 뒤 공백 이후 전부,
        // `full_info_string`). 여기서 파싱해 두면 바로 다음의 write_highlighted가
        // 꺼내 쓴다. 잘못된 옵션은 기록만 하고 계속 간다 — 렌더가 끝난 뒤 빌드가
        // 페이지 경로와 함께 실패한다.
        let meta = attributes
            .get("data-meta")
            .map(|m| m.as_ref())
            .unwrap_or("");
        let opts = if meta.is_empty() {
            FenceOpts::default()
        } else {
            match FenceOpts::parse(meta) {
                Ok(o) => o,
                Err(e) => {
                    self.record_error(e);
                    FenceOpts::default()
                }
            }
        };

        if lang.is_empty() {
            *self.pending.lock().unwrap_or_else(|e| e.into_inner()) = opts;
            return out.write_str("<code>");
        }
        write!(
            out,
            "<code class=\"{}\" data-lang=\"{}\"",
            escape_attr(class),
            escape_attr(lang)
        )?;
        if let Some(name) = &opts.name {
            write!(out, " data-name=\"{}\"", escape_attr(name))?;
        }
        *self.pending.lock().unwrap_or_else(|e| e.into_inner()) = opts;
        out.write_str(">")
    }
}

fn write_plain(out: &mut dyn fmt::Write, code: &str) -> fmt::Result {
    for ch in code.chars() {
        match ch {
            '&' => out.write_str("&amp;")?,
            '<' => out.write_str("&lt;")?,
            '>' => out.write_str("&gt;")?,
            _ => out.write_char(ch)?,
        }
    }
    Ok(())
}

fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// 라이트/다크 두 테마의 CSS를 하나의 스타일시트로 합친다.
///
/// 다크 규칙을 두 번 내보낸다: OS 설정을 따르는 `prefers-color-scheme` 안에 한 번,
/// 수동 토글용 `[data-theme="dark"]`에 한 번. 네이티브 CSS 중첩을 써서 생성된
/// 규칙을 문자열로 파싱하지 않고 그대로 감싼다.
pub fn stylesheet(cfg: &HighlightConfig) -> Result<String> {
    // 테마 집합도 문법 덤프와 같은 이유로 프로세스당 한 번만 로드한다 —
    // serve의 리빌드마다 이 함수가 다시 불린다.
    static THEMES: OnceLock<ThemeSet> = OnceLock::new();
    let themes = THEMES.get_or_init(ThemeSet::load_defaults);

    let get = |name: &str| -> Result<String> {
        let theme = themes.themes.get(name).with_context(|| {
            format!(
                "'{name}' 테마를 찾을 수 없습니다. 사용 가능: {}",
                themes.themes.keys().cloned().collect::<Vec<_>>().join(", ")
            )
        })?;
        css_for_theme_with_class_style(theme, CLASS_STYLE)
            .with_context(|| format!("'{name}' 테마의 CSS 생성 실패"))
    };

    let light = get(&cfg.theme_light)?;
    let dark = get(&cfg.theme_dark)?;

    Ok(format!(
        "/* generated by sqzass — do not edit */\n\
         /* light: {} */\n{}\n\
         /* dark: {} */\n\
         @media (prefers-color-scheme: dark) {{\n  :root:not([data-theme=\"light\"]) {{\n{}\n  }}\n}}\n\
         :root[data-theme=\"dark\"] {{\n{}\n}}\n",
        cfg.theme_light, light, cfg.theme_dark, dark, dark
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Markdown as MarkdownConfig;
    use crate::markdown::Renderer;

    fn render(md: &str) -> String {
        Renderer::new(&MarkdownConfig::default())
            .with_highlighter(Highlighter::new())
            .render(md)
            .html
    }

    #[test]
    fn emits_css_classes_not_inline_styles() {
        let html = render("```rust\nfn main() { let x = 1; }\n```");
        assert!(
            !html.contains("style=\""),
            "인라인 style이 나왔다 — 다크모드가 불가능해진다: {html}"
        );
        assert!(html.contains("hl-"), "하이라이트 클래스가 없다: {html}");
    }

    #[test]
    fn tags_carry_language_metadata() {
        let html = render("```rust\nfn main() {}\n```");
        assert!(html.contains(r#"class="highlight""#), "실제: {html}");
        assert!(html.contains(r#"data-lang="rust""#), "실제: {html}");
    }

    #[test]
    fn unknown_language_falls_back_to_plain_text() {
        let html = render("```notalanguage\nhello\n```");
        assert!(html.contains("hello"), "본문이 사라졌다: {html}");
    }

    #[test]
    fn fence_without_language_still_renders() {
        let html = render("```\nplain code\n```");
        assert!(html.contains("plain code"), "실제: {html}");
    }

    #[test]
    fn code_is_escaped_not_executed() {
        let html = render("```html\n<script>alert(1)</script>\n```");
        assert!(
            !html.contains("<script>alert"),
            "코드 블록 안의 스크립트가 이스케이프되지 않았다: {html}"
        );
    }

    #[test]
    fn hl_lines_wraps_only_the_marked_lines() {
        let html = render("```rust hl_lines=2\nlet a = 1;\nlet b = 2;\nlet c = 3;\n```");
        assert_eq!(
            html.matches("<mark class=\"hl-line\">").count(),
            1,
            "실제: {html}"
        );
        let marked = html
            .split("<mark class=\"hl-line\">")
            .nth(1)
            .unwrap()
            .split("</mark>")
            .next()
            .unwrap();
        // 문자로 판정하면 클래스명(hl-source, hl-constant)의 글자에 걸린다 —
        // 클래스명에 없는 숫자 리터럴로 어느 줄이 감싸였는지 본다.
        assert!(marked.contains('2'), "2번 줄이 아니다: {marked}");
        assert!(!marked.contains('1'), "1번 줄까지 감쌌다: {marked}");
        assert!(!marked.contains('3'), "3번 줄까지 감쌌다: {marked}");
    }

    #[test]
    fn hl_lines_reopens_scopes_across_line_boundaries() {
        // 여러 줄 주석의 가운데 줄만 표시해도 주석 스코프가 그 줄 안에서 다시
        // 열려야 한다. syntect의 scope_to_classes가 비공개라 Scope::build_string()
        // 으로 재구성하는데, 그 재구성이 어긋나면 이 테스트가 깨진다.
        let html = render("```rust hl_lines=2\n/* first\nsecond\nthird */\n```");
        let marked = html
            .split("<mark class=\"hl-line\">")
            .nth(1)
            .unwrap()
            .split("</mark>")
            .next()
            .unwrap();
        assert!(marked.contains("second"), "실제: {marked}");
        assert!(
            marked.contains("hl-comment"),
            "주석 스코프가 안 열렸다: {marked}"
        );
        assert_eq!(
            marked.matches("<span").count(),
            marked.matches("</span>").count(),
            "표시 줄의 span이 균형이 안 맞는다: {marked}"
        );
    }

    #[test]
    fn name_lands_as_an_escaped_attribute() {
        let html = render("```rust name=main.rs\nfn main() {}\n```");
        assert!(html.contains(r#"data-name="main.rs""#), "실제: {html}");
    }

    #[test]
    fn a_fence_without_options_is_byte_identical_to_before() {
        let html = render("```rust\nlet x = 1;\n```");
        assert!(!html.contains("<mark"), "옵션 없는 펜스에 mark: {html}");
        assert!(!html.contains("data-name"), "실제: {html}");
        assert!(
            !html.contains("data-meta"),
            "메타 속성이 새어 나왔다: {html}"
        );
    }

    #[test]
    fn unknown_fence_option_is_an_error_not_a_silent_noop() {
        let r = Renderer::new(&MarkdownConfig::default()).with_highlighter(Highlighter::new());
        let rendered = r.render("```rust linenos=true\nfn main() {}\n```");
        assert_eq!(
            rendered.bad_fences.len(),
            1,
            "실제: {:?}",
            rendered.bad_fences
        );
        assert!(rendered.bad_fences[0].contains("linenos"));
    }

    #[test]
    fn hl_lines_beyond_the_block_is_an_error() {
        let r = Renderer::new(&MarkdownConfig::default()).with_highlighter(Highlighter::new());
        let rendered = r.render("```rust hl_lines=9\nfn main() {}\n```");
        assert_eq!(
            rendered.bad_fences.len(),
            1,
            "실제: {:?}",
            rendered.bad_fences
        );
    }

    #[test]
    fn options_in_the_language_slot_are_an_error() {
        // 언어를 빼먹으면 comrak이 첫 토큰을 언어로 삼아 옵션 파서에 도달하지
        // 않는다. 그대로 두면 "오타는 빌드 에러" 계약이 이 입력에서만 샌다.
        let r = Renderer::new(&MarkdownConfig::default()).with_highlighter(Highlighter::new());
        let rendered = r.render("```hl_lines=2\nlet a = 1;\nlet b = 2;\n```");
        assert_eq!(
            rendered.bad_fences.len(),
            1,
            "실제: {:?}",
            rendered.bad_fences
        );
        assert!(rendered.bad_fences[0].contains("hl_lines=2"));

        // 반쪽 적용 변형: hl_lines=2가 언어로 먹히고 name만 파싱되는 경우도
        // 같은 에러여야 한다 — data-name이 붙은 출력이 성공처럼 보이기 때문.
        let rendered = r.render("```hl_lines=2 name=a.rs\nlet a = 1;\nlet b = 2;\n```");
        assert!(!rendered.bad_fences.is_empty());
    }

    #[test]
    fn a_repeated_fence_option_is_an_error_not_last_wins() {
        let r = Renderer::new(&MarkdownConfig::default()).with_highlighter(Highlighter::new());
        let rendered = r.render("```rust hl_lines=2 hl_lines=1\nlet a = 1;\nlet b = 2;\n```");
        assert_eq!(
            rendered.bad_fences.len(),
            1,
            "실제: {:?}",
            rendered.bad_fences
        );
        assert!(rendered.bad_fences[0].contains("두 번"));
    }

    #[test]
    fn fence_option_parser_accepts_ranges_and_rejects_nonsense() {
        assert_eq!(
            FenceOpts::parse("hl_lines=2-4,7").unwrap().hl_lines,
            vec![(2, 4), (7, 7)]
        );
        assert!(FenceOpts::parse("hl_lines=0").is_err(), "0번 줄은 없다");
        assert!(FenceOpts::parse("hl_lines=4-2").is_err(), "역순 범위");
        assert!(FenceOpts::parse("no-equals-sign").is_err());
    }

    #[test]
    fn two_face_provides_syntaxes_syntect_lacks() {
        // two-face를 넣은 이유가 이것이다 — 개발 도구 문서에 반드시 필요한 언어들이
        // syntect 기본 세트에 없다.
        let h = Highlighter::new();
        for lang in ["toml", "typescript", "dockerfile"] {
            assert!(
                h.syntaxes.find_syntax_by_token(lang).is_some(),
                "'{lang}' 문법이 없다"
            );
        }
    }

    #[test]
    fn stylesheet_contains_both_themes_and_a_manual_toggle() {
        let css = stylesheet(&HighlightConfig::default()).unwrap();
        assert!(
            css.contains("prefers-color-scheme: dark"),
            "OS 다크 대응이 없다"
        );
        assert!(css.contains("[data-theme=\"dark\"]"), "수동 토글 훅이 없다");
        assert!(css.contains(".hl-"), "접두사 붙은 클래스가 없다");
    }

    #[test]
    fn stylesheet_reports_unknown_theme_names() {
        let cfg = HighlightConfig {
            theme_dark: "NoSuchTheme".into(),
            ..HighlightConfig::default()
        };
        let err = stylesheet(&cfg).unwrap_err().to_string();
        assert!(err.contains("NoSuchTheme"), "실제: {err}");
    }
}
