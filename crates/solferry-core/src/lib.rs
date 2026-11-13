//! Device identity, trust, discovery and file transfer for solferry clients.

mod config;
mod error;
mod event;
mod explain;
mod history;
mod identity;
mod link;
mod locale;
mod net;
mod node;
mod platform;
mod queue;
mod relay;
mod route;
mod store;
mod transfer;
mod trust;
pub mod update;

rust_i18n::i18n!("locales", fallback = "en");

pub use config::{Appearance, Config, RELAY_SPEED_LIMITS, Visibility, default_data_dir};
pub use error::{Error, Result};
pub use event::{Direction, Event, TransferId};
pub use explain::{Explanation, about, explain};
pub use history::PastTransfer;
pub use link::{Link, LinkFile};
pub use locale::set_locale;
pub use node::{
    IncomingOffer, Invite, NearbyDevice, Node, NodeOptions, OfferSummary, PairingRequest,
    RelayHealth, Reopen, SendReport,
};
pub use platform::Platform;
pub use queue::{QUEUE_DAYS, QueueOutcome, QueuedFiles, QueuedSend, SharedSource};
pub use relay::RelayLink;
pub use route::{Route, lan_addresses};
pub use transfer::NamedFile;
pub use trust::{BlockedDevice, Device};
