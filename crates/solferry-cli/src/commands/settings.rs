use std::path::Path;
use std::time::Duration;

use solferry_core::{Config, Error, RelayHealth, RelayLink, Result};

use crate::args::RelayCommand;

const RELAY_CHECK_WAIT: Duration = Duration::from_secs(6);

pub async fn status(data_dir: &Path) -> Result<()> {
    let (node, _) = super::start(data_dir.to_owned()).await?;
    let config = node.config();
    println!("name      {}", config.name);
    println!("id        {}", node.id().to_z32());
    println!("downloads {}", config.download_dir.display());
    let urls: Vec<String> = config.relays.iter().map(|link| address(link)).collect();
    let relays = if urls.is_empty() { "none".to_owned() } else { urls.join(", ") };
    println!("relays    {relays}");
    let overlay = node.overlay_address().map_or("none".to_owned(), |ip| ip.to_string());
    println!("overlay   {overlay}");
    let lan: Vec<String> = solferry_core::lan_addresses().iter().map(ToString::to_string).collect();
    let lan = if lan.is_empty() { "none".to_owned() } else { lan.join(", ") };
    println!("lan       {lan}");
    node.shutdown().await
}

pub async fn check_relays(data_dir: &Path) -> Result<()> {
    let (node, _) = super::start(data_dir.to_owned()).await?;
    let relays = node.relay_health(RELAY_CHECK_WAIT).await;
    if relays.is_empty() {
        println!("No relays. This device only uses the local network and overlays.");
    }
    let mut healthy = true;
    for relay in &relays {
        println!("{}", relay.url);
        println!("  server  {}", state(relay.reachable));
        println!("  access  {}", state(relay.reachable && relay.accepted));
        println!("  direct  {}", state(relay.direct_paths));
        for problem in problems(relay) {
            println!("  next: {problem}");
        }
        healthy &= relay.accepted;
    }
    node.shutdown().await?;
    if healthy { Ok(()) } else { Err(Error::RelayCheckFailed) }
}

fn state(ok: bool) -> &'static str {
    if ok { "ok" } else { "failed" }
}

fn problems(relay: &RelayHealth) -> Vec<&'static str> {
    let mut problems = Vec::new();
    if !relay.reachable {
        problems.push("The server can't be reached. Check its DNS name, that it is running, and that TCP 443 is open.");
    } else if !relay.accepted {
        problems.push("The server refused this device. The link's token is probably wrong, so paste the link your relay printed again.");
    }
    if relay.accepted && !relay.direct_paths {
        problems.push("UDP 7842 is blocked, so files stay on the relay. Open it on the server or its firewall.");
    }
    problems
}

pub fn rename(data_dir: &Path, name: String) -> Result<()> {
    update(data_dir, |config| config.name = name)
}

pub fn relay(data_dir: &Path, command: RelayCommand) -> Result<()> {
    match command {
        RelayCommand::Check => Ok(()),
        RelayCommand::Show => {
            let config = Config::load(data_dir)?;
            if config.relays.is_empty() {
                println!("No relays. This device only uses the local network and overlays.");
            }
            for link in &config.relays {
                println!("{}", address(link));
            }
            Ok(())
        }
        RelayCommand::Add { url } => {
            let added = RelayLink::parse(&url)?;
            update(data_dir, |config| {
                config.relays.retain(|existing| !same_relay(existing, &added));
                config.relays.push(url);
            })
        }
        RelayCommand::Remove { url } => {
            let removed = RelayLink::parse(&url)?;
            update(data_dir, |config| {
                config.relays.retain(|existing| !same_relay(existing, &removed));
            })
        }
    }
}

/// Tokens stay out of terminals and screenshots.
fn address(link: &str) -> String {
    RelayLink::parse(link)
        .map_or_else(|_| "invalid relay".to_owned(), |link| link.url().to_string())
}

fn same_relay(link: &str, other: &RelayLink) -> bool {
    RelayLink::parse(link).is_ok_and(|link| link.url() == other.url())
}

fn update(data_dir: &Path, change: impl FnOnce(&mut Config)) -> Result<()> {
    std::fs::create_dir_all(data_dir)
        .map_err(|source| Error::Storage { path: data_dir.to_owned(), source: source.into() })?;
    let mut config = Config::load(data_dir)?;
    change(&mut config);
    config.save(data_dir)
}
