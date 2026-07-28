//! Textual disassembler for CLVM function bodies.

use crate::bytecode;
use crate::isa;
use crate::module::Function;

#[derive(Clone, Debug)]
pub struct Line {
    pub pc: usize,
    pub bytes: Vec<u8>,
    pub text: String,
}

pub fn disassemble_function(f: &Function) -> Vec<Line> {
    let mut out = Vec::new();
    let code = &f.code;
    let mut pc = 0usize;
    while pc < code.len() {
        let op = code[pc];
        let len = bytecode::instr_len(op).unwrap_or(1).min(code.len() - pc);
        let bytes = code[pc..pc + len].to_vec();
        let text = format_instr(code, pc, op, len);
        out.push(Line { pc, bytes, text });
        pc += len;
    }
    out
}

fn format_instr(code: &[u8], pc: usize, op: u8, len: usize) -> String {
    let name = isa::name_of(op).unwrap_or("???");
    match op {
        bytecode::PUSH8 if len >= 2 => format!("{name} {}", code[pc + 1] as i8),
        bytecode::WINDOW if len >= 2 => format!("{name} {:+}", code[pc + 1] as i8),
        bytecode::SJMP if len >= 2 => {
            let rel = code[pc + 1] as i8 as i64;
            format!("{name} {rel:+} ; -> {}", pc as i64 + 2 + rel)
        }
        bytecode::LOADK | bytecode::LOADLOCAL | bytecode::STORELOCAL | bytecode::INTERN
            | bytecode::CALL | bytecode::TAIL | bytecode::GETUPVAL | bytecode::SETUPVAL
            | bytecode::OPENUPVAL
            if len >= 2 =>
        {
            let idx = code[pc + 1] as u16 | ((code.get(pc + 2).copied().unwrap_or(0) as u16) << 8);
            format!("{name} {idx}")
        }
        bytecode::NEWARR | bytecode::GETFIELD if len >= 4 => {
            let a = code[pc + 1] as u32
                | ((code[pc + 2] as u32) << 8)
                | ((code[pc + 3] as u32) << 16);
            format!("{name} 0x{a:06X}")
        }
        _ => name.to_string(),
    }
}

pub fn render(f: &Function) -> String {
    let mut s = format!("; function {} locals={} frame={}\n", f.name, f.max_locals, f.frame_size);
    for line in disassemble_function(f) {
        let hex: String = line.bytes.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" ");
        s.push_str(&format!("  {:>4X}: {:<12}  {}\n", line.pc, hex, line.text));
    }
    s
}

pub fn render_module(funcs: &[Function]) -> String {
    funcs.iter().map(render).collect::<Vec<_>>().join("\n")
}

pub fn find_ops(f: &Function, opcode: u8) -> Vec<usize> {
    disassemble_function(f)
        .into_iter()
        .filter(|l| l.bytes.first() == Some(&opcode))
        .map(|l| l.pc)
        .collect()
}

pub fn approx_size(f: &Function) -> usize {
    f.code.len() + f.consts.len() * 8 + f.string_pool.iter().map(|s| s.len()).sum::<usize>()
}
