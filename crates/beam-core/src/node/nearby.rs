use std::collections::HashMap;
use std::time::Duration;

use futures::StreamExt;
use iroh::{EndpointAddr, EndpointId};
use iroh_mdns_address_lookup::DiscoveryEvent;

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
        let Some(mdns) = &self.shared.mdns else {
            return Vec::new();
        };
        let mut events = mdns.subscribe().await;
        let mut found: HashMap<EndpointId, NearbyDevice> = HashMap::new();
        let collect = async {
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
                    _ => {}
                }
            }
        };
        let _ = tokio::time::timeout(window, collect).await;
        found.into_values().collect()
    }
}
