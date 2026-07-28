//! Relocation records for jump lowering and multi-function linking.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelocKind {
    JumpI8,
    FuncIndex,
    ConstIndex,
    StringIndex,
}

#[derive(Clone, Copy, Debug)]
pub struct Reloc {
    pub at: u32,
    pub kind: RelocKind,
    pub symbol: u32,
}

#[derive(Clone, Debug, Default)]
pub struct RelocTable {
    pub entries: Vec<Reloc>,
}

impl RelocTable {
    pub fn push(&mut self, at: usize, kind: RelocKind, symbol: u32) {
        self.entries.push(Reloc { at: at as u32, kind, symbol });
    }

    pub fn jumps(&self) -> impl Iterator<Item = &Reloc> {
        self.entries.iter().filter(|r| r.kind == RelocKind::JumpI8)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

pub fn encode_tag_jump(at: u32, target: u32) -> [u8; 8] {
    let mut out = [0u8; 8];
    out[..4].copy_from_slice(&at.to_le_bytes());
    out[4..].copy_from_slice(&target.to_le_bytes());
    out
}
