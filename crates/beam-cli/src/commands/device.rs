use std::path::PathBuf;
use std::time::Duration;

use beam_core::{Relationship, Result};
use qrcode::QrCode;
use qrcode::render::unicode::Dense1x2;

const NEARBY_WINDOW: Duration = Duration::from_secs(4);

pub async fn pair(data_dir: PathBuf, contact: bool) -> Result<()> {
    let (node, _offers) = super::start(data_dir).await?;
    let known = node.devices().len();
    let relationship = if contact {
        Relationship::Contact
    } else {
        Relationship::Own
    };
    node.online().await;
    let invite = node.invite(relationship);
    if let Ok(code) = QrCode::new(invite.code()) {
        println!("{}", code.render::<Dense1x2>().quiet_zone(true).build());
    }
    println!(
        "{}\n\nWaiting for the other device. The code works once, for 10 minutes.",
        invite.code()
    );
    let mut poll = tokio::time::interval(Duration::from_millis(250));
    let deadline = tokio::time::sleep(Duration::from_secs(600));
    tokio::pin!(deadline);
    loop {
        tokio::select! {
            _ = poll.tick() => {
                if let Some(device) = node.devices().get(known) {
                    println!("Paired with {}", device.name);
                    break;
                }
            }
            () = &mut deadline => break,
        }
    }
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
        println!("No paired devices. Run `beam pair` to add one.");
    }
    for device in devices {
        let kind = match device.relationship {
            Relationship::Own => "mine",
            Relationship::Contact => "contact",
        };
        println!("{:<24} {:<8} {}", device.name, kind, device.id.to_z32());
    }
    node.shutdown().await
}

pub async fn forget(data_dir: PathBuf, query: &str) -> Result<()> {
    let (node, _offers) = super::start(data_dir).await?;
    let device = node.forget(query)?;
    println!("Forgot {}", device.name);
    node.shutdown().await
}

pub async fn nearby(data_dir: PathBuf) -> Result<()> {
    let (node, _offers) = super::start(data_dir).await?;
    let devices = node.nearby(NEARBY_WINDOW).await;
    if devices.is_empty() {
        println!("Nothing found. Is beam running on the other device, on this network?");
    }
    for device in devices {
        let name = device.name.as_deref().unwrap_or("unnamed");
        let state = if device.known { "paired" } else { "new" };
        println!("{:<24} {:<8} {}", name, state, device.addr.id.to_z32());
    }
    node.shutdown().await
}
