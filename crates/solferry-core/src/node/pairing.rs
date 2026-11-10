use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use data_encoding::BASE32_NOPAD;
use iroh::endpoint::{Connection, RecvStream, SendStream};
use iroh::protocol::{AcceptError, ProtocolHandler};
use iroh::{EndpointAddr, EndpointId, TransportAddr};
use prost::Message;
use rand::Rng;
use solferry_proto::{
    ALPN_PAIR, Finalized, PairReply, PairRequest, Ticket, close, read_small_frame, write_frame,
};

use super::confirm::answer;
use super::{Node, Shared};
use crate::Platform;
use crate::error::{Error, Result};
use crate::event::Event;
use crate::platform::current_code;
use crate::relay::RelayLink;
use crate::route::shareable_addrs;
use crate::transfer::paths::clean_label;
use crate::transfer::{remote_failure, within_handshake};
use crate::trust::Device;

const INVITE_PREFIX: &str = "sf1";
const INVITE_LIFETIME: Duration = Duration::from_secs(600);
const ONLINE_WAIT: Duration = Duration::from_secs(5);
const CONFIRM_WAIT: Duration = Duration::from_secs(30);
const REFUSAL_WAIT: Duration = Duration::from_secs(5);

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

pub(super) fn now_ms() -> u64 {
    let elapsed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
}

fn secret_key(secret: &[u8]) -> Option<[u8; 32]> {
    secret.try_into().ok().map(|bytes: [u8; 32]| *blake3::hash(&bytes).as_bytes())
}

impl Node {
    /// Creates a single-use invite that a QR code or chat message can carry to another device.
    pub fn invite(&self) -> Invite {
        let mut secret = [0; 32];
        rand::rng().fill_bytes(&mut secret);
        let expires_at_ms = now_ms() + u64::try_from(INVITE_LIFETIME.as_millis()).unwrap_or(0);
        let mut invites =
            self.shared.invites.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        invites.retain(|_, expires_at_ms| *expires_at_ms >= now_ms());
        invites.insert(*blake3::hash(&secret).as_bytes(), expires_at_ms);
        drop(invites);

        let addr = self.addr();
        let ticket = Ticket {
            endpoint_id: addr.id.as_bytes().to_vec(),
            secret: secret.to_vec(),
            direct_addrs: shareable_addrs(addr.ip_addrs().copied()),
            relay_urls: relay_links(&self.shared, &[]),
            expires_at_ms,
        };
        let code = format!(
            "{INVITE_PREFIX}{}",
            BASE32_NOPAD.encode(&ticket.encode_to_vec()).to_lowercase()
        );
        Invite { code, expires_at_ms }
    }

    /// Redeems an invite from another device and stores the resulting trust on both sides.
    pub async fn join(&self, code: &str) -> Result<Device> {
        let ticket = decode_ticket(code)?;
        if ticket.expires_at_ms < now_ms() {
            return Err(Error::TicketExpired);
        }
        if ticket.endpoint_id.as_slice() == self.id().as_bytes() {
            return Err(Error::OwnTicket);
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
        let request = PairRequest { secret: ticket.secret, ..request_from(&self.shared, &relays) };
        write_frame(&mut send, &request).await?;
        let reply: PairReply = match read_small_frame(&mut recv).await {
            Ok(reply) => reply,
            Err(error) => {
                return Err(match remote_failure(&connection, error.into()) {
                    Error::NotPaired => Error::PairingRefused,
                    other => other,
                });
            }
        };
        if !reply.accepted {
            return Err(Error::PairingRefused);
        }
        let device = Device {
            id,
            name: clean_label(&reply.name),
            auto_accept: false,
            relays,
            direct_addrs,
            paired_at_ms: now_ms(),
            platform: Platform::parse(&reply.platform),
        };
        store_and_confirm(&self.shared, &connection, send, device).await
    }
}

/// Starts using a relay the other device waits on, so nobody has to configure it by hand.
pub(super) async fn adopt_relays(shared: &Shared, links: &[String]) {
    for link in links.iter().filter_map(|link| RelayLink::parse(link).ok()) {
        shared.endpoint.insert_relay(link.url().clone(), Arc::new(link.config())).await;
    }
}

/// Relays this device waits on, with the access tokens another device needs to use them.
fn relay_links(shared: &Shared, extra: &[String]) -> Vec<String> {
    let trust = shared.trust();
    let stored = trust.all().iter().flat_map(|device| &device.relays);
    let known: Vec<&String> = shared.config.relays.iter().chain(stored).chain(extra).collect();
    let link_for = |url: &iroh::RelayUrl| {
        let matches = |link: &&&String| RelayLink::parse(link).is_ok_and(|link| link.url() == url);
        known.iter().find(matches).map_or_else(|| url.to_string(), |link| (**link).clone())
    };
    shared.endpoint.addr().relay_urls().map(link_for).collect()
}

/// This device's half of a pairing request. `adopted` lists relays it just took from an invite.
pub(super) fn request_from(shared: &Shared, adopted: &[String]) -> PairRequest {
    let mut relay_urls = relay_links(shared, adopted);
    relay_urls.extend(adopted.iter().cloned());
    relay_urls.sort();
    relay_urls.dedup();
    PairRequest {
        name: shared.config.name.clone(),
        relay_urls,
        direct_addrs: shareable_addrs(shared.endpoint.addr().ip_addrs().copied()),
        platform: current_code(),
        ..PairRequest::default()
    }
}

pub(super) fn own_reply(shared: &Shared) -> PairReply {
    PairReply {
        accepted: true,
        name: shared.config.name.clone(),
        relay_urls: relay_links(shared, &[]),
        direct_addrs: shareable_addrs(shared.endpoint.addr().ip_addrs().copied()),
        platform: current_code(),
    }
}

/// The joining side stores first and tells the other side, which stores and hangs up.
pub(super) async fn store_and_confirm(
    shared: &Shared,
    connection: &Connection,
    mut send: SendStream,
    device: Device,
) -> Result<Device> {
    shared.trust().upsert(device.clone())?;
    write_frame(&mut send, &Finalized { ok: true }).await?;
    let _ = tokio::time::timeout(CONFIRM_WAIT, connection.closed()).await;
    Ok(device)
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
    if !shared.has_invites() && shared.pair_listener().is_none() {
        connection.close(close::NOT_PAIRED.into(), b"no invite");
        return Err(Error::PairingRefused);
    }
    let (mut send, recv, request) = within_handshake(async {
        let (send, mut recv) = connection.accept_bi().await.map_err(|_| Error::ConnectionLost)?;
        let request: PairRequest = read_small_frame(&mut recv).await?;
        Ok((send, recv, request))
    })
    .await?;
    if request.secret.is_empty() {
        return answer(shared, connection, send, recv, request).await;
    }
    let expires_at_ms = secret_key(&request.secret).and_then(|key| {
        shared.invites.lock().unwrap_or_else(std::sync::PoisonError::into_inner).remove(&key)
    });
    if expires_at_ms.is_none_or(|expires_at_ms| expires_at_ms < now_ms()) {
        return refuse(connection, &mut send).await;
    }
    write_frame(&mut send, &own_reply(shared)).await?;
    complete(shared, connection, send, recv, request).await
}

/// Says no and waits for the other side to read it, so it sees a refusal and not a lost connection.
pub(super) async fn refuse(connection: &Connection, send: &mut SendStream) -> Result<()> {
    write_frame(send, &PairReply::default()).await?;
    let _ = send.finish();
    let _ = tokio::time::timeout(REFUSAL_WAIT, connection.closed()).await;
    Err(Error::PairingRefused)
}

/// Waits for the other side to store the pairing, then stores it here too.
pub(super) async fn complete(
    shared: &Shared,
    connection: &Connection,
    _send: SendStream,
    mut recv: RecvStream,
    request: PairRequest,
) -> Result<()> {
    let confirmed =
        tokio::time::timeout(CONFIRM_WAIT, read_small_frame::<_, Finalized>(&mut recv)).await;
    if !matches!(confirmed, Ok(Ok(Finalized { ok: true }))) {
        return Err(Error::ConnectionLost);
    }
    let device = Device {
        id: connection.remote_id(),
        name: clean_label(&request.name),
        auto_accept: false,
        relays: request.relay_urls,
        direct_addrs: request.direct_addrs,
        paired_at_ms: now_ms(),
        platform: Platform::parse(&request.platform),
    };
    adopt_relays(shared, &device.relays).await;
    shared.trust().upsert(device.clone())?;
    connection.close(close::DONE.into(), b"paired");
    let _ = shared.events.send(Event::Paired { device });
    Ok(())
}
