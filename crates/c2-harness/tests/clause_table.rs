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
//! The checker runs six checks. Four need nothing but the repo — **ADDRESS**
//! (the `addr` is inside the `owner` function's `FUNCS.tsv` extent), **WITNESS**
//! (a `fitted`/`R-derived` row's cited token is present at its cited path),
//! **ABSENCE** (an `absent`/`unexercisable` row's token is absent from
//! `crates/`) and **CITES** (the set of files under `crates/` citing the row's
//! address equals its frozen `cites` cell). Two need the objdump listing —
//! **ALIGN** (the `addr` starts an instruction) and **DECODE** (the instruction
//! there is the one the `asm` column records).
//!
//! # Two graders, and the second one is why §6.1 can no longer drift
//!
//! **ADDED 2026-08-29, lane `w-clausegen`, board `#3817`–`#3823`.**
//! `work/w-inlmetric/gen_table.py` renders `P_INLINE.md` §6.1 *from*
//! `CLAUSES.tsv`, between two markers in the page. Everything this file already
//! says about `check_table.py` — that a checker nobody invokes is not
//! enforcement (`#3679`, `#3785`) — applied one level up and unnoticed: §6.1
//! was hand-re-synced three times in three days and `check_table.py` printed
//! GREEN through every one, because it grades the machine table and **cannot
//! see the prose copy** (`#3814`). Both graders now run here.
//!
//! The listing is regenerated and never committed (`docs/whitebox/C2_MAP_METHOD.md`),
//! so on a machine without it ALIGN and DECODE **SKIP**. That is correct and it
//! is also the trap: `#3470` — *a clean report over zero rows is not clean*. So
//! this test asserts the **row count** separately from the verdict. A checker
//! that graded nothing prints `GREEN` too; only `rows : 24` tells them apart,
//! and only the ALIGN line tells you whether 24 or 0 of them were checked for
//! alignment.
//!
//! # Blast radius, declared rather than discovered (`#3684`) — and it fired here first
//!
//! The ABSENCE check greps `crates/` for the twenty tokens the `absent` and
//! `unexercisable` rows cite as *must stay absent*. A lane that adopts the
//! inline budget model into `crates/` under one of those spellings turns this
//! target **RED on its own tree**, and it will not be able to fix it, because
//! `CLAUSES.tsv` is owned by one lane at a time. That is a **true positive** —
//! the table's `absent` verdict would have gone stale, which is precisely the
//! reading everything downstream depends on — and `check_table.py`'s failure
//! message says so in those words and names the remedy: a one-cell `state` edit
//! by the table's owner, not a change to the adopting lane's code.
//!
//! **THE FIRST THING TO TRIP IT WAS THIS FILE**, and that is why the tokens are
//! not spelled out above. An earlier draft of this comment listed four of them
//! as examples. The check cannot tell a **mention in a doc comment** from a
//! **counterpart in the port** — `#3641`'s class, *a counter cannot tell a mark
//! from a mention* — so four rows went RED because of prose *about* them.
//! Anything under `crates/` that needs to discuss these clauses must name them
//! by **clause id** (C3, C15, C17, C19), never by token.
//!
//! Worse, and now fixed: `check_table.py` used a bare `git grep`, which is
//! **blind to untracked files**. This file was untracked while its controls
//! were watched, so the controls could not see the defect it was introducing —
//! the verdict changed at `git add` time, not at write time. The checker now
//! passes `--untracked --exclude-standard`. See the rung, §10.

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

/// `(exit-0?, stdout+stderr)` from one of the two graders under
/// `work/w-inlmetric/`, with any extra args.
fn run_tool(root: &Path, tool: &str, args: &[&str]) -> (bool, String) {
    let out = Command::new("python3")
        .arg(root.join("work/w-inlmetric").join(tool))
        .args(args)
        .current_dir(root)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn python3 for {tool}: {e}"));
    let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.success(), s)
}

fn run_checker(root: &Path, args: &[&str]) -> (bool, String) {
    run_tool(root, "check_table.py", args)
}

/// The 24 rows and the split three rungs quote. Kept here, in a compiled file,
/// so that shrinking the table is a *code* change somebody reviews rather than
/// a silent narrowing of what "GREEN" covers — `#3748`'s degenerate re-bless.
const ROWS: usize = 24;
/// **MOVED 2026-08-28 at the wave-18 merge**, from
/// `{'absent': 17, 'fitted': 2, 'R-derived': 2, 'unexercisable': 3}`.
///
/// This is the first time the split has moved for the reason it is supposed to
/// move: the port acquired a counterpart. Lane `w-inlbudget` adopted
/// `P_INLINE.md` §6.6.2's budget model into `splice.rs`, so **C3** (the growth
/// budget's clamp) and **C19** (the charge) each gained an `R-derived`
/// counterpart with a cited, address-backed witness — `INLINE_BUDGET_FLOOR`
/// and `INLINE_CHARGE_EXEMPT_MAX`.
///
/// **C2 did NOT move, and its token did.** The ABSENCE check flagged C2 on a
/// *parameter name* of `BudgetModel::seed`, and flagged C19 on a *substring*
/// of an unrelated field name. Neither match is a counterpart, and `splice.rs`
/// says so at the site: *"the port has no honest caller instruction count to
/// pass"*. So on this check's first real firing it produced **one true
/// positive (C3), one right-answer-wrong-reason (C19), and one false positive
/// (C2)** — the mention-vs-counterpart blindness `#3641` names, which
/// `token_in_crates`'s own doc comment declares KNOWN AND NOT FIXED and leaves
/// to `w-inlmetric` to define. C2's token was re-pointed at a non-colliding
/// one; its `absent` verdict is unchanged and still true.
///
/// **Tokens are named here by clause id and never spelled**, per this file's
/// own rule above — spelling one in this comment makes the ABSENCE screen find
/// it in `crates/` and turns the row red. That happened at this merge, to the
/// person writing this sentence.
/// **MOVED AGAIN 2026-08-29**, from
/// `{'absent': 15, 'R-derived': 4, 'fitted': 2, 'unexercisable': 3}`, by lane
/// `w-inlclause` (board `#3796`–`#3801`).
///
/// **Two of the three moves are STALENESS, not conversion, and that is the
/// finding.** C14 and C18 have had counterparts in the port since
/// `w-inlbudget` landed on 2026-08-28 — each derived from a read, each
/// `PROV[R]` at the row's own address, and that lane's rung names both by
/// clause id. The rows stayed `absent` anyway, because the ABSENCE screen
/// checks the **one spelling the table happens to cite**.
///
/// So the blindness this file's header declares has a second direction, and it
/// was not declared anywhere: `#3641` and `token_in_crates`'s docstring both
/// describe a **mention** being read as a counterpart — a false positive. The
/// other half is a **counterpart adopted under a different name being read as
/// absence** — a false negative, which is silent, which nothing counts, and
/// which is why the column looked stuck. C3 and C19 converted at the wave-18
/// merge only because the adopting lane's chosen tokens happened to collide
/// with the table's.
///
/// The third move, **C15**, is an adoption in the ordinary sense: `w-inlclause`
/// put c2's `0x10b60a2f` arm into `splice.rs` under a required-zero byte delta,
/// byte-neutral because c2's own default switches the clause off.
/// **DELIBERATELY NOT MOVED 2026-08-29 by lane `w-clausegen`** (board `#3817`),
/// and the fact that it did not move is the lane's result.
///
/// That lane repaired the ABSENCE screen's false-negative half — a counterpart
/// adopted under a different name used to be invisible — and the repaired screen
/// flags **five** rows whose `absent`/`unexercisable` verdict sits beside a
/// `crates/` citation of the clause's own address. **Not one `state` cell was
/// edited.** A row that changes state because the screen changed is an
/// *instrument* result, not a conversion, and `#3505` is six for six on lanes
/// that moved a number by constructing one. The five rows are adjudicated by
/// clause id in `work/w-clausegen/RESULT.md` for the wave that owns adoption;
/// two of them (C4, C10) are the ones a reader should look at first.
///
/// So this constant is now load-bearing in **both** directions: it catches a
/// silent shrink (`#3748`) *and* it catches an instrument lane quietly
/// converting rows it was seamed not to touch.
/// **MOVED AGAIN 2026-08-30**, from
/// `{'absent': 12, 'R-derived': 7, 'fitted': 2, 'unexercisable': 3}`, by lane
/// `w-budget` (board `#3849`–`#3855`).
///
/// **Three rows, one missing link, and the link was READ by the previous
/// wave rather than guessed at here.** `w-instrcount` resolved the quantity
/// c2's inline decision tests to the `.gl` function record's `SIZE` field —
/// one writer in the whole image, produced by the front end and read by c2
/// without modification — and found **the port already decoded that field and
/// discarded the value**. This lane threaded it, so `B` is a number and the
/// caller-side clauses have their operands.
///
/// *(No address is spelled in this comment. This file's own doc comments are
/// inside `crates/`, so an address literal here **changes the very citation
/// footprint check 6 measures** — `cites_in_crates`' third declared blindness,
/// and it fired on this paragraph while it was being written.)*
///
/// **The first two are ordinary adoptions and the third was CONDITIONAL.** The
/// third puts a **new refusal on a production path**, and a refusal that fires
/// changes an emit — so its prereg registered it as adoptable *only if measured
/// not to fire*, with the negative result to be published if it did. Both
/// instruments say it does not: the gate's identity diff is 0 lines over 21
/// rows, and the 878-TU scan pair is identical over 566 `gap-metric` keys with
/// the workload stamp held before and after each arm.
///
/// **What did NOT move, and it is the more interesting half.** The lane audited
/// all twelve `absent` rows for a **second** blocker (`#3847`) and found ten of
/// them carry one, five with the second **binding** — so the published
/// one-cell-per-row partition names the cheaper obstacle on nearly half the
/// column. Two of those rows are marked *"read and derivable today"* and are
/// not: their input is a `[sym+0x4c]` bit above the low byte, and the `.gl`
/// `ATTR` reader takes the low byte only. **Not one of those ten changed
/// `state`** — a row with two blockers is exactly as `absent` as a row with
/// one, and what the audit moved is the price, not the verdict. The table
/// grew a `blocker2` column instead.
///
/// **Tokens are named here by clause id and never spelled**, per this file's
/// own rule — the three converted rows are C2, C16 and C17.
const SPLIT: &str = "{'absent': 9, 'R-derived': 10, 'fitted': 2, 'unexercisable': 3}";

/// The count of generated lines `gen_table.py` splices into `P_INLINE.md` §6.1
/// — the marker pair, the six-column header, 24 rows, the two count lines and
/// the provenance line. Compiled in for the same reason as [`ROWS`]: a
/// generator that silently emitted a shorter block would still print GREEN,
/// because a shorter block is exactly what the page would then contain.
const GENERATED_LINES: usize = 34;

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
    for f in [
        "work/w-inlmetric/check_table.py",
        "work/w-inlmetric/CLAUSES.tsv",
        "work/w-inlmetric/gen_table.py",
    ] {
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

    // THE SAME DENOMINATOR RULE FOR CHECK 6. `CITES` needs only the repo, so
    // unlike ALIGN/DECODE it can never legitimately SKIP — but it *can* grade
    // zero rows if the `cites` column is dropped, and it would print GREEN.
    assert!(
        report.contains(&format!("{ROWS} of {ROWS} compared")),
        "the CITES line did not report comparing {ROWS} of {ROWS} rows. Check 6 is \
         the false-negative half of the absence screen and a silent zero-row run \
         of it is the `#3470` failure.\n{report}"
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

/// **The control for check 6 (`CITES`), the false-negative half.**
///
/// `--set` overrides one cell without touching the tracked table. Planting a
/// citation footprint on a row that has none must redden — otherwise the whole
/// of `w-clausegen`'s deliverable 2 is a screen that cannot fire, which is
/// `#3470` and is worse than shipping nothing.
///
/// The row is named by id and its address is not written here: this file's own
/// doc comments are inside `crates/`, so an address literal added to them
/// *changes the very footprint check 6 measures*. That is not a defect to
/// suppress — see `cites_in_crates`' docstring — but it is a reason not to
/// scatter address literals through a test that grades them.
#[test]
fn check_six_goes_red_on_a_planted_citation_footprint() {
    if !python3_is_usable() {
        println!("SKIP: python3 absent — cannot run the CITES control");
        return;
    }
    let root = repo_root();
    let (ok, report) = run_checker(
        &root,
        &["--set", "C1.cites=crates/c2-core/src/splice.rs"],
    );
    assert!(
        report.contains("CONFORMANCE-CHECK: RED"),
        "CHECK 6 CANNOT FAIL. A planted citation footprint was graded GREEN \
         (exit-0 = {ok}), so the absence screen's false-negative half is still \
         open and the green above means nothing.\n{report}"
    );
    assert!(
        report.contains("C1(PLANTED): CITES"),
        "check 6 went RED but not on the planted row — it is failing for some \
         other reason and the control proves nothing.\n{report}"
    );
    assert!(!ok, "the checker printed RED and still exited 0");
    println!("CITES control RED as expected:\n{report}");
}

/// **`P_INLINE.md` §6.1 is a RENDERING of `CLAUSES.tsv`, and this is what makes
/// that true.**
///
/// Before this existed, the page and the table were the same instrument
/// published twice; they diverged at each of three hand re-syncs in three days
/// and `check_table.py` was GREEN through all three (`#3814`). Two copies of a
/// fact is two chances to update one (`#3679`); this makes the second copy
/// generated, so there is no second chance to miss.
#[test]
fn p_inline_section_six_one_is_generated_from_the_clause_table() {
    if !python3_is_usable() {
        println!("SKIP: python3 absent — cannot run gen_table.py");
        return;
    }
    let root = repo_root();
    let (ok, report) = run_tool(&root, "gen_table.py", &["--check"]);
    assert!(
        report.contains("TABLE-GEN: GREEN"),
        "P_INLINE.md §6.1 has DRIFTED from work/w-inlmetric/CLAUSES.tsv \
         (exit-0 = {ok}). Do not hand-edit between the markers — run \
         `python3 work/w-inlmetric/gen_table.py --write`.\n{report}"
    );
    // Denominator beside numerator (`#3470`), twice: the row count the table
    // was read at, and the line count the page was compared over. A generator
    // that emitted an empty block would agree with a page containing an empty
    // block, and print GREEN.
    assert!(
        report.contains(&format!("({ROWS} rows)")),
        "the generator did not read {ROWS} rows from the table.\n{report}"
    );
    assert!(
        report.contains(&format!("{GENERATED_LINES} generated lines match")),
        "the generated block is not {GENERATED_LINES} lines. If a row or a count \
         line was added or dropped on purpose, move GENERATED_LINES in the same \
         commit — that is what it is compiled in for.\n{report}"
    );
    println!("{report}");
}

/// **The control for the generator, and it is watched RED on a MUTATED COPY.**
///
/// `gen_table.py` takes the page path positionally precisely so this can exist
/// without touching the tracked file. A `--check` that has never been seen fail
/// is decoration (`#3336`), and `#3787` is the cautionary case in this repo:
/// `hatch.py check` printed the defect, printed `CLEAN`, and exited 0.
///
/// The mutation is one character inside the generated block, which is the
/// hardest case — a whole-row deletion would also be caught by the line count.
#[test]
fn the_generator_goes_red_when_the_page_is_edited_by_hand() {
    if !python3_is_usable() {
        println!("SKIP: python3 absent — cannot run the generator control");
        return;
    }
    let root = repo_root();
    let page = root.join("docs/whitebox/ref/P_INLINE.md");
    let text = std::fs::read_to_string(&page).expect("P_INLINE.md is readable");

    // Mutate the FIRST generated row: flip one clause id. Chosen because it
    // survives any future re-ordering of the columns.
    let needle = "\n| C1 | ";
    assert!(text.contains(needle), "the generated block has no C1 row");
    let mutated = text.replacen(needle, "\n| C1x | ", 1);
    assert_ne!(mutated, text, "the mutation did not change the page");

    let tmp = std::env::temp_dir().join(format!(
        "c2rs_clausegen_control_{}_{}.md",
        std::process::id(),
        GENERATED_LINES
    ));
    std::fs::write(&tmp, &mutated).expect("temp page is writable");
    let (ok, report) = run_tool(&root, "gen_table.py", &["--check", tmp.to_str().unwrap()]);
    let _ = std::fs::remove_file(&tmp);

    assert!(
        report.contains("TABLE-GEN: RED"),
        "THE GENERATOR'S --check CANNOT FAIL. A hand-edited page was graded \
         GREEN (exit-0 = {ok}), so §6.1 can still drift and the sibling test \
         proves nothing.\n{report}"
    );
    assert!(
        report.contains("C1x"),
        "--check went RED but did not name the edited row, so it is failing for \
         some other reason.\n{report}"
    );
    assert!(!ok, "--check printed RED and still exited 0");
    println!("generator control RED as expected:\n{report}");
}

/// **The MARKERS are the seam, and a page that lost them must not read GREEN.**
///
/// This is the failure mode a naive `--check` has: if the markers vanish (a
/// careless whole-section rewrite, a merge that drops a comment line), there is
/// nothing to compare, and "nothing differs" is the same string as "everything
/// matches". `gen_table.py` reports `MARKERS: MISSING` and RED; here it is
/// watched doing so.
#[test]
fn the_generator_goes_red_when_the_page_loses_its_markers() {
    if !python3_is_usable() {
        println!("SKIP: python3 absent — cannot run the marker control");
        return;
    }
    let root = repo_root();
    let page = root.join("docs/whitebox/ref/P_INLINE.md");
    let text = std::fs::read_to_string(&page).expect("P_INLINE.md is readable");
    let stripped: String = text
        .lines()
        .filter(|l| !l.contains("GENERATED 6.1"))
        .collect::<Vec<_>>()
        .join("\n");
    assert_ne!(stripped.len(), text.len(), "no marker lines were found to strip");

    let tmp = std::env::temp_dir()
        .join(format!("c2rs_clausegen_markers_{}.md", std::process::id()));
    std::fs::write(&tmp, &stripped).expect("temp page is writable");
    let (ok, report) = run_tool(&root, "gen_table.py", &["--check", tmp.to_str().unwrap()]);
    let _ = std::fs::remove_file(&tmp);

    assert!(
        report.contains("MARKERS: MISSING") && report.contains("TABLE-GEN: RED"),
        "a page with no generated block was not RED (exit-0 = {ok}). \
         'nothing to compare' must never render as 'nothing differs'.\n{report}"
    );
    assert!(!ok, "--check printed RED and still exited 0");
    println!("marker control RED as expected:\n{report}");
}
