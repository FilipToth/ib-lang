use std::fs;

use ibc::analysis::{self, AnalysisResult};

/// Collects the `.ib` files in `dir`, sorted so the report is stable across
/// runs and can be diffed against a previous one.
fn corpus_filenames(dir: &str) -> Vec<String> {
    let entries = fs::read_dir(dir).unwrap();
    let mut names: Vec<String> = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".ib") {
            continue;
        }

        names.push(name);
    }

    names.sort();
    names
}

/// Renders one program's analysis as text: whether it bound, every diagnostic,
/// and the full bound tree. Any behavioural change shows up as a diff.
fn report(name: &str, result: &AnalysisResult) -> String {
    let mut out = String::new();

    out += &format!("########## {}\n", name);
    out += &format!("root_is_some: {}\n", result.root.is_some());
    out += &format!("error_count: {}\n", result.errors.errors.len());

    for error in &result.errors.errors {
        let start = error.span.start.char_offset;
        let end = error.span.end.char_offset;
        let message = error.kind.format();

        out += &format!("  [{}..{}] {}\n", start, end, message);
    }

    out += &format!("tree: {:#?}\n", result.root);
    out
}

/// Snapshots the whole corpus to a file. This is a refactoring aid rather than
/// a pass/fail test: capture a report before a change, capture another after,
/// and diff the two.
///
/// IB_CORPUS   directory holding the .ib programs
/// IB_SNAP_OUT file to write the report to
#[test]
fn snapshot_corpus() {
    // opt-in: a plain `cargo test` run skips this rather than failing
    let dir = match std::env::var("IB_CORPUS") {
        Ok(dir) => dir,
        Err(_) => return,
    };

    let out_path = match std::env::var("IB_SNAP_OUT") {
        Ok(path) => path,
        Err(_) => return,
    };

    let mut out = String::new();

    for name in corpus_filenames(&dir) {
        let path = format!("{}/{}", dir, name);
        let contents = fs::read_to_string(path).unwrap();

        let result = analysis::analyze(contents);
        out += &report(&name, &result);
    }

    fs::write(out_path, out).unwrap();
}
