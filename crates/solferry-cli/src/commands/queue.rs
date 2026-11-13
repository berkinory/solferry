use std::path::{Path, PathBuf};

use iroh::EndpointId;
use serde_json::json;
use solferry_core::{Error, Node, QUEUE_DAYS, QueuedFiles, QueuedSend, Result};

/// Queued sends are named by the start of their id, like transfers.
pub fn short_id(id: u64) -> String {
    format!("{id:016x}")
}

/// Keeps a send for later. Paths are made absolute, since `receive` may run somewhere else.
pub fn add(node: &Node, peer: EndpointId, paths: &[PathBuf]) -> Result<QueuedSend> {
    let absolute = paths.iter().map(|path| absolute(path)).collect::<Result<Vec<_>>>()?;
    node.queue(peer, QueuedFiles::Paths(absolute))
}

fn absolute(path: &Path) -> Result<PathBuf> {
    std::path::absolute(path)
        .map_err(|source| Error::Storage { path: path.to_owned(), source: source.into() })
}

pub fn show_queued(target: &str, queued: &QueuedSend, json: bool) {
    if json {
        let event = json!({
            "event": "queued",
            "target": target,
            "queue_id": short_id(queued.id),
            "peer": queued.peer.to_z32(),
            "expires_ms": queued.expires_ms(),
        });
        println!("{event}");
    } else {
        println!(
            "{target} can't be reached, so the send is queued as {}. `solferry receive` \
             delivers it when {target} is back, for up to {QUEUE_DAYS} days.",
            short_id(queued.id)
        );
    }
}

pub async fn run(data_dir: PathBuf, cancel: Option<&str>, json: bool) -> Result<()> {
    let (node, _offers) = super::start(data_dir).await?;
    if let Some(wanted) = cancel {
        let found = node.queued().into_iter().find(|entry| short_id(entry.id).starts_with(wanted));
        let Some(entry) = found else {
            node.shutdown().await?;
            let message = format!("no queued send has the id {wanted}");
            if json {
                let next = "See `solferry queue` for the ids.";
                eprintln!(
                    "{}",
                    json!({ "error": "unknown_queue_id", "message": message, "next": next })
                );
            } else {
                eprintln!("{message}. See `solferry queue` for the ids.");
            }
            std::process::exit(1);
        };
        node.unqueue(entry.id);
        if json {
            println!("{}", json!({ "action": "unqueue", "queue_id": short_id(entry.id) }));
        } else {
            println!("Took back {} for {}", entry.label, entry.peer_name);
        }
        return node.shutdown().await;
    }
    let queued = node.queued();
    if json {
        let entries: Vec<_> = queued
            .iter()
            .map(|entry| {
                json!({
                    "queue_id": short_id(entry.id),
                    "peer": entry.peer.to_z32(),
                    "peer_name": entry.peer_name,
                    "label": entry.label,
                    "queued_ms": entry.queued_ms,
                    "expires_ms": entry.expires_ms(),
                })
            })
            .collect();
        println!("{}", json!({ "queued": entries }));
    } else if queued.is_empty() {
        println!("Nothing is queued.");
    }
    if !json {
        for entry in &queued {
            println!("{}  {:<24} {}", short_id(entry.id), entry.peer_name, entry.label);
        }
    }
    node.shutdown().await
}
