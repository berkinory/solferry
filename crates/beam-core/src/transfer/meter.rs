use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tokio::sync::broadcast;
use tokio::task::AbortHandle;

use crate::event::{Event, TransferId};

const REPORT_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug)]
pub(crate) struct Meter {
    transferred: Arc<AtomicU64>,
    ticker: AbortHandle,
}

impl Meter {
    pub(crate) fn start(
        id: TransferId,
        resumed: u64,
        total: u64,
        events: broadcast::Sender<Event>,
    ) -> Self {
        let transferred = Arc::new(AtomicU64::new(resumed));
        let counter = transferred.clone();
        let ticker = tokio::spawn(async move {
            let mut interval = tokio::time::interval(REPORT_INTERVAL);
            loop {
                interval.tick().await;
                let transferred = counter.load(Ordering::Relaxed);
                let _ = events.send(Event::Progress {
                    id,
                    transferred,
                    total,
                });
            }
        });
        Self {
            transferred,
            ticker: ticker.abort_handle(),
        }
    }

    pub(crate) fn add(&self, bytes: u64) {
        self.transferred.fetch_add(bytes, Ordering::Relaxed);
    }
}

impl Drop for Meter {
    fn drop(&mut self) {
        self.ticker.abort();
    }
}
