//! **The inliner's 24-clause conformance table, graded under `cargo test`** —
//! lane `w-clausefix`, wave 18, board `#3780`–`#3785`.
//!
//! `work/w-inlmetric/CLAUSES.tsv` is the instrument the inliner's entire
//! conformance story is graded on: every "the port has no counterpart to this
//! clause" claim in `docs/whitebox/ref/P_INLINE.md` §6.1, and the split
//! `absent 17 · fitted 2 · R-derived 2 · unexercisable 3` that three rungs
//! quote, are readings of it.
//!
//! **Nothing invoked its checker.** `work/w-inlmetric/check_table.py` was
//! written 2026-08-26 and re-quoted by two later lanes; a `grep` over every
//! `.rs`, `.sh`, `.py` and `.toml` in this repo on 2026-08-28 found it named
//! only in prose. That is `#3679`'s exact shape — *a `scripts/` entry no funnel
//! invokes is not enforcement* — and it is how the table went 48 hours with ten
//! wrong addresses while its green was being cited.
//!
//! This target is the cheapest honest wiring. **It is deliberately NOT a
//! `scripts/gate.sh` row** (`#3691`): a 22nd count-bearing row makes
//! `scripts/gate_identity_diff.sh` exit 2 and refuse to diff for every other
//! live lane in the wave. A `cargo test` target costs those lanes nothing and
//! runs in the merge funnel (`#3687`), which is where a stale table would
//! otherwise reach `master`.
//!
//! # What is asserted, and what is only printed
//!
//! The checker runs five checks. Three need nothing but the repo — **ADDRESS**
//! (the `addr` is inside the `owner` function's `FUNCS.tsv` extent), **WITNESS**
//! (a `fitted`/`R-derived` row's cited token is present at its cited path) and
//! **ABSENCE** (an `absent`/`unexercisable` row's token is absent from
//! `crates/`). Two need the objdump listing — **ALIGN** (the `addr` starts an
//! instruction) and **DECODE** (the instruction there is the one the `asm`
//! column records).
//!
//! The listing is regenerated and never committed (`docs/whitebox/C2_MAP_METHOD.md`),
//! so on a machine without it ALIGN and DECODE **SKIP**. That is correct and it
//! is also the trap: `#3470` — *a clean report over zero rows is not clean*. So
//! this test asserts the **row count** separately from the verdict. A checker
//! that graded nothing prints `GREEN` too; only `rows : 24` tells them apart,
//! and only the ALIGN line tells you whether 24 or 0 of them were checked for
//! alignment.
//!
//! # Blast radius, declared rather than discovered (`#3684`)
//!
//! The ABSENCE check greps `crates/` for tokens that must stay absent —
//! `INLINE_BUDGET` (C3), `budget_decline` (C17), `inline_charge` (C19),
//! `maxlevel` (C15), and sixteen more. A lane that adopts the inline budget
//! model into `crates/` under one of those spellings turns this target **RED on
//! its own tree**, and it will not be able to fix it, because `CLAUSES.tsv` is
//! owned by one lane at a time.
//!
//! That is a **true positive** — it means the table's `absent` verdict has gone
//! stale, which is precisely the reading everything downstream depends on — and
//! `check_table.py`'s failure message says so in those words and names the
//! remedy (a one-cell `state` edit by the table's owner, not a change to the
//! adopting lane's code). It is written down here because a blast radius
//! explained afterwards is an excuse and one declared beforehand is a design.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/c2-harness/../.. is the repo root")
        .to_path_buf()
}

/// Is a usable `python3` on `PATH`? Probed by running it, not by looking for a
/// file — a `python3` that is present and broken is absent for our purposes.
fn python3_is_usable() -> bool {
    Command::new("python3")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// `(exit-0?, stdout+stderr)` from `check_table.py`, with any extra args.
fn run_checker(root: &Path, args: &[&str]) -> (bool, String) {
    let out = Command::new("python3")
        .arg(root.join("work/w-inlmetric/check_table.py"))
        .args(args)
        .current_dir(root)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn python3 for check_table.py: {e}"));
    let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.success(), s)
}

/// The 24 rows and the split three rungs quote. Kept here, in a compiled file,
/// so that shrinking the table is a *code* change somebody reviews rather than
/// a silent narrowing of what "GREEN" covers — `#3748`'s degenerate re-bless.
const ROWS: usize = 24;
const SPLIT: &str = "{'absent': 17, 'fitted': 2, 'R-derived': 2, 'unexercisable': 3}";

#[test]
fn the_clause_table_is_green_and_grades_all_twenty_four_rows() {
    if !python3_is_usable() {
        println!("SKIP: python3 absent — cannot run work/w-inlmetric/check_table.py");
        return;
    }
    let root = repo_root();

    // A MISSING INSTRUMENT IS A FAILURE, NOT A SKIP (`#1496`). If this were a
    // skip, deleting the checker would make the guard silently stop guarding —
    // which is the failure this whole target exists to end.
    for f in ["work/w-inlmetric/check_table.py", "work/w-inlmetric/CLAUSES.tsv"] {
        assert!(
            root.join(f).is_file(),
            "{f} DOES NOT EXIST — the inliner conformance instrument is gone. \
             This is not a portability skip."
        );
    }

    let (ok, report) = run_checker(&root, &[]);

    // Read the VERDICT LINE, never the exit code alone. `gate.sh` prints
    // `GATE: REFUSED` and exits 0; a status is not evidence in this repo.
    assert!(
        report.contains("CONFORMANCE-CHECK: GREEN"),
        "the clause table is RED (exit-0 = {ok}). Full report:\n{report}"
    );

    // THE DENOMINATOR BESIDE THE NUMERATOR (`#3470`). GREEN over zero rows is
    // indistinguishable from GREEN over 24 in the verdict line alone.
    assert!(
        report.contains(&format!("rows     : {ROWS}")),
        "the checker did not grade {ROWS} rows — the table was truncated, or its \
         header changed shape and the parser silently read fewer rows.\n{report}"
    );

    // The conformance split is a published reading (`P_INLINE.md` §6.1,
    // `w-inlmetric`/`w-inlfit`/`w-clausefix` rungs). An address repair must not
    // move it, and neither must anything else, without that being a reviewed edit.
    assert!(
        report.contains(SPLIT),
        "the conformance split MOVED. Expected {SPLIT}.\n{report}"
    );

    // ALIGN/DECODE SKIP without the uncommitted objdump listing, and the SKIP
    // must stay LOUD — the count of rows it did *not* grade has to be visible,
    // or a machine without the listing reports the same GREEN as one with it.
    let skipped = report.contains("ALIGN  : SKIP");
    let loud_skip = format!("ALIGN  : SKIP -- listing absent, so 0 of {ROWS} rows");
    let full_grade = format!("{ROWS} of {ROWS} rows graded");
    assert!(
        report.contains(&loud_skip) || report.contains(&full_grade),
        "the ALIGN line neither graded all {ROWS} rows nor printed a loud SKIP \
         carrying its ungraded count. A silent SKIP is the `#3470` failure: it \
         reports the same GREEN as a full run.\n{report}"
    );
    if skipped {
        println!(
            "PARTIAL: the objdump listing is absent, so ALIGN and DECODE graded 0 of \
             {ROWS} rows. ADDRESS, WITNESS and ABSENCE graded {ROWS} of {ROWS}. \
             Regenerate per docs/whitebox/C2_MAP_METHOD.md, or set C2RS_OBJDUMP_ASM."
        );
    } else {
        println!("FULL: all five checks graded {ROWS} of {ROWS} rows.");
    }
    println!("{report}");
}

/// **The control (`#3336`).** A check nobody has watched fail is decoration,
/// and this one asserts a *green*, so without this it could pass by being
/// unable to fail at all.
///
/// The planted defect is chosen to redden **with or without** the objdump
/// listing: `C16`'s address moved to `0x10b5c06b`, which is a real instruction
/// (so ALIGN cannot catch it) in a *different* function (so ADDRESS can, using
/// only `FUNCS.tsv`, which is committed).
#[test]
fn the_clause_table_check_goes_red_on_a_planted_defect() {
    if !python3_is_usable() {
        println!("SKIP: python3 absent — cannot run the control");
        return;
    }
    let root = repo_root();
    let (ok, report) = run_checker(&root, &["--plant", "C16=10b5c06b"]);

    assert!(
        report.contains("CONFORMANCE-CHECK: RED"),
        "THE CHECKER CANNOT FAIL. A planted address in the wrong function was \
         graded GREEN (exit-0 = {ok}), so the green in the sibling test means \
         nothing. Full report:\n{report}"
    );
    assert!(
        report.contains("C16(PLANTED): ADDRESS"),
        "the checker went RED but not on the planted row — it is failing for \
         some other reason and the control proves nothing.\n{report}"
    );
    assert!(!ok, "the checker printed RED and still exited 0");
    println!("control RED as expected:\n{report}");
}
