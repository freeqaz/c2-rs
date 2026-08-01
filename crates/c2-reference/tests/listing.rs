//! **The listing seam** (board #132, roadmap §9) and its two standing tests.
//!
//! `cl /FAsc … /c` appends `-FAasc -Fa <file>` to c2's own argv and c2 writes a
//! complete assembly listing. Two facts decide whether that listing is usable as
//! an instrument, and neither may be allowed to rot quietly:
//!
//! 1. it does not perturb the obj, so a `.cod` may be captured beside the very
//!    obj the differential grades; and
//! 2. it is byte-faithful **except at relocated branches** — a lane treating it
//!    as raw-byte ground truth would be wrong at every call site.
//!
//! Both are asserted here rather than recorded as one-time observations.
//!
//! All tests skip cleanly (never fail) when the toolchain is absent.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use c2_obj::{ObjDiff, ObjImage};
use c2_reference::cod::CodListing;
use c2_reference::Toolchain;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("fixtures/cpp")
        .join(name)
}

fn work(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let d = std::env::temp_dir().join(format!(
        "c2rs-listing-{tag}-{}-{}-{}",
        std::process::id(),
        nanos,
        n
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// The fixture all three tests run on, and the choice is the whole point: it
/// contains **relocated branches of both kinds** — `b ?plain_func@@YAHHH@Z`
/// (tail call) and `bl ?void_func@@YAXH@Z` (framed call) — plus ordinary
/// arithmetic, indirect loads and a `bctr`.
///
/// `add3.cpp` is the wrong fixture here and was used as one: its bodies are
/// `mullw`/`add`/`blr`, a control structurally incapable of showing the only
/// difference between a `.cod` and an obj that exists. Recorded as the twelfth
/// instance of absence-read-as-success in this project (roadmap §9.1).
const LISTING_FIXTURE: &str = "il_call_return.cpp";

/// Exactly 3 `b` + 7 `bl` relocated branches. Pinned as a *quantity* so the
/// assertions that depend on them cannot become unreachable.
const RELOCATED_BRANCHES: usize = 10;

/// The workload's own optimization profile — what `Toolchain::capture_listing`
/// uses, spelled here so the non-perturbation test can put the *same* flags on
/// both sides of its comparison.
const LISTING_PROFILE: [&str; 5] = ["/O1", "/Oi", "/EHsc", "/GS-", "/c"];

fn guards() -> Option<Toolchain> {
    let tc = match Toolchain::locate() {
        Some(tc) => tc,
        None => {
            eprintln!("SKIP: toolchain absent");
            return None;
        }
    };
    if !tc.has_strace() {
        eprintln!("SKIP: strace absent (needed to keep the IL bundle)");
        return None;
    }
    Some(tc)
}

/// **Standing test 1 — `/FAsc` does not perturb the obj.**
///
/// Capture the same TU twice into the **same work dir** (hence the same `/Fo`
/// path) — once plain, once with the listing — and require the two objs to be
/// byte-identical after `TimeDateStamp` normalization.
///
/// Same `/Fo` is load-bearing, and getting it wrong is a mistake made while
/// measuring this: c2 bakes the output path into `.debug$S` (`S_OBJNAME`), so
/// comparing `out.obj` against `out2.obj` shows ~40 differing bytes that have
/// nothing to do with the listing at all.
#[test]
fn the_listing_does_not_perturb_the_obj() {
    let Some(tc) = guards() else { return };
    let w = work("perturb");
    // The SAME flags on both sides, or this measures the flags and not `/FAsc`.
    // (First run of this test compared `/Ox /GS- /c` against
    // `/O1 /Oi /EHsc /GS- /c` and reported a 1908 vs 2584 byte "perturbation".)
    let src = c2_reference::to_wibo_path(&fixture(LISTING_FIXTURE));
    let flags: Vec<String> = LISTING_PROFILE.iter().map(|s| s.to_string()).collect();
    let plain = tc
        .capture_reference_with(&src, &w, &flags, None)
        .expect("plain capture failed");
    let plain_obj = plain.ref_obj.clone();
    let plain_path = plain.ref_obj_path.clone();
    let (listed, cod) = tc
        .capture_listing_with(&src, &w, &flags, None, false)
        .expect("listing capture failed");

    assert_eq!(
        plain_path, listed.ref_obj_path,
        "the two captures did not use the same /Fo path — c2 bakes it into \
         .debug$S, so this comparison would be measuring the filename"
    );
    assert_eq!(
        ObjImage::diff(&plain_obj, &listed.ref_obj),
        ObjDiff::Identical,
        "/FAsc PERTURBED the obj ({} B plain vs {} B listed) — the listing can \
         no longer be captured beside the obj the differential grades, and every \
         `.cod`-derived decode in the project is now describing a different \
         program",
        plain_obj.len(),
        listed.ref_obj.len(),
    );
    assert!(
        cod.contains("Listing generated by Microsoft"),
        "the capture produced an obj but no recognizable listing ({} bytes) — a \
         silently listing-less seam reads exactly like a working one",
        cod.len(),
    );
    std::fs::remove_dir_all(&w).ok();
}

/// **Standing test 2 — `.cod` is byte-identical to the obj EXCEPT at relocated
/// branches, and that exception is exactly `b` and `bl`.**
///
/// A future lane treating the listing as raw-byte ground truth would be wrong at
/// every call site; one treating "carries a relocation" as the exception would
/// needlessly distrust every data-address row. Both halves are asserted, each
/// with its own failure message.
#[test]
fn the_cod_is_byte_truth_except_at_relocated_branches() {
    let Some(tc) = guards() else { return };
    let w = work("bytes");
    let (captured, cod) = tc
        .capture_listing(&fixture(LISTING_FIXTURE), &w, false)
        .expect("listing capture failed");

    let listing = CodListing::parse(&cod);
    let comdats = captured
        .ref_obj
        .text_comdat_functions_with_bytes()
        .expect("obj .text COMDATs did not decode");

    // The row partition is by MNEMONIC — a property of the fixture's text, not
    // of `CodRow::is_relocated_branch`. That matters: a guard phrased in terms
    // of the classifier would move with the classifier, and a broken classifier
    // could then make every assertion after it unreachable instead of red.
    let mut same = 0usize;
    let mut branch_rows = 0usize;
    let mut branch_same: Vec<String> = Vec::new();
    let mut branch_not_canonical: Vec<String> = Vec::new();
    let mut branch_misclassified: Vec<String> = Vec::new();
    let mut nonbranch_diff: Vec<String> = Vec::new();
    let mut unbound: Vec<String> = Vec::new();

    for f in &listing.functions {
        let Some(bytes) = comdats.iter().find(|(n, _)| *n == f.name).map(|(_, b)| b) else {
            unbound.push(f.name.clone());
            continue;
        };
        for r in &f.rows {
            let o = r.offset as usize;
            let where_ = format!("{} +{o:#x} {}", f.name, r.asm);
            if o + 4 > bytes.len() {
                nonbranch_diff.push(format!("{where_} — runs past the COMDAT"));
                continue;
            }
            let actual = u32::from_be_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]]);
            let is_branch = matches!(r.mnemonic(), "b" | "bl");
            if is_branch {
                branch_rows += 1;
                if actual == r.word {
                    branch_same.push(where_.clone());
                }
                if !matches!(r.word, 0x4800_0000 | 0x4800_0001) {
                    branch_not_canonical.push(format!("{where_} cod={:08x}", r.word));
                }
                if !r.is_relocated_branch() {
                    branch_misclassified.push(where_);
                }
            } else if actual == r.word {
                same += 1;
            } else {
                nonbranch_diff.push(format!(
                    "{where_} cod={:08x} obj={actual:08x}",
                    r.word
                ));
            }
        }
    }

    // (a) THE POSITIVE CONTROL, and its quantity is a fact about the fixture.
    //     Without it every assertion below is vacuously true on a fixture
    //     containing no relocated branch — which is exactly how this claim was
    //     "verified" against `add3`.
    assert_eq!(
        branch_rows, RELOCATED_BRANCHES,
        "{LISTING_FIXTURE} no longer contains the {RELOCATED_BRANCHES} relocated \
         branch rows this control needs (3 `b` + 7 `bl`). Nothing below can go \
         red without them: pick a fixture that has them, do not relax this number"
    );
    // (b) Every one of them must DIFFER from the obj — that is the claim.
    assert!(
        branch_same.is_empty(),
        "a relocated branch printed the obj's real displacement instead of the \
         canonical word — the `.cod` decode rule has changed: {branch_same:?}"
    );
    // (c) …and what it prints instead is the canonical unrelocated word.
    assert!(
        branch_not_canonical.is_empty(),
        "a branch row's listing word is neither 48000000 nor 48000001 — the \
         canonical-word rule is wrong: {branch_not_canonical:?}"
    );
    // (d) The negative half: nothing else may differ. Data-address rows
    //     (`lwz r31,?g_i@@3HA(r11)`) carry relocations too and MUST match,
    //     because c2 leaves the displacement 0 in both artifacts.
    assert!(
        nonbranch_diff.is_empty(),
        "{} non-branch row(s) differ between .cod and obj — the exception class \
         is wider than b/bl and every `.cod`-derived byte fact in the project \
         needs re-checking: {:?}",
        nonbranch_diff.len(),
        nonbranch_diff,
    );
    // (e) The reader's own classifier agrees with the mnemonic partition, so a
    //     consumer using `is_relocated_branch` gets the same answer this test
    //     graded.
    assert!(
        branch_misclassified.is_empty(),
        "CodRow::is_relocated_branch missed a relocated branch this test found \
         by mnemonic — consumers would trust bytes that are canonical: \
         {branch_misclassified:?}"
    );
    // (f) The denominator, so a parse that silently read nothing cannot pass.
    assert!(
        same >= 40,
        "only {same} matching rows were compared — the listing parse is \
         under-reading and (d) passed on rows it never looked at"
    );
    // (g) Totality of the name binding, which board #136 depends on.
    assert!(
        unbound.is_empty(),
        "listing PROC name(s) with no matching obj .text COMDAT: {unbound:?}"
    );
    std::fs::remove_dir_all(&w).ok();
}

/// `/QXSTALLS` annotates the listing — the scheduling-demand axis (board #134).
/// Three things at once: the flag reaches c2, the annotation is **absent**
/// without it (so the #134 reader cannot be matching something ambient), and the
/// flag does not move the obj (so its numbers describe the code the differential
/// actually grades).
#[test]
fn qxstalls_annotates_the_listing_and_only_with_the_flag() {
    let Some(tc) = guards() else { return };
    let w = work("qx");
    let (plain, plain_cod) = tc
        .capture_listing(&fixture(LISTING_FIXTURE), &w, false)
        .expect("plain listing capture failed");
    let plain_obj = plain.ref_obj.clone();
    let (qx, qx_cod) = tc
        .capture_listing(&fixture(LISTING_FIXTURE), &w, true)
        .expect("qxstalls listing capture failed");

    assert!(
        !plain_cod.contains("Stall summary for function"),
        "the un-annotated listing already contains a stall summary — the #134 \
         reader would be matching something ambient, not the flag's effect"
    );
    assert!(
        qx_cod.contains("Stall summary for function"),
        "/QXSTALLS produced no stall summary; if c2 had rejected the flag it \
         would have said `C1007 unrecognized flag`"
    );
    assert!(
        qx_cod.contains("Estimated block IPC"),
        "/QXSTALLS produced no per-block IPC estimate"
    );
    assert_eq!(
        ObjImage::diff(&plain_obj, &qx.ref_obj),
        ObjDiff::Identical,
        "/QXSTALLS PERTURBED the obj — its annotations would then describe a \
         program the differential never grades, and #134's number would be about \
         the wrong code"
    );
    std::fs::remove_dir_all(&w).ok();
}
