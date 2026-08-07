#!/usr/bin/env python3
"""fix_980_tests.py — the #980 boundary tests, after the `len` -> `definitions`
rename landed later in the rebase, plus a pin for the trap that caused it.

Both tests were written at the commit where `TuContext` still spelled its row
count `len`. The rename landed further down the rebase, so `tu.len()` now
resolves through `Deref` to the **E-context** size: one test failed outright and
the other **passed by coincidence**, because its cell's E context happens to
have as many members as the table has rows.

So: assert the row count by its own name (`definitions()`), assert `mentions()`
directly now that it exists, and add a third test that asserts the two counts
are **different** on a bundle where they must be — which is a thing no future
rename can quietly undo.
"""

import sys

P = "crates/c2-core/src/splice.rs"

OLD1 = '''        assert_eq!(
            tu.len(),
            3,
            "A REFUSED ROW VANISHED FROM THE CONTEXT: it would later read as an \\
             external, S6-chain-truncated would stop firing, and the splice \\
             would run off the end of a chain it cannot see"
        );'''
NEW1 = '''        assert!(
            tu.mentions("?g@@YAXXZ"),
            "A REFUSED ROW VANISHED FROM THE CONTEXT: it reads as an external, \\
             S6-chain-truncated stops firing, and the splice runs off the end \\
             of a chain it cannot see"
        );
        assert_eq!(
            tu.definitions(),
            3,
            "every row this TU binds stays in the table, parsed or not"
        );'''

OLD2 = '''        assert_eq!(tu.len(), 2, "still a name this TU defines");
        assert!(tu.definition("?g@@YAXXZ").is_none());'''
NEW2 = '''        assert!(tu.mentions("?g@@YAXXZ"), "still a name this TU defines");
        assert_eq!(tu.definitions(), 2, "and still a row in the table");
        assert!(tu.definition("?g@@YAXXZ").is_none());'''

ANCHOR = '''    /// **S2** — `t08`. Two call sites is SPLICE-N, **0 of 548**.'''

PIN = '''    /// **THE `Deref` SHADOWING TRAP, pinned.**
    ///
    /// `TuContext` derefs to [`TuEmptyCallees`] so existing callers keep
    /// working, and an inherent method on the wrapper therefore **silently
    /// overrides** the target's. While this type spelled its row count `len`,
    /// the scan's `fnbyte-tu-empty-callees` reported the wrong quantity —
    /// 88,894 against 1,474,755 on the dc3 workload — with no compile error and
    /// no test failure. It also made one of the two tests above pass by
    /// coincidence, because that cell's E context happens to have as many
    /// members as the table has rows.
    ///
    /// So the two counts are asserted to be **different** on a bundle where
    /// they must be, which is a thing no rename can quietly undo.
    #[test]
    fn the_row_count_and_the_e_context_are_not_the_same_number() {
        let mut h = leaf("?h@@YAXXZ");
        h.ops = Vec::new();
        h.params = Vec::new();
        h.empty_body = true;
        let g = leaf("?g@@YAHH@Z"); // parses, non-empty: a row, never in E
        let tu = TuContext::of_rows(vec![
            ("?h@@YAXXZ", Some(Reduction::Parsed(&h)), None),
            ("?g@@YAHH@Z", Some(Reduction::Parsed(&g)), None),
            ("?x@@YAXXZ", None, None), // defined here, parser refused it
        ]);
        assert_eq!(tu.definitions(), 3, "three rows this TU binds");
        assert_eq!(tu.empty_callees().len(), 1, "only ?h reduces to nothing");
        assert_ne!(
            tu.definitions(),
            tu.empty_callees().len(),
            "A ROW COUNT WAS READ AS THE E-CONTEXT SIZE (or the reverse): these \\
             are different facts, `TuContext` Derefs to `TuEmptyCallees`, and an \\
             inherent `len` here would shadow the target's and move an existing \\
             scan key by 16x without failing anything"
        );
    }

'''


def main():
    s = open(P).read()
    if "the_row_count_and_the_e_context_are_not_the_same_number" in s:
        sys.exit("already applied")
    for old, new in ((OLD1, NEW1), (OLD2, NEW2)):
        if old not in s:
            sys.exit("MISSING: %r" % old[:70])
        s = s.replace(old, new, 1)
    if ANCHOR not in s:
        sys.exit("anchor moved")
    s = s.replace(ANCHOR, PIN + ANCHOR, 1)
    open(P, "w").write(s)
    print("patched both tests and added the shadowing pin")


if __name__ == "__main__":
    main()
