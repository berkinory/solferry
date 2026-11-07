use std::path::Path;

use beam_core::{Config, Error, RelayLink, Result};

use crate::args::RelayCommand;

pub async fn status(data_dir: &Path) -> Result<()> {
    let (node, _) = super::start(data_dir.to_owned()).await?;
    let config = node.config();
    println!("name      {}", config.name);
    println!("id        {}", node.id().to_z32());
    println!("downloads {}", config.download_dir.display());
    let urls: Vec<String> = config.relays.iter().map(|link| address(link)).collect();
    let relays = if urls.is_empty() { "none".to_owned() } else { urls.join(", ") };
    println!("relays    {relays}");
    node.shutdown().await
}

pub fn rename(data_dir: &Path, name: String) -> Result<()> {
    update(data_dir, |config| config.name = name)
}

pub fn relay(data_dir: &Path, command: RelayCommand) -> Result<()> {
    match command {
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
