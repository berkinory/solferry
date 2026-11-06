use std::path::{Path, PathBuf};

use iroh::EndpointId;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::store;

const DEVICES_FILE: &str = "devices.json";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Relationship {
    Own,
    Contact,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Device {
    pub id: EndpointId,
    pub name: String,
    pub relationship: Relationship,
    pub relays: Vec<String>,
    pub paired_at_ms: u64,
}

#[derive(Debug)]
pub(crate) struct TrustStore {
    path: PathBuf,
    devices: Vec<Device>,
}

impl TrustStore {
    pub(crate) fn load(data_dir: &Path) -> Result<Self> {
        let path = data_dir.join(DEVICES_FILE);
        let devices = store::read_json(&path)?.unwrap_or_default();
        Ok(Self { path, devices })
    }

    pub(crate) fn all(&self) -> &[Device] {
        &self.devices
    }

    pub(crate) fn get(&self, id: &EndpointId) -> Option<&Device> {
        self.devices.iter().find(|device| device.id == *id)
    }

    pub(crate) fn upsert(&mut self, device: Device) -> Result<()> {
        self.devices.retain(|existing| existing.id != device.id);
        self.devices.push(device);
        store::write_json(&self.path, &self.devices)
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
        let first = matches
            .next()
            .ok_or_else(|| Error::UnknownDevice(query.to_owned()))?;
        if matches.next().is_some() {
            return Err(Error::AmbiguousDevice(query.to_owned()));
        }
        Ok(first)
    }
}
