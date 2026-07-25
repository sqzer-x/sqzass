use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use sqzass::error::{Kind, kind_of, message};
use sqzass::serve::{ServeOptions, serve};
use sqzass::{BuildOptions, build, display_path};

#[derive(Parser)]
#[command(
    name = "sqzass",
    version,
    about = "A static site generator written in Rust",
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// 결과를 JSON 한 줄로 낸다 (스크립트·CI용)
    ///
    /// 서브커맨드 앞뒤 어디에 써도 되도록 global로 둔다. 사람이 기억해야 하는
    /// 플래그 위치가 하나 줄어든다.
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Command {
    /// 사이트를 빌드한다
    Build(BuildArgs),
    /// 라이브 리로드가 붙은 개발 서버를 띄운다
    Serve(ServeArgs),
    /// 새 사이트를 만든다
    Init(InitArgs),
}

#[derive(clap::Args)]
struct InitArgs {
    /// 만들 디렉터리. 없으면 만든다.
    #[arg(default_value = ".", value_name = "DIR")]
    dir: PathBuf,
}

#[derive(clap::Args)]
struct ServeArgs {
    /// 사이트 루트 (sqzass.toml이 있는 디렉터리)
    #[arg(short, long, default_value = ".", value_name = "DIR")]
    input: PathBuf,

    /// 바인드 주소
    #[arg(short, long, default_value = "127.0.0.1", value_name = "ADDR")]
    bind: std::net::IpAddr,

    /// 포트
    #[arg(short, long, default_value_t = 3000)]
    port: u16,

    /// 드래프트 페이지도 포함한다
    #[arg(long)]
    drafts: bool,

    /// 설정의 base_url을 덮어쓴다
    #[arg(long, value_name = "URL")]
    base_url: Option<String>,
}

#[derive(clap::Args)]
struct BuildArgs {
    /// 사이트 루트 (sqzass.toml이 있는 디렉터리)
    #[arg(short, long, default_value = ".", value_name = "DIR")]
    input: PathBuf,

    /// 출력 디렉터리. 생략하면 <input>/public.
    /// 지정하면 셸의 현재 디렉터리 기준으로 해석한다.
    #[arg(short, long, value_name = "DIR")]
    output: Option<PathBuf>,

    /// 드래프트 페이지도 포함한다
    #[arg(long)]
    drafts: bool,

    /// 설정의 base_url을 덮어쓴다
    #[arg(long, value_name = "URL")]
    base_url: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    let json = cli.json;
    if let Err(err) = run(cli) {
        let kind = kind_of(&err);
        if json {
            // 실패도 stdout으로 낸다. --json을 켠 쪽은 파이프 하나만 읽는다.
            println!(
                "{}",
                serde_json::json!({
                    "ok": false,
                    "error": message(&err),
                    "kind": kind.id(),
                    "code": kind.code(),
                })
            );
        } else {
            eprintln!("error: {}", message(&err));
        }
        std::process::exit(kind.code());
    }
}

fn run(cli: Cli) -> Result<()> {
    let json = cli.json;
    match cli.command {
        Command::Build(args) => {
            let stats = build(&BuildOptions {
                input: args.input,
                output: args.output,
                drafts: args.drafts,
                base_url: args.base_url,
            })?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "ok": true,
                        "pages": stats.pages_written,
                        "output": display_path(&stats.output_dir),
                    })
                );
            } else {
                println!(
                    "{} pages → {}",
                    stats.pages_written,
                    display_path(&stats.output_dir)
                );
            }
            Ok(())
        }
        Command::Init(args) => {
            let written = sqzass::init::init(&args.dir).map_err(Kind::Io.tag())?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "ok": true,
                        "dir": display_path(&args.dir),
                        "files": written,
                    })
                );
            } else {
                for rel in &written {
                    println!("  {}", display_path(&args.dir.join(rel)));
                }
                println!("\nsqzass serve -i {}", display_path(&args.dir));
            }
            Ok(())
        }
        Command::Serve(args) => {
            // `build`는 동기 경로라 런타임을 서버에서만 만든다.
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(serve(ServeOptions {
                input: args.input,
                bind: args.bind,
                port: args.port,
                drafts: args.drafts,
                base_url: args.base_url,
            }))
        }
    }
}
