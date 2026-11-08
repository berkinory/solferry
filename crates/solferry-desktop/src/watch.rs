use std::time::Duration;

use futures::channel::mpsc::UnboundedSender;
use solferry_core::Node;
use tokio::task::AbortHandle;

use crate::service::{Pending, PendingPairings, Update};

const PROMPT_SWEEP: Duration = Duration::from_secs(1);
const PRESENCE_EVERY: Duration = Duration::from_secs(20);

/// Drops prompts whose peer left or whose wait ran out, so nobody answers a request that is gone.
pub fn sweep_prompts(
    offers: Pending,
    requests: PendingPairings,
    updates: UnboundedSender<Update>,
) -> AbortHandle {
    tokio::spawn(async move {
        let mut sweep = tokio::time::interval(PROMPT_SWEEP);
        loop {
            sweep.tick().await;
            offers.lock().await.retain(|key, offer| {
                let open = !offer.is_closed();
                if !open {
                    let _ = updates.unbounded_send(Update::OfferGone(*key));
                }
                open
            });
            requests.lock().await.retain(|key, request| {
                let open = !request.is_closed();
                if !open {
                    let _ = updates.unbounded_send(Update::PairingGone(*key));
                }
                open
            });
        }
    })
    .abort_handle()
}

/// Keeps asking which paired devices are there, so their rows can say so.
pub fn follow_presence(node: Node, updates: UnboundedSender<Update>) -> AbortHandle {
    tokio::spawn(async move {
        let mut every = tokio::time::interval(PRESENCE_EVERY);
        loop {
            every.tick().await;
            let _ = updates.unbounded_send(Update::Reachable(node.reachable().await));
        }
    })
    .abort_handle()
}
