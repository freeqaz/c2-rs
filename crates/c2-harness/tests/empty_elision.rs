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
//! | `a_returning_callee_is_not_elided` | the **emptiness** condition. `int g(int a){return a;}` is mechanism I, and at `/O1` its caller is observationally identical to an E caller |
//! | `a_callee_this_tu_does_not_define_is_not_elided` | the **same-TU** condition |
//! | `an_indirect_call_site_is_still_refused_by_the_il_parser` | **THE HAZARD.** E does not fire through a function pointer even with an empty callee, and the port is safe only because the parser refuses that caller. If this test goes red because the production now parses, `elide.rs` needs an explicit site condition **before** the elision may run |

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

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
fn grade_cell(tc: &Toolchain, dir: &Path, name: &str, body: &str) -> Vec<(&'static str, FnByte, String, Vec<u8>)> {
    let cpp = dir.join(format!("{name}.cpp"));
    std::fs::write(&cpp, format!("{body}{ANCHOR}")).unwrap();
    let flags: Vec<String> = FLAGS.iter().map(|s| s.to_string()).collect();
    let src = c2_reference::to_wibo_path(&cpp);
    let Ok(cap) = tc.capture_reference_with(&src, dir, &flags, None) else {
        return Vec::new();
    };
    let (Some(census), Some(entries)) = (
        cap.bundle.census_functions(),
        cap.ref_obj.text_comdat_functions_with_bytes(),
    ) else {
        return Vec::new();
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
    out
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
    let rows = grade_cell(&tc, &d, "c00", "void g() {}\nvoid f() { g(); }\n");
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
    let rows = grade_cell(&tc, &d, "g05", "void g(int a) {}\nvoid f() { g(5); }\n");
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

#[test]
fn a_returning_callee_is_not_elided() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("returns");
    // `c19_ret_param`. c2's `?f` here IS a bare `blr` too — but by mechanism I,
    // which `/Ob0` separates (`docs/INLINE_PREDICATE.md` §1) and which this port
    // does not model. The predicate must decline on the BODY, not on the bytes.
    let rows = grade_cell(
        &tc,
        &d,
        "c19",
        "int g(int a) { return a; }\nint f(int a) { return g(a); }\n",
    );
    let r = row(&rows, "?f@@YAHH@Z", "c19_ret_param");
    assert!(
        matches!(r.1, FnByte::Differs { .. }),
        "THE EMPTINESS CONDITION WAS DROPPED: `int g(int a){{return a;}}` has a \
         non-empty IL body and is mechanism I, not E. Its caller came back {:?}, \
         which means the port emitted `blr` for a callee whose body it has not \
         established does nothing — the same bytes for the wrong reason, and the \
         next callee like it will be wrong bytes",
        r.1
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
    let rows = grade_cell(&tc, &d, "c22", "void g();\nvoid f() { g(); }\n");
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
        let rows = grade_cell(&tc, &d, cell, src);
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
