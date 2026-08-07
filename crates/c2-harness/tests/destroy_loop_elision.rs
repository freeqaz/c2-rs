//! **GRID-L — mechanism E through a refused LOOP**, against real `c2`.
//!
//! Board **#980**'s residue. Lane `w-inl0` read the dead-temporary call and
//! closed 138 of the 370; the 232 that remain on master `217d4a85` all stop at
//! the *next* level down, which `fnbyte-blr-stop2` prices at **228** under the
//! production `return-scope-close-cflow-label`. That production is STLport's
//! `__destroy_range_aux(_first, _last, __false_type)` — the overload a **class**
//! element type takes — and it is a **loop**.
//!
//! `crates/c2-il/src/func/body/shapes/no_effect.rs::no_effect_loop` reads that
//! body without accepting it, and hands the loop's callee to the same
//! `c2_core::elide::Reduction::NoEffectCall` link `w-inl0` already ships.
//! **Nothing in `crates/c2-core/` changed and `parse_segment` is byte-for-byte
//! unchanged**: a body this reader recognizes is still `FnVerdict::Blocked`,
//! still `fnbyte-refused`, and `IlBundle::functions` still refuses its whole TU.
//!
//! # The five-level chain, and where it STOPS
//!
//! Read out of `src/lazer/meta_ham/CharacterProvider.cpp` with
//! `c2rs census --fn`, and reproduced by `l01`:
//!
//! | # | function | census key |
//! |---|---|---|
//! | 1 | `??$_Destroy_Range@PAVSymbol@@…` | **in class** — the differ |
//! | 2 | `??$__destroy_range@…` | `expr-intrinsic-memset` — read by `w-inl0` |
//! | 3 | `??$__destroy_range_aux@…` | `return-scope-close-cflow-label` — **read here** |
//! | 4 | `??$_Destroy@…` | `expr-intrinsic-memset` — read by `w-inl0` |
//! | 5 | `??$__destroy_aux@…` | `expr-lit-type-8207` — **the STOP** |
//!
//! Level 5 is `p->~T()` on a class with a trivial destructor: an `int` literal, a
//! `void` literal, a bind and a discard, with **no call in it at all**. For that
//! chain to close, level 5 must **SEED** E's fixpoint — and
//! `c2_core::elide::Reduction` documents that a refused body contributes *"a link
//! and never a seed"*. `l09` is that stop compiled, and it asserts the residue
//! rather than reaching over it.
//!
//! # What each test is FOR
//!
//! | test | the claim, and what going red means |
//! |---|---|
//! | `a_loop_over_an_empty_callee_collapses_to_one_blr` | l02 — the positive. The loop is a LINK and the chain closes through it with no change to E's rule. Also the `/Ob0` row: a caller that is a `blr` at `/O1` and a call at `/Ob0` is mechanism **I** and this reader would have to be withdrawn |
//! | `the_induction_and_the_comparison_are_not_pinned_to_one_value` | l12 — stride 8 and `<` instead of stride 4 and `!=`. Registered as the prediction most likely to lose |
//! | `a_dead_temporary_call_inside_the_loop_composes` | l08 — the workload's own level 3→4 edge: the two readers share one argument walk, not two |
//! | `a_loop_whose_callee_keeps_bytes_stops_the_chain` | l03 — the callee condition, which is board #950's hazard for this rule |
//! | `a_loop_over_an_external_callee_keeps_its_relocation` | l04 — the same-TU condition, one level inside the loop |
//! | `a_second_statement_in_the_loop_body_is_refused` | l05 — "emits nothing" is a property of the whole body |
//! | `an_impure_induction_step_is_refused` | l06 — the step must be one lvalue, one literal, one operator |
//! | `a_condition_over_a_global_is_refused` | l07 — the test must read only formals, or the body materializes data |
//! | `a_loop_that_stores_is_refused` | l10 — the reader is not "any loop with a matched label set" |
//! | `a_cycle_through_a_loop_is_never_admitted` | l11 — never seeded, so never admitted, and the closure terminates |
//! | `the_pseudo_destructor_leaf_is_the_residue_and_needs_a_SEED` | l01/l09 — c2 emits one `blr` for the whole chain and the port converts **nothing**. The stop is a missing capability, not a missing production |

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use c2_core::elide::TuEmptyCallees;
use c2_harness::gap::fnbytes::{grade_one, tu_empty_callees, FnByte};
use c2_reference::Toolchain;

/// The workload's own profile, minus the `/I` paths a standalone cell cannot
/// use. `/O1` implies `/Gy`; `/Ox` does not.
const FLAGS: [&str; 8] = [
    "/nologo", "/wd4355", "/wd4164", "/c", "/GR", "/O1", "/Oi", "/EHsc",
];

/// `w-empty`'s ANCHOR, **prepended** — a callee this TU does not define, whose
/// relocation must survive. Without it "the port emitted no branch" and "nothing
/// in this cell emitted anything" are the same observation. Prepended and not
/// appended for `w-inl0` §4's measured reason: these cells define templates, and
/// a template instantiation's segment is emitted after every source-order
/// function, so an appended anchor lands on the module trailer that `eat_fn_tail`
/// refuses.
const ANCHOR: &str = "\nvoid ext_anchor();\nvoid anchor() { ext_anchor(); }\n";

/// **The TAIL PAD**, appended after every cell — scaffolding for the controls,
/// not part of any cell. `.ex`'s last function segment always refuses as
/// `module-end-0x4D`, and in a five-function cell that would be the empty leaf
/// the whole chain has to be seeded from. Five levels deep, measured by `w-inl0`
/// §4 rather than guessed.
const TAIL: &str = "
template <class T> inline T pad5(T v) { return v; }
template <class T> inline T pad4(T v) { return pad5(v); }
template <class T> inline T pad3(T v) { return pad4(v); }
template <class T> inline T pad2(T v) { return pad3(v); }
template <class T> inline T pad1(T v) { return pad2(v); }
int pad_use(int v) { return pad1(v); }
";

const BLR: [u8; 4] = [0x4e, 0x80, 0x00, 0x20];

// The FROZEN grid — `include_str!` and not a copy, so this test grades the bytes
// whose `sha256` was committed before the first `cl.exe`
// (`work/w-memset/CELLS.sha256`, `work/w-memset/ADDENDUM-1.md`).
const L01: &str = include_str!("../../../work/w-memset/cells/l01.cpp");
const L02: &str = include_str!("../../../work/w-memset/cells/l02.cpp");
const L03: &str = include_str!("../../../work/w-memset/cells/l03.cpp");
const L04: &str = include_str!("../../../work/w-memset/cells/l04.cpp");
const L05: &str = include_str!("../../../work/w-memset/cells/l05.cpp");
const L06: &str = include_str!("../../../work/w-memset/cells/l06.cpp");
const L07: &str = include_str!("../../../work/w-memset/cells/l07.cpp");
const L08: &str = include_str!("../../../work/w-memset/cells/l08.cpp");
const L09: &str = include_str!("../../../work/w-memset/cells/l09.cpp");
const L10: &str = include_str!("../../../work/w-memset/cells/l10.cpp");
const L11: &str = include_str!("../../../work/w-memset/cells/l11.cpp");
const L12: &str = include_str!("../../../work/w-memset/cells/l12.cpp");

fn work(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("c2rs-w-memset-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// `(shape, verdict, symbol, reference bytes, reference relocation count)` per
/// emitted `.text` COMDAT.
type Rows = Vec<(&'static str, FnByte, String, Vec<u8>, usize)>;

/// Grade one frozen cell. `extra` is appended to the profile — `["/Ob0"]` is the
/// E-versus-I separator (`w-fix` #954), and `[]` is the workload's own setting.
fn grade_cell(
    tc: &Toolchain,
    dir: &Path,
    name: &str,
    body: &str,
    extra: &[&str],
) -> (Rows, TuEmptyCallees) {
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
    let empty = tu.empty_callees().clone();
    (out, empty)
}

/// The one row whose mangled name contains `needle`, with the ANCHOR control
/// checked first. A cell whose anchor is not `Exact` graded nothing trustworthy.
fn row<'a>(
    rows: &'a Rows,
    needle: &str,
    cell: &str,
) -> &'a (&'static str, FnByte, String, Vec<u8>, usize) {
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

/// **l02 — THE POSITIVE.** The loop's callee is `empty_body`, so the existing
/// seed is reachable and the whole chain closes through the loop LINK alone.
///
/// The `/Ob0` half is the load-bearing one: mechanism E is not governed by `/Ob`
/// and mechanism I is, so a wrapper that is a bare `blr` at `/O1` and a call at
/// `/Ob0` would be I wearing E's clothes and this reader would have to go.
#[test]
fn a_loop_over_an_empty_callee_collapses_to_one_blr() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("l02");
    for extra in [&[] as &[&str], &["/Ob0"]] {
        let at = if extra.is_empty() { "/O1" } else { "/O1 /Ob0" };
        let (rows, tu) = grade_cell(&tc, &d, "l02", L02, extra);
        let w = row(&rows, "?destroy_range@", "l02");
        assert_eq!(
            (w.3.as_slice(), w.4),
            (BLR.as_slice(), 0),
            "l02 at {at}: c2's own body for the wrapper is not one `blr` with no \
             relocation — the premise this cell rests on has changed"
        );
        assert_eq!(
            (w.0, w.1),
            ("tail", FnByte::Exact),
            "l02 at {at}: the port must select this as a tail call and emit \
             nothing for it. A `Differs` means `no_effect_loop` stopped feeding \
             the fixpoint"
        );
        let a = row(&rows, "?aux@", "l02");
        assert_eq!(
            (a.0, a.1),
            ("parse-refused", FnByte::Refused),
            "l02 at {at}: THE LOOP PARSES NOW. This reader is decode-only by \
             construction and `IlBundle::functions` must keep refusing this TU \
             (board #971 condition 4). Accepting the body is a different rung"
        );
        assert!(
            tu.reduces_to_nothing(&a.2),
            "l02 at {at}: the fixpoint did not admit the refused LOOP `{}` — the \
             link `no_effect_loop` returns is not reaching `elide.rs`",
            a.2
        );
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// **l12 — the induction and the comparison are not pinned to one value.**
/// Stride 8 and `<` where `l02` has stride 4 and `!=`.
///
/// Registered in `ADDENDUM-1.md` as the prediction most likely to lose: the
/// workload only ever shows `++`/`!=` at a stride of `sizeof(T)`, so a reader
/// that had to be widened for this cell would be one with a value smuggled into
/// its grammar (#644). It did not have to be — the stride is read and not
/// constrained, and `<` is in `LOOP_CMP_OPS` **because this cell grades it**.
#[test]
fn the_induction_and_the_comparison_are_not_pinned_to_one_value() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("l12");
    let (rows, tu) = grade_cell(&tc, &d, "l12", L12, &[]);
    let a = row(&rows, "?aux@", "l12");
    assert!(
        tu.reduces_to_nothing(&a.2),
        "l12: a stride of 8 and a `<` test are not read, so the reader is keyed \
         on `l02`'s literal `4` or on its `!=` opcode — which is a value in a \
         grammar's clothing (#644)"
    );
    let w = row(&rows, "?destroy_range@", "l12");
    assert_eq!(
        (w.3.as_slice(), w.4),
        (BLR.as_slice(), 0),
        "l12: c2's body for the wrapper is not one `blr` with no relocation"
    );
    assert_eq!((w.0, w.1), ("tail", FnByte::Exact), "l12: the chain must close");
    let _ = std::fs::remove_dir_all(&d);
}

/// **l08 — the two readers COMPOSE.** The loop's single statement is not a plain
/// call but the tag-dispatch call `w-inl0` already reads, which is the workload's
/// own level 3 → level 4 edge. The argument vocabulary is shared
/// (`eat_no_effect_call_stmt`), not duplicated: `w-relo`'s merge is the reason
/// that matters — two lanes wrote the same reader in different files and
/// auto-merged into duplicate walks with no conflict marker.
#[test]
fn a_dead_temporary_call_inside_the_loop_composes() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("l08");
    let (rows, tu) = grade_cell(&tc, &d, "l08", L08, &[]);
    let a = row(&rows, "?aux@", "l08");
    assert!(
        tu.reduces_to_nothing(&a.2),
        "l08: the loop's statement is a dead-temporary call and the reader did \
         not walk it — the two shapes are not sharing one argument vocabulary"
    );
    let one = row(&rows, "?destroy_one@", "l08");
    assert!(
        tu.reduces_to_nothing(&one.2),
        "l08: the dead-temporary link below the loop is not admitted"
    );
    let w = row(&rows, "?destroy_range@", "l08");
    assert_eq!(
        (w.3.as_slice(), w.4),
        (BLR.as_slice(), 0),
        "l08: c2's body for the wrapper is not one `blr` with no relocation"
    );
    assert_eq!(
        (w.0, w.1),
        ("tail", FnByte::Exact),
        "l08: the chain must close through BOTH readers"
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// **l03 — THE CALLEE CONDITION.** Give the loop's leaf a store and nothing may
/// be elided. This is board #950's hazard for this rule: the answer is keyed on
/// the callee's decoded IL, never on "no relocation appeared".
#[test]
fn a_loop_whose_callee_keeps_bytes_stops_the_chain() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("l03");
    let (rows, tu) = grade_cell(&tc, &d, "l03", L03, &[]);
    let a = row(&rows, "?aux@", "l03");
    assert!(
        !tu.reduces_to_nothing(&a.2),
        "THE CALLEE CONDITION WAS DROPPED: the loop's leaf stores to a global and \
         the fixpoint admitted `{}` anyway. Every caller of it would be emitted \
         as `blr` against a c2 body that is not one",
        a.2
    );
    let w = row(&rows, "?destroy_range@", "l03");
    assert!(
        matches!(w.1, FnByte::Differs { .. }) || w.1 == FnByte::Refused,
        "l03: the wrapper came back {:?}. Its chain does something, so an `Exact` \
         here would mean the port emitted nothing for a body that stores",
        w.1
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// **l04 — CONTROL, the same-TU condition one level inside the loop.** The
/// loop's callee is external, so no definition in this bundle can answer for it
/// and c2 keeps a relocation in the chain.
#[test]
fn a_loop_over_an_external_callee_keeps_its_relocation() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("l04");
    let (rows, tu) = grade_cell(&tc, &d, "l04", L04, &[]);
    let a = row(&rows, "?aux@", "l04");
    assert!(
        !tu.reduces_to_nothing(&a.2),
        "THE SAME-TU CONDITION WAS DROPPED: `{}` calls a function this TU does \
         not define and the fixpoint admitted it",
        a.2
    );
    let w = row(&rows, "?destroy_range@", "l04");
    assert!(
        w.4 > 0 || matches!(w.1, FnByte::Differs { .. }) || w.1 == FnByte::Refused,
        "l04: c2's wrapper carries {} relocations and grades {:?} — the cell is \
         supposed to keep a call to `?ext_leaf` somewhere in the chain, and if it \
         does not, this control cannot fire",
        w.4,
        w.1
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// **l05 — the walk is TOTAL.** A second statement in the loop body and the
/// reader must decline: "emits nothing" is a property of the whole segment.
#[test]
fn a_second_statement_in_the_loop_body_is_refused() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("l05");
    let (rows, tu) = grade_cell(&tc, &d, "l05", L05, &[]);
    let a = row(&rows, "?aux@", "l05");
    assert!(
        !tu.reduces_to_nothing(&a.2),
        "THE TOTALITY OF THE WALK WAS DROPPED: the loop body also stores to a \
         global and `{}` was still read as emitting nothing",
        a.2
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// **l06 — the induction step must be PURE.** `f = advance(f)` puts a call in the
/// increment, so the step is no longer one lvalue, one literal and one operator.
#[test]
fn an_impure_induction_step_is_refused() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("l06");
    let (rows, tu) = grade_cell(&tc, &d, "l06", L06, &[]);
    let a = row(&rows, "?aux@", "l06");
    assert!(
        !tu.reduces_to_nothing(&a.2),
        "THE INDUCTION STEP'S PURITY WAS DROPPED: `{}` advances through a CALL \
         and was still read as emitting nothing",
        a.2
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// **l07 — the exit test must read only FORMALS.** A condition over a global is
/// a body that materializes a data symbol, which is `elide.rs`'s condition 3 one
/// level down (`w-fix`'s `k16`).
#[test]
fn a_condition_over_a_global_is_refused() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("l07");
    let (rows, tu) = grade_cell(&tc, &d, "l07", L07, &[]);
    let a = row(&rows, "?aux@", "l07");
    assert!(
        !tu.reduces_to_nothing(&a.2),
        "THE FORMALS TEST WAS DROPPED: `{}` compares against a GLOBAL and was \
         still read as emitting nothing — the body materializes a data symbol",
        a.2
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// **l10 — CONTROL, a loop that EMITS.** The same skeleton with a store as its
/// body and no call at all. The reader is not "any loop with a matched label
/// set".
#[test]
fn a_loop_that_stores_is_refused() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("l10");
    let (rows, tu) = grade_cell(&tc, &d, "l10", L10, &[]);
    let a = row(&rows, "?aux@", "l10");
    assert_ne!(
        a.3, BLR,
        "l10: c2's own body for the storing loop IS a bare `blr`, so the cell no \
         longer tests what it is named for"
    );
    assert!(
        !tu.reduces_to_nothing(&a.2),
        "THE READER TOOK A LOOP THAT EMITS: `{}` writes through its induction \
         variable and the fixpoint admitted it as emitting nothing",
        a.2
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// **l11 — THE CYCLE.** `aux` calls `dr2` and `dr2` calls `aux`. Nothing seeds
/// it, so the least fixpoint admits neither member; and it terminates, which is
/// the property this cell exists for (`w-fix` §3.1's round ceiling).
#[test]
fn a_cycle_through_a_loop_is_never_admitted() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("l11");
    let (rows, tu) = grade_cell(&tc, &d, "l11", L11, &[]);
    assert!(
        !tu.overflowed(),
        "l11: THE ROUND CEILING FIRED. A cycle made the closure non-monotone and \
         the context now admits nothing at all"
    );
    let a = row(&rows, "?aux@", "l11");
    let c = row(&rows, "?dr2@", "l11");
    assert!(
        !tu.reduces_to_nothing(&a.2) && !tu.reduces_to_nothing(&c.2),
        "A CYCLE WAS TREATED AS REDUCING TO NOTHING: `{}` and `{}` call each \
         other and neither is seeded",
        a.2,
        c.2
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// **l01 and l09 — THE RESIDUE, and it needs a SEED rather than a production.**
///
/// Both cells are the workload's own chain for a **class** element type. c2
/// emits one `4e800020` and no relocation for the wrapper at `/O1` **and** at
/// `/Ob0` — so it is mechanism E and not I, exactly as `w-inl0`'s `m06` found —
/// and the port converts **nothing**, because the chain bottoms out at
/// `p->~T()`: an `int` literal, a `void` literal, a bind and a discard, with no
/// call in it. `c2_core::elide::Reduction` documents that a refused body
/// contributes a link and never a seed, so no chain through this leaf can close.
///
/// **This test asserts the residue.** If it ever goes red because the wrapper
/// became `Exact`, E's seed set has been widened and that widening needs its own
/// grid before this assertion is deleted — it is not a stale expectation, it is
/// the boundary.
#[test]
fn the_pseudo_destructor_leaf_is_the_residue_and_needs_a_seed() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("l09");
    for (name, body) in [("l01", L01), ("l09", L09)] {
        for extra in [&[] as &[&str], &["/Ob0"]] {
            let at = if extra.is_empty() { "/O1" } else { "/O1 /Ob0" };
            let (rows, tu) = grade_cell(&tc, &d, name, body, extra);
            let w = row(&rows, "??$destroy_range@", name);
            assert_eq!(
                (w.3.as_slice(), w.4),
                (BLR.as_slice(), 0),
                "{name} at {at}: c2's body for the wrapper is not one `blr` with \
                 no relocation. If this changed the residue is mechanism I after \
                 all and the whole rung is priced wrong"
            );
            assert!(
                matches!(w.1, FnByte::Differs { .. }),
                "{name} at {at}: the wrapper came back {:?}. THE SEED EXISTS NOW \
                 — a refused body with no call in it is being read as reducing to \
                 nothing, which is a change to E's rule and needs the grid that \
                 earns it (`work/w-memset/PREREG.md` §0.3)",
                w.1
            );
            // …and the reason is precisely one level: the LOOP is read and
            // admitted-as-a-link, and the leaf below it is not a link at all.
            let a = row(&rows, "??$aux@", name);
            assert!(
                !tu.reduces_to_nothing(&a.2),
                "{name} at {at}: the loop `{}` is admitted, so the chain below it \
                 closed and the wrapper's `Differs` above has some other cause",
                a.2
            );
        }
    }
    let _ = std::fs::remove_dir_all(&d);
}
