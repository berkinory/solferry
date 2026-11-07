use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use beam_proto::{
    ALPN_PAIR, Finalized, PairReply, PairRequest, Ticket, close, read_frame, write_frame,
};
use data_encoding::BASE32_NOPAD;
use iroh::endpoint::Connection;
use iroh::protocol::{AcceptError, ProtocolHandler};
use iroh::{EndpointAddr, EndpointId, TransportAddr};
use prost::Message;
use rand::Rng;

use super::{Node, Shared};
use crate::error::{Error, Result};
use crate::event::Event;
use crate::relay::RelayLink;
use crate::transfer::paths::clean_label;
use crate::trust::Device;

const INVITE_PREFIX: &str = "beam1";
const INVITE_LIFETIME: Duration = Duration::from_secs(600);
const ONLINE_WAIT: Duration = Duration::from_secs(5);
const CONFIRM_WAIT: Duration = Duration::from_secs(30);

#[derive(Clone, Debug)]
pub struct Invite {
    code: String,
    pub expires_at_ms: u64,
}

impl Invite {
    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PendingInvite {
    auto_accept: bool,
    expires_at_ms: u64,
}

fn now_ms() -> u64 {
    let elapsed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
}

fn secret_key(secret: &[u8]) -> Option<[u8; 32]> {
    secret.try_into().ok().map(|bytes: [u8; 32]| *blake3::hash(&bytes).as_bytes())
}

impl Node {
    /// Creates a single-use invite that a QR code or chat message can carry to another device.
    /// `auto_accept` applies to this side only, the other device makes its own choice.
    pub fn invite(&self, auto_accept: bool) -> Invite {
        let mut secret = [0; 32];
        rand::rng().fill_bytes(&mut secret);
        let expires_at_ms = now_ms() + u64::try_from(INVITE_LIFETIME.as_millis()).unwrap_or(0);
        let pending = PendingInvite { auto_accept, expires_at_ms };
        let mut invites =
            self.shared.invites.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        invites.retain(|_, invite| invite.expires_at_ms >= now_ms());
        invites.insert(*blake3::hash(&secret).as_bytes(), pending);
        drop(invites);

        let addr = self.addr();
        let ticket = Ticket {
            endpoint_id: addr.id.as_bytes().to_vec(),
            secret: secret.to_vec(),
            direct_addrs: addr.ip_addrs().map(ToString::to_string).collect(),
            relay_urls: self.relay_links(&[]),
            expires_at_ms,
        };
        let code = format!(
            "{INVITE_PREFIX}{}",
            BASE32_NOPAD.encode(&ticket.encode_to_vec()).to_lowercase()
        );
        Invite { code, expires_at_ms }
    }

    /// Relays this device waits on, with the access tokens another device needs to use them.
    fn relay_links(&self, extra: &[String]) -> Vec<String> {
        let trust = self.shared.trust();
        let shared = trust.all().iter().flat_map(|device| &device.relays);
        let known: Vec<&String> = self.config().relays.iter().chain(shared).chain(extra).collect();
        let link_for = |url: &iroh::RelayUrl| {
            let matches =
                |link: &&&String| RelayLink::parse(link).is_ok_and(|link| link.url() == url);
            known.iter().find(matches).map_or_else(|| url.to_string(), |link| (**link).clone())
        };
        self.addr().relay_urls().map(link_for).collect()
    }

    /// Redeems an invite from another device and stores the resulting trust on both sides.
    pub async fn join(&self, code: &str, auto_accept: bool) -> Result<Device> {
        let ticket = decode_ticket(code)?;
        if ticket.expires_at_ms < now_ms() {
            return Err(Error::TicketExpired);
        }
        self.online_within(ONLINE_WAIT).await;
        let relays = ticket.relay_urls.clone();
        let direct_addrs = ticket.direct_addrs.clone();
        adopt_relays(&self.shared, &relays).await;
        let addr = ticket_addr(&ticket)?;
        let id = addr.id;
        let connection = self
            .shared
            .endpoint
            .connect(addr, ALPN_PAIR)
            .await
            .map_err(|error| Error::Unreachable(error.to_string()))?;
        let (mut send, mut recv) = connection.open_bi().await.map_err(|_| Error::ConnectionLost)?;
        let mut relay_urls = self.relay_links(&relays);
        relay_urls.extend(relays.iter().cloned());
        relay_urls.sort();
        relay_urls.dedup();
        let request = PairRequest {
            secret: ticket.secret,
            name: self.config().name.clone(),
            relay_urls,
            direct_addrs: self.addr().ip_addrs().map(ToString::to_string).collect(),
        };
        write_frame(&mut send, &request).await?;
        let reply: PairReply = read_frame(&mut recv).await?;
        if !reply.accepted {
            return Err(Error::PairingRefused);
        }
        let device = Device {
            id,
            name: clean_label(&reply.name),
            auto_accept,
            relays,
            direct_addrs,
            paired_at_ms: now_ms(),
        };
        self.shared.trust().upsert(device.clone())?;
        write_frame(&mut send, &Finalized { ok: true }).await?;
        let _ = tokio::time::timeout(CONFIRM_WAIT, connection.closed()).await;
        Ok(device)
    }
}

/// Starts using a relay the other device waits on, so nobody has to configure it by hand.
async fn adopt_relays(shared: &Shared, links: &[String]) {
    for link in links.iter().filter_map(|link| RelayLink::parse(link).ok()) {
        shared.endpoint.insert_relay(link.url().clone(), Arc::new(link.config())).await;
    }
}

fn decode_ticket(code: &str) -> Result<Ticket> {
    let body = code.trim().strip_prefix(INVITE_PREFIX).ok_or(Error::InvalidTicket)?;
    let bytes =
        BASE32_NOPAD.decode(body.to_uppercase().as_bytes()).map_err(|_| Error::InvalidTicket)?;
    Ticket::decode(bytes.as_slice()).map_err(|_| Error::InvalidTicket)
}

fn ticket_addr(ticket: &Ticket) -> Result<EndpointAddr> {
    let id: [u8; 32] =
        ticket.endpoint_id.as_slice().try_into().map_err(|_| Error::InvalidTicket)?;
    let id = EndpointId::from_bytes(&id).map_err(|_| Error::InvalidTicket)?;
    let ips =
        ticket.direct_addrs.iter().filter_map(|addr| addr.parse().ok()).map(TransportAddr::Ip);
    let relays = ticket
        .relay_urls
        .iter()
        .filter_map(|link| RelayLink::parse(link).ok())
        .map(|link| TransportAddr::Relay(link.url().clone()));
    Ok(EndpointAddr::from_parts(id, ips.chain(relays)))
}

#[derive(Debug, Clone)]
pub(crate) struct PairHandler(pub(crate) Arc<Shared>);

impl ProtocolHandler for PairHandler {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        if let Err(error) = redeem(&self.0, &connection).await {
            tracing::warn!(%error, "pairing failed");
        }
        Ok(())
    }
}

async fn redeem(shared: &Shared, connection: &Connection) -> Result<()> {
    let (mut send, mut recv) = connection.accept_bi().await.map_err(|_| Error::ConnectionLost)?;
    let request: PairRequest = read_frame(&mut recv).await?;
    let pending = secret_key(&request.secret).and_then(|key| {
        shared.invites.lock().unwrap_or_else(std::sync::PoisonError::into_inner).remove(&key)
    });
    let Some(pending) = pending.filter(|invite| invite.expires_at_ms >= now_ms()) else {
        write_frame(&mut send, &PairReply { accepted: false, name: String::new() }).await?;
        return Err(Error::PairingRefused);
    };
    let reply = PairReply { accepted: true, name: shared.config.name.clone() };
    write_frame(&mut send, &reply).await?;
    let confirmed = tokio::time::timeout(CONFIRM_WAIT, read_frame::<_, Finalized>(&mut recv)).await;
    if !matches!(confirmed, Ok(Ok(Finalized { ok: true }))) {
        return Err(Error::ConnectionLost);
    }
    let device = Device {
        id: connection.remote_id(),
        name: clean_label(&request.name),
        auto_accept: pending.auto_accept,
        relays: request.relay_urls,
        direct_addrs: request.direct_addrs,
        paired_at_ms: now_ms(),
    };
    adopt_relays(shared, &device.relays).await;
    shared.trust().upsert(device.clone())?;
    connection.close(close::DONE.into(), b"paired");
    let _ = shared.events.send(Event::Paired { device });
    Ok(())
}
