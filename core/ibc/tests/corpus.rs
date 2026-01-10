use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use ibc::analysis;
use ibc::eval::{evaluator, EvalIO};

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

    async fn runtime_error(&self, msg: String) {
        let line = format!("RUNTIME ERROR: {}\n", msg);
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
