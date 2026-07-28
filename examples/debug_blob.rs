use cellvm::{load_run, LoadRunOpts};

fn main() {
    let path = std::env::args().nth(1).expect("blob");
    let data = std::fs::read(&path).unwrap();
    println!("len={}", data.len());
    match cellvm::loader::load_module(&data) {
        Ok(m) => {
            println!("funcs={} strings0={}", m.functions.len(), m.functions[0].string_pool.len());
            for (i, f) in m.functions.iter().enumerate() {
                println!("  f{i} code_len={} name={}", f.code.len(), f.name);
            }
        }
        Err(e) => println!("load err {e}"),
    }
    match load_run(&data, LoadRunOpts { run_exec: true }) {
        Ok(()) => println!("run Ok"),
        Err(e) => println!("run Err {e}"),
    }
}
