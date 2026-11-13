use std::io::IsTerminal;
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
    let (mut node, mut offers) = super::start(data_dir.clone()).await?;
    let destination = dir.unwrap_or_else(|| node.config().download_dir.clone());
    let mut events = node.events();
    let mut pairings = node.pairing_requests();
    let (blocks, mut to_block) = mpsc::unbounded_channel();
    let questions = prompt::spawn(json, blocks);
    // Under --json stdin is the answer channel by contract, so only plain output can be headless.
    let headless = !json && !std::io::stdin().is_terminal();
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
                let summary = &offer.summary;
                if summary.auto_accept || (yes && summary.paired) {
                    offer.accept(destination.clone());
                } else if yes && headless {
                    eprintln!(
                        "Refused files from {} ({}): not paired, and --yes only takes files from paired devices",
                        summary.peer_name,
                        summary.peer.to_z32(),
                    );
                    offer.decline();
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
            () = node.wants_restart() => {
                node.shutdown().await?;
                (node, offers) = super::start(data_dir.clone()).await?;
                (events, pairings) = (node.events(), node.pairing_requests());
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
