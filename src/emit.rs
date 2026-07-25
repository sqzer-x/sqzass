//! 페이지 말고도 빌드가 내보내는 것들 — 리다이렉트 스텁, 404, llms.txt.
//!
//! 전부 "호스트가 안 해 주니 우리가 한다"는 같은 이유에서 나온다. GitHub Pages에는
//! 리다이렉트 규칙이 없고, 404 페이지는 이름으로 찾으며, 그 둘 다 빌드가 만들지
//! 않으면 아무도 만들어 주지 않는다.

use crate::content::Page;
use anyhow::{Result, bail};

/// 호스트들이 이름으로 찾는 파일. `/404/index.html`은 아무도 찾지 않는다.
pub const NOT_FOUND_PATH: &str = "404.html";
pub const LLMS_PATH: &str = "llms.txt";

/// `aliases` 항목 하나를 출력 경로로 바꾼다.
///
/// 스킴이 붙었거나 루트 절대 경로가 아니면 에러다. 상대 경로를 허용하면 "무엇에
/// 상대적인가"라는 질문이 생기고, 그 답이 파일 위치인지 URL인지 사람마다 다르게
/// 읽는다.
pub fn alias_output_path(alias: &str, page: &Page) -> Result<String> {
    if alias.contains("://") || alias.starts_with("//") {
        bail!(
            "{}: alias `{alias}` 는 외부 URL입니다. alias는 이 사이트 안의 \
             루트 절대 경로여야 합니다 (예: `/old-name/`).",
            page.source.display()
        );
    }
    if !alias.starts_with('/') {
        bail!(
            "{}: alias `{alias}` 가 `/`로 시작하지 않습니다. \
             alias는 루트 절대 경로여야 합니다 (예: `/old-name/`).",
            page.source.display()
        );
    }

    let trimmed = alias.trim_matches('/');
    if trimmed.is_empty() {
        bail!(
            "{}: alias `{alias}` 는 사이트 루트를 가리킵니다. \
             루트는 `_index.md`가 이미 차지하고 있습니다.",
            page.source.display()
        );
    }
    Ok(format!("{trimmed}/index.html"))
}

/// meta-refresh 리다이렉트 스텁.
///
/// 정적 호스트에서 쓸 수 있는 유일한 수단이다. canonical로 원본을 가리키고
/// `noindex`를 붙여, 검색 엔진이 스텁 자체를 색인하지 않게 한다. 본문에 링크를
/// 두는 건 자동 이동이 막힌 환경(스크린리더 설정, 확장 프로그램)을 위한 것이다.
pub fn redirect_stub(target_url: &str, permalink: &str) -> String {
    format!(
        "<!doctype html>\n\
         <html lang=\"en\">\n\
         <head>\n\
         <meta charset=\"utf-8\">\n\
         <title>Redirecting…</title>\n\
         <link rel=\"canonical\" href=\"{permalink}\">\n\
         <meta name=\"robots\" content=\"noindex\">\n\
         <meta http-equiv=\"refresh\" content=\"0; url={target_url}\">\n\
         </head>\n\
         <body><p>이 페이지는 <a href=\"{target_url}\">{target_url}</a> 로 옮겨졌습니다.</p></body>\n\
         </html>\n"
    )
}

/// `llms.txt` — 언어 모델이 사이트 전체를 훑지 않고도 구조를 알 수 있는 목록.
///
/// [제안 규격](https://llmstxt.org)은 제목, 요약, 그리고 링크 목록이다. 우리는 이미
/// 제목·설명·URL을 전부 갖고 있으므로 새로 만들 데이터가 없다.
pub fn llms_txt(title: &str, description: &str, origin: &str, pages: &[Page]) -> String {
    let mut out = format!("# {title}\n");
    if !description.is_empty() {
        out.push_str(&format!("\n> {description}\n"));
    }
    out.push_str("\n## Pages\n\n");
    for page in pages {
        out.push_str(&format!("- [{}]({origin}{})", page.title, page.url));
        if !page.front.description.is_empty() {
            out.push_str(&format!(": {}", page.front.description));
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page() -> Page {
        Page {
            source: "content/new.md".into(),
            rel: "new.md".into(),
            front: crate::content::FrontMatter::default(),
            title: "New".into(),
            body: String::new(),
            body_line_offset: 1,
            language: "en".into(),
            translation_key: "new".into(),
            url: "/new/".into(),
            out_path: "new/index.html".into(),
            is_section: false,
        }
    }

    #[test]
    fn alias_becomes_a_directory_index() {
        assert_eq!(
            alias_output_path("/old/", &page()).unwrap(),
            "old/index.html"
        );
        assert_eq!(
            alias_output_path("/a/b", &page()).unwrap(),
            "a/b/index.html"
        );
    }

    #[test]
    fn alias_must_be_a_local_root_absolute_path() {
        for bad in ["https://elsewhere.example/x", "//cdn.example/x"] {
            let err = alias_output_path(bad, &page()).unwrap_err().to_string();
            assert!(err.contains("외부 URL"), "{bad}: {err}");
        }
        let err = alias_output_path("old/", &page()).unwrap_err().to_string();
        assert!(err.contains("`/`로 시작"), "{err}");
        let err = alias_output_path("/", &page()).unwrap_err().to_string();
        assert!(err.contains("루트"), "{err}");
    }

    /// 스텁이 색인되면 검색 결과에 옛 URL이 뜨고, 원본과 중복 콘텐츠가 된다.
    #[test]
    fn the_stub_points_home_and_refuses_indexing() {
        let html = redirect_stub("/new/", "https://example.com/new/");
        assert!(html.contains(r#"content="noindex""#), "{html}");
        assert!(
            html.contains(r#"rel="canonical" href="https://example.com/new/""#),
            "{html}"
        );
        assert!(html.contains(r#"content="0; url=/new/""#), "{html}");
        // 자동 이동이 막힌 환경을 위해 본문에도 링크가 있어야 한다.
        assert!(html.contains(r#"<a href="/new/""#), "{html}");
    }

    #[test]
    fn llms_txt_lists_every_page_with_an_absolute_url() {
        let mut p = page();
        p.front.description = "A page".into();
        let txt = llms_txt("Site", "About it", "https://example.com", &[p]);
        assert!(txt.starts_with("# Site\n"), "{txt}");
        assert!(txt.contains("> About it"), "{txt}");
        assert!(
            txt.contains("- [New](https://example.com/new/): A page"),
            "{txt}"
        );
    }
}
