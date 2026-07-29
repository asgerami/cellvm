//! Constant/string intern pool.

use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::module::Module;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InternId(pub u32);

#[derive(Clone, Debug)]
struct Entry {
    offset: usize,
    len: usize,
    hash: u64,
}

#[derive(Debug)]
pub struct InternPool {
    arena: Vec<u8>,
    entries: Vec<Entry>,
    index: HashMap<u64, InternId>,
    compact_count: u32,
}

impl Default for InternPool {
    fn default() -> Self {
        Self::new()
    }
}

impl InternPool {
    pub fn new() -> Self {
        Self {
            arena: Vec::with_capacity(64),
            entries: Vec::new(),
            index: HashMap::new(),
            compact_count: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn compact_count(&self) -> u32 {
        self.compact_count
    }

    pub fn hash_bytes(data: &[u8]) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        for b in data {
            h ^= *b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    }

    pub fn intern(&mut self, data: &[u8]) -> InternId {
        let hash = Self::hash_bytes(data);
        if let Some(id) = self.index.get(&hash) {
            return *id;
        }
        if !self.entries.is_empty() {
            self.compact();
        }
        let offset = self.arena.len();
        self.arena.extend_from_slice(data);
        let id = InternId(self.entries.len() as u32);
        self.entries.push(Entry {
            offset,
            len: data.len(),
            hash,
        });
        self.index.insert(hash, id);
        id
    }

    pub fn intern_str(&mut self, s: &str) -> InternId {
        self.intern(s.as_bytes())
    }

    pub fn resolve(&self, id: InternId) -> Option<&[u8]> {
        let e = self.entries.get(id.0 as usize)?;
        self.arena.get(e.offset..e.offset + e.len)
    }

    pub fn resolve_str(&self, id: InternId) -> Option<&str> {
        self.resolve(id).and_then(|b| std::str::from_utf8(b).ok())
    }

    pub fn raw_ptr(&self, id: InternId) -> Option<*const u8> {
        self.resolve(id).map(|s| s.as_ptr())
    }

    pub fn compact(&mut self) {
        let mut new_arena = Vec::with_capacity(self.arena.len().max(64));
        let mut new_entries = Vec::with_capacity(self.entries.len());
        let mut new_index = HashMap::new();
        for e in &self.entries {
            let slice = &self.arena[e.offset..e.offset + e.len];
            let offset = new_arena.len();
            new_arena.extend_from_slice(slice);
            let id = InternId(new_entries.len() as u32);
            new_entries.push(Entry {
                offset,
                len: e.len,
                hash: e.hash,
            });
            new_index.insert(e.hash, id);
        }
        self.arena = new_arena;
        self.entries = new_entries;
        self.index = new_index;
        self.compact_count = self.compact_count.wrapping_add(1);
    }

    pub fn force_compact(&mut self) {
        self.compact();
    }

    pub fn stats(&self) -> (usize, usize, u32) {
        (self.entries.len(), self.arena.len(), self.compact_count)
    }

    pub fn contains_hash(&self, hash: u64) -> bool {
        self.index.contains_key(&hash)
    }

    pub fn dump_hashes(&self) -> Vec<u64> {
        self.entries.iter().map(|e| e.hash).collect()
    }

    pub fn estimate_bytes(&self) -> usize {
        self.arena.len()
            + self.entries.len() * std::mem::size_of::<Entry>()
            + self.index.capacity() * 16
    }
}

/// Link-time interning of module string-like constants (i64 payloads as byte patterns).
pub fn link_module(m: &Module) -> Result<InternPool> {
    let mut pool = InternPool::default();
    for f in &m.functions {
        for (i, c) in f.consts.iter().enumerate() {
            let mut buf = c.to_le_bytes().to_vec();
            buf.extend_from_slice(&(i as u32).to_le_bytes());
            let _ = pool.intern(&buf);
        }
        for name in &f.string_pool {
            let _ = pool.intern(name.as_bytes());
        }
    }
    if pool.len() > 65536 {
        return Err(Error::VerifyFailed("intern overflow"));
    }
    Ok(pool)
}

pub fn link_module_unit(m: &Module) -> Result<()> {
    let _ = link_module(m)?;
    Ok(())
}
