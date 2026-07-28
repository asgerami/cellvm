#![no_main]
use cellvm::pipeline::load_verify_only;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = load_verify_only(data);
});
