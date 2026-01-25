use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use ibc::analysis;
use ibc::eval::{
    evaluator::{self, CancelToken, EvalLimits, EvalUsage, RuntimeError},
    EvalIO,
};

/// Captures output and the one runtime error a run can end with.
#[derive(Default)]
struct RecordingIO {
    lines: Arc<Mutex<Vec<String>>>,
    error: Arc<Mutex<Option<String>>>,
}

#[async_trait]
impl EvalIO for RecordingIO {
    async fn output(&self, output_msg: String) {
        self.lines.lock().unwrap().push(output_msg);
    }

    async fn input(&self) -> String {
        String::new()
    }

    async fn runtime_error(&self, error: RuntimeError) {
        *self.error.lock().unwrap() = Some(error.message);
    }
}

/// Runs `source` under `limits` and returns what it wrote and how it ended.
///
/// The limits are the point of the harness: every program here would run
/// forever under the real ones, so the tests set them small enough to be spent
/// immediately rather than sitting through the real budget.
fn run(source: &str, limits: EvalLimits) -> (Vec<String>, Option<String>, EvalUsage) {
    let result = analysis::analyze(source.to_string());
    let root = result.runnable().expect("the program should compile");

    let io = RecordingIO::default();
    let lines = io.lines.clone();
    let error = io.error.clone();
    let mut io = io;

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let usage = runtime
        .block_on(async { evaluator::eval(root, &mut io, CancelToken::new(), limits).await });

    let lines = lines.lock().unwrap().clone();
    let error = error.lock().unwrap().clone();

    (lines, error, usage)
}

fn small(steps: u64, elements: u64) -> EvalLimits {
    EvalLimits {
        steps: steps,
        elements: elements,
    }
}

#[test]
fn an_endless_loop_is_stopped_by_the_step_budget() {
    let (_, error, _) = run(
        "N = 0\nloop while true\n    N = N + 1\nend",
        small(50, 1000),
    );

    let error = error.expect("a runaway program must end with an error");
    assert!(error.contains("too long"), "got: {}", error);
}

#[test]
fn an_endless_loop_with_no_body_is_stopped_too() {
    // the body is where statements are counted, so a loop without one has to
    // be charged by the loop itself
    let (_, error, _) = run("loop while true\nend", small(50, 1000));

    let error = error.expect("a runaway program must end with an error");
    assert!(error.contains("too long"), "got: {}", error);
}

#[test]
fn unbounded_growth_is_stopped_by_the_element_budget() {
    let (_, error, _) = run(
        "A = new Array<Int>()\nloop while true\n    A.push(1)\nend",
        small(10_000, 8),
    );

    let error = error.expect("a runaway program must end with an error");
    assert!(error.contains("stored too much"), "got: {}", error);
}

#[test]
fn every_collection_that_grows_is_charged() {
    // each of these grows a different type, and each must spend the budget
    let programs = [
        (
            "A = new Array<Int>()\nloop while true\n    A.push(1)\nend",
            "array push",
        ),
        (
            "A = new Array<Int>()\nN = 0\nloop while true\n    A[N] = 1\n    N = N + 1\nend",
            "index append",
        ),
        (
            "S = new Stack<Int>()\nloop while true\n    S.push(1)\nend",
            "stack push",
        ),
        (
            "Q = new Queue<Int>()\nloop while true\n    Q.enqueue(1)\nend",
            "queue enqueue",
        ),
        (
            "C = new Collection<Int>()\nloop while true\n    C.addItem(1)\nend",
            "collection addItem",
        ),
    ];

    for (source, what) in programs {
        let (_, error, _) = run(source, small(10_000, 8));
        let error = error.unwrap_or_else(|| panic!("{} was never charged", what));

        assert!(error.contains("stored too much"), "{}: {}", what, error);
    }
}

#[test]
fn removing_elements_does_not_give_the_budget_back() {
    // the count is of elements added over the run, not elements held, so a
    // loop that pushes and pops still spends it. this pins that down, since it
    // is the price of not tracking every collection that goes out of scope.
    let (_, error, _) = run(
        "S = new Stack<Int>()\nloop I from 1 to 100\n    S.push(1)\n    X = S.pop()\nend",
        small(10_000, 8),
    );

    let error = error.expect("pushing past the budget must stop the program");
    assert!(error.contains("stored too much"), "got: {}", error);
}

/// A message that only said a budget was reached would leave the reader to
/// guess how big it was, so the number is in it -- grouped, since six zeroes
/// in a row do not read as a quantity.
#[test]
fn the_message_names_the_budget_that_was_reached() {
    let (_, error, _) = run("loop while true\nend", small(1_500, 10));
    let error = error.expect("a runaway program must end with an error");

    assert!(error.contains("1,500 steps"), "got: {}", error);

    let (_, error, _) = run(
        "A = new Array<Int>()\nloop while true\n    A.push(1)\nend",
        small(10_000, 2_500),
    );

    let error = error.expect("a runaway program must end with an error");
    assert!(error.contains("2,500 items"), "got: {}", error);
}

/// The counters are reported however the run ended, so a program that was
/// stopped can still be shown how close to the budget it got.
#[test]
fn reports_what_the_run_spent() {
    let (_, error, usage) = run(
        "A = new Array<Int>()\nloop I from 1 to 20\n    A.push(I)\nend",
        EvalLimits::default(),
    );

    assert_eq!(error, None);
    assert_eq!(usage.elements, 20);

    // one per turn of the loop and one per statement in its body, plus the
    // statements around it: the exact figure is not the point, that it counts
    // the work and stays under the budget is
    assert!(usage.steps >= 40, "steps were {}", usage.steps);
    assert!(usage.steps < EvalLimits::default().steps);
}

#[test]
fn a_run_that_was_stopped_still_reports_its_spend() {
    let (_, error, usage) = run("loop while true\nend", small(50, 10));

    assert!(error.is_some());
    assert_eq!(usage.steps, 51, "the step over the budget is counted too");
}

#[test]
fn an_ordinary_program_is_untouched_by_the_real_limits() {
    let (lines, error, _) = run(
        "A = new Array<Int>()\nloop I from 1 to 500\n    A.push(I)\nend\noutput A.len()",
        EvalLimits::default(),
    );

    assert_eq!(error, None);
    assert_eq!(lines, vec!["500\n"]);
}
