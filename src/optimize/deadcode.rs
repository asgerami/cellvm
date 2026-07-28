//! Unreachable-block elimination planning (does not mutate modules in-place).

use std::collections::BTreeSet;

use crate::cfg::{self, Cfg};
use crate::module::Function;

#[derive(Clone, Debug, Default)]
pub struct Plan {
    pub keep: BTreeSet<usize>,
    pub drop_pcs: BTreeSet<usize>,
}

pub fn plan(f: &Function) -> Plan {
    let g = cfg::build(f);
    let keep = cfg::reachable(&g);
    let mut drop_pcs = BTreeSet::new();
    for b in &g.blocks {
        if !keep.contains(&b.id) {
            for pc in b.start..b.end {
                drop_pcs.insert(pc);
            }
        }
    }
    Plan { keep, drop_pcs }
}

pub fn unreachable_byte_count(f: &Function) -> usize {
    plan(f).drop_pcs.len()
}

pub fn cfg_density(f: &Function) -> f64 {
    let g = cfg::build(f);
    if g.blocks.is_empty() {
        return 0.0;
    }
    let edges: usize = g.blocks.iter().map(|b| b.succs.len()).sum();
    edges as f64 / g.blocks.len() as f64
}

pub fn summarize(f: &Function) -> String {
    let p = plan(f);
    format!(
        "keep_blocks={} drop_bytes={} density={:.2}",
        p.keep.len(),
        p.drop_pcs.len(),
        cfg_density(f)
    )
}

pub fn is_trivial(cfg: &Cfg) -> bool {
    cfg.blocks.len() <= 1 && !cfg::has_cycle(cfg)
}
