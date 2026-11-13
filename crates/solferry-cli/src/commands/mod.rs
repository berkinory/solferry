mod device;
mod json;
mod link;
mod pair;
mod prompt;
mod queue;
mod receive;
mod send;
mod settings;
mod stdin;

use std::path::PathBuf;

use solferry_core::{IncomingOffer, Node, NodeOptions, Result, default_data_dir};
use tokio::sync::mpsc;

use crate::args::{Cli, Command, RelayCommand};

pub async fn run(cli: Cli) -> Result<()> {
    let data_dir = cli.data_dir.unwrap_or_else(default_data_dir);
    let json = cli.json;
    match cli.command {
        Command::Status => settings::status(&data_dir, json).await,
        Command::Rename { name } => settings::rename(&data_dir, name, json),
        Command::Relay { command: RelayCommand::Check } => {
            settings::check_relays(&data_dir, json).await
        }
        Command::Relay { command } => settings::relay(&data_dir, command, json),
        Command::Send { text: Some(text), targets, .. } => {
            send::text(data_dir, text, targets, json).await
        }
        Command::Send { paths, name, targets, text: None, queue } => {
            let payload = send::Payload::parse(paths, name);
            send::run(data_dir, payload, targets, queue, json).await
        }
        Command::Queue { cancel } => queue::run(data_dir, cancel.as_deref(), json).await,
        Command::Link { paths } => link::run(data_dir, paths, json).await,
        Command::Receive { dir, yes, once } => receive::run(data_dir, dir, yes, once, json).await,
        Command::Pair { with: Some(name), .. } => pair::pair_nearby(data_dir, &name, json).await,
        Command::Pair { address: Some(address), .. } => {
            pair::pair_address(data_dir, &address, json).await
        }
        Command::Pair { .. } => pair::pair(data_dir, json).await,
        Command::Join { code } => pair::join(data_dir, &code, json).await,
        Command::Devices => device::list(data_dir, json).await,
        Command::Forget { device } => device::forget(data_dir, &device, json).await,
        Command::Approval { device, mode } => device::approval(data_dir, &device, mode, json).await,
        Command::Block { device } => device::block(data_dir, &device, json).await,
        Command::Unblock { id } => device::unblock(data_dir, &id, json).await,
        Command::Nearby { address } => device::nearby(data_dir, address.as_deref(), json).await,
    }
}

async fn start(data_dir: PathBuf) -> Result<(Node, mpsc::Receiver<IncomingOffer>)> {
    Node::start(NodeOptions { data_dir, isolated: false }).await
}

/// Resolves on Ctrl-C, or on SIGTERM where there is one, so a run can close its connections
/// before it exits instead of leaving the other side waiting for them to time out.
async fn interrupted() {
    #[cfg(unix)]
    if let Ok(mut terminate) =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
    {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = terminate.recv() => {}
        }
        return;
    }
    let _ = tokio::signal::ctrl_c().await;
}
