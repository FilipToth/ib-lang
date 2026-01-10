use super::*;

/// Opens `count` connections for `uid` and returns the guards, which the
/// caller must keep alive for the slots to stay reserved.
fn fill_user(throttle: &Throttle, uid: &str, count: usize) -> Vec<ConnectionGuard> {
    let mut held: Vec<ConnectionGuard> = Vec::new();

    for _ in 0..count {
        let guard = throttle.try_connect(uid).expect("should be under the cap");
        held.push(guard);
    }

    held
}

#[test]
fn caps_connections_per_user() {
    let throttle = Throttle::new();
    let _held = fill_user(&throttle, "alice", MAX_CONNECTIONS_PER_USER);

    assert!(
        throttle.try_connect("alice").is_none(),
        "the connection past the cap must be rejected"
    );
}

#[test]
fn one_user_cannot_starve_another() {
    let throttle = Throttle::new();
    let _held = fill_user(&throttle, "alice", MAX_CONNECTIONS_PER_USER);

    assert!(throttle.try_connect("alice").is_none());
    assert!(
        throttle.try_connect("bob").is_some(),
        "bob has his own budget"
    );
}

#[test]
fn dropping_a_guard_frees_the_slot() {
    let throttle = Throttle::new();
    let mut held = fill_user(&throttle, "alice", MAX_CONNECTIONS_PER_USER);

    assert!(throttle.try_connect("alice").is_none());

    held.pop();

    assert!(
        throttle.try_connect("alice").is_some(),
        "the slot must come back when its guard drops"
    );
}

#[test]
fn enforces_global_connection_cap() {
    let throttle = Throttle::new();
    let mut held: Vec<ConnectionGuard> = Vec::new();
    let mut user = 0;

    // fill to exactly the global cap, taking as many users as that needs
    while held.len() < MAX_TOTAL_CONNECTIONS {
        let name = format!("user{}", user);

        while held.len() < MAX_TOTAL_CONNECTIONS {
            let guard = match throttle.try_connect(&name) {
                Some(g) => g,
                // this user is at their own cap, move to the next one
                None => break,
            };

            held.push(guard);
        }

        user += 1;
    }

    assert_eq!(held.len(), MAX_TOTAL_CONNECTIONS);
    assert!(
        throttle.try_connect("late-arrival").is_none(),
        "a fresh user must still be rejected once the server is full"
    );
}

#[test]
fn execute_budget_is_spent_then_refused() {
    let throttle = Throttle::new();

    for i in 0..MAX_EXECUTES_PER_WINDOW {
        assert!(throttle.try_execute("alice"), "execute {} should pass", i);
    }

    assert!(!throttle.try_execute("alice"), "the budget is spent");
    assert!(
        throttle.try_execute("bob"),
        "the budget is per-user, not global"
    );
}

#[test]
fn execute_budget_refills_as_the_window_slides() {
    let throttle = Throttle::new();
    let stale = Instant::now() - EXECUTE_WINDOW - Duration::from_secs(1);

    // spend the whole budget, backdated to before the window opened
    {
        let mut lock = throttle.inner.lock().unwrap();
        let bucket = lock.users.entry("alice".to_string()).or_default();

        for _ in 0..MAX_EXECUTES_PER_WINDOW {
            bucket.executes.push_back(stale);
        }
    }

    assert!(
        throttle.try_execute("alice"),
        "entries older than the window must be evicted"
    );
}

#[test]
fn releasing_every_slot_reclaims_the_user_entry() {
    let throttle = Throttle::new();

    {
        let _guard = throttle.try_connect("alice").unwrap();
        let lock = throttle.inner.lock().unwrap();

        assert_eq!(lock.users.len(), 1);
    }

    let lock = throttle.inner.lock().unwrap();

    assert_eq!(lock.users.len(), 0, "idle users must not accumulate");
    assert_eq!(lock.total_connections, 0);
}
