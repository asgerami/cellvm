#![no_main]
use cellvm::pipeline::deser_teardown;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = deser_teardown(data);
});
