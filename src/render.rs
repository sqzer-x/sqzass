//! minijinja 템플릿 환경과 렌더 컨텍스트.

use anyhow::{Context, Result, bail};
use minijinja::{Environment, UndefinedBehavior, Value};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use walkdir::WalkDir;

pub const TEMPLATE_DIR: &str = "templates";
const TEMPLATE_EXTENSIONS: &[&str] = &["html", "xml", "txt", "json", "jinja", "j2"];

pub struct Templates {
    env: Environment<'static>,
    names: Vec<String>,
}

impl Templates {
    /// `templates/` 전체를 **메모리로 한 번에 읽어** 그 스냅샷에서만 템플릿을 해석한다.
    ///
    /// 이게 중요한 이유: minijinja의 로더는 `{% include %}` / `{% extends %}`를
    /// VM 레벨에서 가로챈다. 로더가 디스크를 직접 읽게 두면, 파일을 저장하는 도중에
    /// 리빌드가 걸렸을 때 **반쯤 쓰인 파셜**을 읽어들일 수 있다. 스냅샷에서 해석하면
    /// 한 번의 빌드가 항상 하나의 일관된 템플릿 상태를 본다.
    pub fn load(root: &Path) -> Result<Self> {
        let dir = root.join(TEMPLATE_DIR);
        if !dir.is_dir() {
            bail!("{} 디렉터리가 없습니다", dir.display());
        }

        let mut snapshot: HashMap<String, String> = HashMap::new();
        for entry in WalkDir::new(&dir).sort_by_file_name() {
            let entry = entry.context("templates/ 순회 실패")?;
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let is_template = path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| TEMPLATE_EXTENSIONS.contains(&e));
            if !is_template {
                continue;
            }

            // 템플릿 이름은 확장자를 포함한 상대 경로다: `base.html`, `partials/nav.html`.
            // Jinja 관례를 그대로 따르므로 `{% extends "base.html" %}`가 자연스럽다.
            let name = path
                .strip_prefix(&dir)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            let source = std::fs::read_to_string(path)
                .with_context(|| format!("{}을(를) 읽을 수 없습니다", path.display()))?;
            snapshot.insert(name, source);
        }

        if snapshot.is_empty() {
            bail!("{}에 템플릿이 없습니다", dir.display());
        }

        let mut names: Vec<String> = snapshot.keys().cloned().collect();
        names.sort();

        let snapshot = Arc::new(snapshot);
        let mut env = Environment::new();

        {
            let snapshot = Arc::clone(&snapshot);
            env.set_loader(move |name| Ok(snapshot.get(name).cloned()));
        }

        // autoescape는 minijinja의 확장자 기반 기본 동작을 그대로 쓴다.
        // html/htm/xml → Html, json → Json, txt → None.
        //
        // "확장자가 없으면 AutoEscape::None이 되어 조용히 이스케이프가 꺼진다"는
        // 함정이 있지만, 우리 로더가 TEMPLATE_EXTENSIONS에 있는 파일만 받으므로
        // 확장자 없는 템플릿은 애초에 존재할 수 없다. 이름 없는 참조는 스냅샷에서
        // 못 찾아 명확한 에러가 된다 — 조용한 실패가 아니다.
        //
        // 여기서 None을 Html로 끌어올리면 안 된다. 우리 목록에서 None이 되는 건
        // `.txt`뿐인데, robots.txt / llms.txt를 HTML 이스케이프하면 `&`가 `&amp;`가
        // 되어 파일이 망가진다.
        env.set_auto_escape_callback(minijinja::default_auto_escape_callback);

        // Strict가 아니라 SemiStrict. Strict는 `{% if optional_field %}`까지 에러로
        // 만드는데, front matter의 선택적 필드를 존재 검사하는 게 SSG 템플릿의
        // 일상이라 쓸 수 없다. SemiStrict는 undefined의 출력/순회/속성 접근만 막는다.
        env.set_undefined_behavior(UndefinedBehavior::SemiStrict);

        Ok(Self { env, names })
    }

    pub fn has(&self, name: &str) -> bool {
        self.names.iter().any(|n| n == name)
    }

    pub fn names(&self) -> &[String] {
        &self.names
    }

    pub fn render(&self, name: &str, ctx: Value) -> Result<String> {
        let tmpl = self
            .env
            .get_template(name)
            .with_context(|| format!("템플릿 '{name}'을(를) 찾을 수 없습니다"))?;
        tmpl.render(ctx)
            .with_context(|| format!("템플릿 '{name}' 렌더 실패"))
    }
}

// ⚠️ 알려진 결함 (M1에서 수정): minijinja의 HTML 이스케이퍼는 `/`를 `&#x2f;`로 바꾼다
// (utils.rs의 `b'/' => "&#x2f;"`, OWASP식 보수적 이스케이핑). `v_htmlescape` 피처로
// 바꿔봤지만 그쪽도 `/`를 이스케이프하므로 우회가 안 된다.
//
// 결과: 모든 URL이 `href="https:&#x2f;&#x2f;..."` 로 나간다. 유효한 HTML이고 브라우저와
// 제대로 된 파서는 디코드하지만, 출력물이 지저분하다.
//
// 제대로 된 해법은 URL을 일반 텍스트 이스케이퍼에 태우지 않는 것이다. URL 속성값에
// 필요한 이스케이프는 `& " < >` 넷뿐이다. 컨텍스트의 URL 필드를 String이 아니라
// `Value::from_safe_string(직접_이스케이프한_값)`으로 넘기면 된다 — 다만 그러려면
// 컨텍스트를 Serialize 구조체가 아니라 Value로 조립해야 하므로, 내부 링크를 루트
// 절대화하는 M1 작업과 함께 처리한다.
#[derive(Debug, Clone, Serialize)]
pub struct SiteCtx {
    pub title: String,
    pub description: String,
    pub base_url: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PageCtx {
    pub title: String,
    pub description: String,
    pub url: String,
    pub permalink: String,
    /// 렌더된 HTML. autoescape가 켜져 있으므로 템플릿에서 `| safe`가 필요하다.
    pub content: String,
    pub weight: i64,
    pub draft: bool,
    pub toc: bool,
    pub language: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use minijinja::context;

    /// 테스트가 병렬로 돌아도 겹치지 않도록 이름으로 구분한 임시 사이트 루트를 만든다.
    /// 반환값을 드롭할 때 디렉터리를 지운다.
    struct Site(std::path::PathBuf);

    impl Site {
        fn new(name: &str, files: &[(&str, &str)]) -> Self {
            let root =
                std::env::temp_dir().join(format!("sqzass-test-{}-{name}", std::process::id()));
            let tdir = root.join(TEMPLATE_DIR);
            std::fs::create_dir_all(&tdir).unwrap();
            for (file, body) in files {
                let path = tdir.join(file);
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(path, body).unwrap();
            }
            Self(root)
        }

        fn templates(&self) -> Templates {
            Templates::load(&self.0).unwrap()
        }
    }

    impl Drop for Site {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).ok();
        }
    }

    #[test]
    fn resolves_extends_through_the_snapshot() {
        let site = Site::new(
            "extends",
            &[
                ("base.html", "<main>{% block body %}{% endblock %}</main>"),
                (
                    "page.html",
                    "{% extends \"base.html\" %}{% block body %}{{ page.title }}{% endblock %}",
                ),
            ],
        );
        let out = site
            .templates()
            .render("page.html", context! { page => context!{ title => "Hi" } })
            .unwrap();
        assert_eq!(out, "<main>Hi</main>");
    }

    #[test]
    fn resolves_include_from_a_subdirectory() {
        let site = Site::new(
            "include",
            &[
                ("partials/nav.html", "<nav>{{ site.title }}</nav>"),
                ("page.html", "{% include \"partials/nav.html\" %}"),
            ],
        );
        let out = site
            .templates()
            .render(
                "page.html",
                context! { site => context!{ title => "sqzass" } },
            )
            .unwrap();
        assert_eq!(out, "<nav>sqzass</nav>");
    }

    #[test]
    fn autoescapes_html_by_default() {
        let site = Site::new("escape", &[("page.html", "{{ page.title }}")]);
        let out = site
            .templates()
            .render(
                "page.html",
                context! { page => context!{ title => "<script>" } },
            )
            .unwrap();
        assert!(!out.contains("<script>"), "이스케이프되지 않았다: {out}");
    }

    #[test]
    fn plain_text_templates_are_not_html_escaped() {
        // robots.txt / llms.txt 같은 평문 출력은 이스케이프하면 안 된다.
        // `&`가 `&amp;`가 되면 파일이 망가진다.
        let site = Site::new("txt", &[("robots.txt", "{{ page.title }}")]);
        let out = site
            .templates()
            .render(
                "robots.txt",
                context! { page => context!{ title => "a & b" } },
            )
            .unwrap();
        assert_eq!(out, "a & b");
    }

    #[test]
    fn extensionless_templates_are_never_loaded() {
        // autoescape가 확장자 기반이라 확장자 없는 템플릿은 이스케이프가 꺼진다.
        // 로더가 애초에 안 받으므로 그 상황이 생길 수 없다는 걸 고정해둔다.
        let site = Site::new(
            "noext",
            &[("page.html", "ok"), ("bare", "{{ page.title }}")],
        );
        let t = site.templates();
        assert!(t.has("page.html"));
        assert!(!t.has("bare"), "확장자 없는 파일이 템플릿으로 로드됐다");
    }

    #[test]
    fn safe_filter_lets_rendered_html_through() {
        let site = Site::new("safe", &[("page.html", "{{ page.content | safe }}")]);
        let out = site
            .templates()
            .render(
                "page.html",
                context! { page => context!{ content => "<p>hi</p>" } },
            )
            .unwrap();
        assert_eq!(out, "<p>hi</p>");
    }

    #[test]
    fn undefined_attribute_access_is_an_error() {
        let site = Site::new("undef", &[("page.html", "{{ page.nope.deeper }}")]);
        let err = site
            .templates()
            .render("page.html", context! { page => context!{} });
        assert!(err.is_err(), "undefined 접근이 통과했다");
    }

    #[test]
    fn optional_field_existence_check_still_works() {
        // SemiStrict를 고른 이유. Strict였다면 이게 에러가 된다.
        let site = Site::new(
            "optional",
            &[(
                "page.html",
                "{% if page.missing %}yes{% else %}no{% endif %}",
            )],
        );
        let out = site
            .templates()
            .render("page.html", context! { page => context!{} })
            .unwrap();
        assert_eq!(out, "no");
    }
}
