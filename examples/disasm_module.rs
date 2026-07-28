//! Disassemble a CLVM module from stdin or a file path argument.

use std::env;
use std::fs;
use std::io::{self, Read};

use cellvm::disasm;
use cellvm::loader;

fn main() {
    let mut buf = Vec::new();
    if let Some(path) = env::args().nth(1) {
        buf = fs::read(path).expect("read");
    } else {
        io::stdin().read_to_end(&mut buf).expect("stdin");
    }
    match loader::load_module(&buf) {
        Ok(m) => print!("{}", disasm::render_module(&m.functions)),
        Err(e) => eprintln!("load error: {e}"),
    }
}
