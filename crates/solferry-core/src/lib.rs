//! Device identity, trust, discovery and file transfer for solferry clients.

mod config;
mod error;
mod event;
mod explain;
mod history;
mod identity;
mod link;
mod net;
mod node;
mod platform;
mod relay;
mod route;
mod store;
mod transfer;
mod trust;
pub mod update;

rust_i18n::i18n!("locales", fallback = "en");

pub use config::{Config, Visibility, default_data_dir};
pub use error::{Error, Result};
pub use event::{Direction, Event, TransferId};
pub use explain::{Explanation, about, explain};
pub use history::PastTransfer;
pub use link::{Link, LinkFile};
pub use node::{
    IncomingOffer, Invite, NearbyDevice, Node, NodeOptions, OfferSummary, PairingRequest,
    RelayHealth, SendReport,
};
pub use platform::Platform;
pub use relay::RelayLink;
pub use route::{Route, lan_addresses};
pub use transfer::NamedFile;
pub use trust::{BlockedDevice, Device};
