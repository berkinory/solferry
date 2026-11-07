use std::path::PathBuf;
use std::time::Duration;

use beam_core::{Direction, Error, Event, Node, Result, Route};
use futures::future::join_all;
use indicatif::{HumanBytes, ProgressBar, ProgressStyle};
use iroh::{EndpointAddr, EndpointId};
use tokio::sync::broadcast;

const NEARBY_WINDOW: Duration = Duration::from_secs(3);
const BAR_TEMPLATE: &str = "{msg} [{bar:32}] {bytes}/{total_bytes} {bytes_per_sec} eta {eta}";

pub async fn run(
    data_dir: PathBuf,
    paths: Vec<PathBuf>,
    targets: Vec<String>,
    json: bool,
) -> Result<()> {
    let (node, _offers) = super::start(data_dir).await?;
    let addrs = resolve(&node, &targets).await?;
    let progress = if json {
        tokio::spawn(super::json::print_events(node.events()))
    } else {
        tokio::spawn(show_progress(node.events()))
    };
    let sends = addrs.into_iter().map(|addr| node.send(addr, paths.clone()));
    let mut failure = None;
    for (target, outcome) in targets.iter().zip(join_all(sends).await) {
        match outcome {
            Ok(report) => {
                let resumed = HumanBytes(report.resumed_bytes);
                let suffix = if report.resumed_bytes > 0 {
                    format!(", resumed {resumed}")
                } else {
                    String::new()
                };
                println!(
                    "Sent {} to {} over {}{suffix}",
                    HumanBytes(report.bytes_sent),
                    report.peer_name,
                    route_name(report.route),
                );
            }
            Err(error) => {
                eprintln!("{target}: {}", crate::explain::render(&error));
                failure = Some(error);
            }
        }
    }
    progress.abort();
    node.shutdown().await?;
    failure.map_or(Ok(()), Err)
}

async fn resolve(node: &Node, targets: &[String]) -> Result<Vec<EndpointAddr>> {
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
                eprintln!(
                    "{target} isn't paired, so its name is unverified. It will be asked first."
                );
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
            Event::Started { .. } | Event::Paired { .. } => {}
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
