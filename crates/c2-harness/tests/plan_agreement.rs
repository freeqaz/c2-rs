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

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use c2_obj::{ObjImage, ObjPlan};
use c2_reference::Toolchain;

/// The workload's own profile, minus the `/I` paths a standalone cell cannot
/// use. `/O1` implies `/Gy`, which is the regime the whole plan lives in.
const FLAGS: [&str; 8] = [
    "/nologo", "/wd4355", "/wd4164", "/c", "/GR", "/O1", "/Oi", "/EHsc",
];

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
static WORK_SEQ: AtomicUsize = AtomicUsize::new(0);

fn work() -> PathBuf {
    let n = WORK_SEQ.fetch_add(1, Ordering::Relaxed);
    let d = std::env::temp_dir().join(format!("c2rs-w-objplan-{}-{n}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn compile(tc: &Toolchain, name: &str, src: &str) -> Option<ObjImage> {
    let dir = work();
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
#[test]
fn each_cell_exhibits_the_shape_it_was_chosen_for() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let Some(gy) = compile(&tc, "gy2", GY) else {
        println!("SKIP: cell gy did not capture");
        return;
    };
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
        return;
    };
    let pw = plan_of(&weak, "weak");
    assert!(
        !pw.weak.is_empty(),
        "cell WEAK must carry at least one COFF weak external or the third \
         comparison is a comparison of two empty lists"
    );

    let Some(relo) = compile(&tc, "relo2", RELO) else {
        println!("SKIP: cell relo did not capture");
        return;
    };
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
        return;
    };
    let before = plan_of(&img, "gy");
    // Find each `.text` COMDAT's raw data by the accessor that already knows
    // where it is, then overwrite it in a copy.
    let entries = img
        .text_comdat_functions_with_bytes()
        .expect("the byte walk decodes");
    let mut bytes = img.as_bytes().to_vec();
    let mut rewritten = 0usize;
    for (_, body) in &entries {
        if body.is_empty() {
            continue;
        }
        // The body's bytes are unique enough in practice; if they are not, the
        // first occurrence is still inside some section's raw data and the
        // property under test is unchanged.
        if let Some(at) = bytes
            .windows(body.len())
            .position(|w| w == body.as_slice())
        {
            for b in &mut bytes[at..at + body.len()] {
                *b ^= 0xFF;
            }
            rewritten += 1;
        }
    }
    assert!(
        rewritten > 0,
        "the test must actually have rewritten a body, or it proves nothing"
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
