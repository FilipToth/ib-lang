use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use ibc::analysis;
use ibc::eval::{
    evaluator::{self, CancelToken, EvalLimits, RuntimeError},
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
fn run(source: &str, limits: EvalLimits) -> (Vec<String>, Option<String>) {
    let result = analysis::analyze(source.to_string());
    let root = result.runnable().expect("the program should compile");

    let io = RecordingIO::default();
    let lines = io.lines.clone();
    let error = io.error.clone();
    let mut io = io;

    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        evaluator::eval(root, &mut io, CancelToken::new(), limits).await;
    });

    let lines = lines.lock().unwrap().clone();
    let error = error.lock().unwrap().clone();

    (lines, error)
}

fn small(steps: u64, elements: u64) -> EvalLimits {
    EvalLimits {
        steps: steps,
        elements: elements,
    }
}

#[test]
fn an_endless_loop_is_stopped_by_the_step_budget() {
    let (_, error) = run(
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
    let (_, error) = run("loop while true\nend", small(50, 1000));

    let error = error.expect("a runaway program must end with an error");
    assert!(error.contains("too long"), "got: {}", error);
}

#[test]
fn unbounded_growth_is_stopped_by_the_element_budget() {
    let (_, error) = run(
        "A = new Array<Int>()\nloop while true\n    A.push(1)\nend",
        small(10_000, 8),
    );

    let error = error.expect("a runaway program must end with an error");
    assert!(error.contains("too much data"), "got: {}", error);
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
        let (_, error) = run(source, small(10_000, 8));
        let error = error.unwrap_or_else(|| panic!("{} was never charged", what));

        assert!(error.contains("too much data"), "{}: {}", what, error);
    }
}

#[test]
fn removing_elements_does_not_give_the_budget_back() {
    // the count is of elements added over the run, not elements held, so a
    // loop that pushes and pops still spends it. this pins that down, since it
    // is the price of not tracking every collection that goes out of scope.
    let (_, error) = run(
        "S = new Stack<Int>()\nloop I from 1 to 100\n    S.push(1)\n    X = S.pop()\nend",
        small(10_000, 8),
    );

    let error = error.expect("pushing past the budget must stop the program");
    assert!(error.contains("too much data"), "got: {}", error);
}

#[test]
fn an_ordinary_program_is_untouched_by_the_real_limits() {
    let (lines, error) = run(
        "A = new Array<Int>()\nloop I from 1 to 500\n    A.push(I)\nend\noutput A.len()",
        EvalLimits::default(),
    );

    assert_eq!(error, None);
    assert_eq!(lines, vec!["500\n"]);
}
