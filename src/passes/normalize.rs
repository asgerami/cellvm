use crate::module::Module;
use crate::optimize::peephole;

pub fn normalized_code(code: &[u8]) -> Vec<u8> {
    peephole::strip_nops(code).0
}

pub fn normalized_hash(m: &Module) -> u64 {
    let mut h = 0x9e3779b97f4a7c15u64;
    for f in &m.functions {
        for b in normalized_code(&f.code) {
            h ^= b as u64;
            h = h.wrapping_mul(0xbf58476d1ce4e5b9);
        }
        h ^= f.frame_size as u64;
        h = h.rotate_left(7);
    }
    h
}

pub fn fingerprint(m: &Module) -> String {
    format!("{:016x}", normalized_hash(m))
}
