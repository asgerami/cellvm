//! Type-flow verification over a CFG built from short jumps.
//!
//! Abstract values: Bottom / Int / Ref / Top. GETFIELD requires Ref.
//! The worklist marks blocks visited and does not re-queue when a join
//! changes lattice state, so merges that only appear after a back-edge
//! can be missed.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::bytecode;
use crate::error::{Error, Result};
use crate::module::Function;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AbsType {
    Bottom,
    Int,
    Ref,
    Top,
}

#[derive(Clone, Copy, Debug)]
enum Op {
    ConstInt,
    NewObj,
    GetField,
}

struct Block {
    ops: Vec<Op>,
    succs: Vec<usize>,
}

fn join(a: AbsType, b: AbsType) -> AbsType {
    match (a, b) {
        (AbsType::Bottom, x) | (x, AbsType::Bottom) => x,
        (x, y) if x == y => x,
        _ => AbsType::Top,
    }
}

fn join_state(a: &[AbsType], b: &[AbsType]) -> Vec<AbsType> {
    let n = a.len().max(b.len());
    (0..n)
        .map(|i| {
            join(
                a.get(i).copied().unwrap_or(AbsType::Bottom),
                b.get(i).copied().unwrap_or(AbsType::Bottom),
            )
        })
        .collect()
}

fn transfer(block_id: usize, ops: &[Op], mut st: Vec<AbsType>) -> Result<Vec<AbsType>> {
    for op in ops {
        match op {
            Op::ConstInt => st.push(AbsType::Int),
            Op::NewObj => st.push(AbsType::Ref),
            Op::GetField => {
                let t = st.pop().ok_or(Error::StackUnderflow { block: block_id })?;
                if t != AbsType::Ref {
                    return Err(Error::UntypedFieldAccess { block: block_id });
                }
                st.push(AbsType::Ref);
            }
        }
    }
    Ok(st)
}

fn leaders(code: &[u8]) -> BTreeSet<usize> {
    let mut set = BTreeSet::from([0usize]);
    let mut pc = 0usize;
    while pc < code.len() {
        let op = code[pc];
        let len = bytecode::instr_len(op).unwrap_or(1);
        let next = pc + len;
        if op == bytecode::SJMP && pc + 1 < code.len() {
            let rel = code[pc + 1] as i8 as i64;
            let target = (pc as i64 + 2 + rel) as usize;
            if target < code.len() {
                set.insert(target);
            }
            if next < code.len() {
                set.insert(next);
            }
        } else if matches!(op, bytecode::RET | bytecode::THROW | bytecode::TAIL) {
            if next < code.len() {
                set.insert(next);
            }
        }
        pc = next;
    }
    set
}

fn blocks_from_function(f: &Function) -> Vec<Block> {
    let code = &f.code;
    if code.is_empty() {
        return vec![Block {
            ops: vec![],
            succs: vec![],
        }];
    }
    let leads: Vec<usize> = leaders(code).into_iter().collect();
    let start_to_id: BTreeMap<usize, usize> =
        leads.iter().enumerate().map(|(i, &pc)| (pc, i)).collect();
    let mut blocks = Vec::with_capacity(leads.len());
    for (i, &start) in leads.iter().enumerate() {
        let end = leads.get(i + 1).copied().unwrap_or(code.len());
        let mut ops = Vec::new();
        let mut pc = start;
        let mut last = start;
        while pc < end {
            last = pc;
            let op = code[pc];
            match op {
                bytecode::PUSH8 => {
                    ops.push(Op::ConstInt);
                    pc += 2;
                }
                bytecode::LOADK => {
                    ops.push(Op::ConstInt);
                    pc += 3;
                }
                bytecode::NEWARR => {
                    ops.push(Op::NewObj);
                    pc += 4;
                }
                bytecode::GETFIELD => {
                    ops.push(Op::GetField);
                    pc += 4;
                }
                _ => {
                    pc += bytecode::instr_len(op).unwrap_or(1);
                }
            }
        }
        let mut succs = Vec::new();
        let last_op = code.get(last).copied().unwrap_or(bytecode::NOP);
        if last_op == bytecode::SJMP && last + 1 < code.len() {
            let rel = code[last + 1] as i8 as i64;
            let target = (last as i64 + 2 + rel) as usize;
            if let Some(&id) = start_to_id.get(&target) {
                succs.push(id);
            }
        } else if !matches!(
            last_op,
            bytecode::RET | bytecode::THROW | bytecode::TAIL
        ) {
            if let Some(&id) = start_to_id.get(&end) {
                succs.push(id);
            }
        }
        blocks.push(Block { ops, succs });
    }
    if blocks.is_empty() {
        blocks.push(Block {
            ops: vec![Op::ConstInt],
            succs: vec![],
        });
    }
    blocks
}

pub struct TypeFlow;

impl TypeFlow {
    pub fn verify_function(f: &Function) -> Result<()> {
        let blocks = blocks_from_function(f);
        let n = blocks.len();
        let mut in_state: Vec<Vec<AbsType>> = vec![Vec::new(); n];
        let mut visited = vec![false; n];
        let mut work: VecDeque<usize> = VecDeque::new();
        work.push_back(0);
        while let Some(b) = work.pop_front() {
            if visited[b] {
                continue;
            }
            visited[b] = true;
            let out = transfer(b, &blocks[b].ops, in_state[b].clone())?;
            for &s in &blocks[b].succs {
                if s < n && !visited[s] {
                    in_state[s] = join_state(&in_state[s], &out);
                    work.push_back(s);
                }
            }
        }
        Ok(())
    }
}
