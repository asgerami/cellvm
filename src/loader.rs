//! P0 Load: bytes -> Module. Panic-free (all Result).

use std::collections::HashSet;

use crate::bytecode::{self, SJMP};
use crate::error::{Error, Result};
use crate::module::{Function, Module};

const MAGIC: &[u8; 4] = b"CLVM";

pub fn load_module(input: &[u8]) -> Result<Module> {
    if input.len() < 8 {
        return Err(Error::Truncated { at: "header" });
    }
    if input.get(0..4) != Some(MAGIC) {
        return Err(Error::BadMagic);
    }
    let code_len = u32::from_le_bytes(input[4..8].try_into().unwrap()) as usize;
    if input.len() < 8 + code_len {
        return Err(Error::Truncated { at: "code" });
    }
    let mut code = input[8..8 + code_len].to_vec();
    let nconst = u32::from_le_bytes(
        input
            .get(8 + code_len..8 + code_len + 4)
            .ok_or(Error::Truncated { at: "nconst" })?
            .try_into()
            .unwrap(),
    ) as usize;
    let mut off = 8 + code_len + 4;
    let mut consts = Vec::with_capacity(nconst);
    for _ in 0..nconst {
        if off + 8 > input.len() {
            return Err(Error::Truncated { at: "const" });
        }
        consts.push(i64::from_le_bytes(input[off..off + 8].try_into().unwrap()));
        off += 8;
    }

    let mut string_pool = Vec::new();
    let mut functions = Vec::new();

    // Optional trailer records, each tagged:
    //   0xFFFFFFFF → string pool
    //   0xFFFFFFFE → extra functions
    //   otherwise  → jump lowering (at, target)
    while off + 4 <= input.len() {
        let tag = u32::from_le_bytes(input[off..off + 4].try_into().unwrap());
        off += 4;
        if tag == 0xFFFF_FFFF {
            if off + 4 > input.len() {
                break;
            }
            let nstrings = u32::from_le_bytes(input[off..off + 4].try_into().unwrap()) as usize;
            off += 4;
            if nstrings > 4096 {
                break;
            }
            for _ in 0..nstrings {
                if off + 4 > input.len() {
                    return Err(Error::Truncated { at: "string len" });
                }
                let len = u32::from_le_bytes(input[off..off + 4].try_into().unwrap()) as usize;
                off += 4;
                if off + len > input.len() {
                    return Err(Error::Truncated { at: "string body" });
                }
                let s = String::from_utf8_lossy(&input[off..off + len]).into_owned();
                off += len;
                string_pool.push(s);
            }
        } else if tag == 0xFFFF_FFFE {
            if off + 4 > input.len() {
                break;
            }
            let nfuncs = u32::from_le_bytes(input[off..off + 4].try_into().unwrap()) as usize;
            off += 4;
            for i in 0..nfuncs.min(64) {
                if off + 4 > input.len() {
                    break;
                }
                let clen = u32::from_le_bytes(input[off..off + 4].try_into().unwrap()) as usize;
                off += 4;
                if off + clen > input.len() {
                    break;
                }
                let fcode = input[off..off + clen].to_vec();
                off += clen;
                functions.push(Function {
                    code: fcode,
                    max_locals: 8,
                    frame_size: 64,
                    consts: Vec::new(),
                    string_pool: Vec::new(),
                    upvalues: Vec::new(),
                    name: format!("f{i}"),
                });
            }
        } else {
            if off + 4 > input.len() {
                break;
            }
            let jmp_at = tag as usize;
            let target = u32::from_le_bytes(input[off..off + 4].try_into().unwrap());
            off += 4;
            if jmp_at < code.len() {
                let boundaries = build_boundaries(&code)?;
                lower_jump(&mut code, jmp_at, target, &boundaries)?;
            }
        }
    }

    let main = Function {
        code,
        max_locals: 8,
        frame_size: 16,
        consts,
        string_pool,
        upvalues: Vec::new(),
        name: "main".into(),
    };
    let mut all = vec![main];
    all.append(&mut functions);
    Ok(Module { functions: all })
}

pub fn build_boundaries(code: &[u8]) -> Result<HashSet<usize>> {
    let mut set = HashSet::new();
    let mut pc = 0usize;
    while pc < code.len() {
        set.insert(pc);
        let op = *code.get(pc).ok_or(Error::TruncatedInstr { pc })?;
        let len = bytecode::instr_len(op).ok_or(Error::UnknownOpcode { pc, op })?;
        let end = pc.checked_add(len).ok_or(Error::TruncatedInstr { pc })?;
        if end > code.len() {
            return Err(Error::TruncatedInstr { pc });
        }
        pc = end;
    }
    Ok(set)
}

/// Lower an absolute jump target into a short relative SJMP at `at`.
///
/// Validates that `target` is in-range and on an instruction boundary, then
/// stores `rel as i8`. Callers that need a lossless encoding must ensure the
/// relative displacement fits in a signed byte (checked by tooling / stricter
/// loaders).
pub fn lower_jump(
    code: &mut [u8],
    at: usize,
    target: u32,
    boundaries: &HashSet<usize>,
) -> Result<()> {
    let code_len = code.len() as u32;
    if target >= code_len {
        return Err(Error::TargetOutOfRange { target });
    }
    if !boundaries.contains(&(target as usize)) {
        return Err(Error::TargetNotBoundary { target });
    }
    let next_pc = at as i64 + 2;
    let rel = target as i64 - next_pc;
    let stored = rel as i8;
    *code.get_mut(at).ok_or(Error::TruncatedInstr { pc: at })? = SJMP;
    *code.get_mut(at + 1).ok_or(Error::TruncatedInstr { pc: at })? = stored as u8;
    Ok(())
}

/// Compute the runtime PC a lowered SJMP would land on.
pub fn sjmp_runtime_target(at: usize, stored: i8) -> usize {
    (at as i64 + 2 + stored as i64) as usize
}

/// Encode a short jump when the displacement already fits in i8.
pub fn try_lower_short_jump(
    code: &mut [u8],
    at: usize,
    target: u32,
    boundaries: &HashSet<usize>,
) -> Result<()> {
    let next_pc = at as i64 + 2;
    let rel = target as i64 - next_pc;
    if rel != rel as i8 as i64 {
        return Err(Error::OffsetTruncation { rel });
    }
    lower_jump(code, at, target, boundaries)
}

pub fn module_code_bytes(m: &Module) -> usize {
    m.functions.iter().map(|f| f.code.len()).sum()
}

pub fn module_const_count(m: &Module) -> usize {
    m.functions.iter().map(|f| f.consts.len()).sum()
}

pub fn sanitize_function_names(m: &mut Module) {
    for (i, f) in m.functions.iter_mut().enumerate() {
        if f.name.is_empty() {
            f.name = format!("fn{i}");
        }
    }
}

pub fn peek_magic(input: &[u8]) -> bool {
    input.get(0..4) == Some(MAGIC)
}

pub fn code_section_len(input: &[u8]) -> Result<usize> {
    if input.len() < 8 || !peek_magic(input) {
        return Err(Error::BadMagic);
    }
    Ok(u32::from_le_bytes(input[4..8].try_into().unwrap()) as usize)
}

pub fn count_opcodes(code: &[u8]) -> Result<usize> {
    let mut n = 0usize;
    let mut pc = 0usize;
    while pc < code.len() {
        let op = *code.get(pc).ok_or(Error::TruncatedInstr { pc })?;
        let len = bytecode::instr_len(op).ok_or(Error::UnknownOpcode { pc, op })?;
        n += 1;
        pc = pc.checked_add(len).ok_or(Error::TruncatedInstr { pc })?;
    }
    Ok(n)
}

pub fn find_opcode_pcs(code: &[u8], want: u8) -> Result<Vec<usize>> {
    let mut out = Vec::new();
    let mut pc = 0usize;
    while pc < code.len() {
        let op = *code.get(pc).ok_or(Error::TruncatedInstr { pc })?;
        if op == want {
            out.push(pc);
        }
        let len = bytecode::instr_len(op).ok_or(Error::UnknownOpcode { pc, op })?;
        pc = pc.checked_add(len).ok_or(Error::TruncatedInstr { pc })?;
    }
    Ok(out)
}
