//! **Grade one frozen `.cpp` cell against real `c2` under wibo** — the shared
//! half of `destroy_loop_elision.rs` and `nothing_seed.rs`.
//!
//! # Why this is a module and not a fourth copy
//!
//! `empty_elision.rs`, `dead_temp_elision.rs` and `destroy_loop_elision.rs` each
//! grew their own `grade_cell`, because each lane wrote one and integration test
//! targets do not share code by default. `w-relo`'s merge is what that costs:
//! **two lanes wrote the same reader in different files, auto-merged with no
//! conflict marker, and the duplicate walks were caught only by a compile
//! error.** Independent invention never produces a conflict, so the count only
//! ever goes up.
//!
//! Board **#1053** would have made it four. It makes it *one shared module and
//! two remaining copies* instead: `destroy_loop_elision.rs` is migrated here in
//! the same commit (this lane is already its editor), and `empty_elision.rs` and
//! `dead_temp_elision.rs` are **deliberately not** — their ANCHOR placement
//! differs (`w-empty` appends where the template cells prepend, `w-inl0` §4), and
//! migrating a peer lane's pinned test to prove a point this rung does not need
//! is how a merge funnel acquires a regression it cannot attribute. Board
//! **#1094** carries the migration.
//!
//! # What a caller gets, and the two controls it must not skip
//!
//! [`grade_cell`] returns one row per emitted `.text` COMDAT —
//! `(shape, verdict, symbol, reference bytes, reference relocation count)` — and
//! the bundle's [`TuEmptyCallees`]. Both are needed and neither substitutes for
//! the other: the verdict says what the PORT did and `reduces_to_nothing` says
//! what the FIXPOINT believes, and a rule can be wrong with those two agreeing
//! only if the judge's own bytes are also checked, which is why the reference
//! bytes are in the tuple rather than compared away inside.
//!
//! * **The ANCHOR** — [`row`] refuses to return anything from a capture whose
//!   `?anchor@@YAXXZ` is not `Exact`. Without it *"the port emitted no branch"*
//!   and *"nothing in this cell emitted anything"* are the same observation.
//! * **`/Ob0`** — `extra` is the flag list appended to the workload's profile.
//!   Mechanism E is not governed by `/Ob` and mechanism I is, and at `/O1` a
//!   mid-chain inline is a bare `blr` at every level (`w-fix` #954), so a cell
//!   graded at one setting has not said which mechanism it measured.

#![allow(dead_code)] // each test target uses a subset; a shared module always does

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use c2_core::elide::TuEmptyCallees;
use c2_harness::gap::fnbytes::{grade_one, tu_empty_callees, FnByte};
use c2_reference::Toolchain;

/// The workload's own profile, minus the `/I` paths a standalone cell cannot
/// use. `/O1` implies `/Gy`; `/Ox` does not.
pub const FLAGS: [&str; 8] = [
    "/nologo", "/wd4355", "/wd4164", "/c", "/GR", "/O1", "/Oi", "/EHsc",
];

/// `w-empty`'s ANCHOR, **prepended** — a callee this TU does not define, whose
/// relocation must survive.
///
/// Prepended and not appended for `w-inl0` §4's measured reason: these cells
/// define templates, and a template instantiation's segment is emitted after every
/// source-order function, so an appended anchor lands on the module trailer that
/// `eat_fn_tail` refuses. Appended, the ANCHOR came back `Refused` in four of eight
/// cells and every verdict under it was worthless.
pub const ANCHOR: &str = "\nvoid ext_anchor();\nvoid anchor() { ext_anchor(); }\n";

/// **The TAIL PAD**, appended after every cell — scaffolding for the controls, not
/// part of any cell.
///
/// The `.ex`'s *last* function segment always refuses as `module-end-0x4D`, and in
/// a five-function cell that would be the empty leaf the whole chain has to be
/// seeded from. A one-level pad was not enough — instantiations are not emitted in
/// source order — and five is what the census reads back as `empty-body`
/// (`w-inl0` §4, measured rather than guessed).
pub const TAIL: &str = "
template <class T> inline T pad5(T v) { return v; }
template <class T> inline T pad4(T v) { return pad5(v); }
template <class T> inline T pad3(T v) { return pad4(v); }
template <class T> inline T pad2(T v) { return pad3(v); }
template <class T> inline T pad1(T v) { return pad2(v); }
int pad_use(int v) { return pad1(v); }
";

/// The one word an elided body is, and the one word c2 emits for a function it
/// emits nothing for.
pub const BLR: [u8; 4] = [0x4e, 0x80, 0x00, 0x20];

/// `(shape, verdict, symbol, reference bytes, reference relocation count)` per
/// emitted `.text` COMDAT.
pub type Rows = Vec<(&'static str, FnByte, String, Vec<u8>, usize)>;

/// **A scratch directory of this cell's own.**
///
/// Keyed on the tag *and* the pid. Board **#1045**: a lane this week gave four
/// parallel tests one PID-keyed directory, the captures raced, and it **fabricated
/// a finding that would have reversed its conclusion**. `cargo test` runs the
/// tests in a target concurrently by default, so the tag is not optional.
pub fn work(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("c2rs-cell-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Grade one frozen cell through **`grade_one`** — the same function the 878-TU
/// scan runs, so a cell and the workload are scored by one instrument.
///
/// `extra` is appended to the profile: `["/Ob0"]` is the E-versus-I separator and
/// `[]` is the workload's own setting. A capture that fails returns empty rows
/// rather than panicking, so a toolchain that is present but cannot compile this
/// cell shows up as [`row`]'s "no anchor" panic with the function list in it.
pub fn grade_cell(
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
    // Keyed on `emit_name` and never on `IlFunction::mangled_name`: the two
    // disagree on 74,955 workload rows (#918), and a name-keyed fact read through
    // the wrong binding is attached to the wrong function.
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
pub fn row<'a>(
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

/// The same as [`row`] but tolerant of absence: `None` when no symbol matches.
///
/// A COMDAT that is not in the obj at all and one whose verdict is wrong are
/// different observations, and a cell whose subject c2 did not emit needs to say
/// so rather than panic — `fnbyte-noeffect-ref-absent` is 1,046 rows on the
/// workload for exactly this reason.
pub fn row_opt<'a>(
    rows: &'a Rows,
    needle: &str,
    cell: &str,
) -> Option<&'a (&'static str, FnByte, String, Vec<u8>, usize)> {
    let _ = row_anchor(rows, cell);
    let hits: Vec<&(&'static str, FnByte, String, Vec<u8>, usize)> =
        rows.iter().filter(|r| r.2.contains(needle)).collect();
    match hits.as_slice() {
        [one] => Some(one),
        [] => None,
        _ => panic!(
            "cell `{cell}`: `{needle}` matches {} of the emitted symbols {:?}",
            hits.len(),
            rows.iter().map(|r| &r.2).collect::<Vec<_>>()
        ),
    }
}

/// The ANCHOR control on its own, so a cell can assert it before doing anything
/// else and a caller cannot reach a verdict without having paid for it.
pub fn row_anchor<'a>(rows: &'a Rows, cell: &str) -> &'a (&'static str, FnByte, String, Vec<u8>, usize) {
    match rows.iter().find(|r| r.2 == "?anchor@@YAXXZ") {
        Some(a) => {
            assert_eq!(
                a.1,
                FnByte::Exact,
                "cell `{cell}`: the ANCHOR control is not Exact — this capture \
                 graded nothing trustworthy, so no verdict below it means anything"
            );
            a
        }
        None => panic!(
            "cell `{cell}`: no `?anchor@@YAXXZ` COMDAT in the reference obj — the \
             capture produced {} functions and none of them is the control",
            rows.len()
        ),
    }
}

/// Render a COMDAT's words as hex — **printed beside every verdict**.
///
/// Board **#950**: `void r(){r();}` emits a self-branch that takes no relocation
/// at all, so the relocation observable reads "nothing happened" on a body that is
/// plainly not nothing. A grid scored on relocation counts calls that E. The bytes
/// are the only thing that does not.
pub fn hex(bytes: &[u8]) -> String {
    bytes
        .chunks(4)
        .map(|w| w.iter().map(|b| format!("{b:02x}")).collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}
