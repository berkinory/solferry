use std::sync::Arc;
use std::time::Duration;

use futures::channel::mpsc::UnboundedSender;
use solferry_core::{Event, Node};
use tokio::sync::broadcast::error::RecvError;
use tokio::task::AbortHandle;

use crate::service::{Command, Pending, PendingPairings, Update};

const PROMPT_SWEEP: Duration = Duration::from_secs(1);
const PRESENCE_EVERY: Duration = Duration::from_secs(3);

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
        // A round that waits on an absent device runs late; the next one still waits its turn.
        every.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            every.tick().await;
            let _ = updates.unbounded_send(Update::Reachable(node.reachable().await));
        }
    })
    .abort_handle()
}

/// Restarts the node once it took a relay's new token and nothing is being sent, since a running
/// relay connection keeps the token it started with.
pub fn restart_after_rotation(node: Node, commands: UnboundedSender<Command>) -> AbortHandle {
    tokio::spawn(async move {
        node.wants_restart().await;
        let _ = commands.unbounded_send(Command::Restart);
    })
    .abort_handle()
}

/// Passes the node's events on, with the history again whenever a transfer ends and the queue
/// again whenever it changes.
pub fn forward_events(node: Node, updates: UnboundedSender<Update>) -> AbortHandle {
    let mut events = node.events();
    tokio::spawn(async move {
        loop {
            match events.recv().await {
                Ok(Event::QueueChanged) => {
                    let _ = updates.unbounded_send(Update::Queue(node.queued()));
                }
                Ok(event) => {
                    let over = matches!(event, Event::Completed { .. } | Event::Failed { .. });
                    let _ = updates.unbounded_send(Update::Transfer(event));
                    if over {
                        let _ = updates.unbounded_send(Update::History(node.history()));
                    }
                }
                Err(RecvError::Lagged(_)) => {}
                Err(RecvError::Closed) => break,
            }
        }
    })
    .abort_handle()
}

/// Delivers queued sends while the app runs. The desktop queues paths only, so nothing needs
/// reopening.
pub fn deliver_queued(node: Node) -> AbortHandle {
    tokio::spawn(async move { node.deliver_queued(Arc::new(|_| None)).await }).abort_handle()
}
