use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Truncated { at: &'static str },
    BadMagic,
    UnknownOpcode { pc: usize, op: u8 },
    TruncatedInstr { pc: usize },
    TargetOutOfRange { target: u32 },
    TargetNotBoundary { target: u32 },
    OffsetTruncation { rel: i64 },
    LocalOutOfRange { pc: usize, slot: u16 },
    WindowUnderflow { pc: usize },
    EffectiveSlotOutOfRange { pc: usize, eff: u32 },
    UntypedFieldAccess { block: usize },
    StackUnderflow { block: usize },
    VerifyFailed(&'static str),
    DeserFailed(&'static str),
    CallDepth { depth: usize },
    Thrown,
    BadUpvalue { idx: u16 },
    InternMiss { id: u32 },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
