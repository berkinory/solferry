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
        Error::TicketExpired => "Codes last 10 minutes. Create a new one with `solferry pair`.",
        Error::PairingRefused => "Codes work once. Create a new one with `solferry pair`.",
        Error::UnknownDevice(_) => {
            "See `solferry devices` and `solferry nearby` for available names."
        }
        Error::AmbiguousDevice(_) => "Use a longer id prefix from `solferry devices`.",
        Error::Protocol(_) => "This is a bug. Please report it with the message above.",
    };
    Advice { what, next }
}
