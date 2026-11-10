use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

use futures::stream::FuturesUnordered;
use futures::{FutureExt, StreamExt};
use iroh::{EndpointAddr, EndpointId};
use iroh_mdns_address_lookup::DiscoveryEvent;
use tokio::sync::{mpsc, watch};

use super::{Node, overlay, probe, subnet};
use crate::error::{Error, Result};
use crate::platform::split_label;
use crate::transfer::paths::clean_label;

const SCAN_EVERY: Duration = Duration::from_secs(20);
/// mDNS usually answers within a second, so the first sweep waits that long before asking every host.
const FIRST_SCAN_AFTER: Duration = Duration::from_millis(1500);
/// A typed address may sit behind a router or a phone saving power, so it gets longer to answer.
const ADDRESS_WAIT: Duration = Duration::from_secs(2);
const ANY: IpAddr = IpAddr::V4(Ipv4Addr::UNSPECIFIED);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NearbyDevice {
    pub addr: EndpointAddr,
    pub name: Option<String>,
    pub known: bool,
    /// Found on an overlay network instead of the local one.
    pub overlay: bool,
    pub platform: Option<crate::Platform>,
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

    /// Asks one IPv4 address whether a solferry device answers there, for a device on another
    /// subnet or on a network that hides it. One that answers joins the nearby list while it keeps
    /// answering.
    pub async fn find_address(&self, address: &str) -> Result<NearbyDevice> {
        let ip: Ipv4Addr =
            address.trim().parse().map_err(|_| Error::InvalidAddress(address.to_owned()))?;
        let found = probe::ask(&self.shared, ANY, &[ip.into()], ADDRESS_WAIT).await;
        let mut device = found.into_iter().next().ok_or(Error::NoAnswer(ip.to_string()))?;
        device.known = self.shared.trust().get(&device.addr.id).is_some();
        self.shared.addresses.send_if_modified(|addresses| {
            let added = !addresses.contains(&ip);
            if added {
                addresses.push(ip);
            }
            added
        });
        Ok(device)
    }

    /// Calls `changed` with the full list whenever a nearby device appears or disappears.
    /// Never returns while the node is running.
    pub async fn follow_nearby(&self, mut changed: impl FnMut(Vec<NearbyDevice>)) {
        let Some(mdns) = &self.shared.mdns else {
            return;
        };
        let mut events = mdns.subscribe().await;
        let mut scan =
            tokio::time::interval_at(tokio::time::Instant::now() + FIRST_SCAN_AFTER, SCAN_EVERY);
        let mut typed = self.shared.addresses.subscribe();
        let mut probing = FuturesUnordered::new();
        let mut announced: HashMap<EndpointId, NearbyDevice> = HashMap::new();
        let mut scanned = Vec::new();
        let mut answered = Vec::new();
        loop {
            tokio::select! {
                event = events.next() => match event {
                    Some(DiscoveryEvent::Discovered { endpoint_info, .. }) => {
                        let id = endpoint_info.endpoint_id;
                        let (platform, name) = endpoint_info.data.user_data().map_or(
                            (None, None),
                            |data| {
                                let (platform, name) = split_label(data.as_ref());
                                (platform, Some(clean_label(name)))
                            },
                        );
                        let addr = endpoint_info.into_endpoint_addr();
                        let device =
                            NearbyDevice { addr, name, known: false, overlay: false, platform };
                        announced.insert(id, device);
                    }
                    Some(DiscoveryEvent::Expired { endpoint_id }) => {
                        announced.remove(&endpoint_id);
                    }
                    Some(_) => continue,
                    None => return,
                },
                _ = scan.tick() => {
                    // Asking every host is only worth it while mDNS finds nothing.
                    let sweep = announced.is_empty();
                    probing.push(self.probe_round(sweep, typed_addresses(&mut typed)).boxed());
                    continue;
                }
                Ok(()) = typed.changed() => {
                    let addresses = typed_addresses(&mut typed);
                    probing.push(async move { Probed::Typed(self.ask_typed(&addresses).await) }.boxed());
                    continue;
                }
                Some(probed) = probing.next() => match probed {
                    Probed::Round { scanned: round, typed } => (scanned, answered) = (round, typed),
                    Probed::Typed(typed) => answered = typed,
                },
            }
            let mut found = announced.clone();
            for device in scanned.iter().chain(&answered) {
                found.entry(device.addr.id).or_insert_with(|| device.clone());
            }
            let trust = self.shared.trust();
            let visible = found.into_values().filter(|device| !trust.is_blocked(&device.addr.id));
            let visible: Vec<NearbyDevice> = visible
                .map(|device| NearbyDevice {
                    known: trust.get(&device.addr.id).is_some(),
                    ..device
                })
                .collect();
            drop(trust);
            changed(visible);
        }
    }

    async fn probe_round(&self, sweep: bool, addresses: Vec<IpAddr>) -> Probed {
        let lan = async { if sweep { subnet::scan(&self.shared).await } else { Vec::new() } };
        let (mut found, lan, typed) =
            tokio::join!(overlay::scan(&self.shared), lan, self.ask_typed(&addresses));
        found.extend(lan);
        Probed::Round { scanned: found, typed }
    }

    async fn ask_typed(&self, addresses: &[IpAddr]) -> Vec<NearbyDevice> {
        if addresses.is_empty() {
            return Vec::new();
        }
        probe::ask(&self.shared, ANY, addresses, probe::ANSWER_WAIT).await
    }
}

/// Devices that answered a unicast question rather than announcing themselves over mDNS.
enum Probed {
    /// The overlay and subnet scans plus the typed addresses, replacing the last round.
    Round {
        scanned: Vec<NearbyDevice>,
        typed: Vec<NearbyDevice>,
    },
    Typed(Vec<NearbyDevice>),
}

fn typed_addresses(typed: &mut watch::Receiver<Vec<Ipv4Addr>>) -> Vec<IpAddr> {
    typed.borrow_and_update().iter().copied().map(IpAddr::V4).collect()
}
