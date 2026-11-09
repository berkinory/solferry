use crate::Error;

/// What went wrong and what to do next, in words for people rather than logs.
#[derive(Clone, Copy, Debug)]
pub struct Explanation {
    pub title: &'static str,
    pub next: &'static str,
}

const fn say(title: &'static str, next: &'static str) -> Explanation {
    Explanation { title, next }
}

/// One short line about something that failed with a particular device.
#[must_use]
pub fn about(error: &Error, name: &str) -> String {
    match error {
        Error::Unreachable(_) => format!("Can't reach {name}"),
        Error::Declined => format!("{name} declined"),
        Error::Busy => format!("{name} is busy"),
        Error::NotPaired => format!("Pair with {name} first"),
        other => explain(other).title.to_owned(),
    }
}

#[must_use]
pub fn explain(error: &Error) -> Explanation {
    match error {
        Error::Storage { .. } => {
            say("Couldn't read or write a file", "Check the folder, permissions and free space.")
        }
        Error::CorruptData { .. } => say(
            "Solferry's settings are damaged",
            "Delete the damaged file in Solferry's data folder. You may need to pair devices again.",
        ),
        Error::NetworkStart(_) => say("Networking didn't start", "Restart Solferry to try again."),
        Error::InvalidRelayUrl(_) => say(
            "That relay address isn't valid",
            "Paste the full link your relay printed, like https://relay.example.com#token.",
        ),
        Error::Unreachable(_) => say(
            "Device can't be reached",
            "Open Solferry there. On another network, use Tailscale or add your own relay.",
        ),
        Error::RelayCheckFailed => say("A relay is not working", "See its status in Settings."),
        Error::NotPaired => {
            say("Pair with this device first", "Only paired devices can reach it from here.")
        }
        Error::VersionMismatch { .. } => {
            say("Solferry versions don't match", "Update Solferry on both devices.")
        }
        Error::Busy => {
            say("The other device is busy", "Wait for its transfers to finish, then try again.")
        }
        Error::Cancelled => say("Transfer cancelled", "Send again to continue where it stopped."),
        Error::Declined => say("Transfer declined", "The other person said no."),
        Error::NoSpace => {
            say("Not enough disk space", "Free up space on the receiving device and try again.")
        }
        Error::DeliveryFailed => {
            say("The files couldn't be saved", "Check the download folder on the receiving device.")
        }
        Error::InvalidOffer(_) => {
            say("Unsafe transfer refused", "The other device offered files Solferry can't accept.")
        }
        Error::SourceChanged(_) => {
            say("A file changed during sending", "Wait until it stops changing and try again.")
        }
        Error::UnsupportedName(_) => {
            say("A file name can't be sent", "Rename the file and try again.")
        }
        Error::NothingToSend => {
            say("Nothing to send", "Choose at least one file. Shortcuts and links are skipped.")
        }
        Error::HashMismatch(_) => {
            say("Damaged data was caught", "The bad copy was discarded. Send it again.")
        }
        Error::ConnectionLost => {
            say("Connection lost", "Try again. Transfers continue where they stopped.")
        }
        Error::InvalidTicket => {
            say("That code isn't valid", "Paste the whole code, starting with sf1.")
        }
        Error::TicketExpired => {
            say("That code expired", "Codes last 10 minutes. Create a new one.")
        }
        Error::PairingRefused => {
            say("Pairing was refused", "Ask them to accept, or create a new code.")
        }
        Error::UnknownDevice(_) => say("Device not found", "It may have been removed."),
        Error::AmbiguousDevice(_) => {
            say("More than one device matches", "Choose the device from the list.")
        }
        Error::Protocol(_) => say(
            "Something went wrong in Solferry",
            "This is a bug. Please report it with the details.",
        ),
    }
}
