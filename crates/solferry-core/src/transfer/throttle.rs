//! The relay speed limit. Data waits for its share only while the connection's chosen path goes
//! through a relay, so a direct path always runs at full speed.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use iroh::endpoint::Connection;

use crate::route::{Route, current_route};

/// Lets a short burst through at once, so the limit doesn't stall small messages.
const BURST: Duration = Duration::from_millis(250);

/// A token bucket shared by every transfer in one direction. Takers reserve before they wait, so
/// concurrent streams queue up fairly instead of all waking at once.
#[derive(Debug)]
pub(crate) struct Bucket {
    rate: f64,
    capacity: f64,
    state: Mutex<(f64, Instant)>,
}

impl Bucket {
    pub(crate) fn new(bytes_per_second: u64) -> Self {
        #[allow(clippy::cast_precision_loss)]
        let rate = bytes_per_second.max(1) as f64;
        let capacity = rate * BURST.as_secs_f64();
        Self { rate, capacity, state: Mutex::new((capacity, Instant::now())) }
    }

    async fn take(&self, bytes: usize) {
        let wait = {
            let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            let (available, since) = &mut *state;
            let now = Instant::now();
            *available = (*available + now.duration_since(*since).as_secs_f64() * self.rate)
                .min(self.capacity);
            *since = now;
            #[allow(clippy::cast_precision_loss)]
            let wanted = bytes as f64;
            *available -= wanted;
            (*available < 0.0).then(|| Duration::from_secs_f64(-*available / self.rate))
        };
        if let Some(wait) = wait {
            tokio::time::sleep(wait).await;
        }
    }
}

/// The relay limits for both directions, set when the node starts.
#[derive(Clone, Debug, Default)]
pub(crate) struct RelayLimit {
    up: Option<Arc<Bucket>>,
    down: Option<Arc<Bucket>>,
}

impl RelayLimit {
    pub(crate) fn new(bytes_per_second: Option<u64>) -> Self {
        let bucket = || bytes_per_second.map(|rate| Arc::new(Bucket::new(rate)));
        Self { up: bucket(), down: bucket() }
    }

    pub(crate) fn sending(&self, connection: &Connection) -> Throttle {
        Throttle { connection: connection.clone(), bucket: self.up.clone() }
    }

    pub(crate) fn receiving(&self, connection: &Connection) -> Throttle {
        Throttle { connection: connection.clone(), bucket: self.down.clone() }
    }
}

/// One transfer's view of the limit.
#[derive(Clone, Debug)]
pub(crate) struct Throttle {
    connection: Connection,
    bucket: Option<Arc<Bucket>>,
}

impl Throttle {
    /// Waits until `bytes` may pass, which is at once unless the transfer is on a relay.
    pub(crate) async fn pass(&self, bytes: usize) {
        if let Some(bucket) = &self.bucket
            && current_route(&self.connection) == Route::Relay
        {
            bucket.take(bytes).await;
        }
    }
}
