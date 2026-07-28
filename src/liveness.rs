//! Local slot liveness over a function CFG.

use crate::bitvec::BitSet;
use crate::bytecode;
use crate::cfg;
use crate::module::Function;

#[derive(Clone, Debug)]
pub struct Liveness {
    pub live_in: Vec<BitSet>,
    pub live_out: Vec<BitSet>,
}

fn uses_defs(code: &[u8], start: usize, end: usize, nlocals: usize) -> (BitSet, BitSet) {
    let mut uses = BitSet::with_len(nlocals);
    let mut defs = BitSet::with_len(nlocals);
    let mut pc = start;
    while pc < end {
        let op = code[pc];
        let len = bytecode::instr_len(op).unwrap_or(1);
        match op {
            bytecode::LOADLOCAL if pc + 2 < code.len() => {
                let slot = code[pc + 1] as usize;
                if slot < nlocals && !defs.get(slot) {
                    uses.set(slot);
                }
            }
            bytecode::STORELOCAL if pc + 2 < code.len() => {
                let slot = code[pc + 1] as usize;
                if slot < nlocals {
                    defs.set(slot);
                }
            }
            _ => {}
        }
        pc += len;
    }
    (uses, defs)
}

pub fn analyze(f: &Function) -> Liveness {
    let g = cfg::build(f);
    let n = g.blocks.len();
    let nlocals = f.max_locals as usize;
    let mut uses = Vec::with_capacity(n);
    let mut defs = Vec::with_capacity(n);
    for b in &g.blocks {
        let (u, d) = uses_defs(&f.code, b.start, b.end, nlocals);
        uses.push(u);
        defs.push(d);
    }
    let mut live_in: Vec<BitSet> = (0..n).map(|_| BitSet::with_len(nlocals)).collect();
    let mut live_out: Vec<BitSet> = (0..n).map(|_| BitSet::with_len(nlocals)).collect();
    let mut changed = true;
    let mut fuel = n * nlocals * 4 + 8;
    while changed && fuel > 0 {
        changed = false;
        fuel -= 1;
        for b in (0..n).rev() {
            let mut out = BitSet::with_len(nlocals);
            for &s in &g.blocks[b].succs {
                let _ = out.union_with(&live_in[s]);
            }
            if out.count_ones() != live_out[b].count_ones() {
                changed = true;
            }
            live_out[b] = out.clone();

            let mut inn = out;
            for i in 0..nlocals {
                if defs[b].get(i) {
                    inn.clear(i);
                }
            }
            let _ = inn.union_with(&uses[b]);
            if inn.count_ones() != live_in[b].count_ones() {
                changed = true;
            }
            live_in[b] = inn;
        }
    }
    Liveness { live_in, live_out }
}

pub fn max_live(f: &Function) -> u32 {
    analyze(f)
        .live_in
        .iter()
        .map(|b| b.count_ones())
        .max()
        .unwrap_or(0)
}
