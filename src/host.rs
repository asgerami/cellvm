//! Hostcall registry — declared imports for native bridges.

use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct HostFn {
    pub name: String,
    pub arity: u8,
    pub tag: u32,
}

#[derive(Clone, Debug, Default)]
pub struct HostTable {
    by_name: BTreeMap<String, HostFn>,
    by_tag: BTreeMap<u32, String>,
}

impl HostTable {
    pub fn register(&mut self, name: impl Into<String>, arity: u8, tag: u32) {
        let name = name.into();
        self.by_tag.insert(tag, name.clone());
        self.by_name.insert(name.clone(), HostFn { name, arity, tag });
    }

    pub fn lookup(&self, name: &str) -> Option<&HostFn> {
        self.by_name.get(name)
    }

    pub fn lookup_tag(&self, tag: u32) -> Option<&str> {
        self.by_tag.get(&tag).map(|s| s.as_str())
    }

    pub fn len(&self) -> usize {
        self.by_name.len()
    }
}

pub fn standard_hosts() -> HostTable {
    let mut t = HostTable::default();
    t.register("print", 1, 1);
    t.register("clock", 0, 2);
    t.register("len", 1, 3);
    t.register("assert", 1, 4);
    t.register("hash", 1, 5);
    t.register("read", 2, 6);
    t.register("write", 2, 7);
    t
}

pub fn encode_host_stub(tag: u32, arity: u8) -> Vec<u8> {
    let mut v = Vec::with_capacity(8);
    v.extend_from_slice(&tag.to_le_bytes());
    v.push(arity);
    v.extend_from_slice(&[0, 0, 0]);
    v
}
