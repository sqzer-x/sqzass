//! `sqzass init` — 새 사이트 스캐폴드.
//!
//! 파일 셋만 만든다. 문서의 "첫 사이트 만들기"가 손으로 만들라고 적어 둔 바로 그
//! 셋이고, 그래야 튜토리얼과 도구가 같은 것을 말한다.
//!
//! 예제 콘텐츠를 잔뜩 깔지 않는 이유는, 스캐폴드가 크면 처음 하는 일이 "만드는 것"이
//! 아니라 "지우는 것"이 되기 때문이다.

use anyhow::{Context, Result, bail};
use std::path::Path;

/// 스캐폴드가 만드는 파일들. `(경로, 내용)` — `{title}`만 치환한다.
const FILES: &[(&str, &str)] = &[
    (
        "sqzass.toml",
        r#"title    = "{title}"
base_url = "https://example.com"

# 나머지는 전부 기본값이 있다. `sqzass.toml`에 없는 키는 없는 대로 동작한다.
"#,
    ),
    (
        "content/_index.md",
        r#"+++
title = "{title}"
+++

이 파일이 사이트의 첫 페이지입니다. `content/` 아래에 마크다운을 더하면 페이지가
늘어납니다. `about.md`는 `/about/`이 되고, `guide/_index.md`가 있는 디렉터리는
섹션이 됩니다.

```bash
sqzass serve
```
"#,
    ),
    (
        "templates/page.html",
        r#"<!doctype html>
<html lang="{{ page.language }}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{{ page.title }}</title>
{# 코드 강조 스타일시트. 빌드가 테마에서 만들어 준다. #}
{%- if site.highlight_css %}
<link rel="stylesheet" href="{{ site.highlight_css }}">
{%- endif %}
</head>
<body>
{# page.content 는 이미 HTML이라 `| safe`가 필요하다. 나머지 값은 전부
   자동으로 이스케이프된다. #}
{{ page.content | safe }}
</body>
</html>
"#,
    ),
];

/// `dir`에 새 사이트를 만든다.
///
/// 이미 `sqzass.toml`이 있으면 아무것도 하지 않는다 — 남의 사이트를 반쯤 덮어쓴
/// 상태로 만드는 것보다 아무 일도 안 하는 게 낫다.
pub fn init(dir: &Path) -> Result<Vec<String>> {
    let config = dir.join("sqzass.toml");
    if config.exists() {
        bail!(
            "{}에 이미 사이트가 있습니다 (sqzass.toml).\n\
             비어 있는 디렉터리를 지정하거나, 새 디렉터리 이름을 주세요.",
            dir.display()
        );
    }

    // 디렉터리 이름을 제목으로 쓴다. `.`처럼 이름이랄 게 없으면 현재 디렉터리
    // 이름으로 떨어지고, 그것도 없으면 마지막 폴백을 쓴다.
    let title = dir
        .canonicalize()
        .ok()
        .as_deref()
        .and_then(Path::file_name)
        .or_else(|| dir.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "My site".into());

    let mut written = Vec::new();
    for (rel, body) in FILES {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("{}을(를) 만들 수 없습니다", parent.display()))?;
        }
        std::fs::write(&path, body.replace("{title}", &title))
            .with_context(|| format!("{}을(를) 쓸 수 없습니다", path.display()))?;
        written.push((*rel).to_string());
    }

    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!("sqzass-init-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        p
    }

    #[test]
    fn scaffolds_a_buildable_site() {
        let dir = tmp("build");
        let written = init(&dir).unwrap();
        assert_eq!(written.len(), 3, "파일 셋: {written:?}");

        // 스캐폴드가 곧바로 빌드되지 않으면 스캐폴드가 아니다.
        let out = crate::build(&crate::BuildOptions {
            input: dir.clone(),
            output: None,
            drafts: false,
            base_url: None,
        })
        .unwrap();
        assert_eq!(out.pages_written, 1);
        assert!(dir.join("public/index.html").is_file());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn refuses_to_overwrite_an_existing_site() {
        let dir = tmp("existing");
        init(&dir).unwrap();
        let err = init(&dir).unwrap_err().to_string();
        assert!(err.contains("이미 사이트가 있습니다"), "실제: {err}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn title_comes_from_the_directory_name() {
        let dir = tmp("mysite");
        init(&dir).unwrap();
        let cfg = std::fs::read_to_string(dir.join("sqzass.toml")).unwrap();
        assert!(cfg.contains("mysite"), "실제: {cfg}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
