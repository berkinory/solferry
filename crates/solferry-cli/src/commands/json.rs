use serde_json::{Value, json};
use solferry_core::{Direction, Error, Event, OfferSummary, PairingRequest, Platform, Route};
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
        Event::LinkReceived { name, bytes } => {
            json!({ "event": "link_received", "name": name, "bytes": bytes })
        }
        Event::LinkText { text } => json!({ "event": "link_text", "text": text }),
        Event::Present { peer } => json!({ "event": "present", "peer": peer.to_z32() }),
        Event::Failed { id, error } => {
            with_error(json!({ "event": "failed", "id": id.to_string() }), error)
        }
    }
}

/// Adds the `error`, `message` and `next` fields that every JSON error carries.
pub fn with_error(mut value: Value, error: &Error) -> Value {
    if let (Some(base), Value::Object(fields)) =
        (value.as_object_mut(), crate::explain::fields(error))
    {
        base.extend(fields);
    }
    value
}

pub fn offer(summary: &OfferSummary) -> Value {
    json!({
        "event": "offer",
        "id": summary.id.to_string(),
        "peer": summary.peer.to_z32(),
        "peer_name": summary.peer_name,
        "paired": summary.paired,
        "files": summary.files,
        "bytes": summary.bytes,
        "first_path": summary.first_path,
    })
}

pub fn pairing_request(request: &PairingRequest) -> Value {
    json!({
        "event": "pairing_request",
        "peer": request.peer.to_z32(),
        "peer_name": request.peer_name,
        "code": request.code,
    })
}

pub fn platform(platform: Option<Platform>) -> Option<&'static str> {
    platform.map(Platform::code)
}

fn direction_name(direction: Direction) -> &'static str {
    match direction {
        Direction::Sending => "sending",
        Direction::Receiving => "receiving",
    }
}

pub fn route_name(route: Route) -> &'static str {
    match route {
        Route::Local => "local",
        Route::Overlay => "overlay",
        Route::Direct => "direct",
        Route::Relay => "relay",
    }
}
