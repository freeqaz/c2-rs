#!/usr/bin/env python3
"""Append W-PARK's unit tests to `shapes::calls`' own `mod tests`."""
TESTS = '''
    /// **W-PARK — [`ArgSite`] is the ONE axis the literal path's in-place
    /// requirement depends on, and both of its values are asserted here.**
    ///
    /// Board **#1920**. The clause is the whole of this lane's `crates/` change,
    /// so it gets a test that names both sides rather than one that names the
    /// side that ships. `[Formal(1), Formal(0), Lit(72)]` is `mmioGetInfo`'s own
    /// slot list.
    #[test]
    fn arg_site_decides_the_literal_paths_permutation_clause() {
        let slots = vec![SlotArg::Formal(1), SlotArg::Formal(0), SlotArg::Lit(72)];
        let seg: &[u8] = &[0u8; 8];
        assert!(
            lit_arg_tail_call(seg, 0, vec![1, 2, 3], slots.clone(), 9, ArgSite::Tail)
                .is_err(),
            "at the TAIL site there is no park, so a permuted literal list must \\
             stay refused — the historical behaviour, unchanged"
        );
        assert!(
            matches!(
                lit_arg_tail_call(seg, 0, vec![1, 2, 3], slots, 9, ArgSite::Sequence),
                Ok(BodyShape::MultiArgTailCall { .. })
            ),
            "at the SEQUENCE site the permutation is decided downstream by \\
             park_in_class, on the same slot_sources view"
        );
    }

    /// The in-place list is accepted at **both** sites, so the widening is a
    /// strictly larger accept set at one site and a no-op at the other. This is
    /// the algebraic half of the verdict-neutrality claim; the measured half is
    /// GRID-P's 45 cells and the 878-TU set comparison.
    #[test]
    fn arg_site_does_not_change_the_in_place_literal_list() {
        let slots = vec![SlotArg::Formal(0), SlotArg::Formal(1), SlotArg::Lit(72)];
        let seg: &[u8] = &[0u8; 8];
        for site in [ArgSite::Tail, ArgSite::Sequence] {
            assert!(
                matches!(
                    lit_arg_tail_call(seg, 0, vec![1, 2, 3], slots.clone(), 9, site),
                    Ok(BodyShape::MultiArgTailCall { .. })
                ),
                "the in-place list is unchanged at {site:?}"
            );
        }
    }

    /// **The clauses [`ArgSite::Sequence`] does NOT relax.** A widening that
    /// dropped the whole guard rather than its permutation half would take these
    /// with it, and neither has a park to legalise it: the eight-slot bound is a
    /// property of the argument registers, not of the call site.
    #[test]
    fn arg_site_sequence_still_refuses_the_slot_bound() {
        let slots: Vec<SlotArg> = (0..MAX_REGISTER_FORMALS)
            .map(SlotArg::Formal)
            .chain(std::iter::once(SlotArg::Lit(72)))
            .collect();
        let seg: &[u8] = &[0u8; 8];
        for site in [ArgSite::Tail, ArgSite::Sequence] {
            assert!(
                lit_arg_tail_call(seg, 0, vec![1], slots.clone(), 9, site).is_err(),
                "nine slots is over the bound at {site:?}"
            );
        }
    }
'''
p = "crates/c2-il/src/func/body/shapes/calls.rs"
s = open(p).read()
assert "arg_site_decides_the_literal_paths_permutation_clause" not in s
i = s.rstrip().rfind("\n}")
assert i > 0
open(p, "w").write(s.rstrip()[:i] + "\n" + TESTS + "}\n")
print("appended")
