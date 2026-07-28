//! Calling convention / frame layout helpers.

#[derive(Clone, Copy, Debug)]
pub struct FrameLayout {
    pub locals: u32,
    pub frame_size: u32,
    pub arg_slots: u32,
    pub spill_base: u32,
}

impl FrameLayout {
    pub fn new(locals: u32, frame_size: u32) -> Self {
        let arg_slots = locals.min(4);
        Self {
            locals,
            frame_size,
            arg_slots,
            spill_base: arg_slots,
        }
    }

    pub fn slot_ok(self, base: u32, slot: u32) -> bool {
        base.saturating_add(slot) < self.frame_size
    }

    pub fn window_ok(self, base: u32, delta: i8) -> Option<u32> {
        let next = base as i64 + delta as i64;
        if next < 0 || next as u32 >= self.frame_size {
            None
        } else {
            Some(next as u32)
        }
    }
}

pub fn align_up(n: u32, align: u32) -> u32 {
    (n + align - 1) & !(align - 1)
}

pub fn word_bytes() -> usize {
    8
}

pub fn shadow_stack_bytes(depth: usize) -> usize {
    depth.saturating_mul(word_bytes() * 4)
}
