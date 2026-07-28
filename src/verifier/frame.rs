//! Frame / local-window verification.
//!
//! Tracks WINDOW base and checks LOADLOCAL/STORELOCAL slot indices against
//! `max_locals`. Effective address is `base + slot` at runtime.

use crate::bytecode;
use crate::error::{Error, Result};
use crate::module::Function;

#[derive(Default)]
pub struct FrameFacts {
    pub base_at: Vec<u32>,
    pub max_base: u32,
    pub window_ops: usize,
}

pub struct FrameVerifier<'a> {
    func: &'a Function,
}

impl<'a> FrameVerifier<'a> {
    pub fn new(func: &'a Function) -> Self {
        Self { func }
    }

    pub fn verify(&self) -> Result<FrameFacts> {
        let mut facts = FrameFacts::default();
        let code = &self.func.code;
        let mut pc = 0usize;
        let mut base: u32 = 0;
        while pc < code.len() {
            facts.base_at.push(base);
            let op = *code.get(pc).ok_or(Error::TruncatedInstr { pc })?;
            match op {
                bytecode::WINDOW => {
                    let delta = *code.get(pc + 1).ok_or(Error::TruncatedInstr { pc })? as i8;
                    let next = (base as i64) + (delta as i64);
                    if next < 0 {
                        return Err(Error::WindowUnderflow { pc });
                    }
                    base = next as u32;
                    facts.window_ops += 1;
                    facts.max_base = facts.max_base.max(base);
                    pc += 2;
                }
                bytecode::LOADLOCAL | bytecode::STORELOCAL => {
                    let slot = u16::from(*code.get(pc + 1).ok_or(Error::TruncatedInstr { pc })?);
                    // Slot index is checked against max_locals. WINDOW base is tracked
                    // for facts / tooling; runtime applies base+slot against the frame.
                    if slot as u32 >= self.func.max_locals as u32 {
                        return Err(Error::LocalOutOfRange { pc, slot });
                    }
                    pc += 3;
                }
                _ => {
                    let len = bytecode::instr_len(op).ok_or(Error::UnknownOpcode { pc, op })?;
                    pc += len;
                }
            }
        }
        Ok(facts)
    }
}
