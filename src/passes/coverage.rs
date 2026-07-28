use std::collections::BTreeSet;

use crate::bytecode;
use crate::isa;
use crate::module::Module;

pub fn opcode_coverage(m: &Module) -> BTreeSet<u8> {
    let mut set = BTreeSet::new();
    for f in &m.functions {
        let mut pc = 0usize;
        while pc < f.code.len() {
            let op = f.code[pc];
            set.insert(op);
            pc += bytecode::instr_len(op).unwrap_or(1);
        }
    }
    set
}

pub fn coverage_ratio(m: &Module) -> f64 {
    let cov = opcode_coverage(m).len() as f64;
    let total = isa::OPS.len() as f64;
    if total == 0.0 {
        0.0
    } else {
        cov / total
    }
}

pub fn missing_opcodes(m: &Module) -> Vec<&'static str> {
    let cov = opcode_coverage(m);
    isa::OPS
        .iter()
        .filter(|i| !cov.contains(&i.opcode))
        .map(|i| i.name)
        .collect()
}
