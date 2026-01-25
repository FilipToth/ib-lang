use tokio::time::timeout;

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

/// The budget has to be readable without being charged, or looking at how many
/// runs are left would use one up.
#[test]
fn reports_the_execute_budget_without_spending_it() {
    let throttle = Throttle::new();

    assert_eq!(throttle.executes_used("alice"), 0);

    throttle.try_execute("alice");
    throttle.try_execute("alice");

    assert_eq!(throttle.executes_used("alice"), 2);

    // reading it twice more must not move it
    assert_eq!(throttle.executes_used("alice"), 2);
    assert_eq!(throttle.executes_used("alice"), 2);

    assert_eq!(throttle.executes_used("bob"), 0, "bob has spent nothing");
}

// --- concurrency slots ---

#[test]
fn caps_the_programs_running_at_once() {
    let throttle = Throttle::new();

    let _held: Vec<OwnedSemaphorePermit> = (0..MAX_CONCURRENT_RUNS)
        .map(|_| throttle.try_run_slot().expect("should be under the cap"))
        .collect();

    assert!(
        throttle.try_run_slot().is_none(),
        "the run past the cap must be turned away"
    );
}

#[test]
fn a_finished_run_gives_its_slot_back() {
    let throttle = Throttle::new();

    let mut held: Vec<OwnedSemaphorePermit> = (0..MAX_CONCURRENT_RUNS)
        .map(|_| throttle.try_run_slot().expect("should be under the cap"))
        .collect();

    assert!(throttle.try_run_slot().is_none());

    held.pop();

    assert!(
        throttle.try_run_slot().is_some(),
        "the slot must come back when the run ends"
    );
}

#[tokio::test(start_paused = true)]
async fn analyses_wait_for_a_slot_rather_than_being_turned_away() {
    let throttle = Throttle::new();

    let mut held: Vec<OwnedSemaphorePermit> = Vec::new();
    for _ in 0..MAX_CONCURRENT_ANALYSES {
        held.push(throttle.analysis_slot().await);
    }

    let waiting = timeout(Duration::from_secs(1), throttle.analysis_slot()).await;
    assert!(
        waiting.is_err(),
        "a fourth analysis has nothing to take yet"
    );

    held.pop();

    let freed = timeout(Duration::from_secs(1), throttle.analysis_slot()).await;
    assert!(
        freed.is_ok(),
        "the waiter must be let through once a slot frees"
    );
}

#[tokio::test(start_paused = true)]
async fn a_busy_program_does_not_hold_up_analysis() {
    // the reason runs and analyses have a semaphore each: a program sitting at
    // an input prompt holds its run slot for as long as the person takes, and
    // that must not stop anyone else's editor underlining errors
    let throttle = Throttle::new();

    let _running: Vec<OwnedSemaphorePermit> = (0..MAX_CONCURRENT_RUNS)
        .map(|_| throttle.try_run_slot().expect("should be under the cap"))
        .collect();

    let analysis = timeout(Duration::from_secs(1), throttle.analysis_slot()).await;
    assert!(analysis.is_ok(), "analysis has its own budget");
}
