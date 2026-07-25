use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
}

#[derive(Subcommand)]
enum Command {
    /// 사이트를 빌드한다
    Build(BuildArgs),
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
    if let Err(err) = run() {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Build(args) => {
            let stats = build(&BuildOptions {
                input: args.input,
                output: args.output,
                drafts: args.drafts,
                base_url: args.base_url,
            })?;
            println!(
                "{} pages → {}",
                stats.pages_written,
                display_path(&stats.output_dir)
            );
            Ok(())
        }
    }
}
