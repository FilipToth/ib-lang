use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use ibc::analysis;
use ibc::eval::{
    evaluator::{self, CancelToken, RuntimeError},
    EvalIO,
};

/// Captures output, and asks the program to stop once it has produced
/// `stop_after` lines. Standing in for a user pressing stop, or a socket
/// closing, part way through a run.
struct StoppingIO {
    lines: Arc<Mutex<Vec<String>>>,
    cancel: CancelToken,
    stop_after: usize,
}

#[async_trait]
impl EvalIO for StoppingIO {
    async fn output(&self, output_msg: String) {
        let mut lines = self.lines.lock().unwrap();
        lines.push(output_msg);

        if lines.len() >= self.stop_after {
            self.cancel.cancel();
        }
    }

    async fn input(&self) -> String {
        String::new()
    }

    async fn runtime_error(&self, _error: RuntimeError) {}
}

/// Runs `source`, stopping it once it has written `stop_after` lines, and
/// returns everything it wrote. Every program here is bounded, so a broken
/// cancellation fails the assertions rather than hanging the suite.
fn run_until(source: &str, stop_after: usize) -> Vec<String> {
    let result = analysis::analyze(source.to_string());
    let root = result.runnable().expect("the program should compile");

    let cancel = CancelToken::new();
    let lines = Arc::new(Mutex::new(Vec::new()));

    let mut io = StoppingIO {
        lines: lines.clone(),
        cancel: cancel.clone(),
        stop_after: stop_after,
    };

    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        evaluator::eval(root, &mut io, cancel).await;
    });

    let lines = lines.lock().unwrap();
    lines.clone()
}

#[test]
fn a_counted_loop_stops_when_cancelled() {
    let lines = run_until(
        "loop I from 1 to 100000\n    output I\nend\noutput \"after\"",
        3,
    );

    assert_eq!(lines, vec!["1\n", "2\n", "3\n"]);
}

#[test]
fn a_while_loop_stops_when_cancelled() {
    let lines = run_until(
        "I = 0\n\
         loop while I < 100000\n\
             output I\n\
             I = I + 1\n\
         end",
        2,
    );

    assert_eq!(lines, vec!["0\n", "1\n"]);
}

/// The signal travels out of the call the same way a return or an error does.
#[test]
fn a_program_stops_inside_a_function() {
    let lines = run_until(
        "function count(N: Int)\n\
             loop I from 1 to N\n\
                 output I\n\
             end\n\
         end\n\
         count(100000)\n\
         output \"after\"",
        2,
    );

    assert_eq!(lines, vec!["1\n", "2\n"]);
}

#[test]
fn a_program_cancelled_before_it_starts_does_nothing() {
    let result = analysis::analyze("output \"never\"".to_string());
    let root = result.runnable().expect("the program should compile");

    let cancel = CancelToken::new();
    cancel.cancel();

    let lines = Arc::new(Mutex::new(Vec::new()));
    let mut io = StoppingIO {
        lines: lines.clone(),
        cancel: cancel.clone(),
        stop_after: usize::MAX,
    };

    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        evaluator::eval(root, &mut io, cancel).await;
    });

    assert!(lines.lock().unwrap().is_empty());
}

/// `loop while true end` has no statements in its body and no call in its
/// condition, so the loop itself is the only place a stop can be noticed. This
/// is also the program most likely to be written by accident.
///
/// The run happens on another thread and the result comes back through a
/// channel, so a loop that ignores the stop fails the test on the timeout
/// rather than hanging the suite.
#[test]
fn an_empty_loop_stops_when_cancelled() {
    let cancel = CancelToken::new();
    let (sender, receiver) = std::sync::mpsc::channel();

    let token = cancel.clone();
    std::thread::spawn(move || {
        let source = "loop while true\nend\noutput \"after\"".to_string();
        let result = analysis::analyze(source);
        let root = result.runnable().expect("the program should compile");

        let lines = Arc::new(Mutex::new(Vec::new()));
        let mut io = StoppingIO {
            lines: lines.clone(),
            cancel: token.clone(),
            stop_after: usize::MAX,
        };

        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            evaluator::eval(root, &mut io, token).await;
        });

        let lines = lines.lock().unwrap().clone();
        let _ = sender.send(lines);
    });

    std::thread::sleep(std::time::Duration::from_millis(50));
    cancel.cancel();

    let lines = receiver
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("an endless loop should stop when cancelled");

    // the statement after the loop never runs
    assert!(lines.is_empty(), "{:?}", lines);
}
