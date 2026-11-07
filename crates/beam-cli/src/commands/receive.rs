use std::io::{BufRead, Write};
use std::path::PathBuf;

use beam_core::{Direction, Event, IncomingOffer, Relationship, Result};
use indicatif::HumanBytes;

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
    if !json {
        println!("{} is ready ({})", node.config().name, node.id().to_z32());
        println!("Saving to {}", destination.display());
    }
    loop {
        tokio::select! {
            offer = offers.recv() => {
                let Some(offer) = offer else { break };
                handle_offer(offer, destination.clone(), yes).await;
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

fn report(event: &Event) {
    match event {
        Event::Started {
            peer_name, direction: Direction::Receiving, route, files, bytes, ..
        } => {
            let size = HumanBytes(*bytes);
            println!(
                "Receiving {files} files, {size} from {peer_name} over {}",
                route_name(*route)
            );
        }
        Event::Completed { id } => println!("Transfer {id} finished"),
        Event::Failed { id, error } => {
            eprintln!("Transfer {id}: {}", crate::explain::render(error));
        }
        _ => {}
    }
}

async fn handle_offer(offer: IncomingOffer, destination: PathBuf, yes: bool) {
    let summary = &offer.summary;
    let own_device = summary.relationship == Some(Relationship::Own);
    let trust = if summary.relationship.is_some() { "paired" } else { "NOT PAIRED" };
    let prompt = format!(
        "{} ({trust}) wants to send {} files, {}, starting with {}. Accept? [y/N] ",
        summary.peer_name,
        summary.files,
        HumanBytes(summary.bytes),
        summary.first_path,
    );
    let accepted = yes
        || own_device
        || tokio::task::spawn_blocking(move || ask(&prompt)).await.unwrap_or(false);
    if accepted {
        offer.accept(destination);
    } else {
        offer.decline();
    }
}

fn ask(prompt: &str) -> bool {
    print!("{prompt}");
    let _ = std::io::stdout().flush();
    let mut answer = String::new();
    std::io::stdin().lock().read_line(&mut answer).is_ok()
        && matches!(answer.trim().to_lowercase().as_str(), "y" | "yes")
}
