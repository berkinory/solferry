mod args;
mod commands;
mod explain;
mod help;

use std::process::ExitCode;

use clap::Parser;
use tracing_subscriber::EnvFilter;

use args::Cli;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("SOLFERRY_LOG").unwrap_or_else(|_| EnvFilter::new("off")),
        )
        .with_writer(std::io::stderr)
        .init();
    let cli = Cli::parse();
    let json = cli.json;
    match commands::run(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) if json => {
            eprintln!("{}", explain::fields(&error));
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!("{}", explain::render(&error));
            ExitCode::FAILURE
        }
    }
}
