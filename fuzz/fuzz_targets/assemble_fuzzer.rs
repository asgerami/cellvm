#![no_main]
use cellvm::asm::assemble;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        let _ = assemble(text);
    }
});
