use std::path::PathBuf;
use std::time::Duration;

use iroh::EndpointId;
use qrcode::QrCode;
use qrcode::render::unicode::Dense1x2;
use solferry_core::{Error, Event, Node, Result};

use crate::args::ApprovalArg;

const NEARBY_WINDOW: Duration = Duration::from_secs(4);
const ONLINE_WAIT: Duration = Duration::from_secs(5);
const INVITE_WAIT: Duration = Duration::from_secs(600);

pub async fn pair(data_dir: PathBuf) -> Result<()> {
    let (node, _offers) = super::start(data_dir).await?;
    let mut events = node.events();
    node.online_within(ONLINE_WAIT).await;
    let invite = node.invite();
    if let Ok(code) = QrCode::new(invite.code()) {
        println!("{}", code.render::<Dense1x2>().quiet_zone(true).build());
    }
    println!(
        "{}\n\nWaiting for the other device. The code works once, for 10 minutes.",
        invite.code()
    );
    let paired = async {
        loop {
            if let Ok(Event::Paired { device }) = events.recv().await {
                return device;
            }
        }
    };
    if let Ok(device) = tokio::time::timeout(INVITE_WAIT, paired).await {
        println!("Paired with {}", device.name);
    }
    node.shutdown().await
}

pub async fn pair_nearby(data_dir: PathBuf, name: &str) -> Result<()> {
    let (node, _offers) = super::start(data_dir).await?;
    let device = node.find_nearby(name, NEARBY_WINDOW).await;
    let device = device.ok_or_else(|| Error::UnknownDevice(name.to_owned()))?;
    let paired = node
        .pair_nearby(device.addr, |code| {
            println!("Code: {code}\nCheck that {name} shows the same code, then accept there.");
        })
        .await?;
    println!("Paired with {}", paired.name);
    node.shutdown().await
}

pub async fn pair_address(data_dir: PathBuf, address: &str) -> Result<()> {
    let (node, _offers) = super::start(data_dir).await?;
    let device = node.find_address(address).await?;
    let name = device.name.unwrap_or_else(|| address.to_owned());
    let paired = node
        .pair_nearby(device.addr, |code| {
            println!("Code: {code}\nCheck that {name} shows the same code, then accept there.");
        })
        .await?;
    println!("Paired with {}", paired.name);
    node.shutdown().await
}

pub async fn join(data_dir: PathBuf, code: &str) -> Result<()> {
    let (node, _offers) = super::start(data_dir).await?;
    let device = node.join(code).await?;
    println!("Paired with {}", device.name);
    node.shutdown().await
}

pub async fn list(data_dir: PathBuf) -> Result<()> {
    let (node, _offers) = super::start(data_dir).await?;
    let devices = node.devices();
    if devices.is_empty() {
        println!("No paired devices. Run `solferry pair` to add one.");
    }
    for device in devices {
        let approval = if device.auto_accept { "auto" } else { "ask" };
        println!("{:<24} {:<8} {}", device.name, approval, device.id.to_z32());
    }
    for device in node.blocked() {
        let name = device.name.as_deref().unwrap_or("blocked device");
        println!("{:<24} {:<8} {}", name, "blocked", device.id.to_z32());
    }
    node.shutdown().await
}

pub async fn forget(data_dir: PathBuf, query: &str) -> Result<()> {
    let (node, _offers) = super::start(data_dir).await?;
    let device = node.forget(query)?;
    println!("Forgot {}", device.name);
    node.shutdown().await
}

pub async fn approval(data_dir: PathBuf, query: &str, mode: ApprovalArg) -> Result<()> {
    let (node, _offers) = super::start(data_dir).await?;
    let device = node.find_device(query)?;
    node.set_auto_accept(device.id, matches!(mode, ApprovalArg::Auto))?;
    node.shutdown().await
}

fn device_id(node: &Node, query: &str) -> Result<EndpointId> {
    match node.find_device(query) {
        Ok(device) => Ok(device.id),
        Err(Error::UnknownDevice(_)) => {
            EndpointId::from_z32(query).map_err(|_| Error::UnknownDevice(query.to_owned()))
        }
        Err(error) => Err(error),
    }
}

pub async fn block(data_dir: PathBuf, query: &str) -> Result<()> {
    let (node, _offers) = super::start(data_dir).await?;
    node.block(device_id(&node, query)?, None)?;
    node.shutdown().await
}

pub async fn unblock(data_dir: PathBuf, query: &str) -> Result<()> {
    let (node, _offers) = super::start(data_dir).await?;
    let mut matches = node
        .blocked()
        .into_iter()
        .map(|device| device.id)
        .filter(|id| id.to_z32().starts_with(query));
    let id = matches.next().ok_or_else(|| Error::UnknownDevice(query.to_owned()))?;
    if matches.next().is_some() {
        return Err(Error::AmbiguousDevice(query.to_owned()));
    }
    node.unblock(id)?;
    node.shutdown().await
}

pub async fn nearby(data_dir: PathBuf, address: Option<&str>) -> Result<()> {
    let (node, _offers) = super::start(data_dir).await?;
    let devices = match address {
        Some(address) => vec![node.find_address(address).await?],
        None => node.nearby(NEARBY_WINDOW).await,
    };
    if devices.is_empty() {
        println!("Nothing found. Is solferry running on the other device, on this network?");
    }
    for device in devices {
        let name = device.name.as_deref().unwrap_or("unnamed");
        let state = if device.known { "paired" } else { "new" };
        println!("{:<24} {:<8} {}", name, state, device.addr.id.to_z32());
    }
    node.shutdown().await
}
