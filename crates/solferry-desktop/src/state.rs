use std::borrow::Cow;
use std::collections::HashSet;
use std::path::PathBuf;

use iroh::{EndpointAddr, EndpointId};
use rust_i18n::t;
use solferry_core::{
    BlockedDevice, Config, Device, Direction, Error, Event, NearbyDevice, OfferSummary,
    PastTransfer, Platform, RelayHealth, TransferId, about, explain,
};

use crate::format;
use crate::service::Update;
use crate::updater::Status;

const MAX_TOASTS: usize = 3;

#[derive(Clone)]
pub enum Phase {
    Waiting,
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

/// A link a browser can open, with the files and texts browsers sent through it so far.
#[derive(Clone)]
pub struct LinkView {
    pub urls: Vec<String>,
    pub receiving: bool,
    pub received: Vec<String>,
    pub texts: Vec<String>,
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
pub enum Action {
    Retry(FailedSend),
    Restart,
}

#[derive(Clone)]
pub struct Toast {
    pub key: u64,
    pub tone: Tone,
    pub title: String,
    pub action: Option<Action>,
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
    pub name: String,
    pub text: String,
    pub copied: bool,
}

#[derive(Clone, Default)]
pub struct AppState {
    pub id: Option<EndpointId>,
    pub link: Option<LinkView>,
    pub config: Option<Config>,
    pub devices: Vec<Device>,
    pub blocked: Vec<BlockedDevice>,
    pub nearby: Vec<NearbyDevice>,
    /// Paired devices that answered the last check. `None` until the first one finishes.
    pub reachable: Option<HashSet<EndpointId>>,
    pub transfers: Vec<Transfer>,
    pub history: Vec<PastTransfer>,
    pub toasts: Vec<Toast>,
    pub updater: Status,
    toasts_made: u64,
    /// Goes up each time the last transfer ends, which starts the pill's way out.
    pub pill_settled: u64,
    pub pill_leaving: bool,
    pub offers: Vec<Offer>,
    pub text: Option<SharedText>,
    texts_seen: u64,
    pub pairing_prompts: Vec<PairingPrompt>,
    pub pairing_code: Option<String>,
    pub invite: Option<String>,
    pub relay_health: Vec<RelayHealth>,
    pub joining: bool,
    /// Why the last code didn't work, shown under the field it was typed into.
    pub join_error: Option<Cow<'static, str>>,
    pub pairing_address: bool,
    pub address_error: Option<Cow<'static, str>>,
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
            Update::Devices(devices) => self.devices = devices,
            Update::Blocked(blocked) => self.blocked = blocked,
            Update::Updater(status) => {
                if let Status::Ready { version } = &status
                    && self.updater != status
                {
                    let title = t!("toast.update_ready", version = version);
                    self.toast(Tone::Success, title, Some(Action::Restart));
                }
                self.updater = status;
            }
            Update::InstallFailed => {
                self.toast(Tone::Error, t!("toast.install_failed"), None);
            }
            Update::Restarting => {}
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
            Update::LinkReady { urls, receiving } => {
                self.link =
                    Some(LinkView { urls, receiving, received: Vec::new(), texts: Vec::new() });
            }
            Update::LinkStopped => self.link = None,
            Update::LinkFailed(error) => {
                self.link = None;
                self.toast(Tone::Error, explain(&error).title, None);
            }
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
                let title = explain(&error).title;
                if self.joining {
                    self.join_error = Some(title);
                } else if self.pairing_address {
                    self.address_error = Some(title);
                } else {
                    self.toast(Tone::Error, title, None);
                }
                self.joining = false;
                self.pairing_address = false;
            }
            Update::AddressFailed(error) => {
                self.address_error = Some(explain(&error).title);
                self.pairing_address = false;
            }
            Update::TextShared { name, result } => {
                if let Err(error) = result {
                    self.text_failed(&name, &error);
                }
            }
            Update::SendFailed { name, addr, paths, error } => {
                if !matches!(error, Error::Cancelled) {
                    let title = about(&error, &name);
                    self.toast(
                        Tone::Error,
                        title,
                        Some(Action::Retry(FailedSend { name, addr, paths })),
                    );
                }
            }
        }
    }

    pub fn toast(&mut self, tone: Tone, title: impl Into<String>, action: Option<Action>) {
        self.toasts_made += 1;
        let key = self.toasts_made;
        self.toasts.push(Toast { key, tone, title: title.into(), action, leaving: false });
        if self.toasts.len() > MAX_TOASTS {
            self.toasts.remove(0);
        }
    }

    pub fn text_failed(&mut self, name: &str, error: &Error) {
        if matches!(error, Error::NothingToSend) {
            self.toast(Tone::Error, t!("toast.clipboard_empty"), None);
        } else {
            self.toast(Tone::Error, about(error, name), None);
        }
    }

    fn add_device(&mut self, device: Device) {
        self.toast(Tone::Success, t!("toast.paired", name = device.name), None);
        self.invite = None;
        self.joining = false;
        self.join_error = None;
        self.pairing_address = false;
        self.address_error = None;
        self.paired_count += 1;
        self.devices.retain(|known| known.id != device.id);
        if let Some(reachable) = &mut self.reachable {
            reachable.insert(device.id);
        }
        self.devices.push(device);
    }

    fn on_event(&mut self, event: Event) {
        match event {
            Event::Paired { device } => self.add_device(device),
            Event::TextReceived { peer_name, text, auto_copy, .. } => {
                self.texts_seen += 1;
                let key = self.texts_seen;
                self.text = Some(SharedText { key, name: peer_name, text, copied: auto_copy });
            }
            Event::Waiting { id, peer_name, label, bytes, .. } => {
                let transfer = Transfer::new(id, label, peer_name, Direction::Sending, bytes);
                self.begin(Transfer { phase: Phase::Waiting, ..transfer });
            }
            Event::Started { id, peer_name, label, direction, bytes, .. } => {
                self.begin(Transfer::new(id, label, peer_name, direction, bytes));
            }
            Event::RouteChanged { .. } => {}
            Event::LinkReceived { name, .. } => {
                if let Some(link) = &mut self.link {
                    link.received.push(name.clone());
                }
                self.toast(Tone::Success, t!("toast.link_received", name = name), None);
            }
            Event::Present { peer } => {
                self.reachable.get_or_insert_with(HashSet::new).insert(peer);
            }
            Event::Absent { peer } => {
                if let Some(reachable) = &mut self.reachable {
                    reachable.remove(&peer);
                }
            }
            Event::LinkText { text } => {
                if let Some(link) = &mut self.link {
                    link.texts.push(text);
                }
            }
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

    fn begin(&mut self, transfer: Transfer) {
        self.transfers.retain(|known| known.id != transfer.id && known.running());
        self.pill_leaving = false;
        self.pill_settled += 1;
        self.transfers.insert(0, transfer);
    }

    fn transfer(&mut self, id: TransferId) -> Option<&mut Transfer> {
        self.transfers.iter_mut().find(|transfer| transfer.id == id)
    }
}

impl Transfer {
    fn new(id: TransferId, title: String, peer: String, direction: Direction, total: u64) -> Self {
        Self { id, title, peer, direction, transferred: 0, total, phase: Phase::Active }
    }

    pub fn running(&self) -> bool {
        matches!(self.phase, Phase::Waiting | Phase::Active | Phase::Reconnecting)
    }

    pub fn outcome(&self) -> (Tone, String) {
        if let Phase::Failed(error) = &self.phase {
            return (Tone::Error, explain(error).title.into_owned());
        }
        let title = match self.direction {
            Direction::Sending => t!("toast.sent", title = self.title),
            Direction::Receiving => t!("toast.received", title = self.title),
        };
        (Tone::Success, title.into_owned())
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
