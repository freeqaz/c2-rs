#!/usr/bin/env python3
"""addtests.py — append board #1053's unit-test block to `no_effect.rs`'s test
module. A one-shot editor, kept only so the edit is reproducible.
"""
BLOCK = r'''
    // =====================================================================
    // BOARD #1053 — THE SEED. `no_effect_nothing`, and the guards no `.cpp`
    // can reach.
    //
    // GRID-N's eleven cells grade the RULE against real c2. What they cannot
    // reach is a mistyped literal, a truncated statement or a spliced-in
    // second one: every source-level perturbation changes the statement
    // sequence first and the walk refuses there, so the census key moves and
    // the cell stops being about the guard. Those are graded on the bytes of
    // a pinned live capture or not at all — `w-memset` §4.1's finding, one
    // reader over.
    // =====================================================================

    /// `??$da@US@@@@YAXPAUS@@@Z` — the whole `.ex` segment, transcribed
    /// **verbatim** out of a live capture of GRID-N's `n01` at the workload's own
    /// flags (`work/w-seed/extractseg.py` on the bundle `c2rs census --keep-il`
    /// kept), not hand-assembled.
    ///
    /// It is `p->~T()` on a class with a trivial destructor — STLport's
    /// `__destroy_aux(_pointer, __false_type)` with the names shortened — and c2's
    /// whole `.text` COMDAT for it is `4e800020` with no relocation.
    ///
    /// **It is the workload's production and not a cell's dialect** (#953, which
    /// says that need not hold): the workload's own
    /// `??$__destroy_aux@V?$Key@M@@@stlpmtx_std@@...` on
    /// `src/lazer/meta_ham/CharacterProvider.cpp` reads
    ///
    /// ```text
    ///   4c 4f 11 53 4f 01 36 . 33 86 41 74 00 . 33 82 07 03 00 . 44 . 4b . 4f 01 38 . 3a ...
    /// ```
    ///
    /// and this segment reads the same, modulo the two `4F 01 <line>` markers a
    /// one-line cell does not carry — the difference `no_effect_call` already
    /// measured and eats.
    const NOTHING_BODY: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0x20, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x0E, 0x53, 0x53, 0x26, 0xF6, 0x09,
        0x46, 0x2D, 0xF4, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x33, 0x86, 0x41, 0x74, 0x00, 0x33, 0x82,
        0x07, 0x03, 0x00, 0x44, 0x4B, 0x3A, 0x08, 0x0A, 0x54, 0x02, 0x29, 0x08, 0x0A, 0x4F, 0x12,
        0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// The whole statement, as one run — every mutation below is expressed
    /// relative to a pattern it has FOUND, never to an index it counted.
    const NOTHING_STMT: &[u8] = &[
        0x33, 0x86, 0x41, 0x74, 0x00, 0x33, 0x82, 0x07, 0x03, 0x00, 0x44, 0x4B,
    ];

    fn at_nothing(pat: &[u8]) -> usize {
        crate::func::readers::find_subslice(NOTHING_BODY, pat)
            .unwrap_or_else(|| panic!("pattern {pat:02x?} is not in the pinned segment"))
    }

    /// The positive, and the whole rung in one line.
    #[test]
    fn the_pseudo_destructor_body_emits_nothing() {
        assert!(no_effect_nothing(NOTHING_BODY));
    }

    /// **Recognizing it does not ACCEPT it** — #971 condition 4, the containment
    /// every reader in this module is under. The census key is unchanged too,
    /// because a widening that moved it would be a widening of the parser.
    #[test]
    fn recognizing_the_nothing_body_does_not_accept_it() {
        use crate::func::test_fixtures::NO_LOCALS;
        assert!(crate::func::body::parse_segment(NOTHING_BODY, NO_LOCALS).is_none());
        let b = crate::func::body::parse_segment_detail(NOTHING_BODY, NO_LOCALS).unwrap_err();
        assert_eq!(b.feature(), "expr-lit-type-8207");
    }

    /// **THE TOTALITY TERMINAL** — mutation **M1**'s target.
    ///
    /// The walk must reach the end of the segment. Without that, "there is nothing
    /// else in this body" is a search over the part that was walked rather than a
    /// property of the whole, and a seed asserts the whole.
    #[test]
    fn trailing_bytes_after_the_nothing_statement_are_refused() {
        let mut seg = NOTHING_BODY.to_vec();
        seg.push(0x26);
        seg.push(0x11);
        assert!(!no_effect_nothing(&seg));
    }

    /// **A SECOND STATEMENT is refused**, spliced in at the byte level so the
    /// statement sequence is the only thing that changed. GRID-N's `n10` is the
    /// source-reachable half; this is the one that pins the walk.
    #[test]
    fn a_second_nothing_statement_is_refused() {
        let mut seg = NOTHING_BODY.to_vec();
        let k = at_nothing(NOTHING_STMT);
        seg.splice(k..k, NOTHING_STMT.iter().copied());
        assert!(!no_effect_nothing(&seg));
    }

    /// **THE DISCARD IS REQUIRED.** `4B` is what says the statement's value is
    /// thrown away; without it the walk would be reading a fragment of a larger
    /// expression as a whole body.
    #[test]
    fn a_nothing_statement_whose_value_is_consumed_is_refused() {
        let mut seg = NOTHING_BODY.to_vec();
        seg[at_nothing(&[0x44, 0x4B]) + 1] = 0x41;
        assert!(!no_effect_nothing(&seg));
    }

    /// **THE BIND IS REQUIRED.** `44` is consumed as a byte, not as an operator
    /// whose arity this module claims to know — but it is consumed, so a statement
    /// without it is a statement this reader has not seen.
    #[test]
    fn a_nothing_statement_without_the_bind_is_refused() {
        let mut seg = NOTHING_BODY.to_vec();
        let k = at_nothing(&[0x44, 0x4B]);
        seg.remove(k);
        assert!(!no_effect_nothing(&seg));
    }

    /// **THE LITERAL TYPES ARE PINNED**, and this is the soundness half rather
    /// than a fit. A `float`/`double` literal drags `_fltused` into the TU and the
    /// obj **grows a symbol** — `CallRet::discarded`'s reason one operand over — so
    /// the two types the capture carries are the only two admitted.
    ///
    /// `86 41 12` is `long`, which c2 treats identically to `int` for every
    /// operator this crate accepts; it is refused here anyway, because the pin is
    /// the exact [`INT_TYPE`] triple and widening it is a decision with a cell
    /// behind it, not a convenience.
    #[test]
    fn a_nothing_literal_of_another_type_is_refused() {
        for wrong in [
            [0x86u8, 0x41, 0x12], // long
            [0x86, 0x42, 0x75],   // unsigned
            [0x88, 0x85, 0x41],   // double — the one that would grow a symbol
        ] {
            let mut seg = NOTHING_BODY.to_vec();
            let k = at_nothing(&[0x33, 0x86, 0x41, 0x74, 0x00]);
            seg.splice(k + 1..k + 4, wrong);
            assert!(
                !no_effect_nothing(&seg),
                "a literal typed {wrong:02x?} was read as the int operand"
            );
        }
        // …and the VOID operand's tag/kind pair, one bit over.
        let mut seg = NOTHING_BODY.to_vec();
        seg[at_nothing(&[0x33, 0x82, 0x07, 0x03, 0x00]) + 2] = 0x08;
        assert!(!no_effect_nothing(&seg));
    }

    /// **THE LITERAL VALUES ARE NOT CONSTRAINED**, and that is #644's rule applied
    /// rather than an oversight: a literal is pure whatever its value and the
    /// statement is discarded, so the value cannot change what is emitted.
    /// Constraining a field that happens to be constant on one corpus is exactly
    /// the mistake `no_effect_call`'s align/count/fill avoid.
    ///
    /// The **void type's id** is not constrained either — it is a per-TU type-table
    /// index, and pinning it would make this reader a property of one bundle.
    #[test]
    fn the_nothing_values_and_the_void_type_id_are_not_constrained() {
        let mut seg = NOTHING_BODY.to_vec();
        seg[at_nothing(&[0x33, 0x86, 0x41, 0x74, 0x00]) + 4] = 0x07; // int value
        assert!(no_effect_nothing(&seg));

        let mut seg = NOTHING_BODY.to_vec();
        let k = at_nothing(&[0x33, 0x82, 0x07, 0x03, 0x00]);
        seg[k + 3] = 0x09; // the void TYPE's id
        seg[k + 4] = 0x7F; // the void literal's value
        assert!(no_effect_nothing(&seg));
    }

    /// **THE FORMAL-LOAD VARIANT IS DECLINED, and it is the residue this lane
    /// publishes rather than takes.**
    ///
    /// The workload carries a second body under the same census key: where a class
    /// element type with a trivial destructor folds `p` away to a literal, an
    /// **enum** element type keeps it, and
    /// `??$__destroy_aux@W4CubeFace@RndCubeTex@@...` on
    /// `src/system/rndobj/CubeTex.cpp` reads
    ///
    /// ```text
    ///   4c 4f 11 53 4f 01 36 . b9 <formal> 86 43 c9 50 . 33 82 07 03 00 . 44 . 4b . ...
    /// ```
    ///
    /// — a formal LOAD where this one has an int literal. It is very probably just
    /// as pure, and it is **not admitted**: GRID-N has no cell for it, and adding
    /// the arm now would be fitting a reader so that four more functions convert.
    /// It is worth exactly **4** `fnbyte-differs`
    /// (`fnbyte-blr-stop3-expr-lit-type-8207` at the tip) and it is board **#1090**.
    #[test]
    fn a_formal_load_in_place_of_the_int_literal_is_declined() {
        let mut seg = NOTHING_BODY.to_vec();
        let k = at_nothing(&[0x33, 0x86, 0x41, 0x74, 0x00]);
        // `B9 <formal-tok> <TYPE>` — the shape CubeTex.cpp carries, using this
        // segment's own formal token so nothing but the operand form changed.
        seg.splice(k..k + 5, [0xB9, 0xF4, 0x09, 0x86, 0x43, 0xC9, 0x50]);
        assert!(
            !no_effect_nothing(&seg),
            "the formal-load variant was admitted — it is a SECOND production and \
             GRID-N graded no cell for it (board #1090)"
        );
    }

    /// **A DEEPER SCOPE IS REFUSED.** A body that opens a block is a body with a
    /// statement this reader has not walked, and the walk would read the block's
    /// first statement as the whole body.
    #[test]
    fn a_nothing_body_that_opens_a_deeper_scope_is_refused() {
        let mut seg = NOTHING_BODY.to_vec();
        let k = at_nothing(&[0x4C, 0x4F, 0x11, 0x53]);
        seg.insert(k + 4, 0x53);
        assert!(!no_effect_nothing(&seg));
    }
}
'''

P = "crates/c2-il/src/func/body/shapes/no_effect.rs"
s = open(P).read().rstrip()
assert s.endswith("}")
s = s[: s.rfind("}")] + BLOCK.lstrip("\n")
open(P, "w").write(s)
print("ok")
