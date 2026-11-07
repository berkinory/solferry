use std::collections::HashMap;

use beam_core::{Direction, Event, TransferId};
use notify_rust::Notification;

/// Tells the user about files that arrived, because trusted devices never show a prompt.
#[derive(Default)]
pub struct ArrivalNotices {
    incoming: HashMap<TransferId, (String, String)>,
}

impl ArrivalNotices {
    pub fn observe(&mut self, event: &Event) {
        match event {
            Event::Started { id, peer_name, label, direction: Direction::Receiving, .. } => {
                self.incoming.insert(*id, (peer_name.clone(), label.clone()));
            }
            Event::Completed { id } => {
                if let Some((peer, label)) = self.incoming.remove(id) {
                    let summary = format!("Received from {peer}");
                    let _ = Notification::new().summary(&summary).body(&label).show();
                }
            }
            Event::Failed { id, .. } => {
                self.incoming.remove(id);
            }
            _ => {}
        }
    }
}
