use rust_i18n::t;
use solferry_core::{Error, QueueOutcome, QueuedSend, about};

use crate::state::{AppState, Tone};

/// What the sender hears when a queued send is over. Expiry was announced when it was queued,
/// and cancelling is the user's own doing, so neither says anything.
pub fn settled_message(entry: &QueuedSend, outcome: &QueueOutcome) -> Option<(Tone, String)> {
    match outcome {
        QueueOutcome::Delivered => Some((
            Tone::Success,
            t!("toast.queue_delivered", title = entry.label, name = entry.peer_name).into_owned(),
        )),
        QueueOutcome::Failed(Error::Cancelled)
        | QueueOutcome::Expired
        | QueueOutcome::Cancelled => None,
        QueueOutcome::Failed(error) => Some((Tone::Error, about(error, &entry.peer_name))),
    }
}

impl AppState {
    pub(crate) fn queue_settled(&mut self, entry: &QueuedSend, outcome: &QueueOutcome) {
        if let Some((tone, title)) = settled_message(entry, outcome) {
            self.toast(tone, title, None);
        }
    }
}
