use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use tokio::sync::{OwnedSemaphorePermit, Semaphore};

/// Maximum sockets a single user may hold open at once.
const MAX_CONNECTIONS_PER_USER: usize = 3;
/// Maximum sockets held open across all users.
const MAX_TOTAL_CONNECTIONS: usize = 128;
/// Maximum execute requests a single user may issue per window.
pub const MAX_EXECUTES_PER_WINDOW: usize = 12;
/// Sliding window the execute budget is measured over.
pub const EXECUTE_WINDOW: Duration = Duration::from_secs(60);
/// Analyses running at once, across all users. Parsing and binding allocate
/// for the whole program, so this is what bounds the memory a burst of
/// keystrokes can ask for at the same time.
const MAX_CONCURRENT_ANALYSES: usize = 3;
/// Programs running at once, across all users.
const MAX_CONCURRENT_RUNS: usize = 3;

#[derive(Default)]
struct UserBucket {
    connections: usize,
    executes: VecDeque<Instant>,
}

impl UserBucket {
    /// Drops the executes that have aged out of the window, so what is left is
    /// what the user has spent of it.
    fn prune_executes(&mut self, now: Instant) {
        while let Some(oldest) = self.executes.front() {
            if now.duration_since(*oldest) >= EXECUTE_WINDOW {
                self.executes.pop_front();
            } else {
                break;
            }
        }
    }
}

#[derive(Default)]
struct ThrottleInner {
    total_connections: usize,
    users: HashMap<String, UserBucket>,
}

/// Admission control for the analysis and websocket endpoints: per-user
/// counters plus caps on the work running at once, shared across handlers via
/// an `Extension`.
///
/// Analyses and runs get a semaphore each rather than sharing one. A run holds
/// its slot until the program ends, and a program sitting at an input prompt
/// does not end until the person types something, so a shared cap of three
/// would let three idle prompts stop everyone else getting diagnostics.
#[derive(Clone)]
pub struct Throttle {
    inner: Arc<Mutex<ThrottleInner>>,
    analyses: Arc<Semaphore>,
    runs: Arc<Semaphore>,
}

impl Default for Throttle {
    fn default() -> Self {
        Throttle {
            inner: Arc::new(Mutex::new(ThrottleInner::default())),
            analyses: Arc::new(Semaphore::new(MAX_CONCURRENT_ANALYSES)),
            runs: Arc::new(Semaphore::new(MAX_CONCURRENT_RUNS)),
        }
    }
}

impl Throttle {
    pub fn new() -> Self {
        Self::default()
    }

    /// Waits for an analysis slot. Analysis is milliseconds of CPU, so callers
    /// queue for one rather than being turned away.
    pub async fn analysis_slot(&self) -> OwnedSemaphorePermit {
        self.analyses
            .clone()
            .acquire_owned()
            .await
            .expect("the analysis semaphore is never closed")
    }

    /// Takes a slot for a program run, or None when they are all taken. The
    /// slot is held until the program ends, which has no bound when the
    /// program is waiting on input, so callers are turned away with a message
    /// instead of queueing behind a wait that may never come.
    pub fn try_run_slot(&self) -> Option<OwnedSemaphorePermit> {
        self.runs.clone().try_acquire_owned().ok()
    }

    /// Reserves a connection slot for `uid`. The returned guard releases the
    /// slot when dropped, including while unwinding from a panic.
    pub fn try_connect(&self, uid: &str) -> Option<ConnectionGuard> {
        let mut lock = self.inner.lock().unwrap();
        let inner = &mut *lock;

        if inner.total_connections >= MAX_TOTAL_CONNECTIONS {
            return None;
        }

        let bucket = inner.users.entry(uid.to_string()).or_default();
        if bucket.connections >= MAX_CONNECTIONS_PER_USER {
            return None;
        }

        bucket.connections += 1;
        inner.total_connections += 1;

        Some(ConnectionGuard {
            throttle: self.clone(),
            uid: uid.to_string(),
        })
    }

    /// Records an execute request against `uid`'s budget. Returns false when
    /// the user has spent the window's allowance.
    pub fn try_execute(&self, uid: &str) -> bool {
        let mut lock = self.inner.lock().unwrap();
        let bucket = lock.users.entry(uid.to_string()).or_default();
        let now = Instant::now();

        bucket.prune_executes(now);

        if bucket.executes.len() >= MAX_EXECUTES_PER_WINDOW {
            return false;
        }

        bucket.executes.push_back(now);
        true
    }

    /// How much of the window `uid` has spent. Takes nothing: this is for
    /// showing the budget, not for charging against it.
    pub fn executes_used(&self, uid: &str) -> usize {
        let mut lock = self.inner.lock().unwrap();

        let Some(bucket) = lock.users.get_mut(uid) else {
            return 0;
        };

        bucket.prune_executes(Instant::now());
        bucket.executes.len()
    }

    fn release(&self, uid: &str) {
        let mut lock = self.inner.lock().unwrap();
        let inner = &mut *lock;

        inner.total_connections = inner.total_connections.saturating_sub(1);

        let drop_entry = match inner.users.get_mut(uid) {
            Some(bucket) => {
                bucket.connections = bucket.connections.saturating_sub(1);
                bucket.connections == 0 && bucket.executes.is_empty()
            }
            None => false,
        };

        // don't let the map grow unboundedly with idle users
        if drop_entry {
            inner.users.remove(uid);
        }
    }
}

/// Releases a reserved connection slot on drop.
pub struct ConnectionGuard {
    throttle: Throttle,
    uid: String,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.throttle.release(&self.uid);
    }
}

#[cfg(test)]
mod tests;
