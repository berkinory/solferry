use std::borrow::Cow;

use rust_i18n::t;

use crate::Error;

/// What went wrong and what to do next, in words for people rather than logs.
#[derive(Clone, Debug)]
pub struct Explanation {
    pub title: Cow<'static, str>,
    pub next: Cow<'static, str>,
}

fn say(title: Cow<'static, str>, next: Cow<'static, str>) -> Explanation {
    Explanation { title, next }
}

/// One short line about something that failed with a particular device.
#[must_use]
pub fn about(error: &Error, name: &str) -> String {
    match error {
        Error::Unreachable(_) => t!("about.unreachable", name = name).into_owned(),
        Error::Declined => t!("about.declined", name = name).into_owned(),
        Error::Busy => t!("about.busy", name = name).into_owned(),
        Error::NotPaired => t!("about.not_paired", name = name).into_owned(),
        other => explain(other).title.into_owned(),
    }
}

#[must_use]
pub fn explain(error: &Error) -> Explanation {
    match error {
        Error::Storage { .. } => say(t!("explain.storage.title"), t!("explain.storage.next")),
        Error::CorruptData { .. } => {
            say(t!("explain.corrupt_data.title"), t!("explain.corrupt_data.next"))
        }
        Error::NetworkStart(_) => {
            say(t!("explain.network_start.title"), t!("explain.network_start.next"))
        }
        Error::InvalidRelayUrl(_) => {
            say(t!("explain.invalid_relay_url.title"), t!("explain.invalid_relay_url.next"))
        }
        Error::Unreachable(_) => {
            say(t!("explain.unreachable.title"), t!("explain.unreachable.next"))
        }
        Error::RelayCheckFailed => {
            say(t!("explain.relay_check_failed.title"), t!("explain.relay_check_failed.next"))
        }
        Error::NotPaired => say(t!("explain.not_paired.title"), t!("explain.not_paired.next")),
        Error::VersionMismatch { .. } => {
            say(t!("explain.version_mismatch.title"), t!("explain.version_mismatch.next"))
        }
        Error::Busy => say(t!("explain.busy.title"), t!("explain.busy.next")),
        Error::Cancelled => say(t!("explain.cancelled.title"), t!("explain.cancelled.next")),
        Error::Declined => say(t!("explain.declined.title"), t!("explain.declined.next")),
        Error::NoSpace => say(t!("explain.no_space.title"), t!("explain.no_space.next")),
        Error::DeliveryFailed => {
            say(t!("explain.delivery_failed.title"), t!("explain.delivery_failed.next"))
        }
        Error::InvalidOffer(_) => {
            say(t!("explain.invalid_offer.title"), t!("explain.invalid_offer.next"))
        }
        Error::SourceChanged(_) => {
            say(t!("explain.source_changed.title"), t!("explain.source_changed.next"))
        }
        Error::UnsupportedName(_) => {
            say(t!("explain.unsupported_name.title"), t!("explain.unsupported_name.next"))
        }
        Error::NothingToSend => {
            say(t!("explain.nothing_to_send.title"), t!("explain.nothing_to_send.next"))
        }
        Error::HashMismatch(_) => {
            say(t!("explain.hash_mismatch.title"), t!("explain.hash_mismatch.next"))
        }
        Error::ConnectionLost => {
            say(t!("explain.connection_lost.title"), t!("explain.connection_lost.next"))
        }
        Error::InvalidTicket => {
            say(t!("explain.invalid_ticket.title"), t!("explain.invalid_ticket.next"))
        }
        Error::TicketExpired => {
            say(t!("explain.ticket_expired.title"), t!("explain.ticket_expired.next"))
        }
        Error::Update(_) => say(t!("explain.update.title"), t!("explain.update.next")),
        Error::OwnTicket => say(t!("explain.own_ticket.title"), t!("explain.own_ticket.next")),
        Error::NoLocalNetwork => {
            say(t!("explain.no_local_network.title"), t!("explain.no_local_network.next"))
        }
        Error::PairingRefused => {
            say(t!("explain.pairing_refused.title"), t!("explain.pairing_refused.next"))
        }
        Error::UnknownDevice(_) => {
            say(t!("explain.unknown_device.title"), t!("explain.unknown_device.next"))
        }
        Error::AmbiguousDevice(_) => {
            say(t!("explain.ambiguous_device.title"), t!("explain.ambiguous_device.next"))
        }
        Error::InvalidAddress(_) => {
            say(t!("explain.invalid_address.title"), t!("explain.invalid_address.next"))
        }
        Error::NoAnswer(_) => say(t!("explain.no_answer.title"), t!("explain.no_answer.next")),
        Error::NoRecentTarget => {
            say(t!("explain.no_recent_target.title"), t!("explain.no_recent_target.next"))
        }
        Error::Protocol(_) => say(t!("explain.protocol.title"), t!("explain.protocol.next")),
    }
}
