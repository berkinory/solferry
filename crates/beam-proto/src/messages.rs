use prost::{Enumeration, Message};

#[derive(Clone, PartialEq, Message)]
pub struct Hello {
    #[prost(uint32, tag = "1")]
    pub version: u32,
    #[prost(string, tag = "2")]
    pub name: String,
    #[prost(uint64, tag = "3")]
    pub capabilities: u64,
}

#[derive(Clone, PartialEq, Message)]
pub struct Entry {
    #[prost(string, tag = "1")]
    pub path: String,
    #[prost(uint64, tag = "2")]
    pub size: u64,
    #[prost(uint64, tag = "3")]
    pub modified_ms: u64,
    #[prost(bool, tag = "4")]
    pub is_dir: bool,
}

#[derive(Clone, PartialEq, Message)]
pub struct Offer {
    #[prost(bytes = "vec", tag = "1")]
    pub transfer_id: Vec<u8>,
    #[prost(message, repeated, tag = "2")]
    pub entries: Vec<Entry>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
#[repr(i32)]
pub enum Decision {
    Unspecified = 0,
    Accepted = 1,
    Declined = 2,
    NoSpace = 3,
    InvalidOffer = 4,
}

#[derive(Clone, PartialEq, Message)]
pub struct OfferReply {
    #[prost(enumeration = "Decision", tag = "1")]
    pub decision: i32,
    #[prost(uint64, repeated, tag = "2")]
    pub offsets: Vec<u64>,
}

#[derive(Clone, PartialEq, Message)]
pub struct FileStart {
    #[prost(uint32, tag = "1")]
    pub index: u32,
    #[prost(uint64, tag = "2")]
    pub offset: u64,
}

#[derive(Clone, PartialEq, Message)]
pub struct FileResult {
    #[prost(uint32, tag = "1")]
    pub index: u32,
    #[prost(bool, tag = "2")]
    pub ok: bool,
}

#[derive(Clone, PartialEq, Message)]
pub struct Finalized {
    #[prost(bool, tag = "1")]
    pub ok: bool,
}

#[derive(Clone, PartialEq, Message)]
pub struct PairRequest {
    #[prost(bytes = "vec", tag = "1")]
    pub secret: Vec<u8>,
    #[prost(string, tag = "2")]
    pub name: String,
    #[prost(string, repeated, tag = "3")]
    pub relay_urls: Vec<String>,
    #[prost(string, repeated, tag = "4")]
    pub direct_addrs: Vec<String>,
}

#[derive(Clone, PartialEq, Message)]
pub struct PairReply {
    #[prost(bool, tag = "1")]
    pub accepted: bool,
    #[prost(string, tag = "2")]
    pub name: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct Ticket {
    #[prost(bytes = "vec", tag = "1")]
    pub endpoint_id: Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub secret: Vec<u8>,
    #[prost(string, repeated, tag = "3")]
    pub direct_addrs: Vec<String>,
    #[prost(string, repeated, tag = "4")]
    pub relay_urls: Vec<String>,
    #[prost(uint64, tag = "5")]
    pub expires_at_ms: u64,
    #[prost(bool, tag = "6")]
    pub own_device: bool,
}
