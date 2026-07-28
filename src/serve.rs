//! 개발 서버. 인메모리 서빙 + 라이브 리로드.

use crate::{BuildOptions, BuildOutput, build_to_memory};
use anyhow::{Context, Result};
use axum::Router;
use axum::body::Body;
use axum::extract::{State, WebSocketUpgrade, ws};
use axum::http::{HeaderMap, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use notify::RecursiveMode;
use notify_debouncer_full::new_debouncer;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::broadcast;

/// 라이브 리로드 엔드포인트. 사이트 경로와 겹치지 않게 접두사를 붙인다.
const LIVE_PATH: &str = "/__sqzass/live";
/// 편집기의 원자적 저장(임시 파일 + rename)은 한 번의 "저장"에 3~5개 이벤트를 낸다.
const DEBOUNCE: Duration = Duration::from_millis(150);

#[derive(Debug, Clone)]
pub struct ServeOptions {
    pub input: PathBuf,
    pub bind: IpAddr,
    pub port: u16,
    pub drafts: bool,
    pub base_url: Option<String>,
}

/// 브라우저로 보내는 신호.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Signal {
    /// 전체 새로고침
    Reload,
    /// CSS만 바뀜 — `<link>`의 href만 갈아끼운다. 스크롤과 열린 상태가 유지된다.
    Css,
    /// 빌드 실패 — 오버레이를 띄운다. 마지막으로 성공한 사이트는 계속 서빙된다.
    Error(String),
    /// 에러가 해소됨
    Clear,
}

impl Signal {
    fn encode(&self) -> String {
        match self {
            Signal::Reload => "reload".into(),
            Signal::Css => "css".into(),
            Signal::Clear => "clear".into(),
            // 개행이 프레임을 깨지 않도록 한 줄로 만든다.
            Signal::Error(msg) => format!("error:{}", msg.replace('\n', "\\n")),
        }
    }
}

struct Shared {
    output: RwLock<BuildOutput>,
    /// 현재 미해결 빌드 에러. 뒤늦게 접속한 탭도 이걸 즉시 받는다.
    error: RwLock<Option<String>>,
    tx: broadcast::Sender<Signal>,
}

type SharedState = Arc<Shared>;

pub async fn serve(opts: ServeOptions) -> Result<()> {
    let build_opts = BuildOptions {
        input: opts.input.clone(),
        output: None,
        drafts: opts.drafts,
        base_url: opts.base_url.clone(),
        profile: false,
    };

    let initial = build_to_memory(&build_opts)
        .with_context(|| format!("{}의 첫 빌드 실패", opts.input.display()))?;
    println!("{} pages", initial.pages);

    let (tx, _) = broadcast::channel(64);
    let shared: SharedState = Arc::new(Shared {
        output: RwLock::new(initial),
        error: RwLock::new(None),
        tx,
    });

    let _watcher = spawn_watcher(&opts.input, build_opts, Arc::clone(&shared))?;

    let app = Router::new()
        .route(LIVE_PATH, get(live_socket))
        .fallback(get(serve_file))
        .with_state(shared);

    let addr = SocketAddr::new(opts.bind, opts.port);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("{addr}에 바인드할 수 없습니다"))?;

    println!("http://{addr}/  (Ctrl+C to stop)");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .context("서버 오류")?;
    Ok(())
}

/// 감시 대상 디렉터리. 출력 디렉터리는 넣지 않는다 — 자기가 쓴 걸 보고 다시 빌드하는
/// 무한 루프가 된다.
const WATCH_DIRS: &[&str] = &["content", "templates", "static", "i18n"];

fn spawn_watcher(
    root: &Path,
    build_opts: BuildOptions,
    shared: SharedState,
) -> Result<
    notify_debouncer_full::Debouncer<
        notify::RecommendedWatcher,
        notify_debouncer_full::RecommendedCache,
    >,
> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut debouncer =
        new_debouncer(DEBOUNCE, None, tx).context("파일 감시자를 만들 수 없습니다")?;

    for dir in WATCH_DIRS {
        let path = root.join(dir);
        if path.is_dir() {
            debouncer
                .watch(&path, RecursiveMode::Recursive)
                .with_context(|| format!("{}를 감시할 수 없습니다", path.display()))?;
        }
    }
    let cfg = root.join(crate::config::CONFIG_FILE);
    if cfg.is_file() {
        debouncer.watch(&cfg, RecursiveMode::NonRecursive).ok();
    }

    std::thread::spawn(move || {
        for res in rx {
            let Ok(events) = res else { continue };
            let paths: Vec<PathBuf> = events
                .iter()
                .flat_map(|e| e.paths.clone())
                .filter(|p| !is_noise(p))
                .collect();
            if paths.is_empty() {
                continue;
            }
            rebuild(&build_opts, &shared, &paths);
        }
    });

    Ok(debouncer)
}

/// 편집기 임시 파일과 VCS 내부 파일은 무시한다. 이걸 안 걸러내면 저장 한 번에
/// 리빌드가 여러 번 돈다.
fn is_noise(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if path.components().any(|c| c.as_os_str() == ".git") {
        return true;
    }
    name.starts_with('.')
        || name.ends_with('~')
        || name.ends_with(".swp")
        || name.ends_with(".swx")
        || name.ends_with(".tmp")
        || (name.starts_with('#') && name.ends_with('#'))
}

fn rebuild(build_opts: &BuildOptions, shared: &SharedState, paths: &[PathBuf]) {
    match build_to_memory(build_opts) {
        Ok(output) => {
            let had_error = shared.error.read().map(|e| e.is_some()).unwrap_or(false);
            if let Ok(mut slot) = shared.output.write() {
                *slot = output;
            }
            if had_error {
                if let Ok(mut e) = shared.error.write() {
                    *e = None;
                }
                let _ = shared.tx.send(Signal::Clear);
            }

            // CSS만 바뀌었으면 페이지를 새로 그리지 않는다 — 스크롤 위치와 열어둔
            // details/사이드바 상태가 유지되어야 디자인 작업이 견딜 만해진다.
            let css_only = paths
                .iter()
                .all(|p| p.extension().and_then(|e| e.to_str()) == Some("css"));
            let signal = if css_only {
                Signal::Css
            } else {
                Signal::Reload
            };
            println!("  rebuilt ({})", if css_only { "css" } else { "full" });
            let _ = shared.tx.send(signal);
        }
        Err(err) => {
            let msg = format!("{err:#}");
            eprintln!("error: {msg}");
            if let Ok(mut e) = shared.error.write() {
                *e = Some(msg.clone());
            }
            // 마지막으로 성공한 출력은 그대로 둔다. 빌드가 깨졌다고 사이트가
            // 사라지면 무엇이 깨졌는지 볼 수가 없다.
            let _ = shared.tx.send(Signal::Error(msg));
        }
    }
}

// --- HTTP ---

async fn serve_file(State(shared): State<SharedState>, uri: Uri) -> Response {
    let path = uri.path();
    let key = resolve_key(path);

    let (body, is_html) = {
        let files = shared.output.read().expect("출력 락");
        match files.files.get(&key) {
            Some(bytes) => (bytes.clone(), key.ends_with(".html")),
            None => {
                return (
                    StatusCode::NOT_FOUND,
                    [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                    format!("<!doctype html><meta charset=utf-8><title>404</title><h1>404</h1><p><code>{}</code> not found.</p>{}", html_escape(path), reload_script()),
                )
                    .into_response();
            }
        }
    };

    if !is_html {
        return ([(header::CONTENT_TYPE, content_type(&key))], body).into_response();
    }

    // 리로드 스크립트는 **서브 시점에** 주입한다. 빌드 산출물 자체는 프로덕션과
    // 바이트 단위로 동일하게 유지되어야 한다.
    let mut html = String::from_utf8_lossy(&body).into_owned();
    let error = shared.error.read().ok().and_then(|e| e.clone());
    let injected = format!("{}{}", overlay(error.as_deref()), reload_script());
    match html.rfind("</body>") {
        Some(i) => html.insert_str(i, &injected),
        None => html.push_str(&injected),
    }

    Response::builder()
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        // 두 번의 저장이 같은 초에 들어가면 검증자가 같아져 엉뚱한 304가 나간다.
        .header(header::CACHE_CONTROL, "no-store")
        .body(Body::from(html))
        .expect("응답 생성")
}

/// 요청 경로를 출력 파일 키로 바꾼다. `/a/b/` → `a/b/index.html`
fn resolve_key(path: &str) -> String {
    let p = path.trim_start_matches('/');
    if p.is_empty() {
        return "index.html".into();
    }
    if p.ends_with('/') {
        return format!("{p}index.html");
    }
    let last = Path::new(p)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    // `Path::extension()`은 `.nojekyll` 같은 점 파일에 None을 준다(파일명 전체가
    // stem으로 취급된다). 그것만 보고 판단하면 점 파일을 디렉터리로 오인한다.
    if Path::new(p).extension().is_some() || last.starts_with('.') {
        return p.to_string();
    }
    format!("{p}/index.html")
}

fn content_type(key: &str) -> &'static str {
    match Path::new(key).extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("xml") => "application/xml",
        Some("txt") => "text/plain; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("avif") => "image/avif",
        Some("ico") => "image/x-icon",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

// --- WebSocket ---

async fn live_socket(
    State(shared): State<SharedState>,
    headers: HeaderMap,
    upgrade: WebSocketUpgrade,
) -> Response {
    let origin = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok());
    let host = headers.get(header::HOST).and_then(|v| v.to_str().ok());

    if !origin_allowed(origin, host) {
        // Cross-Site WebSocket Hijacking 방어. 판단할 수 없으면 거절한다.
        return (StatusCode::FORBIDDEN, "origin not allowed").into_response();
    }

    upgrade.on_upgrade(move |socket| handle_socket(socket, shared))
}

async fn handle_socket(mut socket: ws::WebSocket, shared: SharedState) {
    let mut rx = shared.tx.subscribe();

    // 지금 빌드가 깨져 있다면 새로 붙은 탭에도 즉시 알려준다. 이게 없으면 리로드
    // 도중에 접속한 탭은 왜 화면이 옛날인지 알 수 없다.
    if let Some(err) = shared.error.read().ok().and_then(|e| e.clone())
        && socket
            .send(ws::Message::Text(Signal::Error(err).encode().into()))
            .await
            .is_err()
    {
        return;
    }

    while let Ok(signal) = rx.recv().await {
        if socket
            .send(ws::Message::Text(signal.encode().into()))
            .await
            .is_err()
        {
            break;
        }
    }
}

/// WebSocket 핸드셰이크의 Origin 검증.
///
/// **판단할 수 없으면 거절한다(fail closed).** Origin이 없거나 파싱이 안 되면 통과시키지
/// 않는다 — 개발 서버는 로컬에만 떠 있지만, 사용자가 아무 사이트나 열어둔 상태에서
/// 그 페이지가 이 소켓에 붙어 리빌드 신호를 엿보거나 트리거할 수 있으면 안 된다.
fn origin_allowed(origin: Option<&str>, host: Option<&str>) -> bool {
    let Some(origin) = origin else { return false };
    let Some(origin_host) = host_of_origin(origin) else {
        return false;
    };
    if is_loopback_host(&origin_host) {
        return true;
    }
    match host.and_then(strip_port) {
        Some(h) => h.eq_ignore_ascii_case(&origin_host),
        None => false,
    }
}

/// `scheme://host[:port]` 에서 host를 뽑는다. 대괄호 IPv6(`[::1]:3000`)를 처리한다.
fn host_of_origin(origin: &str) -> Option<String> {
    let rest = origin.split_once("://")?.1;
    let authority = rest.split(['/', '?', '#']).next()?;
    strip_port(authority).map(|h| h.to_ascii_lowercase())
}

fn strip_port(authority: &str) -> Option<&str> {
    if let Some(end) = authority.find(']') {
        // 대괄호 IPv6: `[::1]:3000` → `[::1]`
        return authority.get(..=end);
    }
    Some(match authority.rsplit_once(':') {
        Some((h, port)) if port.chars().all(|c| c.is_ascii_digit()) => h,
        _ => authority,
    })
}

fn is_loopback_host(host: &str) -> bool {
    matches!(host, "localhost" | "127.0.0.1" | "[::1]" | "::1")
}

// --- 브라우저로 주입되는 코드 ---

fn overlay(error: Option<&str>) -> String {
    let Some(err) = error else {
        return String::new();
    };
    format!(
        r#"<div id="__sqzass_error" style="position:fixed;inset:0;z-index:2147483647;background:#1a1114;color:#ffd7d7;font:13px/1.6 ui-monospace,monospace;padding:2rem;overflow:auto;white-space:pre-wrap"><strong style="color:#ff8080;font-size:15px">build failed</strong>

{}</div>"#,
        html_escape(err)
    )
}

fn reload_script() -> String {
    format!(
        r#"<script>(function(){{
var path={LIVE_PATH:?};
var back=500, ws;
function overlay(msg){{
  var el=document.getElementById("__sqzass_error");
  if(!msg){{ if(el) el.remove(); return; }}
  if(!el){{ el=document.createElement("div"); el.id="__sqzass_error";
    el.setAttribute("style","position:fixed;inset:0;z-index:2147483647;background:#1a1114;color:#ffd7d7;font:13px/1.6 ui-monospace,monospace;padding:2rem;overflow:auto;white-space:pre-wrap");
    document.body.appendChild(el); }}
  el.textContent="build failed\n\n"+msg;
}}
function swapCss(){{
  document.querySelectorAll('link[rel="stylesheet"]').forEach(function(l){{
    var u=new URL(l.href,location.href); u.searchParams.set("__r",Date.now()); l.href=u.href;
  }});
}}
function connect(){{
  ws=new WebSocket((location.protocol==="https:"?"wss://":"ws://")+location.host+path);
  ws.onopen=function(){{ back=500; }};
  ws.onmessage=function(e){{
    var m=e.data;
    if(m==="reload"){{ sessionStorage.setItem("__sqzass_y",String(scrollY)); location.reload(); }}
    else if(m==="css"){{ swapCss(); overlay(null); }}
    else if(m==="clear"){{ overlay(null); location.reload(); }}
    else if(m.indexOf("error:")===0){{ overlay(m.slice(6).replace(/\\n/g,"\n")); }}
  }};
  // 지수 백오프. 서버를 재시작해도 탭이 알아서 다시 붙는다.
  ws.onclose=function(){{ setTimeout(connect,back); back=Math.min(back*2,10000); }};
}}
var y=sessionStorage.getItem("__sqzass_y");
if(y!==null){{ sessionStorage.removeItem("__sqzass_y");
  addEventListener("load",function(){{ scrollTo(0,parseInt(y,10)||0); }}); }}
connect();
}})();</script>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_directory_urls_to_index() {
        assert_eq!(resolve_key("/"), "index.html");
        assert_eq!(resolve_key("/ko/"), "ko/index.html");
        assert_eq!(
            resolve_key("/start/installation/"),
            "start/installation/index.html"
        );
    }

    #[test]
    fn resolves_extensionless_urls_to_index() {
        // 트레일링 슬래시가 없어도 같은 페이지를 준다. 프로덕션 호스트가
        // 리다이렉트해 주는 경우가 많아서, 개발 중에만 404가 나면 혼란스럽다.
        assert_eq!(resolve_key("/start"), "start/index.html");
    }

    #[test]
    fn passes_through_files_with_extensions() {
        assert_eq!(resolve_key("/assets/highlight.css"), "assets/highlight.css");
        assert_eq!(resolve_key("/.nojekyll"), ".nojekyll");
    }

    #[test]
    fn origin_must_be_present_and_parseable() {
        // fail closed: 판단할 수 없으면 거절
        assert!(!origin_allowed(None, Some("127.0.0.1:3000")));
        assert!(!origin_allowed(Some("garbage"), Some("127.0.0.1:3000")));
        assert!(!origin_allowed(Some(""), Some("127.0.0.1:3000")));
    }

    #[test]
    fn loopback_origins_are_allowed() {
        assert!(origin_allowed(
            Some("http://localhost:3000"),
            Some("localhost:3000")
        ));
        assert!(origin_allowed(
            Some("http://127.0.0.1:3000"),
            Some("127.0.0.1:3000")
        ));
        assert!(origin_allowed(
            Some("http://[::1]:3000"),
            Some("[::1]:3000")
        ));
    }

    #[test]
    fn foreign_origins_are_rejected() {
        assert!(!origin_allowed(
            Some("https://evil.example"),
            Some("127.0.0.1:3000")
        ));
        // 접두사만 같은 도메인도 막아야 한다
        assert!(!origin_allowed(
            Some("http://localhost.evil.example"),
            Some("localhost:3000")
        ));
    }

    #[test]
    fn matching_non_loopback_host_is_allowed() {
        // LAN의 다른 기기에서 --bind 0.0.0.0 으로 접근하는 경우
        assert!(origin_allowed(
            Some("http://192.168.1.50:3000"),
            Some("192.168.1.50:3000")
        ));
        assert!(!origin_allowed(
            Some("http://192.168.1.99:3000"),
            Some("192.168.1.50:3000")
        ));
    }

    #[test]
    fn strips_ports_but_keeps_ipv6_brackets() {
        assert_eq!(strip_port("localhost:3000"), Some("localhost"));
        assert_eq!(strip_port("example.com"), Some("example.com"));
        assert_eq!(strip_port("[::1]:3000"), Some("[::1]"));
        assert_eq!(strip_port("[::1]"), Some("[::1]"));
    }

    #[test]
    fn editor_temp_files_are_ignored() {
        assert!(is_noise(Path::new("content/.post.md.swp")));
        assert!(is_noise(Path::new("content/post.md~")));
        assert!(is_noise(Path::new("content/#post.md#")));
        assert!(is_noise(Path::new("/repo/.git/index")));
        assert!(!is_noise(Path::new("content/post.md")));
        assert!(!is_noise(Path::new("templates/base.html")));
    }

    #[test]
    fn signals_encode_on_one_line() {
        assert_eq!(Signal::Reload.encode(), "reload");
        assert_eq!(Signal::Css.encode(), "css");
        let e = Signal::Error("line1\nline2".into()).encode();
        assert!(!e.contains('\n'), "개행이 프레임을 깬다: {e}");
        assert!(e.starts_with("error:"));
    }
}
