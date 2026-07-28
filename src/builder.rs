//! Program builder for constructing CLVM functions from typed ops.

use crate::bytecode;
use crate::module::{Function, Module};

#[derive(Clone, Debug, Default)]
pub struct ProgramBuilder {
    code: Vec<u8>,
    consts: Vec<i64>,
    strings: Vec<String>,
    max_locals: u16,
    frame_size: u32,
    name: String,
    labels: Vec<(String, usize)>,
    fixups: Vec<(usize, String)>,
}

impl ProgramBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            max_locals: 8,
            frame_size: 16,
            ..Self::default()
        }
    }

    pub fn locals(mut self, n: u16) -> Self {
        self.max_locals = n;
        self
    }

    pub fn frame(mut self, n: u32) -> Self {
        self.frame_size = n;
        self
    }

    pub fn label(&mut self, name: impl Into<String>) {
        self.labels.push((name.into(), self.code.len()));
    }

    pub fn nop(&mut self) {
        self.code.push(bytecode::NOP);
    }

    pub fn ret(&mut self) {
        self.code.push(bytecode::RET);
    }

    pub fn throw(&mut self) {
        self.code.push(bytecode::THROW);
    }

    pub fn push8(&mut self, v: i8) {
        self.code.push(bytecode::PUSH8);
        self.code.push(v as u8);
    }

    pub fn window(&mut self, delta: i8) {
        self.code.push(bytecode::WINDOW);
        self.code.push(delta as u8);
    }

    pub fn sjmp_label(&mut self, label: impl Into<String>) {
        self.code.push(bytecode::SJMP);
        self.fixups.push((self.code.len(), label.into()));
        self.code.push(0);
    }

    pub fn loadk(&mut self, v: i64) {
        let idx = self.intern_const(v);
        self.code.push(bytecode::LOADK);
        self.code.extend_from_slice(&idx.to_le_bytes());
    }

    pub fn loadlocal(&mut self, slot: u16) {
        self.code.push(bytecode::LOADLOCAL);
        self.code.extend_from_slice(&slot.to_le_bytes());
    }

    pub fn storelocal(&mut self, slot: u16) {
        self.code.push(bytecode::STORELOCAL);
        self.code.extend_from_slice(&slot.to_le_bytes());
    }

    pub fn intern_str(&mut self, s: impl Into<String>) {
        let idx = self.intern_string(s.into());
        self.code.push(bytecode::INTERN);
        self.code.extend_from_slice(&idx.to_le_bytes());
    }

    pub fn strcmp(&mut self) {
        self.code.push(bytecode::STRCMP);
    }

    pub fn call(&mut self, idx: u16) {
        self.code.push(bytecode::CALL);
        self.code.extend_from_slice(&idx.to_le_bytes());
    }

    pub fn tail(&mut self, idx: u16) {
        self.code.push(bytecode::TAIL);
        self.code.extend_from_slice(&idx.to_le_bytes());
    }

    pub fn close(&mut self, start: u8) {
        self.code.push(bytecode::CLOSE);
        self.code.push(start);
    }

    pub fn openupval(&mut self, slot: u8) {
        self.code.push(bytecode::OPENUPVAL);
        self.code.push(slot);
    }

    pub fn getupval(&mut self, idx: u8) {
        self.code.push(bytecode::GETUPVAL);
        self.code.push(idx);
    }

    pub fn setupval(&mut self, idx: u8) {
        self.code.push(bytecode::SETUPVAL);
        self.code.push(idx);
    }

    pub fn newarr(&mut self) {
        self.code.push(bytecode::NEWARR);
        self.code.extend_from_slice(&[0, 0, 0]);
    }

    pub fn getfield(&mut self, bits: u32) {
        self.code.push(bytecode::GETFIELD);
        self.code.push((bits & 0xff) as u8);
        self.code.push(((bits >> 8) & 0xff) as u8);
        self.code.push(((bits >> 16) & 0xff) as u8);
    }

    fn intern_const(&mut self, v: i64) -> u16 {
        if let Some(i) = self.consts.iter().position(|c| *c == v) {
            return i as u16;
        }
        self.consts.push(v);
        (self.consts.len() - 1) as u16
    }

    fn intern_string(&mut self, s: String) -> u16 {
        if let Some(i) = self.strings.iter().position(|x| *x == s) {
            return i as u16;
        }
        self.strings.push(s);
        (self.strings.len() - 1) as u16
    }

    pub fn finish(mut self) -> Result<Function, &'static str> {
        for (at, lab) in &self.fixups {
            let target = self
                .labels
                .iter()
                .find(|(n, _)| n == lab)
                .map(|(_, pc)| *pc)
                .ok_or("missing label")?;
            let next = *at as i64 + 1;
            let rel = target as i64 - next;
            let stored = rel as i8;
            if stored as i64 != rel {
                return Err("jump too far");
            }
            self.code[*at] = stored as u8;
        }
        Ok(Function {
            code: self.code,
            max_locals: self.max_locals,
            frame_size: self.frame_size,
            consts: self.consts,
            string_pool: self.strings,
            upvalues: Vec::new(),
            name: self.name,
        })
    }

    pub fn finish_module(self) -> Result<Module, &'static str> {
        Ok(Module {
            functions: vec![self.finish()?],
        })
    }

    pub fn code_len(&self) -> usize {
        self.code.len()
    }

    pub fn peek_code(&self) -> &[u8] {
        &self.code
    }
}

pub fn hello_module() -> Module {
    let mut b = ProgramBuilder::new("main").locals(4).frame(16);
    b.push8(1);
    b.ret();
    b.finish_module().expect("hello")
}

pub fn countdown_module(n: i8) -> Module {
    let mut b = ProgramBuilder::new("main").locals(4).frame(16);
    b.push8(n);
    b.storelocal(0);
    b.label("loop");
    b.loadlocal(0);
    b.push8(0);
    // compare via host later; just window/ret skeleton
    b.window(0);
    b.loadlocal(0);
    b.push8(1);
    b.storelocal(0);
    b.sjmp_label("done");
    b.sjmp_label("loop");
    b.label("done");
    b.ret();
    b.finish_module().unwrap_or_else(|_| hello_module())
}
