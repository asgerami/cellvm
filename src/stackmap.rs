//! Stack-depth maps at instruction boundaries.

use crate::bytecode;
use crate::isa;
use crate::module::Function;

#[derive(Clone, Debug, Default)]
pub struct StackMap {
    pub depth_at: Vec<i32>,
    pub max_depth: i32,
}

pub fn compute(f: &Function) -> Option<StackMap> {
    let mut depth_at = Vec::with_capacity(f.code.len());
    let mut depth = 0i32;
    let mut max_depth = 0i32;
    let mut pc = 0usize;
    while pc < f.code.len() {
        while depth_at.len() < pc {
            depth_at.push(depth);
        }
        depth_at.push(depth);
        let op = f.code[pc];
        let info = isa::lookup(op)?;
        depth += info.stack_delta as i32;
        if depth < 0 {
            return None;
        }
        max_depth = max_depth.max(depth);
        pc += info.len;
    }
    Some(StackMap { depth_at, max_depth })
}

pub fn max_depth_of(f: &Function) -> i32 {
    compute(f).map(|m| m.max_depth).unwrap_or(0)
}

pub fn is_balanced(f: &Function) -> bool {
    compute(f).is_some()
}

pub fn annotate(f: &Function) -> String {
    let mut s = String::new();
    if let Some(m) = compute(f) {
        let mut pc = 0usize;
        while pc < f.code.len() {
            let op = f.code[pc];
            let d = m.depth_at.get(pc).copied().unwrap_or(0);
            s.push_str(&format!("{:>4X} depth={d} {}\n", pc, isa::name_of(op).unwrap_or("?")));
            pc += bytecode::instr_len(op).unwrap_or(1);
        }
    }
    s
}
