use futures::SinkExt;
use futures::channel::mpsc::UnboundedSender;
use iroh::EndpointAddr;
use solferry_core::Node;

use super::{Command, ONLINE_WAIT, Target, Update};

/// Runs a command that only needs the node, without holding up the next one.
pub(super) fn dispatch(node: &Node, command: Command, updates: &UnboundedSender<Update>) {
    let mut updates = updates.clone();
    let node = node.clone();
    tokio::spawn(async move {
        match command {
            Command::Send { targets, paths } => {
                for Target { addr, name } in targets {
                    let (node, paths, mut updates) = (node.clone(), paths.clone(), updates.clone());
                    tokio::spawn(async move {
                        if let Err(error) = node.send(addr.clone(), paths.clone()).await {
                            let failed = Update::SendFailed { name, addr, paths, error };
                            let _ = updates.send(failed).await;
                        }
                    });
                }
            }
            Command::SendText { target: Target { addr, name }, text } => {
                let result = node.send_text(addr, text).await;
                let _ = updates.send(Update::TextShared { name, result }).await;
            }
            Command::Cancel(id) => node.cancel(id),
            Command::PairNearby { addr } => pair(&node, addr, &mut updates).await,
            Command::PairAddress { address } => match node.find_address(&address).await {
                Ok(device) => pair(&node, device.addr, &mut updates).await,
                Err(error) => {
                    let _ = updates.send(Update::AddressFailed(error)).await;
                }
            },
            Command::Invite => {
                node.online_within(ONLINE_WAIT).await;
                let code = node.invite().code().to_owned();
                let _ = updates.send(Update::Invite(code)).await;
            }
            Command::Join { code } => {
                let result = match node.join(&code).await {
                    Ok(device) => Update::Paired(device),
                    Err(error) => Update::PairingFailed(error),
                };
                let _ = updates.send(result).await;
                let _ = updates.send(Update::Devices(node.devices())).await;
            }
            Command::ClearHistory => {
                node.clear_history();
                let _ = updates.send(Update::History(Vec::new())).await;
            }
            Command::Forget(id) => {
                let _ = node.forget(&id.to_z32());
                let _ = updates.send(Update::Devices(node.devices())).await;
            }
            Command::SetAutoAccept { id, enabled } => {
                let _ = node.set_auto_accept(id, enabled);
                let _ = updates.send(Update::Devices(node.devices())).await;
            }
            Command::Unblock(id) => {
                let _ = node.unblock(id);
                let _ = updates.send(Update::Blocked(node.blocked())).await;
            }
            Command::Answer { .. }
            | Command::AcceptAlways(_)
            | Command::AnswerPairing { .. }
            | Command::BlockOffer(_)
            | Command::Apply(_)
            | Command::CheckForUpdates
            | Command::InstallUpdate
            | Command::SetCheckUpdates(_)
            | Command::SetShowInTray(_)
            | Command::Link(_)
            | Command::StopLink
            | Command::Quit => {}
        }
    });
}

async fn pair(node: &Node, addr: EndpointAddr, updates: &mut UnboundedSender<Update>) {
    let shown = updates.clone();
    let show = move |code: &str| {
        let _ = shown.unbounded_send(Update::PairingCode(code.to_owned()));
    };
    let result = match node.pair_nearby(addr, show).await {
        Ok(device) => Update::Paired(device),
        Err(error) => Update::PairingFailed(error),
    };
    let _ = updates.send(result).await;
    let _ = updates.send(Update::Devices(node.devices())).await;
}
