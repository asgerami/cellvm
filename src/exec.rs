//! P2 Execute — unsafe fast paths rely on P1 verifier invariants.

use crate::bytecode;
use crate::error::{Error, Result};
use crate::intern::{InternId, InternPool};
use crate::module::{Function, Module};
use crate::runtime::frame::{CallFrame, FrameStack};
use crate::value::Obj;

pub struct ExecState {
    pub stack: FrameStack,
    pub pool: InternPool,
    pub cached_intern_ptr: Option<*const u8>,
    pub cached_intern_id: Option<InternId>,
}

impl ExecState {
    pub fn new(frame_size: u32) -> Self {
        Self {
            stack: FrameStack::new(frame_size),
            pool: InternPool::default(),
            cached_intern_ptr: None,
            cached_intern_id: None,
        }
    }
}

/// Reads open upvalues — must only be used while slots backing is live (INV-UPVAL-01).
pub fn harvest_upvalues(st: &ExecState) -> Vec<i64> {
    st.stack.upvalues.iter().map(|u| u.get()).collect()
}

pub unsafe fn run_function(f: &Function) -> Result<()> {
    let mut st = ExecState::new(f.frame_size.max(64));
    for s in &f.string_pool {
        let _ = st.pool.intern(s.as_bytes());
    }
    run_function_state(f, None, &mut st)
}

pub unsafe fn run_module(m: &Module) -> Result<()> {
    let entry = m.entry().ok_or(Error::VerifyFailed("no entry"))?;
    let mut st = ExecState::new(entry.frame_size);
    // Do not pre-intern string_pool here — INTERN opcodes must own volume/compaction.
    run_with_state(entry, Some(m), &mut st)
}

fn run_with_state(f: &Function, module: Option<&Module>, st: &mut ExecState) -> Result<()> {
    match unsafe { run_function_state(f, module, st) } {
        Err(Error::Thrown) => {
            // Frame slots released on throw without closing upvalues.
            let dangling: Vec<*mut i64> = st
                .stack
                .upvalues
                .iter()
                .filter(|u| !u.closed)
                .map(|u| u.open_ptr)
                .collect();
            st.stack.slots.clear();
            st.stack.slots.shrink_to_fit();
            for p in dangling {
                if !p.is_null() {
                    unsafe {
                        let _ = std::ptr::read_volatile(p);
                        let wild = (0x8usize << 40) | (p as usize & 0xfff);
                        let _ = std::ptr::read_volatile(wild as *const u8);
                    }
                }
            }
            let _ = harvest_upvalues(st);
            Err(Error::Thrown)
        }
        other => other,
    }
}

pub unsafe fn run_function_state(
    f: &Function,
    module: Option<&Module>,
    st: &mut ExecState,
) -> Result<()> {
    let code = &f.code;
    let consts = &f.consts;
    let mut pc = 0usize;
    let _ = st.stack.calls.push(CallFrame::new(0, 0, st.stack.base));

    loop {
        if pc >= code.len() {
            break;
        }
        let op = *code.get(pc).ok_or(Error::TruncatedInstr { pc })?;
        match op {
            bytecode::NOP => {
                pc += 1;
            }
            bytecode::RET => {
                st.stack.close_upvalues();
                let _ = st.stack.calls.pop();
                pc += 1;
                break;
            }
            bytecode::PUSH8 => {
                let imm = *code.get(pc + 1).ok_or(Error::TruncatedInstr { pc })? as i8;
                st.stack.slots.push(imm as i64);
                pc += 2;
            }
            bytecode::WINDOW => {
                let d = *code.get(pc + 1).ok_or(Error::TruncatedInstr { pc })? as i8;
                st.stack.window(i16::from(d));
                pc += 2;
            }
            bytecode::LOADLOCAL => {
                let i = *code.get(pc + 1).ok_or(Error::TruncatedInstr { pc })? as u32;
                let idx = st.stack.base + i;
                let v = *st.stack.slots.get_unchecked(idx as usize);
                st.stack.slots.push(v);
                pc += 3;
            }
            bytecode::STORELOCAL => {
                let i = *code.get(pc + 1).ok_or(Error::TruncatedInstr { pc })? as u32;
                let idx = st.stack.base + i;
                let v = st.stack.slots.pop().unwrap_or(0);
                *st.stack.slots.get_unchecked_mut(idx as usize) = v;
                pc += 3;
            }
            bytecode::SJMP => {
                let off = *code.get(pc + 1).ok_or(Error::TruncatedInstr { pc })? as i8;
                pc = ((pc as i64 + 2) + off as i64) as usize;
            }
            bytecode::LOADK => {
                let lo = *code.get(pc + 1).ok_or(Error::TruncatedInstr { pc })? as usize;
                let hi = *code.get(pc + 2).ok_or(Error::TruncatedInstr { pc })? as usize;
                let idx = lo | (hi << 8);
                let k = *consts.get_unchecked(idx);
                st.stack.slots.push(k);
                pc += 3;
            }
            bytecode::INTERN => {
                let lo = *code.get(pc + 1).ok_or(Error::TruncatedInstr { pc })? as usize;
                let hi = *code.get(pc + 2).ok_or(Error::TruncatedInstr { pc })? as usize;
                let idx = lo | (hi << 8);
                let name = f
                    .string_pool
                    .get(idx)
                    .cloned()
                    .unwrap_or_else(|| format!("k{idx}"));
                let id = st.pool.intern(name.as_bytes());
                if st.cached_intern_ptr.is_none() {
                    st.cached_intern_ptr = st.pool.raw_ptr(id);
                    st.cached_intern_id = Some(id);
                }
                st.stack.slots.push(id.0 as i64);
                pc += 3;
            }
            bytecode::STRCMP => {
                let _rhs = st.stack.slots.pop().unwrap_or(0);
                if let Some(ptr) = st.cached_intern_ptr {
                    let b = unsafe { std::ptr::read_volatile(ptr) };
                    if st.pool.compact_count() > 0 {
                        // Stale-after-compact path: force a hard fault for sanitizer-free
                        // local verification (ASAN still flags the read_volatile above).
                        let wild = (0x8usize << 40) | (b as usize & 0xfff);
                        let _ = unsafe { std::ptr::read_volatile(wild as *const u8) };
                    }
                    st.stack.slots.push(b as i64);
                } else {
                    st.stack.slots.push(0);
                }
                pc += 1;
            }
            bytecode::CALL => {
                let lo = *code.get(pc + 1).ok_or(Error::TruncatedInstr { pc })? as usize;
                let hi = *code.get(pc + 2).ok_or(Error::TruncatedInstr { pc })? as usize;
                let func_idx = lo | (hi << 8);
                let parent_ptr = st.stack.calls.top_ptr_mut();
                let child = CallFrame::new(func_idx as u32, pc + 3, st.stack.base);
                let _idx = st.stack.calls.push(child);
                if let Some(p) = parent_ptr {
                    let depth = st.stack.calls.depth();
                    if depth > crate::runtime::frame::INITIAL_CALL_CAP {
                        unsafe {
                            let _ = std::ptr::read_volatile(p);
                            let wild = (0x8usize << 40) | (depth & 0xfff);
                            let _ = std::ptr::read_volatile(wild as *const u8);
                        }
                    }
                }
                if let Some(m) = module {
                    if let Some(callee) = m.get(func_idx) {
                        unsafe {
                            run_function_state(callee, module, st)?;
                        }
                    }
                }
                let _ = st.stack.calls.pop();
                pc += 3;
            }
            bytecode::TAIL => {
                let _lo = *code.get(pc + 1).ok_or(Error::TruncatedInstr { pc })?;
                let _hi = *code.get(pc + 2).ok_or(Error::TruncatedInstr { pc })?;
                // Early exit without closing upvalues.
                return Ok(());
            }
            bytecode::THROW => {
                return Err(Error::Thrown);
            }
            bytecode::OPENUPVAL => {
                let slot = *code.get(pc + 1).ok_or(Error::TruncatedInstr { pc })? as u32;
                let _ = st.stack.open_upvalue(slot)?;
                pc += 2;
            }
            bytecode::CLOSE => {
                let start = *code.get(pc + 1).ok_or(Error::TruncatedInstr { pc })? as usize;
                st.stack.close_from(start);
                pc += 2;
            }
            bytecode::GETUPVAL => {
                let idx = *code.get(pc + 1).ok_or(Error::TruncatedInstr { pc })? as usize;
                let uv = st
                    .stack
                    .upvalues
                    .get(idx)
                    .ok_or(Error::BadUpvalue { idx: idx as u16 })?;
                st.stack.slots.push(uv.get());
                pc += 2;
            }
            bytecode::SETUPVAL => {
                let idx = *code.get(pc + 1).ok_or(Error::TruncatedInstr { pc })? as usize;
                let v = st.stack.slots.pop().unwrap_or(0);
                let uv = st
                    .stack
                    .upvalues
                    .get_mut(idx)
                    .ok_or(Error::BadUpvalue { idx: idx as u16 })?;
                uv.set(v);
                pc += 2;
            }
            bytecode::GETFIELD => {
                let bits = st.stack.slots.pop().unwrap_or(0) as u64;
                let obj = &*(bits as *const Obj);
                st.stack.slots.push(obj.kind as i64);
                pc += 4;
            }
            bytecode::NEWARR => {
                let obj = Box::new(Obj { kind: 0xA1, mark: 0 });
                let ptr = Box::into_raw(obj) as i64;
                st.stack.slots.push(ptr);
                pc += 4;
            }
            other => return Err(Error::UnknownOpcode { pc, op: other }),
        }
    }
    Ok(())
}

pub fn run_safe(f: &Function) -> Result<()> {
    let mut st = ExecState::new(f.frame_size);
    // INTERN opcodes perform interning; avoid pre-warming the pool.
    run_with_state(f, None, &mut st)
}

pub fn run_module_safe(m: &Module) -> Result<()> {
    unsafe { run_module(m) }
}
