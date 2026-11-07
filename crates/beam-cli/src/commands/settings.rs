use std::path::Path;

use beam_core::{Config, Error, Result};

use crate::args::RelayCommand;

pub async fn status(data_dir: &Path) -> Result<()> {
    let (node, _) = super::start(data_dir.to_owned()).await?;
    let config = node.config();
    println!("name      {}", config.name);
    println!("id        {}", node.id().to_z32());
    println!("downloads {}", config.download_dir.display());
    let relays =
        if config.relays.is_empty() { "none".to_owned() } else { config.relays.join(", ") };
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
            for url in &config.relays {
                println!("{url}");
            }
            Ok(())
        }
        RelayCommand::Add { url } => {
            url.parse::<iroh::RelayUrl>().map_err(|_| Error::InvalidRelayUrl(url.clone()))?;
            update(data_dir, |config| {
                config.relays.retain(|existing| *existing != url);
                config.relays.push(url);
            })
        }
        RelayCommand::Remove { url } => update(data_dir, |config| {
            config.relays.retain(|existing| *existing != url);
        }),
    }
}

fn update(data_dir: &Path, change: impl FnOnce(&mut Config)) -> Result<()> {
    std::fs::create_dir_all(data_dir)
        .map_err(|source| Error::Storage { path: data_dir.to_owned(), source: source.into() })?;
    let mut config = Config::load(data_dir)?;
    change(&mut config);
    config.save(data_dir)
}
