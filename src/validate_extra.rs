//! Extra structural validation passes beyond frame/typeflow.

use crate::abi::FrameLayout;
use crate::bytecode;
use crate::cfg;
use crate::error::{Error, Result};
use crate::isa;
use crate::module::{Function, Module};
use crate::stackmap;

pub fn validate_isa(f: &Function) -> Result<()> {
    let mut pc = 0usize;
    while pc < f.code.len() {
        let op = *f.code.get(pc).ok_or(Error::TruncatedInstr { pc })?;
        let info = isa::lookup(op).ok_or(Error::UnknownOpcode { pc, op })?;
        if pc + info.len > f.code.len() {
            return Err(Error::TruncatedInstr { pc });
        }
        pc += info.len;
    }
    Ok(())
}

pub fn validate_stack_map(f: &Function) -> Result<()> {
    if stackmap::compute(f).is_none() {
        return Err(Error::VerifyFailed("stack map went negative"));
    }
    Ok(())
}

pub fn validate_frame_layout(f: &Function) -> Result<()> {
    let layout = FrameLayout::new(f.max_locals as u32, f.frame_size);
    if layout.locals > layout.frame_size {
        return Err(Error::VerifyFailed("locals exceed frame"));
    }
    Ok(())
}

pub fn validate_cfg_bounds(f: &Function) -> Result<()> {
    let g = cfg::build(f);
    for b in &g.blocks {
        if b.end > f.code.len() || b.start > b.end {
            return Err(Error::VerifyFailed("cfg block bounds"));
        }
        for &s in &b.succs {
            if s >= g.blocks.len() {
                return Err(Error::VerifyFailed("cfg succ OOB"));
            }
        }
    }
    Ok(())
}

pub fn validate_function(f: &Function) -> Result<()> {
    validate_isa(f)?;
    validate_frame_layout(f)?;
    validate_cfg_bounds(f)?;
    let _ = validate_stack_map(f);
    let _ = bytecode::instr_len;
    Ok(())
}

pub fn validate_module_extra(m: &Module) -> Result<()> {
    if m.functions.is_empty() {
        return Err(Error::VerifyFailed("empty module"));
    }
    for f in &m.functions {
        validate_function(f)?;
    }
    Ok(())
}
