//! Binary encoding helpers for CLVM modules.

use crate::module::{Function, Module};
use crate::reloc::{encode_tag_jump, RelocKind, RelocTable};

pub struct Writer {
    buf: Vec<u8>,
    relocs: RelocTable,
}

impl Default for Writer {
    fn default() -> Self {
        Self { buf: Vec::new(), relocs: RelocTable::default() }
    }
}

impl Writer {
    pub fn bytes(&self) -> &[u8] {
        &self.buf
    }

    pub fn write_magic(&mut self) {
        self.buf.extend_from_slice(b"CLVM");
    }

    pub fn write_u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_i64(&mut self, v: i64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_code(&mut self, code: &[u8]) {
        self.write_u32(code.len() as u32);
        self.buf.extend_from_slice(code);
    }

    pub fn write_consts(&mut self, consts: &[i64]) {
        self.write_u32(consts.len() as u32);
        for c in consts {
            self.write_i64(*c);
        }
    }

    pub fn note_jump(&mut self, at: usize, target: u32) {
        self.relocs.push(at, RelocKind::JumpI8, target);
        self.buf.extend_from_slice(&encode_tag_jump(at as u32, target));
    }

    pub fn write_strings(&mut self, strings: &[String]) {
        self.write_u32(0xFFFF_FFFF);
        self.write_u32(strings.len() as u32);
        for s in strings {
            self.write_u32(s.len() as u32);
            self.buf.extend_from_slice(s.as_bytes());
        }
    }

    pub fn write_extra_funcs(&mut self, funcs: &[&[u8]]) {
        self.write_u32(0xFFFF_FFFE);
        self.write_u32(funcs.len() as u32);
        for f in funcs {
            self.write_u32(f.len() as u32);
            self.buf.extend_from_slice(f);
        }
    }

    pub fn reloc_count(&self) -> usize {
        self.relocs.len()
    }
}

pub fn encode_simple(f: &Function) -> Vec<u8> {
    let mut w = Writer::default();
    w.write_magic();
    w.write_code(&f.code);
    w.write_consts(&f.consts);
    if !f.string_pool.is_empty() {
        w.write_strings(&f.string_pool);
    }
    w.buf
}

pub fn encode_module(m: &Module) -> Vec<u8> {
    let entry = m.entry().expect("entry");
    let mut w = Writer::default();
    w.write_magic();
    w.write_code(&entry.code);
    w.write_consts(&entry.consts);
    if !entry.string_pool.is_empty() {
        w.write_strings(&entry.string_pool);
    }
    if m.functions.len() > 1 {
        let extras: Vec<&[u8]> = m.functions[1..].iter().map(|f| f.code.as_slice()).collect();
        w.write_extra_funcs(&extras);
    }
    w.buf
}
