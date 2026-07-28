//! Heap / object registry for deserialize and runtime objects.
//!
//! INV-OWN-01: registration transfers ownership to the registry.

use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::value::Obj;

#[derive(Debug, Default)]
pub struct Registry {
    /// Owned object pointers (freed on Drop).
    slots: Vec<*mut Obj>,
    by_kind: HashMap<u32, usize>,
    generation: u32,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    pub fn generation(&self) -> u32 {
        self.generation
    }

    pub fn alloc_obj(kind: u32) -> Box<Obj> {
        Box::new(Obj { kind, mark: 0 })
    }

    /// Register transfers ownership into the registry.
    pub fn register(&mut self, obj: Box<Obj>) -> usize {
        let kind = obj.kind;
        let ptr = Box::into_raw(obj);
        let idx = self.slots.len();
        self.slots.push(ptr);
        self.by_kind.insert(kind, idx);
        self.generation = self.generation.wrapping_add(1);
        idx
    }

    pub fn get(&self, idx: usize) -> Option<&Obj> {
        let p = *self.slots.get(idx)?;
        if p.is_null() {
            return None;
        }
        Some(unsafe { &*p })
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Obj> {
        let p = *self.slots.get(idx)?;
        if p.is_null() {
            return None;
        }
        Some(unsafe { &mut *p })
    }

    pub fn find_kind(&self, kind: u32) -> Option<usize> {
        self.by_kind.get(&kind).copied()
    }

    pub fn mark_all(&mut self, mark: u32) {
        for p in &self.slots {
            if !p.is_null() {
                unsafe { (**p).mark = mark; }
            }
        }
    }

    pub fn sweep_unmarked(&mut self, keep: u32) {
        for p in &mut self.slots {
            if p.is_null() {
                continue;
            }
            let mark = unsafe { (**p).mark };
            if mark != keep {
                unsafe {
                    drop(Box::from_raw(*p));
                }
                *p = std::ptr::null_mut();
            }
        }
        self.slots.retain(|p| !p.is_null());
        self.rebuild_index();
    }

    fn rebuild_index(&mut self) {
        self.by_kind.clear();
        for (i, p) in self.slots.iter().enumerate() {
            if !p.is_null() {
                let kind = unsafe { (**p).kind };
                self.by_kind.insert(kind, i);
            }
        }
    }

    /// Unregister without freeing (correct error-path helper).
    pub fn unregister(&mut self, idx: usize) -> Option<Box<Obj>> {
        let p = self.slots.get_mut(idx)?;
        if p.is_null() {
            return None;
        }
        let ptr = *p;
        *p = std::ptr::null_mut();
        self.rebuild_index();
        Some(unsafe { Box::from_raw(ptr) })
    }

    pub fn stats(&self) -> (usize, u32) {
        (self.slots.iter().filter(|p| !p.is_null()).count(), self.generation)
    }
}

impl Drop for Registry {
    fn drop(&mut self) {
        for p in self.slots.drain(..) {
            if !p.is_null() {
                unsafe {
                    drop(Box::from_raw(p));
                }
            }
        }
    }
}

pub fn alloc_obj(kind: u32) -> Box<Obj> {
    Registry::alloc_obj(kind)
}

/// Validate object kind ranges used by deser.
pub fn validate_kind(kind: u32) -> Result<()> {
    if kind == 0 || kind == 0xFFFF_FFFE {
        return Err(Error::DeserFailed("reserved kind"));
    }
    Ok(())
}

pub fn validate_mark(mark: u32) -> Result<()> {
    if mark > 0x00FF_FFFF {
        return Err(Error::DeserFailed("mark out of range"));
    }
    Ok(())
}

pub fn kind_class(kind: u32) -> u8 {
    match kind {
        1..=15 => 1,
        16..=255 => 2,
        256..=0xFFFF => 3,
        _ => 4,
    }
}

pub fn estimate_graph_bytes(n_nodes: usize, avg_children: usize) -> usize {
    n_nodes
        .saturating_mul(std::mem::size_of::<Obj>())
        .saturating_add(n_nodes.saturating_mul(avg_children).saturating_mul(8))
}
