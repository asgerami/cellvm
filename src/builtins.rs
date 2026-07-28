//! Host-facing builtin helpers and value coercions used by tools and tests.

use crate::error::{Error, Result};
use crate::module::{Function, Module};
use crate::value::{Tag, Value};

#[derive(Clone, Debug, Default)]
pub struct BuiltinRegistry {
    names: Vec<&'static str>,
    arities: Vec<u8>,
}

impl BuiltinRegistry {
    pub fn standard() -> Self {
        let mut r = Self::default();
        for (n, a) in [
            ("print", 1u8),
            ("len", 1),
            ("hash", 1),
            ("assert", 1),
            ("add", 2),
            ("sub", 2),
            ("mul", 2),
            ("div", 2),
            ("eq", 2),
            ("lt", 2),
            ("lte", 2),
            ("gt", 2),
            ("gte", 2),
            ("and", 2),
            ("or", 2),
            ("not", 1),
            ("str_len", 1),
            ("str_eq", 2),
            ("concat", 2),
            ("arr_len", 1),
            ("arr_get", 2),
            ("arr_set", 3),
            ("clock", 0),
            ("rand", 0),
            ("id", 1),
        ] {
            r.register(n, a);
        }
        r
    }

    pub fn register(&mut self, name: &'static str, arity: u8) {
        if !self.names.contains(&name) {
            self.names.push(name);
            self.arities.push(arity);
        }
    }

    pub fn lookup(&self, name: &str) -> Option<(usize, u8)> {
        self.names
            .iter()
            .position(|n| *n == name)
            .map(|i| (i, self.arities[i]))
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn names(&self) -> &[&'static str] {
        &self.names
    }

    pub fn arity_of(&self, idx: usize) -> Option<u8> {
        self.arities.get(idx).copied()
    }
}

pub fn coerce_int(v: Value) -> Result<i64> {
    v.as_int().ok_or(Error::VerifyFailed("expected int"))
}

pub fn coerce_ref(v: Value) -> Result<u32> {
    v.as_ref_id().ok_or(Error::VerifyFailed("expected ref"))
}

pub fn truthy(v: Value) -> bool {
    match v.tag() {
        Tag::Int => v.as_int().unwrap_or(0) != 0,
        Tag::Ref => v.as_ref_id().unwrap_or(0) != 0,
    }
}

pub fn binary_int(op: &str, a: i64, b: i64) -> Result<i64> {
    Ok(match op {
        "add" => a.wrapping_add(b),
        "sub" => a.wrapping_sub(b),
        "mul" => a.wrapping_mul(b),
        "div" => {
            if b == 0 {
                return Err(Error::VerifyFailed("div0"));
            }
            a.wrapping_div(b)
        }
        "mod" => {
            if b == 0 {
                return Err(Error::VerifyFailed("mod0"));
            }
            a.wrapping_rem(b)
        }
        "eq" => i64::from(a == b),
        "lt" => i64::from(a < b),
        "lte" => i64::from(a <= b),
        "gt" => i64::from(a > b),
        "gte" => i64::from(a >= b),
        "and" => a & b,
        "or" => a | b,
        "xor" => a ^ b,
        "shl" => a.wrapping_shl((b as u32) & 63),
        "shr" => ((a as u64) >> ((b as u32) & 63)) as i64,
        _ => return Err(Error::VerifyFailed("unknown binary")),
    })
}

pub fn unary_int(op: &str, a: i64) -> Result<i64> {
    Ok(match op {
        "not" => i64::from(a == 0),
        "neg" => a.wrapping_neg(),
        "abs" => a.wrapping_abs(),
        "id" => a,
        "ctz" => (a as u64).trailing_zeros() as i64,
        "clz" => (a as u64).leading_zeros() as i64,
        "popcnt" => (a as u64).count_ones() as i64,
        _ => return Err(Error::VerifyFailed("unknown unary")),
    })
}

pub fn hash_i64(v: i64) -> u64 {
    let mut x = v as u64;
    x = x.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(0x85EBCA77C2B2AE63);
    x ^= x >> 33;
    x = x.wrapping_mul(0xFF51AFD7ED558CCD);
    x ^= x >> 33;
    x
}

pub fn hash_bytes(data: &[u8]) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for b in data {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

pub fn module_exports(m: &Module) -> Vec<String> {
    m.functions.iter().map(|f| f.name.clone()).collect()
}

pub fn function_arity_hint(f: &Function) -> u16 {
    f.max_locals.min(8)
}

pub fn summarize_builtins(r: &BuiltinRegistry) -> String {
    let mut s = format!("builtins={}\n", r.len());
    for (i, name) in r.names().iter().enumerate() {
        s.push_str(&format!("  [{i}] {name}/{}\n", r.arities[i]));
    }
    s
}

pub fn eval_const_expr(op: &str, args: &[i64]) -> Result<i64> {
    match (op, args.len()) {
        ("add" | "sub" | "mul" | "div" | "mod" | "eq" | "lt" | "lte" | "gt" | "gte" | "and" | "or" | "xor" | "shl" | "shr", 2) => {
            binary_int(op, args[0], args[1])
        }
        ("not" | "neg" | "abs" | "id" | "ctz" | "clz" | "popcnt", 1) => unary_int(op, args[0]),
        ("hash", 1) => Ok(hash_i64(args[0]) as i64),
        _ => Err(Error::VerifyFailed("bad const expr")),
    }
}

pub fn fold_const_list(ops: &[(&str, Vec<i64>)]) -> Result<Vec<i64>> {
    let mut out = Vec::with_capacity(ops.len());
    for (op, args) in ops {
        out.push(eval_const_expr(op, args)?);
    }
    Ok(out)
}
