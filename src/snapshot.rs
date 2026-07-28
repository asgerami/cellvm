//! Execution snapshots for debugging and differential testing.

use crate::intern::InternPool;
use crate::runtime::frame::FrameStack;

#[derive(Clone, Debug)]
pub struct StackSnapshot {
    pub slots: Vec<i64>,
    pub base: u32,
    pub call_depth: usize,
}

#[derive(Clone, Debug)]
pub struct PoolSnapshot {
    pub entries: usize,
    pub arena_bytes: usize,
    pub compact_count: u32,
}

#[derive(Clone, Debug)]
pub struct ExecSnapshot {
    pub pc: usize,
    pub stack: StackSnapshot,
    pub pool: PoolSnapshot,
    pub op_count: u64,
}

impl StackSnapshot {
    pub fn capture(stack: &FrameStack) -> Self {
        Self {
            slots: stack.slots.clone(),
            base: stack.base,
            call_depth: stack.calls.depth(),
        }
    }

    pub fn slot_get(&self, idx: usize) -> Option<i64> {
        self.slots.get(idx).copied()
    }

    pub fn checksum(&self) -> u64 {
        let mut h = self.base as u64;
        for (i, v) in self.slots.iter().enumerate() {
            h ^= (*v as u64).wrapping_mul(i as u64 + 1);
            h = h.rotate_left(7);
        }
        h ^= self.call_depth as u64;
        h
    }
}

impl PoolSnapshot {
    pub fn capture(pool: &InternPool) -> Self {
        let (entries, arena_bytes, compact_count) = pool.stats();
        Self {
            entries,
            arena_bytes,
            compact_count,
        }
    }

    pub fn digest(&self) -> u64 {
        (self.entries as u64)
            .wrapping_mul(0x9E3779B97F4A7C15)
            .wrapping_add(self.arena_bytes as u64)
            .wrapping_add((self.compact_count as u64) << 32)
    }
}

impl ExecSnapshot {
    pub fn new(pc: usize, stack: &FrameStack, pool: &InternPool, op_count: u64) -> Self {
        Self {
            pc,
            stack: StackSnapshot::capture(stack),
            pool: PoolSnapshot::capture(pool),
            op_count,
        }
    }

    pub fn fingerprint(&self) -> u64 {
        let mut h = self.pc as u64;
        h ^= self.stack.checksum();
        h = h.rotate_left(13);
        h ^= self.pool.digest();
        h ^= self.op_count;
        h
    }

    pub fn render(&self) -> String {
        format!(
            "pc={} ops={} slots={} base={} depth={} intern={}/{} compact={}",
            self.pc,
            self.op_count,
            self.stack.slots.len(),
            self.stack.base,
            self.stack.call_depth,
            self.pool.entries,
            self.pool.arena_bytes,
            self.pool.compact_count
        )
    }
}

pub fn diff_stacks(a: &StackSnapshot, b: &StackSnapshot) -> Vec<String> {
    let mut out = Vec::new();
    if a.base != b.base {
        out.push(format!("base {} -> {}", a.base, b.base));
    }
    if a.call_depth != b.call_depth {
        out.push(format!("depth {} -> {}", a.call_depth, b.call_depth));
    }
    let n = a.slots.len().max(b.slots.len());
    for i in 0..n {
        let av = a.slots.get(i).copied();
        let bv = b.slots.get(i).copied();
        if av != bv {
            out.push(format!("slot[{i}] {av:?} -> {bv:?}"));
        }
    }
    out
}

pub fn snapshots_equal(a: &ExecSnapshot, b: &ExecSnapshot) -> bool {
    a.fingerprint() == b.fingerprint()
}
