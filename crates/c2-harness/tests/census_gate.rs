//! **The census/gate invariant** (roadmap #44), as a test — in **both** linkage
//! modes (roadmap #47), over **two populations**.
//!
//! Acceptance is supposed to live in the IL parser precisely so that
//! `IlBundle::function_census` — the public coverage numerator — and `PortC2`
//! cannot disagree about what is in class. They did: `int f(int a,int b,int c)
//! { return a + b*c; }` censused in class and the port returned
//! `NotImplemented`, because a `*` after the first operator was gated in codegen
//! where the census could not see it (`docs/IL_CALL_IN_EXPR.md` §24.7). On the
//! 878-TU workload that over-claim measured **9,230 functions, 2.24 % of the
//! numerator**, and none of it was the §24.7 shape.
//!
//! The gates have been moved, and this test is what keeps them moved: it runs
//! `PortC2`'s own per-function selector over **every function the census calls
//! in class** and requires the disagreement to stay at its recorded value.
//! `docs/GAPS.md` §6 states the general form — a diagnostic that runs outside
//! the parser needs a population whose answer is already known, and this is that
//! population: every in-class function, whose answer must be "accepted".
//!
//! # Why both modes, and what the second one was hiding
//!
//! `function_gate` takes a `fn_level_linking` flag, because `/Gy` (implied by
//! `/O1` and `/O2`, i.e. by the **entire real workload**) puts each function in
//! its own COMDAT `.text` and two of the port's refusals exist only in that
//! shape. This test used to run the `false` lane only — so it asserted the
//! invariant in the mode the fixtures capture in and said nothing at all about
//! the mode the workload compiles in. The gap scan *does* pass the real flags,
//! so the `/Gy` lane was live and ungated on every scan and merely happened to
//! read 0 there.
//!
//! # THE POPULATION WAS THE DEFECT — board #1304, lane `w-disagree`
//!
//! This test was **green on master while 42 of lane `w-midrun`'s 94 grid cells
//! were a live disagreement** (board **#1275**), two of them cells `w-carrier`
//! had committed to the repo days earlier. It was green because its population
//! was the 286 `fixtures/cpp` files and **not one of them spells
//! `h->m.f = &h->m;`** — the direct-spelling interior address. A test whose
//! population cannot contain the failure is not a test; it reports absence as
//! success, which is the shape this project has now recorded **fifteen** times
//! across eight instruments (boards #299, #1077, #1140, #1236).
//!
//! Two repairs, and only the second one generalizes:
//!
//! 1. **The population is widened** to the *generated sweep corpus* —
//!    `scripts/sweep_gen.py` over `scripts/sweep.d`, **19,556 cases** at the time
//!    of writing, enumerated over axes rather than hand-picked. That is the
//!    corpus that found board #232, and the argument for it is `docs/GAPS.md`'s:
//!    a hand-written corpus is systematically biased toward the shapes whoever
//!    wrote it was already thinking about, and *this test's* fixtures are the
//!    most hand-picked corpus in the repo.
//! 2. **The DISCRIMINATING-CELL COUNT is printed and asserted.** A count of
//!    disagreements says nothing about whether the run could have found one.
//!    A cell is **discriminating** when both verdicts exist and are reached
//!    independently: the census calls the function in class (so the port's answer
//!    is compared at all) **and** the port's answer is produced by
//!    `codegen::function_gate` running `select_function` on a real `IlFunction`,
//!    rather than by something upstream failing first. Those are the cells in
//!    which a disagreement *can* appear. Zero of them is a vacuous run and fails
//!    loudly; so does a count that has collapsed, and so does a population that
//!    has collapsed onto too few census shape keys — the failure a raw total
//!    cannot see.
//!
//! This is `scripts/sweep_shapes.py --check`'s method (assert the zero rows,
//! never merely report them) applied to the agreement check, with board #1140's
//! caution attached: a marker count is only as good as what it counts, so the key
//! here is the census's **own** `FnVerdict::key` and not a regex over source text.
//!
//! **The checks are POSITIVE and their ORDER is load-bearing.** None of them
//! enumerates a way the run can be empty; each demands that something in the
//! class that can fail was actually graded. They run emptiness → floor → breadth
//! → disagreement, because an earlier guard that fires first makes every later
//! assertion unreachable — the lane-registry trap, where a count floor fired and
//! the `/EH` and `/Oi` assertions never executed (`docs/GAPS.md` §7). Each
//! carries a message no other one can produce, so the assertion a red run drove
//! is identifiable from its first line.
//!
//! # What the wide lane pins, and why it is not an exact total
//!
//! The fixture lane pins **exact counts with named causes**, because 286 files
//! are stable and trading one refusal for another at an equal total is a real
//! change. The wide lane pins **the SET OF REFUSAL FAMILIES** instead, matched by
//! substring: the generated corpus grows every time any lane adds a
//! `scripts/sweep.d` fragment (14,484 → 14,817 → 19,556 in five days), so an
//! exact total would be a constant every unrelated lane bumps without reading —
//! and a constant everybody bumps is a constant nobody checks. A refusal leaking
//! into codegen introduces a **new family**, which fails; a family with no cases
//! left is a **closure**, which also fails, because a closure is a result and
//! belongs in a rung doc rather than in a silently shrinking number. The
//! per-family counts are printed on every run.
//!
//! **What the first wide run found, on an unmodified master:** 124 disagreements
//! in the packed lane and 127 under `/Gy`, in **three** families, where the
//! fixture corpus contains **one**. All three were live and none had ever been
//! seen. Boards **#1306**, **#1307**, **#1308**; none is a mis-emit.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use c2_core::codegen;
use c2_core::codegen::cfg_class::{self, CflowClass};
use c2_reference::Toolchain;

// ---------------------------------------------------------------------------
// The fixture lane's recorded disagreements.
// ---------------------------------------------------------------------------

/// The ONE disagreement left in the fixture corpus **without** `/Gy`, with its
/// cause.
///
/// `w13_fscratch.cpp`'s `fm13` — thirteen `float` parameters and twelve
/// multiplies — is refused by `float_leaf_text`'s FP scratch allocator, which
/// never retires a parameter from its live set, so thirteen parameters leave
/// exactly one free pool slot and the second temporary has nowhere to go. It is
/// a refusal, not a mis-emit, and it costs **0 functions on the 878-TU
/// workload**; moving it would mean lifting the whole FP register allocator into
/// the IL crate, which is a byte-visible refactor and is written up as a handoff
/// rather than done here.
///
/// The number is asserted rather than allow-listed away so that it cannot grow
/// quietly: a new gate landing in codegen instead of the parser fails this test.
const KNOWN_DISAGREEMENTS_PACKED: usize = 1;

/// The disagreements under **`/Gy`** (function-level linking), which is what the
/// real workload compiles with — `/O1` and `/O2` both imply it.
///
/// It is the `/Ox` residual above **plus the pooled floating-point constants**:
/// `coff::emit_comdat_obj` does not place the `.rdata` COMDAT a pooled FP
/// constant needs, so `function_gate` refuses every W13b body in the `/Gy`
/// shape. That is a refusal, not a mis-emit, and like the one above it costs
/// **0 functions on the 878-TU workload** (no `match`-class TU there carries a
/// pooled constant). Moving *this* gate means teaching the census about a
/// whole-obj layout decision, which is `c2-core`'s seam, not the harness's —
/// recorded here so the number cannot drift, not endorsed as where it belongs.
///
/// **8 → 11 pooled-constant entries (9 → 12 total) when the FP-leaf-beside-framed
/// pair landed**, and the three new ones are *not* a new refusal: they are three
/// more W13b bodies in the fixture corpus, hitting the same standing
/// `emit_comdat_obj` limit as the eight already here. `wunw_float_neg.cpp` gains
/// one and `w28_fp_store_framed_neg.cpp` two — those two fixtures are the
/// negatives that hold the pooled-constant half of the pair, so a pooled constant
/// is precisely what they have to contain. The `causes` table below pins them by
/// name, so trading one of these for a genuinely new refusal still fails even
/// though the total would not move.
const KNOWN_DISAGREEMENTS_GY: usize = 12;

// ---------------------------------------------------------------------------
// THE FLOORS. Each is a POSITIVE demand — the run must have graded something in
// the class that can fail — and none of them is an enumeration of the ways a run
// can come back empty. That is the mitigation this repo has watched fail fifteen
// times.
//
// Every floor sits BELOW its measured value with headroom, because its job is to
// catch a COLLAPSE (a generator that stopped emitting, a capture that started
// failing, a parser that stopped accepting) and not to be a second copy of the
// measurement. The measured values are printed on every run and recorded in
// `docs/rungs/2026-08-09-w-disagree.md` §3.
// ---------------------------------------------------------------------------

/// Fixture lane, discriminating cells. Measured **1,692** over 286 sources,
/// identical in both linkage modes — the mode changes the verdict, not the
/// population.
const FLOOR_FIXTURE_CELLS: usize = 1_200;

/// Fixture lane, distinct census shape keys among the discriminating cells.
/// Measured **35**.
const FLOOR_FIXTURE_SHAPES: usize = 25;

/// Wide lane, discriminating cells over the generated corpus. Measured
/// **14,275** over 19,467 captured of 19,556 generated cases.
///
/// **It is only 8.4x the fixture lane's, and it finds 124x as many
/// disagreements.** That ratio is the argument for this whole file: what a
/// generated corpus buys is BREADTH, not bulk. Sixteen thousand more cells found
/// three families the fixtures cannot express, not more of the one they can.
const FLOOR_WIDE_CELLS: usize = 10_000;

/// Wide lane, distinct census shape keys. Measured **31** — four FEWER than the
/// fixture corpus's 35, which is the same point from the other side: the
/// fixtures are denser in hand-chosen shapes and blind to whole constructs.
const FLOOR_WIDE_SHAPES: usize = 24;

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/cpp")
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/c2-harness/../.. is the repo root")
        .to_path_buf()
}

fn work(tag: &str) -> PathBuf {
    c2_harness::testsupport::scratch_dir("census-gate", tag)
}

fn sources_in(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "cpp"))
        .collect();
    v.sort();
    v
}

/// One population, one linkage mode.
#[derive(Default)]
struct Scan {
    /// Sources whose IL the front end produced at all.
    captured: usize,
    /// Functions the census calls in class.
    in_class: usize,
    /// **Discriminating cells** — in-class functions on which
    /// `codegen::function_gate` ran `select_function` to its own verdict. These
    /// are the cells in which a disagreement can appear.
    discriminating: usize,
    /// The discriminating cells, by the census's own shape key.
    shapes: BTreeMap<String, usize>,
    /// `"<source> :: <fn> :: <refusal>"` → count, for every disagreement.
    found: BTreeMap<String, usize>,
}

impl Scan {
    fn disagreements(&self) -> usize {
        self.found.values().sum()
    }
    fn listing(&self) -> String {
        self.found
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join("\n")
    }
    /// The refusal text of each disagreement, deduplicated — the wide lane's
    /// pinned quantity.
    fn causes(&self) -> BTreeMap<String, usize> {
        let mut c: BTreeMap<String, usize> = BTreeMap::new();
        for (k, n) in &self.found {
            let cause = k.rsplit(" :: ").next().unwrap_or(k).to_string();
            *c.entry(cause).or_insert(0) += *n;
        }
        c
    }
    fn merge(&mut self, other: Scan) {
        self.captured += other.captured;
        self.in_class += other.in_class;
        self.discriminating += other.discriminating;
        for (k, n) in other.shapes {
            *self.shapes.entry(k).or_insert(0) += n;
        }
        for (k, n) in other.found {
            *self.found.entry(k).or_insert(0) += n;
        }
    }
}

/// Cross-check one linkage mode over one list of sources.
fn cross_check(tc: &Toolchain, gy: bool, sources: &[PathBuf], work_root: &Path) -> Scan {
    let mut s = Scan::default();
    for (i, cpp) in sources.iter().enumerate() {
        let dir = work_root.join(format!("f{i:05}"));
        let _ = std::fs::create_dir_all(&dir);
        let Ok(bundle) = tc.capture_il(cpp, &dir) else {
            // A source the front end declines is not this test's business. The
            // generated corpus has ~96 of these (board #281) and the fixtures
            // none; either way they are counted OUT of `captured`, so a corpus
            // that stopped compiling can never pad a floor.
            let _ = std::fs::remove_dir_all(&dir);
            continue;
        };
        s.captured += 1;
        if let Some(rows) = bundle.census_functions() {
            for (f, gate) in &rows {
                if !f.verdict.in_class() {
                    continue;
                }
                s.in_class += 1;
                // Fixtures capture at the default `/Ox`, which does not imply
                // `/Gy`; the mode itself is read per function from `.ex`, and the
                // linkage shape is the argv fact the bundle cannot record — which
                // is exactly why it is a parameter here.
                let refusal = match gate {
                    // `shape_to_function` refused: the port never reached its own
                    // dispatch, so this cell is a disagreement but NOT
                    // discriminating. Keeping the two counters apart is what lets
                    // the discriminating floor fire on a tree where every cell
                    // fails upstream — the case in which a disagreement total is
                    // large and completely uninformative.
                    Err(e) => Some((*e).to_string()),
                    Ok(func) => match codegen::opt_mode_of_word(f.opt_word) {
                        Err(e) => Some(e.to_string()),
                        Ok(mode) => {
                            s.discriminating += 1;
                            *s.shapes.entry(f.verdict.key()).or_insert(0) += 1;
                            codegen::function_gate(func, mode, gy).err().map(|e| e.to_string())
                        }
                    },
                };
                if let Some(r) = refusal {
                    let name = f.name.clone().unwrap_or_else(|| format!("#{}", f.index));
                    *s.found
                        .entry(format!(
                            "{} :: {name} :: {r}",
                            cpp.file_name().unwrap().to_string_lossy()
                        ))
                        .or_insert(0) += 1;
                }
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
    s
}

/// [`cross_check`] across `jobs` threads, each with its own capture work dir.
///
/// The split is a STRIDE and not a prefix, for `scripts/expr_sweep.sh`'s own
/// reason: the case list is sorted by fragment name, so contiguous chunks would
/// give one worker a whole fragment and skew nothing but the wall clock — until
/// somebody uses a partial result, at which point it is `head -400` covering one
/// fragment of 62 all over again.
fn cross_check_par(tc: &Toolchain, gy: bool, sources: &[PathBuf], tag: &str, jobs: usize) -> Scan {
    let root = work(tag);
    let mut out = Scan::default();
    if jobs <= 1 || sources.len() < jobs {
        out = cross_check(tc, gy, sources, &root);
    } else {
        let chunks: Vec<Vec<PathBuf>> = (0..jobs)
            .map(|w| sources.iter().skip(w).step_by(jobs).cloned().collect::<Vec<_>>())
            .collect();
        let parts: Vec<Scan> = std::thread::scope(|sc| {
            let hs: Vec<_> = chunks
                .iter()
                .enumerate()
                .map(|(w, c)| {
                    let wr = root.join(format!("w{w}"));
                    sc.spawn(move || cross_check(tc, gy, c, &wr))
                })
                .collect();
            hs.into_iter().map(|h| h.join().expect("worker")).collect()
        });
        for p in parts {
            out.merge(p);
        }
    }
    let _ = std::fs::remove_dir_all(&root);
    out
}

/// **The positive checks, in the order that keeps each one reachable.**
///
/// See the module doc: emptiness → floor → breadth, each with a message no other
/// one can produce. A mutation that drives any one of them is identifiable from
/// the first line of the panic.
fn assert_population_can_fail(label: &str, gy: bool, s: &Scan, cells: usize, shapes: usize) {
    assert!(
        s.captured > 0,
        "POPULATION EMPTY [{label}, fn_level_linking={gy}]: not one source in this \
         population produced IL, so the agreement check graded nothing and would \
         have passed by absence. This is the instrument, not the port."
    );
    assert!(
        s.discriminating > 0,
        "NO DISCRIMINATING CELLS [{label}, fn_level_linking={gy}]: {} sources \
         captured and {} functions censused in class, but codegen::function_gate \
         reached its own verdict on ZERO of them — so no cell in this run could \
         have produced a disagreement, and a disagreement count taken over it says \
         only that the run was vacuous. This is the instrument, not the port.",
        s.captured,
        s.in_class
    );
    assert!(
        s.discriminating >= cells,
        "DISCRIMINATING CELLS COLLAPSED [{label}, fn_level_linking={gy}]: {} of {} \
         in-class functions reached codegen::function_gate, below the floor of \
         {cells}. The population this check runs over has shrunk, so its \
         disagreement count is not comparable with the recorded one. Raise the \
         floor deliberately, or find what stopped being generated, captured or \
         accepted.",
        s.discriminating,
        s.in_class
    );
    assert!(
        s.shapes.len() >= shapes,
        "DISCRIMINATING BREADTH COLLAPSED [{label}, fn_level_linking={gy}]: the {} \
         discriminating cells span only {} distinct census shape keys, below the \
         floor of {shapes}. A population concentrated on a few shapes cannot \
         contain a disagreement in any other, which is exactly how this test \
         stayed green over board #1275's 42 live cells. Keys: {:?}",
        s.discriminating,
        s.shapes.len(),
        s.shapes.keys().collect::<Vec<_>>()
    );
}

fn report(label: &str, gy: bool, s: &Scan) {
    println!(
        "census/gate [{label}, fn_level_linking={gy}]: {} captured, {} in class, \
         {} DISCRIMINATING cells over {} shape keys, {} disagreements",
        s.captured,
        s.in_class,
        s.discriminating,
        s.shapes.len(),
        s.disagreements()
    );
    for (cause, n) in s.causes() {
        println!("    {n:6}  {cause}");
    }
    // **The whole listing, on demand.** A cause histogram says which families
    // over-claim; it does not say which *cases*, and the case names are what a
    // successor needs to reproduce one. Off by default because 124 lines of
    // generated case names in every `cargo test` is noise, and named rather than
    // dumped to a fixed path because `work/` belongs to whoever ran the test.
    if let Ok(p) = std::env::var("C2RS_CENSUS_GATE_DUMP") {
        let path = format!("{p}.{label}.gy{}.txt", u8::from(gy));
        let body: String = s
            .found
            .iter()
            .map(|(k, n)| format!("{n}\t{k}\n"))
            .collect();
        std::fs::write(&path, body).unwrap_or_else(|e| panic!("write {path}: {e}"));
        println!("    -> {} disagreement rows written to {path}", s.found.len());
    }
}

#[test]
fn the_census_and_the_port_agree_about_what_is_in_class() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };

    let sources = sources_in(&fixtures_dir());
    assert!(!sources.is_empty(), "no fixtures found");

    // Both lanes, at their recorded values. `/Gy` is the mode the whole 878-TU
    // workload compiles in (`/O1` implies it), so leaving it unasserted left the
    // invariant unmeasured exactly where it is load-bearing.
    for (gy, expected, causes) in [
        (
            false,
            KNOWN_DISAGREEMENTS_PACKED,
            &[("no free FP scratch register", 1usize)][..],
        ),
        (
            true,
            KNOWN_DISAGREEMENTS_GY,
            &[
                ("no free FP scratch register", 1),
                ("pooled floating-point constant under function-level linking", 11),
            ][..],
        ),
    ] {
        let s = cross_check_par(&tc, gy, &sources, if gy { "gy" } else { "packed" }, 1);
        report("fixtures", gy, &s);
        assert_population_can_fail("fixtures", gy, &s, FLOOR_FIXTURE_CELLS, FLOOR_FIXTURE_SHAPES);

        let n = s.disagreements();
        assert_eq!(
            n, expected,
            "census/gate disagreement changed with fn_level_linking={gy} ({} \
             functions in class across the fixture corpus, {} of them \
             discriminating). Every entry is a function the census counts and \
             PortC2 refuses, i.e. an error term on the published coverage \
             numerator — move the gate into the IL parser (see docs/GAPS.md §6). \
             Found:\n{}",
            s.in_class,
            s.discriminating,
            s.listing()
        );
        // A total is not a diagnosis. Pin the *causes* too, so a residual that
        // stays at 12 while one refusal is traded for a different one — a real
        // change, invisible to the count — still fails.
        for (cause, want) in causes {
            let got: usize = s
                .found
                .iter()
                .filter(|(k, _)| k.contains(cause))
                .map(|(_, n)| *n)
                .sum();
            assert_eq!(
                got, *want,
                "the `{cause}` refusal moved from {want} to {got} with \
                 fn_level_linking={gy}. Found:\n{}",
                s.listing()
            );
        }
        let named: usize = causes
            .iter()
            .map(|(cause, _)| {
                s.found
                    .iter()
                    .filter(|(k, _)| k.contains(cause))
                    .map(|(_, n)| *n)
                    .sum::<usize>()
            })
            .sum();
        assert_eq!(
            named, n,
            "an UNNAMED census/gate refusal appeared with fn_level_linking={gy} — the \
             count still matches but the causes do not. Found:\n{}",
            s.listing()
        );
    }
}

// ---------------------------------------------------------------------------
// The wide lane — the population that CAN contain the failure.
// ---------------------------------------------------------------------------

/// **THE THREE CENSUS OVER-CLAIMS THE GENERATED CORPUS CONTAINS, PACKED LANE.**
///
/// Every one of them was live on master and **invisible to every standing
/// instrument** until this lane widened the population — 124 functions the census
/// counts in class and `PortC2` refuses, where the 286 fixtures contain exactly
/// **one**. None is a mis-emit; `mismatch` is 0 and none of these bodies has ever
/// reached an obj. What is wrong is the *published numerator*.
///
/// Matched as **substrings**, not as whole refusal texts. A refusal's sentence is
/// prose that its owning lane rewrites when it learns something, and a check that
/// went red on a reworded comment would be a check people route around. The
/// substring is the part that names the family.
///
/// | substring | count | who owns it |
/// |---|---:|---|
/// | interior address BESIDE another producer | **110** | board **#1306** — the mixed-kind allocation rule, deliberately left in class so a reader refusal cannot make it unreachable (#1291's shape) |
/// | store-run-before-a-call … materialises nothing | **12** | board **#867**'s slot rule, board **#1307** |
/// | bitwise or shift operand that is not a bare register | **2** | `cmp_shift_or`'s immediate forms, board **#1308** |
///
/// **The three cases `scripts/sweep_mode.sh` carries as its baseline are NOT
/// here**, and that is a seam fact worth keeping: that script grades through
/// `c2rs gap`, which asks `IlBundle::functions()` a **whole-TU** question, while
/// this test asks `census_functions()` + `function_gate` a **per-function** one.
/// Two instruments, one name, disjoint findings.
const WIDE_CAUSES_PACKED: &[&str] = &[
    "a store run with an interior address BESIDE another producer",
    "a store-run-before-a-call whose run materialises nothing",
    "a bitwise or shift operand that is not a bare register",
];

/// The packed set plus the pooled-constant refusal, which only `/Gy` raises.
///
/// **It is 3 cases over 19,467, against 11 over 286 fixtures** — the opposite of
/// what a wider corpus was expected to do to it, and worth stating because the
/// prereg registered `>= 150` in the other direction. The fixture corpus is
/// deliberately dense in `float` leaves carrying pooled constants (they are what
/// `w13_fscratch` and the W13b family exist to test); the generated corpus
/// enumerates FP *shapes* and rarely needs a pooled constant at all. A generated
/// corpus is broader, not uniformly denser.
const WIDE_CAUSES_GY: &[&str] = &[
    "a store run with an interior address BESIDE another producer",
    "a store-run-before-a-call whose run materialises nothing",
    "a bitwise or shift operand that is not a bare register",
    "pooled floating-point constant under function-level linking",
];

/// Generate the sweep corpus into `out`. `None` when python3 is absent — the
/// portable lane must degrade, never fail.
fn generate_wide(out: &Path) -> Option<usize> {
    let root = repo_root();
    let res = Command::new("python3")
        .arg(root.join("scripts/sweep_gen.py"))
        .arg(out)
        .arg(root.join("scripts/sweep.d"))
        .output();
    let output = match res {
        Ok(o) => o,
        Err(e) => {
            println!("SKIP: the wide census/gate lane needs python3 to generate its corpus ({e})");
            return None;
        }
    };
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(
        output.status.success(),
        "scripts/sweep_gen.py failed — the wide census/gate lane will not run over a \
         corpus that did not generate:\n{stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    // `"62 fragments, 19556 cases total (19556 .cpp on disk)"`.
    //
    // **A parse failure here is a HARD ERROR and not a `None`, and the reason is
    // that this function returned `None` on its very first run.** The first
    // revision took the second whitespace token instead of the third, the parse
    // failed, `None` flowed into the caller's `let … else { return }`, and the
    // wide lane — the whole point of this file — reported `ok` in 0.0s having
    // graded nothing. That is the sixteenth instance of absence reading as
    // success in this repo and it happened *inside the fix for the fifteenth*.
    // So the two outcomes are separated by construction: `None` means and can
    // only mean "python3 is absent", which is the one case that may degrade.
    let total = stdout.lines().find(|l| l.contains("cases total")).unwrap_or_else(|| {
        panic!(
            "scripts/sweep_gen.py printed no `cases total` line — the wide \
             census/gate lane cannot size its own corpus and will not report a \
             pass over one it did not count:\n{stdout}"
        )
    });
    // The FIRST token of that line is the FRAGMENT count — smaller, and a
    // perfectly plausible-looking corpus size. Take the number that immediately
    // precedes the word `cases`, and refuse rather than guess.
    let words: Vec<&str> = total.split_whitespace().collect();
    let k = words
        .iter()
        .position(|w| *w == "cases")
        .and_then(|i| i.checked_sub(1))
        .and_then(|i| words[i].parse::<usize>().ok())
        .unwrap_or_else(|| panic!("no case count before `cases` in `{total}`"));
    Some(k)
}

/// **The agreement check over a population that can contain the failure.**
///
/// `C2RS_CENSUS_GATE_WIDE=0` skips it while iterating; it is on by default and a
/// skip prints a named reason, never a silent pass. `C2RS_CENSUS_GATE_JOBS` sets
/// the capture parallelism (default `available_parallelism`, capped at 16).
#[test]
fn the_census_and_the_port_agree_over_the_generated_corpus() {
    if std::env::var("C2RS_CENSUS_GATE_WIDE").as_deref() == Ok("0") {
        println!("SKIP: C2RS_CENSUS_GATE_WIDE=0 — the wide lane was disabled by env");
        return;
    }
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };

    let out = work("wide-corpus");
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).unwrap();
    let Some(generated) = generate_wide(&out) else {
        let _ = std::fs::remove_dir_all(&out);
        return;
    };
    let sources = sources_in(&out);
    assert!(
        generated > 0 && sources.len() == generated,
        "the generator printed {generated} cases and {} are on disk — the wide lane \
         refuses to run over a corpus it cannot account for (the counter bug that \
         overwrote 1,233 cases, docs/ARCHITECTURE_SEAMS.md §2.4)",
        sources.len()
    );

    let jobs: usize = std::env::var("C2RS_CENSUS_GATE_JOBS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| {
            std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).min(16)
        })
        .max(1);

    for (gy, want_causes) in [(false, WIDE_CAUSES_PACKED), (true, WIDE_CAUSES_GY)] {
        let s = cross_check_par(&tc, gy, &sources, if gy { "wgy" } else { "wpacked" }, jobs);
        report("generated", gy, &s);
        assert_population_can_fail("generated", gy, &s, FLOOR_WIDE_CELLS, FLOOR_WIDE_SHAPES);

        // **The SET of families, not the total** — see the module doc. Matched by
        // substring, and BOTH directions are asserted: an unrecorded family is a
        // refusal that leaked into codegen, a recorded family with no cases left
        // is a closure, and a closure is a result that belongs in a rung doc
        // rather than in a silently-shrinking number.
        let counts = s.causes();
        let unrecorded: Vec<&String> = counts
            .keys()
            .filter(|c| !want_causes.iter().any(|w| c.contains(w)))
            .collect();
        assert!(
            unrecorded.is_empty(),
            "A NEW census/gate refusal family appeared over the generated corpus \
             (fn_level_linking={gy}):\n  {}\nThe census counts these functions in \
             class and PortC2 refuses them, so the published coverage numerator \
             over-claims by that many. It is NOT a mis-emit — no bytes were \
             produced — but acceptance belongs in the IL parser (docs/GAPS.md §6). \
             Family counts this run: {counts:?}",
            unrecorded
                .iter()
                .map(|c| c.as_str())
                .collect::<Vec<_>>()
                .join("\n  ")
        );
        let empty: Vec<&&str> = want_causes
            .iter()
            .filter(|w| !counts.keys().any(|c| c.contains(**w)))
            .collect();
        assert!(
            empty.is_empty(),
            "A RECORDED census/gate refusal family has NO CASES LEFT over the \
             generated corpus (fn_level_linking={gy}): {empty:?}\nEither it was \
             closed — a result, which belongs in a rung doc and in WIDE_CAUSES_* — \
             or the corpus stopped generating the shape, which is the population \
             defect this whole file exists for. Family counts this run: {counts:?}"
        );
        // A family list is not a diagnosis either: nothing above would notice one
        // family's cases being re-attributed to another. `BTreeSet` here only to
        // state that the recorded families are distinct.
        let want: BTreeSet<&str> = want_causes.iter().copied().collect();
        assert_eq!(
            want.len(),
            want_causes.len(),
            "WIDE_CAUSES_* lists a family twice, so one of them can never be the \
             reason a run went red"
        );
    }
    let _ = std::fs::remove_dir_all(&out);
}

// ===========================================================================
// W-CFGCLASS — the emitter CFG-class registry, graded (CEILING §6.1 phase 1)
// ===========================================================================

/// **The floor for the registry grading.** Below its measured value with
/// headroom, for `FLOOR_FIXTURE_CELLS`' reason: a floor's job is to fail a run
/// that collapsed, not to pin a number that moves whenever a fixture lands.
const FLOOR_REGISTRY_CELLS: usize = 1_200;
/// How many distinct `Lowering` arms the fixtures must exercise. A registry
/// graded on two arms says nothing about the other thirty-three.
const FLOOR_REGISTRY_LOWERINGS: usize = 10;

/// One run of the CFG-class registry against `select_function` and the census.
#[derive(Default)]
struct RegistryScan {
    /// Rows where the census produced an `IlFunction` at all — the population
    /// in which either direction of disagreement can appear.
    cells: usize,
    /// Cells where `select_function` accepted **and** `lowering_of` named an
    /// arm: the ones on which a CFG class is graded.
    graded: usize,
    /// `lowering_of` named an arm where `select_function` REFUSED — the
    /// **over-claiming** direction, board #3270's.
    over: BTreeMap<String, usize>,
    /// `select_function` accepted where `lowering_of` named nothing — the
    /// under-claiming direction.
    under: BTreeMap<String, usize>,
    /// `class_of(lowering)` disagreed with the census's own `cflow` string.
    wrong_class: BTreeMap<String, usize>,
    /// Cells the census never gave a control-flow class (the `cf-expr-…` bail).
    /// Not a disagreement — the census did not answer.
    unclassified: usize,
    /// The observed `(lowering, census cflow)` cross-tab. **The table the
    /// registry's declarations were derived from**, printed on every run so a
    /// declaration can never quietly stop matching what is measured.
    cross: BTreeMap<String, usize>,
    /// `"<lowering> <class>"` for every declared pair a capture actually
    /// produced — the input to the over-claim check. `classes_of` asserts
    /// *observed ⊆ declared*; this set is what makes *declared ⊆ observed*
    /// askable, and without it listing all seven classes everywhere would pass.
    observed: BTreeSet<String>,
}

impl RegistryScan {
    fn merge(&mut self, o: RegistryScan) {
        self.cells += o.cells;
        self.graded += o.graded;
        self.unclassified += o.unclassified;
        for (m, src) in [
            (&mut self.over, o.over),
            (&mut self.under, o.under),
            (&mut self.wrong_class, o.wrong_class),
            (&mut self.cross, o.cross),
        ] {
            for (k, n) in src {
                *m.entry(k).or_insert(0) += n;
            }
        }
        self.observed.extend(o.observed);
    }
    fn lowerings_seen(&self) -> usize {
        self.cross
            .keys()
            .filter_map(|k| k.split_once(" -> ").map(|(l, _)| l))
            .collect::<BTreeSet<_>>()
            .len()
    }
}

/// A census row's symbol name, or its index when the binding gave it none —
/// the row is still a cell and dropping it would shrink the population that can
/// fail, which is this file's own recorded defect.
fn fname(f: &c2_il::FnCensus) -> String {
    f.name.clone().unwrap_or_else(|| format!("#{}", f.index))
}

/// One capture profile. **Two of them are run**, and that is not thoroughness
/// for its own sake: the first version of this test ran the default profile
/// alone and exercised **15 of 35** `Lowering` arms, because most of the
/// branchy transcriptions (`if_call_join`, `guard_ret_chain`, every loop) are
/// admitted by their readers at `/O1` **only** — which is the mode the entire
/// dc3 workload compiles in and the mode the fixtures do *not* capture in by
/// default. A registry graded at `/Ox` alone would have declared twenty arms
/// and measured none of them.
const PROFILES: &[(&str, &[&str])] = &[
    ("Ox", &["/Ox", "/GS-", "/c"]),
    ("O1", &["/O1", "/GS-", "/c"]),
];

fn registry_scan(
    tc: &Toolchain,
    sources: &[PathBuf],
    work_root: &Path,
    flags: &[&str],
) -> RegistryScan {
    let flags: Vec<String> = flags.iter().map(|f| (*f).to_string()).collect();
    let mut s = RegistryScan::default();
    for (i, cpp) in sources.iter().enumerate() {
        let dir = work_root.join(format!("r{i:05}"));
        let _ = std::fs::create_dir_all(&dir);
        let Ok(bundle) = tc.capture_il_flags(cpp, &dir, &flags, None) else {
            let _ = std::fs::remove_dir_all(&dir);
            continue;
        };
        let name = cpp.file_name().unwrap_or_default().to_string_lossy().to_string();
        if let Some(rows) = bundle.census_functions() {
            for (f, gate) in &rows {
                let Ok(func) = gate else { continue };
                let Ok(mode) = codegen::opt_mode_of_word(f.opt_word) else {
                    continue;
                };
                s.cells += 1;
                let accepted = codegen::select_function(func, mode).is_ok();
                let low = cfg_class::lowering_of(func, mode);
                match (accepted, low) {
                    (false, Some(l)) => {
                        *s.over
                            .entry(format!("{name} :: {} :: {}", fname(f), l.name()))
                            .or_insert(0) += 1;
                    }
                    (true, None) => {
                        *s.under
                            .entry(format!("{name} :: {}", fname(f)))
                            .or_insert(0) += 1;
                    }
                    (true, Some(l)) => {
                        s.graded += 1;
                        *s.cross
                            .entry(format!("{} -> {}", l.name(), f.cflow))
                            .or_insert(0) += 1;
                        match CflowClass::from_census_str(&f.cflow) {
                            None => s.unclassified += 1,
                            Some(c) if !cfg_class::emits(l, c) => {
                                *s.wrong_class
                                    .entry(format!(
                                        "{name} :: {} :: {} declares [{}], census says {}",
                                        fname(f),
                                        l.name(),
                                        cfg_class::classes_of(l)
                                            .iter()
                                            .map(|c| c.short())
                                            .collect::<Vec<_>>()
                                            .join(" "),
                                        f.cflow
                                    ))
                                    .or_insert(0) += 1;
                            }
                            Some(c) => {
                                s.observed.insert(format!("{} {}", l.name(), c.short()));
                            }
                        }
                    }
                    (false, None) => {}
                }
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
    s
}

/// **CEILING §6.1 phase 1 — the emitter's CFG-class registry, graded in three
/// directions at once.**
///
/// `c2_core::codegen::cfg_class` declares, per dispatch arm of
/// `select_function`, which `cflow-*` class that arm emits. Three things can be
/// wrong with such a declaration and this test asks about all three, because
/// the mirror it replaces was wrong in the first two and nothing asked:
///
/// 1. **Over-claim** — `lowering_of` names an arm for a function
///    `select_function` refuses. Board **#3270**: the unsound direction is the
///    one that can be free in the metric used to choose a predicate.
/// 2. **Under-claim** — `select_function` accepts and `lowering_of` names
///    nothing.
/// 3. **Wrong class** — the arm is right and `class_of` disagrees with the
///    census's own `cflow` string for that body.
///
/// The observed `(lowering -> census class)` cross-tab is **printed on every
/// run**. The declarations in `class_of` were derived from it; printing it is
/// what makes a silently-drifting declaration visible rather than a comment
/// nobody re-checks — which is the failure this whole module exists to fix.
///
/// **Positive floors, in the order the other tests here use**: emptiness, then
/// cell floor, then breadth, then the three disagreement counts. A run that
/// captured nothing must fail on the first, not read green on the last.
#[test]
fn the_emitter_cfg_class_registry_agrees_with_select_function_and_the_census() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    let sources = sources_in(&fixtures_dir());
    assert!(!sources.is_empty(), "no fixtures found");
    let mut s = RegistryScan::default();
    for (tag, flags) in PROFILES {
        let root = work(&format!("cfgclass-{tag}"));
        let one = registry_scan(&tc, &sources, &root, flags);
        eprintln!(
            "  profile {tag:3} : {} cells, {} graded, {} arms",
            one.cells,
            one.graded,
            one.lowerings_seen()
        );
        s.merge(one);
        let _ = std::fs::remove_dir_all(&root);
    }

    eprintln!("\ncfg-class registry — fixtures, both capture profiles");
    eprintln!("  cells (census produced an IlFunction) : {}", s.cells);
    eprintln!("  graded (accepted AND named)           : {}", s.graded);
    eprintln!("  distinct Lowering arms exercised      : {}", s.lowerings_seen());
    eprintln!("  census gave no cflow class            : {}", s.unclassified);
    eprintln!("  OVER-claim  (named, gate refused)     : {}", s.over.len());
    eprintln!("  UNDER-claim (gate accepted, unnamed)  : {}", s.under.len());
    eprintln!("  WRONG-class (declaration vs census)   : {}", s.wrong_class.len());
    eprintln!("  observed lowering -> census class cross-tab:");
    for (k, n) in &s.cross {
        eprintln!("    {n:6}  {k}");
    }

    // 1. emptiness — before any count is trusted.
    assert!(
        s.cells > 0,
        "the registry grading captured nothing: 0 cells. A green run over an \
         empty population is the failure mode this file's header names."
    );
    // 2. the cell floor.
    assert!(
        s.cells >= FLOOR_REGISTRY_CELLS,
        "registry grading ran on {} cells, floor {FLOOR_REGISTRY_CELLS} — the \
         population collapsed",
        s.cells
    );
    // 3. breadth — a registry graded on a handful of arms grades nothing.
    assert!(
        s.lowerings_seen() >= FLOOR_REGISTRY_LOWERINGS,
        "only {} distinct Lowering arms were exercised, floor \
         {FLOOR_REGISTRY_LOWERINGS}",
        s.lowerings_seen()
    );
    // 4a. the OVER-claiming direction, first, because it is the unsound one.
    assert_eq!(
        s.over.len(),
        0,
        "cfg_class::lowering_of named an arm for {} function(s) \
         select_function REFUSES — the over-claiming direction:\n{}",
        s.over.len(),
        s.over.keys().cloned().collect::<Vec<_>>().join("\n")
    );
    // 4b. the under-claiming direction.
    assert_eq!(
        s.under.len(),
        0,
        "select_function accepted {} function(s) cfg_class::lowering_of names \
         no arm for:\n{}",
        s.under.len(),
        s.under.keys().cloned().collect::<Vec<_>>().join("\n")
    );
    // 4c. the declarations themselves — observed ⊆ declared.
    assert_eq!(
        s.wrong_class.len(),
        0,
        "{} body/bodies whose census CFG class is not in the arm's declared \
         set:\n{}",
        s.wrong_class.len(),
        s.wrong_class.keys().cloned().collect::<Vec<_>>().join("\n")
    );
    // 4d. **declared ⊆ observed, for every arm the fixtures exercise.** Without
    // this the safe move is to declare all seven classes on every arm, which
    // passes 4c and says nothing. Restricted to exercised arms on purpose: an
    // arm no fixture reaches is UNMEASURED, and reporting it as an over-claim
    // would be absence read as failure — the mirror image of this repo's most
    // repeated defect.
    let exercised: BTreeSet<&str> = s
        .cross
        .keys()
        .filter_map(|k| k.split_once(" -> ").map(|(l, _)| l))
        .collect();
    let mut unobserved: Vec<String> = Vec::new();
    for l in cfg_class::Lowering::ALL {
        if !exercised.contains(l.name()) {
            continue;
        }
        for c in cfg_class::classes_of(l) {
            if !s.observed.contains(&format!("{} {}", l.name(), c.short())) {
                unobserved.push(format!("{} declares {} and never emitted one", l.name(), c.short()));
            }
        }
    }
    assert!(
        unobserved.is_empty(),
        "{} declared (lowering, class) pair(s) on EXERCISED arms that no \
         capture produced — the over-claiming direction of a set-valued \
         declaration:\n{}",
        unobserved.len(),
        unobserved.join("\n")
    );
    // 5. and the arms no fixture reaches at all, named rather than counted, so
    // the registry's measured fraction is on the record every run.
    let unmeasured: Vec<&str> = cfg_class::Lowering::ALL
        .iter()
        .map(|l| l.name())
        .filter(|n| !exercised.contains(n))
        .collect();
    eprintln!(
        "  UNMEASURED arms ({} of {}): {}",
        unmeasured.len(),
        cfg_class::Lowering::ALL.len(),
        unmeasured.join(" ")
    );
}

/// **The screen's list is the registry's `Whole` claims and nothing else.**
///
/// `c2_harness::gap::factors::PORT_CFG_CLASSES` is built from
/// `cfg_class::CflowClass::census_str`, so the census spellings are typed once.
/// What a shared spelling cannot check is *which classes* are on the list, and
/// that is this test: the screen's class set must equal the registry's `Whole`
/// claims, in order.
///
/// This is the construct rung's identity. It is what makes the registry's
/// arrival a **re-expression** — `cfg-reach-shipped` cannot move unless a lane
/// deliberately promotes a `Partial` claim to `Whole`, at which point this test
/// and the screen move together instead of drifting apart.
///
/// Portable: no toolchain, no capture.
#[test]
fn port_cfg_classes_is_exactly_the_registrys_whole_claims() {
    let shipped: Vec<&str> = c2_harness::gap::port_cfg_classes()
        .iter()
        .map(|e| e.class)
        .collect();
    assert_eq!(
        shipped,
        cfg_class::whole_claim_census_strings(),
        "PORT_CFG_CLASSES and cfg_class::SHIPPED_CFG_CLAIMS disagree about \
         which CFG classes the port claims wholesale"
    );
    // And every shipped entry is unrestricted — the `CfgSub::Whole` end of
    // board #778. A `Keys` entry reaching the screen through a `Whole` claim
    // would be a partial claim published as a total one.
    for e in c2_harness::gap::port_cfg_classes() {
        assert!(
            !e.is_restricted(),
            "{} is shipped restricted; a Whole claim cannot produce one",
            e.class
        );
    }
}
