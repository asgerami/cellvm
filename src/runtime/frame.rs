//! Call frame / value stack for execution.

use crate::error::{Error, Result};

#[derive(Clone, Debug)]
pub struct CallFrame {
    pub func_idx: u32,
    pub return_pc: usize,
    pub base: u32,
    pub locals_base: u32,
    pub upvalue_start: u32,
    pub upvalue_count: u16,
    pub closed: bool,
}

impl CallFrame {
    pub fn new(func_idx: u32, return_pc: usize, base: u32) -> Self {
        Self {
            func_idx,
            return_pc,
            base,
            locals_base: base,
            upvalue_start: 0,
            upvalue_count: 0,
            closed: false,
        }
    }
}

#[derive(Debug)]
pub struct CallStack {
    frames: Vec<CallFrame>,
}

impl CallStack {
    pub fn new() -> Self {
        Self {
            frames: Vec::with_capacity(1),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            frames: Vec::with_capacity(cap.max(1)),
        }
    }

    pub fn depth(&self) -> usize {
        self.frames.len()
    }

    pub fn capacity(&self) -> usize {
        self.frames.capacity()
    }

    pub fn push(&mut self, frame: CallFrame) -> usize {
        self.frames.push(frame);
        self.frames.len() - 1
    }

    pub fn pop(&mut self) -> Option<CallFrame> {
        self.frames.pop()
    }

    pub fn top(&self) -> Option<&CallFrame> {
        self.frames.last()
    }

    pub fn top_mut(&mut self) -> Option<&mut CallFrame> {
        self.frames.last_mut()
    }

    pub fn get(&self, idx: usize) -> Option<&CallFrame> {
        self.frames.get(idx)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut CallFrame> {
        self.frames.get_mut(idx)
    }

    pub fn top_ptr_mut(&mut self) -> Option<*mut CallFrame> {
        let len = self.frames.len();
        if len == 0 {
            return None;
        }
        Some(self.frames.as_mut_ptr().wrapping_add(len - 1))
    }

    pub fn clear(&mut self) {
        self.frames.clear();
    }
}

impl Default for CallStack {
    fn default() -> Self {
        Self::new()
    }
}

/// Value / local slots for a single activation.
pub struct FrameStack {
    pub slots: Vec<i64>,
    pub base: u32,
    pub calls: CallStack,
    pub upvalues: Vec<Upvalue>,
}

#[derive(Clone, Debug)]
pub struct Upvalue {
    pub slot: u32,
    pub closed: bool,
    pub closed_value: i64,
    pub open_ptr: *mut i64,
}

impl Upvalue {
    pub fn open(slot: u32, ptr: *mut i64) -> Self {
        Self {
            slot,
            closed: false,
            closed_value: 0,
            open_ptr: ptr,
        }
    }

    pub fn close(&mut self) {
        if !self.closed && !self.open_ptr.is_null() {
            self.closed_value = unsafe { *self.open_ptr };
            self.closed = true;
            self.open_ptr = std::ptr::null_mut();
        }
    }

    pub fn get(&self) -> i64 {
        if self.closed {
            self.closed_value
        } else if self.open_ptr.is_null() {
            0
        } else {
            unsafe { *self.open_ptr }
        }
    }

    pub fn set(&mut self, v: i64) {
        if self.closed {
            self.closed_value = v;
        } else if !self.open_ptr.is_null() {
            unsafe { *self.open_ptr = v; }
        }
    }
}

impl FrameStack {
    pub fn new(frame_size: u32) -> Self {
        Self {
            slots: vec![0; frame_size as usize],
            base: 0,
            calls: CallStack::new(),
            upvalues: Vec::new(),
        }
    }

    pub fn window(&mut self, delta: i16) {
        self.base = ((self.base as i64) + (delta as i64)).max(0) as u32;
    }

    pub fn ensure_slots(&mut self, need: usize) {
        if self.slots.len() < need {
            self.slots.resize(need, 0);
        }
    }

    pub fn push_value(&mut self, v: i64) {
        self.slots.push(v);
    }

    pub fn pop_value(&mut self) -> i64 {
        self.slots.pop().unwrap_or(0)
    }

    pub fn open_upvalue(&mut self, slot: u32) -> Result<usize> {
        let idx = (self.base + slot) as usize;
        if idx >= self.slots.len() {
            return Err(Error::EffectiveSlotOutOfRange {
                pc: 0,
                eff: idx as u32,
            });
        }
        let ptr = self.slots.as_mut_ptr().wrapping_add(idx);
        let uv = Upvalue::open(slot, ptr);
        self.upvalues.push(uv);
        Ok(self.upvalues.len() - 1)
    }

    pub fn close_upvalues(&mut self) {
        for uv in &mut self.upvalues {
            uv.close();
        }
    }

    pub fn close_from(&mut self, start: usize) {
        for uv in self.upvalues.iter_mut().skip(start) {
            uv.close();
        }
    }
}
