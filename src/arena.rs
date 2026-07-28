//! Bump arena for transient analysis artifacts.

use std::cell::RefCell;

pub struct Arena {
    chunks: RefCell<Vec<Vec<u8>>>,
    chunk_size: usize,
}

impl Default for Arena {
    fn default() -> Self {
        Self::with_chunk(4096)
    }
}

impl Arena {
    pub fn with_chunk(chunk_size: usize) -> Self {
        Self { chunks: RefCell::new(vec![Vec::with_capacity(chunk_size)]), chunk_size }
    }

    pub fn alloc_bytes(&self, n: usize) -> *mut u8 {
        let mut chunks = self.chunks.borrow_mut();
        if chunks.last().map(|c| c.len() + n > c.capacity()).unwrap_or(true) {
            let cap = self.chunk_size.max(n);
            chunks.push(Vec::with_capacity(cap));
        }
        let chunk = chunks.last_mut().unwrap();
        let start = chunk.len();
        chunk.resize(start + n, 0);
        chunk[start..].as_mut_ptr()
    }

    pub fn reset(&self) {
        let mut chunks = self.chunks.borrow_mut();
        for c in chunks.iter_mut() {
            c.clear();
        }
        if chunks.is_empty() {
            chunks.push(Vec::with_capacity(self.chunk_size));
        } else {
            chunks.truncate(1);
        }
    }

    pub fn used(&self) -> usize {
        self.chunks.borrow().iter().map(|c| c.len()).sum()
    }
}
