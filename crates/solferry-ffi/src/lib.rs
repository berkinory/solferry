//! The bridge mobile apps use: one object that runs the node and reports everything through a
//! listener. Every call returns at once; outcomes arrive as updates.

mod session;
mod types;

use std::path::PathBuf;
use std::sync::Arc;

use iroh::{EndpointAddr, EndpointId};
use solferry_core::{Config, NamedFile, RelayLink, Visibility};
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

use session::{Session, Shelf, blocked, devices, lock};
pub use types::*;

uniffi::setup_scaffolding!();

const CONFIG_FILE: &str = "config.json";

type Edit = Box<dyn FnOnce(&mut Config) + Send>;

/// A QR code as rows of dark and light modules, for the app to draw at any size.
#[derive(Clone, Debug, uniffi::Record)]
pub struct QrMatrix {
    pub width: u32,
    pub dark: Vec<bool>,
}

#[uniffi::export]
#[must_use]
pub fn qr_code(text: String) -> Option<QrMatrix> {
    let code = qrcode::QrCode::new(text.as_bytes()).ok()?;
    let width = u32::try_from(code.width()).ok()?;
    let dark = code.to_colors().into_iter().map(|color| color == qrcode::Color::Dark).collect();
    Some(QrMatrix { width, dark })
}

#[uniffi::export(with_foreign)]
pub trait Listener: Send + Sync {
    fn update(&self, update: Update);
}

/// A file the app opened for sending. The bridge takes ownership of `fd` and closes it when the
/// transfer ends, so the app must detach it rather than close it.
#[derive(Clone, Debug, uniffi::Record)]
pub struct SharedFile {
    pub name: String,
    pub fd: i32,
}

#[derive(Clone, Copy, Debug, uniffi::Enum)]
pub enum Reach {
    Everyone,
    Paired,
    Hidden,
}

#[derive(uniffi::Object)]
pub struct Solferry {
    runtime: Runtime,
    data_dir: PathBuf,
    listener: Arc<dyn Listener>,
    shelf: Arc<Shelf>,
    session: Arc<Mutex<Option<Session>>>,
}

#[uniffi::export]
impl Solferry {
    /// Starts the node. The first start names this device and picks where received files go.
    ///
    /// # Panics
    ///
    /// When the system refuses to start the runtime's threads.
    #[uniffi::constructor]
    pub fn new(
        data_dir: String,
        download_dir: String,
        device_name: String,
        listener: Arc<dyn Listener>,
    ) -> Arc<Self> {
        logging();
        let data_dir = PathBuf::from(data_dir);
        if !data_dir.join(CONFIG_FILE).exists() {
            let config = Config {
                name: device_name,
                download_dir: PathBuf::from(download_dir),
                ..Config::default()
            };
            let _ = std::fs::create_dir_all(&data_dir);
            let _ = config.save(&data_dir);
        }
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("solferry-core")
            .build()
            .expect("the runtime starts");
        let bridge = Arc::new(Self {
            runtime,
            data_dir,
            listener,
            shelf: Arc::new(Shelf::default()),
            session: Arc::new(Mutex::new(None)),
        });
        bridge.restart(None);
        bridge
    }

    pub fn send_files(&self, peer: String, files: Vec<SharedFile>) {
        let named: Vec<NamedFile> = files
            .into_iter()
            .filter_map(|shared| Some(NamedFile { file: adopt(shared.fd)?, name: shared.name }))
            .collect();
        self.with_node(
            move |node, listener, target| async move {
                let Some(target) = target else { return };
                let name = peer_name(&node, &target);
                if let Err(error) = node.send_named(target, named).await {
                    listener.update(Update::SendFailed { name, problem: (&error).into() });
                }
            },
            Some(peer),
        );
    }

    pub fn send_text(&self, peer: String, text: String) {
        self.with_node(
            move |node, listener, target| async move {
                let Some(target) = target else { return };
                let name = peer_name(&node, &target);
                let problem = node.send_text(target, text).await.err().map(|error| (&error).into());
                listener.update(Update::TextShared { name, problem });
            },
            Some(peer),
        );
    }

    pub fn cancel(&self, transfer: String) {
        let id = lock(&self.shelf.transfers).get(&transfer).copied();
        self.with_node(
            move |node, _, _| async move {
                if let Some(id) = id {
                    node.cancel(id);
                }
            },
            None,
        );
    }

    pub fn answer_offer(&self, key: u64, accept: bool) {
        let offer = lock(&self.shelf.offers).remove(&key);
        self.with_node(
            move |node, _, _| async move {
                match (offer, accept) {
                    (Some(offer), true) => offer.accept(node.config().download_dir.clone()),
                    (Some(offer), false) => offer.decline(),
                    (None, _) => {}
                }
            },
            None,
        );
    }

    pub fn accept_always(&self, key: u64) {
        let offer = lock(&self.shelf.offers).remove(&key);
        self.with_node(
            move |node, listener, _| async move {
                if let Some(offer) = offer {
                    let destination = node.config().download_dir.clone();
                    let _ = offer.accept_always(&node, destination);
                    listener.update(devices(&node));
                }
            },
            None,
        );
    }

    pub fn block_offer(&self, key: u64) {
        let offer = lock(&self.shelf.offers).remove(&key);
        self.with_node(
            move |node, listener, _| async move {
                if let Some(offer) = offer {
                    let peer = offer.summary.peer;
                    offer.decline();
                    let _ = node.block(peer);
                    listener.update(devices(&node));
                    listener.update(blocked(&node));
                }
            },
            None,
        );
    }

    pub fn answer_pairing(&self, key: u64, accept: bool) {
        if let Some(request) = lock(&self.shelf.pairings).remove(&key) {
            if accept { request.accept() } else { request.decline() }
        }
    }

    pub fn pair_nearby(&self, peer: String) {
        self.with_node(
            move |node, listener, target| async move {
                let Some(target) = target else { return };
                let shown = listener.clone();
                let show =
                    move |code: &str| shown.update(Update::PairingCode { code: code.to_owned() });
                match node.pair_nearby(target, show).await {
                    Ok(device) => listener.update(Update::Paired { device: (&device).into() }),
                    Err(error) => {
                        listener.update(Update::PairingFailed { problem: (&error).into() });
                    }
                }
                listener.update(devices(&node));
            },
            Some(peer),
        );
    }

    pub fn invite(&self) {
        self.with_node(
            move |node, listener, _| async move {
                node.online_within(std::time::Duration::from_secs(5)).await;
                listener.update(Update::Invite { code: node.invite().code().to_owned() });
            },
            None,
        );
    }

    pub fn join(&self, code: String) {
        self.with_node(
            move |node, listener, _| async move {
                match node.join(&code).await {
                    Ok(device) => listener.update(Update::Paired { device: (&device).into() }),
                    Err(error) => {
                        listener.update(Update::PairingFailed { problem: (&error).into() });
                    }
                }
                listener.update(devices(&node));
            },
            None,
        );
    }

    /// Asks every paired device whether it is there. Apps call this only while on screen.
    pub fn check_presence(&self) {
        self.with_node(
            move |node, listener, _| async move {
                let ids = node.reachable().await.iter().map(device_id).collect();
                listener.update(Update::Reachable { ids });
            },
            None,
        );
    }

    pub fn forget(&self, peer: String) {
        self.with_node(
            move |node, listener, _| async move {
                let _ = node.forget(&peer);
                listener.update(devices(&node));
            },
            None,
        );
    }

    pub fn set_auto_accept(&self, peer: String, enabled: bool) {
        self.with_node(
            move |node, listener, _| async move {
                if let Ok(id) = EndpointId::from_z32(&peer) {
                    let _ = node.set_auto_accept(id, enabled);
                }
                listener.update(devices(&node));
            },
            None,
        );
    }

    pub fn unblock(&self, peer: String) {
        self.with_node(
            move |node, listener, _| async move {
                if let Ok(id) = EndpointId::from_z32(&peer) {
                    let _ = node.unblock(id);
                }
                listener.update(blocked(&node));
            },
            None,
        );
    }

    pub fn add_relay(&self, link: String) {
        if let Err(error) = RelayLink::parse(&link) {
            self.listener.update(Update::RelayFailed { problem: (&error).into() });
            return;
        }
        self.change(move |config| {
            config.relays.retain(|existing| *existing != link);
            config.relays.push(link);
        });
    }

    pub fn remove_relay(&self, link: String) {
        self.change(move |config| config.relays.retain(|existing| *existing != link));
    }

    pub fn rename(&self, name: String) {
        self.change(move |config| config.name = name);
    }

    pub fn set_reach(&self, reach: Reach) {
        self.change(move |config| {
            config.visibility = match reach {
                Reach::Everyone => Visibility::Everyone,
                Reach::Paired => Visibility::KnownDevices,
                Reach::Hidden => Visibility::Hidden,
            };
        });
    }

    pub fn shutdown(&self) {
        let session = self.session.clone();
        self.runtime.block_on(async move {
            if let Some(active) = session.lock().await.take() {
                active.stop().await;
            }
        });
    }
}

impl Solferry {
    /// Runs `work` against the live node, with the address of `peer` when one was named.
    fn with_node<F, W>(&self, work: W, peer: Option<String>)
    where
        W: FnOnce(solferry_core::Node, Arc<dyn Listener>, Option<EndpointAddr>) -> F
            + Send
            + 'static,
        F: Future<Output = ()> + Send + 'static,
    {
        let (current, listener, memory) =
            (self.session.clone(), self.listener.clone(), self.shelf.clone());
        self.runtime.spawn(async move {
            let Some(node) = current.lock().await.as_ref().map(|active| active.node.clone()) else {
                return;
            };
            let target = match peer {
                Some(peer) => {
                    let found = address(&node, &memory, &peer);
                    if found.is_none() {
                        let problem = (&solferry_core::Error::UnknownDevice(peer.clone())).into();
                        listener.update(Update::SendFailed { name: peer, problem });
                        return;
                    }
                    found
                }
                None => None,
            };
            work(node, listener, target).await;
        });
    }

    /// Saves a settings change and restarts the node with it, as the desktop app does.
    fn change(&self, edit: impl FnOnce(&mut Config) + Send + 'static) {
        self.restart(Some(Box::new(edit)));
    }

    fn restart(&self, edit: Option<Edit>) {
        let (current, listener, memory) =
            (self.session.clone(), self.listener.clone(), self.shelf.clone());
        let data_dir = self.data_dir.clone();
        self.runtime.spawn(async move {
            let mut active = current.lock().await;
            if let Some(running) = active.take() {
                running.stop().await;
            }
            if let Some(edit) = edit {
                let mut config = Config::load(&data_dir).unwrap_or_default();
                edit(&mut config);
                let _ = config.save(&data_dir);
            }
            *active = session::start(data_dir, &memory, &listener).await;
        });
    }
}

/// Where to reach a device: the address it was just seen at nearby, else what pairing stored.
fn address(node: &solferry_core::Node, shelf: &Shelf, peer: &str) -> Option<EndpointAddr> {
    let id = EndpointId::from_z32(peer).ok()?;
    if let Some(found) = lock(&shelf.nearby).get(&id) {
        return Some(found.addr.clone());
    }
    node.devices().into_iter().find(|device| device.id == id).map(|device| device.addr())
}

/// Takes ownership of a descriptor the app detached. Only phones hand over descriptors.
#[cfg(unix)]
#[expect(clippy::unnecessary_wraps, reason = "platforms without descriptors return None")]
fn adopt(fd: i32) -> Option<std::fs::File> {
    use std::os::fd::{FromRawFd, OwnedFd};
    // SAFETY: the app detached the descriptor and hands its ownership to the bridge.
    Some(std::fs::File::from(unsafe { OwnedFd::from_raw_fd(fd) }))
}

#[cfg(not(unix))]
fn adopt(_: i32) -> Option<std::fs::File> {
    None
}

fn peer_name(node: &solferry_core::Node, target: &EndpointAddr) -> String {
    node.devices()
        .into_iter()
        .find(|device| device.id == target.id)
        .map_or_else(|| target.id.fmt_short().to_string(), |device| device.name)
}

fn logging() {
    #[cfg(target_os = "android")]
    {
        use tracing_subscriber::filter::Targets;
        use tracing_subscriber::layer::SubscriberExt;
        let quiet = Targets::new()
            .with_default(tracing::Level::WARN)
            .with_target("solferry_core", tracing::Level::INFO)
            .with_target("solferry_ffi", tracing::Level::INFO);
        if let Ok(layer) = tracing_android::layer("Solferry") {
            let subscriber = tracing_subscriber::registry().with(quiet).with(layer);
            let _ = tracing::subscriber::set_global_default(subscriber);
        }
    }
}
