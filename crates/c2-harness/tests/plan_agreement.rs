//! **THE AGREEMENT CONTROL** on `ObjImage::observe` — the new whole-object walk
//! against the four accessors that already exist, on objs the real toolchain
//! produced. Lane `w-objplan`.
//!
//! # Why a new walk plus an agreement assertion, and not a replacement
//!
//! `c2-obj` already carries four readers of facts the object plan re-reads:
//! `text_comdat_functions` (the emit set), `section_names`, `weak_externals`
//! and `text_comdat_relocs_named`. `docs/GAPS.md` §6's *one fact, one locator*
//! rule says a second copy of a walk is how derivations drift — and this crate
//! has the record to prove it, in `text_comdat_relocs_named`'s own doc: two
//! lanes added a resolving relocation reader within a day of each other, the
//! two files auto-merged with **no conflict marker**, and the crate briefly
//! carried two walks over one fact under one name.
//!
//! So why add one here rather than build the plan out of the four? Because
//! replacing them is a *construct* move that touches every consumer of every
//! one of them, in the same central files ARCHITECTURE_SEAMS §1.1 lists as
//! conflicting on **every** merge — and because the plan needs facts none of
//! them exposes (COMDAT selection, associativity, the symbol table's shape,
//! undefined externals), so it is a walk in any case. A new walk plus **this
//! file** is cheaper, uncontended, and catches exactly the drift the
//! one-locator rule exists to catch. The replacement is a later lane's work.
//!
//! # The population is stated, not implied (STATUS trap 0)
//!
//! Three cells, each chosen because it makes a *different* one of the four
//! comparisons non-vacuous, and every assertion below names which:
//!
//! | cell | what it makes non-vacuous |
//! |---|---|
//! | `GY` | several `.text` COMDATs — the emit set has ORDER and the section names REPEAT, so a name-keyed plan is forced to disambiguate |
//! | `WEAK` | a virtual destructor, which c2 realises as a `??_E`/`??_G` **weak external** pair (`weak_externals`' own doc) |
//! | `RELO` | calls to symbols this TU does not define — relocation records with resolvable targets |
//!
//! A green run here is a statement about those three shapes and nothing wider.
//! The **workload-wide** version of this control is the `c2rs gap` scan's
//! `plan-observable` count over all 870 graded TUs, which is where the decode
//! rate is actually measured.
//!
//! `SKIP: toolchain absent` when there is no toolchain, like every other
//! integration test here — and `C2RS_REQUIRE_TOOLCHAIN=1` is what turns that
//! SKIP into a failure for a caller who needs the grade (`require_toolchain.rs`).

use c2_harness::testsupport::{unique_scratch_dir, WORKLOAD_FLAGS};
use c2_obj::{ObjImage, ObjPlan};
use c2_reference::Toolchain;

/// The workload's own profile, minus the `/I` paths a standalone cell cannot
/// use. `/O1` implies `/Gy`, which is the regime the whole plan lives in — and
/// it is the ONLY mode in which the emit set exists at all, because `/Gy` is
/// what puts each function in its own COMDAT `.text`. At `/Ox` every assertion
/// in this file would compare two empty lists and pass.
///
/// **One spelling, from `c2_harness::testsupport`** (lane `w-refrev`): this file
/// carried the fifteenth copy of the literal until the fix round adopted the
/// funnel. A missed copy keeps grading the old profile and reads green, which is
/// the absence family wearing a flags list.
const FLAGS: [&str; 8] = WORKLOAD_FLAGS;

const GY: &str = "\
int a(int x) { return x + 1; }
int b(int x) { return x * 3; }
int c(int x) { return x - 7; }
";

const WEAK: &str = "\
struct B { virtual ~B(); virtual void f(); };
struct D : B { ~D(); void f(); };
D::~D() {}
void D::f() {}
B *make() { return new D(); }
";

const RELO: &str = "\
extern int g;
void ext();
void ext2(int);
void use() { ext(); ext2(g); }
int take() { return g; }
";

/// Per-CALL scratch, never per-process: `capture_reference_with` points
/// `TMP`/`TEMP` at the work dir, deletes every `_CL_*` in it and writes a fixed
/// `out.obj`, so a shared path is a write-write race whose symptom reads as a
/// port defect (`reloc_identity.rs`'s `work()`, board `w-gateperf`).
///
/// [`unique_scratch_dir`] is the funnel `w-refrev` built for exactly that — a
/// fresh directory per CALL, so two cells cannot alias even inside one test.
/// The per-file `static COUNTER` + `temp_dir()` pair this file used to carry was
/// the 22nd copy of it.
fn compile(tc: &Toolchain, name: &str, src: &str) -> Option<ObjImage> {
    let dir = unique_scratch_dir("w-objplan", name);
    let cpp = dir.join(format!("{name}.cpp"));
    std::fs::write(&cpp, src).unwrap();
    let flags: Vec<String> = FLAGS.iter().map(|s| s.to_string()).collect();
    let path = c2_reference::to_wibo_path(&cpp);
    tc.capture_reference_with(&path, &dir, &flags, None)
        .ok()
        .map(|c| c.ref_obj)
}

/// `observe` must decode every obj the four accessors decode — the plan's
/// denominator is exactly as honest as this.
fn plan_of(img: &ObjImage, cell: &str) -> ObjPlan {
    img.observe().unwrap_or_else(|| {
        panic!(
            "cell {cell}: `observe` refused an obj that real c2 produced. The plan \
             walk is fail-closed on the WHOLE object by design, so this is not a \
             short answer — it is the instrument's denominator collapsing, and \
             every ratio taken against it would be inflated."
        )
    })
}

/// **The four agreements**, run over every cell, each naming the accessor it
/// pairs with and what a disagreement would mean.
fn agree(img: &ObjImage, cell: &str) {
    let p = plan_of(img, cell);

    // 1. The emit set. `text_comdat_functions` is `docs/GAPS.md` §8's
    //    denominator — *what c2 actually compiled* — and it is the one number
    //    every emitted-census ratio is taken against.
    assert_eq!(
        p.emit_set,
        img.text_comdat_functions().expect("the emit-set walk decodes"),
        "cell {cell}: the plan's emit set disagrees with `text_comdat_functions`. \
         One of the two walks is wrong about which symbol leads a `.text` COMDAT, \
         and the section-led rule exists because the symbol-led one over-counts \
         2.35x on a real TU."
    );

    // 2. The section names, IN ORDER. `section_names` is factor C's input.
    assert_eq!(
        p.section_names(),
        img.section_names().expect("the section walk decodes"),
        "cell {cell}: the plan's section sequence disagrees with `section_names`. \
         Both must go through `section_name_at` — the `/NNN` string-table form and \
         the not-NUL-terminated 8-byte form are three chances for two readers to \
         differ, and ROADMAP §10.14 is the record of that costing a session."
    );

    // 3. Weak externals. c2 realises a `.gl` tag-0x10 ALIAS as a COFF weak
    //    external rather than as a substitution, so this list is the alias
    //    table's real obj-level observable.
    let weak: Vec<(String, String, u32)> = p
        .weak
        .iter()
        .map(|w| (w.weak.clone(), w.default.clone(), w.characteristics))
        .collect();
    assert_eq!(
        weak,
        img.weak_externals().expect("the weak-external walk decodes"),
        "cell {cell}: the plan's weak externals disagree with `weak_externals`."
    );

    // 4. Relocations, by (type, target), over the `.text` COMDATs only — which
    //    is the population `text_comdat_relocs_named` is defined over. The plan
    //    covers every section, so the comparison is restricted to the accessor's
    //    own domain rather than the accessor being blamed for a wider walk.
    //
    //    The plan drops the `VirtualAddress` on purpose (it is a body offset),
    //    so the pairing is on the two fields both sides carry.
    let named = img
        .text_comdat_relocs_named()
        .expect("the named-relocation walk decodes");
    for (leader, rows) in &named {
        let ours = p
            .relocs
            .iter()
            .find(|r| r.section.leader.as_deref() == Some(leader.as_str()))
            .unwrap_or_else(|| {
                panic!(
                    "cell {cell}: no plan relocation set for COMDAT leader {leader} — \
                     the plan keys sections by NAME and LEADER precisely so a `/Gy` \
                     obj's several `.text` sections stay distinguishable"
                )
            });
        let theirs: Vec<(u16, Option<String>)> =
            rows.iter().map(|(_, ty, t)| (*ty, t.clone())).collect();
        let mine: Vec<(u16, Option<String>)> = ours
            .entries
            .iter()
            .map(|(ty, t)| {
                (
                    *ty,
                    match t {
                        c2_obj::RelocTarget::Symbol(n) | c2_obj::RelocTarget::Section(n) => {
                            Some(n.clone())
                        }
                        c2_obj::RelocTarget::PairDisplacement(_) => None,
                    },
                )
            })
            .collect();
        assert_eq!(
            mine, theirs,
            "cell {cell}, COMDAT {leader}: the plan's relocation inventory disagrees \
             with `text_comdat_relocs_named`. A SHORT list on either side is the \
             dangerous answer — it reads as `this body relocates less than it does`."
        );
    }
}

#[test]
fn observe_agrees_with_every_existing_accessor_on_real_objs() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let mut graded = 0usize;
    for (name, src) in [("gy", GY), ("weak", WEAK), ("relo", RELO)] {
        let Some(img) = compile(&tc, name, src) else {
            println!("SKIP: cell {name} did not capture");
            continue;
        };
        agree(&img, name);
        graded += 1;
    }
    // **A run that graded nothing is a failure, not a pass** (`gate.sh`'s
    // `--require-graded` argument, applied to a test): the three SKIP branches
    // above are individually legitimate and collectively vacuous.
    assert_eq!(
        graded, 3,
        "the agreement control graded {graded} of 3 cells; a control that ran \
         over an empty population reports absence as success"
    );
}

/// **The population is non-vacuous, asserted rather than assumed.** Each cell
/// must actually exhibit the shape it was chosen for — otherwise the agreement
/// above is three comparisons of empty lists, which passes and measures
/// nothing. This is board #1140's caution: a marker count is only as good as
/// what it counts.
/// **The three cells must ALL be graded, and the counter is why.**
///
/// The first version of this test `return`ed on any per-cell capture failure
/// with no counter at all, so a run where every capture failed passed — unlike
/// its sibling above, which asserts 3 of 3 for exactly this reason and says so.
/// A test whose skip path and success path are the same exit is a test that
/// cannot distinguish them.
fn graded_all(n: usize, what: &str) {
    assert_eq!(
        n, 3,
        "{what} graded {n} of 3 cells; the per-cell SKIP branches are individually \
         legitimate and collectively vacuous — a control that ran over an empty \
         population reports absence as success"
    );
}

#[test]
fn each_cell_exhibits_the_shape_it_was_chosen_for() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let mut graded = 0usize;
    let Some(gy) = compile(&tc, "gy2", GY) else {
        println!("SKIP: cell gy did not capture");
        graded_all(graded, "each_cell_exhibits_the_shape_it_was_chosen_for");
        return;
    };
    graded += 1;
    let p = plan_of(&gy, "gy");
    assert!(
        p.emit_set.len() >= 3,
        "cell GY must produce several `.text` COMDATs or the ORDER half of the \
         emit-set comparison is vacuous; it produced {:?}",
        p.emit_set
    );
    let names = p.section_names();
    let text = names.iter().filter(|n| n.starts_with(".text")).count();
    assert!(
        text >= 3,
        "cell GY must REPEAT the section name `.text` or the name-keying is never \
         exercised; sections were {names:?}"
    );

    let Some(weak) = compile(&tc, "weak2", WEAK) else {
        println!("SKIP: cell weak did not capture");
        graded_all(graded, "each_cell_exhibits_the_shape_it_was_chosen_for");
        return;
    };
    graded += 1;
    let pw = plan_of(&weak, "weak");
    assert!(
        !pw.weak.is_empty(),
        "cell WEAK must carry at least one COFF weak external or the third \
         comparison is a comparison of two empty lists"
    );

    let Some(relo) = compile(&tc, "relo2", RELO) else {
        println!("SKIP: cell relo did not capture");
        graded_all(graded, "each_cell_exhibits_the_shape_it_was_chosen_for");
        return;
    };
    graded += 1;
    let pr = plan_of(&relo, "relo");
    assert!(
        pr.relocs.iter().any(|r| !r.entries.is_empty()),
        "cell RELO must carry relocations or the fourth comparison is vacuous"
    );
    assert!(
        !pr.undef.is_empty(),
        "cell RELO must reference undefined externals — that is the component the \
         `undef` list is for"
    );
    graded_all(graded, "each_cell_exhibits_the_shape_it_was_chosen_for");
}

/// **THE ORDERING DIVERGENCE, STATED AND FENCED.**
///
/// `ObjPlan::observe` builds its emit set in SECTION-table order;
/// `text_comdat_entries` walks the SYMBOL table. On these three objs the two
/// coincide, which is why the ordered `assert_eq!` in [`agree`] holds — and the
/// review is right that nothing else ever tested it. The two orders are NOT
/// guaranteed equal on a real TU, so the workload-scale agreement wired into
/// `gap::scan` compares them as SETS and says so.
///
/// This test pins the fact rather than leaving the ordered assertion to imply a
/// guarantee it does not have: on the `GY` cell (three COMDATs, so an order
/// exists to be wrong about) the two walks are asserted equal AS SETS
/// unconditionally, and the ordered equality is reported as a property of THIS
/// cell rather than of the walks.
#[test]
fn the_two_emit_set_walks_agree_as_sets_and_the_order_is_a_property_of_the_cell() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let Some(img) = compile(&tc, "gy4", GY) else {
        println!("SKIP: cell gy did not capture");
        panic!("the ordering fence graded 0 cells — see graded_all");
    };
    let p = plan_of(&img, "gy");
    let incumbent = img.text_comdat_functions().expect("the emit-set walk decodes");
    assert!(
        p.emit_set.len() >= 3,
        "the cell must carry several COMDATs or there is no order to be wrong \
         about; it carried {:?}",
        p.emit_set
    );
    let ours: std::collections::BTreeSet<&str> =
        p.emit_set.iter().map(String::as_str).collect();
    let theirs: std::collections::BTreeSet<&str> =
        incumbent.iter().map(String::as_str).collect();
    assert_eq!(
        ours, theirs,
        "the two emit-set walks disagree AS SETS — that is a membership defect in \
         one of them and it is what the workload-scale `plan-agree-emitset-disagree` \
         counter grades on every TU"
    );
    // The ORDER: reported, not asserted as a guarantee. Section order and symbol
    // order coincide here; a lane that needs the ordered form must establish it
    // for its own population.
    println!(
        "note: section-order vs symbol-order emit sets are {} on this cell",
        if p.emit_set == incumbent { "IDENTICAL" } else { "DIFFERENT" }
    );
}

/// **Body-independence on a REAL obj**, not only on the synthetic image the
/// unit test mutates: rewrite every `.text` COMDAT's raw bytes in an obj real
/// c2 produced and the plan must be identical.
///
/// The unit test in `c2-obj` proves the property against a hand-built image;
/// this proves the *walk* does not reach a body byte on the real section layout,
/// where the raw data is interleaved with relocation tables and the symbol
/// table sits after both.
#[test]
fn the_plan_does_not_move_when_a_real_objs_text_bytes_are_rewritten() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let Some(img) = compile(&tc, "gy3", GY) else {
        println!("SKIP: cell gy did not capture");
        // A capture failure here is NOT a pass: the test's whole content is the
        // mutation, and a run that mutated nothing proves nothing. `graded_all`
        // says so in one place for all three tests in this file.
        panic!("body-independence graded 0 of 1 cells — see graded_all");
    };
    let before = plan_of(&img, "gy");
    // Find each `.text` COMDAT's raw data by the accessor that already knows
    // where it is, then overwrite it in a copy.
    let entries = img
        .text_comdat_functions_with_bytes()
        .expect("the byte walk decodes");
    let mut bytes = img.as_bytes().to_vec();
    let mut rewritten = 0usize;
    let mut ambiguous = 0usize;
    for (_, body) in &entries {
        if body.is_empty() {
            continue;
        }
        // **EXACTLY ONE OCCURRENCE, OR SKIP THIS BODY.** The first version took
        // `position(..)` — the FIRST match — and reasoned that a look-alike run
        // is "still inside some section's raw data". That is not good enough: a
        // look-alike could sit in `.debug$S` or in a relocation table, and then
        // the test would XOR a region that is not the body it means to mutate
        // and still satisfy `rewritten > 0`. So an ambiguous body is skipped and
        // COUNTED, and the assertion below is on the unambiguous ones.
        let hits: Vec<usize> = bytes
            .windows(body.len())
            .enumerate()
            .filter(|(_, w)| *w == body.as_slice())
            .map(|(i, _)| i)
            .collect();
        if hits.len() != 1 {
            ambiguous += 1;
            continue;
        }
        let at = hits[0];
        for b in &mut bytes[at..at + body.len()] {
            *b ^= 0xFF;
        }
        rewritten += 1;
    }
    assert!(
        rewritten > 0,
        "the test must actually have rewritten a body, or it proves nothing \
         ({ambiguous} body/bodies were skipped as ambiguous — they occur more \
         than once in the file and XOR-ing the first hit could have mutated \
         something other than the body)"
    );
    let mutated = ObjImage::new(bytes);
    assert_ne!(img.as_bytes(), mutated.as_bytes());
    assert_eq!(
        before,
        plan_of(&mutated, "gy-mutated"),
        "the object plan moved when only `.text` raw bytes changed — invariant 1 \
         (body-independence) is broken, and every `plan-*` figure would then be \
         partly a body comparison wearing a structural name"
    );
}
