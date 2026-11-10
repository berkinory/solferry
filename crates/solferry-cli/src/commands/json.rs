use serde_json::{Value, json};
use solferry_core::{Direction, Event, Route};
use tokio::sync::broadcast;

pub async fn print_events(mut events: broadcast::Receiver<Event>) {
    while let Ok(event) = events.recv().await {
        println!("{}", encode(&event));
    }
}

pub fn encode(event: &Event) -> Value {
    match event {
        Event::TextReceived { peer, peer_name, text, auto_copy } => json!({
            "event": "text_received",
            "peer": peer.to_z32(),
            "peer_name": peer_name,
            "text": text,
            "auto_copy": auto_copy,
        }),
        Event::Paired { device } => json!({
            "event": "paired",
            "peer": device.id.to_z32(),
            "peer_name": device.name,
        }),
        Event::Started { id, peer, peer_name, label, direction, route, files, bytes, .. } => {
            json!({
                "event": "started",
                "id": id.to_string(),
                "peer": peer.to_z32(),
                "peer_name": peer_name,
                "label": label,
                "direction": direction_name(*direction),
                "route": route_name(*route),
                "files": files,
                "bytes": bytes,
            })
        }
        Event::RouteChanged { id, route } => json!({
            "event": "route_changed",
            "id": id.to_string(),
            "route": route_name(*route),
        }),
        Event::Waiting { id, peer, peer_name, label, bytes } => json!({
            "event": "waiting",
            "id": id.to_string(),
            "peer": peer.to_z32(),
            "peer_name": peer_name,
            "label": label,
            "bytes": bytes,
        }),
        Event::Progress { id, transferred, total } => json!({
            "event": "progress",
            "id": id.to_string(),
            "transferred": transferred,
            "total": total,
        }),
        Event::Reconnecting { id, reason } => json!({
            "event": "reconnecting",
            "id": id.to_string(),
            "reason": reason.to_string(),
        }),
        Event::Completed { id } => json!({ "event": "completed", "id": id.to_string() }),
        Event::Failed { id, error } => json!({
            "event": "failed",
            "id": id.to_string(),
            "error": error.to_string(),
        }),
    }
}

fn direction_name(direction: Direction) -> &'static str {
    match direction {
        Direction::Sending => "sending",
        Direction::Receiving => "receiving",
    }
}

fn route_name(route: Route) -> &'static str {
    match route {
        Route::Local => "local",
        Route::Overlay => "overlay",
        Route::Direct => "direct",
        Route::Relay => "relay",
    }
}
