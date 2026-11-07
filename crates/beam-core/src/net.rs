use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use iroh::endpoint::{Builder, QuicTransportConfig, presets};
use iroh::tls::CaTlsConfig;
use iroh::{Endpoint, RelayMap, RelayMode, RelayUrl, SecretKey};
use iroh_mainline_address_lookup::DhtAddressLookup;
use iroh_mdns_address_lookup::MdnsAddressLookup;
use noq_proto::congestion::Bbr3Config;
use rustls_pki_types::CertificateDer;
use rustls_pki_types::pem::PemObject;

use crate::config::{Config, RelayPolicy, Visibility};
use crate::error::{Error, Result};

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

pub(crate) async fn bind(secret: SecretKey, config: &Config, isolated: bool) -> Result<Network> {
    let online = !isolated && config.relay_policy != RelayPolicy::Off;
    let endpoint = if isolated {
        configure(secret.clone(), config, online)?.bind().await
    } else {
        let fixed = configure(secret.clone(), config, online)?
            .bind_addr(PREFERRED_ADDR)
            .map_err(|error| Error::NetworkStart(error.to_string()))?;
        match fixed.bind().await {
            Ok(endpoint) => Ok(endpoint),
            Err(_) => configure(secret, config, online)?.bind().await,
        }
    }
    .map_err(|error| Error::NetworkStart(error.to_string()))?;
    let mdns = if isolated { None } else { Some(start_mdns(&endpoint, config)?) };
    Ok(Network { endpoint, mdns })
}

fn configure(secret: SecretKey, config: &Config, online: bool) -> Result<Builder> {
    let builder = if online {
        let builder = Endpoint::builder(presets::N0)
            .relay_mode(relay_mode(config)?)
            .address_lookup(DhtAddressLookup::builder());
        match &config.relay_ca_file {
            Some(path) => builder.ca_tls_config(private_roots(path)?),
            None => builder,
        }
    } else {
        Endpoint::builder(presets::Minimal).relay_mode(RelayMode::Disabled)
    };
    Ok(builder.secret_key(secret).transport_config(transport_config()?))
}

fn private_roots(path: &Path) -> Result<CaTlsConfig> {
    let roots = CertificateDer::pem_file_iter(path)
        .and_then(Iterator::collect::<std::result::Result<Vec<_>, _>>)
        .map_err(|error| Error::NetworkStart(format!("{}: {error}", path.display())))?;
    Ok(CaTlsConfig::default().with_extra_roots(roots))
}

fn relay_mode(config: &Config) -> Result<RelayMode> {
    if config.relays.is_empty() {
        return Ok(RelayMode::Default);
    }
    Ok(RelayMode::Custom(RelayMap::from_iter(parse_relays(&config.relays)?)))
}

pub(crate) fn parse_relays(urls: &[String]) -> Result<Vec<RelayUrl>> {
    urls.iter().map(|url| url.parse().map_err(|_| Error::InvalidRelayUrl(url.clone()))).collect()
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
