//! **`__declspec(noinline)` — the chain c2 does NOT close** (lane `w-target`,
//! board **#1037**–**#1039**).
//!
//! `w-splice` established that c2 closes a splice chain and shipped
//! SPLICE-0-PORT on it; `w-relo` left **861** byte-exact bodies whose relocation
//! names the wrong function, and the standing hypothesis for #1013 is that c2's
//! target is the port's target closed under the inline relation. On the 878-TU
//! workload that closure converts **158** of the 861 and fires on **zero** of
//! the 3,803 relocating bodies the judge credits today.
//!
//! **This file is the counterexample that stops it.** `__declspec(noinline)` is
//! a chain c2 declines to close. The port could not read the attribute when this
//! file was written and can since lane `w-mmioclose` — the verdict column below
//! is the *current* one and the history of each cell is in its own note:
//!
//! | cell | port | c2 | verdict |
//! |---|---|---|---|
//! | `w04a` — `noinline` intermediate, caller `?f` | `b ?g` | `b ?g` | **`Exact`** — see the 2026-08-09 entries below: `Refused` for one day, then graded again |
//! | `w04a` — the intermediate `?g` itself | `b ?ext` | `b ?ext` | `Exact` |
//! | `w10` — `noinline` LEAF | `b ?g` | `b ?g` | **`Exact`** since `w-mmioclose` — it read `Differs (2, 1, 0)` from the day it was written until the attribute was decoded |
//! | `w12` — `w10` without the attribute | the callee's body | the callee's body | `Exact` — the control |
//!
//! # 2026-08-09, lane `w-inlfence2` — `w04a`'s caller moved `Exact` → `Refused`
//!
//! The inline fence (`c2_core::comdat::fenced_inlined_callee`) refuses a
//! composed body that emits a `REL24` against a same-TU callee whose lowered
//! body is at most `INLINE_UNBOUNDED_BYTES`. `?g` qualifies on size — and
//! `__declspec(noinline)` is exactly the field that makes the prediction wrong,
//! and exactly the field board **#1039** measured to be undecoded. So this cell
//! is the fence's **measured reach cost**, and it is one function.
//!
//! It is in the direction that cannot be a wrong emit: a mis-predicted *"c2
//! inlines this"* makes the port **decline**. On the 878-TU workload the cost is
//! **0** (`work/w-inlfence2/crossing.md` §4). The counterexample this file exists
//! for is unaffected and is now asserted **against c2's own relocation table**
//! rather than inferred from the port agreeing — which is stronger, because a
//! verdict of `Exact` never said *what* the two sides agreed on.
//!
//! # 2026-08-09, lane `w-mmioclose` — BOTH open items closed, and it is ONE field
//!
//! `__declspec(noinline)` clears **bit 0x40** of the attribute byte in the `.gl`
//! FUNCTION record, three fields past the body-start offset
//! (`c2_il::func::gl::gl_function_attrs`). That is board **#1039**'s undecoded
//! field, and 0x40 is bit 6 — the bit `WB_INLINE_FINDINGS.md` §1 read off c2's
//! own legality test at `0x10b5c06b` before this byte was located.
//!
//! Two consequences land here in the same commit:
//!
//! * `w04a`'s `?f` goes **`Refused` → `Exact`**: `comdat::callee_is_one_c2_expands`
//!   stops predicting an expansion c2 does not perform, so the fence's measured
//!   reach cost is back to **zero**.
//! * `w10` goes **`Differs (2, 1, 0)` → `Exact`**: `splice_body_why` gains
//!   `S7-callee-noinline`, and declining the splice leaves `Selected::Tail` to
//!   emit the branch c2 emits. **Board #1038 is closed.**
//!
//! The note below stands unchanged and is the reason both tests are kept rather
//! than deleted: the corpus still does not exercise either shape.
//!
//! # The attribute is present in the workload and the exposure is LATENT, not live
//!
//! `src/lazer/game/BustAMovePanel.cpp` is TU #4 of the 878 and carries three
//! `__declspec(noinline)` functions. None of them is a body the splice reaches,
//! so the workload reads `fnbyte-spliced 723 / -spliced-exact 723 / 0 differ`.
//! **`w10` is therefore a demonstrated defect that this corpus does not
//! exercise** — which is exactly the shape `CLAUDE.md`'s coverage-bounded rule
//! warns about, and the reason it is pinned here rather than left to a scan.
//!
//! No obj ships wrong either way: `IlBundle::functions()` refuses any TU where a
//! callee is also defined, which is every TU either mechanism can fire in, so
//! `mismatch` is 0 at both ends.
//!
//! `SKIP: toolchain absent` when there is no toolchain, like every other
//! integration test here.

use std::collections::BTreeMap;
use std::path::PathBuf;

use c2_harness::gap::fnbytes::{grade_one, tu_empty_callees, FnByte};
use c2_reference::Toolchain;

/// The workload's own profile, minus the `/I` paths a standalone cell cannot
/// use. `/O1` implies `/Gy`, which is the regime FBM's denominator lives in.
const FLAGS: [&str; 8] = [
    "/nologo", "/wd4355", "/wd4164", "/c", "/GR", "/O1", "/Oi", "/EHsc",
];

/// **GRID-W cell `w04a`** — `work/w-target/cells/w04a_noinline.cpp`. Kept as a
/// literal rather than read from `work/`, which is gitignored: a test that
/// silently skips because its input is not checked in is a test that reports
/// absence as success.
const W04A: &str = "\
void ext();
__declspec(noinline) void g() { ext(); }
void f() { g(); }

void ext_anchor();
void anchor() { ext_anchor(); }
";

/// **GRID-W2 cell `w10`** — the same attribute on a leaf the splice DOES reach.
const W10: &str = "\
int gsink;
__declspec(noinline) int g(int a) { return a + 1; }
int f(int a) { return g(a); }

void ext_anchor();
void anchor() { ext_anchor(); }
";

/// **GRID-W2 cell `w12`** — `w10` without the attribute. The negative control,
/// in its own TU: a grid with only the suspicious cell cannot tell "the rule is
/// wrong here" from "the rule is off in this build".
const W12: &str = "\
int gsink;
int g(int a) { return a + 1; }
int f(int a) { return g(a); }

void ext_anchor();
void anchor() { ext_anchor(); }
";

/// **One directory per cell, and that is not tidiness.** The first version of
/// this file gave every test one shared directory keyed on the process id;
/// `cargo test` runs the four in parallel threads of the *same* process, so the
/// captures raced and `?f@@YAXXZ` was graded against another cell's obj. The
/// failure presented as *"the attribute is visible in `.ex`"* — a false finding
/// that would have reversed this lane's conclusion. `work/w-target/nicmp2.sh`
/// re-measured it serially: `.ex`, `.sy`, `.in` and `.db` are byte-identical on
/// **both** shapes and only `.gl` moves, by 2 bytes.
///
/// Recorded as a defect of this file's first version, not as a design.
fn work(cell: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("c2rs-w-target-{}-{cell}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Grade every emitted `.text` COMDAT of one cell on the FULL identity — bytes
/// **and** relocations — through the scan's own `grade_one`, never a copy.
fn grade(tc: &Toolchain, tag: &str, src_text: &str) -> Vec<(&'static str, FnByte, String)> {
    grade_with_targets(tc, tag, src_text).0
}

/// [`grade`] plus **c2's own REL24 targets, by COMDAT and by name**.
///
/// Added by lane `w-inlfence2` so `w04a`'s counterexample can be asserted against
/// the reference obj directly rather than inferred from the port agreeing with
/// it. A verdict of `Exact` says the two sides match; it does not say *what*
/// they matched on, and the fact this cell exists to pin is a fact about **c2**.
fn grade_with_targets(
    tc: &Toolchain,
    tag: &str,
    src_text: &str,
) -> (Vec<(&'static str, FnByte, String)>, Vec<(String, Vec<String>)>) {
    let dir = work(tag);
    let cpp = dir.join(format!("{tag}.cpp"));
    std::fs::write(&cpp, src_text).unwrap();
    let flags: Vec<String> = FLAGS.iter().map(|s| s.to_string()).collect();
    let src = c2_reference::to_wibo_path(&cpp);
    let Ok(cap) = tc.capture_reference_with(&src, &dir, &flags, None) else {
        return (Vec::new(), Vec::new());
    };
    let (Some(census), Some(entries)) = (
        cap.bundle.census_functions(),
        cap.ref_obj.text_comdat_functions_with_bytes(),
    ) else {
        return (Vec::new(), Vec::new());
    };
    let mut claim: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (i, (f, _)) in census.iter().enumerate() {
        // #918: the binding is `emit_name`, never `mangled_name`.
        if let Some(n) = f.emit_name.as_deref() {
            claim.entry(n).or_default().push(i);
        }
    }
    let tu = tu_empty_callees(&census);
    let rel = cap.ref_obj.text_comdat_relocs();
    assert!(
        rel.is_some(),
        "the reference obj's .text relocation table did not decode — no verdict \
         in this cell means anything, and reading that as a pass is exactly the \
         defect `w-relo` closed"
    );
    let mut out = Vec::new();
    for (idx, (sym, bytes)) in entries.iter().enumerate() {
        let row = match claim.get(sym.as_str()).map(Vec::as_slice) {
            Some([i]) => Some(&census[*i]),
            _ => None,
        };
        let rr = rel.as_ref().and_then(|v| v.get(idx)).map(|(_, r)| r.as_slice());
        let g = grade_one(row, Some(bytes.as_slice()), &tu, rr);
        out.push((g.shape, g.verdict, sym.clone()));
    }
    // c2's REL24 targets per COMDAT, by NAME (#644/#918's rule — never an index
    // and never a position). `None` is a decode failure and is returned as an
    // empty list, which every caller asserts against rather than reading as
    // agreement.
    let targets: Vec<(String, Vec<String>)> = cap
        .ref_obj
        .text_comdat_call_targets()
        .unwrap_or_default()
        .into_iter()
        .map(|(n, v)| (n, v.into_iter().map(|(_, t)| t).collect()))
        .collect();
    (out, targets)
}

fn find<'a>(
    rows: &'a [(&'static str, FnByte, String)],
    sym: &str,
) -> &'a (&'static str, FnByte, String) {
    rows.iter().find(|r| r.2 == sym).unwrap_or_else(|| {
        panic!(
            "no `{sym}` COMDAT in the reference obj — the capture produced {} \
             functions and none of them is the one this test grades: {:?}",
            rows.len(),
            rows.iter().map(|r| &r.2).collect::<Vec<_>>()
        )
    })
}

/// **THE COUNTEREXAMPLE, now asserted on c2's OWN OBJ.** `?f` calls a
/// `noinline` `?g`; c2 obeys the attribute and emits `b ?g` rather than closing
/// the chain to `?ext`. That is `w-target`'s registered unconditional stop
/// against #1013's closure rule, and it is still true and still checked — but it
/// is checked HERE now, against the reference obj's own relocation, instead of
/// being read off the port's verdict.
///
/// # Why the assertion moved (2026-08-09, lane `w-inlfence2`)
///
/// `?f@@YAXXZ` used to be **`Exact`**: the port emitted the same `b ?g`. It is
/// **`Refused`** now, and the refusal is `ComdatDecline::InlinedCallee` — the
/// fence that stops the port emitting a `bl` to a same-TU callee c2 expands.
/// `?g` is defined here and its lowered body is 4 bytes, so the fence's
/// predicate (`<= INLINE_UNBOUNDED_BYTES`) holds and it fires. **The attribute
/// is what makes that prediction wrong, and the port cannot read it**: board
/// **#1039** measured the discriminator to be an undecoded two-byte `.gl` field
/// — `.ex`, `.sy`, `.in` and `.db` are byte-identical across a matched pair, and
/// the per-function optimization word is `00a00005` either way — so no clause
/// over the body IL or the opt word can separate the two cases.
///
/// **This is the fence's reach cost, and it is in the safe direction.** A
/// mis-predicted *"c2 inlines this"* makes the port **decline** a function it
/// would have got right; it can never make it emit a wrong byte. Measured on the
/// 878-TU workload the cost is **zero** — `xw-fence-fires|fnbyte-exact` = 0,
/// `work/w-inlfence2/crossing.md` §4 — because the three `noinline` functions in
/// the corpus (`src/lazer/game/BustAMovePanel.cpp`, TU #4) are not bodies either
/// mechanism reaches. On this cell it is one function, and it is recorded rather
/// than absorbed.
///
/// **When the port learns to read the `.gl` field, this test goes red** and the
/// fixing commit restores `Exact`. That is the same contract
/// `the_shipped_splice_emits_the_wrong_body_through_a_noinline_callee` carries,
/// with the sign flipped: that one pins a wrong EMIT, this one pins a
/// conservative REFUSAL.
#[test]
fn c2_does_not_close_a_chain_through_a_noinline_intermediate() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let (rows, targets) = grade_with_targets(&tc, "w04a", W04A);
    if rows.is_empty() {
        println!("SKIP: capture produced no graded function");
        return;
    }
    // **THE STOP ITSELF, read off c2's obj and not off a port verdict.** This is
    // strictly stronger evidence than the assertion it replaces: the old form
    // could only say "the port and c2 agree", which a change on either side
    // moves. This says what c2 DID.
    let f_targets = targets
        .iter()
        .find(|(n, _)| n == "?f@@YAXXZ")
        .map(|(_, t)| t.clone())
        .expect("no `?f@@YAXXZ` in the reference obj's REL24 table");
    assert_eq!(
        f_targets,
        vec!["?g@@YAXXZ".to_string()],
        "c2's `?f` must branch to the `noinline` `?g` and NOT close the chain to \
         `?ext`. If this ever reads `?ext`, c2 has started closing this chain and \
         #1013's closure rule is unblocked — which is the whole reason this cell \
         exists (board #1037)"
    );
    // …and the port's side, back at `Exact` since lane `w-mmioclose`.
    let f = find(&rows, "?f@@YAXXZ");
    assert_eq!(
        f.1,
        FnByte::Exact,
        "`?f@@YAXXZ` is graded again. `?g` is defined in this TU and lowers to 4 \
         bytes, so the SIZE half of `comdat::callee_is_one_c2_expands` predicts \
         c2 expands it — and `__declspec(noinline)` is what makes that \
         prediction wrong. This is the line the 2026-08-09 note below said would \
         have to move: the field is `c2_il::func::gl::FN_FLAG_INLINABLE`, the \
         fence reads it, and the reach cost is back to zero"
    );
    // The inverse, on the same obj: without the attribute in the way, `?g`'s own
    // branch to the external is right on both sides. A cell where everything is
    // Refused cannot distinguish "the fence fired" from "the grader is asleep",
    // so the anchor is asserted too — and `?g` calls only an EXTERNAL, which is
    // the fence's N1 clause and must leave it alone.
    for sym in ["?g@@YAXXZ", "?anchor@@YAXXZ"] {
        let r = find(&rows, sym);
        assert_eq!(
            r.1,
            FnByte::Exact,
            "`{sym}` is byte- and relocation-exact — it calls an EXTERNAL, which \
             the fence must never touch"
        );
    }
}

/// **BOARD #1038, CLOSED — and this is the cell that was red on purpose until
/// it was.**
///
/// Until lane `w-mmioclose` the shipped `SPLICE-0-PORT` fired through this
/// `noinline` leaf and emitted the callee's body where c2 emits a branch: two
/// words against one, **zero words equal**, and no relocation where c2 emits a
/// `REL24` against `?g`. The assertion below said so, in those numbers, and
/// said in terms that the commit which fixed it had to come here — *"do not
/// delete it"*.
///
/// It is fixed by `c2_core::splice`'s `S7-callee-noinline`, which reads
/// `c2_il::func::gl::FN_FLAG_INLINABLE` — board **#1039**'s undecoded `.gl`
/// field, decoded. **The wrong emit is now byte-exact**, because declining to
/// splice leaves `Selected::Tail` to emit the `b ?g` c2 emits.
///
/// The test is kept and inverted rather than deleted, for the reason it was
/// written: the 878-TU workload does not exercise this shape
/// (`fnbyte-spliced-exact` is 723 of 723), so a regression here would be
/// invisible to every scan.
#[test]
fn a_noinline_callee_is_no_longer_spliced_through() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let (rows, targets) = grade_with_targets(&tc, "w10", W10);
    if rows.is_empty() {
        println!("SKIP: capture produced no graded function");
        return;
    }
    // **What c2 DID, off its own relocation table** — the same strengthening
    // `w04a` took above. `Exact` on its own never said what the two sides
    // agreed on, and here the whole content of the fix is that they agree on a
    // BRANCH TO `?g` rather than on `?g`'s body.
    let f_targets = targets
        .iter()
        .find(|(n, _)| n == "?f@@YAHH@Z")
        .map(|(_, t)| t.clone())
        .expect("no `?f@@YAHH@Z` in the reference obj's REL24 table");
    assert_eq!(
        f_targets,
        vec!["?g@@YAHH@Z".to_string()],
        "c2 must still emit a REL24 against the `noinline` `?g`; if this ever \
         reads empty, c2 has started expanding a `noinline` callee and the \
         splice clause is wrong rather than merely untested"
    );
    let f = find(&rows, "?f@@YAHH@Z");
    assert_eq!(
        f.1,
        FnByte::Exact,
        "the port must emit that same branch. A `Differs` of (2, 1, 0) here is \
         the ORIGINAL defect returning — the splice taking `?g`'s body — and a \
         `Refused` means the attribute is being read but `Selected::Tail` is no \
         longer picking the branch up behind it"
    );
}

/// **THE CONTROL for the test above**, in its own TU. The identical source
/// without the attribute splices to a byte-exact body — so the red verdict above
/// is the attribute's doing and not the splice's.
#[test]
fn the_same_cell_without_the_attribute_splices_byte_exactly() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let rows = grade(&tc, "w12", W12);
    if rows.is_empty() {
        println!("SKIP: capture produced no graded function");
        return;
    }
    let f = find(&rows, "?f@@YAHH@Z");
    assert_eq!(
        f.1,
        FnByte::Exact,
        "without `noinline`, c2 inlines `?g` into `?f` and the shipped splice \
         emits exactly that. A red verdict here would mean the splice is broken \
         generally and the `noinline` cell above is measuring the wrong thing"
    );
}

/// **WHY THE PORT CANNOT SIMPLY REFUSE THE ATTRIBUTE.** The two sources differ
/// only by `__declspec(noinline)` and are given **the same filename length**,
/// because the `.gl` embeds the source path and an unmatched pair would show a
/// difference that is the path and not the attribute.
///
/// `.ex` (the bodies), `.sy` and `.in` come back **byte-identical**; only `.gl`
/// moves, and by 2 bytes. `docs/OPT_MODE.md` §2 already records that the opt
/// word does not move either. So no clause over the body IL can separate the two
/// cases, and the discriminator is an undecoded `.gl` field — board **#1039**.
#[test]
fn the_attribute_is_invisible_outside_the_gl_record() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let dir = work("ilpair");
    let flags: Vec<String> = FLAGS.iter().map(|s| s.to_string()).collect();
    let mut caps = Vec::new();
    // Same length, so the embedded path contributes the same byte count.
    for (name, text) in [("aaaaaaaaaaaaa", W12), ("bbbbbbbbbbbbb", W10)] {
        let cpp = dir.join(format!("{name}.cpp"));
        std::fs::write(&cpp, text).unwrap();
        let src = c2_reference::to_wibo_path(&cpp);
        let Ok(cap) = tc.capture_reference_with(&src, &dir, &flags, None) else {
            println!("SKIP: capture failed");
            return;
        };
        caps.push(cap);
    }
    for part in ["ex", "sy", "in"] {
        let a = caps[0].bundle.get(part);
        let b = caps[1].bundle.get(part);
        assert!(
            a.is_some() && b.is_some(),
            "`.{part}` missing from a capture — an absent file compares equal to \
             another absent file, which would make this test pass by silence"
        );
        assert_eq!(
            a, b,
            "`.{part}` differs between the two cells. If this fires, the \
             attribute IS readable outside `.gl` and #1039's decline is \
             overturned — which is a finding, not a failure"
        );
    }
    let (ga, gb) = (caps[0].bundle.get("gl"), caps[1].bundle.get("gl"));
    assert!(ga.is_some() && gb.is_some(), "`.gl` missing from a capture");
    assert_ne!(
        ga, gb,
        "`.gl` must differ — it is the ONLY file that carries the attribute, and \
         if it stopped differing the attribute would be unreadable anywhere and \
         #1013 would be permanently closed rather than blocked"
    );
}
