use clap::Parser as ClapParser;
use poke_http::{http::Parser, tui};
use std::path::PathBuf;

#[derive(ClapParser)]
#[command(name = "poke")]
#[command(author, version, about = "Interactive HTTP client for .http files")]
struct Args {
    #[arg(value_name = "FILE", help = "Path to the .http file")]
    file: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let http_file = Parser::parse_file(&args.file)?;

    if http_file.requests.is_empty() {
        eprintln!("No requests found in {:?}", args.file);
        std::process::exit(1);
    }

    tui::run(http_file).await
}
