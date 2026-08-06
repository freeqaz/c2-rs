//! **MECHANISM E, against real `c2`** — the call the compiler does not emit
//! because its callee's body is empty (`crates/c2-core/src/elide.rs`,
//! `docs/INLINE_PREDICATE.md` §1, lane `w-empty`).
//!
//! Every case here compiles a source cell with the **real toolchain** at the
//! **workload's own profile** and compares the port's `/Gy` COMDAT body against
//! `c2`'s own bytes through the FBM route — the same `grade_one` the 878-TU scan
//! runs, never a copy. `SKIP: toolchain absent` when there is no toolchain.
//!
//! The cells are written into a temp directory rather than added to
//! `fixtures/cpp`, on purpose: they are *negative* and *boundary* shapes whose
//! whole point is that the port refuses the surrounding TU, and putting them in
//! the fixture corpus would move every fixture-gate count in `docs/STATUS.md`
//! for no measurement gained. They are the same sources as
//! `work/w-empty/cells/`, whose `sha256` grid is what graded the rule.
//!
//! # What each test is FOR
//!
//! | test | the claim, and what going red means |
//! |---|---|
//! | `an_empty_same_tu_callee_leaves_the_caller_a_bare_blr` | the rule fires and the bytes are c2's |
//! | `the_argument_setup_goes_with_the_dropped_call` | E discards the setup — a 2-word port body becomes one word |
//! | `a_returning_callee_is_mechanism_i_and_never_e` | the **emptiness** condition. `int g(int a){return a;}` is mechanism I, and at `/O1` its caller is observationally identical to an E caller. Since lane `w-splice` the port emits those bytes — through `c2_core::splice`, and the test asserts BOTH that the verdict is `Exact` and that the E context still refuses the callee |
//! | `a_callee_this_tu_does_not_define_is_not_elided` | the **same-TU** condition |
//! | `an_indirect_call_site_is_still_refused_by_the_il_parser` | **THE HAZARD.** E does not fire through a function pointer even with an empty callee, and the port is safe only because the parser refuses that caller. If this test goes red because the production now parses, `elide.rs` needs an explicit site condition **before** the elision may run |

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

/// Every cell carries this, and its relocation must survive: a callee this TU
/// does not define. Without it "the port emitted no branch" is indistinguishable
/// from "nothing in this cell emitted anything", which is `docs/STATUS.md`
/// trap 5 in its most literal form.
const ANCHOR: &str = "\nvoid ext_anchor();\nvoid anchor() { ext_anchor(); }\n";

fn work(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("c2rs-w-empty-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Grade one source cell at the workload profile.
///
/// Returns `(shape, verdict, symbol, reference bytes)` per emitted `.text`
/// COMDAT, exactly as `super::gap::fnbytes::measure` would — the port's body is
/// composed by `c2_core::comdat`, which is what `PortC2::build` calls.
type Rows = Vec<(&'static str, FnByte, String, Vec<u8>)>;

fn grade_cell(tc: &Toolchain, dir: &Path, name: &str, body: &str) -> (Rows, TuEmptyCallees) {
    let cpp = dir.join(format!("{name}.cpp"));
    std::fs::write(&cpp, format!("{body}{ANCHOR}")).unwrap();
    let flags: Vec<String> = FLAGS.iter().map(|s| s.to_string()).collect();
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
    let mut claim: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (i, (f, _)) in census.iter().enumerate() {
        if let Some(n) = f.emit_name.as_deref() {
            claim.entry(n).or_default().push(i);
        }
    }
    let tu = tu_empty_callees(&census);
    let mut out = Vec::new();
    for (sym, bytes) in &entries {
        let row = match claim.get(sym.as_str()).map(Vec::as_slice) {
            Some([i]) => Some(&census[*i]),
            _ => None,
        };
        let g = grade_one(row, Some(bytes.as_slice()), &tu);
        out.push((g.shape, g.verdict, sym.clone(), bytes.clone()));
    }
    // The E half, CLONED out of the composite context. `tu` borrows `census`,
    // which is local; the elision context this test asserts against does not,
    // so it can outlive the capture. (`TuContext` gained mechanism I's splice
    // sources in lane `w-splice`; nothing about mechanism E's set changed, and
    // these assertions are about mechanism E.)
    let empty = tu.empty_callees().clone();
    (out, empty)
}

/// One row by symbol, with the anchor control checked first.
///
/// The anchor is `?anchor@@YAXXZ`, a tail call to a callee this TU does not
/// define; the port emits `b ?ext_anchor` and so does c2, so its verdict is
/// `Exact` in every cell. A cell whose anchor is missing or not `Exact` is a
/// cell whose capture went wrong, and the test says so rather than reading a
/// missing row as a passing one.
fn row<'a>(
    rows: &'a [(&'static str, FnByte, String, Vec<u8>)],
    sym: &str,
    cell: &str,
) -> &'a (&'static str, FnByte, String, Vec<u8>) {
    let anchor = rows.iter().find(|r| r.2 == "?anchor@@YAXXZ");
    match anchor {
        Some(a) => assert_eq!(
            a.1,
            FnByte::Exact,
            "cell `{cell}`: the ANCHOR control is not Exact — this capture graded nothing \
             trustworthy, so no verdict below it means anything"
        ),
        None => panic!(
            "cell `{cell}`: no `?anchor@@YAXXZ` COMDAT in the reference obj — the capture \
             produced {} functions and none of them is the control",
            rows.len()
        ),
    }
    rows.iter().find(|r| r.2 == sym).unwrap_or_else(|| {
        panic!(
            "cell `{cell}`: no emitted COMDAT named `{sym}`; got {:?}",
            rows.iter().map(|r| &r.2).collect::<Vec<_>>()
        )
    })
}

const BLR: [u8; 4] = [0x4e, 0x80, 0x00, 0x20];

#[test]
fn an_empty_same_tu_callee_leaves_the_caller_a_bare_blr() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("fires");
    let (rows, _tu) = grade_cell(&tc, &d, "c00", "void g() {}\nvoid f() { g(); }\n");
    let r = row(&rows, "?f@@YAXXZ", "c00_empty");
    assert_eq!(
        r.0, "tail",
        "the port must still SELECT this as a tail call — mechanism E is a \
         composition rule, not an acceptance one"
    );
    assert_eq!(
        r.1,
        FnByte::Exact,
        "`void g(){{}} void f(){{ g(); }}`: c2 emits no branch and no relocation for \
         `?f`, and the port must not either"
    );
    // Independent of the verdict: c2's own bytes for `?f` are one `blr`.
    assert_eq!(
        r.3, BLR,
        "the reference COMDAT for `?f` is not a single `blr` — the premise this \
         whole rule rests on has changed"
    );
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn the_argument_setup_goes_with_the_dropped_call() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("setup");
    // `g05_const_arg`: without E the port emits `li r3,5 ; b ?g` — two words.
    let (rows, _tu) = grade_cell(&tc, &d, "g05", "void g(int a) {}\nvoid f() { g(5); }\n");
    let r = row(&rows, "?f@@YAXXZ", "g05_const_arg");
    assert_eq!(
        r.1,
        FnByte::Exact,
        "E discards the argument setup as well as the branch; a rule that kept \
         the setup would leave `li r3,5 ; blr` against c2's single word"
    );
    assert_eq!(r.3, BLR);
    let _ = std::fs::remove_dir_all(&d);
}

/// **`c19_ret_param` — THE CELL THAT SEPARATES THE TWO MECHANISMS, and the port
/// now gets it right by the right one.**
///
/// `int g(int a){ return a; } int f(int a){ return g(a); }`. c2 emits `?f` as a
/// bare `blr`, and it is **mechanism I** — `/Ob0` restores the `bl ?g`
/// (`docs/INLINE_PREDICATE.md` §1, probe p6). E must never claim it: `?g`'s IL
/// body is not empty, and a rule fitted to the *bytes* would take the whole
/// family and be wrong about all of it (`elide.rs`'s `k12` note).
///
/// Until lane `w-splice` this test asserted `Differs`, because the port modelled
/// only E and therefore emitted `b ?g`. It now emits `?g`'s own body — one
/// `blr`, no relocation — through `c2_core::splice`, so the assertion is
/// inverted and **strengthened**: the verdict must be `Exact` **and** the E
/// context must still refuse the callee. Both halves matter. `Exact` alone would
/// pass if E had quietly widened to cover it, which is precisely the failure the
/// original test existed to catch.
#[test]
fn a_returning_callee_is_mechanism_i_and_never_e() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("returns");
    let (rows, tu) = grade_cell(
        &tc,
        &d,
        "c19",
        "int g(int a) { return a; }\nint f(int a) { return g(a); }\n",
    );
    let r = row(&rows, "?f@@YAHH@Z", "c19_ret_param");
    assert_eq!(
        r.1,
        FnByte::Exact,
        "MECHANISM I REGRESSED ON ITS OWN DISCRIMINATING CELL: c2 emits `?f` as \
         `?g`'s body — one blr, no relocation — and `c2_core::splice` is what \
         produces it. Verdict came back {:?}",
        r.1
    );
    assert_eq!(r.3, BLR, "c2's body for `?f` is the single word `blr`");
    assert!(
        !tu.reduces_to_nothing("?g@@YAHH@Z"),
        "THE EMPTINESS CONDITION WAS DROPPED: `int g(int a){{return a;}}` has a \
         non-empty IL body and is mechanism I, not E. If the E context admits \
         it, the port is emitting the right bytes for the WRONG REASON and the \
         next callee like it will be wrong bytes — `elide.rs`'s k12/c19 trap"
    );
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn a_callee_this_tu_does_not_define_is_not_elided() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("extern");
    // `c22_extern_callee`: the identical call, to a `?g` this TU does not define.
    // c2 keeps the REL24 at `/O1` and at `/Ob0`.
    let (rows, _tu) = grade_cell(&tc, &d, "c22", "void g();\nvoid f() { g(); }\n");
    let r = row(&rows, "?f@@YAXXZ", "c22_extern_callee");
    assert_ne!(
        r.3, BLR,
        "THE SAME-TU CONDITION WAS DROPPED: c2 emits `b ?g` with a relocation \
         here, because `?g` is not defined in this TU. A reference body of `blr` \
         means the cell no longer tests what it is named for"
    );
    assert_eq!(
        r.1,
        FnByte::Exact,
        "the port must emit the branch, exactly as c2 does"
    );
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn an_indirect_call_site_is_still_refused_by_the_il_parser() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("indirect");
    // **THE HAZARD** (`crates/c2-core/src/elide.rs`, "stated where the next lane
    // will read it"). Both cells have an `empty-body` callee and in BOTH c2
    // emits a call: `f09` a direct `b ?g` WITH a relocation, `f10` a `bcctrl`.
    // Mechanism E is a property of the call SITE as well as of the callee, and
    // `elide.rs` does not model the site — the port is safe only because the IL
    // parser refuses these two callers outright.
    for (cell, src, sym) in [
        (
            "f09_fnptr",
            "void g() {}\nvoid f() { void (*p)() = g; p(); }\n",
            "?f@@YAXXZ",
        ),
        (
            "f10_virtual_ptr",
            "struct S { virtual void g() {} };\nvoid f(S* s) { s->g(); }\n",
            "?f@@YAXPAUS@@@Z",
        ),
    ] {
        let (rows, _tu) = grade_cell(&tc, &d, cell, src);
        let r = row(&rows, sym, cell);
        assert_eq!(
            (r.0, r.1),
            ("parse-refused", FnByte::Refused),
            "cell `{cell}`: this caller PARSES now, and `crates/c2-core/src/elide.rs` \
             has no call-SITE condition. c2 emits a call here even though the callee's \
             body is empty, so the elision must be given an explicit site test — or \
             restricted away from this production — BEFORE the parser is widened. \
             Got shape `{}` verdict {:?}",
            r.0,
            r.1
        );
    }
    let _ = std::fs::remove_dir_all(&d);
}

// ===========================================================================
// THE FIXPOINT — board #924, lane `w-fix`. Every case below is a cell of
// `work/w-fix/cells3*`, graded there against real c2 per CALL EDGE at the
// workload's flags and again at `/Ob0`; these tests grade the PORT's bytes for
// the same cells through the same `grade_one` the 878-TU scan runs.
// ===========================================================================

/// `k4_chain_d4` — four empty-bodied links. c2 emits **every** function in this
/// TU as one `4e800020`, and one-step E reaches only the bottom link.
///
/// Going red here means the closure stopped propagating: the port emits a
/// branch where c2 emits nothing, which is `fnbyte-differs` and not a wrong obj
/// (`IlBundle::functions()` refuses the TU) — but it is the rule failing.
#[test]
fn a_chain_of_empty_callees_collapses_at_every_link() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("chain4");
    let (rows, tu) = grade_cell(
        &tc,
        &d,
        "k4",
        "void h() {}\nvoid g3() { h(); }\nvoid g2() { g3(); }\n\
         void g1() { g2(); }\nvoid f() { g1(); }\n",
    );
    for sym in ["?g3@@YAXXZ", "?g2@@YAXXZ", "?g1@@YAXXZ", "?f@@YAXXZ"] {
        let r = row(&rows, sym, "k4_chain_d4");
        assert_eq!(
            r.3, BLR,
            "c2's own COMDAT for `{sym}` is not a single `blr` — the premise the \
             fixpoint rests on has changed (work/w-fix/grid3.out grades all four \
             edges of k4_chain_d4 as E)"
        );
        assert!(
            tu.reduces_to_nothing(sym),
            "THE FIXPOINT DID NOT PROPAGATE: `{sym}` is a tail call to a name that \
             reduces to nothing and c2 emits nothing for it, but the closure did \
             not admit it"
        );
        assert_eq!(
            r.1,
            FnByte::Exact,
            "cell k4_chain_d4, `{sym}`: the port's body is not c2's. Verdict {:?}",
            r.1
        );
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// `k6_stop_d2` — a link whose body calls an external stops the chain. At
/// `/Ob0` every caller at or above it keeps its REL24 (`work/w-fix/grid3.out`),
/// so nothing above the break may be elided.
///
/// The byte verdict is deliberately **not** what this asserts: c2 emits
/// `b ?ext` for `?f` and the port emits `b ?g1`, which is the same word with a
/// different relocation target — `fnbyte-exact-relocated`, board #882. The
/// claim here is about the closure, which is the thing that could be wrong.
#[test]
fn a_non_empty_link_stops_the_chain() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("stop");
    let (rows, tu) = grade_cell(
        &tc,
        &d,
        "k6",
        "void ext();\nvoid h() { ext(); }\nvoid g1() { h(); }\nvoid f() { g1(); }\n",
    );
    let r = row(&rows, "?f@@YAXXZ", "k6_stop_d2");
    assert_ne!(
        r.3, BLR,
        "c2's `?f` in k6_stop_d2 IS a bare blr — the chain did not stop at the \
         non-empty body and this whole test is measuring the wrong cell"
    );
    for sym in ["?h@@YAXXZ", "?g1@@YAXXZ", "?f@@YAXXZ"] {
        assert!(
            !tu.reduces_to_nothing(sym),
            "THE FIXPOINT WAS APPLIED THROUGH A NON-EMPTY LINK: `{sym}` sits at or \
             above a body that calls an external, and GRID-3 grades every edge \
             there as mechanism I — the caller keeps its REL24 at /Ob0"
        );
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// `k10_cycle2` — `void a(){b();} void b(){a();}`. Neither member is a bare
/// `blr` in c2, and the closure must admit neither. **It must also
/// terminate**: a test that hangs here is the failure, not a red one.
#[test]
fn a_cycle_is_not_elided() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("cycle");
    let (rows, tu) = grade_cell(
        &tc,
        &d,
        "k10",
        "void b();\nvoid a() { b(); }\nvoid b() { a(); }\nvoid f() { a(); }\n",
    );
    for sym in ["?a@@YAXXZ", "?b@@YAXXZ", "?f@@YAXXZ"] {
        let r = row(&rows, sym, "k10_cycle2");
        assert_ne!(
            r.3, BLR,
            "c2's COMDAT for `{sym}` is a bare blr — c2 DOES reduce a cycle to \
             nothing, and the closure's refusal of it is now an under-fire to be \
             sized rather than a correctness matter"
        );
        assert!(
            !tu.reduces_to_nothing(sym),
            "A CYCLE WAS TREATED AS REDUCING TO NOTHING: `{sym}` is in a call \
             cycle that no empty body seeds, and c2 emits a branch for it"
        );
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// `k12_cross_i` — **the trap, one level up from `c19`.** `int m(int a){return
/// a;}` is mechanism I; at `/O1` c2 emits **both** `?g1` and `?f` as a bare
/// `blr`, so a fixpoint fitted to the emitted bytes would take the whole chain.
/// Only `/Ob0` separates them, and the port must decline on the BODY.
#[test]
fn mechanism_i_mid_chain_is_not_elided() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("crossi");
    let (rows, tu) = grade_cell(
        &tc,
        &d,
        "k12",
        "int m(int a) { return a; }\nint g1(int a) { return m(a); }\n\
         int f(int a) { return g1(a); }\n",
    );
    let r = row(&rows, "?f@@YAHH@Z", "k12_cross_i");
    assert_eq!(
        r.3, BLR,
        "c2's `?f` in k12_cross_i is no longer a bare blr — the cell's whole point \
         is that mechanism I is OBSERVATIONALLY IDENTICAL to E at /O1"
    );
    for sym in ["?m@@YAHH@Z", "?g1@@YAHH@Z", "?f@@YAHH@Z"] {
        assert!(
            !tu.reduces_to_nothing(sym),
            "THE FIXPOINT WAS APPLIED THROUGH A NON-EMPTY LINK: `{sym}`'s chain \
             bottoms out at `int m(int a){{return a;}}`, which GRID-3 k12 grades \
             mechanism I at both edges. Its bytes at /O1 are a bare blr and its \
             cause is not E — 2.8 % of a guess is a wrong emit"
        );
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// `k17_dtor_chain_d2` — **the cell where c2 does the fixpoint and the port
/// cannot even see it.**
///
/// GRID-3 grades both edges of this destructor chain `E`, and c2 emits `??1B`
/// and `??1C` as a bare `blr` each. The port converts **neither**: both bodies
/// are parse-refused as `expr-call-in-expr-recv-field-off0-whole` (the
/// inheritance spelling is refused as `…-recv-intrinsic-this-adjust-whole`),
/// so no row reaches the closure at all.
///
/// That is board #922's population one level up, and it is pinned here rather
/// than left as a paragraph: board #924's own 143 workload functions reach the
/// rule through a **different** production (`empty-dtor-member`, which this
/// hand-written chain does not produce), so a reader would otherwise assume
/// this cell is the family and that the family's IL looks like this. **If this
/// test goes red because the shapes now parse, the fixpoint gains a population
/// nothing here has graded end to end** — re-run `work/w-fix/grade3.py` and
/// check the byte verdicts before believing the gain.
#[test]
fn a_destructor_chain_is_elided_by_c2_and_refused_by_the_parser() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("dtor");
    let (rows, tu) = grade_cell(
        &tc,
        &d,
        "k17",
        "struct A { ~A() {} };\nstruct B { A a; ~B(); };\nstruct C { B b; ~C(); };\n\
         B::~B() {}\nC::~C() {}\n",
    );
    // The anchor control, through the same reader every other case uses.
    let _ = row(&rows, "?anchor@@YAXXZ", "k17_dtor_chain_d2");
    for pre in ["??1B@@", "??1C@@"] {
        let hits: Vec<_> = rows.iter().filter(|r| r.2.starts_with(pre)).collect();
        assert_eq!(
            hits.len(),
            1,
            "cell k17_dtor_chain_d2: expected exactly one emitted COMDAT named \
             `{pre}…`; got {:?}",
            rows.iter().map(|r| &r.2).collect::<Vec<_>>()
        );
        let r = hits[0];
        assert_eq!(
            r.3, BLR,
            "c2's `{}` is not a single blr — GRID-3 k17 grades both edges of this \
             destructor chain as E, and that grading is the premise here",
            r.2
        );
        assert_eq!(
            (r.0, r.1),
            ("parse-refused", FnByte::Refused),
            "`{}` PARSES now. c2 elides both links of this chain; if the port can \
             read them, the closure reaches a destructor-delegation population \
             that GRID-3 graded in c2 and no test has graded end to end in the \
             port. Got shape `{}` verdict {:?}",
            r.2, r.0, r.1
        );
        assert!(
            !tu.reduces_to_nothing(&r.2),
            "`{}` is in the closure although its own row did not parse — a name \
             admitted from a body nobody read",
            r.2
        );
    }
    let _ = std::fs::remove_dir_all(&d);
}
