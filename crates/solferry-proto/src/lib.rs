//! Wire format shared by every solferry client. See `docs/protocol.md`.

mod frame;
mod messages;

pub use frame::{FrameError, MAX_FRAME_LEN, read_frame, read_small_frame, write_frame};
pub use messages::{
    Decision, Entry, FileResult, FileStart, Finalized, Hello, Offer, OfferReply, PairChallenge,
    PairReply, PairRequest, PairReveal, SharedText, Ticket,
};

pub const ALPN_TRANSFER: &[u8] = b"solferry/1";
pub const ALPN_PAIR: &[u8] = b"solferry-pair/1";
pub const ALPN_TEXT: &[u8] = b"solferry-text/1";
pub const ALPN_PRESENCE: &[u8] = b"solferry-presence/1";
pub const ALPN_LEAVING: &[u8] = b"solferry-leaving/1";
pub const PROTOCOL_VERSION: u32 = 2;
pub const HASH_LEN: usize = 32;

/// QUIC application close codes.
pub mod close {
    pub const DONE: u32 = 0;
    pub const NOT_PAIRED: u32 = 1;
    pub const VERSION_MISMATCH: u32 = 2;
    pub const INTERNAL: u32 = 3;
    pub const BUSY: u32 = 4;
    pub const CANCELLED: u32 = 5;
}
