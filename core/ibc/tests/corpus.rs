use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use ibc::analysis;
use ibc::eval::{
    evaluator::{self, RuntimeError},
    EvalIO,
};

/// Captures everything a program writes, so a test can assert on it.
/// `EvalIO` takes `&self`, hence the interior mutability.
struct CapturingIO {
    output: Arc<Mutex<String>>,
}

impl CapturingIO {
    fn new() -> CapturingIO {
        CapturingIO {
            output: Arc::new(Mutex::new(String::new())),
        }
    }

    fn captured(&self) -> String {
        self.output.lock().unwrap().clone()
    }
}

#[async_trait]
impl EvalIO for CapturingIO {
    async fn output(&self, output_msg: String) {
        self.output.lock().unwrap().push_str(&output_msg);
    }

    async fn input(&self) -> String {
        // no corpus program reads input
        String::new()
    }

    async fn runtime_error(&self, error: RuntimeError) {
        let line = format!("RUNTIME ERROR: {}\n", error);
        self.output.lock().unwrap().push_str(&line);
    }
}

/// Analyzes and runs `source`, returning its output. Panics with the collected
/// diagnostics if the program does not compile, since every corpus program is
/// expected to be valid.
fn run(name: &str, source: String) -> String {
    let result = analysis::analyze(source);

    let mut messages: Vec<String> = Vec::new();
    for error in &result.errors.errors {
        messages.push(error.kind.format());
    }

    let root = match result.root {
        Some(root) => root,
        None => panic!("{} failed to compile: {:?}", name, messages),
    };

    assert!(
        messages.is_empty(),
        "{} produced diagnostics: {:?}",
        name,
        messages
    );

    let mut io = CapturingIO::new();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        evaluator::eval(&root, &mut io).await;
    });

    io.captured()
}

/// Every `.ib` file in tests/corpus has a matching `.expected` file holding the
/// exact output it should produce. Running the corpus and diffing against those
/// files is what guards the operator implementations.
#[test]
fn corpus_matches_expected_output() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus");
    let entries = fs::read_dir(&dir).expect("tests/corpus should exist");

    let mut names: Vec<String> = Vec::new();
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".ib") {
            names.push(name);
        }
    }

    names.sort();
    assert!(!names.is_empty(), "corpus should not be empty");

    let mut failures: Vec<String> = Vec::new();

    for name in &names {
        let source = fs::read_to_string(dir.join(name)).unwrap();
        let expected_path = dir.join(name.replace(".ib", ".expected"));

        let expected = match fs::read_to_string(&expected_path) {
            Ok(e) => e,
            Err(_) => {
                failures.push(format!("{}: missing {:?}", name, expected_path));
                continue;
            }
        };

        let actual = run(name, source);
        if actual != expected {
            failures.push(format!(
                "{}:\n  expected: {:?}\n  actual:   {:?}",
                name, expected, actual
            ));
        }
    }

    assert!(failures.is_empty(), "corpus mismatches:\n{}", failures.join("\n"));
}

/// A runtime error stops the program where it happens -- including from inside
/// a function, a loop, a condition or a method argument -- and is reported once,
/// with its line. Each program can only reach one error, so these are separate
/// programs rather than a corpus file.
#[test]
fn runtime_errors_stop_the_program() {
    let cases = [
        (
            "output \"before\"\n\
             output 1 / 0\n\
             output \"after\"",
            "before\n\
             RUNTIME ERROR: Division by zero on line 2\n",
        ),
        (
            // integer division traps a zero divisor the same way
            "output 1 div 0",
            "RUNTIME ERROR: Division by zero on line 1\n",
        ),
        (
            // the caller stops too, not just the function
            "function f() -> Int\n\
                 X = 1 / 0\n\
                 output \"rest of f\"\n\
                 return 1\n\
             end\n\
             output f()\n\
             output \"after\"",
            "RUNTIME ERROR: Division by zero on line 2\n",
        ),
        (
            "loop I from 0 to 3\n\
                 output I\n\
                 output 1 / (I - 1)\n\
             end\n\
             output \"after\"",
            "0\n\
             -1\n\
             1\n\
             RUNTIME ERROR: Division by zero on line 3\n",
        ),
        (
            // the failed value is never used, so there is no second error
            "X = 1 / 0\n\
             Y = X + 1\n\
             output Y",
            "RUNTIME ERROR: Division by zero on line 1\n",
        ),
        (
            // a condition that fails used to panic the evaluator
            "I = 0\n\
             loop while I / 0 < 3\n\
                 output I\n\
             end",
            "RUNTIME ERROR: Division by zero on line 2\n",
        ),
        (
            "S = new Stack<Int>()\n\
             S.push(1 / 0)\n\
             output \"after\"",
            "RUNTIME ERROR: Division by zero on line 2\n",
        ),
        (
            "A = new Array<Int>()\n\
             A.push(1)\n\
             output A[5]",
            "RUNTIME ERROR: Index 5 is out of bounds for an array of length 1 on line 3\n",
        ),
        (
            "A = new Array<Int>()\n\
             output A[-1]",
            "RUNTIME ERROR: Index -1 is out of bounds for an array of length 0 on line 2\n",
        ),
        (
            // one past the end appends, anything further is an error
            "A = new Array<Int>()\n\
             A[1] = 5",
            "RUNTIME ERROR: Index 1 is out of bounds for an array of length 0 on line 2\n",
        ),
        (
            "S = new Stack<Int>()\n\
             output S.pop()",
            "RUNTIME ERROR: Popping element from an empty stack on line 2\n",
        ),
        (
            "Q = new Queue<Int>()\n\
             output Q.dequeue()",
            "RUNTIME ERROR: Dequeuing from an empty queue on line 2\n",
        ),
        (
            "C = new Collection<Int>()\n\
             output C.getNext()",
            "RUNTIME ERROR: Getting item from an empty collection on line 2\n",
        ),
    ];

    for (source, expected) in cases {
        let actual = run("runtime error case", source.to_string());
        assert_eq!(actual, expected, "{:?}", source);
    }
}

/// A program that did not pass analysis is never run. Analysis stops at the
/// first error it cannot recover from, so running what is left would do less
/// than the program says without saying so.
#[test]
fn a_program_with_errors_is_not_runnable() {
    let broken = [
        // a statement that cannot start ends the scope early
        "output 1 ,\noutput 2",
        "output \"hello\"\noutput MISSING",
        "X = \"s\"\nX = 1",
    ];

    for source in broken {
        let result = analysis::analyze(source.to_string());

        assert!(!result.errors.errors.is_empty(), "{:?}", source);
        assert!(result.runnable().is_none(), "{:?}", source);
    }

    let result = analysis::analyze("output \"fine\"".to_string());

    assert!(result.errors.errors.is_empty());
    assert!(result.runnable().is_some());
}
