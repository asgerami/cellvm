//! Runtime and verifier resource limits.

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_code_bytes: usize,
    pub max_consts: usize,
    pub max_strings: usize,
    pub max_functions: usize,
    pub max_call_depth: usize,
    pub max_locals: u16,
    pub max_frame_size: u32,
    pub max_intern_bytes: usize,
    pub max_deser_nodes: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self::production()
    }
}

impl Limits {
    pub fn production() -> Self {
        Self {
            max_code_bytes: 1 << 20,
            max_consts: 1 << 16,
            max_strings: 4096,
            max_functions: 256,
            max_call_depth: 1024,
            max_locals: 256,
            max_frame_size: 4096,
            max_intern_bytes: 1 << 22,
            max_deser_nodes: 4096,
        }
    }

    pub fn fuzzing() -> Self {
        Self {
            max_code_bytes: 1 << 16,
            max_consts: 4096,
            max_strings: 1024,
            max_functions: 64,
            max_call_depth: 128,
            max_locals: 64,
            max_frame_size: 512,
            max_intern_bytes: 1 << 18,
            max_deser_nodes: 1024,
        }
    }

    pub fn tiny() -> Self {
        Self {
            max_code_bytes: 4096,
            max_consts: 256,
            max_strings: 64,
            max_functions: 16,
            max_call_depth: 32,
            max_locals: 16,
            max_frame_size: 64,
            max_intern_bytes: 1 << 14,
            max_deser_nodes: 128,
        }
    }

    pub fn check_code_len(&self, n: usize) -> bool {
        n <= self.max_code_bytes
    }

    pub fn check_consts(&self, n: usize) -> bool {
        n <= self.max_consts
    }

    pub fn check_strings(&self, n: usize) -> bool {
        n <= self.max_strings
    }

    pub fn check_functions(&self, n: usize) -> bool {
        n <= self.max_functions
    }

    pub fn check_call_depth(&self, n: usize) -> bool {
        n <= self.max_call_depth
    }

    pub fn check_frame(&self, locals: u16, frame: u32) -> bool {
        locals <= self.max_locals && frame <= self.max_frame_size && (locals as u32) <= frame
    }

    pub fn check_intern_bytes(&self, n: usize) -> bool {
        n <= self.max_intern_bytes
    }

    pub fn check_deser_nodes(&self, n: usize) -> bool {
        n <= self.max_deser_nodes
    }

    pub fn render(&self) -> String {
        format!(
            "limits code={} consts={} strings={} funcs={} depth={} locals={} frame={} intern={} deser={}",
            self.max_code_bytes,
            self.max_consts,
            self.max_strings,
            self.max_functions,
            self.max_call_depth,
            self.max_locals,
            self.max_frame_size,
            self.max_intern_bytes,
            self.max_deser_nodes
        )
    }
}

pub fn clamp_call_depth(requested: usize, limits: &Limits) -> usize {
    requested.min(limits.max_call_depth)
}

pub fn clamp_frame(locals: u16, frame: u32, limits: &Limits) -> (u16, u32) {
    let locals = locals.min(limits.max_locals);
    let frame = frame.min(limits.max_frame_size).max(locals as u32);
    (locals, frame)
}

pub fn budget_remaining(used: usize, limit: usize) -> usize {
    limit.saturating_sub(used)
}

pub fn exceeds(used: usize, limit: usize) -> bool {
    used > limit
}
