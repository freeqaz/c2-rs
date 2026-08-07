#!/usr/bin/env python3
"""add_980_tests.py — pin board #980's boundary in `crates/c2-core/src/splice.rs`.

A `Reduction::NoEffectCall` row is mechanism **E**'s and never the splice's, and
the row must still be **visible** to `mentions()`. Both halves matter and only
one of them is obvious, so both are asserted.
"""

import sys

TESTS = '''    /// **BOARD #980's BOUNDARY — a `NoEffectCall` row is E's and NEVER the
    /// splice's.**
    ///
    /// Lane `w-inl0` feeds mechanism E edges from rows the IL parser **refused**:
    /// a refused body whose grammar still proves it emits nothing but a call to
    /// one callee contributes `Reduction::NoEffectCall`. E can close through
    /// such a node because closing needs only *"does this emit anything"*.
    ///
    /// The splice cannot, and the reason is S6 rather than a policy: its rule is
    /// *"the caller's body IS the callee's body"*, and there is no body — the
    /// parser refused it. So [`TuContext::definition`] returns `None` for those
    /// rows and the walk refuses with `S6-callee-parse-refused`.
    ///
    /// **But the row must still be VISIBLE.** `?g` is a name this TU defines,
    /// and `mentions` says so, which is what `S6-chain-truncated` reads to tell
    /// a chain that *ended* from one the port could not *follow*. A resolution
    /// that dropped refused rows from the context would make `?g` read as an
    /// external — and running a splice off the end of a chain the port cannot
    /// see is the wrong-relocation defect this lane already closed once.
    #[test]
    fn a_no_effect_call_row_feeds_e_and_never_the_splice() {
        let mut h = leaf("?h@@YAXXZ");
        h.ops = Vec::new();
        h.params = Vec::new();
        h.empty_body = true;
        let mut caller = tail("?f@@YAXXZ", "?g@@YAXXZ");
        caller.params = Vec::new();
        // `?g` is REFUSED by the parser — there is no `IlFunction` for it at
        // all — and carries only the edge board #980 reads out of its grammar.
        let tu = TuContext::of_rows(vec![
            ("?h@@YAXXZ", Some(Reduction::Parsed(&h)), None),
            ("?g@@YAXXZ", Some(Reduction::NoEffectCall("?h@@YAXXZ")), None),
            ("?f@@YAXXZ", Some(Reduction::Parsed(&caller)), None),
        ]);

        // E closes through the refused node — that is #980's whole content.
        assert!(
            tu.reduces_to_nothing("?g@@YAXXZ"),
            "BOARD #980 REGRESSED: mechanism E must close through a refused row \\
             that carries a NoEffectCall edge"
        );

        // The splice must not be able to reach it as a body...
        assert!(
            tu.definition("?g@@YAXXZ").is_none(),
            "A REFUSED BODY WAS OFFERED TO THE SPLICE: S6 needs a COMPOSED body \\
             for the chain's end and the parser refused this one"
        );
        // ...and must still SEE it, or `S6-chain-truncated` goes blind.
        assert!(
            tu.mentions("?g@@YAXXZ"),
            "A REFUSED ROW VANISHED FROM THE CONTEXT: it would read as an \\
             external, S6-chain-truncated would stop firing, and the splice \\
             would run off the end of a chain it cannot see"
        );

        // And the two mechanisms still do not both claim `?f`: E takes it.
        let sel = select_function(&caller, OptMode::O1).unwrap();
        assert!(
            crate::elide::drops_tail_call(&caller, tu.empty_callees()),
            "?f tail-calls a name that reduces to nothing, so E answers"
        );
        assert_eq!(
            splice_callee_why(&caller, &sel, &tu),
            Err("S9-mechanism-e"),
            "S9: mechanism E is asked first and keeps its blr"
        );
    }

    /// A refused row with **no** readable `no_effect_callee` feeds neither
    /// mechanism and is still visible. That is the third value of the
    /// constructor's `Option<Reduction>`, and it is the majority of the refused
    /// population.
    #[test]
    fn a_refused_row_with_no_edge_is_visible_and_nothing_else() {
        let mut h = leaf("?h@@YAXXZ");
        h.ops = Vec::new();
        h.params = Vec::new();
        h.empty_body = true;
        let tu = TuContext::of_rows(vec![
            ("?h@@YAXXZ", Some(Reduction::Parsed(&h)), None),
            ("?g@@YAXXZ", None, None),
        ]);
        assert!(tu.mentions("?g@@YAXXZ"), "still a name this TU defines");
        assert!(tu.definition("?g@@YAXXZ").is_none());
        assert!(
            !tu.reduces_to_nothing("?g@@YAXXZ"),
            "#980 IS CONSERVATIVE: a refused row with nothing readable \\
             contributes NO edge to the closure"
        );
    }

'''

ANCHOR = "    /// **S2** — `t08`. Two call sites is SPLICE-N, **0 of 548**."

p = "crates/c2-core/src/splice.rs"
s = open(p).read()
if "a_no_effect_call_row_feeds_e_and_never_the_splice" in s:
    sys.exit("already added")
if ANCHOR not in s:
    sys.exit("anchor moved")
open(p, "w").write(s.replace(ANCHOR, TESTS + ANCHOR, 1))
print("added 2 tests")
