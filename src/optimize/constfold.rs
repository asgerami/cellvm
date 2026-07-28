//! Constant folding helpers for PUSH8 / LOADK patterns.

use crate::bytecode;
use crate::module::Function;

#[derive(Clone, Debug)]
pub enum Folded {
    Imm(i64),
    Unknown,
}

pub fn eval_straight_line(f: &Function) -> Vec<(usize, Folded)> {
    let mut out = Vec::new();
    let mut pc = 0usize;
    let code = &f.code;
    while pc < code.len() {
        let op = code[pc];
        let len = bytecode::instr_len(op).unwrap_or(1);
        let folded = match op {
            bytecode::PUSH8 if pc + 1 < code.len() => Folded::Imm(code[pc + 1] as i8 as i64),
            bytecode::LOADK if pc + 2 < code.len() => {
                let idx = code[pc + 1] as usize | ((code[pc + 2] as usize) << 8);
                f.consts.get(idx).copied().map(Folded::Imm).unwrap_or(Folded::Unknown)
            }
            _ => Folded::Unknown,
        };
        out.push((pc, folded));
        pc += len;
    }
    out
}

pub fn known_imm_count(f: &Function) -> usize {
    eval_straight_line(f)
        .into_iter()
        .filter(|(_, v)| matches!(v, Folded::Imm(_)))
        .count()
}

pub fn const_digest(f: &Function) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for c in &f.consts {
        h ^= *c as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    for (pc, v) in eval_straight_line(f) {
        if let Folded::Imm(x) = v {
            h ^= (pc as u64).wrapping_shl(32) ^ (x as u64);
            h = h.wrapping_mul(0x100000001b3);
        }
    }
    h
}
