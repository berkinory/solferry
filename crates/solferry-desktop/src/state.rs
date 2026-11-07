use std::path::PathBuf;
use std::time::Instant;

use iroh::{EndpointAddr, EndpointId};
use solferry_core::{
    Config, Device, Direction, Error, Event, NearbyDevice, OfferSummary, RelayHealth, TransferId,
};

use crate::format;
use crate::service::Update;

const SPEED_SMOOTHING: f64 = 0.2;

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
    pub route: solferry_core::Route,
    pub transferred: u64,
    pub total: u64,
    pub speed: f64,
    pub phase: Phase,
    measured: Option<(Instant, u64)>,
}

#[derive(Clone)]
pub struct FailedSend {
    pub name: String,
    pub addr: EndpointAddr,
    pub paths: Vec<PathBuf>,
    pub error: Error,
}

#[derive(Clone)]
pub struct Offer {
    pub key: u64,
    pub summary: OfferSummary,
}

#[derive(Clone)]
pub struct PairingPrompt {
    pub key: u64,
    pub name: String,
    pub code: String,
}

#[derive(Clone, Default)]
pub struct AppState {
    pub id: Option<EndpointId>,
    pub config: Option<Config>,
    pub devices: Vec<Device>,
    pub blocked: Vec<EndpointId>,
    pub ask_after_pairing: bool,
    pub nearby: Vec<NearbyDevice>,
    pub transfers: Vec<Transfer>,
    pub failures: Vec<FailedSend>,
    pub offers: Vec<Offer>,
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
            Update::Transfer(event) => self.on_event(event),
            Update::Offer { key, summary } => self.offers.push(Offer { key, summary }),
            Update::OfferGone(key) => self.offers.retain(|offer| offer.key != key),
            Update::PairingGone(key) => {
                self.pairing_prompts.retain(|prompt| prompt.key != key);
            }
            Update::Invite(code) => self.invite = Some(code),
            Update::PairingRequest { key, name, code } => {
                self.pairing_prompts.push(PairingPrompt { key, name, code });
            }
            Update::PairingCode(code) => self.pairing_code = Some(code),
            Update::Paired(device) => {
                self.pairing_code = None;
                self.add_device(device);
            }
            Update::PairingFailed(error) => {
                self.pairing_code = None;
                self.joining = false;
                self.pairing_error = Some(error);
            }
            Update::SendFailed { name, addr, paths, error } => {
                if !matches!(error, Error::Cancelled) {
                    self.failures.push(FailedSend { name, addr, paths, error });
                }
            }
        }
    }

    fn add_device(&mut self, device: Device) {
        self.invite = None;
        self.pairing_error = None;
        self.joining = false;
        self.paired_count += 1;
        self.devices.retain(|known| known.id != device.id);
        for nearby in &mut self.nearby {
            nearby.known |= nearby.addr.id == device.id;
        }
        self.devices.push(device);
    }

    fn on_event(&mut self, event: Event) {
        match event {
            Event::Paired { device } => self.add_device(device),
            Event::Started { id, peer_name, label, direction, route, bytes, .. } => {
                self.transfers.retain(|transfer| transfer.id != id);
                self.transfers.insert(
                    0,
                    Transfer {
                        id,
                        title: label,
                        peer: peer_name,
                        direction,
                        route,
                        transferred: 0,
                        total: bytes,
                        speed: 0.0,
                        phase: Phase::Active,
                        measured: None,
                    },
                );
            }
            Event::RouteChanged { id, route } => {
                if let Some(transfer) = self.transfers.iter_mut().find(|item| item.id == id) {
                    transfer.route = route;
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
            }
            Event::Failed { id, error } => self.fail(id, error),
        }
    }

    fn fail(&mut self, id: TransferId, error: Error) {
        if matches!(error, Error::Cancelled) {
            self.transfers.retain(|transfer| transfer.id != id);
        } else if let Some(transfer) = self.transfer(id) {
            match transfer.direction {
                Direction::Receiving => transfer.phase = Phase::Failed(error),
                Direction::Sending => self.transfers.retain(|transfer| transfer.id != id),
            }
        }
    }

    fn transfer(&mut self, id: TransferId) -> Option<&mut Transfer> {
        self.transfers.iter_mut().find(|transfer| transfer.id == id)
    }
}

impl Transfer {
    pub fn running(&self) -> bool {
        matches!(self.phase, Phase::Active | Phase::Reconnecting)
    }

    fn record(&mut self, transferred: u64, total: u64) {
        let now = Instant::now();
        if let Some((at, before)) = self.measured {
            let elapsed = now.duration_since(at).as_secs_f64();
            if elapsed > 0.0 {
                let sample = format::precise(transferred.saturating_sub(before)) / elapsed;
                self.speed += (sample - self.speed) * SPEED_SMOOTHING;
            }
        }
        self.measured = Some((now, transferred));
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

    pub fn seconds_left(&self) -> Option<f64> {
        (self.speed > 1.0)
            .then(|| format::precise(self.total.saturating_sub(self.transferred)) / self.speed)
    }
}
