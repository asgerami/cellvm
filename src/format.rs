//! CLVM container format versions, feature flags, and section codecs.

use crate::error::{Error, Result};

pub const MAGIC: &[u8; 4] = b"CLVM";
pub const VERSION_1: u16 = 1;
pub const VERSION_2: u16 = 2;

pub const FEATURE_STRINGS: u32 = 1 << 0;
pub const FEATURE_MULTI_FUNC: u32 = 1 << 1;
pub const FEATURE_UPVALUES: u32 = 1 << 2;
pub const FEATURE_DEBUG_MAP: u32 = 1 << 3;
pub const FEATURE_RELOCS: u32 = 1 << 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FormatHeader {
    pub version: u16,
    pub features: u32,
    pub code_len: u32,
    pub const_count: u32,
}

impl FormatHeader {
    pub fn v1(code_len: u32, const_count: u32) -> Self {
        Self {
            version: VERSION_1,
            features: 0,
            code_len,
            const_count,
        }
    }

    pub fn has(&self, feature: u32) -> bool {
        self.features & feature != 0
    }

    pub fn enable(&mut self, feature: u32) {
        self.features |= feature;
    }

    pub fn encoded_size(&self) -> usize {
        // magic(4) + version(2) + features(4) + code_len(4) + const_count(4)  [v2]
        // v1 layout historically: magic(4) + code_len(4) only before consts
        if self.version >= VERSION_2 {
            18
        } else {
            8
        }
    }
}

pub fn parse_magic(input: &[u8]) -> Result<()> {
    if input.len() < 4 {
        return Err(Error::Truncated { at: "magic" });
    }
    if &input[0..4] != MAGIC {
        return Err(Error::BadMagic);
    }
    Ok(())
}

pub fn detect_version(input: &[u8]) -> Result<u16> {
    parse_magic(input)?;
    if input.len() >= 6 {
        let v = u16::from_le_bytes(input[4..6].try_into().unwrap());
        if v == VERSION_2 {
            return Ok(VERSION_2);
        }
    }
    // Legacy containers store code_len at offset 4; treat as v1.
    Ok(VERSION_1)
}

pub fn write_v1_prefix(code_len: u32) -> Vec<u8> {
    let mut out = Vec::with_capacity(8);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&code_len.to_le_bytes());
    out
}

pub fn write_v2_prefix(h: &FormatHeader) -> Vec<u8> {
    let mut out = Vec::with_capacity(18);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&h.version.to_le_bytes());
    out.extend_from_slice(&h.features.to_le_bytes());
    out.extend_from_slice(&h.code_len.to_le_bytes());
    out.extend_from_slice(&h.const_count.to_le_bytes());
    out
}

pub fn section_tag_name(tag: u32) -> &'static str {
    match tag {
        0xFFFF_FFFF => "strings",
        0xFFFF_FFFE => "functions",
        0xFFFF_FFFD => "debug_map",
        0xFFFF_FFFC => "relocs",
        _ => "jump_fixup",
    }
}

pub fn is_trailer_tag(tag: u32) -> bool {
    matches!(tag, 0xFFFF_FFFF | 0xFFFF_FFFE | 0xFFFF_FFFD | 0xFFFF_FFFC) || tag < 0x8000_0000
}

#[derive(Clone, Debug, Default)]
pub struct SectionStats {
    pub code_bytes: usize,
    pub const_bytes: usize,
    pub string_bytes: usize,
    pub func_bytes: usize,
    pub reloc_count: usize,
}

impl SectionStats {
    pub fn total(&self) -> usize {
        self.code_bytes + self.const_bytes + self.string_bytes + self.func_bytes
    }

    pub fn render(&self) -> String {
        format!(
            "code={} consts={} strings={} funcs={} relocs={} total={}",
            self.code_bytes,
            self.const_bytes,
            self.string_bytes,
            self.func_bytes,
            self.reloc_count,
            self.total()
        )
    }
}

pub fn estimate_const_bytes(n: usize) -> usize {
    4 + n * 8
}

pub fn estimate_string_section(strings: &[String]) -> usize {
    8 + strings.iter().map(|s| 4 + s.len()).sum::<usize>()
}

pub fn estimate_func_section(funcs: &[&[u8]]) -> usize {
    8 + funcs.iter().map(|f| 4 + f.len()).sum::<usize>()
}

pub fn validate_header_sanity(h: &FormatHeader) -> Result<()> {
    if h.code_len > 1 << 24 {
        return Err(Error::VerifyFailed("code too large"));
    }
    if h.const_count > 1 << 20 {
        return Err(Error::VerifyFailed("too many consts"));
    }
    if h.version != VERSION_1 && h.version != VERSION_2 {
        return Err(Error::VerifyFailed("unsupported version"));
    }
    Ok(())
}

pub fn feature_list(features: u32) -> Vec<&'static str> {
    let mut out = Vec::new();
    if features & FEATURE_STRINGS != 0 {
        out.push("strings");
    }
    if features & FEATURE_MULTI_FUNC != 0 {
        out.push("multi_func");
    }
    if features & FEATURE_UPVALUES != 0 {
        out.push("upvalues");
    }
    if features & FEATURE_DEBUG_MAP != 0 {
        out.push("debug_map");
    }
    if features & FEATURE_RELOCS != 0 {
        out.push("relocs");
    }
    out
}
