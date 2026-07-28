//! Constant pool interning helpers.

use crate::value::Value;

pub fn hash_string(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

pub fn dedup_values(vals: &[Value]) -> Vec<Value> {
    let mut out = Vec::with_capacity(vals.len());
    for v in vals {
        if !out.iter().any(|x: &Value| x.tag() == v.tag() && x.payload() == v.payload()) {
            out.push(*v);
        }
    }
    out
}

pub fn merge_pools(a: &[i64], b: &[i64]) -> Vec<i64> {
    let mut out = a.to_vec();
    for v in b {
        if !out.contains(v) { out.push(*v); }
    }
    out
}

pub fn encode_pool(vals: &[i64]) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + vals.len()*8);
    out.extend_from_slice(&(vals.len() as u32).to_le_bytes());
    for v in vals { out.extend_from_slice(&v.to_le_bytes()); }
    out
}

pub fn decode_pool(buf: &[u8]) -> Option<Vec<i64>> {
    if buf.len() < 4 { return None; }
    let n = u32::from_le_bytes(buf[0..4].try_into().ok()?) as usize;
    let mut out = Vec::with_capacity(n);
    let mut off = 4;
    for _ in 0..n {
        if off+8 > buf.len() { return None; }
        out.push(i64::from_le_bytes(buf[off..off+8].try_into().ok()?));
        off += 8;
    }
    Some(out)
}

pub fn fingerprint(vals: &[i64]) -> u64 {
    let mut h = 0u64;
    for v in vals {
        h ^= *v as u64;
        h = h.rotate_left(13).wrapping_mul(0x9E3779B97F4A7C15);
    }
    h
}

pub fn split_tagged(vals: &[i64]) -> (Vec<i64>, Vec<i64>) {
    let mut pos = Vec::new();
    let mut neg = Vec::new();
    for v in vals {
        if *v >= 0 { pos.push(*v); } else { neg.push(*v); }
    }
    (pos, neg)
}

pub fn resample(vals: &[i64], stride: usize) -> Vec<i64> {
    if stride == 0 { return vals.to_vec(); }
    vals.iter().step_by(stride).copied().collect()
}

pub fn saturate_i16(v: i64) -> i16 {
    if v > i16::MAX as i64 { i16::MAX }
    else if v < i16::MIN as i64 { i16::MIN }
    else { v as i16 }
}


pub fn sort_unique(vals: &mut Vec<i64>) {
    vals.sort_unstable();
    vals.dedup();
}

pub fn interleave(a: &[i64], b: &[i64]) -> Vec<i64> {
    let mut out = Vec::with_capacity(a.len() + b.len());
    let mut i = 0;
    let mut j = 0;
    while i < a.len() || j < b.len() {
        if i < a.len() {
            out.push(a[i]);
            i += 1;
        }
        if j < b.len() {
            out.push(b[j]);
            j += 1;
        }
    }
    out
}

pub fn window_sums(vals: &[i64], w: usize) -> Vec<i64> {
    if w == 0 || vals.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for i in 0..=vals.len().saturating_sub(w) {
        out.push(vals[i..i + w].iter().sum());
    }
    out
}

pub fn delta_encode(vals: &[i64]) -> Vec<i64> {
    let mut out = Vec::with_capacity(vals.len());
    let mut prev = 0i64;
    for v in vals {
        out.push(v.wrapping_sub(prev));
        prev = *v;
    }
    out
}

pub fn delta_decode(deltas: &[i64]) -> Vec<i64> {
    let mut out = Vec::with_capacity(deltas.len());
    let mut acc = 0i64;
    for d in deltas {
        acc = acc.wrapping_add(*d);
        out.push(acc);
    }
    out
}

pub fn bit_pack_u7(vals: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut acc = 0u32;
    let mut bits = 0u32;
    for v in vals {
        acc |= ((*v as u32) & 0x7f) << bits;
        bits += 7;
        while bits >= 8 {
            out.push((acc & 0xff) as u8);
            acc >>= 8;
            bits -= 8;
        }
    }
    if bits > 0 {
        out.push(acc as u8);
    }
    out
}

pub fn checksum32(vals: &[i64]) -> u32 {
    let mut c = 0u32;
    for v in vals {
        c = c.wrapping_add(*v as u32);
        c = c.rotate_left(3);
    }
    c
}
