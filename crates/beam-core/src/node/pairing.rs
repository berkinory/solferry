use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use beam_proto::{ALPN_PAIR, PairReply, PairRequest, Ticket, read_frame, write_frame};
use data_encoding::BASE32_NOPAD;
use iroh::endpoint::Connection;
use iroh::protocol::{AcceptError, ProtocolHandler};
use iroh::{EndpointAddr, EndpointId, TransportAddr};
use prost::Message;
use rand::RngCore;

use super::{Node, Shared};
use crate::error::{Error, Result};
use crate::trust::{Device, Relationship};

const INVITE_PREFIX: &str = "beam1";
const INVITE_LIFETIME: Duration = Duration::from_secs(600);

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
    relationship: Relationship,
    expires_at_ms: u64,
}

fn now_ms() -> u64 {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
}

fn secret_key(secret: &[u8]) -> Option<[u8; 32]> {
    secret
        .try_into()
        .ok()
        .map(|bytes: [u8; 32]| *blake3::hash(&bytes).as_bytes())
}

impl Node {
    /// Creates a single-use invite that a QR code or chat message can carry to another device.
    pub fn invite(&self, relationship: Relationship) -> Invite {
        let mut secret = [0; 32];
        rand::rng().fill_bytes(&mut secret);
        let expires_at_ms = now_ms() + u64::try_from(INVITE_LIFETIME.as_millis()).unwrap_or(0);
        let pending = PendingInvite {
            relationship,
            expires_at_ms,
        };
        self.shared
            .invites
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(*blake3::hash(&secret).as_bytes(), pending);

        let addr = self.addr();
        let ticket = Ticket {
            endpoint_id: addr.id.as_bytes().to_vec(),
            secret: secret.to_vec(),
            direct_addrs: addr.ip_addrs().map(ToString::to_string).collect(),
            relay_urls: addr.relay_urls().map(ToString::to_string).collect(),
            expires_at_ms,
            own_device: relationship == Relationship::Own,
        };
        let code = format!(
            "{INVITE_PREFIX}{}",
            BASE32_NOPAD.encode(&ticket.encode_to_vec()).to_lowercase()
        );
        Invite {
            code,
            expires_at_ms,
        }
    }

    /// Redeems an invite from another device and stores the resulting trust on both sides.
    pub async fn join(&self, code: &str) -> Result<Device> {
        let ticket = decode_ticket(code)?;
        if ticket.expires_at_ms < now_ms() {
            return Err(Error::TicketExpired);
        }
        let relays = ticket.relay_urls.clone();
        let relationship = if ticket.own_device {
            Relationship::Own
        } else {
            Relationship::Contact
        };
        let addr = ticket_addr(&ticket)?;
        let id = addr.id;
        let connection = self
            .shared
            .endpoint
            .connect(addr, ALPN_PAIR)
            .await
            .map_err(|error| Error::Unreachable(error.to_string()))?;
        let (mut send, mut recv) = connection
            .open_bi()
            .await
            .map_err(|_| Error::ConnectionLost)?;
        let request = PairRequest {
            secret: ticket.secret,
            name: self.config().name.clone(),
            relay_urls: self.addr().relay_urls().map(ToString::to_string).collect(),
        };
        write_frame(&mut send, &request).await?;
        let reply: PairReply = read_frame(&mut recv).await?;
        connection.close(0u32.into(), b"paired");
        if !reply.accepted {
            return Err(Error::PairingRefused);
        }
        let device = Device {
            id,
            name: reply.name,
            relationship,
            relays,
            paired_at_ms: now_ms(),
        };
        self.shared.trust().upsert(device.clone())?;
        Ok(device)
    }
}

fn decode_ticket(code: &str) -> Result<Ticket> {
    let body = code
        .trim()
        .strip_prefix(INVITE_PREFIX)
        .ok_or(Error::InvalidTicket)?;
    let bytes = BASE32_NOPAD
        .decode(body.to_uppercase().as_bytes())
        .map_err(|_| Error::InvalidTicket)?;
    Ticket::decode(bytes.as_slice()).map_err(|_| Error::InvalidTicket)
}

fn ticket_addr(ticket: &Ticket) -> Result<EndpointAddr> {
    let id: [u8; 32] = ticket
        .endpoint_id
        .as_slice()
        .try_into()
        .map_err(|_| Error::InvalidTicket)?;
    let id = EndpointId::from_bytes(&id).map_err(|_| Error::InvalidTicket)?;
    let ips = ticket
        .direct_addrs
        .iter()
        .filter_map(|addr| addr.parse().ok())
        .map(TransportAddr::Ip);
    let relays = ticket
        .relay_urls
        .iter()
        .filter_map(|url| iroh::RelayUrl::from_str(url).ok())
        .map(TransportAddr::Relay);
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
    let (mut send, mut recv) = connection
        .accept_bi()
        .await
        .map_err(|_| Error::ConnectionLost)?;
    let request: PairRequest = read_frame(&mut recv).await?;
    let pending = secret_key(&request.secret).and_then(|key| {
        shared
            .invites
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&key)
    });
    let Some(pending) = pending.filter(|invite| invite.expires_at_ms >= now_ms()) else {
        write_frame(
            &mut send,
            &PairReply {
                accepted: false,
                name: String::new(),
            },
        )
        .await?;
        return Err(Error::PairingRefused);
    };
    shared.trust().upsert(Device {
        id: connection.remote_id(),
        name: request.name,
        relationship: pending.relationship,
        relays: request.relay_urls,
        paired_at_ms: now_ms(),
    })?;
    let reply = PairReply {
        accepted: true,
        name: shared.config.name.clone(),
    };
    write_frame(&mut send, &reply).await?;
    let _ = tokio::time::timeout(Duration::from_secs(5), connection.closed()).await;
    Ok(())
}
