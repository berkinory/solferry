use beam_core::Error;

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
        Error::InvalidRelayUrl(_) => "Use a full address such as https://relay.example.com.",
        Error::Unreachable(_) => "Make sure beam is running on that device, then try again.",
        Error::NoDirectPath => {
            "These networks block direct connections. Use Tailscale, add your own relay with `beam relay add`, or allow relaying with `beam relay policy always`."
        }
        Error::NotPaired => "Pair first: run `beam pair` there and `beam join <code>` here.",
        Error::VersionMismatch { .. } => "Update beam on both devices.",
        Error::Declined => "The other side said no. Ask them to accept and send again.",
        Error::NoSpace => "Free up space on the receiving device and send again. Progress is kept.",
        Error::InvalidOffer(_) => "The other device sent something unsafe, so it was refused.",
        Error::SourceChanged(_) => "Wait until the file stops changing and send it again.",
        Error::UnsupportedName(_) => "Rename the file to valid Unicode.",
        Error::NothingToSend => "Check the paths you gave. Symbolic links are skipped.",
        Error::HashMismatch(_) => "Send again. The damaged copy was discarded.",
        Error::ConnectionLost => "Send again and the transfer continues where it stopped.",
        Error::InvalidTicket => "Copy the whole code, starting with beam1.",
        Error::TicketExpired => "Codes last 10 minutes. Create a new one with `beam pair`.",
        Error::PairingRefused => "Codes work once. Create a new one with `beam pair`.",
        Error::UnknownDevice(_) => "See `beam devices` and `beam nearby` for available names.",
        Error::AmbiguousDevice(_) => "Use a longer id prefix from `beam devices`.",
        Error::Protocol(_) => "This is a bug. Please report it with the message above.",
    };
    Advice { what, next }
}
