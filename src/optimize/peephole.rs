//! Local peephole rewrites on bytecode buffers (analysis / tooling only).

use crate::bytecode;

#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub nops_removed: usize,
    pub push_pop_pairs: usize,
}

pub fn strip_nops(code: &[u8]) -> (Vec<u8>, Stats) {
    let mut out = Vec::with_capacity(code.len());
    let mut st = Stats::default();
    let mut pc = 0usize;
    while pc < code.len() {
        let op = code[pc];
        let len = bytecode::instr_len(op).unwrap_or(1);
        if op == bytecode::NOP {
            st.nops_removed += 1;
            pc += len;
            continue;
        }
        out.extend_from_slice(&code[pc..pc + len.min(code.len() - pc)]);
        pc += len;
    }
    (out, st)
}

pub fn count_redundant_windows(code: &[u8]) -> usize {
    let mut n = 0;
    let mut pc = 0usize;
    let mut last_was_window = false;
    while pc < code.len() {
        let op = code[pc];
        let len = bytecode::instr_len(op).unwrap_or(1);
        if op == bytecode::WINDOW {
            if last_was_window {
                n += 1;
            }
            last_was_window = true;
        } else {
            last_was_window = false;
        }
        pc += len;
    }
    n
}

pub fn estimate_hotness(code: &[u8]) -> u32 {
    let mut score = 0u32;
    let mut pc = 0usize;
    while pc < code.len() {
        let op = code[pc];
        let len = bytecode::instr_len(op).unwrap_or(1);
        score += match op {
            bytecode::CALL | bytecode::GETFIELD => 4,
            bytecode::SJMP => 2,
            bytecode::LOADLOCAL | bytecode::STORELOCAL => 1,
            _ => 0,
        };
        pc += len;
    }
    score
}
