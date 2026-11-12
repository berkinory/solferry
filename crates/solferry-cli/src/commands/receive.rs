use std::path::PathBuf;

use indicatif::HumanBytes;
use serde_json::json;
use solferry_core::{Direction, Event, Result};
use tokio::sync::mpsc;

use super::prompt::{self, Question};
use super::send::route_name;

pub async fn run(
    data_dir: PathBuf,
    dir: Option<PathBuf>,
    yes: bool,
    once: bool,
    json: bool,
) -> Result<()> {
    let (node, mut offers) = super::start(data_dir).await?;
    let destination = dir.unwrap_or_else(|| node.config().download_dir.clone());
    let mut events = node.events();
    let mut pairings = node.pairing_requests();
    let (blocks, mut to_block) = mpsc::unbounded_channel();
    let questions = prompt::spawn(json, blocks);
    if json {
        let ready = json!({
            "event": "ready",
            "id": node.id().to_z32(),
            "name": node.config().name,
            "dir": destination,
        });
        println!("{ready}");
    } else {
        println!("{} is ready ({})", node.config().name, node.id().to_z32());
        println!("Saving to {}", destination.display());
    }
    loop {
        tokio::select! {
            offer = offers.recv() => {
                let Some(offer) = offer else { break };
                if yes || offer.summary.auto_accept {
                    offer.accept(destination.clone());
                } else {
                    let _ = questions.send(Question::Offer(offer, destination.clone()));
                }
            }
            Some(request) = pairings.recv() => {
                let _ = questions.send(Question::Pairing(request));
            }
            Some(peer) = to_block.recv() => {
                match node.block(peer, None) {
                    Err(error) if json => eprintln!("{}", crate::explain::fields(&error)),
                    Err(error) => eprintln!("{}", crate::explain::render(&error)),
                    Ok(()) => {}
                }
            }
            event = events.recv() => {
                let Ok(event) = event else { continue };
                if json {
                    println!("{}", super::json::encode(&event));
                } else {
                    report(&event);
                }
                if once && matches!(event, Event::Completed { .. }) {
                    break;
                }
            }
            _ = tokio::signal::ctrl_c() => break,
        }
    }
    node.shutdown().await
}

pub fn file_count(files: usize) -> String {
    if files == 1 { "1 file".to_owned() } else { format!("{files} files") }
}

fn report(event: &Event) {
    match event {
        Event::Started {
            peer_name, direction: Direction::Receiving, route, files, bytes, ..
        } => {
            let size = HumanBytes(*bytes);
            println!(
                "Receiving {}, {size} from {peer_name} over {}",
                file_count(*files),
                route_name(*route)
            );
        }
        Event::TextReceived { peer_name, text, .. } => {
            let printable: String =
                text.chars().filter(|c| !c.is_control() || matches!(c, '\n' | '\t')).collect();
            println!("Text from {peer_name}:\n{printable}");
        }
        Event::RouteChanged { route, .. } => println!("Now on {}", route_name(*route)),
        Event::Completed { id } => println!("Transfer {id} finished"),
        Event::Failed { id, error } => {
            eprintln!("Transfer {id}: {}", crate::explain::render(error));
        }
        _ => {}
    }
}
