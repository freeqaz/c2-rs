//! **GRID-M — mechanism E through a REFUSED callee**, against real `c2`.
//!
//! Board **#980**: 370 workload functions where the port emits `li rN,0 ; b
//! callee` and c2's whole body is one `4e800020` with no relocation. The callee
//! is not empty — it is *parse-refused*, production `expr-intrinsic-memset` —
//! and lane `w-seq` (#966/#971) measured that every one of them is blocked
//! there. `crates/c2-il/src/func/body/shapes/no_effect.rs` reads that body
//! without accepting it, and `c2_core::elide::Reduction::NoEffectCall` feeds the
//! answer to E's own least fixpoint.
//!
//! Every cell here is the **frozen source** of `work/w-inl0/cells/`, pulled in
//! by `include_str!` rather than copied: the grid's `sha256` was committed
//! before its first `cl.exe` (`work/w-inl0/CELLS.sha256`,
//! `work/w-inl0/ADDENDUM-1.md`), and including the file is what keeps this test
//! grading the bytes that were frozen instead of a transcription of them.
//!
//! The grading route is `w-empty`'s: compile with the **real toolchain** at the
//! workload's own profile and hand the census rows and the reference COMDAT
//! bytes to **`grade_one`**, the same function the 878-TU scan runs. `SKIP:
//! toolchain absent` when there is no toolchain.
//!
//! # What each test is FOR
//!
//! | test | the claim, and what going red means |
//! |---|---|
//! | `the_dead_temporary_chain_collapses_to_one_blr` | m01 — the rule fires and the bytes are c2's. Also the `/Ob0` row: if the caller is a `blr` at `/O1` and a call at `/Ob0`, this is mechanism **I** and the rule must be withdrawn |
//! | `a_callee_that_keeps_bytes_stops_the_chain` | m02 — the **callee** condition, which is the whole of board #950's hazard for this rule |
//! | `a_real_memset_is_not_a_dead_temporary` | m03 — c2 lowers a real `memset` to a REL24 tail call, so the reader must decline it |
//! | `a_second_statement_stops_the_reader` | m04 — "emits nothing" is a property of the whole body, not of the call |
//! | `the_chain_closes_one_link_deeper` | m05 — the fact is a **link into the fixpoint**, not a one-step rule |
//! | `the_loop_overload_converts_once_its_leaf_can_seed` | m06 — the 228 members of #980 this lane did **not** close. It asserted the decline until board **#1053** closed them, and its going red was the signal `w-inl0` planted it for; the `/Ob0` row is unchanged and is what said they were **E behind an unreadable body** rather than mechanism I |
//! | `an_external_callee_keeps_its_relocation` | m07 — the same-TU condition |
//! | `a_cycle_of_dead_temporary_bodies_is_never_admitted` | m08 — a cycle is never seeded, so it is never admitted, and the closure terminates. **c2 collapses this cycle and the port declines it**: a registered prediction that lost, kept in the test as what it turned out to be |

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use c2_core::elide::TuEmptyCallees;
use c2_harness::gap::fnbytes::{grade_one, tu_empty_callees, FnByte};
use c2_reference::Toolchain;

/// The workload's own profile, minus the `/I` paths a standalone cell cannot
/// use. `/O1` implies `/Gy`; `/Ox` does not.
const FLAGS: [&str; 8] = c2_harness::testsupport::WORKLOAD_FLAGS;

/// `w-empty`'s ANCHOR, **prepended** to every cell: a callee this TU does not
/// define, whose relocation must survive. Without it "the port emitted no
/// branch" and "nothing in this cell emitted anything" are the same observation
/// — STATUS trap 5 in its most literal form.
///
/// **Prepended, and that is not cosmetic.** `w-empty`'s cells append it, and
/// appending is fine there because those cells define no templates. These do,
/// and a template instantiation's segment is emitted *after* every source-order
/// function — so the **last source-order** function is the one that carries the
/// module trailer `4F 02 20 00 4F 01 <line>` without the `4D` that closes it,
/// and `eat_fn_tail` refuses exactly that. Appended, the ANCHOR came back
/// `Refused` in four of the eight cells and the control could not fire at all.
/// Prepended, the trailer lands on `?use`, which no test asserts about.
const ANCHOR: &str = "\nvoid ext_anchor();\nvoid anchor() { ext_anchor(); }\n";

/// **The TAIL PAD**, appended after every cell — and it is a control's
/// scaffolding, not part of any cell.
///
/// `.ex`'s **last** function segment ends `47 54 01 54 00 4D` with no
/// `4F 02 20 00` before it, and `eat_fn_tail` requires either the segment end or
/// the full module trailer, so the last segment in a bundle always refuses as
/// `module-end-0x4D`. In a real workload TU that is some anonymous instantiation
/// nobody asks about; in a five-function cell it was `??$aux@…` — the empty leaf
/// the whole chain has to be **seeded** from. Measured: without this pad m01's
/// wrapper grades `Differs` (port `li r5,0 ; b`, c2 `blr`) purely because the
/// leaf could not parse, and the rule under test never runs.
///
/// A template, because template instantiations are emitted after every
/// source-order function and this one has to be the last of them.
/// **Five levels deep, measured rather than guessed.** Instantiations are not
/// emitted in source order: a one-level pad after m01 still left `??$aux@…`
/// last, because `aux` is reached three template levels down from `?use` and
/// c1xx emits the deeper instantiation later. Five levels puts the pad's leaf
/// past it, and the census then reads `??$aux@…` as `empty-body` — which is what
/// seeds the chain.
const TAIL: &str = "
template <class T> inline T pad5(T v) { return v; }
template <class T> inline T pad4(T v) { return pad5(v); }
template <class T> inline T pad3(T v) { return pad4(v); }
template <class T> inline T pad2(T v) { return pad3(v); }
template <class T> inline T pad1(T v) { return pad2(v); }
int pad_use(int v) { return pad1(v); }
";

const BLR: [u8; 4] = [0x4e, 0x80, 0x00, 0x20];

// The FROZEN grid. `include_str!` and not a copy — see the module header.
const M01: &str = include_str!("../../../work/w-inl0/cells/m01.cpp");
const M02: &str = include_str!("../../../work/w-inl0/cells/m02.cpp");
const M03: &str = include_str!("../../../work/w-inl0/cells/m03.cpp");
const M04: &str = include_str!("../../../work/w-inl0/cells/m04.cpp");
const M05: &str = include_str!("../../../work/w-inl0/cells/m05.cpp");
const M06: &str = include_str!("../../../work/w-inl0/cells/m06.cpp");
const M07: &str = include_str!("../../../work/w-inl0/cells/m07.cpp");
const M08: &str = include_str!("../../../work/w-inl0/cells/m08.cpp");

fn work(tag: &str) -> PathBuf {
    c2_harness::testsupport::scratch_dir("w-inl0", tag)
}

/// `(shape, verdict, symbol, reference bytes, reference relocation count)` per
/// emitted `.text` COMDAT.
type Rows = Vec<(&'static str, FnByte, String, Vec<u8>, usize)>;

/// Grade one frozen cell. `extra` is appended to the profile — `["/Ob0"]` is the
/// E-versus-I separator (`w-fix` #954), and `[]` is the workload's own setting.
fn grade_cell(tc: &Toolchain, dir: &Path, name: &str, body: &str, extra: &[&str]) -> (Rows, TuEmptyCallees) {
    let cpp = dir.join(format!("{name}.cpp"));
    std::fs::write(&cpp, format!("{ANCHOR}{body}{TAIL}")).unwrap();
    let mut flags: Vec<String> = FLAGS.iter().map(|s| s.to_string()).collect();
    flags.extend(extra.iter().map(|s| s.to_string()));
    let src = c2_reference::to_wibo_path(&cpp);
    let Ok(cap) = tc.capture_reference_with(&src, dir, &flags, None) else {
        return (Vec::new(), TuEmptyCallees::none());
    };
    let (Some(census), Some(entries)) = (
        cap.bundle.census_functions(),
        cap.ref_obj.text_comdat_functions_with_bytes(),
    ) else {
        return (Vec::new(), TuEmptyCallees::none());
    };
    let relocs = cap.ref_obj.text_comdat_reloc_sites().unwrap_or_default();
    let mut claim: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (i, (f, _)) in census.iter().enumerate() {
        if let Some(n) = f.emit_name.as_deref() {
            claim.entry(n).or_default().push(i);
        }
    }
    let tu = tu_empty_callees(&census);
    // The reference obj's own relocation records, positionally paired with the
    // COMDAT walk (both walk `text_comdat_entries`). Handed to `grade_one` so
    // these cells are graded on the FULL identity — bytes AND relocations —
    // which is what the 878-TU scan does since lane `w-relo`.
    let rel = cap.ref_obj.text_comdat_relocs();
    let mut out = Vec::new();
    for (idx, (sym, bytes)) in entries.iter().enumerate() {
        let row = match claim.get(sym.as_str()).map(Vec::as_slice) {
            Some([i]) => Some(&census[*i]),
            _ => None,
        };
        let rr = rel.as_ref().and_then(|v| v.get(idx)).map(|(_, r)| r.as_slice());
        let g = grade_one(row, Some(bytes.as_slice()), &tu, rr);
        let n = relocs
            .iter()
            .find(|(n, _)| n == sym)
            .map(|(_, v)| v.len())
            .unwrap_or(0);
        out.push((g.shape, g.verdict, sym.clone(), bytes.clone(), n));
    }
    // The E half, CLONED out of the composite context. Since lane `w-splice`,
    // `tu_empty_callees` returns a `TuContext` — mechanism E's callee set plus
    // mechanism I's splice sources — and it borrows `census`, which is local.
    // These assertions are about mechanism E, whose set is unchanged, so the
    // owned half is lifted out and outlives the capture.
    let empty = tu.empty_callees().clone();
    (out, empty)
}

/// The one row whose mangled name contains `needle`, with the ANCHOR control
/// checked first.
///
/// Symbols are matched by **substring** because a template instantiation's
/// mangling carries its argument types, and transcribing those by hand is a
/// second copy of the capture that can rot against it. The match is required to
/// be **unique** — two hits is a cell whose names are ambiguous and it panics
/// rather than picking one.
fn row<'a>(rows: &'a Rows, needle: &str, cell: &str) -> &'a (&'static str, FnByte, String, Vec<u8>, usize) {
    match rows.iter().find(|r| r.2 == "?anchor@@YAXXZ") {
        Some(a) => assert_eq!(
            a.1,
            FnByte::Exact,
            "cell `{cell}`: the ANCHOR control is not Exact — this capture graded \
             nothing trustworthy, so no verdict below it means anything"
        ),
        None => panic!(
            "cell `{cell}`: no `?anchor@@YAXXZ` COMDAT in the reference obj — the \
             capture produced {} functions and none of them is the control",
            rows.len()
        ),
    }
    let hits: Vec<&(&'static str, FnByte, String, Vec<u8>, usize)> =
        rows.iter().filter(|r| r.2.contains(needle)).collect();
    match hits.as_slice() {
        [one] => one,
        _ => panic!(
            "cell `{cell}`: `{needle}` matches {} of the emitted symbols {:?}",
            hits.len(),
            rows.iter().map(|r| &r.2).collect::<Vec<_>>()
        ),
    }
}

/// **m01 — THE SHAPE.** The wrapper's whole body is one `blr`, and so is the
/// refused callee's; the port must emit the first and still refuse the second.
///
/// The `/Ob0` half is the load-bearing one. Mechanism E is not governed by
/// `/Ob` and mechanism I is, so a caller that is a bare `blr` at `/O1` and a
/// call at `/Ob0` would be **I** wearing E's clothes — `c19_ret_param`'s trap
/// one level out — and this rule would have to be withdrawn.
#[test]
fn the_dead_temporary_chain_collapses_to_one_blr() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("m01");
    for extra in [&[] as &[&str], &["/Ob0"]] {
        let at = if extra.is_empty() { "/O1" } else { "/O1 /Ob0" };
        let (rows, tu) = grade_cell(&tc, &d, "m01", M01, extra);
        let w = row(&rows, "??$destroy_range@", "m01");
        assert_eq!(
            (w.3.as_slice(), w.4),
            (BLR.as_slice(), 0),
            "m01 at {at}: c2's own body for the wrapper is not one `blr` with no \
             relocation — the premise board #980 rests on has changed"
        );
        assert_eq!(
            (w.0, w.1),
            ("tail", FnByte::Exact),
            "m01 at {at}: the port must select this as a tail call and then emit \
             nothing for it. A `Differs` here means the no-effect reader stopped \
             feeding the fixpoint; a shape other than `tail` means the parser moved"
        );
        let c = row(&rows, "??$dr@", "m01");
        assert_eq!(
            (c.3.as_slice(), c.4),
            (BLR.as_slice(), 0),
            "m01 at {at}: c2's body for the REFUSED callee is not one `blr`"
        );
        assert_eq!(
            (c.0, c.1),
            ("parse-refused", FnByte::Refused),
            "m01 at {at}: THE CALLEE PARSES NOW. This rule is decode-only by \
             construction and `IlBundle::functions` must keep refusing this TU \
             (board #971 condition 4). Widening it is a different rung and needs \
             its own grid"
        );
        assert!(
            tu.reduces_to_nothing(&c.2),
            "m01 at {at}: the fixpoint did not admit the refused callee `{}`",
            c.2
        );
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// **m02 — THE CALLEE CONDITION.** Give the leaf a store and nothing may be
/// elided. This is the guard whose removal is board #950's hazard: the rule is
/// keyed on the callee's decoded IL, never on "no relocation appeared".
#[test]
fn a_callee_that_keeps_bytes_stops_the_chain() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("m02");
    let (rows, tu) = grade_cell(&tc, &d, "m02", M02, &[]);
    let c = row(&rows, "??$dr@", "m02");
    assert_ne!(
        c.3, BLR,
        "m02: c2's body for `?dr` IS a bare `blr`, so the cell no longer tests \
         what it is named for — the leaf's store was optimized away"
    );
    assert!(
        !tu.reduces_to_nothing(&c.2),
        "THE CALLEE CONDITION WAS DROPPED: `{}` keeps bytes and the fixpoint \
         admitted it anyway. Every caller of it would now be emitted as `blr` \
         against a c2 body that is not one",
        c.2
    );
    let w = row(&rows, "??$destroy_range@", "m02");
    // **2026-08-09, lane `w-inlfence` (board #2224): this was `Differs` and it
    // is `Refused` now, and the change is the point of that lane.**
    //
    // The wrapper's callee is defined in this TU and does NOT reduce to nothing
    // (the assertion above is exactly that), so c2 may inline it — and here it
    // does, folding the leaf's store into the wrapper. The port has no model of
    // that (mechanism I), and until this lane it emitted `b ?dr` anyway and this
    // cell pinned the resulting **wrong body** as the honest outcome. It is not
    // the honest outcome; `CLAUDE.md`'s rule is that a refusal is strictly
    // better than a wrong emit, and `IlBundle::functions` had always refused the
    // whole TU for exactly this reason while the per-function census went on
    // claiming the body.
    //
    // So the cell now grades the fence: the wrapper is refused **before**
    // codegen, by `callee-defined-in-tu`. Everything m02 was written to guard is
    // unchanged and still asserted above — the callee condition, and that `?dr`
    // keeps bytes. An `Exact` here would still be the alarm it always was.
    assert_eq!(
        w.1,
        FnByte::Refused,
        "m02: the wrapper came back {:?}. c2 inlines the leaf's store into it \
         (mechanism I, which this port does not model), and the inline fence \
         must refuse the wrapper rather than let the port emit a body c2 does \
         not have. `Differs` means the fence stopped firing and the port is \
         emitting a measured-wrong body again; `Exact` would mean it emitted \
         nothing for a body that does something",
        w.1
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// **m03 — a real `memset` is the opposite of nothing.** `docs/IL_CAST_CONVERT.md`
/// §1.3 records that c2 lowers selector 173 over a pointer to `b <memset>` with
/// a REL24, and this cell is that claim compiled.
#[test]
fn a_real_memset_is_not_a_dead_temporary() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("m03");
    let (rows, tu) = grade_cell(&tc, &d, "m03", M03, &[]);
    let r = row(&rows, "?clear@", "m03");
    assert!(
        r.4 > 0,
        "m03: c2's `?clear` carries {} relocations. The whole point of the cell \
         is that a real `memset` is a CALL — with none, it is not this shape and \
         the reader's decline below proves nothing",
        r.4
    );
    assert!(
        !tu.reduces_to_nothing(&r.2),
        "THE READER TOOK A REAL `memset`: `{}` emits a relocated branch to \
         `memset` and the fixpoint admitted it as emitting nothing",
        r.2
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// **m04 — the walk is TOTAL.** One more statement in the body and the reader
/// must decline: "emits nothing" is a property of the whole segment.
#[test]
fn a_second_statement_stops_the_reader() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("m04");
    let (rows, tu) = grade_cell(&tc, &d, "m04", M04, &[]);
    let c = row(&rows, "??$dr@", "m04");
    assert!(
        !tu.reduces_to_nothing(&c.2),
        "THE TOTALITY TEST WAS DROPPED: `{}` stores through its formal and the \
         fixpoint admitted it as emitting nothing",
        c.2
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// **m05 — the fact is a LINK, not a verdict.** One more wrapper above m01's,
/// reachable only through the refused body. A one-step rule fails this cell.
#[test]
fn the_chain_closes_one_link_deeper() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("m05");
    let (rows, _tu) = grade_cell(&tc, &d, "m05", M05, &[]);
    for needle in ["??$destroy_range2@", "??$destroy_range@"] {
        let w = row(&rows, needle, "m05");
        assert_eq!(
            w.3, BLR,
            "m05: c2's body for `{}` is not one `blr`",
            w.2
        );
        assert_eq!(
            w.1,
            FnByte::Exact,
            "m05: `{}` came back {:?}. THE CLOSURE STOPPED PROPAGATING through \
             the refused link — the no-effect fact must enter the least fixpoint, \
             not be applied one step",
            w.2,
            w.1
        );
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// **m06 — THE RESIDUE, CLOSED.** A class element type takes STLport's other
/// overload, whose body is a LOOP over a pseudo-destructor call.
///
/// > **This assertion was INVERTED on 2026-08-08, and its going red was the
/// > intended signal.** `w-inl0` wrote it as a **decline**, saying in as many
/// > words that *"a lane that later converts them will turn this test red and
/// > should — with the rung that explains why"*. Two lanes did it between them:
/// > `w-memset` read the LOOP and handed E a link, which converted nothing on its
/// > own, and `w-seed` (board **#1053**) let the chain's leaf — `p->~T()` on a
/// > trivially destructible class, a body with no call in it at all — **SEED**
/// > the fixpoint. The rung is `docs/rungs/2026-08-08-w-seed.md` and the grid
/// > that earns it is GRID-N (`work/w-seed/cells/`).
///
/// The `/Ob0` row is unchanged and is still the finding it was: `ADDENDUM-1` §2
/// predicted c2 would keep a call somewhere in this chain at `/Ob0` — that the
/// loop vanishes by *inlining*. It does not, so what erases it is c2's own
/// dead-code elimination and the residue was **mechanism E behind a body the
/// parser cannot read**. That reading is what made the follow-on a parser rung,
/// and this test now records it as having been the right one.
#[test]
fn the_loop_overload_converts_once_its_leaf_can_seed() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("m06");
    // **THE /Ob0 ROW, and a registered prediction that lost.** `ADDENDUM-1` §2
    // predicted c2 would keep a call somewhere in this chain at `/Ob0` — that
    // the loop vanishes by *inlining*, mechanism I. It does not: the wrapper is
    // one `4e800020` at `/Ob0` as well, so what erases the loop is c2's own
    // dead-code elimination, and the residue is **mechanism E behind a body the
    // parser cannot read** rather than an inlining question. That makes the
    // follow-on rung a parser rung (board #922's population), which is a
    // materially better answer than the one predicted.
    let (rows_ob0, _) = grade_cell(&tc, &d, "m06", M06, &["/Ob0"]);
    let w0 = row(&rows_ob0, "?destroy_range@", "m06");
    assert_eq!(
        w0.3, BLR,
        "m06 at /Ob0: c2 kept bytes for the wrapper. The residue is then \
         mechanism I after all and the rung's §5 reading is wrong — re-derive it"
    );
    let (rows, tu) = grade_cell(&tc, &d, "m06", M06, &[]);
    let w = row(&rows, "?destroy_range@", "m06");
    assert_eq!(
        w.3, BLR,
        "m06: c2's body for the wrapper is not one `blr` — the cell no longer \
         reproduces the residue it is named for"
    );
    assert!(
        tu.reduces_to_nothing(&w.2),
        "m06 STOPPED CONVERTING. c2's body for the wrapper is one `blr` and the \
         port no longer agrees — either the LOOP link (`no_effect_loop`, lane \
         w-memset) or the SEED at its leaf (`no_effect_nothing`, board #1053) has \
         stopped reaching `elide.rs`'s fixpoint"
    );
    assert_eq!(
        (w.0, w.1),
        ("tail", FnByte::Exact),
        "m06: the wrapper is {:?} — it is in the closure but the emitter did not \
         act on it, which is the two halves of one rule disagreeing",
        w.1
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// **m07 — the same-TU condition.** The tag temporary is built for a call to a
/// function this TU only declares; c2 keeps the relocation and nothing may be
/// elided.
#[test]
fn an_external_callee_keeps_its_relocation() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("m07");
    let (rows, tu) = grade_cell(&tc, &d, "m07", M07, &[]);
    let c = row(&rows, "?dr@", "m07");
    assert!(
        c.4 > 0,
        "m07: c2's `?dr` carries no relocation, so the external call was not \
         emitted and the cell tests nothing"
    );
    assert!(
        !tu.reduces_to_nothing(&c.2),
        "THE SAME-TU CONDITION WAS DROPPED: `{}` calls a function this TU does \
         not define, c2 emits a REL24 for it, and the fixpoint admitted it",
        c.2
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// **m08 — THE CYCLE.** Two dead-temporary bodies calling each other. Neither is
/// ever *seeded*, so the least fixpoint admits neither and terminates; the round
/// ceiling must not fire.
#[test]
fn a_cycle_of_dead_temporary_bodies_is_never_admitted() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("m08");
    let (rows, tu) = grade_cell(&tc, &d, "m08", M08, &[]);
    assert!(
        !tu.overflowed(),
        "m08: the round ceiling FIRED on a two-node cycle — the iteration is no \
         longer monotone"
    );
    // **A REGISTERED PREDICTION THAT LOST, kept as an assertion of what is
    // actually true.** `ADDENDUM-1` §2 predicted "neither of c2's bodies is a
    // bare `blr`", by analogy with `w-fix`'s `k10_cycle2`, where
    // `void a(){b();} void b(){a();}` keeps a branch word in each member. What
    // c2 does with THIS cycle is **asymmetric**: `?a_` collapses to one
    // `4e800020` and `?b_` comes back a 12-word FRAMED body that still calls.
    // Both are declined here, which is the only safe answer — a rule that
    // admitted a cycle would have to admit `k10` too — and the `?a_` half is a
    // match the port gives up on purpose, `c19_ret_param`'s shape.
    for (needle, ref_is_blr) in [("?a_@", true), ("?b_@", false)] {
        let r = row(&rows, needle, "m08");
        assert!(
            !tu.reduces_to_nothing(&r.2),
            "A CYCLE WAS ADMITTED: `{}` reaches nothing but itself and the \
             fixpoint took it. A cycle is never SEEDED, so this can only mean a \
             refused body was made a seed instead of a link",
            r.2
        );
        assert_eq!(
            r.3 == BLR,
            ref_is_blr,
            "m08: c2's body for `{}` changed sides — this cell records that c2 \
             collapses one member of the cycle and not the other, and the \
             rung's §5 quotes it. Re-derive before repeating the claim. Got {:?}",
            r.2,
            r.3
        );
    }
    let _ = std::fs::remove_dir_all(&d);
}
