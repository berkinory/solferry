use std::collections::HashMap;
use std::time::Duration;

use futures::StreamExt;
use iroh::{EndpointAddr, EndpointId};
use iroh_mdns_address_lookup::DiscoveryEvent;
use tokio::sync::mpsc;

use super::Node;
use crate::transfer::paths::clean_label;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NearbyDevice {
    pub addr: EndpointAddr,
    pub name: Option<String>,
    pub known: bool,
}

impl Node {
    /// Collects devices announced on the local network during `window`.
    pub async fn nearby(&self, window: Duration) -> Vec<NearbyDevice> {
        let mut latest = Vec::new();
        let _ = tokio::time::timeout(window, self.follow_nearby(|found| latest = found)).await;
        latest
    }

    /// Waits for a device with this name, returning as soon as it shows up.
    pub async fn find_nearby(&self, name: &str, window: Duration) -> Option<NearbyDevice> {
        let (found, mut updates) = mpsc::unbounded_channel();
        let follow = self.follow_nearby(move |devices| {
            let _ = found.send(devices);
        });
        let search = async {
            tokio::pin!(follow);
            loop {
                tokio::select! {
                    () = &mut follow => return None,
                    Some(devices) = updates.recv() => {
                        let mut devices = devices.into_iter();
                        let wanted = devices.find(|device| device.name.as_deref() == Some(name));
                        if wanted.is_some() {
                            return wanted;
                        }
                    }
                }
            }
        };
        tokio::time::timeout(window, search).await.ok().flatten()
    }

    /// Calls `changed` with the full list whenever a nearby device appears or disappears.
    /// Never returns while the node is running.
    pub async fn follow_nearby(&self, mut changed: impl FnMut(Vec<NearbyDevice>)) {
        let Some(mdns) = &self.shared.mdns else {
            return;
        };
        let mut events = mdns.subscribe().await;
        let mut found: HashMap<EndpointId, NearbyDevice> = HashMap::new();
        while let Some(event) = events.next().await {
            match event {
                DiscoveryEvent::Discovered { endpoint_info, .. } => {
                    let id = endpoint_info.endpoint_id;
                    let name =
                        endpoint_info.data.user_data().map(|name| clean_label(name.as_ref()));
                    let addr = endpoint_info.into_endpoint_addr();
                    let known = self.shared.trust().get(&id).is_some();
                    found.insert(id, NearbyDevice { addr, name, known });
                }
                DiscoveryEvent::Expired { endpoint_id } => {
                    found.remove(&endpoint_id);
                }
                _ => continue,
            }
            changed(found.values().cloned().collect());
        }
    }
}
