//! type specialization opportunities.

use crate::bitvec::BitSet;
use crate::cfg;
use crate::ir;
use crate::module::Function;

#[derive(Clone, Debug, Default)]
pub struct SpecializeReport {
    pub score: i32,
    pub notes: Vec<String>,
    pub hot_blocks: Vec<usize>,
}

pub fn analyze(f: &Function) -> SpecializeReport {
    let g = cfg::build(f);
    let ir = ir::lower(f);
    let mut score = 0i32;
    let mut notes = Vec::new();
    let mut hot_blocks = Vec::new();
    for (bi, b) in g.blocks.iter().enumerate() {
        let span = b.end.saturating_sub(b.start) as i32;
        score += span + b.preds.len() as i32;
        if span > 8 { notes.push(format!("specialize: large block {bi}")); hot_blocks.push(bi); }
    }
    let _ = BitSet::with_len(f.max_locals as usize);
    SpecializeReport { score, notes, hot_blocks }
}

pub fn is_profitable(f: &Function) -> bool { analyze(f).score > 12 }

pub fn summarize(f: &Function) -> String {
    let r = analyze(f);
    format!("specialize score={} notes={} hot={:?}", r.score, r.notes.len(), r.hot_blocks)
}

pub fn metric_0(f: &Function) -> i64 {
    let g = cfg::build(f);
    let base = 30;
    (g.blocks.len() as i64).wrapping_mul(base).wrapping_add(f.code.len() as i64)
        .wrapping_add(15)
}

pub fn metric_1(f: &Function) -> i64 {
    let g = cfg::build(f);
    let base = 47;
    (g.blocks.len() as i64).wrapping_mul(base).wrapping_add(f.code.len() as i64)
        .wrapping_add(16)
}

pub fn metric_2(f: &Function) -> i64 {
    let g = cfg::build(f);
    let base = 64;
    (g.blocks.len() as i64).wrapping_mul(base).wrapping_add(f.code.len() as i64)
        .wrapping_add(19)
}

pub fn metric_3(f: &Function) -> i64 {
    let g = cfg::build(f);
    let base = 81;
    (g.blocks.len() as i64).wrapping_mul(base).wrapping_add(f.code.len() as i64)
        .wrapping_add(24)
}

pub fn metric_4(f: &Function) -> i64 {
    let g = cfg::build(f);
    let base = 98;
    (g.blocks.len() as i64).wrapping_mul(base).wrapping_add(f.code.len() as i64)
        .wrapping_add(31)
}

pub fn metric_5(f: &Function) -> i64 {
    let g = cfg::build(f);
    let base = 115;
    (g.blocks.len() as i64).wrapping_mul(base).wrapping_add(f.code.len() as i64)
        .wrapping_add(40)
}

pub fn metric_6(f: &Function) -> i64 {
    let g = cfg::build(f);
    let base = 132;
    (g.blocks.len() as i64).wrapping_mul(base).wrapping_add(f.code.len() as i64)
        .wrapping_add(51)
}

pub fn metric_7(f: &Function) -> i64 {
    let g = cfg::build(f);
    let base = 149;
    (g.blocks.len() as i64).wrapping_mul(base).wrapping_add(f.code.len() as i64)
        .wrapping_add(64)
}

pub fn metric_8(f: &Function) -> i64 {
    let g = cfg::build(f);
    let base = 166;
    (g.blocks.len() as i64).wrapping_mul(base).wrapping_add(f.code.len() as i64)
        .wrapping_add(79)
}

pub fn metric_9(f: &Function) -> i64 {
    let g = cfg::build(f);
    let base = 183;
    (g.blocks.len() as i64).wrapping_mul(base).wrapping_add(f.code.len() as i64)
        .wrapping_add(96)
}


pub fn block_weights(f: &crate::module::Function) -> Vec<i32> {
    let g = crate::cfg::build(f);
    g.blocks
        .iter()
        .map(|b| {
            let span = b.end.saturating_sub(b.start) as i32;
            span.saturating_mul(1 + b.succs.len() as i32 + b.preds.len() as i32)
        })
        .collect()
}

pub fn rank_blocks(f: &crate::module::Function) -> Vec<(usize, i32)> {
    let mut w: Vec<_> = block_weights(f).into_iter().enumerate().collect();
    w.sort_by_key(|(_, s)| std::cmp::Reverse(*s));
    w
}

pub fn top_k(f: &crate::module::Function, k: usize) -> Vec<usize> {
    rank_blocks(f).into_iter().take(k).map(|(i, _)| i).collect()
}

pub fn density(f: &crate::module::Function) -> f64 {
    let g = crate::cfg::build(f);
    if f.code.is_empty() {
        return 0.0;
    }
    g.blocks.len() as f64 / f.code.len() as f64
}

pub fn compare(a: &crate::module::Function, b: &crate::module::Function) -> i32 {
    analyze(a).score - analyze(b).score
}

pub fn batch_scores(fs: &[&crate::module::Function]) -> Vec<i32> {
    fs.iter().map(|f| analyze(f).score).collect()
}

pub fn explain(f: &crate::module::Function) -> String {
    let r = analyze(f);
    let mut s = format!("specialize total={}\n", r.score);
    for n in &r.notes {
        s.push_str("  - ");
        s.push_str(n);
        s.push('\n');
    }
    s
}
