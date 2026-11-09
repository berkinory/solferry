use std::collections::HashSet;
use std::path::PathBuf;

use iroh::{EndpointAddr, EndpointId};
use solferry_core::{
    Config, Device, Direction, Error, Event, NearbyDevice, OfferSummary, PastTransfer, Platform,
    RelayHealth, TransferId, about, explain,
};

use crate::format;
use crate::service::Update;

const MAX_TEXTS: usize = 3;
const MAX_TOASTS: usize = 3;

#[derive(Clone)]
pub enum Phase {
    Active,
    Reconnecting,
    Done,
    Failed(Error),
}

#[derive(Clone)]
pub struct Transfer {
    pub id: TransferId,
    pub title: String,
    pub peer: String,
    pub direction: Direction,
    pub transferred: u64,
    pub total: u64,
    pub phase: Phase,
}

#[derive(Clone)]
pub struct FailedSend {
    pub name: String,
    pub addr: EndpointAddr,
    pub paths: Vec<PathBuf>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Success,
    Error,
}

/// A short message under the transfer pill, gone after a few seconds.
#[derive(Clone)]
pub struct Toast {
    pub key: u64,
    pub tone: Tone,
    pub title: String,
    pub retry: Option<FailedSend>,
    pub leaving: bool,
}

#[derive(Clone)]
pub struct Offer {
    pub key: u64,
    pub summary: OfferSummary,
}

#[derive(Clone)]
pub struct PairingPrompt {
    pub key: u64,
    pub peer: EndpointId,
    pub platform: Option<Platform>,
    pub name: String,
    pub code: String,
}

#[derive(Clone)]
pub struct SharedText {
    pub key: u64,
    pub peer: EndpointId,
    pub name: String,
    pub text: String,
    pub copied: bool,
}

#[derive(Clone, Default)]
pub struct AppState {
    pub id: Option<EndpointId>,
    pub config: Option<Config>,
    pub devices: Vec<Device>,
    pub blocked: Vec<EndpointId>,
    pub nearby: Vec<NearbyDevice>,
    /// Paired devices that answered the last check. `None` until the first one finishes.
    pub reachable: Option<HashSet<EndpointId>>,
    pub transfers: Vec<Transfer>,
    pub history: Vec<PastTransfer>,
    pub toasts: Vec<Toast>,
    toasts_made: u64,
    /// Goes up each time the last transfer ends, which starts the pill's way out.
    pub pill_settled: u64,
    pub pill_leaving: bool,
    pub offers: Vec<Offer>,
    pub texts: Vec<SharedText>,
    texts_seen: u64,
    pub pairing_prompts: Vec<PairingPrompt>,
    pub pairing_code: Option<String>,
    pub invite: Option<String>,
    pub pairing_error: Option<Error>,
    pub relay_error: Option<Error>,
    pub relay_health: Vec<RelayHealth>,
    pub joining: bool,
    pub paired_count: u32,
    pub fatal: Option<Error>,
}

impl AppState {
    /// Questions waiting for an answer, which only the home page shows.
    pub fn asking(&self) -> usize {
        self.offers.len() + self.pairing_prompts.len()
    }

    pub fn apply(&mut self, update: Update) {
        match update {
            Update::Ready { id, config, devices, blocked } => {
                self.blocked = blocked;
                self.id = Some(id);
                self.config = Some(*config);
                self.devices = devices;
                self.fatal = None;
            }
            Update::StartFailed(error) => self.fatal = Some(error),
            Update::Devices(devices) => {
                for nearby in &mut self.nearby {
                    nearby.known = devices.iter().any(|device| device.id == nearby.addr.id);
                }
                self.devices = devices;
            }
            Update::Blocked(blocked) => self.blocked = blocked,
            Update::RelayHealth(health) => self.relay_health = health,
            Update::Nearby(found) => self.nearby = found,
            Update::Reachable(reachable) => self.reachable = Some(reachable),
            Update::Transfer(event) => self.on_event(event),
            Update::History(history) => self.history = history,
            Update::Offer { key, summary } => self.offers.push(Offer { key, summary }),
            Update::OfferGone(key) => self.offers.retain(|offer| offer.key != key),
            Update::PairingGone(key) => {
                self.pairing_prompts.retain(|prompt| prompt.key != key);
            }
            Update::Invite(code) => self.invite = Some(code),
            Update::PairingRequest { key, peer, platform, name, code } => {
                self.pairing_prompts.push(PairingPrompt { key, peer, platform, name, code });
            }
            Update::PairingCode(code) => self.pairing_code = Some(code),
            Update::Paired(device) => {
                self.pairing_code = None;
                self.add_device(device);
            }
            Update::PairingFailed(error) => {
                self.pairing_code = None;
                if !self.joining {
                    self.toast(Tone::Error, explain(&error).title, None);
                }
                self.joining = false;
                self.pairing_error = Some(error);
            }
            Update::TextShared { name, result } => {
                if let Err(error) = result {
                    self.text_failed(&name, &error);
                }
            }
            Update::SendFailed { name, addr, paths, error } => {
                if !matches!(error, Error::Cancelled) {
                    let title = about(&error, &name);
                    self.toast(Tone::Error, title, Some(FailedSend { name, addr, paths }));
                }
            }
        }
    }

    pub fn toast(&mut self, tone: Tone, title: impl Into<String>, retry: Option<FailedSend>) {
        self.toasts_made += 1;
        let key = self.toasts_made;
        self.toasts.push(Toast { key, tone, title: title.into(), retry, leaving: false });
        if self.toasts.len() > MAX_TOASTS {
            self.toasts.remove(0);
        }
    }

    pub fn text_failed(&mut self, name: &str, error: &Error) {
        if matches!(error, Error::NothingToSend) {
            self.toast(Tone::Error, "Your clipboard has no text", None);
        } else {
            self.toast(Tone::Error, about(error, name), None);
        }
    }

    fn add_device(&mut self, device: Device) {
        self.toast(Tone::Success, format!("Paired with {}", device.name), None);
        self.invite = None;
        self.pairing_error = None;
        self.joining = false;
        self.paired_count += 1;
        self.devices.retain(|known| known.id != device.id);
        if let Some(reachable) = &mut self.reachable {
            reachable.insert(device.id);
        }
        for nearby in &mut self.nearby {
            nearby.known |= nearby.addr.id == device.id;
        }
        self.devices.push(device);
    }

    fn on_event(&mut self, event: Event) {
        match event {
            Event::Paired { device } => self.add_device(device),
            Event::TextReceived { peer, peer_name, text, auto_copy } => {
                self.texts_seen += 1;
                let key = self.texts_seen;
                self.texts.retain(|shared| shared.peer != peer);
                self.texts
                    .insert(0, SharedText { key, peer, name: peer_name, text, copied: auto_copy });
                self.texts.truncate(MAX_TEXTS);
            }
            Event::Started { id, peer_name, label, direction, bytes, .. } => {
                self.transfers.retain(|transfer| transfer.id != id && transfer.running());
                self.pill_leaving = false;
                self.pill_settled += 1;
                self.transfers.insert(
                    0,
                    Transfer {
                        id,
                        title: label,
                        peer: peer_name,
                        direction,
                        transferred: 0,
                        total: bytes,
                        phase: Phase::Active,
                    },
                );
            }
            Event::RouteChanged { .. } => {}
            Event::Progress { id, transferred, total } => {
                if let Some(transfer) = self.transfer(id) {
                    transfer.record(transferred, total);
                }
            }
            Event::Reconnecting { id, .. } => {
                if let Some(transfer) = self.transfer(id) {
                    transfer.phase = Phase::Reconnecting;
                }
            }
            Event::Completed { id } => {
                if let Some(transfer) = self.transfer(id) {
                    transfer.transferred = transfer.total;
                    transfer.phase = Phase::Done;
                }
                self.settle(id);
            }
            Event::Failed { id, error } => self.fail(id, error),
        }
    }

    fn fail(&mut self, id: TransferId, error: Error) {
        let Some(transfer) = self.transfer(id) else { return };
        if matches!(error, Error::Cancelled) || transfer.direction == Direction::Sending {
            self.transfers.retain(|transfer| transfer.id != id);
            return;
        }
        transfer.phase = Phase::Failed(error);
        self.settle(id);
    }

    /// The last transfer to end keeps the pill for a moment. One ending beside others moves
    /// to a toast underneath, so the pill stays on what is still running.
    fn settle(&mut self, id: TransferId) {
        if !self.transfers.iter().any(Transfer::running) {
            self.pill_settled += 1;
            return;
        }
        let Some(index) = self.transfers.iter().position(|transfer| transfer.id == id) else {
            return;
        };
        let ended = self.transfers.remove(index);
        let (tone, title) = ended.outcome();
        self.toast(tone, title, None);
    }

    fn transfer(&mut self, id: TransferId) -> Option<&mut Transfer> {
        self.transfers.iter_mut().find(|transfer| transfer.id == id)
    }
}

impl Transfer {
    pub fn running(&self) -> bool {
        matches!(self.phase, Phase::Active | Phase::Reconnecting)
    }

    pub fn outcome(&self) -> (Tone, String) {
        if let Phase::Failed(error) = &self.phase {
            return (Tone::Error, explain(error).title.to_owned());
        }
        let verb = match self.direction {
            Direction::Sending => "Sent",
            Direction::Receiving => "Received",
        };
        (Tone::Success, format!("{verb} {}", self.title))
    }

    fn record(&mut self, transferred: u64, total: u64) {
        self.transferred = transferred;
        self.total = total;
        self.phase = Phase::Active;
    }

    #[expect(clippy::cast_possible_truncation, reason = "a progress ratio fits in f32")]
    pub fn fraction(&self) -> f32 {
        if self.total == 0 {
            return 1.0;
        }
        (format::precise(self.transferred) / format::precise(self.total)) as f32
    }
}
