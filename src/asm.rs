//! Text assembler front-end (H4).

use std::collections::HashMap;

use crate::bytecode;
use crate::error::{Error, Result};
use crate::module::{Function, Module};

#[derive(Default)]
struct AsmState {
    code: Vec<u8>,
    consts: Vec<i64>,
    strings: Vec<String>,
    labels: HashMap<String, usize>,
    fixups: Vec<(usize, String)>,
    max_locals: u16,
    frame_size: u32,
}

impl AsmState {
    fn emit_op(&mut self, op: u8) {
        self.code.push(op);
    }

    fn emit_u8(&mut self, v: u8) {
        self.code.push(v);
    }

    fn emit_u16(&mut self, v: u16) {
        self.code.extend_from_slice(&v.to_le_bytes());
    }

    fn add_const(&mut self, v: i64) -> u16 {
        if let Some(i) = self.consts.iter().position(|c| *c == v) {
            return i as u16;
        }
        self.consts.push(v);
        (self.consts.len() - 1) as u16
    }

    fn add_string(&mut self, s: String) -> u16 {
        if let Some(i) = self.strings.iter().position(|x| *x == s) {
            return i as u16;
        }
        self.strings.push(s);
        (self.strings.len() - 1) as u16
    }
}

pub fn assemble(text: &str) -> Result<Module> {
    let mut st = AsmState {
        max_locals: 4,
        frame_size: 64,
        ..AsmState::default()
    };

    for (lineno, raw) in text.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix(".locals ") {
            st.max_locals = rest.trim().parse().unwrap_or(4);
            continue;
        }
        if let Some(rest) = line.strip_prefix(".frame ") {
            st.frame_size = rest.trim().parse().unwrap_or(64);
            continue;
        }
        if let Some(name) = line.strip_suffix(':') {
            st.labels.insert(name.trim().to_string(), st.code.len());
            continue;
        }
        let mut parts = line.split_whitespace();
        let mnem = parts.next().ok_or(Error::VerifyFailed("asm empty"))?;
        match mnem {
            "nop" => st.emit_op(bytecode::NOP),
            "ret" => st.emit_op(bytecode::RET),
            "throw" => st.emit_op(bytecode::THROW),
            "strcmp" => st.emit_op(bytecode::STRCMP),
            "push8" => {
                let v: i8 = parts
                    .next()
                    .ok_or(Error::VerifyFailed("push8"))?
                    .parse()
                    .map_err(|_| Error::VerifyFailed("push8 parse"))?;
                st.emit_op(bytecode::PUSH8);
                st.emit_u8(v as u8);
            }
            "window" => {
                let v: i8 = parts
                    .next()
                    .ok_or(Error::VerifyFailed("window"))?
                    .parse()
                    .map_err(|_| Error::VerifyFailed("window parse"))?;
                st.emit_op(bytecode::WINDOW);
                st.emit_u8(v as u8);
            }
            "sjmp" => {
                let lab = parts.next().ok_or(Error::VerifyFailed("sjmp"))?;
                st.emit_op(bytecode::SJMP);
                st.fixups.push((st.code.len(), lab.to_string()));
                st.emit_u8(0);
            }
            "loadk" => {
                let v: i64 = parts
                    .next()
                    .ok_or(Error::VerifyFailed("loadk"))?
                    .parse()
                    .map_err(|_| Error::VerifyFailed("loadk parse"))?;
                let idx = st.add_const(v);
                st.emit_op(bytecode::LOADK);
                st.emit_u16(idx);
            }
            "loadlocal" | "storelocal" => {
                let slot: u8 = parts
                    .next()
                    .ok_or(Error::VerifyFailed("local"))?
                    .parse()
                    .map_err(|_| Error::VerifyFailed("local parse"))?;
                st.emit_op(if mnem == "loadlocal" {
                    bytecode::LOADLOCAL
                } else {
                    bytecode::STORELOCAL
                });
                st.emit_u8(slot);
                st.emit_u8(0);
            }
            "intern" => {
                let s = parts.next().ok_or(Error::VerifyFailed("intern"))?;
                let idx = st.add_string(s.to_string());
                st.emit_op(bytecode::INTERN);
                st.emit_u16(idx);
            }
            "call" => {
                let idx: u16 = parts
                    .next()
                    .ok_or(Error::VerifyFailed("call"))?
                    .parse()
                    .map_err(|_| Error::VerifyFailed("call parse"))?;
                st.emit_op(bytecode::CALL);
                st.emit_u16(idx);
            }
            "tail" => {
                let idx: u16 = parts
                    .next()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);
                st.emit_op(bytecode::TAIL);
                st.emit_u16(idx);
            }
            "close" => {
                let start: u8 = parts.next().unwrap_or("0").parse().unwrap_or(0);
                st.emit_op(bytecode::CLOSE);
                st.emit_u8(start);
            }
            "getupval" | "setupval" => {
                let idx: u8 = parts.next().unwrap_or("0").parse().unwrap_or(0);
                st.emit_op(if mnem == "getupval" {
                    bytecode::GETUPVAL
                } else {
                    bytecode::SETUPVAL
                });
                st.emit_u8(idx);
            }
            "newarr" => {
                st.emit_op(bytecode::NEWARR);
                st.emit_u8(0);
                st.emit_u8(0);
                st.emit_u8(0);
            }
            "openupval" => {
                let slot: u8 = parts.next().unwrap_or("0").parse().unwrap_or(0);
                st.emit_op(bytecode::OPENUPVAL);
                st.emit_u8(slot);
            }
            "getfield" => {
                st.emit_op(bytecode::GETFIELD);
                st.emit_u8(0);
                st.emit_u8(0);
                st.emit_u8(0);
            }
            other => {
                let msg = format!("asm unknown {other} @ {lineno}");
                return Err(Error::VerifyFailed(Box::leak(msg.into_boxed_str())));
            }
        }
    }

    for (at, lab) in &st.fixups {
        let target = *st
            .labels
            .get(lab)
            .ok_or(Error::VerifyFailed("missing label"))?;
        let next = *at as i64 + 1;
        let rel = target as i64 - next;
        let stored = rel as i8;
        if stored as i64 != rel {
            return Err(Error::OffsetTruncation { rel });
        }
        st.code[*at] = stored as u8;
    }

    let func = Function {
        code: st.code,
        max_locals: st.max_locals,
        frame_size: st.frame_size,
        consts: st.consts,
        string_pool: st.strings,
        upvalues: Vec::new(),
        name: "main".into(),
    };
    Ok(Module {
        functions: vec![func],
    })
}

pub fn disassemble(f: &Function) -> String {
    let mut out = String::new();
    let mut pc = 0usize;
    while pc < f.code.len() {
        let op = f.code[pc];
        let name = bytecode::name(op).unwrap_or("???");
        let len = bytecode::instr_len(op).unwrap_or(1);
        out.push_str(&format!("{pc:04x}: {name}"));
        for b in f.code.iter().skip(pc + 1).take(len.saturating_sub(1)) {
            out.push_str(&format!(" {b:02x}"));
        }
        out.push('\n');
        pc += len;
    }
    out
}


/// Assemble multiple `.fn name` sections into one module.
pub fn assemble_multi(text: &str) -> Result<Module> {
    let mut chunks: Vec<(String, String)> = Vec::new();
    let mut cur_name = String::from("main");
    let mut cur_body = String::new();
    for line in text.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix(".fn ") {
            if !cur_body.trim().is_empty() {
                chunks.push((cur_name, cur_body));
            }
            cur_name = rest.trim().to_string();
            cur_body = String::new();
        } else {
            cur_body.push_str(line);
            cur_body.push('\n');
        }
    }
    if !cur_body.trim().is_empty() {
        chunks.push((cur_name, cur_body));
    }
    if chunks.is_empty() {
        return assemble(text);
    }
    let mut functions = Vec::new();
    for (name, body) in chunks {
        let mut m = assemble(&body)?;
        if let Some(f) = m.functions.first_mut() {
            f.name = name;
            functions.push(f.clone());
        }
    }
    Ok(Module { functions })
}

pub fn list_mnemonics() -> &'static [&'static str] {
    &[
        "nop", "ret", "throw", "strcmp", "push8", "window", "sjmp", "loadk",
        "loadlocal", "storelocal", "intern", "call", "tail", "close",
        "getupval", "setupval", "openupval", "newarr", "getfield",
    ]
}

pub fn estimate_assembled_size(text: &str) -> usize {
    text.lines()
        .map(|l| l.split('#').next().unwrap_or("").trim())
        .filter(|l| !l.is_empty() && !l.starts_with('.') && !l.ends_with(':'))
        .map(|l| {
            let m = l.split_whitespace().next().unwrap_or("");
            match m {
                "nop" | "ret" | "throw" | "strcmp" => 1usize,
                "push8" | "window" | "sjmp" | "close" | "getupval" | "setupval" | "openupval" => 2,
                "loadk" | "loadlocal" | "storelocal" | "intern" | "call" | "tail" => 3,
                "newarr" | "getfield" => 4,
                _ => 1,
            }
        })
        .sum()
}

pub fn validate_asm_text(text: &str) -> Result<()> {
    let _ = assemble(text)?;
    Ok(())
}

pub fn roundtrip_listing(text: &str) -> Result<String> {
    let m = assemble(text)?;
    let f = m.entry().ok_or(Error::VerifyFailed("no entry"))?;
    Ok(disassemble(f))
}

pub fn openupval_supported() -> bool {
    list_mnemonics().contains(&"openupval")
}
