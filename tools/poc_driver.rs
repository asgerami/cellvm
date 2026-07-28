use std::env;
use std::fs;
use std::process;

use cellvm::pipeline::{deser_teardown, load_run, LoadRunOpts};

fn main() {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    let mut deser = false;
    if args.first().map(|s| s.as_str()) == Some("--deser") {
        deser = true;
        args.remove(0);
    }
    let path = args.first().cloned().unwrap_or_else(|| {
        eprintln!("usage: poc_driver [--deser] <blob>");
        process::exit(2);
    });
    let data = fs::read(&path).unwrap_or_else(|e| {
        eprintln!("read {path}: {e}");
        process::exit(2);
    });
    if deser {
        match deser_teardown(&data) {
            Ok(()) => {}
            Err(e) => eprintln!("rejected: {e}"),
        }
    } else {
        match load_run(&data, LoadRunOpts { run_exec: true }) {
            Ok(()) => {}
            Err(e) => eprintln!("rejected: {e}"),
        }
    }
}
