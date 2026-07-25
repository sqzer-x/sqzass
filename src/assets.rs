//! 정적 에셋 수집과 콘텐츠 해시 기반 파일명.
//!
//! 파일명에 해시를 넣는다. 쿼리 문자열(`?v=…`) 방식은 두 가지가 나쁘다: 일부 CDN과
//! 프록시가 쿼리를 캐시 키에서 빼버리고, 실무에서 흔히 보이는 "빌드마다 하나의 스탬프를
//! 전 파일에 붙이는" 형태는 CSS 한 줄만 고쳐도 나머지 전부를 무효화한다.
//! 파일별 콘텐츠 해시는 바뀐 파일만 무효화되고, 모든 호스트에서 동일하게 동작한다.

use crate::config::Assets as AssetsConfig;
use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::path::Path;
use walkdir::WalkDir;

/// 해시를 붙일 확장자. 이미지·폰트는 대개 마크다운이나 CSS에서 문자열 경로로 직접
/// 참조되므로 이름을 바꾸면 그 참조가 깨진다. `CNAME`처럼 이름 자체가 계약인 파일도
/// 있어서, 해시는 `asset()`으로 조회하는 것들에만 적용한다.
const FINGERPRINTED: &[&str] = &["css", "js"];

const MANIFEST_PATH: &str = "asset-manifest.json";

#[derive(Debug, Default)]
pub struct Assets {
    /// 출력 경로(상대) → 내용
    pub files: BTreeMap<String, Vec<u8>>,
    /// 논리 경로(`css/main.css`) → 출력 URL(`/css/main.a1b2c3d4.css`)
    pub manifest: BTreeMap<String, String>,
    /// 사이트가 도메인 루트가 아닐 때의 접두사. 페이지 URL과 같은 규칙을 따라야
    /// `<link href>`와 내비게이션 링크가 같은 곳을 가리킨다.
    base_path: String,
}

impl Assets {
    /// `static/` 아래를 전부 읽어들인다.
    pub fn collect(root: &Path, cfg: &AssetsConfig, base_path: &str) -> Result<Self> {
        let mut out = Self {
            base_path: base_path.to_string(),
            ..Self::default()
        };
        let dir = root.join(&cfg.source_dir);
        if !dir.is_dir() {
            return Ok(out);
        }

        for entry in WalkDir::new(&dir).sort_by_file_name() {
            let entry = entry.with_context(|| format!("{} 순회 실패", dir.display()))?;
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let logical = path
                .strip_prefix(&dir)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            let bytes = std::fs::read(path)
                .with_context(|| format!("{}을(를) 읽을 수 없습니다", path.display()))?;
            out.insert(&logical, bytes, cfg.fingerprint);
        }
        Ok(out)
    }

    /// 빌드가 생성한 에셋(예: 하이라이트 스타일시트)을 같은 규칙으로 넣는다.
    pub fn insert(&mut self, logical: &str, bytes: Vec<u8>, fingerprint: bool) {
        let ext = Path::new(logical)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let out_path = if fingerprint && FINGERPRINTED.contains(&ext) {
            hashed_name(logical, &bytes)
        } else {
            logical.to_string()
        };
        let base = &self.base_path;
        self.manifest
            .insert(logical.to_string(), format!("{base}/{out_path}"));
        self.files.insert(out_path, bytes);
    }

    /// 논리 경로로 출력 URL을 찾는다.
    pub fn url(&self, logical: &str) -> Option<&str> {
        self.manifest
            .get(logical.trim_start_matches('/'))
            .map(|s| s.as_str())
    }

    /// 매니페스트를 출력에 포함시킨다. 서비스 워커나 외부 도구가 논리 이름과
    /// 실제 파일을 맞출 수 있어야 한다.
    pub fn write_manifest(&mut self) {
        let json = serde_json::to_string_pretty(&self.manifest).unwrap_or_else(|_| "{}".into());
        self.files
            .insert(MANIFEST_PATH.to_string(), json.into_bytes());
    }
}

/// `css/main.css` + 내용 → `css/main.a1b2c3d4.css`
///
/// blake3를 쓴다. 캐시 무효화가 목적이라 암호학적 강도는 필요 없고, 속도가 중요하다.
fn hashed_name(logical: &str, bytes: &[u8]) -> String {
    let hash = blake3::hash(bytes);
    let short = &hash.to_hex()[..8];

    let path = Path::new(logical);
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or(logical);
    let parent = path.parent().map(|p| p.to_string_lossy().into_owned());

    let name = if ext.is_empty() {
        format!("{stem}.{short}")
    } else {
        format!("{stem}.{short}.{ext}")
    };
    match parent.as_deref() {
        Some("") | None => name,
        Some(p) => format!("{p}/{name}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> AssetsConfig {
        AssetsConfig::default()
    }

    #[test]
    fn hashes_go_before_the_extension() {
        let n = hashed_name("css/main.css", b"body{}");
        assert!(n.starts_with("css/main."), "실제: {n}");
        assert!(n.ends_with(".css"), "실제: {n}");
        // css/main.<8자>.css
        assert_eq!(n.matches('.').count(), 2, "실제: {n}");
    }

    #[test]
    fn different_content_gives_a_different_name() {
        assert_ne!(
            hashed_name("a.css", b"one"),
            hashed_name("a.css", b"two"),
            "내용이 다른데 이름이 같으면 캐시가 안 바뀐다"
        );
    }

    #[test]
    fn same_content_is_stable_across_builds() {
        // 빌드마다 이름이 달라지면 재현 가능한 빌드가 깨진다.
        assert_eq!(hashed_name("a.css", b"same"), hashed_name("a.css", b"same"));
    }

    #[test]
    fn root_level_files_keep_a_flat_name() {
        let n = hashed_name("main.css", b"x");
        assert!(!n.contains('/'), "실제: {n}");
    }

    #[test]
    fn only_css_and_js_are_fingerprinted() {
        let mut a = Assets::default();
        a.insert("css/main.css", b"body{}".to_vec(), true);
        a.insert("js/app.js", b"//x".to_vec(), true);
        a.insert("images/logo.png", b"\x89PNG".to_vec(), true);
        // 이름 자체가 계약인 파일들 — 해시가 붙으면 기능이 깨진다
        a.insert("CNAME", b"example.com".to_vec(), true);
        a.insert("robots.txt", b"User-agent: *".to_vec(), true);

        assert_ne!(a.url("css/main.css"), Some("/css/main.css"));
        assert_ne!(a.url("js/app.js"), Some("/js/app.js"));
        assert_eq!(a.url("images/logo.png"), Some("/images/logo.png"));
        assert_eq!(a.url("CNAME"), Some("/CNAME"));
        assert_eq!(a.url("robots.txt"), Some("/robots.txt"));
    }

    #[test]
    fn fingerprinting_can_be_disabled() {
        let mut a = Assets::default();
        a.insert("css/main.css", b"body{}".to_vec(), false);
        assert_eq!(a.url("css/main.css"), Some("/css/main.css"));
    }

    #[test]
    fn lookup_tolerates_a_leading_slash() {
        let mut a = Assets::default();
        a.insert("css/main.css", b"x".to_vec(), false);
        assert_eq!(a.url("/css/main.css"), a.url("css/main.css"));
    }

    #[test]
    fn collects_from_disk_and_maps_logical_names() {
        let root = std::env::temp_dir().join(format!("sqzass-assets-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let sdir = root.join("static/css");
        std::fs::create_dir_all(&sdir).unwrap();
        std::fs::write(sdir.join("main.css"), "body{color:red}").unwrap();
        std::fs::write(root.join("static/CNAME"), "example.com").unwrap();

        let a = Assets::collect(&root, &cfg(), "").unwrap();
        assert!(a.url("css/main.css").unwrap().starts_with("/css/main."));
        assert_eq!(a.url("CNAME"), Some("/CNAME"));
        assert_eq!(a.files.len(), 2);

        // 서브경로에 올라가는 사이트는 에셋 URL도 같은 접두사를 가져야 한다.
        // 페이지 링크만 접두사를 갖고 스타일시트는 안 갖는 상태가 가장 나쁘다:
        // 사이트는 뜨는데 아무 스타일도 안 먹는다.
        let sub = Assets::collect(&root, &cfg(), "/repo").unwrap();
        assert!(
            sub.url("css/main.css")
                .unwrap()
                .starts_with("/repo/css/main."),
            "실제: {:?}",
            sub.url("css/main.css")
        );
        assert_eq!(sub.url("CNAME"), Some("/repo/CNAME"));

        std::fs::remove_dir_all(&root).ok();
    }

    /// 호스트별 설정 파일은 **이름이 곧 계약**이다. `_headers.9f8e7d6c`는 Netlify가
    /// 영영 찾지 않을 파일이고, `.domains.abc`도 Codeberg가 찾지 않는다. 이게
    /// 조용히 깨지면 배포는 성공하면서 도메인만 안 붙는다.
    #[test]
    fn host_config_files_pass_through_with_their_names() {
        let root =
            std::env::temp_dir().join(format!("sqzass-assets-passthru-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let s = root.join("static");
        std::fs::create_dir_all(s.join(".well-known")).unwrap();
        for (name, body) in [
            ("_headers", "/*\n  X-Frame-Options: DENY\n"),
            ("_redirects", "/old /new 301\n"),
            (".domains", "example.com\n"),
            ("CNAME", "example.com\n"),
            ("robots.txt", "User-agent: *\n"),
        ] {
            std::fs::write(s.join(name), body).unwrap();
        }
        std::fs::write(s.join(".well-known/security.txt"), "Contact: x\n").unwrap();

        let a = Assets::collect(&root, &cfg(), "").unwrap();
        for name in [
            "_headers",
            "_redirects",
            ".domains",
            "CNAME",
            "robots.txt",
            ".well-known/security.txt",
        ] {
            assert_eq!(
                a.url(name),
                Some(format!("/{name}").as_str()),
                "{name} 의 이름이 바뀌었다"
            );
            assert!(a.files.contains_key(name), "{name} 이 출력에 없다");
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn missing_static_dir_is_not_an_error() {
        let root = std::env::temp_dir().join("sqzass-assets-nonexistent-xyz");
        let a = Assets::collect(&root, &cfg(), "").unwrap();
        assert!(a.files.is_empty());
    }
}
