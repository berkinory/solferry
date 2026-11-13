use serde_json::{Value, json};
use solferry_core::Error;

struct Advice {
    what: String,
    next: &'static str,
}

pub fn render(error: &Error) -> String {
    let Advice { what, next } = advice(error);
    if next.is_empty() {
        format!("error: {what}")
    } else {
        format!("error: {what}\n  next: {next}")
    }
}

/// The error as JSON fields, for `--json` output. `error` is a stable identifier to branch on.
pub fn fields(error: &Error) -> Value {
    let Advice { what, next } = advice(error);
    let next = if next.is_empty() { Value::Null } else { Value::from(next) };
    json!({ "error": kind(error), "message": what, "next": next })
}

fn kind(error: &Error) -> &'static str {
    match error {
        Error::Storage { .. } => "storage",
        Error::CorruptData { .. } => "corrupt_data",
        Error::NetworkStart(_) => "network_start",
        Error::InvalidRelayUrl(_) => "invalid_relay_url",
        Error::RelayCheckFailed => "relay_check_failed",
        Error::Unreachable(_) => "unreachable",
        Error::NotPaired => "not_paired",
        Error::VersionMismatch { .. } => "version_mismatch",
        Error::Busy => "busy",
        Error::Cancelled => "cancelled",
        Error::Declined => "declined",
        Error::NoSpace => "no_space",
        Error::DeliveryFailed => "delivery_failed",
        Error::InvalidOffer(_) => "invalid_offer",
        Error::SourceChanged(_) => "source_changed",
        Error::UnsupportedName(_) => "unsupported_name",
        Error::NothingToSend => "nothing_to_send",
        Error::HashMismatch(_) => "hash_mismatch",
        Error::ConnectionLost => "connection_lost",
        Error::InvalidTicket => "invalid_ticket",
        Error::TicketExpired => "ticket_expired",
        Error::Update(_) => "update",
        Error::OwnTicket => "own_ticket",
        Error::NoLocalNetwork => "no_local_network",
        Error::PairingRefused => "pairing_refused",
        Error::UnknownDevice(_) => "unknown_device",
        Error::AmbiguousDevice(_) => "ambiguous_device",
        Error::InvalidAddress(_) => "invalid_address",
        Error::NoAnswer(_) => "no_answer",
        Error::NoRecentTarget => "no_recent_target",
        Error::Protocol(_) => "protocol",
    }
}

fn advice(error: &Error) -> Advice {
    let what = error.to_string();
    let next = match error {
        Error::Storage { .. } => "Check the path, its permissions and free disk space.",
        Error::CorruptData { .. } => {
            "Fix or delete that file. Paired devices may need pairing again."
        }
        Error::NetworkStart(_) => "Check that a network interface is up and try again.",
        Error::InvalidRelayUrl(_) => {
            "Paste the full link your relay printed, such as https://relay.example.com#token."
        }
        Error::Unreachable(_) => {
            "Make sure solferry is running there. On another network, use Tailscale or your own relay with `solferry relay add`."
        }
        Error::RelayCheckFailed => "",
        Error::NotPaired => {
            "Pair first: run `solferry pair` there and `solferry join <code>` here."
        }
        Error::VersionMismatch { .. } => "Update solferry on both devices.",
        Error::Busy => "Wait for its current transfers to finish and send again.",
        Error::Cancelled => "Send again to continue from where it stopped.",
        Error::Declined => "The other side said no. Ask them to accept and send again.",
        Error::NoSpace => "Free up space on the receiving device and send again. Progress is kept.",
        Error::DeliveryFailed => "Check the destination folder on the receiving device.",
        Error::InvalidOffer(_) => "The other device sent something unsafe, so it was refused.",
        Error::SourceChanged(_) => "Wait until the file stops changing and send it again.",
        Error::UnsupportedName(_) => "Rename the file to valid Unicode.",
        Error::NothingToSend => "Check the paths you gave. Symbolic links are skipped.",
        Error::HashMismatch(_) => "Send again. The damaged copy was discarded.",
        Error::ConnectionLost => "Try again. Transfers continue where they stopped.",
        Error::InvalidTicket => "Copy the whole code, starting with sf1.",
        Error::Update(_) => "Try again later, or download it from GitHub.",
        Error::TicketExpired => "Codes last 10 minutes. Create a new one with `solferry pair`.",
        Error::OwnTicket => "Run `solferry join` with it on the other device instead.",
        Error::NoLocalNetwork => "Join Wi-Fi or a hotspot, then share again.",
        Error::PairingRefused => "Ask them to accept, or create a new code with `solferry pair`.",
        Error::UnknownDevice(_) => {
            "See `solferry devices` and `solferry nearby` for available names."
        }
        Error::AmbiguousDevice(_) => "Use a longer id prefix from `solferry devices`.",
        Error::InvalidAddress(_) => "Give an IPv4 address such as 192.168.1.20.",
        Error::NoAnswer(_) => {
            "Make sure solferry is running there and visible to everyone. `solferry status` shows its address."
        }
        Error::NoRecentTarget => {
            "Name the device with --to once. After a finished send, --to last picks it again."
        }
        Error::Protocol(_) => "This is a bug. Please report it with the message above.",
    };
    Advice { what, next }
}
