//! Control-flow graph construction over verified bytecode.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::bytecode;
use crate::isa;
use crate::module::Function;

#[derive(Clone, Debug, Default)]
pub struct BasicBlock {
    pub id: usize,
    pub start: usize,
    pub end: usize,
    pub succs: Vec<usize>,
    pub preds: Vec<usize>,
}

#[derive(Clone, Debug, Default)]
pub struct Cfg {
    pub blocks: Vec<BasicBlock>,
    pub entry: usize,
}

pub fn build(f: &Function) -> Cfg {
    let code = &f.code;
    if code.is_empty() {
        return Cfg::default();
    }
    let mut leaders = BTreeSet::from([0usize]);
    let mut pc = 0usize;
    while pc < code.len() {
        let op = code[pc];
        let len = bytecode::instr_len(op).unwrap_or(1);
        let next = pc + len;
        if op == bytecode::SJMP && pc + 1 < code.len() {
            let rel = code[pc + 1] as i8 as i64;
            let target = (pc as i64 + 2 + rel) as usize;
            if target < code.len() {
                leaders.insert(target);
            }
            if next < code.len() {
                leaders.insert(next);
            }
        } else if isa::lookup(op).map(|i| i.is_branch).unwrap_or(false) {
            if next < code.len() {
                leaders.insert(next);
            }
        }
        pc = next;
    }
    let leaders: Vec<usize> = leaders.into_iter().collect();
    let mut blocks = Vec::new();
    for (i, &start) in leaders.iter().enumerate() {
        let end = leaders.get(i + 1).copied().unwrap_or(code.len());
        blocks.push(BasicBlock { id: i, start, end, succs: vec![], preds: vec![] });
    }
    let start_to_id: BTreeMap<usize, usize> = blocks.iter().map(|b| (b.start, b.id)).collect();
    for b in 0..blocks.len() {
        let start = blocks[b].start;
        let end = blocks[b].end;
        if start >= end {
            continue;
        }
        let last_pc = {
            let mut p = start;
            let mut last = start;
            while p < end {
                last = p;
                let op = code[p];
                p += bytecode::instr_len(op).unwrap_or(1);
            }
            last
        };
        let op = code[last_pc];
        if op == bytecode::SJMP && last_pc + 1 < code.len() {
            let rel = code[last_pc + 1] as i8 as i64;
            let target = (last_pc as i64 + 2 + rel) as usize;
            if let Some(&id) = start_to_id.get(&target) {
                blocks[b].succs.push(id);
            }
        } else if op != bytecode::RET && op != bytecode::THROW && op != bytecode::TAIL {
            if let Some(&id) = start_to_id.get(&end) {
                blocks[b].succs.push(id);
            }
        }
    }
    for b in 0..blocks.len() {
        for s in blocks[b].succs.clone() {
            blocks[s].preds.push(b);
        }
    }
    Cfg { blocks, entry: 0 }
}

pub fn reachable(cfg: &Cfg) -> BTreeSet<usize> {
    let mut seen = BTreeSet::new();
    let mut q = VecDeque::from([cfg.entry]);
    while let Some(b) = q.pop_front() {
        if !seen.insert(b) {
            continue;
        }
        for &s in &cfg.blocks[b].succs {
            q.push_back(s);
        }
    }
    seen
}

pub fn has_cycle(cfg: &Cfg) -> bool {
    let mut color = vec![0u8; cfg.blocks.len()];
    fn dfs(b: usize, cfg: &Cfg, color: &mut [u8]) -> bool {
        color[b] = 1;
        for &s in &cfg.blocks[b].succs {
            if color[s] == 1 {
                return true;
            }
            if color[s] == 0 && dfs(s, cfg, color) {
                return true;
            }
        }
        color[b] = 2;
        false
    }
    (0..cfg.blocks.len()).any(|b| color[b] == 0 && dfs(b, cfg, &mut color))
}

pub fn summarize(cfg: &Cfg) -> String {
    format!(
        "cfg blocks={} reachable={} cyclic={}",
        cfg.blocks.len(),
        reachable(cfg).len(),
        has_cycle(cfg)
    )
}
