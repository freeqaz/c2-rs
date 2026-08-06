#!/usr/bin/env python3
"""resolve_inl0.py — resolve the w-inl0 rebase in `splice.rs` + `fnbytes.rs`.

WHAT COLLIDED. Lane `w-inl0` (board #980) changed `tu_empty_callees` to feed
**parse-refused** rows into mechanism E's closure: a refused row whose `c2-il`
dead-temporary reader can read a `no_effect_callee` contributes one
`Reduction::NoEffectCall` edge. Lane `w-splice` changed the same function to
return a `TuContext` carrying each row's `opt_word`, so the splice can lower a
chain's end under the callee's own optimization mode.

THE RESOLUTION IS NOT "TAKE BOTH ITERATORS". `TuContext` has to serve three
questions and they do not have the same row set:

    E's closure      only rows that qualify under #980 — parsed, or refused
                     WITH a readable `no_effect_callee`. Every other refused
                     row contributes nothing. That is #980's conservative
                     direction and it is preserved exactly.
    `definition()`   only PARSED rows. A `NoEffectCall` row is a body the
                     parser refused, so the port has no bytes for it and S6
                     cannot compose a chain end out of it.
    `mentions()`     EVERY row with an `emit_name`, parsed or not, qualifying
                     or not.

The third is the one a naive resolution loses, and losing it is a wrong-bytes
emit rather than a missing count. `S6-chain-truncated` refuses a splice when the
chain's last link still names a callee **this TU carries**; if a refused row
without a readable `no_effect_callee` were dropped from the context entirely,
`mentions()` would go false for it, the clause would stop firing, and the splice
would run off the end of a chain it cannot see — which is exactly the 72
relocation disagreements this lane already had to close once.

So the constructor takes `(name, Option<Reduction>, opt_word)`: `None` means
"this TU defines the name and neither mechanism can use it", which is a row that
counts for `mentions` and for nothing else.
"""

import sys

SPLICE = "crates/c2-core/src/splice.rs"
FNBYTES = "crates/c2-harness/src/gap/fnbytes.rs"

OLD_ROWS = """    /// `(name, definition, the `.ex` optimization word)`, sorted by name.
    /// A name with more than one row is **refused** rather than resolved to the
    /// first — see [`TuContext::definition`].
    rows: Vec<(&'a str, &'a IlFunction, Option<u32>)>,"""

NEW_ROWS = """    /// `(name, definition, the `.ex` optimization word)`, sorted by name.
    /// A name with more than one row is **refused** rather than resolved to the
    /// first — see [`TuContext::definition`].
    ///
    /// The definition is `None` for a name this bundle **defines and whose IL
    /// the parser refused**. Those rows are kept, and keeping them is not
    /// bookkeeping: [`TuContext::mentions`] is what tells a chain that *ended*
    /// from one the port could not *follow*, and a refused row that vanished
    /// from this vector would read as an external. See the type's docs.
    rows: Vec<(&'a str, Option<&'a IlFunction>, Option<u32>)>,"""

OLD_CTOR = """    pub fn of_named(
        named: impl IntoIterator<Item = (&'a str, &'a IlFunction, Option<u32>)>,
    ) -> Self {
        let mut rows: Vec<(&'a str, &'a IlFunction, Option<u32>)> =
            named.into_iter().filter(|(n, _, _)| !n.is_empty()).collect();
        rows.sort_by_key(|(n, _, _)| *n);
        let empty = TuEmptyCallees::of_named(rows.iter().map(|(n, f, _)| (*n, *f)));
        Self { empty, rows }
    }"""

NEW_CTOR = """    pub fn of_named(
        named: impl IntoIterator<Item = (&'a str, &'a IlFunction, Option<u32>)>,
    ) -> Self {
        Self::of_rows(
            named
                .into_iter()
                .map(|(n, f, w)| (n, Some(Reduction::Parsed(f)), w)),
        )
    }

    /// **The general constructor** — one row per name this TU defines, carrying
    /// what each of the two mechanisms can make of it.
    ///
    /// # Three questions, three row sets, and only one of them is every row
    ///
    /// | asked by | which rows |
    /// |---|---|
    /// | mechanism **E**'s closure ([`TuEmptyCallees::of_rows`]) | the rows with a [`Reduction`] — parsed, or refused **with a readable `no_effect_callee`** (board **#980**). Every other refused row contributes nothing, which is that board's conservative direction and is preserved here exactly |
    /// | [`TuContext::definition`], i.e. the splice's S5/S6 | **parsed rows only**. A `NoEffectCall` row is a body the parser refused, so the port has no bytes for it and S6 cannot compose a chain end out of it |
    /// | [`TuContext::mentions`] | **every row**, parsed or not, qualifying or not |
    ///
    /// **The third is the one a careless merge loses.** `S6-chain-truncated`
    /// refuses a splice when the chain's last link still names a callee this TU
    /// carries; a refused row dropped from this vector would read as an
    /// *external*, the clause would stop firing, and the splice would run off
    /// the end of a chain it cannot see. That is not a missing count — it is
    /// the wrong-relocation defect (#1009's 72 witnesses) coming back.
    ///
    /// So `Option<Reduction>` is three-valued on purpose: `Some(Parsed)` is a
    /// body both mechanisms can use, `Some(NoEffectCall)` is one only E can use,
    /// and `None` is a name this TU defines that **neither** can use and that
    /// still has to be visible.
    pub fn of_rows(
        rows: impl IntoIterator<Item = (&'a str, Option<Reduction<'a>>, Option<u32>)>,
    ) -> Self {
        let mut rows: Vec<(&'a str, Option<Reduction<'a>>, Option<u32>)> =
            rows.into_iter().filter(|(n, _, _)| !n.is_empty()).collect();
        rows.sort_by_key(|(n, _, _)| *n);
        // Mechanism E sees exactly what board #980 gives it and nothing else.
        let empty = TuEmptyCallees::of_rows(
            rows.iter().filter_map(|(n, r, _)| Some((*n, (*r)?))),
        );
        let rows = rows
            .into_iter()
            .map(|(n, r, w)| {
                let def = match r {
                    Some(Reduction::Parsed(f)) => Some(f),
                    // Refused: E may still have an edge, the splice may not.
                    _ => None,
                };
                (n, def, w)
            })
            .collect();
        Self { empty, rows }
    }"""

OLD_DEF = """        Some((self.rows[i].1, self.rows[i].2))"""
NEW_DEF = """        Some((self.rows[i].1?, self.rows[i].2))"""

OLD_USE = "use crate::elide::{TuEmptyCallees, drops_tail_call};"
NEW_USE = "use crate::elide::{Reduction, TuEmptyCallees, drops_tail_call};"


def edit(path, pairs):
    s = open(path).read()
    for old, new in pairs:
        if old not in s:
            sys.exit("MISSING in %s: %r" % (path, old[:80]))
        s = s.replace(old, new, 1)
    open(path, "w").write(s)
    print("  %-44s %d edit(s)" % (path, len(pairs)))


def main():
    edit(SPLICE, [(OLD_USE, NEW_USE), (OLD_ROWS, NEW_ROWS),
                  (OLD_CTOR, NEW_CTOR), (OLD_DEF, NEW_DEF)])
    print("splice.rs: of_rows is the general constructor; definition() is "
          "parsed-only; every row stays visible to mentions()")


if __name__ == "__main__":
    main()
