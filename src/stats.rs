//! Module-level statistics for tooling and fuzz coverage hints.

use crate::cfg;
use crate::disasm;
use crate::module::{Function, Module};
use crate::optimize::{constfold, deadcode, peephole};
use crate::stackmap;

#[derive(Clone, Debug, Default)]
pub struct FunctionStats {
    pub name: String,
    pub code_len: usize,
    pub consts: usize,
    strings: usize,
    pub blocks: usize,
    pub max_stack: i32,
    pub hotness: u32,
    pub known_imms: usize,
}

#[derive(Clone, Debug, Default)]
pub struct ModuleStats {
    pub functions: Vec<FunctionStats>,
    pub total_code: usize,
}

pub fn function_stats(f: &Function) -> FunctionStats {
    let g = cfg::build(f);
    FunctionStats {
        name: f.name.clone(),
        code_len: f.code.len(),
        consts: f.consts.len(),
        strings: f.string_pool.len(),
        blocks: g.blocks.len(),
        max_stack: stackmap::max_depth_of(f),
        hotness: peephole::estimate_hotness(&f.code),
        known_imms: constfold::known_imm_count(f),
    }
}

pub fn module_stats(m: &Module) -> ModuleStats {
    let functions: Vec<_> = m.functions.iter().map(function_stats).collect();
    let total_code = functions.iter().map(|f| f.code_len).sum();
    ModuleStats { functions, total_code }
}

pub fn render(m: &Module) -> String {
    let st = module_stats(m);
    let mut s = format!("module functions={} total_code={}\n", st.functions.len(), st.total_code);
    for f in &st.functions {
        s.push_str(&format!(
            "  {} code={} blocks={} stack={} hot={} imms={} unreachable={}\n",
            f.name,
            f.code_len,
            f.blocks,
            f.max_stack,
            f.hotness,
            f.known_imms,
            deadcode::unreachable_byte_count(
                m.functions.iter().find(|x| x.name == f.name).unwrap()
            ),
        ));
        s.push_str(&format!("    approx_size={}\n", disasm::approx_size(
            m.functions.iter().find(|x| x.name == f.name).unwrap()
        )));
    }
    s
}
