use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// Maximum sockets a single user may hold open at once.
const MAX_CONNECTIONS_PER_USER: usize = 3;
/// Maximum sockets held open across all users.
const MAX_TOTAL_CONNECTIONS: usize = 128;
/// Maximum execute requests a single user may issue per window.
const MAX_EXECUTES_PER_WINDOW: usize = 12;
/// Sliding window the execute budget is measured over.
const EXECUTE_WINDOW: Duration = Duration::from_secs(60);

#[derive(Default)]
struct UserBucket {
    connections: usize,
    executes: VecDeque<Instant>,
}

#[derive(Default)]
struct ThrottleInner {
    total_connections: usize,
    users: HashMap<String, UserBucket>,
}

/// Per-user admission control for the websocket endpoint. Cheap in-process
/// counters, shared across handlers via an `Extension`.
#[derive(Clone, Default)]
pub struct Throttle {
    inner: Arc<Mutex<ThrottleInner>>,
}

impl Throttle {
    pub fn new() -> Self {
        Self::default()
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

        while let Some(oldest) = bucket.executes.front() {
            if now.duration_since(*oldest) >= EXECUTE_WINDOW {
                bucket.executes.pop_front();
            } else {
                break;
            }
        }

        if bucket.executes.len() >= MAX_EXECUTES_PER_WINDOW {
            return false;
        }

        bucket.executes.push_back(now);
        true
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
