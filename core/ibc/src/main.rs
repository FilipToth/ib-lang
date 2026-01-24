use std::{
    env, fs,
    io::{self, BufRead, BufReader},
    process, thread,
};

use async_trait::async_trait;
use eval::{evaluator::RuntimeError, EvalIO};

use analysis::error_bag::ErrorBag;

mod analysis;
mod eval;

const DEFAULT_PATH: &str = "test.ib";

struct IBEvaluator;

#[async_trait]
impl EvalIO for IBEvaluator {
    async fn output(&self, output_msg: String) {
        print!("{}", output_msg);
    }

    async fn input(&self) -> String {
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut buffer = String::new();

        match reader.read_line(&mut buffer) {
            Ok(_) => buffer.trim().to_string(),
            Err(_) => unreachable!(),
        }
    }

    async fn runtime_error(&self, error: RuntimeError) {
        println!("Runtime Error: {}", error);
    }
}

/// What the tool should do with the program.
enum Mode {
    /// Analyze and evaluate, the default.
    Run,
    /// Dump the lexer's token stream.
    Tokens,
    /// Dump the syntax tree the parser produces.
    Syntax,
    /// Dump the bound tree the binder produces.
    Bound,
    /// Write the control flow graph as graphviz to the given path.
    Cfg(String),
}

const USAGE: &str = "\
ibc -- IB Pseudocode analyzer

usage: ibc [mode] [file]

modes:
  (none)          analyze and run the program
  --tokens        print the lexer's token stream
  --syntax        print the syntax tree
  --bound         print the bound tree
  --cfg <out>     write the control flow graph to <out> as graphviz
  --help          print this message

file defaults to test.ib in the working directory.

Diagnostics are always printed first. The tree dumps stop at the stage they
name, so --syntax reports parse errors only, while --bound also reports
binding errors.";

/// Reads the mode and file path from the command line. Exits with the usage
/// message if the arguments do not make sense.
fn parse_args() -> (Mode, String) {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut mode = Mode::Run;
    let mut path: Option<String> = None;
    let mut i = 0;

    while i < args.len() {
        let arg = args[i].as_str();

        match arg {
            "--help" | "-h" => {
                println!("{}", USAGE);
                process::exit(0);
            }
            "--tokens" => mode = Mode::Tokens,
            "--syntax" => mode = Mode::Syntax,
            "--bound" => mode = Mode::Bound,
            "--cfg" => {
                i += 1;

                let out = match args.get(i) {
                    Some(o) => o.clone(),
                    None => {
                        eprintln!("--cfg needs an output path\n\n{}", USAGE);
                        process::exit(2);
                    }
                };

                mode = Mode::Cfg(out);
            }
            _ => {
                if arg.starts_with('-') {
                    eprintln!("unknown option: {}\n\n{}", arg, USAGE);
                    process::exit(2);
                }

                path = Some(arg.to_string());
            }
        }

        i += 1;
    }

    let path = match path {
        Some(p) => p,
        None => DEFAULT_PATH.to_string(),
    };

    (mode, path)
}

fn read_source(path: &str) -> String {
    match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("cannot read {}: {}", path, e);
            process::exit(1);
        }
    }
}

/// Prints whatever the stage collected. Kept separate from the dumps so a tree
/// is always preceded by the diagnostics that describe it.
fn report(errors: &ErrorBag) {
    errors.report();
}

fn dump_tokens(source: String) {
    let tokens = analysis::syntax::lexer::lex(source);

    for token in tokens {
        println!("{:?}", token);
    }
}

fn dump_syntax(source: String) {
    let mut errors = ErrorBag::new();
    let tokens = analysis::syntax::lexer::lex(source);
    let root = analysis::syntax::parser::parse(tokens, &mut errors);

    report(&errors);

    match root {
        Some(root) => println!("{:#?}", root),
        None => println!("<no syntax tree>"),
    }
}

/// Runs lexing, parsing and binding, returning whatever the binder produced.
/// Control flow analysis is deliberately skipped so the bound tree can be
/// inspected even when a function fails its return checks.
fn bind_source(source: String, errors: &mut ErrorBag) -> Option<analysis::binding::bound_node::BoundNode> {
    let tokens = analysis::syntax::lexer::lex(source);
    let root = analysis::syntax::parser::parse(tokens, errors)?;

    analysis::binding::bind_root(&root, errors)
}

fn dump_bound(source: String) {
    let mut errors = ErrorBag::new();
    let bound = bind_source(source, &mut errors);

    report(&errors);

    match bound {
        Some(bound) => println!("{:#?}", bound),
        None => println!("<no bound tree>"),
    }
}

fn dump_cfg(source: String, out: &str) {
    let mut errors = ErrorBag::new();
    let bound = bind_source(source, &mut errors);

    let bound = match bound {
        Some(bound) => bound,
        None => {
            report(&errors);
            eprintln!("cannot build a control flow graph without a bound tree");
            process::exit(1);
        }
    };

    let graphs = analysis::control_flow::analyze(&bound, &mut errors);
    analysis::control_flow::digraph(&graphs, out);

    report(&errors);
    println!("wrote {}", out);
}

async fn run(source: String) {
    let result = analysis::analyze(source);
    result.errors.report();

    let Some(root) = result.runnable() else {
        return;
    };

    let cancel = eval::evaluator::CancelToken::new();
    eval::evaluator::eval(
        root,
        &mut IBEvaluator,
        cancel,
        eval::evaluator::EvalLimits::default(),
    )
    .await;
}

fn main() {
    // evaluation recurses, so it runs on a stack sized for it rather than on
    // whatever the main thread happens to get
    let worker = thread::Builder::new()
        .stack_size(eval::evaluator::EVAL_STACK_SIZE)
        .spawn(run_cli)
        .expect("cannot start the interpreter thread");

    worker.join().expect("the interpreter thread panicked");
}

#[tokio::main]
async fn run_cli() {
    let (mode, path) = parse_args();
    let source = read_source(&path);

    match mode {
        Mode::Run => run(source).await,
        Mode::Tokens => dump_tokens(source),
        Mode::Syntax => dump_syntax(source),
        Mode::Bound => dump_bound(source),
        Mode::Cfg(out) => dump_cfg(source, &out),
    }
}
