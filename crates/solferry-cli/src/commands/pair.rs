use std::path::PathBuf;
use std::time::Duration;

use qrcode::QrCode;
use qrcode::render::unicode::Dense1x2;
use serde_json::json;
use solferry_core::{Device, Error, Event, NearbyDevice, Node, Result};

const NEARBY_WINDOW: Duration = Duration::from_secs(4);
const ONLINE_WAIT: Duration = Duration::from_secs(5);
const INVITE_WAIT: Duration = Duration::from_secs(600);

pub async fn pair(data_dir: PathBuf, json: bool) -> Result<()> {
    let (node, _offers) = super::start(data_dir).await?;
    let mut events = node.events();
    node.online_within(ONLINE_WAIT).await;
    let invite = node.invite();
    if json {
        let code = json!({
            "event": "pair_code",
            "code": invite.code(),
            "expires_at_ms": invite.expires_at_ms,
        });
        println!("{code}");
    } else {
        if let Ok(code) = QrCode::new(invite.code()) {
            println!("{}", code.render::<Dense1x2>().quiet_zone(true).build());
        }
        println!(
            "{}\n\nWaiting for the other device. The code works once, for 10 minutes.",
            invite.code()
        );
    }
    let paired = async {
        loop {
            if let Ok(Event::Paired { device }) = events.recv().await {
                return device;
            }
        }
    };
    let paired = tokio::time::timeout(INVITE_WAIT, paired).await;
    if let Ok(device) = &paired {
        show_paired(device, json);
    }
    node.shutdown().await?;
    paired.map(drop).map_err(|_| Error::TicketExpired)
}

pub async fn pair_nearby(data_dir: PathBuf, name: &str, json: bool) -> Result<()> {
    let (node, _offers) = super::start(data_dir).await?;
    let device = node.find_nearby(name, NEARBY_WINDOW).await;
    let device = device.ok_or_else(|| Error::UnknownDevice(name.to_owned()))?;
    compare_codes(node, device, name, json).await
}

pub async fn pair_address(data_dir: PathBuf, address: &str, json: bool) -> Result<()> {
    let (node, _offers) = super::start(data_dir).await?;
    let device = node.find_address(address).await?;
    let name = device.name.clone().unwrap_or_else(|| address.to_owned());
    compare_codes(node, device, &name, json).await
}

async fn compare_codes(node: Node, device: NearbyDevice, name: &str, json: bool) -> Result<()> {
    let peer = device.addr.id;
    let paired = node
        .pair_nearby(device.addr, |code| {
            if json {
                let event = json!({
                    "event": "compare_code",
                    "code": code,
                    "peer": peer.to_z32(),
                    "peer_name": name,
                });
                println!("{event}");
            } else {
                println!("Code: {code}\nCheck that {name} shows the same code, then accept there.");
            }
        })
        .await?;
    show_paired(&paired, json);
    node.shutdown().await
}

pub async fn join(data_dir: PathBuf, code: &str, json: bool) -> Result<()> {
    let (node, _offers) = super::start(data_dir).await?;
    let device = node.join(code).await?;
    show_paired(&device, json);
    node.shutdown().await
}

fn show_paired(device: &Device, json: bool) {
    if json {
        println!("{}", super::json::encode(&Event::Paired { device: device.clone() }));
    } else {
        println!("Paired with {}", device.name);
    }
}
