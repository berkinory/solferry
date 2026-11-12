use std::path::PathBuf;
use std::time::Duration;

use iroh::EndpointId;
use serde_json::json;
use solferry_core::{Error, Node, Result};

use super::json::platform;
use crate::args::ApprovalArg;

const NEARBY_WINDOW: Duration = Duration::from_secs(4);

pub async fn list(data_dir: PathBuf, json: bool) -> Result<()> {
    let (node, _offers) = super::start(data_dir).await?;
    let devices = node.devices();
    if json {
        let devices: Vec<_> = devices
            .iter()
            .map(|device| {
                json!({
                    "id": device.id.to_z32(),
                    "name": device.name,
                    "approval": approval_name(device.auto_accept),
                    "platform": platform(device.platform),
                })
            })
            .collect();
        let blocked: Vec<_> = node
            .blocked()
            .iter()
            .map(|device| json!({ "id": device.id.to_z32(), "name": device.name }))
            .collect();
        println!("{}", json!({ "devices": devices, "blocked": blocked }));
        return node.shutdown().await;
    }
    if devices.is_empty() {
        println!("No paired devices. Run `solferry pair` to add one.");
    }
    for device in devices {
        let approval = approval_name(device.auto_accept);
        println!("{:<24} {:<8} {}", device.name, approval, device.id.to_z32());
    }
    for device in node.blocked() {
        let name = device.name.as_deref().unwrap_or("blocked device");
        println!("{:<24} {:<8} {}", name, "blocked", device.id.to_z32());
    }
    node.shutdown().await
}

fn approval_name(auto_accept: bool) -> &'static str {
    if auto_accept { "auto" } else { "ask" }
}

pub async fn forget(data_dir: PathBuf, query: &str, json: bool) -> Result<()> {
    let (node, _offers) = super::start(data_dir).await?;
    let device = node.forget(query)?;
    if json {
        let result =
            json!({ "action": "forget", "peer": device.id.to_z32(), "peer_name": device.name });
        println!("{result}");
    } else {
        println!("Forgot {}", device.name);
    }
    node.shutdown().await
}

pub async fn approval(data_dir: PathBuf, query: &str, mode: ApprovalArg, json: bool) -> Result<()> {
    let (node, _offers) = super::start(data_dir).await?;
    let device = node.find_device(query)?;
    let auto_accept = matches!(mode, ApprovalArg::Auto);
    node.set_auto_accept(device.id, auto_accept)?;
    if json {
        let result = json!({
            "action": "approval",
            "peer": device.id.to_z32(),
            "peer_name": device.name,
            "approval": approval_name(auto_accept),
        });
        println!("{result}");
    }
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

pub async fn block(data_dir: PathBuf, query: &str, json: bool) -> Result<()> {
    let (node, _offers) = super::start(data_dir).await?;
    let id = device_id(&node, query)?;
    node.block(id, None)?;
    if json {
        println!("{}", json!({ "action": "block", "peer": id.to_z32() }));
    }
    node.shutdown().await
}

pub async fn unblock(data_dir: PathBuf, query: &str, json: bool) -> Result<()> {
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
    if json {
        println!("{}", json!({ "action": "unblock", "peer": id.to_z32() }));
    }
    node.shutdown().await
}

pub async fn nearby(data_dir: PathBuf, address: Option<&str>, json: bool) -> Result<()> {
    let (node, _offers) = super::start(data_dir).await?;
    let devices = match address {
        Some(address) => vec![node.find_address(address).await?],
        None => node.nearby(NEARBY_WINDOW).await,
    };
    if json {
        let devices: Vec<_> = devices
            .iter()
            .map(|device| {
                json!({
                    "id": device.addr.id.to_z32(),
                    "name": device.name,
                    "paired": device.known,
                    "overlay": device.overlay,
                    "platform": platform(device.platform),
                })
            })
            .collect();
        println!("{}", json!({ "devices": devices }));
        return node.shutdown().await;
    }
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
