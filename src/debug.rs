//! Execution tracing hooks (optional; used by tools / examples).

use crate::bytecode;
use crate::isa;
use crate::module::Function;

#[derive(Clone, Debug, Default)]
pub struct TraceEvent {
    pub pc: usize,
    pub op: u8,
    pub name: &'static str,
}

#[derive(Clone, Debug, Default)]
pub struct TraceBuf {
    pub events: Vec<TraceEvent>,
    pub max: usize,
}

impl TraceBuf {
    pub fn new(max: usize) -> Self {
        Self { events: Vec::new(), max }
    }

    pub fn record(&mut self, pc: usize, op: u8) {
        if self.events.len() >= self.max {
            return;
        }
        self.events.push(TraceEvent {
            pc,
            op,
            name: isa::name_of(op).unwrap_or("???"),
        });
    }

    pub fn render(&self) -> String {
        self.events
            .iter()
            .map(|e| format!("{:>4X} {}", e.pc, e.name))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub fn dry_run_trace(f: &Function, max_steps: usize) -> TraceBuf {
    let mut buf = TraceBuf::new(max_steps);
    let mut pc = 0usize;
    let mut steps = 0usize;
    while pc < f.code.len() && steps < max_steps {
        let op = f.code[pc];
        buf.record(pc, op);
        if op == bytecode::RET || op == bytecode::THROW {
            break;
        }
        pc += bytecode::instr_len(op).unwrap_or(1);
        steps += 1;
    }
    buf
}

pub fn opcode_histogram(f: &Function) -> Vec<(u8, usize)> {
    let mut counts = [0usize; 256];
    let mut pc = 0usize;
    while pc < f.code.len() {
        let op = f.code[pc];
        counts[op as usize] += 1;
        pc += bytecode::instr_len(op).unwrap_or(1);
    }
    let mut out: Vec<_> = counts
        .iter()
        .enumerate()
        .filter(|(_, c)| **c > 0)
        .map(|(op, c)| (op as u8, *c))
        .collect();
    out.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
    out
}
