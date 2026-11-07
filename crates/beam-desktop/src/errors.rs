use beam_core::Error;

pub struct Explanation {
    pub title: &'static str,
    pub next: &'static str,
}

const fn say(title: &'static str, next: &'static str) -> Explanation {
    Explanation { title, next }
}

pub fn explain(error: &Error) -> Explanation {
    match error {
        Error::Storage { .. } => {
            say("Couldn't read or write a file", "Check the folder, permissions and free space.")
        }
        Error::CorruptData { .. } => say(
            "beam's settings are damaged",
            "Delete the damaged file in beam's data folder. You may need to pair devices again.",
        ),
        Error::NetworkStart(_) => {
            say("Networking didn't start", "Check that you are connected, then restart beam.")
        }
        Error::InvalidRelayUrl(_) => say(
            "That relay address isn't valid",
            "Paste the full link your relay printed, like https://relay.example.com#token.",
        ),
        Error::Unreachable(_) => say(
            "Device can't be reached",
            "Open beam there. On another network, use Tailscale or add your own relay.",
        ),
        Error::RelayCheckFailed => say("A relay is not working", "See its status in Settings."),
        Error::NotPaired => {
            say("This device only accepts paired devices", "Pair them first from Add device.")
        }
        Error::VersionMismatch { .. } => {
            say("beam versions don't match", "Update beam on both devices.")
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
            say("Unsafe transfer refused", "The other device offered files beam can't accept.")
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
            say("Connection lost", "Try again. The transfer continues where it stopped.")
        }
        Error::InvalidTicket => {
            say("That code isn't valid", "Paste the whole code, starting with beam1.")
        }
        Error::TicketExpired => {
            say("That code expired", "Codes last 10 minutes. Create a new one.")
        }
        Error::PairingRefused => say("The code was refused", "Codes work once. Create a new one."),
        Error::UnknownDevice(_) => say("Device not found", "It may have been removed."),
        Error::AmbiguousDevice(_) => {
            say("More than one device matches", "Choose the device from the list.")
        }
        Error::Protocol(_) => {
            say("Something went wrong in beam", "This is a bug. Please report it with the details.")
        }
    }
}
