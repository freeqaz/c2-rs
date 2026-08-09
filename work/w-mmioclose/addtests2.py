#!/usr/bin/env python3
"""One-shot: append this lane's consumer cells to `comdat.rs` and `splice.rs`."""

COMDAT = r'''
    /// **N7 — `__declspec(noinline)`, the clause this lane added.**
    ///
    /// Byte for byte cell `P`'s TU, with one bit of the callee's `.gl`
    /// attribute cleared. c2 does **not** expand a `noinline` callee, so the
    /// port's `bl` is right and the fence must not fire — and the pair is what
    /// makes that a measurement rather than a claim, because the ONLY
    /// difference between the two cells is `IlFunction::inlinable`.
    ///
    /// The shape is `mmio.cpp`'s own: `mmioClose` calls `mmioFlush`, defined in
    /// that TU, eight bytes long, `__declspec(noinline)` — and the reference obj
    /// keeps the `bl`. `work/w-mmioclose/probe/inl.cpp` is the control from the
    /// other side: eight cells of the same shape WITHOUT the attribute and c2
    /// expands seven of them, so size does not separate this pair.
    #[test]
    fn n7_a_noinline_same_tu_callee_is_untouched() {
        let mut g = leaf("?g@@YAHH@Z");
        g.inlinable = Some(false);
        let funcs = vec![g, caller_with_setup("?f@@YAHH@Z", "?g@@YAHH@Z")];
        let body = compose(&funcs, 1).expect("c2 keeps the call, so the port may emit it");
        assert_eq!(
            body.calls.len(),
            1,
            "the REL24 must SURVIVE — a cell that passes because the body has \
             no call at all tests nothing"
        );
        assert_eq!(body.calls[0].callee, "?g@@YAHH@Z");
    }

    /// **N7's must-fail mutation, and it is the pair `P` cannot supply on its
    /// own.** `Some(true)` and `None` are the two values that are NOT the
    /// attribute, and both must land exactly where the fence has always put
    /// them. A clause written `!= Some(true)` would pass N7 and fail here.
    #[test]
    fn n7_only_some_false_moves_the_fence() {
        for flag in [None, Some(true)] {
            let mut g = leaf("?g@@YAHH@Z");
            g.inlinable = flag;
            let funcs = vec![g, caller_with_setup("?f@@YAHH@Z", "?g@@YAHH@Z")];
            assert!(
                is_fenced(&compose(&funcs, 1)),
                "inlinable = {flag:?} must leave the fence exactly where it was: \
                 None is UNASKED and Some(true) is a positive permission, and \
                 neither is `__declspec(noinline)`"
            );
        }
    }
'''

SPLICE = r'''
    /// **`S7-callee-noinline` — the shipped wrong emit, closed.**
    ///
    /// `crates/c2-harness/tests/noinline_boundary.rs` cell `w10` is
    /// `__declspec(noinline) int g(int a){return a+1;} int f(int a){return g(a);}`
    /// and it records what the port does today: the splice puts `?g`'s body into
    /// `?f` where c2 emits `b ?g`. That file's note says the port *"cannot read
    /// the attribute"*; `c2_il::func::gl::FN_FLAG_INLINABLE` is the attribute,
    /// and this is the refusal it buys.
    ///
    /// The control is the same TU with the flag left `None`, which still
    /// splices — so the cell measures the attribute and not the shape.
    #[test]
    fn a_noinline_callee_is_not_spliced() {
        let g = leaf("?g@@YAHH@Z");
        let caller = tail_caller("?f@@YAHH@Z", "?g@@YAHH@Z");
        let sel = select_function(&caller, OptMode::O1).expect("the caller lowers");

        let tu_ok = TuContext::of_rows(vec![("?g@@YAHH@Z", Some(Reduction::Parsed(&g)), None)]);
        assert!(
            splice_body_why(&caller, &sel, OptMode::O1, &tu_ok).is_ok(),
            "the CONTROL must splice, or the cell below is measuring the shape"
        );

        let mut g_ni = leaf("?g@@YAHH@Z");
        g_ni.inlinable = Some(false);
        let tu_ni =
            TuContext::of_rows(vec![("?g@@YAHH@Z", Some(Reduction::Parsed(&g_ni)), None)]);
        assert_eq!(
            splice_body_why(&caller, &sel, OptMode::O1, &tu_ni).err(),
            Some(SpliceDecline::Refused("S7-callee-noinline")),
            "c2 emits `b ?g` here; the port must not emit ?g's body"
        );

        // `Some(true)` is a positive permission and `None` is UNASKED, and
        // neither may move the splice — the must-fail half of the pair.
        for flag in [None, Some(true)] {
            let mut g2 = leaf("?g@@YAHH@Z");
            g2.inlinable = flag;
            let tu = TuContext::of_rows(vec![("?g@@YAHH@Z", Some(Reduction::Parsed(&g2)), None)]);
            assert!(
                splice_body_why(&caller, &sel, OptMode::O1, &tu).is_ok(),
                "inlinable = {flag:?} must leave the splice exactly where it was"
            );
        }
    }
'''


def append_to_test_mod(path, text):
    s = open(path).read().rstrip()
    assert s.endswith('}'), path
    s = s[: s.rfind('}')] + text + '}\n'
    open(path, 'w').write(s)


append_to_test_mod('crates/c2-core/src/comdat.rs', COMDAT)
append_to_test_mod('crates/c2-core/src/splice.rs', SPLICE)
print('done')
