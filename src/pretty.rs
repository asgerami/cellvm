//! Human-readable rendering of modules and values.

use crate::disasm;
use crate::module::{Function, Module};
use crate::value::{Obj, Tag, Value};

pub fn function(f: &Function) -> String {
    let mut s = format!(
        "fn {} // max_locals={} frame_size={} consts={} strings={} upvalues={}\n",
        f.name,
        f.max_locals,
        f.frame_size,
        f.consts.len(),
        f.string_pool.len(),
        f.upvalues.len()
    );
    s.push_str(&disasm::render(f));
    s
}

pub fn module(m: &Module) -> String {
    m.functions.iter().map(function).collect::<Vec<_>>().join("\n")
}

pub fn value(v: &Value) -> String {
    match v.tag() {
        Tag::Int => format!("Int({})", v.as_int().unwrap_or(0)),
        Tag::Ref => format!("Ref({})", v.as_ref_id().unwrap_or(0)),
    }
}

pub fn obj(o: &Obj) -> String {
    format!("Obj{{kind={}, mark={}}}", o.kind, o.mark)
}

pub fn hex_preview(bytes: &[u8], max: usize) -> String {
    let n = bytes.len().min(max);
    let mut s = bytes[..n]
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ");
    if bytes.len() > max {
        s.push_str(" ...");
    }
    s
}
