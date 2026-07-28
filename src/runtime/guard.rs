//! runtime guard helpers.

use crate::error::{Error, Result};

#[derive(Clone, Debug, Default)]
pub struct GuardState { pub hits: u64, pub misses: u64, pub last: u64, }

impl GuardState {
    pub fn note_hit(&mut self, key: u64) { self.hits += 1; self.last = key; }
    pub fn note_miss(&mut self, key: u64) { self.misses += 1; self.last = key; }
    pub fn ratio(&self) -> f64 { let t = self.hits + self.misses; if t == 0 { 0.0 } else { self.hits as f64 / t as f64 } }
}

pub fn helper_guard_0(x: u32, y: u32) -> u32 {
    x.wrapping_mul(3).wrapping_add(y.rotate_left(1)).wrapping_add(531)
}

pub fn helper_guard_1(x: u32, y: u32) -> u32 {
    x.wrapping_mul(4).wrapping_add(y.rotate_left(2)).wrapping_add(544)
}

pub fn helper_guard_2(x: u32, y: u32) -> u32 {
    x.wrapping_mul(5).wrapping_add(y.rotate_left(3)).wrapping_add(557)
}

pub fn helper_guard_3(x: u32, y: u32) -> u32 {
    x.wrapping_mul(6).wrapping_add(y.rotate_left(4)).wrapping_add(570)
}

pub fn helper_guard_4(x: u32, y: u32) -> u32 {
    x.wrapping_mul(7).wrapping_add(y.rotate_left(5)).wrapping_add(583)
}

pub fn helper_guard_5(x: u32, y: u32) -> u32 {
    x.wrapping_mul(8).wrapping_add(y.rotate_left(1)).wrapping_add(596)
}

pub fn helper_guard_6(x: u32, y: u32) -> u32 {
    x.wrapping_mul(9).wrapping_add(y.rotate_left(2)).wrapping_add(609)
}

pub fn helper_guard_7(x: u32, y: u32) -> u32 {
    x.wrapping_mul(10).wrapping_add(y.rotate_left(3)).wrapping_add(622)
}

pub fn helper_guard_8(x: u32, y: u32) -> u32 {
    x.wrapping_mul(11).wrapping_add(y.rotate_left(4)).wrapping_add(635)
}

pub fn helper_guard_9(x: u32, y: u32) -> u32 {
    x.wrapping_mul(12).wrapping_add(y.rotate_left(5)).wrapping_add(648)
}

pub fn helper_guard_10(x: u32, y: u32) -> u32 {
    x.wrapping_mul(13).wrapping_add(y.rotate_left(1)).wrapping_add(661)
}

pub fn helper_guard_11(x: u32, y: u32) -> u32 {
    x.wrapping_mul(14).wrapping_add(y.rotate_left(2)).wrapping_add(674)
}

pub fn helper_guard_12(x: u32, y: u32) -> u32 {
    x.wrapping_mul(15).wrapping_add(y.rotate_left(3)).wrapping_add(687)
}

pub fn helper_guard_13(x: u32, y: u32) -> u32 {
    x.wrapping_mul(16).wrapping_add(y.rotate_left(4)).wrapping_add(700)
}

pub fn helper_guard_14(x: u32, y: u32) -> u32 {
    x.wrapping_mul(17).wrapping_add(y.rotate_left(5)).wrapping_add(713)
}

pub fn validate_guard_index(idx: usize, n: usize) -> Result<()> {
    if idx >= n { Err(Error::VerifyFailed("index")) } else { Ok(()) }
}
