//! Lowered IR view of bytecode for analyses.

use crate::bytecode;
use crate::isa;
use crate::module::Function;

#[derive(Clone, Debug)]
pub enum IrOp {
    Nop,
    Push(i8),
    Window(i8),
    Jump { rel: i8, from: usize },
    LoadK(u16),
    LoadLocal(u16),
    StoreLocal(u16),
    Intern(u16),
    StrCmp,
    Call(u16),
    Tail(u16),
    Close,
    GetUp(u16),
    SetUp(u16),
    Throw,
    OpenUp(u16),
    NewArr(u32),
    GetField(u32),
    Ret,
    Raw(u8),
}

#[derive(Clone, Debug)]
pub struct IrInst {
    pub pc: usize,
    pub op: IrOp,
}

pub fn lower(f: &Function) -> Vec<IrInst> {
    let mut out = Vec::new();
    let mut pc = 0usize;
    let code = &f.code;
    while pc < code.len() {
        let opc = code[pc];
        let len = bytecode::instr_len(opc).unwrap_or(1);
        let op = match opc {
            bytecode::NOP => IrOp::Nop,
            bytecode::PUSH8 if pc + 1 < code.len() => IrOp::Push(code[pc + 1] as i8),
            bytecode::WINDOW if pc + 1 < code.len() => IrOp::Window(code[pc + 1] as i8),
            bytecode::SJMP if pc + 1 < code.len() => IrOp::Jump { rel: code[pc + 1] as i8, from: pc },
            bytecode::LOADK => IrOp::LoadK(read_u16(code, pc)),
            bytecode::LOADLOCAL => IrOp::LoadLocal(read_u16(code, pc)),
            bytecode::STORELOCAL => IrOp::StoreLocal(read_u16(code, pc)),
            bytecode::INTERN => IrOp::Intern(read_u16(code, pc)),
            bytecode::STRCMP => IrOp::StrCmp,
            bytecode::CALL => IrOp::Call(read_u16(code, pc)),
            bytecode::TAIL => IrOp::Tail(read_u16(code, pc)),
            bytecode::CLOSE => IrOp::Close,
            bytecode::GETUPVAL => IrOp::GetUp(read_u16(code, pc)),
            bytecode::SETUPVAL => IrOp::SetUp(read_u16(code, pc)),
            bytecode::THROW => IrOp::Throw,
            bytecode::OPENUPVAL => IrOp::OpenUp(code.get(pc + 1).copied().unwrap_or(0) as u16),
            bytecode::NEWARR => IrOp::NewArr(read_u24(code, pc)),
            bytecode::GETFIELD => IrOp::GetField(read_u24(code, pc)),
            bytecode::RET => IrOp::Ret,
            _ => IrOp::Raw(opc),
        };
        out.push(IrInst { pc, op });
        pc += len;
        let _ = isa::lookup(opc);
    }
    out
}

fn read_u16(code: &[u8], pc: usize) -> u16 {
    let lo = code.get(pc + 1).copied().unwrap_or(0) as u16;
    let hi = code.get(pc + 2).copied().unwrap_or(0) as u16;
    lo | (hi << 8)
}

fn read_u24(code: &[u8], pc: usize) -> u32 {
    let a = code.get(pc + 1).copied().unwrap_or(0) as u32;
    let b = code.get(pc + 2).copied().unwrap_or(0) as u32;
    let c = code.get(pc + 3).copied().unwrap_or(0) as u32;
    a | (b << 8) | (c << 16)
}

pub fn count_calls(ir: &[IrInst]) -> usize {
    ir.iter().filter(|i| matches!(i.op, IrOp::Call(_) | IrOp::Tail(_))).count()
}

pub fn count_throws(ir: &[IrInst]) -> usize {
    ir.iter().filter(|i| matches!(i.op, IrOp::Throw)).count()
}

pub fn windows(ir: &[IrInst]) -> Vec<(usize, i8)> {
    ir.iter()
        .filter_map(|i| match i.op {
            IrOp::Window(d) => Some((i.pc, d)),
            _ => None,
        })
        .collect()
}
