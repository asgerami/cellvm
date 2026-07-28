//! Lightweight profiling counters for executor instrumentation.

use std::cell::Cell;

#[derive(Clone, Debug, Default)]
pub struct Counters {
    pub ops: Cell<u64>,
    pub calls: Cell<u64>,
    pub throws: Cell<u64>,
    pub interns: Cell<u64>,
    pub jumps: Cell<u64>,
}

impl Counters {
    pub fn bump_op(&self) {
        self.ops.set(self.ops.get() + 1);
    }
    pub fn bump_call(&self) {
        self.calls.set(self.calls.get() + 1);
    }
    pub fn bump_throw(&self) {
        self.throws.set(self.throws.get() + 1);
    }
    pub fn bump_intern(&self) {
        self.interns.set(self.interns.get() + 1);
    }
    pub fn bump_jump(&self) {
        self.jumps.set(self.jumps.get() + 1);
    }
    pub fn snapshot(&self) -> (u64, u64, u64, u64, u64) {
        (
            self.ops.get(),
            self.calls.get(),
            self.throws.get(),
            self.interns.get(),
            self.jumps.get(),
        )
    }
    pub fn render(&self) -> String {
        let (o, c, t, i, j) = self.snapshot();
        format!("ops={o} calls={c} throws={t} interns={i} jumps={j}")
    }
}

thread_local! {
    static GLOBAL: Counters = Counters::default();
}

pub fn with_global<R>(f: impl FnOnce(&Counters) -> R) -> R {
    GLOBAL.with(f)
}

pub fn reset_global() {
    GLOBAL.with(|c| {
        c.ops.set(0);
        c.calls.set(0);
        c.throws.set(0);
        c.interns.set(0);
        c.jumps.set(0);
    });
}
