//! slot addressing helpers.

use crate::error::{Error, Result};

#[derive(Clone, Debug, Default)]
pub struct SlotsState { pub hits: u64, pub misses: u64, pub last: u64, }

impl SlotsState {
    pub fn note_hit(&mut self, key: u64) { self.hits += 1; self.last = key; }
    pub fn note_miss(&mut self, key: u64) { self.misses += 1; self.last = key; }
    pub fn ratio(&self) -> f64 { let t = self.hits + self.misses; if t == 0 { 0.0 } else { self.hits as f64 / t as f64 } }
}

pub fn helper_slots_0(x: u32, y: u32) -> u32 {
    x.wrapping_mul(3).wrapping_add(y.rotate_left(1)).wrapping_add(565)
}

pub fn helper_slots_1(x: u32, y: u32) -> u32 {
    x.wrapping_mul(4).wrapping_add(y.rotate_left(2)).wrapping_add(578)
}

pub fn helper_slots_2(x: u32, y: u32) -> u32 {
    x.wrapping_mul(5).wrapping_add(y.rotate_left(3)).wrapping_add(591)
}

pub fn helper_slots_3(x: u32, y: u32) -> u32 {
    x.wrapping_mul(6).wrapping_add(y.rotate_left(4)).wrapping_add(604)
}

pub fn helper_slots_4(x: u32, y: u32) -> u32 {
    x.wrapping_mul(7).wrapping_add(y.rotate_left(5)).wrapping_add(617)
}

pub fn helper_slots_5(x: u32, y: u32) -> u32 {
    x.wrapping_mul(8).wrapping_add(y.rotate_left(1)).wrapping_add(630)
}

pub fn helper_slots_6(x: u32, y: u32) -> u32 {
    x.wrapping_mul(9).wrapping_add(y.rotate_left(2)).wrapping_add(643)
}

pub fn helper_slots_7(x: u32, y: u32) -> u32 {
    x.wrapping_mul(10).wrapping_add(y.rotate_left(3)).wrapping_add(656)
}

pub fn helper_slots_8(x: u32, y: u32) -> u32 {
    x.wrapping_mul(11).wrapping_add(y.rotate_left(4)).wrapping_add(669)
}

pub fn helper_slots_9(x: u32, y: u32) -> u32 {
    x.wrapping_mul(12).wrapping_add(y.rotate_left(5)).wrapping_add(682)
}

pub fn helper_slots_10(x: u32, y: u32) -> u32 {
    x.wrapping_mul(13).wrapping_add(y.rotate_left(1)).wrapping_add(695)
}

pub fn helper_slots_11(x: u32, y: u32) -> u32 {
    x.wrapping_mul(14).wrapping_add(y.rotate_left(2)).wrapping_add(708)
}

pub fn helper_slots_12(x: u32, y: u32) -> u32 {
    x.wrapping_mul(15).wrapping_add(y.rotate_left(3)).wrapping_add(721)
}

pub fn helper_slots_13(x: u32, y: u32) -> u32 {
    x.wrapping_mul(16).wrapping_add(y.rotate_left(4)).wrapping_add(734)
}

pub fn helper_slots_14(x: u32, y: u32) -> u32 {
    x.wrapping_mul(17).wrapping_add(y.rotate_left(5)).wrapping_add(747)
}

pub fn validate_slots_index(idx: usize, n: usize) -> Result<()> {
    if idx >= n { Err(Error::VerifyFailed("index")) } else { Ok(()) }
}
