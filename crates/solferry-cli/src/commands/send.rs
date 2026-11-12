use std::path::PathBuf;
use std::time::Duration;

use futures::future::join_all;
use indicatif::{HumanBytes, ProgressBar, ProgressStyle};
use iroh::{EndpointAddr, EndpointId};
use serde_json::json;
use solferry_core::{Direction, Error, Event, Node, Result, Route, SendReport};
use tokio::sync::broadcast;

const NEARBY_WINDOW: Duration = Duration::from_secs(3);
const DRAIN_WAIT: Duration = Duration::from_secs(2);
const BAR_TEMPLATE: &str = "{msg} [{bar:32}] {bytes}/{total_bytes} {bytes_per_sec} eta {eta}";

pub async fn run(
    data_dir: PathBuf,
    paths: Vec<PathBuf>,
    targets: Vec<String>,
    json: bool,
) -> Result<()> {
    let (node, _offers) = super::start(data_dir).await?;
    let addrs = resolve(&node, &targets, json).await?;
    let mut progress = if json {
        tokio::spawn(super::json::print_events(node.events()))
    } else {
        tokio::spawn(show_progress(node.events()))
    };
    let sends = addrs.into_iter().map(|addr| node.send(addr, paths.clone()));
    let mut failure = None;
    for (target, outcome) in targets.iter().zip(join_all(sends).await) {
        match outcome {
            Ok(report) => show_sent(target, &report, json),
            Err(error) => {
                show_failure(target, &error, json);
                failure = Some(error);
            }
        }
    }
    let shutdown = node.shutdown().await;
    // Shutdown closes the event channel, so the printer ends once it has printed the last event.
    if tokio::time::timeout(DRAIN_WAIT, &mut progress).await.is_err() {
        progress.abort();
    }
    shutdown?;
    failure.map_or(Ok(()), Err)
}

/// Shares text with paired devices. They copy it, or show it to copy if they ask first.
fn show_sent(target: &str, report: &SendReport, json: bool) {
    if json {
        let sent = json!({
            "event": "sent",
            "target": target,
            "id": report.id.to_string(),
            "peer_name": report.peer_name,
            "route": super::json::route_name(report.route),
            "bytes_sent": report.bytes_sent,
            "resumed_bytes": report.resumed_bytes,
        });
        println!("{sent}");
        return;
    }
    let resumed = HumanBytes(report.resumed_bytes);
    let suffix =
        if report.resumed_bytes > 0 { format!(", resumed {resumed}") } else { String::new() };
    println!(
        "Sent {} to {} over {}{suffix}",
        HumanBytes(report.bytes_sent),
        report.peer_name,
        route_name(report.route),
    );
}

/// Under `--json` this goes to stdout as an event; the command's final error still goes to stderr.
fn show_failure(target: &str, error: &Error, json: bool) {
    if json {
        let failed = json!({ "event": "send_failed", "target": target });
        println!("{}", super::json::with_error(failed, error));
    } else {
        eprintln!("{target}: {}", crate::explain::render(error));
    }
}

pub async fn text(data_dir: PathBuf, text: String, targets: Vec<String>, json: bool) -> Result<()> {
    let text = if text == "-" {
        let mut input = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut input)
            .map_err(|error| Error::Protocol(format!("reading stdin: {error}")))?;
        input
    } else {
        text
    };
    let (node, _offers) = super::start(data_dir).await?;
    let addrs = resolve(&node, &targets, json).await?;
    let sends = addrs.into_iter().map(|addr| node.send_text(addr, text.clone()));
    let mut failure = None;
    for (target, outcome) in targets.iter().zip(join_all(sends).await) {
        match outcome {
            Ok(()) if json => println!("{}", json!({ "event": "text_sent", "target": target })),
            Ok(()) => println!("Shared text with {target}"),
            Err(error) => {
                show_failure(target, &error, json);
                failure = Some(error);
            }
        }
    }
    node.shutdown().await?;
    failure.map_or(Ok(()), Err)
}

async fn resolve(node: &Node, targets: &[String], json: bool) -> Result<Vec<EndpointAddr>> {
    let mut addrs = Vec::with_capacity(targets.len());
    for target in targets {
        match node.find_device(target) {
            Ok(device) => addrs.push(device.addr()),
            Err(Error::UnknownDevice(_)) => {
                if let Ok(id) = target.parse::<EndpointId>() {
                    addrs.push(EndpointAddr::new(id));
                    continue;
                }
                let device = node.find_nearby(target, NEARBY_WINDOW).await;
                let device = device.ok_or_else(|| Error::UnknownDevice(target.clone()))?;
                if json {
                    let unpaired = json!({
                        "event": "unpaired_target",
                        "target": target,
                        "peer": device.addr.id.to_z32(),
                    });
                    println!("{unpaired}");
                } else {
                    eprintln!(
                        "{target} isn't paired, so its name is unverified. It will be asked first."
                    );
                }
                addrs.push(device.addr);
            }
            Err(error) => return Err(error),
        }
    }
    Ok(addrs)
}

async fn show_progress(mut events: broadcast::Receiver<Event>) {
    let style =
        ProgressStyle::with_template(BAR_TEMPLATE).unwrap_or_else(|_| ProgressStyle::default_bar());
    let bar = ProgressBar::hidden();
    while let Ok(event) = events.recv().await {
        match event {
            Event::Started { peer_name, direction: Direction::Sending, bytes, .. } => {
                bar.set_draw_target(indicatif::ProgressDrawTarget::stderr());
                bar.set_style(style.clone());
                bar.set_length(bytes);
                bar.set_message(peer_name);
            }
            Event::RouteChanged { route, .. } => {
                bar.println(format!("Now on {}", route_name(route)));
            }
            Event::Progress { transferred, .. } => bar.set_position(transferred),
            Event::Completed { .. } | Event::Failed { .. } => bar.finish_and_clear(),
            Event::Reconnecting { .. } => bar.set_message("reconnecting"),
            Event::Waiting { .. }
            | Event::LinkReceived { .. }
            | Event::LinkText { .. }
            | Event::Present { .. }
            | Event::Started { .. }
            | Event::Paired { .. }
            | Event::TextReceived { .. } => {}
        }
    }
}

pub fn route_name(route: Route) -> &'static str {
    match route {
        Route::Local => "the local network",
        Route::Overlay => "an overlay network",
        Route::Direct => "a direct connection",
        Route::Relay => "a relay",
    }
}
