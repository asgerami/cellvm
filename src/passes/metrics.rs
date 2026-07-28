use crate::cfg;
use crate::ir;
use crate::module::{Function, Module};
use crate::optimize::peephole;

pub fn function_metric_vector(f: &Function) -> Vec<u64> {
    let g = cfg::build(f);
    let ir = ir::lower(f);
    vec![
        f.code.len() as u64,
        g.blocks.len() as u64,
        cfg::reachable(&g).len() as u64,
        u64::from(cfg::has_cycle(&g)),
        ir::count_calls(&ir) as u64,
        ir::count_throws(&ir) as u64,
        peephole::estimate_hotness(&f.code) as u64,
        ir::windows(&ir).len() as u64,
    ]
}

pub fn module_metric_vector(m: &Module) -> Vec<u64> {
    let mut acc = vec![0u64; 8];
    for f in &m.functions {
        for (a, b) in acc.iter_mut().zip(function_metric_vector(f)) {
            *a = a.saturating_add(b);
        }
    }
    acc
}

pub fn entropy_hint(code: &[u8]) -> f64 {
    if code.is_empty() {
        return 0.0;
    }
    let mut hist = [0u64; 256];
    for &b in code {
        hist[b as usize] += 1;
    }
    let n = code.len() as f64;
    let mut h = 0.0;
    for c in hist {
        if c > 0 {
            let p = c as f64 / n;
            h -= p * p.log2();
        }
    }
    h
}
