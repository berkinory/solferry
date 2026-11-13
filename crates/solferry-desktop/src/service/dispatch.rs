use std::path::{Path, PathBuf};
use std::sync::Arc;

use futures::SinkExt;
use futures::channel::mpsc::UnboundedSender;
use iroh::EndpointAddr;
use solferry_core::{Config, Error, Link, LinkFile, Node, QueuedFiles};

use super::{Command, ONLINE_WAIT, Target, Update};
use crate::heic;

/// Runs a command that only needs the node, without holding up the next one.
pub(super) fn dispatch(
    node: &Node,
    data_dir: &Path,
    command: Command,
    updates: &UnboundedSender<Update>,
) {
    let mut updates = updates.clone();
    let node = node.clone();
    let data_dir = data_dir.to_owned();
    tokio::spawn(async move {
        match command {
            Command::Send { targets, paths } => {
                send(&node, &data_dir, targets, paths, &updates).await;
            }
            Command::Queue { targets, paths } => {
                for target in targets {
                    let _ = updates.send(queue(&node, target, paths.clone())).await;
                }
            }
            Command::Unqueue(id) => node.unqueue(id),
            Command::SendText { target: Target { addr, name, .. }, text } => {
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
            | Command::Restart
            | Command::CheckForUpdates
            | Command::InstallUpdate
            | Command::SetCheckUpdates(_)
            | Command::SetShowInTray(_)
            | Command::SetLanguage(_)
            | Command::SetAppearance(_)
            | Command::SetConvertHeic(_)
            | Command::Link(_)
            | Command::StopLink
            | Command::Quit => {}
        }
    });
}

fn queue(node: &Node, target: Target, paths: Vec<PathBuf>) -> Update {
    match node.queue(target.addr.id, QueuedFiles::Paths(paths.clone())) {
        Ok(_) => Update::Queued { name: target.name },
        Err(error) => Update::SendFailed { target, paths, error },
    }
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

pub(super) async fn open_link(node: &Node, paths: Vec<PathBuf>) -> (Option<Link>, Update) {
    let receiving = paths.is_empty();
    let started = if receiving {
        node.receive_link().await
    } else {
        node.share_link(paths.into_iter().map(LinkFile::Path).collect()).await
    };
    match started {
        Ok(link) => {
            let urls = link.urls().to_vec();
            (Some(link), Update::LinkReady { urls, receiving })
        }
        Err(error) => (None, Update::LinkFailed(error)),
    }
}

/// Sends to each target at once. Targets that can't open HEIC get JPEG copies, made once and
/// kept until the last of their sends ends.
async fn send(
    node: &Node,
    data_dir: &Path,
    targets: Vec<Target>,
    paths: Vec<PathBuf>,
    updates: &UnboundedSender<Update>,
) {
    let convert = cfg!(target_os = "macos")
        && targets.iter().any(|target| heic::wanted(target.platform))
        && Config::load(data_dir).is_ok_and(|config| config.convert_heic);
    let staged = if convert {
        let originals = paths.clone();
        tokio::task::spawn_blocking(move || heic::stage(&originals))
            .await
            .ok()
            .flatten()
            .map(Arc::new)
    } else {
        None
    };
    for target in targets {
        let jpegs = staged.clone().filter(|_| heic::wanted(target.platform));
        let sent = jpegs.as_ref().map_or_else(|| paths.clone(), |staged| staged.paths.clone());
        let (node, paths, mut updates) = (node.clone(), paths.clone(), updates.clone());
        tokio::spawn(async move {
            let result = node.send(target.addr.clone(), sent).await;
            drop(jpegs);
            let update = match result {
                // A paired device that didn't answer gets it once it is back.
                Err(Error::Unreachable(_))
                    if node.find_device(&target.addr.id.to_z32()).is_ok() =>
                {
                    queue(&node, target, paths)
                }
                Err(error) => Update::SendFailed { target, paths, error },
                Ok(_) => return,
            };
            let _ = updates.send(update).await;
        });
    }
}
