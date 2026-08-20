//! **The stage oracle's named controls** — `c2-reference::stage`.
//!
//! Seven tests, and each is pinned to a NAMED fixture rather than to a count,
//! because a control pinned by count passes in an unprovisioned worktree the
//! moment the count matches (`docs/rungs/README.md`, boards #3219/#3231).
//!
//! | test | grades |
//! |---|---|
//! | [`the_tapped_run_actually_armed`] | the ENVIRONMENT. Fails, never skips, under `C2RS_REQUIRE_TOOLCHAIN` |
//! | [`taps_are_inert_unarmed_and_never_move_the_obj`] | **G1 neutrality** — the sole judge's own criterion |
//! | [`a_wrong_slide_arms_nothing_and_never_moves_the_obj`] | **the FAIL-CLOSED check at a nonzero slide**, against a live image |
//! | [`scheduler_taps_are_silent_at_od_and_loud_at_o1`] | **G3 discrimination** — the null control |
//! | [`the_snapshot_is_nonempty_and_agrees_with_a_second_derivation`] | **G5 content**, cross-derived three ways |
//! | [`the_tuple_walk_sees_the_scheduler_move_the_list`] | the LIVENESS control that makes the COLOR null interpretable |
//! | [`the_two_site_tables_are_one_table`] | the C table and the Rust table cannot drift |
//!
//! # Why `il_call_perm.cpp` and why `add3.cpp` is BANNED here
//!
//! `crates/c2-reference/src/cod.rs`'s module doc records `add3` as the control
//! that **cannot detect the property it was run against** — `mullw`/`add`/`blr`,
//! no relocated branch — and that is the twelfth recorded instance of
//! absence-read-as-success in this project. The same trap is live here in a
//! sharper form: the region tap fires on *scheduling regions*, and
//! `P_DAG.md` §4.5 records (15/15 cells) that **a call ends a region**. A
//! single-region, call-free fixture would make a zero region count look like a
//! property of the mechanism when it is a property of the fixture.
//!
//! `il_call_perm.cpp` has multiple functions, relocated branches of both kinds,
//! and calls. If a future lane changes the fixture here, it owes the same
//! argument.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use c2_obj::{ObjDiff, ObjImage};
use c2_reference::stage::{OPT_GATED_SITES, STAGE_SITES};
use c2_reference::Toolchain;

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// The fixture every capture-based test in this file runs on. See the module
/// doc: it is chosen for calls and relocated branches, not for convenience.
const STAGE_FIXTURE: &str = "il_call_perm.cpp";

/// The fixture that MUST NOT be used as the positive control here, named so a
/// future edit trips over the reason rather than rediscovering it.
const BANNED_CONTROL: &str = "add3.cpp";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture(name: &str) -> PathBuf {
    repo_root().join("fixtures/cpp").join(name)
}

fn work(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let d = std::env::temp_dir().join(format!(
        "c2rs-stage-{tag}-{}-{}-{}",
        std::process::id(),
        nanos,
        n
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// `C2RS_REQUIRE_TOOLCHAIN=1` turns every skip in this file into a failure.
/// Armed 2026-08-18 across the suite; here it is load-bearing twice over,
/// because a skipped stage test produces exactly the same output as a stage
/// test whose taps silently refused.
fn require() -> bool {
    std::env::var_os("C2RS_REQUIRE_TOOLCHAIN").is_some()
}

fn guards(what: &str) -> Option<Toolchain> {
    let missing = |why: &str| -> Option<Toolchain> {
        if require() {
            panic!("{what}: {why} — and C2RS_REQUIRE_TOOLCHAIN is set, so this is a FAILURE, not a skip");
        }
        eprintln!("SKIP: {why}");
        None
    };
    let Some(tc) = Toolchain::locate() else {
        return missing("toolchain absent");
    };
    if !tc.has_strace() {
        return missing("strace absent (needed to keep the IL bundle)");
    }
    if !tc.has_mingw() {
        return missing("i686-w64-mingw32-gcc absent (needed to build c2host)");
    }
    Some(tc)
}

/// The workload's own optimization profile.
fn o1() -> Vec<String> {
    ["/O1", "/Oi", "/EHsc", "/GS-", "/c"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// The same profile with the optimizer OFF. `/Od` is the whole point: the
/// optimizer flag `DAT_10c2e2fc` is checked FIRST at each of the four
/// scheduler sites, so at `/Od` none of them is reached. It is **necessary and
/// not sufficient** — see [`c2_reference::stage::OPT_GATED_SITES`] for the
/// second per-function gate at `[esi+0x1c]`, corrected in the fix round.
fn od() -> Vec<String> {
    ["/Od", "/Oi", "/EHsc", "/GS-", "/c"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// **THE ENVIRONMENT CONTROL.**
///
/// Everything else in this file is a statement about c2. This one is a
/// statement about the box the measurement was taken on, and without it every
/// other result in the lane is void rather than provisional.
///
/// The failure it exists to catch: a fresh `git worktree add` has no
/// `compilers/` (gitignored, and it does not follow a worktree), so every
/// capture test **skips**, cargo swallows the SKIP line for a passing test, and
/// a registered RED reads GREEN with a clean suite and the right exit code
/// (#3219, #3231). The stage-tap version is worse, because "armed 0 sites" and
/// "the pass never ran" are the same observation: zero.
#[test]
fn the_tapped_run_actually_armed() {
    let Some(tc) = guards("the_tapped_run_actually_armed") else {
        return;
    };
    let w = work("armed");
    let captured = tc
        .capture_reference_with(
            &c2_reference::to_wibo_path(&fixture(STAGE_FIXTURE).canonicalize().unwrap()),
            &w.join("cap"),
            &o1(),
            None,
        )
        .expect("capture_reference_with failed");
    let (_obj, rep) = tc
        .replay_tapped(
            &captured,
            &w.join("il"),
            &captured.ref_obj_path.clone(),
            STAGE_SITES,
        )
        .expect("tapped replay failed");

    assert!(
        rep.armed_ok(),
        "the stage tap did NOT arm — every other measurement in this lane is \
         VOID, not provisional.\n  armed:   {:?}\n  refused: {:?}\n  lines:\n{}",
        rep.armed,
        rep.refused,
        rep.lines.join("\n")
    );
    assert_eq!(
        rep.armed.len(),
        STAGE_SITES.len(),
        "armed {} of {} sites — a partial arm is a refusal in disguise",
        rep.armed.len(),
        STAGE_SITES.len()
    );
    std::fs::remove_dir_all(&w).ok();
}

/// **G1 — NEUTRALITY. The sole judge's own criterion, and the lane's go/no-go.**
///
/// An oracle that changes the compiler is not an oracle; it is a fifth
/// wrong-emit family with a friendly interface. The comparison is made through
/// **one** function ([`Toolchain::replay_tapped`] with an empty tap list is the
/// disarmed leg) so that the two legs cannot differ in anything but the arming.
///
/// Same shape as the listing seam's `the_listing_does_not_perturb_the_obj`.
#[test]
fn taps_are_inert_unarmed_and_never_move_the_obj() {
    let Some(tc) = guards("taps_are_inert_unarmed_and_never_move_the_obj") else {
        return;
    };
    for name in [STAGE_FIXTURE, "il_call_return.cpp", "add3.cpp"] {
        let w = work("neutral");
        let captured = tc
            .capture_reference_with(
                &c2_reference::to_wibo_path(&fixture(name).canonicalize().unwrap()),
                &w.join("cap"),
                &o1(),
                None,
            )
            .unwrap_or_else(|e| panic!("{name}: capture failed: {e}"));

        // BOTH legs write to the reference's OWN /Fo path, and that is not
        // tidiness: c2 embeds the output path in the obj, so replaying to a
        // different path changes the obj's LENGTH. Writing to `w/out.obj`
        // here made the third assertion below read
        // `Differs { first_offset: 8, a_len: 1725, b_len: 1721 }` — offset 8
        // is PointerToSymbolTable, and the 4-byte delta was the path string,
        // not c2. A "stronger" check that is actually comparing two different
        // commands is worth less than no check; `captured.ref_obj` is already
        // in memory, so overwriting the file is safe.
        let out = captured.ref_obj_path.clone();
        let (disarmed, rep0) = tc
            .replay_tapped(&captured, &w.join("il"), &out, &[])
            .unwrap_or_else(|e| panic!("{name}: disarmed replay failed: {e}"));
        let (armed, rep1) = tc
            .replay_tapped(&captured, &w.join("il"), &out, STAGE_SITES)
            .unwrap_or_else(|e| panic!("{name}: armed replay failed: {e}"));

        // The disarmed leg must really be disarmed, and the armed leg must
        // really be armed. Without both, "identical" is trivially true.
        assert!(
            rep0.lines.is_empty(),
            "{name}: the DISARMED leg printed stage-tap output — it was not \
             inert:\n{}",
            rep0.lines.join("\n")
        );
        assert!(
            rep1.armed_ok(),
            "{name}: the ARMED leg did not arm, so this comparison grades \
             nothing:\n{}",
            rep1.lines.join("\n")
        );

        assert_eq!(
            ObjImage::diff(&disarmed, &armed),
            ObjDiff::Identical,
            "{name}: THE STAGE TAP MOVED THE OBJ. The oracle is grading a \
             different compiler than the judge does; this is a DECLINE, not a \
             tuning problem. disarmed={}B armed={}B",
            disarmed.len(),
            armed.len()
        );
        // And the untapped path's own product, for a third point of contact.
        assert_eq!(
            ObjImage::diff(&captured.ref_obj, &armed),
            ObjDiff::Identical,
            "{name}: the armed replay does not reproduce the PIPELINE obj"
        );
        std::fs::remove_dir_all(&w).ok();
    }
}

/// **G3 — DISCRIMINATION. The null control, and it is free.**
///
/// The optimizer flag `DAT_10c2e2fc` (bit 21, set at `0x10b82429`) is tested
/// FIRST at each scheduler site — `0x10b7dc83`/`0x10b7dcc2`/`0x10b7dd01` are
/// `cmp DWORD PTR ds:0x10c2e2fc,edi` with `edi == 0`, and `0x10b7dfd9` is the
/// same test ahead of `sched0`. So at `/Od` none of the four is reached, which
/// is the only direction this test asserts.
///
/// **FIX-ROUND CORRECTION.** This doc used to say the four runs are gated
/// *"only"* by that flag, citing `P_DAG.md` §1. The disassembly refutes it:
/// each of the three in-band sites carries a second per-function gate
/// (`test BYTE PTR [esi+0x1c],bl`, `bl == 1`, at
/// `0x10b7dc8b`/`0x10b7dcca`/`0x10b7dd09`) and `sched0` carries three more
/// (`0x10b7dfe3`, `0x10b7dff2`, `0x10b7dff9`). The `/Od` ⇒ 0 direction is
/// unaffected; what is not structural is the converse. See
/// [`c2_reference::stage::OPT_GATED_SITES`].
///
/// If the two counts come out equal, **the instrument is measuring itself** —
/// the fifth entry in this repo's "ranking instruments measure themselves"
/// family, four for four so far. That would be a decline of the SITE TABLE,
/// not of the mechanism.
#[test]
fn scheduler_taps_are_silent_at_od_and_loud_at_o1() {
    let Some(tc) = guards("scheduler_taps_are_silent_at_od_and_loud_at_o1") else {
        return;
    };
    assert_ne!(
        STAGE_FIXTURE, BANNED_CONTROL,
        "add3.cpp is BANNED as this test's fixture: it is the recorded control \
         that cannot detect the property it is run against (cod.rs module doc)"
    );

    let mut counts = Vec::new();
    for (label, flags) in [("Od", od()), ("O1", o1())] {
        let w = work(&format!("disc{label}"));
        let captured = tc
            .capture_reference_with(
                &c2_reference::to_wibo_path(&fixture(STAGE_FIXTURE).canonicalize().unwrap()),
                &w.join("cap"),
                &flags,
                None,
            )
            .unwrap_or_else(|e| panic!("/{label}: capture failed: {e}"));
        let (_obj, rep) = tc
            .replay_tapped(
                &captured,
                &w.join("il"),
                &captured.ref_obj_path.clone(),
                STAGE_SITES,
            )
            .unwrap_or_else(|e| panic!("/{label}: tapped replay failed: {e}"));
        assert!(
            rep.armed_ok(),
            "/{label}: taps did not arm — the comparison below would be void"
        );
        let sched: u64 = OPT_GATED_SITES.iter().map(|s| rep.hits_at(s)).sum();
        counts.push((label, sched, rep));
        std::fs::remove_dir_all(&w).ok();
    }

    let (_, od_hits, od_rep) = &counts[0];
    let (_, o1_hits, o1_rep) = &counts[1];
    assert_eq!(
        *od_hits, 0,
        "/Od fired {od_hits} optimizer-gated taps and P_DAG.md §1 says it \
         cannot: the site table is wrong, or the gate is not the one recorded.\n\
         {}",
        od_rep.lines.join("\n")
    );
    assert!(
        *o1_hits > 0,
        "/O1 fired ZERO optimizer-gated taps: the instrument cannot see the \
         thing it was built to see, and a zero payload would have read as a \
         clean result.\n{}",
        o1_rep.lines.join("\n")
    );
}

/// **THE FAIL-CLOSED CHECK, AGAINST A LIVE IMAGE, AT A NONZERO SLIDE.**
///
/// Review finding: the `+ slide` half of `tap_arm`'s check — the half the
/// lane's first plan defect was about, when an `HMODULE`-derived slide of
/// `ef500018` sent every site to garbage — had never executed anywhere except
/// at slide 0, because c2.dll loads at its preferred base on every run on this
/// box. The only standing test of the refusal path was a string parse over a
/// synthetic stderr. **A guard nobody has watched fire is a guard nobody has
/// tested.**
///
/// So this displaces every site address by `0x18` — the same value wibo hands
/// back as an `HMODULE`, which is how the plan defect arrived — and asserts the
/// two things a fail-closed check owes:
///
/// 1. **NOTHING is patched**: all seven sites refuse, and (the sharp part) five
///    of them refuse on the TARGET check rather than the opcode check, because
///    at `+0x18` five of the seven displaced addresses really do hold an
///    `e8 rel32`. An opcode-only guard would have patched five call sites in
///    the middle of c2's phase driver.
/// 2. **The obj is byte-identical anyway.** A refusal that still perturbs the
///    compiler is not a refusal.
#[test]
fn a_wrong_slide_arms_nothing_and_never_moves_the_obj() {
    let Some(tc) = guards("a_wrong_slide_arms_nothing_and_never_moves_the_obj") else {
        return;
    };
    let w = work("wrongslide");
    let captured = tc
        .capture_reference_with(
            &c2_reference::to_wibo_path(&fixture(STAGE_FIXTURE).canonicalize().unwrap()),
            &w.join("cap"),
            &o1(),
            None,
        )
        .expect("capture failed");
    let out = captured.ref_obj_path.clone();

    // The control leg: the same command at the REAL slide, so the comparison
    // below is against a run that did arm and fire.
    let (armed_obj, real) = tc
        .replay_tapped(&captured, &w.join("il"), &out, STAGE_SITES)
        .expect("tapped replay failed");
    assert!(
        real.armed_and_fired(),
        "the control leg did not arm and fire, so the contrast below grades \
         nothing:\n{}",
        real.lines.join("\n")
    );

    // NOTE the shape: a missing obj is not an Err here, so the arming
    // assertions below are REACHED even when the mutation this test exists to
    // catch makes c2 crash. Ordering of failures is part of the test.
    let (wrong_obj, wrong) = tc
        .replay_tapped_forced_slide(&captured, &w.join("il"), &out, STAGE_SITES, 0x18)
        .expect("forced-slide replay could not be launched at all");

    assert!(
        wrong.armed.is_empty(),
        "A WRONG SLIDE PATCHED {} SITE(S). The fail-closed check is the only \
         thing standing between a relocated image and five patched addresses \
         in the middle of c2's phase driver: {:?}",
        wrong.armed.len(),
        wrong.armed
    );
    assert_eq!(
        wrong.refused.len(),
        STAGE_SITES.len(),
        "expected every one of the {} sites to refuse, got {}: {:?}",
        STAGE_SITES.len(),
        wrong.refused.len(),
        wrong.refused
    );
    assert!(
        !wrong.armed_and_fired(),
        "a run that armed nothing must not read as armed-and-fired"
    );
    // THE HALF THAT HAD NEVER RUN. `never patch a guess` is the message from
    // the TARGET+SLIDE check; `expected e8` is the opcode check. If the target
    // check never fires here, this test has degenerated into a second opcode
    // test and the `+ slide` arithmetic is untested again.
    let by_target = wrong
        .refused
        .iter()
        .filter(|(_, line)| line.contains("never patch a guess"))
        .count();
    assert!(
        by_target >= 4,
        "only {by_target} of {} refusals came from the TARGET+SLIDE check; the \
         rest are opcode refusals, so the slide arithmetic is still untested. \
         Refusals: {:?}",
        wrong.refused.len(),
        wrong.refused
    );
    let wrong_obj = wrong_obj.expect(
        "the refusing run produced NO obj: c2 crashed or aborted, which a run \
         that patched nothing cannot do",
    );
    assert_eq!(
        ObjImage::diff(&armed_obj, &wrong_obj),
        ObjDiff::Identical,
        "the REFUSING run produced a different obj than the armed one — a \
         refusal that still perturbs the compiler is not a refusal"
    );
    std::fs::remove_dir_all(&w).ok();
}

/// The C site table and the Rust site list are two readers of one definition,
/// and nothing in the build makes them agree — `c2host/` is not in the Rust
/// workspace and never will be (std-only, zero external crates). So a test
/// stands where a shared header cannot.
///
/// This is the pattern `docs/ARCHITECTURE_SEAMS.md` §0 used for the `..base`
/// and `bind.rs` moves. Needs no toolchain: it reads the repo.
#[test]
fn the_two_site_tables_are_one_table() {
    let src = repo_root().join("c2host/stagetap.c");
    let text = std::fs::read_to_string(&src)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", src.display()));
    let body = text
        .split_once("static const TapSite g_sites[] = {")
        .expect("c2host/stagetap.c no longer declares `g_sites` the way this test reads it")
        .1
        .split_once("};")
        .expect("unterminated g_sites table")
        .0;
    let names: Vec<String> = body
        .lines()
        .filter_map(|l| {
            let l = l.trim();
            let rest = l.strip_prefix("{ \"")?;
            let (n, _) = rest.split_once('"')?;
            Some(n.to_string())
        })
        .collect();
    assert!(
        !names.is_empty(),
        "parsed ZERO site names out of c2host/stagetap.c — an empty parse is \
         not agreement, it is a broken test reading as a green"
    );
    assert_eq!(
        names,
        STAGE_SITES.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "c2host/stagetap.c's table and c2-reference::stage::STAGE_SITES have \
         drifted — order included, because the C side indexes by position"
    );
}

/// **G5 — CONTENT. A positive, cross-derived check, never an inspected green.**
///
/// A structurally deterministic **empty** snapshot passes G1, G2, G2b and G3
/// trivially, and absence-read-as-success is this project's own signature
/// defect (twelve recorded instances; `add3` is the twelfth). So content is a
/// named criterion with a second derivation attached, not something a human
/// notices while reading output.
///
/// The second derivation (#3288) is genuinely differently-built: the tap's
/// per-function hit count is produced by **patched call sites inside c2's own
/// code**, counted in `c2host`; the `PROC` count is produced by **c2's own
/// `/FAsc` listing writer**; the `.text` COMDAT count is produced by the
/// **COFF section table** in the obj. Three paths, no shared step after c2's
/// front end.
#[test]
fn the_snapshot_is_nonempty_and_agrees_with_a_second_derivation() {
    let Some(tc) = guards("the_snapshot_is_nonempty_and_agrees_with_a_second_derivation") else {
        return;
    };
    let w = work("content");
    let src = c2_reference::to_wibo_path(&fixture(STAGE_FIXTURE).canonicalize().unwrap());

    // Derivation A — the tap.
    let captured = tc
        .capture_reference_with(&src, &w.join("cap"), &o1(), None)
        .expect("capture failed");
    let (_obj, rep) = tc
        .replay_tapped_with(
            &captured,
            &w.join("il"),
            &captured.ref_obj_path.clone(),
            STAGE_SITES,
            true,
        )
        .expect("tapped replay failed");
    assert!(rep.armed_ok(), "taps did not arm: {:?}", rep.refused);
    assert!(
        !rep.tuples.is_empty(),
        "THE PAYLOAD IS EMPTY. Deterministic and vacuous passes G1/G2/G3 \
         trivially and is not a green.\n{}",
        rep.lines.join("\n")
    );
    assert!(
        rep.walk_refusals.is_empty(),
        "the bounded walk was TRUNCATED, so the tuple count below is a floor \
         and not a measurement: {:?}",
        rep.walk_refusals
    );
    assert!(rep.regions > 0, "no scheduling region was observed at all");

    // Derivation B — c2's own /FAsc listing writer.
    let (listing_cap, cod) = tc
        .capture_listing_with(&src, &w.join("cod"), &o1(), None, false)
        .expect("listing capture failed");
    let procs = c2_reference::cod::CodListing::parse(&cod).functions.len();

    // Derivation C — the COFF section table of the obj.
    let comdats = listing_cap
        .ref_obj
        .text_comdat_functions()
        .map(|v| v.len())
        .unwrap_or(0);

    // Every per-function site fires once per function ON THIS FIXTURE, so all
    // six must equal the function count. `region` is per REGION and is
    // excluded.
    //
    // FIX-ROUND CORRECTION: this equality is EMPIRICAL, not structural. The
    // rung once justified it from "the scheduler runs are gated only by the
    // optimizer flag"; three of the four scheduler sites also test
    // `[esi+0x1c] & 1` per function (`OPT_GATED_SITES`' doc), so a function
    // with that bit clear would break the equality without anything being
    // wrong with the tap. If this fires, read it as a fact about the fixture
    // first.
    for site in STAGE_SITES.iter().filter(|s| **s != "region") {
        assert_eq!(
            rep.hits_at(site) as usize,
            procs,
            "site {site} fired {} times but c2's own listing prints {procs} PROC \
             — the tap and c2's narration disagree about how many functions c2 \
             compiled",
            rep.hits_at(site)
        );
    }
    assert_eq!(
        procs, comdats,
        "the listing and the obj disagree about the function count, so the \
         cross-check above has no fixed point"
    );
    assert!(
        procs > 1,
        "{STAGE_FIXTURE} has {procs} function(s): a single-function fixture \
         cannot detect a per-function count that is wrong by a constant"
    );

    // And P_DAG.md §1's "four scheduler runs per function", re-derived as an
    // EQUALITY BETWEEN FOUR SEPARATELY PATCHED SITES rather than as a reading.
    let s1 = rep.hits_at("sched1");
    assert!(
        s1 == rep.hits_at("sched2")
            && s1 == rep.hits_at("sched3")
            && s1 == rep.hits_at("sched0"),
        "the four scheduler sites disagree ({} {} {} {}) — P_DAG.md §1 says \
         four runs per function",
        s1,
        rep.hits_at("sched2"),
        rep.hits_at("sched3"),
        rep.hits_at("sched0")
    );
    std::fs::remove_dir_all(&w).ok();
}

/// **The observable is LIVE, and the COLOR null is about COLOR.**
///
/// This is the control that stops [`the_snapshot_is_nonempty_and_agrees_with_a_second_derivation`]
/// from being satisfied by a frozen structure. Measured on this fixture: the
/// tuple rows read at `sched1` differ from those at `sched2` (a scheduler run
/// plus globregs happened in between) and those at `sched3` differ from
/// `sched0` (the lowering band) — while `sched2` vs `sched3`, which brackets
/// the register allocator, is IDENTICAL on every function, and a 128-byte raw
/// window says COLOR wrote nothing in the tuple record at all.
///
/// So a ported COLOR **cannot** be graded against this observable, and that is
/// a finding about where the allocator's output lives — not a defect in the
/// tap. If this test ever fails at the `sched1`/`sched2` end, the walk has gone
/// blind and every COLOR conclusion drawn from it is void.
#[test]
fn the_tuple_walk_sees_the_scheduler_move_the_list() {
    let Some(tc) = guards("the_tuple_walk_sees_the_scheduler_move_the_list") else {
        return;
    };
    let w = work("live");
    let captured = tc
        .capture_reference_with(
            &c2_reference::to_wibo_path(&fixture(STAGE_FIXTURE).canonicalize().unwrap()),
            &w.join("cap"),
            &o1(),
            None,
        )
        .expect("capture failed");
    let (_obj, rep) = tc
        .replay_tapped_with(
            &captured,
            &w.join("il"),
            &captured.ref_obj_path.clone(),
            STAGE_SITES,
            true,
        )
        .expect("tapped replay failed");
    assert!(rep.armed_ok());

    let funcs = rep.blocks.iter().map(|b| b.func).max().unwrap_or(0);
    assert!(funcs > 0, "no phase-tagged region blocks at all");
    let cat = |phase: &str, f: u32| -> Vec<String> {
        rep.blocks_at(phase, f)
            .into_iter()
            .flat_map(|b| b.tuples.iter().cloned())
            .collect()
    };
    let mut sched_moved = 0;
    for f in 1..=funcs {
        if cat("sched1", f) != cat("sched2", f) {
            sched_moved += 1;
        }
    }
    assert!(
        sched_moved > 0,
        "the tuple rows are IDENTICAL across the first scheduler run on all \
         {funcs} functions. The walk is reading a frozen structure and every \
         phase conclusion drawn from it — including the COLOR null — is void."
    );
    std::fs::remove_dir_all(&w).ok();
}
