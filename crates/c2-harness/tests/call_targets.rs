//! **A `/Gy` BRANCH WORD DOES NOT CARRY ITS CALLEE, and the relocation table
//! does** — `c2_obj::ObjImage::text_comdat_call_targets`, boards **#984**–#986,
//! lane `w-drop3`.
//!
//! # The claim, and why it needs the real compiler
//!
//! Under function-level linking every emitted function starts at offset 0 of its
//! own COMDAT, and c2 writes a call out of that COMDAT with the placeholder
//! displacement `-(offset of the branch word)` — **whatever the callee is**. So
//! two functions that call two entirely different symbols from the same word
//! index carry the *same four bytes*, and every byte-level instrument in this
//! repo scores that word equal. `docs/FUNCTION_BYTE_MATCH.md`'s
//! `fnbyte-exact-relocated` (board **#882**, 4,664 credited functions) is that
//! gap stated as a caveat; this reader is what lets it be counted.
//!
//! A synthetic obj can pin the *decoder* (`crates/c2-obj/src/lib.rs`'s unit
//! tests do, six of them). Only real `c2` can pin the *premise* — that it
//! really does emit the same bytes for different callees — so that is what this
//! file asserts, at the workload's own profile.
//!
//! # What each test is FOR
//!
//! | test | the claim, and what going red means |
//! |---|---|
//! | `two_byte_identical_comdats_call_two_different_symbols` | **the premise.** Two 4-byte COMDATs, byte-identical, different `REL24` targets. If this goes red the placeholder rule changed and every count keyed on it must be re-derived |
//! | `the_port_and_the_reference_agree_on_a_call_the_port_gets_right` | the **positive control**: the comparison runs and can come out green. A control that only ever fires is one whose silence means nothing |
//! | `the_ports_call_list_comes_from_the_emitter_and_not_from_a_copy` | `comdat_function_body`'s own `calls` is the port's side. A second walk over `IlFunction` would drift from the writer — mechanism E alone emits no branch and no `REL24` |
//!
//! Degrades to a printed `SKIP: toolchain absent` rather than failing, per
//! `CLAUDE.md`.

use std::path::{Path, PathBuf};

use c2_core::codegen::OptMode;
use c2_core::comdat::comdat_function_body;
use c2_reference::Toolchain;

/// The workload's own profile, minus the `/I` paths a standalone cell cannot
/// use. **`/O1` implies `/Gy`; `/Ox` does not** — a cell compiled at the
/// fixture default would produce a packed obj with no `.text` COMDAT at all and
/// every assertion below would pass vacuously.
/// PROV[N] not load-bearing — a MEASUREMENT PROFILE, named under [N] in DISCLOSURE: the compiler flag list this cell is captured and graded at. It selects which behaviour is observed; it is not a value read from c2.
const FLAGS: [&str; 8] = c2_harness::testsupport::WORKLOAD_FLAGS;

/// The cell. Three one-word bodies fall out of it at `/O1`:
///
/// ```text
///   ?anchor@@YAXXZ   48000000  b -> ?ext_anchor@@YAXXZ
///   ??1B@@QAA@XZ     48000000  b -> ?ext_clear@@YAXXZ
///   ??1D@@QAA@XZ     48000000  b -> ?ext_clear@@YAXXZ   (c2 inlined ~B into ~D)
/// ```
///
/// The first two are the pair this file is about: **identical bytes, different
/// callees.** The third is the mechanism this lane came from — c2 expanding a
/// same-TU callee — kept in the cell so the shape is on the record next to the
/// claim, even though the IL parser refuses it.
/// PROV[N] not load-bearing — a FIXTURE SOURCE string. It is INPUT to c2, graded by the byte judge against real c2's output; nothing about its value is derived from `c2.dll`. DISCLOSURE names fixture material under [N].
const CELL: &str = r#"
void ext_anchor();
void anchor() { ext_anchor(); }
void ext_clear();
struct B { ~B(); };
B::~B() { ext_clear(); }
struct D : B { };
void keep(D *p) { delete p; }
"#;

fn work(tag: &str) -> PathBuf {
    c2_harness::testsupport::scratch_dir("w-drop3", tag)
}

/// `(call targets by symbol, the capture)`. `None` when the capture or the
/// relocation walk failed — reported by the caller as a failure, never read as
/// an empty answer.
type Targets = Vec<(String, Vec<(u32, String)>)>;

fn capture(tc: &Toolchain, dir: &Path) -> Option<(Targets, Vec<(String, Vec<u8>)>, c2_reference::CapturedReference)> {
    let cpp = dir.join("ct.cpp");
    std::fs::write(&cpp, CELL).unwrap();
    let flags: Vec<String> = FLAGS.iter().map(|s| s.to_string()).collect();
    let src = c2_reference::to_wibo_path(&cpp);
    let cap = tc.capture_reference_with(&src, dir, &flags, None).ok()?;
    let targets = cap.ref_obj.text_comdat_call_targets()?;
    let bytes = cap.ref_obj.text_comdat_functions_with_bytes()?;
    Some((targets, bytes, cap))
}

fn find<'a, T>(v: &'a [(String, T)], name: &str) -> &'a T {
    v.iter()
        .find(|(n, _)| n == name)
        .map(|(_, t)| t)
        .unwrap_or_else(|| {
            panic!(
                "no COMDAT named `{name}`; got {:?}",
                v.iter().map(|(n, _)| n).collect::<Vec<_>>()
            )
        })
}

/// **THE PREMISE**, read off real `c2`: same bytes, different symbol.
#[test]
fn two_byte_identical_comdats_call_two_different_symbols() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let dir = work("premise");
    let Some((targets, bytes, _cap)) = capture(&tc, &dir) else {
        panic!("the capture or the relocation walk failed — no assertion below is meaningful");
    };
    let ab = find(&bytes, "?anchor@@YAXXZ");
    let bb = find(&bytes, "??1B@@QAA@XZ");
    assert_eq!(
        ab, bb,
        "the two bodies must be BYTE-IDENTICAL for this test to say anything: \
         a /Gy placeholder displacement is -(offset), and both branches sit at offset 0"
    );
    assert_eq!(ab.len(), 4, "each body is one branch word");
    let at = find(&targets, "?anchor@@YAXXZ");
    let bt = find(&targets, "??1B@@QAA@XZ");
    assert_eq!(at, &vec![(0u32, "?ext_anchor@@YAXXZ".to_string())]);
    assert_eq!(bt, &vec![(0u32, "?ext_clear@@YAXXZ".to_string())]);
    assert_ne!(
        at, bt,
        "identical bytes, different callees — this is the whole gap board #882 names, \
         and if it ever stops holding then a byte compare IS a call compare and this \
         instrument can be retired"
    );
}

/// The **positive control**. A control that can only ever fire is one whose
/// silence means nothing, so the agreeing case is asserted too — on the one
/// function in this cell that the port lowers and gets right.
#[test]
fn the_port_and_the_reference_agree_on_a_call_the_port_gets_right() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let dir = work("agree");
    let Some((targets, _bytes, cap)) = capture(&tc, &dir) else {
        panic!("the capture or the relocation walk failed");
    };
    let census = cap.bundle.census_functions().expect("census");
    let tu = c2_harness::gap::fnbytes::tu_empty_callees(&census);
    let (_, parsed) = census
        .iter()
        .find(|(c, _)| c.emit_name.as_deref() == Some("?anchor@@YAXXZ"))
        .expect("the anchor has a census row");
    let f = parsed.as_ref().expect("the anchor parses — it is a plain tail call");
    let body = comdat_function_body(f, OptMode::O1, &tu).expect("the port lowers the anchor");
    let port: Vec<(u32, String)> = body
        .calls
        .iter()
        .map(|c| (c.reloc_offset, c.callee.to_string()))
        .collect();
    assert_eq!(
        &port,
        find(&targets, "?anchor@@YAXXZ"),
        "the port's own REL24 list must equal real c2's for a function it gets right"
    );
}

/// The port's side is the **emitter's** `calls` list, not a second walk over
/// `IlFunction`. Asserted as an identity on the shape where the two would
/// differ: mechanism E emits no branch and no `REL24` at all, so a copy that
/// forgot the elision would report a call that is not in the obj.
#[test]
fn the_ports_call_list_comes_from_the_emitter_and_not_from_a_copy() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let dir = work("locator");
    let cpp = dir.join("e.cpp");
    std::fs::write(
        &cpp,
        "void ext_anchor();\nvoid anchor() { ext_anchor(); }\n\
         void g() {}\nvoid f() { g(); }\n",
    )
    .unwrap();
    let flags: Vec<String> = FLAGS.iter().map(|s| s.to_string()).collect();
    let src = c2_reference::to_wibo_path(&cpp);
    let Ok(cap) = tc.capture_reference_with(&src, &dir, &flags, None) else {
        panic!("capture failed");
    };
    let census = cap.bundle.census_functions().expect("census");
    let tu = c2_harness::gap::fnbytes::tu_empty_callees(&census);
    let targets = cap
        .ref_obj
        .text_comdat_call_targets()
        .expect("the relocation walk decodes");
    let (_, parsed) = census
        .iter()
        .find(|(c, _)| c.emit_name.as_deref() == Some("?f@@YAXXZ"))
        .expect("?f@@YAXXZ has a census row");
    let f = parsed.as_ref().expect("?f@@YAXXZ parses");
    assert!(
        f.tail_call().is_some(),
        "the IlFunction still NAMES a callee — that is exactly why reading the port's \
         call list off it instead of off the emitter would be wrong"
    );
    assert!(
        c2_core::elide::drops_tail_call(f, &tu),
        "mechanism E must fire on the empty-callee cell; if it does not, this test \
         is asserting nothing and elide.rs's rule has moved"
    );
    let body = comdat_function_body(f, OptMode::O1, &tu).expect("the port lowers ?f");
    assert!(
        body.calls.is_empty(),
        "the EMITTER emits no REL24 for an elided tail call, so the port's call list \
         must be empty even though IlFunction::tail_call is Some"
    );
    assert_eq!(
        find(&targets, "?f@@YAXXZ"),
        &Vec::<(u32, String)>::new(),
        "and real c2 agrees: its COMDAT for ?f carries no REL24 either"
    );
}
