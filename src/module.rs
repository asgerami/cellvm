//! In-memory module representation.

#[derive(Clone, Debug, Default)]
pub struct Function {
    pub code: Vec<u8>,
    pub max_locals: u16,
    pub frame_size: u32,
    pub consts: Vec<i64>,
    /// String constants for intern pool.
    pub string_pool: Vec<String>,
    /// Upvalue descriptors: (is_local, index).
    pub upvalues: Vec<(bool, u16)>,
    pub name: String,
}

#[derive(Clone, Debug, Default)]
pub struct Module {
    pub functions: Vec<Function>,
}

impl Module {
    pub fn entry(&self) -> Option<&Function> {
        self.functions.first()
    }

    pub fn get(&self, idx: usize) -> Option<&Function> {
        self.functions.get(idx)
    }

    pub fn func_count(&self) -> usize {
        self.functions.len()
    }
}

impl Function {
    pub fn with_strings(mut self, strings: Vec<String>) -> Self {
        self.string_pool = strings;
        self
    }
}
