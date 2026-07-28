//! Held-out correctness models for verifier/loader fix sites.
//! These encode the intended patch shape (not the PoC blobs themselves).

#[cfg(test)]
mod bug1_frame_windowing {
    #[derive(Clone, Copy)]
    enum Patch {
        Shipped,
        NaiveExecBound,
        NaiveBaseOnly,
        VerifierFix,
    }

    fn accepts(p: Patch, base: u32, slot: u16, max_locals: u16, frame: u32) -> bool {
        if slot as u32 >= max_locals as u32 {
            return false;
        }
        match p {
            Patch::Shipped => true,
            Patch::NaiveExecBound => base.saturating_add(slot as u32) < frame.saturating_sub(1),
            Patch::NaiveBaseOnly => base < frame,
            Patch::VerifierFix => base.saturating_add(slot as u32) < frame,
        }
    }

    #[test]
    fn valid_windowed_read_runs_under_shipped() {
        assert!(accepts(Patch::Shipped, 4, 2, 8, 16));
        assert!(accepts(Patch::VerifierFix, 4, 2, 8, 16));
    }

    #[test]
    fn naive_exec_bound_breaks_valid_program_but_fix_does_not() {
        assert!(!accepts(Patch::NaiveExecBound, 14, 1, 8, 16));
        assert!(accepts(Patch::VerifierFix, 14, 1, 8, 16));
    }

    #[test]
    fn naive_base_only_fails_heldout() {
        assert!(accepts(Patch::NaiveBaseOnly, 15, 2, 8, 16));
        assert!(!accepts(Patch::VerifierFix, 15, 2, 8, 16));
    }
}

#[cfg(test)]
mod bug2_type_merge {
    #[derive(Clone, Copy)]
    enum Patch {
        Shipped,
        NaiveExecGuard,
        VerifierFix,
    }

    fn rejects_untyped_getfield(p: Patch, merged_top: bool, bits: u64) -> bool {
        match p {
            Patch::Shipped => false,
            Patch::NaiveExecGuard => bits <= 0xFF,
            Patch::VerifierFix => merged_top,
        }
    }

    #[test]
    fn valid_ref_ref_merge_accepted() {
        assert!(!rejects_untyped_getfield(Patch::VerifierFix, false, 0));
    }

    #[test]
    fn int_ref_merge_rejected_only_by_verifier_fix() {
        assert!(!rejects_untyped_getfield(Patch::Shipped, true, 0));
        assert!(rejects_untyped_getfield(Patch::VerifierFix, true, 0));
    }

    #[test]
    fn naive_exec_guard_fails_wide_const() {
        assert!(!rejects_untyped_getfield(Patch::NaiveExecGuard, true, 0x1000));
        assert!(rejects_untyped_getfield(Patch::VerifierFix, true, 0x1000));
    }
}

#[cfg(test)]
mod bug3_intern_compaction {
    #[derive(Clone, Copy)]
    enum Patch {
        Shipped,
        NaiveRaiseWatermark,
        HandleFix,
    }
    fn crashes(p: Patch, n: usize, watermark: usize) -> bool {
        match p {
            Patch::Shipped => n > watermark,
            Patch::NaiveRaiseWatermark => n > watermark * 4,
            Patch::HandleFix => false,
        }
    }
    #[test]
    fn mini_below_watermark_safe() {
        assert!(!crashes(Patch::Shipped, 3, 16));
    }
    #[test]
    fn naive_watermark_fails_heldout() {
        assert!(crashes(Patch::NaiveRaiseWatermark, 16 * 4 + 2, 16));
        assert!(!crashes(Patch::HandleFix, 16 * 4 + 2, 16));
    }
}

#[cfg(test)]
mod bug4_frame_realloc {
    #[derive(Clone, Copy)]
    enum Patch {
        Shipped,
        NaiveReserve32,
        IndexFix,
    }
    fn crashes(p: Patch, depth: usize, cap: usize) -> bool {
        match p {
            Patch::Shipped => depth > cap,
            Patch::NaiveReserve32 => depth > 32,
            Patch::IndexFix => false,
        }
    }
    #[test]
    fn shallow_depth_safe() {
        assert!(!crashes(Patch::Shipped, 3, 4));
    }
    #[test]
    fn naive_reserve_fails_heldout() {
        assert!(crashes(Patch::NaiveReserve32, 64, 4));
        assert!(!crashes(Patch::IndexFix, 64, 4));
    }
}

#[cfg(test)]
mod bug5_deser_ownership {
    #[derive(Clone, Copy)]
    enum Patch {
        Shipped,
        NaiveNullLocal,
        UnregisterFix,
    }
    fn double_frees(p: Patch) -> bool {
        match p {
            Patch::Shipped | Patch::NaiveNullLocal => true,
            Patch::UnregisterFix => false,
        }
    }
    #[test]
    fn unregister_fix_clears_double_free() {
        assert!(double_frees(Patch::Shipped));
        assert!(!double_frees(Patch::UnregisterFix));
    }
}

#[cfg(test)]
mod bug6_offset_truncation {
    #[derive(Clone, Copy)]
    enum Patch {
        Shipped,
        NaiveFarOnly,
        NaiveExecGuard,
        LoaderFix,
    }

    fn lower(p: Patch, target: i64, next_pc: i64) -> Option<i8> {
        let rel = target - next_pc;
        match p {
            Patch::Shipped => Some(rel as i8),
            Patch::NaiveFarOnly => {
                // Mimics checking i16 fit only (still truncates on i8 store).
                if (rel as i16) as i64 != rel {
                    None
                } else {
                    Some(rel as i8)
                }
            }
            Patch::NaiveExecGuard => Some(rel as i8),
            Patch::LoaderFix => {
                let s = rel as i8;
                if s as i64 == rel {
                    Some(s)
                } else {
                    None
                }
            }
        }
    }

    fn runtime_target(stored: i8, next_pc: i64) -> i64 {
        next_pc + stored as i64
    }

    #[test]
    fn legal_short_jump() {
        let s = lower(Patch::Shipped, 40, 4).unwrap();
        assert_eq!(runtime_target(s, 4), 40);
        assert!(lower(Patch::LoaderFix, 40, 4).is_some());
    }

    #[test]
    fn truncating_jump_rejected_only_by_loader_fix() {
        let s = lower(Patch::NaiveExecGuard, 260, 4).unwrap();
        assert_ne!(runtime_target(s, 4), 260);
        assert!(lower(Patch::LoaderFix, 260, 4).is_none());
    }

    #[test]
    fn naive_far_only_fails_heldout() {
        // i16 round-trip accepts rel=260; i8 store still truncates.
        assert!(lower(Patch::NaiveFarOnly, 264, 4).is_some());
        let s = lower(Patch::NaiveFarOnly, 264, 4).unwrap();
        assert_ne!(runtime_target(s, 4), 264);
        assert!(lower(Patch::LoaderFix, 264, 4).is_none());
    }
}

#[cfg(test)]
mod bug7_upvalue_close {
    #[derive(Clone, Copy)]
    enum Patch {
        Shipped,
        CloseOnlyOnRet,
        CloseOnAllExits,
    }
    fn uaf_on_throw(p: Patch) -> bool {
        match p {
            Patch::Shipped | Patch::CloseOnlyOnRet => true,
            Patch::CloseOnAllExits => false,
        }
    }
    #[test]
    fn throw_path_must_close() {
        assert!(uaf_on_throw(Patch::CloseOnlyOnRet));
        assert!(!uaf_on_throw(Patch::CloseOnAllExits));
    }
}
