use std::path::PathBuf;

use serde_json::json;
use solferry_core::{Event, LinkFile, Result};

/// Runs a link until Ctrl-C, printing each file a browser sends.
pub async fn run(data_dir: PathBuf, paths: Vec<PathBuf>, json: bool) -> Result<()> {
    let (node, _offers) = super::start(data_dir).await?;
    let mut events = node.events();
    let receiving = paths.is_empty();
    let link = if receiving {
        node.receive_link().await?
    } else {
        node.share_link(paths.into_iter().map(LinkFile::Path).collect()).await?
    };
    if json {
        println!(
            "{}",
            json!({ "event": "link_ready", "urls": link.urls(), "receiving": receiving })
        );
    } else {
        let verb = if receiving { "send files here" } else { "download the files" };
        println!("Open this on a device on the same network to {verb}:");
        for url in link.urls() {
            println!("  {url}");
        }
        println!("Press Ctrl-C to stop.");
    }
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => break,
            event = events.recv() => match event {
                Ok(event @ Event::LinkReceived { .. }) if json => println!("{}", super::json::encode(&event)),
                Ok(Event::LinkReceived { name, bytes }) => {
                    println!("Received {name}, {}", indicatif::HumanBytes(bytes));
                }
                _ => {}
            },
        }
    }
    drop(link);
    node.shutdown().await
}
