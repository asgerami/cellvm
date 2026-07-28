//! cellvm — compact bytecode VM with a load → verify → link → execute pipeline.
//!
//! The verifier establishes structural and type invariants in safe Rust. The
//! executor uses unchecked fast paths that assume those invariants hold.

pub mod abi;
pub mod analysis;
pub mod arena;
pub mod asm;
pub mod bitvec;
pub mod bytecode;
pub mod snapshot;
pub mod limits;
pub mod decode;
pub mod builder;
pub mod format;
pub mod builtins;
pub mod catalog;
pub mod cfg;
pub mod constpool;
pub mod debug;
pub mod deser;
pub mod disasm;
pub mod docs;
pub mod encode;
pub mod error;
pub mod exec;
pub mod gc;
pub mod heap;
pub mod host;
pub mod intern;
pub mod interp;
pub mod interval;
pub mod ir;
pub mod isa;
pub mod liveness;
pub mod loader;
pub mod mirror;
pub mod module;
pub mod optimize;
pub mod passes;
pub mod pipeline;
pub mod pretty;
pub mod profile;
pub mod reloc;
pub mod report;
pub mod runtime;
pub mod session;
pub mod source;
pub mod span;
pub mod stackmap;
pub mod stats;
pub mod validate_extra;
pub mod value;
pub mod verifier;

pub use pipeline::{load_run, LoadRunOpts};
