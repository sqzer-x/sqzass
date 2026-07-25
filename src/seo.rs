//! `sitemap.xml`과 `robots.txt`.
//!
//! 둘 다 사이트를 빌드하면 그냥 생긴다. 크롤러가 이름으로 찾는 파일이라 이름이 곧
//! 계약이고, 그래서 콘텐츠 해시를 붙이지 않는다.
//!
//! **`static/`에 같은 이름의 파일이 있으면 그쪽이 이긴다.** 생성물을 덮어쓰는 게
//! 아니라 애초에 만들지 않는다 — 자기 `robots.txt`를 넣었는데 조용히 무시당하는
//! 것만큼 나쁜 것도 없다.

use crate::content::Page;
use crate::site::Site;

pub const SITEMAP_PATH: &str = "sitemap.xml";
pub const ROBOTS_PATH: &str = "robots.txt";

/// `<loc>`과 언어 대체 링크만 낸다.
///
/// **`priority`와 `changefreq`는 넣지 않는다.** 구글은 2023년에 둘 다 무시한다고
/// 명시했고, 무시되는 값을 만드는 코드는 그냥 틀릴 기회일 뿐이다.
///
/// **`lastmod`도 넣지 않는다.** 믿을 만한 출처가 없기 때문이다. 파일 mtime은
/// 체크아웃 시각이라 CI에서는 전부 "방금"이 되고, 빌드가 결정적이어야 한다는
/// 원칙과도 어긋난다. git 커밋 시각은 정확하지만 전체 이력이 있어야 하는데
/// `actions/checkout`은 기본이 얕은 클론이라, 조용히 모든 페이지가 같은 날짜를
/// 갖게 된다. 틀린 `lastmod`는 없는 것보다 나쁘다 — 구글은 부정확한 값을 보면
/// 그 사이트의 `lastmod`를 통째로 무시한다.
pub fn sitemap(pages: &[Page], site: &Site, origin: &str) -> String {
    let mut out = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\" \
         xmlns:xhtml=\"http://www.w3.org/1999/xhtml\">\n",
    );

    for page in pages {
        out.push_str("  <url>\n    <loc>");
        escape_into(&mut out, &format!("{origin}{}", page.url));
        out.push_str("</loc>\n");

        // 번역이 있는 페이지만 대체 링크를 낸다. 혼자인 페이지에 자기 자신만
        // 가리키는 hreflang을 다는 건 바이트 낭비다.
        let langs = site.language_set_of(page);
        if langs.len() > 1 {
            for (code, url) in langs {
                out.push_str("    <xhtml:link rel=\"alternate\" hreflang=\"");
                escape_into(&mut out, &code);
                out.push_str("\" href=\"");
                escape_into(&mut out, &format!("{origin}{url}"));
                out.push_str("\"/>\n");
            }
        }
        out.push_str("  </url>\n");
    }

    out.push_str("</urlset>\n");
    out
}

/// 전부 허용하고 sitemap을 가리킨다.
///
/// 크롤러를 막고 싶으면 `static/robots.txt`에 직접 쓰면 된다.
pub fn robots(origin: &str, base_path: &str) -> String {
    // sitemap은 출력 루트에 있고, 출력 루트는 도메인의 `base_path` 아래에 놓인다.
    format!("User-agent: *\nAllow: {base_path}/\n\nSitemap: {origin}{base_path}/{SITEMAP_PATH}\n")
}

/// XML 텍스트/속성값 이스케이프.
///
/// URL에는 쿼리가 붙을 수 있고 `&`는 XML에서 엔티티의 시작이다. 이걸 빼먹으면
/// sitemap 전체가 파싱되지 않는데, 크롤러는 그 사실을 알려주지 않는다.
fn escape_into(out: &mut String, s: &str) {
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn robots_points_at_the_sitemap() {
        let r = robots("https://example.com", "");
        assert!(
            r.contains("Sitemap: https://example.com/sitemap.xml"),
            "{r}"
        );
    }

    #[test]
    fn escapes_ampersands() {
        let mut s = String::new();
        escape_into(&mut s, "https://example.com/a?b=1&c=2");
        assert_eq!(s, "https://example.com/a?b=1&amp;c=2");
    }

    /// 번역이 있는 페이지에는 hreflang 세트를, 없는 페이지에는 `<loc>`만.
    #[test]
    fn sitemap_lists_every_page_and_its_translations() {
        let cfg: crate::config::Config = toml::from_str(
            r#"
            title = "t"
            base_url = "https://example.com"
            default_language = "en"
            [languages.en]
            name = "English"
            weight = 1
            [languages.ko]
            name = "한국어"
            weight = 2
            "#,
        )
        .unwrap();

        let root = std::env::temp_dir().join(format!("sqzass-seo-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (rel, title) in [
            ("_index.md", "Home"),
            ("_index.ko.md", "홈"),
            ("solo.md", "Solo"),
        ] {
            let path = root.join("content").join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, format!("+++\ntitle = \"{title}\"\n+++\n\nbody\n")).unwrap();
        }
        let pages = crate::content::discover(&root, &cfg, false).unwrap();
        let site = Site::build(&pages, &cfg);
        let xml = sitemap(&pages, &site, "https://example.com");
        let _ = std::fs::remove_dir_all(&root);

        assert_eq!(xml.matches("<loc>").count(), 3, "페이지 3개: {xml}");
        assert!(xml.contains("<loc>https://example.com/</loc>"), "{xml}");
        assert!(xml.contains("<loc>https://example.com/ko/</loc>"), "{xml}");
        assert!(
            xml.contains("<loc>https://example.com/solo/</loc>"),
            "{xml}"
        );

        // 번역이 있는 홈은 자기 자신을 포함해 둘 다 나열해야 한다.
        assert_eq!(xml.matches("hreflang=\"en\"").count(), 2, "{xml}");
        assert_eq!(xml.matches("hreflang=\"ko\"").count(), 2, "{xml}");
        // 혼자인 페이지에는 대체 링크가 없다.
        let solo = xml.split("<url>").find(|u| u.contains("/solo/")).unwrap();
        assert!(!solo.contains("xhtml:link"), "{solo}");

        assert!(!xml.contains("priority"), "구글이 무시하는 값이다: {xml}");
        assert!(!xml.contains("changefreq"), "구글이 무시하는 값이다: {xml}");
        assert!(!xml.contains("lastmod"), "믿을 만한 출처가 없다: {xml}");
    }
}
