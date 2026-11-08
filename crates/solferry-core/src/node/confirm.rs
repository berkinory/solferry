use std::time::Duration;

use iroh::endpoint::{Connection, RecvStream, SendStream};
use iroh::{EndpointAddr, EndpointId};
use rand::RngExt;
use solferry_proto::{
    ALPN_PAIR, PairChallenge, PairReply, PairRequest, PairReveal, read_small_frame, write_frame,
};
use tokio::sync::{mpsc, oneshot};

use super::inbound::admits_stranger;
use super::pairing::{
    adopt_relays, complete, now_ms, own_reply, refuse, request_from, store_and_confirm,
};
use super::{Node, Shared};
use crate::error::{Error, Result};
use crate::transfer::paths::clean_label;
use crate::transfer::{remote_failure, within_handshake};
use crate::trust::Device;

const ANSWER_WAIT: Duration = Duration::from_secs(120);
const REQUEST_BUFFER: usize = 4;
const NONCE_LEN: usize = 32;
const CODE_RANGE: u64 = 1_000_000;

/// A nearby device asking to pair. Show `code` and accept only if the other screen shows the same.
#[derive(Debug)]
pub struct PairingRequest {
    pub peer: EndpointId,
    pub peer_name: String,
    pub platform: Option<crate::Platform>,
    pub code: String,
    reply: oneshot::Sender<bool>,
}

impl PairingRequest {
    /// True once the answer can no longer matter: the peer left or the wait ran out.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.reply.is_closed()
    }

    pub fn accept(self) {
        let _ = self.reply.send(true);
    }

    pub fn decline(self) {
        let _ = self.reply.send(false);
    }
}

/// The people comparing screens are the proof. Both sides fix a random value before seeing the
/// other's, so nobody can search for a key that makes the codes of two connections match.
fn pairing_code(a: EndpointId, b: EndpointId, first: &[u8], second: &[u8]) -> String {
    let (low, high) = if a.as_bytes() < b.as_bytes() { (a, b) } else { (b, a) };
    let mut hasher = blake3::Hasher::new_derive_key("solferry pairing code v1");
    hasher.update(low.as_bytes()).update(high.as_bytes()).update(first).update(second);
    let mut value = [0; 8];
    hasher.finalize_xof().fill(&mut value);
    let code = u64::from_le_bytes(value) % CODE_RANGE;
    format!("{:03} {:03}", code / 1000, code % 1000)
}

fn nonce() -> [u8; NONCE_LEN] {
    let mut value = [0; NONCE_LEN];
    rand::rng().fill(&mut value);
    value
}

impl Node {
    /// Receives pairing requests from nearby devices. Without this, such requests are refused.
    #[must_use]
    pub fn pairing_requests(&self) -> mpsc::Receiver<PairingRequest> {
        let (requests, incoming) = mpsc::channel(REQUEST_BUFFER);
        *self.shared.pair_listener() = Some(requests);
        incoming
    }

    /// Pairs with a nearby device without a code to copy. `show_code` receives the digits both
    /// screens must show, and this returns once the other person has answered.
    pub async fn pair_nearby(
        &self,
        target: impl Into<EndpointAddr>,
        show_code: impl FnOnce(&str),
    ) -> Result<Device> {
        let shared = &self.shared;
        let connection = shared
            .endpoint
            .connect(target, ALPN_PAIR)
            .await
            .map_err(|error| Error::Unreachable(error.to_string()))?;
        let (mut send, mut recv) = connection.open_bi().await.map_err(|_| Error::ConnectionLost)?;
        let own_nonce = nonce();
        let commitment = blake3::hash(&own_nonce).as_bytes().to_vec();
        write_frame(&mut send, &PairRequest { commitment, ..request_from(shared, &[]) }).await?;
        let challenge: PairChallenge =
            within_handshake(async { read_small_frame(&mut recv).await.map_err(Into::into) })
                .await
                .map_err(|error| refused(&connection, error))?;
        write_frame(&mut send, &PairReveal { nonce: own_nonce.to_vec() }).await?;
        let peer = connection.remote_id();
        show_code(&pairing_code(self.id(), peer, &own_nonce, &challenge.nonce));

        let answer = tokio::time::timeout(ANSWER_WAIT, read_small_frame::<_, PairReply>(&mut recv));
        let reply = answer.await.map_err(|_| Error::PairingRefused)?;
        let reply = reply.map_err(|error| refused(&connection, error.into()))?;
        if !reply.accepted {
            return Err(Error::PairingRefused);
        }
        adopt_relays(shared, &reply.relay_urls).await;
        let device = Device {
            id: peer,
            name: clean_label(&reply.name),
            auto_accept: false,
            relays: reply.relay_urls,
            direct_addrs: reply.direct_addrs,
            paired_at_ms: now_ms(),
            platform: crate::Platform::parse(&reply.platform),
        };
        store_and_confirm(shared, &connection, send, device).await
    }
}

fn refused(connection: &Connection, error: Error) -> Error {
    match remote_failure(connection, error) {
        Error::NotPaired => Error::PairingRefused,
        other => other,
    }
}

/// Handles a request that came without an invite. It gets the same admission as unpaired files.
pub(super) async fn answer(
    shared: &Shared,
    connection: &Connection,
    mut send: SendStream,
    mut recv: RecvStream,
    request: PairRequest,
) -> Result<()> {
    let peer = connection.remote_id();
    let listener = shared.pair_listener().clone();
    let admitted = !shared.trust().is_blocked(&peer) && admits_stranger(shared, connection);
    let (Some(listener), true) = (listener, admitted) else {
        return refuse(connection, &mut send).await;
    };
    let Ok(_slot) = shared.stranger_slots.clone().try_acquire_owned() else {
        return refuse(connection, &mut send).await;
    };

    let own_nonce = nonce();
    write_frame(&mut send, &PairChallenge { nonce: own_nonce.to_vec() }).await?;
    let reveal: PairReveal =
        within_handshake(async { read_small_frame(&mut recv).await.map_err(Into::into) }).await?;
    if blake3::hash(&reveal.nonce).as_bytes().as_slice() != request.commitment {
        return refuse(connection, &mut send).await;
    }

    let (reply, decision) = oneshot::channel();
    let prompt = PairingRequest {
        peer,
        peer_name: clean_label(&request.name),
        platform: crate::Platform::parse(&request.platform),
        code: pairing_code(peer, shared.endpoint.id(), &reveal.nonce, &own_nonce),
        reply,
    };
    if listener.send(prompt).await.is_err() {
        return refuse(connection, &mut send).await;
    }
    let answer = tokio::select! {
        answer = tokio::time::timeout(ANSWER_WAIT, decision) => answer.ok().and_then(Result::ok),
        _ = connection.closed() => None,
    };
    if answer != Some(true) {
        return refuse(connection, &mut send).await;
    }
    write_frame(&mut send, &own_reply(shared)).await?;
    complete(shared, connection, send, recv, request).await
}

#[cfg(test)]
mod tests {
    use super::pairing_code;
    use iroh::SecretKey;

    #[test]
    fn both_sides_compute_the_same_code_and_any_change_breaks_it() {
        let (a, b) = (SecretKey::generate().public(), SecretKey::generate().public());
        let code = pairing_code(a, b, b"first", b"second");
        assert_eq!(code, pairing_code(b, a, b"first", b"second"));
        assert_ne!(code, pairing_code(a, b, b"other", b"second"));
    }
}
