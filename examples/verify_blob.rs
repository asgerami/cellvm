use cellvm::pipeline::load_verify_only;
use std::{env, fs, process};

fn main() {
    let path = env::args().nth(1).expect("usage: verify_blob <file>");
    let data = fs::read(path).expect("read");
    process::exit(if load_verify_only(&data).is_ok() { 0 } else { 1 });
}
