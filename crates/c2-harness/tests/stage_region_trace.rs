//! **PROBE C** — can a port emit c2's region-boundary trace at all?
//!
//! `docs/ARCH_REVIEW_2026-08-21.md` finding 1, last bullet:
//!
//! > Because walks are interleaved mid-pass, matching the stream at all may
//! > require a port to reproduce c2's region decomposition and per-region
//! > relink schedule — whether a port could ever emit this stream is
//! > unmeasured. The deciding probe is cheap: have `PortC2` emit a
//! > region-boundary trace on one already-byte-exact fixture and diff it
//! > against the tap's.
//!
//! This is that probe, and its answer is a **coordinate-system** answer rather
//! than an "off by N boundaries" one, which is why it is stated as an
//! inequality between two counts and not as a diff.
//!
//! # What is measured, on ONE fixture the port emits byte-exact
//!
//! | side | the unit a boundary is expressed in | how many |
//! |---|---|---|
//! | real c2 | **tuple index at a pre-lowering phase** — the region finder cuts the tuple list at `<= 0x50` tuples or at a terminator category | `T` tuples per function, `R` regions |
//! | the port | **emitted instruction index** — `coff::Function` is a byte run; the port's most granular structure is `block_ir::BasicBlock`, whose `body()` is `&[u8]` | `I = bytes / 4` instructions, no tuples, no categories |
//!
//! `T != I` is the finding: a region boundary given as a tuple index has **no
//! image** in the port's coordinate system, so the port→trace projection is not
//! merely unequal to c2's — it is **undefined**. That is arch review finding 4
//! (a) and (c) turned into a number on a fixture the port gets right.
//!
//! # Why this is a positive test and not "grep for the word region"
//!
//! `crates/c2-core/` contains zero occurrences of `region` in c2's sense, and
//! saying so would be an argument from absence — this project's signature
//! defect. Instead the test demands EVIDENCE on both sides: the port must
//! actually emit the fixture byte-exact (or the comparison is between c2 and
//! nothing), and the tap must actually arm and fire (or `T` is not a
//! measurement). Only then is the inequality read.

use std::path::PathBuf;

use c2_core::PortC2;
use c2_reference::stage::STAGE_SITES;
use c2_reference::Toolchain;

/// The fixture profile `stage` and the fixture gate both use.
const FLAGS: [&str; 5] = ["/O1", "/Oi", "/EHsc", "/GS-", "/c"];

fn fixture(name: &str) -> PathBuf {
    c2_harness::fixtures_dir().join(name)
}

/// One fixture the port emits **byte-exact**, so "the port has no region
/// concept" is a statement about a function the port GETS RIGHT — not about
/// one it refuses. A refused function would make the whole comparison vacuous
/// in the most ordinary way.
const FIXTURE: &str = "w5_chain.cpp";

#[test]
fn a_port_cannot_emit_c2s_region_trace_because_the_two_have_no_common_coordinate() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() {
        eprintln!("SKIP: strace absent (needed to keep the IL bundle)");
        return;
    }
    if !tc.has_mingw() {
        eprintln!("SKIP: i686-w64-mingw32-gcc absent (needed to build c2host)");
        return;
    }
    let w = c2_harness::testsupport::clean_scratch_dir("w-restim", "regiontrace");
    let flags: Vec<String> = FLAGS.iter().map(|s| s.to_string()).collect();
    let src = c2_reference::to_wibo_path(&fixture(FIXTURE));
    let cap = tc
        .capture_reference_with(&src, &w, &flags, None)
        .expect("capture of the probe fixture failed");

    // ---- THE PORT SIDE, and it must be byte-exact or there is no probe. ----
    // `/O1` implies `/Gy`, and without function-level linking the port emits
    // one `.text` and no COMDATs at all — which would make the comparison
    // below read "0 functions" and mean nothing.
    let port = PortC2::new(FIXTURE)
        .with_function_level_linking(PortC2::flags_imply_function_level_linking(&FLAGS));
    let port_obj = match port.build(&cap.bundle, FIXTURE) {
        Ok(o) => o,
        Err(e) => panic!(
            "{FIXTURE} is supposed to be in the port's byte-exact class and the port \
             refused it ({e:?}). Probe C compares c2's region trace against a function \
             the port GETS RIGHT; against a refused one the comparison is vacuous. \
             Pick another fixture rather than weakening this assertion."
        ),
    };
    let port_fns = port_obj
        .text_comdat_functions_with_bytes()
        .expect("the port's obj has no .text COMDAT functions");
    let ref_fns = cap
        .ref_obj
        .text_comdat_functions_with_bytes()
        .expect("the reference obj has no .text COMDAT functions");
    assert_eq!(
        port_fns.len(),
        ref_fns.len(),
        "the port and c2 disagree about how many functions {FIXTURE} has, so no \
         per-function comparison below is meaningful"
    );
    let mut exact = 0usize;
    for ((_, pb), (_, rb)) in port_fns.iter().zip(ref_fns.iter()) {
        if pb == rb {
            exact += 1;
        }
    }
    assert!(
        exact > 0,
        "the port emitted {} function(s) of {FIXTURE} and NONE matched c2's bytes — \
         probe C needs at least one byte-exact function or it is comparing c2 against \
         a wrong answer",
        port_fns.len()
    );

    // ---- THE c2 SIDE: the region trace, from the tap. ----
    let out = w.join("tapped.obj");
    let (_obj, rep) = tc
        .replay_tapped_probe(&cap, &w.join("il"), &out, STAGE_SITES, false, true)
        .expect("the tapped replay failed");
    assert!(
        rep.armed_and_fired(),
        "the tap did not arm and fire on {FIXTURE}, so every tuple count below is \
         zero for a reason that has nothing to do with regions"
    );
    assert!(
        rep.walk_refusals.is_empty(),
        "the payload is TRUNCATED ({:?}) — every count below would be a floor",
        rep.walk_refusals
    );
    assert!(
        rep.regions > 0,
        "c2 found ZERO regions on {FIXTURE}: there is no trace to compare against"
    );

    // Per function: c2's tuple count at the EARLIEST phase (`sched1`, before
    // any scheduler run has touched the list) against the port's emitted
    // instruction count. Both are counts of "the thing a boundary indexes".
    // ---- board #3459: pair BY NAME, not by ordinal. ----
    //
    // This loop used to read `rep.funcs.find(func == i + 1)` against the port's
    // COMDAT list in address order. The funcwalk ordinal is `g_fn`, a count of
    // `sched1` entries, and c2 walks functions it never emits into `.text` at
    // all (`w-ordid`: eight such on the fixture corpus), so the two indexes
    // shift apart the moment one of those appears. The payload now carries the
    // function's own name; the pairing is read from it and the ordinal
    // agreement is reported instead of assumed.
    let want: Vec<String> = port_fns.iter().map(|(n, _)| n.clone()).collect();
    let (paired, verdict) = rep.pair_by_identity("sched1", &want);
    eprintln!("  PROBE-C pairing verdict at sched1: {verdict:?}");
    assert!(
        !matches!(verdict, c2_reference::stage::OrdinalVerdict::NoIdentity { .. }),
        "the funcwalk payload carries no function identity, so this probe would be \
         pairing c2's walks to the port's functions by position — board #3459"
    );

    let mut compared = 0usize;
    let mut differing = 0usize;
    for (i, (name, bytes)) in port_fns.iter().enumerate() {
        let Some(w1) = paired[i] else {
            eprintln!("  PROBE-C {name}: c2 walked no function of this name at sched1");
            continue;
        };
        let f = w1.func;
        let tuples = w1.rows().len();
        let insns = bytes.len() / 4;
        compared += 1;
        if tuples != insns {
            differing += 1;
        }
        eprintln!(
            "  PROBE-C fn{f} {name}: c2 tuples at sched1 = {tuples}, port instructions = {insns} \
             ({})",
            if tuples == insns { "equal" } else { "NO COMMON INDEX" }
        );
    }
    eprintln!(
        "  PROBE-C: {} c2 region(s) over {compared} compared function(s); \
         {differing} function(s) where the port's instruction index and c2's tuple \
         index have different cardinality",
        rep.regions
    );
    assert!(
        compared > 0,
        "no function was compared: the function walk produced no `sched1` entry, so \
         this test measured nothing"
    );
    assert!(
        differing > 0,
        "PROBE C's premise no longer holds: the port's emitted instruction count now \
         EQUALS c2's pre-lowering tuple count on every compared function of {FIXTURE}. \
         That would mean a tuple index has an image in the port's coordinates after \
         all, and the cost curve in docs/STEP5_PRICING_2026-08-21.md must be re-priced \
         rather than this assertion relaxed."
    );
    std::fs::remove_dir_all(&w).ok();
}
