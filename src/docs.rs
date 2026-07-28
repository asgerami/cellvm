//! Narrative documentation strings for the CLVM pipeline stages.

pub fn pipeline_overview() -> &'static str {
    "CLVM modules are loaded from a little-endian binary container, verified for frame and type safety, linked against an intern pool, then executed."
}

pub fn loader_notes() -> &'static str {
    "The loader is panic-free and returns structured errors for truncated inputs, bad magic, unknown opcodes, and jump targets that miss instruction boundaries."
}

pub fn verifier_notes() -> &'static str {
    "Frame verification tracks WINDOW bases and local slots. Type-flow verification builds a block graph and checks GETFIELD against abstract reference types."
}

pub fn executor_notes() -> &'static str {
    "The executor assumes verifier invariants and uses unchecked indexing on hot paths. Call frames, intern handles, and upvalues are the primary unsafe surfaces."
}

pub fn format_notes() -> &'static str {
    "Binary layout: magic CLVM | code_len | code | nconst | consts | optional trailers. Trailers: string pool (0xFFFFFFFF), extra functions (0xFFFFFFFE), or jump fixups."
}

pub fn fuzz_notes() -> &'static str {
    "Harnesses exercise load/run, typeflow, deserialize, and assemble entry points independently so each trust boundary can be stressed in isolation."
}

pub fn all_docs() -> String {
    [
        pipeline_overview(),
        loader_notes(),
        verifier_notes(),
        executor_notes(),
        format_notes(),
        fuzz_notes(),
    ]
    .concat()
}

pub fn stage_name(i: usize) -> &'static str {
    match i {
        0 => "load",
        1 => "verify",
        2 => "link",
        3 => "execute",
        _ => "unknown",
    }
}

pub fn stage_count() -> usize { 4 }
