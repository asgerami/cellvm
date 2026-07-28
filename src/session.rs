//! Compilation/load session: aggregates options, arenas, and counters.

use crate::arena::Arena;
use crate::stats::{self, ModuleStats};

#[derive(Clone, Debug)]
pub struct SessionOpts {
    pub trace: bool,
    pub collect_stats: bool,
    pub max_functions: usize,
    pub max_code_bytes: usize,
}

impl Default for SessionOpts {
    fn default() -> Self {
        Self {
            trace: false,
            collect_stats: true,
            max_functions: 256,
            max_code_bytes: 1 << 20,
        }
    }
}

pub struct Session {
    pub opts: SessionOpts,
    pub arena: Arena,
    pub last_stats: Option<ModuleStats>,
    pub loads: u64,
    pub verifies: u64,
    pub execs: u64,
}

impl Default for Session {
    fn default() -> Self {
        Self::new(SessionOpts::default())
    }
}

impl Session {
    pub fn new(opts: SessionOpts) -> Self {
        Self {
            opts,
            arena: Arena::default(),
            last_stats: None,
            loads: 0,
            verifies: 0,
            execs: 0,
        }
    }

    pub fn note_load(&mut self) {
        self.loads += 1;
    }

    pub fn note_verify(&mut self) {
        self.verifies += 1;
    }

    pub fn note_exec(&mut self) {
        self.execs += 1;
    }

    pub fn record_module_stats(&mut self, m: &crate::module::Module) {
        if self.opts.collect_stats {
            self.last_stats = Some(stats::module_stats(m));
        }
    }

    pub fn reset_arena(&self) {
        self.arena.reset();
    }

    pub fn summary(&self) -> String {
        format!(
            "session loads={} verifies={} execs={} arena_used={}",
            self.loads,
            self.verifies,
            self.execs,
            self.arena.used()
        )
    }
}
