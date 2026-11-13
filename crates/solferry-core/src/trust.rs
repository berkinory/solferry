use std::path::{Path, PathBuf};

use iroh::{EndpointAddr, EndpointId, TransportAddr};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::relay::RelayLink;
use crate::store;

const DEVICES_FILE: &str = "devices.json";
const BLOCKED_FILE: &str = "blocked_devices.json";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Device {
    pub id: EndpointId,
    pub name: String,
    /// Set on this device only, and only from a yes on an actual offer or in the device's options.
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

/// A device refused from now on, under the name it last went by.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockedDevice {
    pub id: EndpointId,
    pub name: Option<String>,
}

#[derive(Debug)]
pub(crate) struct TrustStore {
    path: PathBuf,
    devices: Vec<Device>,
    blocked_path: PathBuf,
    blocked: Vec<BlockedDevice>,
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

    /// Takes new tokens for relays `peer` gave this device, and ignores any other relay. Every
    /// stored copy of such a relay changes with it, since the token belongs to the server, not to
    /// whoever passed it on. Returns the links that changed.
    pub(crate) fn rotate_relays(
        &mut self,
        peer: &EndpointId,
        links: &[String],
    ) -> Result<Vec<String>> {
        let Some(device) = self.get(peer) else { return Ok(Vec::new()) };
        let given: Vec<RelayLink> =
            device.relays.iter().filter_map(|link| RelayLink::parse(link).ok()).collect();
        let rotated: Vec<(RelayLink, &String)> = links
            .iter()
            .filter_map(|text| Some((RelayLink::parse(text).ok()?, text)))
            .filter(|(new, _)| given.iter().any(|old| old.same_server(new) && old != new))
            .collect();
        if rotated.is_empty() {
            return Ok(Vec::new());
        }
        for stored in self.devices.iter_mut().flat_map(|device| device.relays.iter_mut()) {
            let Ok(old) = RelayLink::parse(stored) else { continue };
            if let Some((_, text)) = rotated.iter().find(|(new, _)| new.same_server(&old)) {
                stored.clone_from(text);
            }
        }
        store::write_json(&self.path, &self.devices)?;
        Ok(rotated.into_iter().map(|(_, text)| text.clone()).collect())
    }

    pub(crate) fn blocked(&self) -> &[BlockedDevice] {
        &self.blocked
    }

    pub(crate) fn is_blocked(&self, id: &EndpointId) -> bool {
        self.blocked.iter().any(|blocked| blocked.id == *id)
    }

    pub(crate) fn block(&mut self, id: EndpointId, name: Option<String>) -> Result<()> {
        let name = name.or_else(|| self.get(&id).map(|device| device.name.clone()));
        self.remove(&id)?;
        if !self.is_blocked(&id) {
            self.blocked.push(BlockedDevice { id, name });
            store::write_json(&self.blocked_path, &self.blocked)?;
        }
        Ok(())
    }

    pub(crate) fn unblock(&mut self, id: &EndpointId) -> Result<()> {
        if self.is_blocked(id) {
            self.blocked.retain(|blocked| blocked.id != *id);
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
