use std::path::Path;

use beam_core::{Config, Error, RelayPolicy, Result};

use crate::args::{PolicyArg, RelayCommand};

pub async fn status(data_dir: &Path) -> Result<()> {
    let (node, _) = super::start(data_dir.to_owned()).await?;
    let config = node.config();
    println!("name      {}", config.name);
    println!("id        {}", node.id().to_z32());
    println!("downloads {}", config.download_dir.display());
    println!("relays    {}", policy_name(config.relay_policy));
    node.shutdown().await
}

pub fn rename(data_dir: &Path, name: String) -> Result<()> {
    update(data_dir, |config| config.name = name)
}

pub fn relay(data_dir: &Path, command: RelayCommand) -> Result<()> {
    match command {
        RelayCommand::Show => {
            let config = Config::load(data_dir)?;
            println!("policy  {}", policy_name(config.relay_policy));
            for url in &config.relays {
                println!("relay   {url}");
            }
            Ok(())
        }
        RelayCommand::Policy { policy } => update(data_dir, |config| {
            config.relay_policy = match policy {
                PolicyArg::HandshakeOnly => RelayPolicy::HandshakeOnly,
                PolicyArg::OwnRelays => RelayPolicy::OwnRelays,
                PolicyArg::Always => RelayPolicy::Always,
                PolicyArg::Off => RelayPolicy::Off,
            };
        }),
        RelayCommand::Add { url } => {
            url.parse::<iroh::RelayUrl>().map_err(|_| Error::InvalidRelayUrl(url.clone()))?;
            update(data_dir, |config| {
                config.relays.retain(|existing| *existing != url);
                config.relays.push(url);
                if config.relay_policy == RelayPolicy::HandshakeOnly {
                    config.relay_policy = RelayPolicy::OwnRelays;
                }
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

fn policy_name(policy: RelayPolicy) -> &'static str {
    match policy {
        RelayPolicy::HandshakeOnly => "handshake only",
        RelayPolicy::OwnRelays => "own relays",
        RelayPolicy::Always => "always",
        RelayPolicy::Off => "off",
    }
}
