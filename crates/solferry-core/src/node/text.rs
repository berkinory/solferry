use std::sync::Arc;
use std::time::Duration;

use iroh::EndpointAddr;
use iroh::endpoint::Connection;
use iroh::protocol::{AcceptError, ProtocolHandler};
use solferry_proto::{ALPN_TEXT, Finalized, SharedText, close, read_frame, write_frame};

use super::{Node, Shared};
use crate::error::{Error, Result};
use crate::event::Event;
use crate::transfer::{exchange_hello, remote_failure, within_handshake};

/// About a book's worth of text. Anything larger belongs in a file.
pub const MAX_TEXT_LEN: usize = 1024 * 1024;
const CLOSE_WAIT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
pub(crate) struct TextHandler(pub(crate) Arc<Shared>);

impl ProtocolHandler for TextHandler {
    async fn accept(&self, connection: Connection) -> std::result::Result<(), AcceptError> {
        if let Err(error) = receive(&self.0, &connection).await {
            tracing::warn!(peer = %connection.remote_id().fmt_short(), %error, "shared text failed");
        }
        Ok(())
    }
}

/// Only paired devices may put text on this clipboard, whatever the visibility setting.
async fn receive(shared: &Shared, connection: &Connection) -> Result<()> {
    let peer = connection.remote_id();
    let blocked = shared.trust().is_blocked(&peer);
    let device = shared.trust().get(&peer).cloned();
    let Some(device) = device.filter(|_| !blocked) else {
        connection.close(close::NOT_PAIRED.into(), b"pair first");
        return Err(Error::NotPaired);
    };
    let (mut send, shared_text) = within_handshake(async {
        let (mut send, mut recv) =
            connection.accept_bi().await.map_err(|_| Error::ConnectionLost)?;
        exchange_hello(&mut send, &mut recv, &shared.config.name).await?;
        let shared_text: SharedText = read_frame(&mut recv).await?;
        Ok((send, shared_text))
    })
    .await?;
    let fits = shared_text.text.len() <= MAX_TEXT_LEN;
    write_frame(&mut send, &Finalized { ok: fits }).await?;
    let _ = send.finish();
    let _ = tokio::time::timeout(CLOSE_WAIT, connection.closed()).await;
    if !fits {
        return Err(Error::Protocol("shared text is too long".to_owned()));
    }
    let _ = shared.events.send(Event::TextReceived {
        peer,
        peer_name: device.name,
        text: shared_text.text,
        auto_copy: device.auto_accept,
    });
    Ok(())
}

impl Node {
    /// Puts text on a paired device's clipboard, or shows it there to copy.
    pub async fn send_text(&self, target: impl Into<EndpointAddr>, text: String) -> Result<()> {
        if text.len() > MAX_TEXT_LEN {
            return Err(Error::Protocol("text is longer than 1 MiB, send it as a file".to_owned()));
        }
        let shared = &self.shared;
        let connection = shared
            .endpoint
            .connect(target.into(), ALPN_TEXT)
            .await
            .map_err(|error| Error::Unreachable(error.to_string()))?;
        let result = async {
            let (mut send, mut recv) =
                connection.open_bi().await.map_err(|_| Error::ConnectionLost)?;
            exchange_hello(&mut send, &mut recv, &shared.config.name).await?;
            write_frame(&mut send, &SharedText { text }).await?;
            let reply: Finalized = read_frame(&mut recv).await?;
            if reply.ok { Ok(()) } else { Err(Error::DeliveryFailed) }
        }
        .await
        .map_err(|error| remote_failure(&connection, error));
        connection.close(close::DONE.into(), b"done");
        result
    }
}
