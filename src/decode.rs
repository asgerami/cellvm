//! Instruction decoder and encoder for CLVM bytecode streams.

use crate::bytecode;
use crate::error::{Error, Result};
use crate::isa;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decoded {
    Nop,
    Push8(i8),
    Window(i8),
    Sjmp(i8),
    LoadK(u16),
    LoadLocal(u16),
    StoreLocal(u16),
    Intern(u16),
    StrCmp,
    Call(u16),
    Tail(u16),
    Close(u8),
    GetUp(u8),
    SetUp(u8),
    Throw,
    OpenUp(u8),
    NewArr(u32),
    GetField(u32),
    Ret,
    Unknown(u8),
}

pub fn decode_at(code: &[u8], pc: usize) -> Result<(Decoded, usize)> {
    let op = *code.get(pc).ok_or(Error::TruncatedInstr { pc })?;
    let len = bytecode::instr_len(op).ok_or(Error::UnknownOpcode { pc, op })?;
    if pc + len > code.len() {
        return Err(Error::TruncatedInstr { pc });
    }
    let d = match op {
        bytecode::NOP => Decoded::Nop,
        bytecode::PUSH8 => Decoded::Push8(code[pc + 1] as i8),
        bytecode::WINDOW => Decoded::Window(code[pc + 1] as i8),
        bytecode::SJMP => Decoded::Sjmp(code[pc + 1] as i8),
        bytecode::LOADK => Decoded::LoadK(read_u16(code, pc)),
        bytecode::LOADLOCAL => Decoded::LoadLocal(read_u16(code, pc)),
        bytecode::STORELOCAL => Decoded::StoreLocal(read_u16(code, pc)),
        bytecode::INTERN => Decoded::Intern(read_u16(code, pc)),
        bytecode::STRCMP => Decoded::StrCmp,
        bytecode::CALL => Decoded::Call(read_u16(code, pc)),
        bytecode::TAIL => Decoded::Tail(read_u16(code, pc)),
        bytecode::CLOSE => Decoded::Close(code[pc + 1]),
        bytecode::GETUPVAL => Decoded::GetUp(code[pc + 1]),
        bytecode::SETUPVAL => Decoded::SetUp(code[pc + 1]),
        bytecode::THROW => Decoded::Throw,
        bytecode::OPENUPVAL => Decoded::OpenUp(code[pc + 1]),
        bytecode::NEWARR => Decoded::NewArr(read_u24(code, pc)),
        bytecode::GETFIELD => Decoded::GetField(read_u24(code, pc)),
        bytecode::RET => Decoded::Ret,
        other => Decoded::Unknown(other),
    };
    Ok((d, len))
}

fn read_u16(code: &[u8], pc: usize) -> u16 {
    u16::from_le_bytes([code[pc + 1], code.get(pc + 2).copied().unwrap_or(0)])
}

fn read_u24(code: &[u8], pc: usize) -> u32 {
    code[pc + 1] as u32
        | ((code.get(pc + 2).copied().unwrap_or(0) as u32) << 8)
        | ((code.get(pc + 3).copied().unwrap_or(0) as u32) << 16)
}

pub fn decode_all(code: &[u8]) -> Result<Vec<(usize, Decoded)>> {
    let mut out = Vec::new();
    let mut pc = 0usize;
    while pc < code.len() {
        let (d, len) = decode_at(code, pc)?;
        out.push((pc, d));
        pc += len;
    }
    Ok(out)
}

pub fn encode(d: &Decoded) -> Vec<u8> {
    match *d {
        Decoded::Nop => vec![bytecode::NOP],
        Decoded::Push8(v) => vec![bytecode::PUSH8, v as u8],
        Decoded::Window(v) => vec![bytecode::WINDOW, v as u8],
        Decoded::Sjmp(v) => vec![bytecode::SJMP, v as u8],
        Decoded::LoadK(v) => encode_u16(bytecode::LOADK, v),
        Decoded::LoadLocal(v) => encode_u16(bytecode::LOADLOCAL, v),
        Decoded::StoreLocal(v) => encode_u16(bytecode::STORELOCAL, v),
        Decoded::Intern(v) => encode_u16(bytecode::INTERN, v),
        Decoded::StrCmp => vec![bytecode::STRCMP],
        Decoded::Call(v) => encode_u16(bytecode::CALL, v),
        Decoded::Tail(v) => encode_u16(bytecode::TAIL, v),
        Decoded::Close(v) => vec![bytecode::CLOSE, v],
        Decoded::GetUp(v) => vec![bytecode::GETUPVAL, v],
        Decoded::SetUp(v) => vec![bytecode::SETUPVAL, v],
        Decoded::Throw => vec![bytecode::THROW],
        Decoded::OpenUp(v) => vec![bytecode::OPENUPVAL, v],
        Decoded::NewArr(v) => encode_u24(bytecode::NEWARR, v),
        Decoded::GetField(v) => encode_u24(bytecode::GETFIELD, v),
        Decoded::Ret => vec![bytecode::RET],
        Decoded::Unknown(op) => vec![op],
    }
}

fn encode_u16(op: u8, v: u16) -> Vec<u8> {
    let b = v.to_le_bytes();
    vec![op, b[0], b[1]]
}

fn encode_u24(op: u8, v: u32) -> Vec<u8> {
    vec![op, (v & 0xff) as u8, ((v >> 8) & 0xff) as u8, ((v >> 16) & 0xff) as u8]
}

pub fn encode_all(ops: &[Decoded]) -> Vec<u8> {
    let mut out = Vec::new();
    for d in ops {
        out.extend(encode(d));
    }
    out
}

pub fn roundtrip_ok(code: &[u8]) -> bool {
    match decode_all(code) {
        Ok(decoded) => {
            let ops: Vec<_> = decoded.into_iter().map(|(_, d)| d).collect();
            encode_all(&ops) == code
        }
        Err(_) => false,
    }
}

pub fn describe(d: &Decoded) -> String {
    match d {
        Decoded::Nop => "nop".into(),
        Decoded::Push8(v) => format!("push8 {v}"),
        Decoded::Window(v) => format!("window {v:+}"),
        Decoded::Sjmp(v) => format!("sjmp {v:+}"),
        Decoded::LoadK(v) => format!("loadk {v}"),
        Decoded::LoadLocal(v) => format!("loadlocal {v}"),
        Decoded::StoreLocal(v) => format!("storelocal {v}"),
        Decoded::Intern(v) => format!("intern {v}"),
        Decoded::StrCmp => "strcmp".into(),
        Decoded::Call(v) => format!("call {v}"),
        Decoded::Tail(v) => format!("tail {v}"),
        Decoded::Close(v) => format!("close {v}"),
        Decoded::GetUp(v) => format!("getupval {v}"),
        Decoded::SetUp(v) => format!("setupval {v}"),
        Decoded::Throw => "throw".into(),
        Decoded::OpenUp(v) => format!("openupval {v}"),
        Decoded::NewArr(v) => format!("newarr 0x{v:06X}"),
        Decoded::GetField(v) => format!("getfield 0x{v:06X}"),
        Decoded::Ret => "ret".into(),
        Decoded::Unknown(op) => format!("unknown 0x{op:02X}"),
    }
}

pub fn stack_delta_of(d: &Decoded) -> i8 {
    match d {
        Decoded::Push8(_) | Decoded::LoadK(_) | Decoded::LoadLocal(_) | Decoded::Intern(_) | Decoded::GetUp(_) | Decoded::NewArr(_) => 1,
        Decoded::StoreLocal(_) | Decoded::SetUp(_) | Decoded::StrCmp => -1,
        Decoded::GetField(_) => 0,
        Decoded::Unknown(op) => isa::lookup(*op).map(|i| i.stack_delta).unwrap_or(0),
        _ => 0,
    }
}

pub fn count_by_opcode(code: &[u8]) -> Result<[usize; 256]> {
    let mut counts = [0usize; 256];
    let mut pc = 0usize;
    while pc < code.len() {
        let op = *code.get(pc).ok_or(Error::TruncatedInstr { pc })?;
        let len = bytecode::instr_len(op).ok_or(Error::UnknownOpcode { pc, op })?;
        counts[op as usize] += 1;
        pc += len;
    }
    Ok(counts)
}
