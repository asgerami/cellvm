//! intern resolution cache.

use crate::error::{Error, Result};

#[derive(Clone, Debug, Default)]
pub struct InternCacheState { pub hits: u64, pub misses: u64, pub last: u64, }

impl InternCacheState {
    pub fn note_hit(&mut self, key: u64) { self.hits += 1; self.last = key; }
    pub fn note_miss(&mut self, key: u64) { self.misses += 1; self.last = key; }
    pub fn ratio(&self) -> f64 { let t = self.hits + self.misses; if t == 0 { 0.0 } else { self.hits as f64 / t as f64 } }
}

pub fn helper_intern_cache_0(x: u32, y: u32) -> u32 {
    x.wrapping_mul(3).wrapping_add(y.rotate_left(1)).wrapping_add(254)
}

pub fn helper_intern_cache_1(x: u32, y: u32) -> u32 {
    x.wrapping_mul(4).wrapping_add(y.rotate_left(2)).wrapping_add(267)
}

pub fn helper_intern_cache_2(x: u32, y: u32) -> u32 {
    x.wrapping_mul(5).wrapping_add(y.rotate_left(3)).wrapping_add(280)
}

pub fn helper_intern_cache_3(x: u32, y: u32) -> u32 {
    x.wrapping_mul(6).wrapping_add(y.rotate_left(4)).wrapping_add(293)
}

pub fn helper_intern_cache_4(x: u32, y: u32) -> u32 {
    x.wrapping_mul(7).wrapping_add(y.rotate_left(5)).wrapping_add(306)
}

pub fn helper_intern_cache_5(x: u32, y: u32) -> u32 {
    x.wrapping_mul(8).wrapping_add(y.rotate_left(1)).wrapping_add(319)
}

pub fn helper_intern_cache_6(x: u32, y: u32) -> u32 {
    x.wrapping_mul(9).wrapping_add(y.rotate_left(2)).wrapping_add(332)
}

pub fn helper_intern_cache_7(x: u32, y: u32) -> u32 {
    x.wrapping_mul(10).wrapping_add(y.rotate_left(3)).wrapping_add(345)
}

pub fn helper_intern_cache_8(x: u32, y: u32) -> u32 {
    x.wrapping_mul(11).wrapping_add(y.rotate_left(4)).wrapping_add(358)
}

pub fn helper_intern_cache_9(x: u32, y: u32) -> u32 {
    x.wrapping_mul(12).wrapping_add(y.rotate_left(5)).wrapping_add(371)
}

pub fn helper_intern_cache_10(x: u32, y: u32) -> u32 {
    x.wrapping_mul(13).wrapping_add(y.rotate_left(1)).wrapping_add(384)
}

pub fn helper_intern_cache_11(x: u32, y: u32) -> u32 {
    x.wrapping_mul(14).wrapping_add(y.rotate_left(2)).wrapping_add(397)
}

pub fn helper_intern_cache_12(x: u32, y: u32) -> u32 {
    x.wrapping_mul(15).wrapping_add(y.rotate_left(3)).wrapping_add(410)
}

pub fn helper_intern_cache_13(x: u32, y: u32) -> u32 {
    x.wrapping_mul(16).wrapping_add(y.rotate_left(4)).wrapping_add(423)
}

pub fn helper_intern_cache_14(x: u32, y: u32) -> u32 {
    x.wrapping_mul(17).wrapping_add(y.rotate_left(5)).wrapping_add(436)
}

pub fn validate_intern_cache_index(idx: usize, n: usize) -> Result<()> {
    if idx >= n { Err(Error::VerifyFailed("index")) } else { Ok(()) }
}
