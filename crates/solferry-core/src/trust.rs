use std::path::{Path, PathBuf};

use iroh::{EndpointAddr, EndpointId, TransportAddr};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::relay::RelayLink;
use crate::store;

const DEVICES_FILE: &str = "devices.json";
const BLOCKED_FILE: &str = "blocked.json";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Device {
    pub id: EndpointId,
    pub name: String,
    /// Set on this device only: a pairing code can never grant it.
    pub auto_accept: bool,
    pub relays: Vec<String>,
    /// Addresses seen when pairing, so a device stays reachable without any server.
    #[serde(default)]
    pub direct_addrs: Vec<String>,
    pub paired_at_ms: u64,
    #[serde(default)]
    pub platform: Option<crate::Platform>,
}

impl Device {
    /// Dialable address: the id plus the relays this device told us about.
    #[must_use]
    pub fn addr(&self) -> EndpointAddr {
        let relays = self.relays.iter().filter_map(|link| RelayLink::parse(link).ok());
        let direct = self.direct_addrs.iter().filter_map(|addr| addr.parse().ok());
        let relays = relays.map(|link| TransportAddr::Relay(link.url().clone()));
        let addrs = relays.chain(direct.map(TransportAddr::Ip));
        EndpointAddr::from_parts(self.id, addrs)
    }
}

#[derive(Debug)]
pub(crate) struct TrustStore {
    path: PathBuf,
    devices: Vec<Device>,
    blocked_path: PathBuf,
    blocked: Vec<EndpointId>,
}

impl TrustStore {
    pub(crate) fn load(data_dir: &Path) -> Result<Self> {
        let path = data_dir.join(DEVICES_FILE);
        let devices = store::read_json(&path)?.unwrap_or_default();
        let blocked_path = data_dir.join(BLOCKED_FILE);
        let blocked = store::read_json(&blocked_path)?.unwrap_or_default();
        Ok(Self { path, devices, blocked_path, blocked })
    }

    pub(crate) fn all(&self) -> &[Device] {
        &self.devices
    }

    pub(crate) fn get(&self, id: &EndpointId) -> Option<&Device> {
        self.devices.iter().find(|device| device.id == *id)
    }

    pub(crate) fn upsert(&mut self, device: Device) -> Result<()> {
        self.devices.retain(|existing| existing.id != device.id);
        self.unblock(&device.id)?;
        self.devices.push(device);
        store::write_json(&self.path, &self.devices)
    }

    pub(crate) fn set_auto_accept(&mut self, id: &EndpointId, enabled: bool) -> Result<()> {
        if let Some(device) = self.devices.iter_mut().find(|device| device.id == *id) {
            device.auto_accept = enabled;
            store::write_json(&self.path, &self.devices)?;
        }
        Ok(())
    }

    pub(crate) fn blocked(&self) -> &[EndpointId] {
        &self.blocked
    }

    pub(crate) fn is_blocked(&self, id: &EndpointId) -> bool {
        self.blocked.contains(id)
    }

    pub(crate) fn block(&mut self, id: EndpointId) -> Result<()> {
        self.remove(&id)?;
        if !self.is_blocked(&id) {
            self.blocked.push(id);
            store::write_json(&self.blocked_path, &self.blocked)?;
        }
        Ok(())
    }

    pub(crate) fn unblock(&mut self, id: &EndpointId) -> Result<()> {
        if self.is_blocked(id) {
            self.blocked.retain(|blocked| blocked != id);
            store::write_json(&self.blocked_path, &self.blocked)?;
        }
        Ok(())
    }

    pub(crate) fn remove(&mut self, id: &EndpointId) -> Result<bool> {
        let before = self.devices.len();
        self.devices.retain(|device| device.id != *id);
        let removed = self.devices.len() != before;
        if removed {
            store::write_json(&self.path, &self.devices)?;
        }
        Ok(removed)
    }

    pub(crate) fn resolve(&self, query: &str) -> Result<&Device> {
        let needle = query.to_lowercase();
        let mut matches = self.devices.iter().filter(|device| {
            device.name.to_lowercase() == needle || device.id.to_z32().starts_with(&needle)
        });
        let first = matches.next().ok_or_else(|| Error::UnknownDevice(query.to_owned()))?;
        if matches.next().is_some() {
            return Err(Error::AmbiguousDevice(query.to_owned()));
        }
        Ok(first)
    }
}
