use std::io::{BufRead, Write};
use std::path::PathBuf;

use indicatif::HumanBytes;
use iroh::EndpointId;
use solferry_core::{Direction, Event, IncomingOffer, Node, PairingRequest, Result};

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
    if !json {
        println!("{} is ready ({})", node.config().name, node.id().to_z32());
        println!("Saving to {}", destination.display());
    }
    loop {
        tokio::select! {
            offer = offers.recv() => {
                let Some(offer) = offer else { break };
                handle_offer(&node, offer, destination.clone(), yes).await;
            }
            Some(request) = pairings.recv() => handle_pairing(&node, request, json).await,
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

fn file_count(files: usize) -> String {
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

async fn handle_pairing(node: &Node, request: PairingRequest, json: bool) {
    if json {
        request.decline();
        return;
    }
    let prompt = format!(
        "{} wants to pair. Code: {}. Is it the same on both screens? [y/N/block] ",
        request.peer_name, request.code
    );
    let peer = request.peer;
    match tokio::task::spawn_blocking(move || ask(&prompt)).await {
        Ok(Answer::Accept) => request.accept(true),
        Ok(Answer::Block) => {
            request.decline();
            block(node, peer);
        }
        _ => request.decline(),
    }
}

fn block(node: &Node, peer: EndpointId) {
    if let Err(error) = node.block(peer) {
        eprintln!("{}", crate::explain::render(&error));
    }
}

async fn handle_offer(node: &Node, offer: IncomingOffer, destination: PathBuf, yes: bool) {
    let summary = &offer.summary;
    let trust = if summary.paired { "paired" } else { "NOT PAIRED" };
    let prompt = format!(
        "{} ({trust}) wants to send {}, {}, starting with {}. Accept? [y/N/block] ",
        summary.peer_name,
        file_count(summary.files),
        HumanBytes(summary.bytes),
        summary.first_path,
    );
    if yes || summary.auto_accept {
        offer.accept(destination);
        return;
    }
    let peer = summary.peer;
    match tokio::task::spawn_blocking(move || ask(&prompt)).await {
        Ok(Answer::Accept) => offer.accept(destination),
        Ok(Answer::Block) => {
            offer.decline();
            block(node, peer);
        }
        _ => offer.decline(),
    }
}

enum Answer {
    Accept,
    Decline,
    Block,
}

fn ask(prompt: &str) -> Answer {
    print!("{prompt}");
    let _ = std::io::stdout().flush();
    let mut answer = String::new();
    if std::io::stdin().lock().read_line(&mut answer).is_err() {
        return Answer::Decline;
    }
    match answer.trim().to_lowercase().as_str() {
        "y" | "yes" => Answer::Accept,
        "b" | "block" => Answer::Block,
        _ => Answer::Decline,
    }
}
