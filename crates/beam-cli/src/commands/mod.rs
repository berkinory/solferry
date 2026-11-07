mod device;
mod json;
mod receive;
mod send;
mod settings;

use std::path::PathBuf;

use beam_core::{IncomingOffer, Node, NodeOptions, Result, default_data_dir};
use tokio::sync::mpsc;

use crate::args::{Cli, Command};

pub async fn run(cli: Cli) -> Result<()> {
    let data_dir = cli.data_dir.unwrap_or_else(default_data_dir);
    match cli.command {
        Command::Status => settings::status(&data_dir).await,
        Command::Rename { name } => settings::rename(&data_dir, name),
        Command::Relay { command } => settings::relay(&data_dir, command),
        Command::Send { paths, targets } => send::run(data_dir, paths, targets, cli.json).await,
        Command::Receive { dir, yes, once } => {
            receive::run(data_dir, dir, yes, once, cli.json).await
        }
        Command::Pair { ask } => device::pair(data_dir, !ask).await,
        Command::Join { code, ask } => device::join(data_dir, &code, !ask).await,
        Command::Devices => device::list(data_dir).await,
        Command::Forget { device } => device::forget(data_dir, &device).await,
        Command::Approval { device, mode } => device::approval(data_dir, &device, mode).await,
        Command::Block { device } => device::block(data_dir, &device).await,
        Command::Unblock { id } => device::unblock(data_dir, &id).await,
        Command::Nearby => device::nearby(data_dir).await,
    }
}

async fn start(data_dir: PathBuf) -> Result<(Node, mpsc::Receiver<IncomingOffer>)> {
    Node::start(NodeOptions { data_dir, isolated: false }).await
}
