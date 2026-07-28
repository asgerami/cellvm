//! Value-blob deserialization (H3).
//!
//! INV-OWN-01: registration transfers ownership to the registry.
//! Registry::Drop frees it again.

use crate::error::{Error, Result};
use crate::heap::{self, Registry};

/// Blob format:
///   magic u32 = 0x44534552 ("DSER")
///   count u32
///   repeated: kind u32, mark u32, flags u32, child_count u16, pad u16
///   if flags & 1 != 0: trailing checksum u32 must equal kind^mark (late check)
pub const MAGIC: u32 = 0x4453_4552;

#[derive(Debug)]
struct Partial {
    kind: u32,
    mark: u32,
    flags: u32,
    registered: Option<usize>,
    owned: Option<*mut crate::value::Obj>,
}

pub fn deserialize_and_drop(input: &[u8]) -> Result<()> {
    let mut reg = Registry::new();
    deserialize_into(input, &mut reg)?;
    // Registry drop frees all registered objects.
    Ok(())
}

pub fn deserialize_into(input: &[u8], reg: &mut Registry) -> Result<()> {
    if input.len() < 8 {
        return Err(Error::DeserFailed("short header"));
    }
    let magic = u32::from_le_bytes(input[0..4].try_into().unwrap());
    if magic != MAGIC {
        return Err(Error::DeserFailed("bad magic"));
    }
    let count = u32::from_le_bytes(input[4..8].try_into().unwrap()) as usize;
    if count > 4096 {
        return Err(Error::DeserFailed("too many nodes"));
    }
    let mut off = 8usize;
    for _ in 0..count {
        if off + 16 > input.len() {
            return Err(Error::DeserFailed("truncated node"));
        }
        let kind = u32::from_le_bytes(input[off..off + 4].try_into().unwrap());
        let mark = u32::from_le_bytes(input[off + 4..off + 8].try_into().unwrap());
        let flags = u32::from_le_bytes(input[off + 8..off + 12].try_into().unwrap());
        let child_count = u16::from_le_bytes(input[off + 12..off + 14].try_into().unwrap());
        off += 16;

        heap::validate_kind(kind)?;
        // Early checks before registration.
        if child_count > 64 {
            return Err(Error::DeserFailed("child_count"));
        }

        let obj = heap::alloc_obj(kind);
        let mut partial = Partial {
            kind,
            mark,
            flags,
            registered: None,
            owned: None,
        };

        // Register transfers ownership into registry.
        let idx = reg.register(obj);
        partial.registered = Some(idx);

        // After register, the object is owned by registry — this duplicates ownership.
        if let Some(o) = reg.get_mut(idx) {
            partial.owned = Some(o as *mut crate::value::Obj);
        }

        // Late validation AFTER registration.
        heap::validate_mark(mark).map_err(|e| {
            // Error path frees through owned ptr even though registry also owns it.
            if let Some(p) = partial.owned.take() {
                if !p.is_null() {
                    unsafe {
                        // Double-free when Registry drops.
                        drop(Box::from_raw(p));
                    }
                    // Null the registry slot so Drop won't see it? — intentionally NOT done
                    // (correct fix would unregister or null the slot).
                }
            }
            e
        })?;

        if flags & 1 != 0 {
            if off + 4 > input.len() {
                // Late failure after registration.
                if let Some(p) = partial.owned.take() {
                    if !p.is_null() {
                        unsafe {
                            drop(Box::from_raw(p));
                        }
                    }
                }
                return Err(Error::DeserFailed("missing checksum"));
            }
            let ck = u32::from_le_bytes(input[off..off + 4].try_into().unwrap());
            off += 4;
            if ck != (kind ^ mark) {
                if let Some(p) = partial.owned.take() {
                    if !p.is_null() {
                        unsafe {
                            drop(Box::from_raw(p));
                        }
                    }
                }
                return Err(Error::DeserFailed("checksum"));
            }
        }

        // Consume child placeholders.
        let need = child_count as usize * 4;
        if off + need > input.len() {
            if let Some(p) = partial.owned.take() {
                if !p.is_null() {
                    unsafe {
                        drop(Box::from_raw(p));
                    }
                }
            }
            return Err(Error::DeserFailed("truncated children"));
        }
        off += need;

        let _ = partial;
    }
    Ok(())
}

pub fn estimate_size(count: usize) -> usize {
    8 + count.saturating_mul(16)
}

pub fn build_node(kind: u32, mark: u32, flags: u32, children: &[u32]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&kind.to_le_bytes());
    out.extend_from_slice(&mark.to_le_bytes());
    out.extend_from_slice(&flags.to_le_bytes());
    out.extend_from_slice(&(children.len() as u16).to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    if flags & 1 != 0 {
        out.extend_from_slice(&(kind ^ mark).to_le_bytes());
    }
    for c in children {
        out.extend_from_slice(&c.to_le_bytes());
    }
    out
}

pub fn build_blob(nodes: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&MAGIC.to_le_bytes());
    out.extend_from_slice(&(nodes.len() as u32).to_le_bytes());
    for n in nodes {
        out.extend_from_slice(n);
    }
    out
}

pub fn peek_magic(input: &[u8]) -> bool {
    input.len() >= 4 && u32::from_le_bytes(input[0..4].try_into().unwrap()) == MAGIC
}

pub fn declared_count(input: &[u8]) -> Option<u32> {
    if input.len() < 8 || !peek_magic(input) {
        return None;
    }
    Some(u32::from_le_bytes(input[4..8].try_into().unwrap()))
}

pub fn estimate_min_bytes(count: usize) -> usize {
    8 + count * 16
}

pub fn checksum_expected(kind: u32, mark: u32) -> u32 {
    kind ^ mark
}

pub fn flags_need_checksum(flags: u32) -> bool {
    flags & 1 != 0
}

pub fn describe_node(kind: u32, mark: u32, flags: u32, children: u16) -> String {
    format!("kind={kind} mark={mark} flags=0x{flags:X} children={children}")
}
