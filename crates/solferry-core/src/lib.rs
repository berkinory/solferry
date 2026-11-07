//! Device identity, trust, discovery and file transfer for solferry clients.

mod config;
mod error;
mod event;
mod identity;
mod net;
mod node;
mod platform;
mod relay;
mod route;
mod store;
mod transfer;
mod trust;

pub use config::{Config, Visibility, default_data_dir};
pub use error::{Error, Result};
pub use event::{Direction, Event, TransferId};
pub use node::{
    IncomingOffer, Invite, NearbyDevice, Node, NodeOptions, OfferSummary, PairingRequest,
    RelayHealth, SendReport,
};
pub use platform::Platform;
pub use relay::RelayLink;
pub use route::Route;
pub use trust::Device;
