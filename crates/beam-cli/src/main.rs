mod args;
mod commands;
mod explain;

use std::process::ExitCode;

use clap::Parser;
use tracing_subscriber::EnvFilter;

use args::Cli;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_env("BEAM_LOG"))
        .with_writer(std::io::stderr)
        .init();
    match commands::run(Cli::parse()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{}", explain::render(&error));
            ExitCode::FAILURE
        }
    }
}
