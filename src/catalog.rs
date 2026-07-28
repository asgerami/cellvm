//! Extended opcode catalog for tooling and future ISA proposals.
//! These entries are descriptive metadata — not all are wired into the
//! current executor, but disassemblers and docs consume them.

#[derive(Clone, Copy, Debug)]
pub struct CatalogEntry {
    pub id: u16,
    pub name: &'static str,
    pub meaning: &'static str,
    pub stack_in: u8,
    pub stack_out: u8,
    pub purity: bool,
}


pub fn catalog_00_arithmetic_add() -> CatalogEntry {
    CatalogEntry {
        id: 0,
        name: "arithmetic_add",
        meaning: "integer addition on stack tops",
        stack_in: 0,
        stack_out: 1,
        purity: true,
    }
}


pub fn catalog_01_arithmetic_sub() -> CatalogEntry {
    CatalogEntry {
        id: 1,
        name: "arithmetic_sub",
        meaning: "integer subtraction",
        stack_in: 1,
        stack_out: 2,
        purity: false,
    }
}


pub fn catalog_02_arithmetic_mul() -> CatalogEntry {
    CatalogEntry {
        id: 2,
        name: "arithmetic_mul",
        meaning: "integer multiplication",
        stack_in: 2,
        stack_out: 1,
        purity: true,
    }
}


pub fn catalog_03_arithmetic_div() -> CatalogEntry {
    CatalogEntry {
        id: 3,
        name: "arithmetic_div",
        meaning: "integer division with zero check",
        stack_in: 0,
        stack_out: 2,
        purity: false,
    }
}


pub fn catalog_04_compare_eq() -> CatalogEntry {
    CatalogEntry {
        id: 4,
        name: "compare_eq",
        meaning: "equality compare → i8",
        stack_in: 1,
        stack_out: 1,
        purity: true,
    }
}


pub fn catalog_05_compare_lt() -> CatalogEntry {
    CatalogEntry {
        id: 5,
        name: "compare_lt",
        meaning: "less-than compare",
        stack_in: 2,
        stack_out: 2,
        purity: false,
    }
}


pub fn catalog_06_compare_le() -> CatalogEntry {
    CatalogEntry {
        id: 6,
        name: "compare_le",
        meaning: "less-or-equal compare",
        stack_in: 0,
        stack_out: 1,
        purity: true,
    }
}


pub fn catalog_07_branch_if() -> CatalogEntry {
    CatalogEntry {
        id: 7,
        name: "branch_if",
        meaning: "conditional short jump",
        stack_in: 1,
        stack_out: 2,
        purity: false,
    }
}


pub fn catalog_08_branch_if_not() -> CatalogEntry {
    CatalogEntry {
        id: 8,
        name: "branch_if_not",
        meaning: "inverted conditional",
        stack_in: 2,
        stack_out: 1,
        purity: true,
    }
}


pub fn catalog_09_loop_header() -> CatalogEntry {
    CatalogEntry {
        id: 9,
        name: "loop_header",
        meaning: "canonical loop header marker",
        stack_in: 0,
        stack_out: 2,
        purity: false,
    }
}


pub fn catalog_10_phi_merge() -> CatalogEntry {
    CatalogEntry {
        id: 10,
        name: "phi_merge",
        meaning: "SSA-like merge annotation",
        stack_in: 1,
        stack_out: 1,
        purity: true,
    }
}


pub fn catalog_11_spill_slot() -> CatalogEntry {
    CatalogEntry {
        id: 11,
        name: "spill_slot",
        meaning: "spill to frame spill region",
        stack_in: 2,
        stack_out: 2,
        purity: false,
    }
}


pub fn catalog_12_reload_slot() -> CatalogEntry {
    CatalogEntry {
        id: 12,
        name: "reload_slot",
        meaning: "reload from spill",
        stack_in: 0,
        stack_out: 1,
        purity: true,
    }
}


pub fn catalog_13_guard_null() -> CatalogEntry {
    CatalogEntry {
        id: 13,
        name: "guard_null",
        meaning: "null reference guard",
        stack_in: 1,
        stack_out: 2,
        purity: false,
    }
}


pub fn catalog_14_guard_type() -> CatalogEntry {
    CatalogEntry {
        id: 14,
        name: "guard_type",
        meaning: "runtime type guard",
        stack_in: 2,
        stack_out: 1,
        purity: true,
    }
}


pub fn catalog_15_box_int() -> CatalogEntry {
    CatalogEntry {
        id: 15,
        name: "box_int",
        meaning: "box integer into heap object",
        stack_in: 0,
        stack_out: 2,
        purity: false,
    }
}


pub fn catalog_16_unbox_int() -> CatalogEntry {
    CatalogEntry {
        id: 16,
        name: "unbox_int",
        meaning: "unbox integer",
        stack_in: 1,
        stack_out: 1,
        purity: true,
    }
}


pub fn catalog_17_array_len() -> CatalogEntry {
    CatalogEntry {
        id: 17,
        name: "array_len",
        meaning: "array length",
        stack_in: 2,
        stack_out: 2,
        purity: false,
    }
}


pub fn catalog_18_array_load() -> CatalogEntry {
    CatalogEntry {
        id: 18,
        name: "array_load",
        meaning: "array element load",
        stack_in: 0,
        stack_out: 1,
        purity: true,
    }
}


pub fn catalog_19_array_store() -> CatalogEntry {
    CatalogEntry {
        id: 19,
        name: "array_store",
        meaning: "array element store",
        stack_in: 1,
        stack_out: 2,
        purity: false,
    }
}


pub fn catalog_20_str_len() -> CatalogEntry {
    CatalogEntry {
        id: 20,
        name: "str_len",
        meaning: "string length via intern id",
        stack_in: 2,
        stack_out: 1,
        purity: true,
    }
}


pub fn catalog_21_str_cat() -> CatalogEntry {
    CatalogEntry {
        id: 21,
        name: "str_cat",
        meaning: "concatenate interned strings",
        stack_in: 0,
        stack_out: 2,
        purity: false,
    }
}


pub fn catalog_22_closure_new() -> CatalogEntry {
    CatalogEntry {
        id: 22,
        name: "closure_new",
        meaning: "allocate closure",
        stack_in: 1,
        stack_out: 1,
        purity: true,
    }
}


pub fn catalog_23_closure_call() -> CatalogEntry {
    CatalogEntry {
        id: 23,
        name: "closure_call",
        meaning: "invoke closure",
        stack_in: 2,
        stack_out: 2,
        purity: false,
    }
}


pub fn catalog_24_iter_new() -> CatalogEntry {
    CatalogEntry {
        id: 24,
        name: "iter_new",
        meaning: "create iterator",
        stack_in: 0,
        stack_out: 1,
        purity: true,
    }
}


pub fn catalog_25_iter_next() -> CatalogEntry {
    CatalogEntry {
        id: 25,
        name: "iter_next",
        meaning: "advance iterator",
        stack_in: 1,
        stack_out: 2,
        purity: false,
    }
}


pub fn catalog_26_match_tag() -> CatalogEntry {
    CatalogEntry {
        id: 26,
        name: "match_tag",
        meaning: "sum-type tag match",
        stack_in: 2,
        stack_out: 1,
        purity: true,
    }
}


pub fn catalog_27_yield_val() -> CatalogEntry {
    CatalogEntry {
        id: 27,
        name: "yield_val",
        meaning: "generator yield",
        stack_in: 0,
        stack_out: 2,
        purity: false,
    }
}


pub fn catalog_28_resume_gen() -> CatalogEntry {
    CatalogEntry {
        id: 28,
        name: "resume_gen",
        meaning: "resume generator",
        stack_in: 1,
        stack_out: 1,
        purity: true,
    }
}


pub fn catalog_29_atomic_cas() -> CatalogEntry {
    CatalogEntry {
        id: 29,
        name: "atomic_cas",
        meaning: "compare-and-swap stub",
        stack_in: 2,
        stack_out: 2,
        purity: false,
    }
}

pub fn all_entries() -> Vec<CatalogEntry> {
    vec![
        catalog_00_arithmetic_add(),
        catalog_01_arithmetic_sub(),
        catalog_02_arithmetic_mul(),
        catalog_03_arithmetic_div(),
        catalog_04_compare_eq(),
        catalog_05_compare_lt(),
        catalog_06_compare_le(),
        catalog_07_branch_if(),
        catalog_08_branch_if_not(),
        catalog_09_loop_header(),
        catalog_10_phi_merge(),
        catalog_11_spill_slot(),
        catalog_12_reload_slot(),
        catalog_13_guard_null(),
        catalog_14_guard_type(),
        catalog_15_box_int(),
        catalog_16_unbox_int(),
        catalog_17_array_len(),
        catalog_18_array_load(),
        catalog_19_array_store(),
        catalog_20_str_len(),
        catalog_21_str_cat(),
        catalog_22_closure_new(),
        catalog_23_closure_call(),
        catalog_24_iter_new(),
        catalog_25_iter_next(),
        catalog_26_match_tag(),
        catalog_27_yield_val(),
        catalog_28_resume_gen(),
        catalog_29_atomic_cas()
    ]
}

pub fn render_catalog() -> String {
    let mut s = String::new();
    for e in all_entries() {
        s.push_str(&format!(
            "{:02} {:<16} in={} out={} pure={} — {}\n",
            e.id, e.name, e.stack_in, e.stack_out, e.purity, e.meaning
        ));
    }
    s
}

pub fn lookup_name(name: &str) -> Option<CatalogEntry> {
    all_entries().into_iter().find(|e| e.name == name)
}
