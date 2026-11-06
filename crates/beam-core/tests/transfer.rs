use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use beam_core::{Config, Error, Event, IncomingOffer, Node, NodeOptions, Relationship, Visibility};
use tempfile::TempDir;
use tokio::sync::mpsc;

struct Peer {
    node: Node,
    offers: mpsc::Receiver<IncomingOffer>,
    data: PathBuf,
    _dir: TempDir,
}

async fn peer(name: &str, visibility: Visibility) -> Peer {
    let dir = TempDir::new().unwrap();
    let data = dir.path().join("data");
    fs::create_dir_all(&data).unwrap();
    let config = Config {
        name: name.to_owned(),
        visibility,
        download_dir: dir.path().join("downloads"),
        ..Config::default()
    };
    config.save(&data).unwrap();
    let (node, offers) = start(&data).await;
    Peer {
        node,
        offers,
        data,
        _dir: dir,
    }
}

async fn start(data: &Path) -> (Node, mpsc::Receiver<IncomingOffer>) {
    Node::start(NodeOptions {
        data_dir: data.to_owned(),
        isolated: true,
    })
    .await
    .unwrap()
}

fn write_random(path: &Path, size: usize) {
    let data: Vec<u8> = (0..size)
        .map(|i| i.wrapping_mul(2_654_435_761).to_le_bytes()[1])
        .collect();
    fs::write(path, data).unwrap();
}

fn accept_all(mut offers: mpsc::Receiver<IncomingOffer>, destination: PathBuf) {
    tokio::spawn(async move {
        while let Some(offer) = offers.recv().await {
            offer.accept(destination.clone());
        }
    });
}

#[tokio::test(flavor = "multi_thread")]
async fn delivers_a_directory_tree_intact() {
    let sender = peer("sender", Visibility::Everyone).await;
    let receiver = peer("receiver", Visibility::Everyone).await;
    let source = TempDir::new().unwrap();
    let root = source.path().join("album");
    fs::create_dir_all(root.join("empty")).unwrap();
    fs::create_dir_all(root.join("nested")).unwrap();
    write_random(&root.join("nested/big.bin"), 5 * 1024 * 1024);
    fs::write(root.join("zero.txt"), b"").unwrap();

    let destination = receiver.data.join("inbox");
    accept_all(receiver.offers, destination.clone());
    let report = sender
        .node
        .send(receiver.node.addr(), vec![root.clone()])
        .await
        .unwrap();

    assert_eq!(report.resumed_bytes, 0);
    assert_eq!(
        fs::read(root.join("nested/big.bin")).unwrap(),
        fs::read(destination.join("album/nested/big.bin")).unwrap()
    );
    assert!(destination.join("album/zero.txt").is_file());
    assert!(destination.join("album/empty").is_dir());
    assert!(!destination.join(".beam-partial").exists());
}

#[tokio::test(flavor = "multi_thread")]
async fn resumes_after_the_receiver_restarts() {
    let sender = peer("sender", Visibility::Everyone).await;
    let mut receiver = peer("receiver", Visibility::Everyone).await;
    let source = TempDir::new().unwrap();
    let file = source.path().join("movie.bin");
    write_random(&file, 64 * 1024 * 1024);
    let destination = receiver.data.join("inbox");

    let mut events = receiver.node.events();
    let receiver_node = receiver.node.clone();
    let offers = std::mem::replace(&mut receiver.offers, mpsc::channel(1).1);
    accept_all(offers, destination.clone());
    let target = receiver.node.addr();
    let first = tokio::spawn({
        let node = sender.node.clone();
        let file = file.clone();
        async move { node.send(target, vec![file]).await }
    });
    wait_for_progress(&mut events, 8 * 1024 * 1024).await;
    first.abort();
    receiver_node.shutdown().await.unwrap();

    let (restarted, offers) = start(&receiver.data).await;
    accept_all(offers, destination.clone());
    let report = sender
        .node
        .send(restarted.addr(), vec![file.clone()])
        .await
        .unwrap();

    assert!(
        report.resumed_bytes > 0,
        "resumed {} bytes",
        report.resumed_bytes
    );
    assert_eq!(
        fs::read(&file).unwrap(),
        fs::read(destination.join("movie.bin")).unwrap()
    );
}

async fn wait_for_progress(events: &mut tokio::sync::broadcast::Receiver<Event>, bytes: u64) {
    let wait = async {
        loop {
            if let Ok(Event::Progress { transferred, .. }) = events.recv().await
                && transferred >= bytes
            {
                return;
            }
        }
    };
    tokio::time::timeout(Duration::from_secs(30), wait)
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn corrupted_partial_data_is_never_delivered() {
    let sender = peer("sender", Visibility::Everyone).await;
    let mut receiver = peer("receiver", Visibility::Everyone).await;
    let source = TempDir::new().unwrap();
    let file = source.path().join("doc.bin");
    write_random(&file, 64 * 1024 * 1024);
    let destination = receiver.data.join("inbox");

    let mut events = receiver.node.events();
    let node = receiver.node.clone();
    let offers = std::mem::replace(&mut receiver.offers, mpsc::channel(1).1);
    accept_all(offers, destination.clone());
    let target = receiver.node.addr();
    let first = tokio::spawn({
        let sender = sender.node.clone();
        let file = file.clone();
        async move { sender.send(target, vec![file]).await }
    });
    wait_for_progress(&mut events, 8 * 1024 * 1024).await;
    first.abort();
    node.shutdown().await.unwrap();

    let staging = fs::read_dir(destination.join(".beam-partial"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap();
    let part = staging.path().join("0");
    let mut bytes = fs::read(&part).unwrap();
    bytes[1024] ^= 0xff;
    fs::write(&part, bytes).unwrap();

    let (restarted, offers) = start(&receiver.data).await;
    accept_all(offers, destination.clone());
    let failed = sender.node.send(restarted.addr(), vec![file.clone()]).await;
    assert!(matches!(failed, Err(Error::HashMismatch(_))), "{failed:?}");
    assert!(!destination.join("doc.bin").exists());

    sender
        .node
        .send(restarted.addr(), vec![file.clone()])
        .await
        .unwrap();
    assert_eq!(
        fs::read(&file).unwrap(),
        fs::read(destination.join("doc.bin")).unwrap()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn known_devices_mode_refuses_strangers_until_paired() {
    let sender = peer("sender", Visibility::Everyone).await;
    let receiver = peer("receiver", Visibility::KnownDevices).await;
    let source = TempDir::new().unwrap();
    let file = source.path().join("note.txt");
    fs::write(&file, b"hello").unwrap();

    let refused = sender
        .node
        .send(receiver.node.addr(), vec![file.clone()])
        .await;
    assert!(matches!(refused, Err(Error::NotPaired)), "{refused:?}");

    let invite = receiver.node.invite(Relationship::Own);
    let device = sender.node.join(invite.code()).await.unwrap();
    assert_eq!(device.name, "receiver");
    assert_eq!(device.relationship, Relationship::Own);
    assert!(
        sender.node.join(invite.code()).await.is_err(),
        "invites are single use"
    );

    let destination = receiver.data.join("inbox");
    accept_all(receiver.offers, destination.clone());
    sender
        .node
        .send(receiver.node.addr(), vec![file])
        .await
        .unwrap();
    assert!(destination.join("note.txt").is_file());
}
