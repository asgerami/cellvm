//! Canonical ISA metadata used by disasm, cfg, and validation.

use crate::bytecode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpInfo {
    pub name: &'static str,
    pub opcode: u8,
    pub len: usize,
    pub stack_delta: i8,
    pub is_branch: bool,
    pub doc: &'static str,
}

pub const OPS: &[OpInfo] = &[
    OpInfo { name: "nop", opcode: 0x00, len: 1, stack_delta: 0, is_branch: false, doc: "no operation" },
    OpInfo { name: "push8", opcode: 0x01, len: 2, stack_delta: 1, is_branch: false, doc: "push immediate i8" },
    OpInfo { name: "window", opcode: 0x02, len: 2, stack_delta: 0, is_branch: false, doc: "adjust local window base" },
    OpInfo { name: "sjmp", opcode: 0x20, len: 2, stack_delta: 0, is_branch: true, doc: "short relative jump" },
    OpInfo { name: "loadk", opcode: 0x30, len: 3, stack_delta: 1, is_branch: false, doc: "load constant by index" },
    OpInfo { name: "loadlocal", opcode: 0x31, len: 3, stack_delta: 1, is_branch: false, doc: "load local at window+slot" },
    OpInfo { name: "storelocal", opcode: 0x32, len: 3, stack_delta: -1, is_branch: false, doc: "store local at window+slot" },
    OpInfo { name: "intern", opcode: 0x33, len: 3, stack_delta: 1, is_branch: false, doc: "intern string pool entry" },
    OpInfo { name: "strcmp", opcode: 0x34, len: 1, stack_delta: 0, is_branch: false, doc: "compare against cached intern" },
    OpInfo { name: "call", opcode: 0x35, len: 3, stack_delta: 0, is_branch: false, doc: "call function by index" },
    OpInfo { name: "tail", opcode: 0x36, len: 3, stack_delta: 0, is_branch: true, doc: "tail-call function" },
    OpInfo { name: "close", opcode: 0x37, len: 2, stack_delta: 0, is_branch: false, doc: "close open upvalues" },
    OpInfo { name: "getupval", opcode: 0x38, len: 2, stack_delta: 1, is_branch: false, doc: "read upvalue" },
    OpInfo { name: "setupval", opcode: 0x39, len: 2, stack_delta: -1, is_branch: false, doc: "write upvalue" },
    OpInfo { name: "throw", opcode: 0x3A, len: 1, stack_delta: 0, is_branch: true, doc: "throw / unwind" },
    OpInfo { name: "openupval", opcode: 0x3B, len: 2, stack_delta: 0, is_branch: false, doc: "capture local as open upvalue" },
    OpInfo { name: "newarr", opcode: 0x40, len: 4, stack_delta: 1, is_branch: false, doc: "allocate array-like object" },
    OpInfo { name: "getfield", opcode: 0x41, len: 4, stack_delta: 0, is_branch: false, doc: "field load from reference" },
    OpInfo { name: "ret", opcode: 0xFF, len: 1, stack_delta: 0, is_branch: true, doc: "return from frame" },
];

pub fn lookup(op: u8) -> Option<&'static OpInfo> {
    OPS.iter().find(|i| i.opcode == op)
}

pub fn name_of(op: u8) -> Option<&'static str> {
    lookup(op).map(|i| i.name)
}

pub fn validate_against_bytecode_table() -> bool {
    for info in OPS {
        match bytecode::instr_len(info.opcode) {
            Some(l) if l == info.len => {}
            _ => return false,
        }
    }
    true
}

pub fn stack_effect_prefix(code: &[u8]) -> Option<i32> {
    let mut depth = 0i32;
    let mut pc = 0usize;
    while pc < code.len() {
        let op = *code.get(pc)?;
        let info = lookup(op)?;
        depth += info.stack_delta as i32;
        if depth < 0 {
            return None;
        }
        pc = pc.checked_add(info.len)?;
    }
    Some(depth)
}

pub fn describe_all() -> String {
    let mut out = String::new();
    for info in OPS {
        out.push_str(&format!(
            "0x{:02X} {:<12} len={} delta={:+} branch={} — {}\n",
            info.opcode, info.name, info.len, info.stack_delta, info.is_branch, info.doc
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn table_matches_bytecode() {
        assert!(validate_against_bytecode_table());
    }
}
