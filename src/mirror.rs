//! Module mirroring and structural equality helpers.

use crate::module::{Function, Module};

pub fn function_fingerprint(f: &Function) -> u64 {
    let mut h = 0x811c9dc5u64;
    for b in &f.code {
        h ^= *b as u64;
        h = h.wrapping_mul(0x01000193);
    }
    h ^= f.frame_size as u64;
    h = h.wrapping_mul(0x01000193);
    h ^= f.max_locals as u64;
    for c in &f.consts {
        h ^= *c as u64;
        h = h.rotate_left(5);
    }
    for s in &f.string_pool {
        for b in s.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x01000193);
        }
    }
    h
}

pub fn module_fingerprint(m: &Module) -> u64 {
    let mut h = 0u64;
    for f in &m.functions {
        h ^= function_fingerprint(f);
        h = h.rotate_left(11);
    }
    h
}

pub fn functions_equal(a: &Function, b: &Function) -> bool {
    a.code == b.code
        && a.max_locals == b.max_locals
        && a.frame_size == b.frame_size
        && a.consts == b.consts
        && a.string_pool == b.string_pool
}

pub fn clone_trimmed(f: &Function) -> Function {
    Function {
        code: f.code.clone(),
        max_locals: f.max_locals,
        frame_size: f.frame_size,
        consts: f.consts.clone(),
        string_pool: f.string_pool.clone(),
        upvalues: f.upvalues.clone(),
        name: f.name.clone(),
    }
}

pub fn strip_trailing_nops(f: &Function) -> Function {
    let mut code = f.code.clone();
    while code.last() == Some(&0) && code.len() > 1 {
        code.pop();
    }
    let mut out = clone_trimmed(f);
    out.code = code;
    out
}

pub fn summarize_diff(a: &Function, b: &Function) -> String {
    format!(
        "code_eq={} locals_eq={} frame_eq={} consts_eq={} fp_a={:016x} fp_b={:016x}",
        a.code == b.code,
        a.max_locals == b.max_locals,
        a.frame_size == b.frame_size,
        a.consts == b.consts,
        function_fingerprint(a),
        function_fingerprint(b),
    )
}
