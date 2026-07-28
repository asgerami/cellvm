#![no_main]
use cellvm::{load_run, LoadRunOpts};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = load_run(data, LoadRunOpts { run_exec: true });
});
