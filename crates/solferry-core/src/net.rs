use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use iroh::endpoint::{Builder, PortmapperConfig, QuicTransportConfig, presets};
use iroh::tls::CaTlsConfig;
use iroh::{Endpoint, RelayMap, RelayMode, SecretKey};
use iroh_mdns_address_lookup::MdnsAddressLookup;
use noq_proto::congestion::Bbr3Config;
use rustls_pki_types::CertificateDer;
use rustls_pki_types::pem::PemObject;

use crate::config::{Config, Visibility};
use crate::error::{Error, Result};
use crate::relay::RelayLink;

const MIB: u32 = 1024 * 1024;
/// A stable port keeps stored addresses valid across restarts and makes firewall rules possible.
const PREFERRED_ADDR: &str = "0.0.0.0:53318";
/// noq drops the connection past 1024 gaps per stream. Keeping the window under about 2400
/// minimum-size packets makes that limit unreachable, whatever the loss pattern.
const STREAM_WINDOW: u32 = 2 * MIB;
const CONNECTION_WINDOW: u32 = 128 * MIB;
const MAX_FILE_STREAMS: u32 = 64;
const IDLE_TIMEOUT: Duration = Duration::from_secs(30);
const KEEP_ALIVE: Duration = Duration::from_secs(5);
const MDNS_NAME_LIMIT: usize = 200;

#[derive(Debug)]
pub(crate) struct Network {
    pub(crate) endpoint: Endpoint,
    pub(crate) mdns: Option<MdnsAddressLookup>,
}

pub(crate) async fn bind(
    secret: SecretKey,
    config: &Config,
    relays: &[RelayLink],
    isolated: bool,
) -> Result<Network> {
    let endpoint = if isolated {
        configure(secret.clone(), config, relays)?.bind().await
    } else {
        let fixed = configure(secret.clone(), config, relays)?
            .bind_addr(PREFERRED_ADDR)
            .map_err(|error| Error::NetworkStart(error.to_string()))?;
        match fixed.bind().await {
            Ok(endpoint) => Ok(endpoint),
            Err(_) => configure(secret, config, relays)?.bind().await,
        }
    }
    .map_err(|error| Error::NetworkStart(error.to_string()))?;
    let mdns = if isolated { None } else { Some(start_mdns(&endpoint, config)?) };
    Ok(Network { endpoint, mdns })
}

/// A short-lived endpoint that uses one relay and nothing else.
pub(crate) async fn bind_probe(
    secret: SecretKey,
    config: &Config,
    relay: &RelayLink,
) -> Result<Endpoint> {
    configure(secret, config, std::slice::from_ref(relay))?
        .bind()
        .await
        .map_err(|error| Error::NetworkStart(error.to_string()))
}

fn configure(secret: SecretKey, config: &Config, relays: &[RelayLink]) -> Result<Builder> {
    let map: RelayMap = relays.iter().map(RelayLink::config).collect();
    let builder = Endpoint::builder(presets::Minimal).relay_mode(RelayMode::Custom(map));
    let builder = if relays.is_empty() {
        builder.portmapper_config(PortmapperConfig::Disabled)
    } else {
        builder
    };
    let builder = match &config.relay_ca_file {
        Some(path) => builder.ca_tls_config(private_roots(path)?),
        None => builder,
    };
    Ok(builder.secret_key(secret).transport_config(transport_config()?))
}

fn private_roots(path: &Path) -> Result<CaTlsConfig> {
    let roots = CertificateDer::pem_file_iter(path)
        .and_then(Iterator::collect::<std::result::Result<Vec<_>, _>>)
        .map_err(|error| Error::NetworkStart(format!("{}: {error}", path.display())))?;
    Ok(CaTlsConfig::default().with_extra_roots(roots))
}

fn transport_config() -> Result<QuicTransportConfig> {
    let idle = IDLE_TIMEOUT.try_into().map_err(|_| Error::NetworkStart("idle timeout".into()))?;
    Ok(QuicTransportConfig::builder()
        .congestion_controller_factory(Arc::new(Bbr3Config::default()))
        .stream_receive_window(STREAM_WINDOW.into())
        .receive_window(CONNECTION_WINDOW.into())
        .send_window(u64::from(CONNECTION_WINDOW))
        .max_concurrent_uni_streams(MAX_FILE_STREAMS.into())
        .max_idle_timeout(Some(idle))
        .keep_alive_interval(KEEP_ALIVE)
        .build())
}

fn start_mdns(endpoint: &Endpoint, config: &Config) -> Result<MdnsAddressLookup> {
    let name: String = config.name.chars().take(MDNS_NAME_LIMIT).collect();
    if let Ok(user_data) = name.parse() {
        endpoint.set_user_data_for_address_lookup(Some(user_data));
    }
    let mdns = MdnsAddressLookup::builder()
        .advertise(config.visibility != Visibility::Hidden)
        .build(endpoint.id())
        .map_err(|error| Error::NetworkStart(error.to_string()))?;
    let services =
        endpoint.address_lookup().map_err(|error| Error::NetworkStart(error.to_string()))?;
    services.add(mdns.clone());
    Ok(mdns)
}
