# REFACTOR REVIEW — the harness, the tests, the scripts: warranty vs mass

    Status:    second architecture review, deliberately scoped to what
               ARCHITECTURE_PROPOSAL_2026-08-20.md did not examine:
               c2-harness factoring, test architecture, scripts/, code
               quality, crate boundaries. Analysis and design only —
               nothing in crates/ or scripts/ moved by this document.
    Date:      2026-08-20
    Basis:     tree c277d3bb0 (clean master), STATUS generated block of
               2026-08-19 (match 26 / mismatch 0 / vocab-gap 844),
               rungs/2026-08-19-suitecost.md (merged d8e80d837),
               BOARD #1406 / #3134 / #3219 / #3226 / #3247,
               and the code cited inline. No gate run, no full suite run
               (box under external load); all measurements quoted are
               from committed rungs or from read-only inspection.
    Judge:     unchanged and untouchable. Nothing here proposes a second
               judge, moves an acceptance gate, or touches
               crates/{c2-il,c2-core,c2-obj,c2-reference}. Every move in
               §8 is instrument/test/script work with a required-zero
               effect on match/mismatch/census by construction.

---

## 0. The most important thing: the defect family is live in the warranty layer, right now, in three places — and one of them is being paid for and discarded

The repo's defining defect family is **absence read as success** (~15 recorded
instances; STATUS trap 5). All three findings below are instances of it sitting
*inside the instruments*, none needs a design, and together they cost less than
one lane-day to close.

**0.1 The suite's most expensive test executes the corpus-wide byte gate and
then throws the verdict away.** The three tests that are 99.1 % of
`cli_flags.rs` — `…_accepted_selftest`, `…_accepted_bench`, `…_accepted_perf`
(`crates/c2-harness/tests/cli_flags.rs:1036-1051`) — run `c2rs selftest`,
`c2rs bench` and `c2rs perf` to completion over all 386 fixtures (77.7 s +
77.7 s + 41.1 s, suitecost §3). Their entire assertion is
`assert_ne!(out.status.code(), Some(2), …)` (`cli_flags.rs:1007-1015`): "the
parser did not refuse this argv." But `cmd_bench` returns
`ExitCode::FAILURE` (= 1) on a `Port=Mismatch` or a determinism failure
(`crates/c2-harness/src/cli/reference.rs:617-628`, `:663-668`), and **1 ≠ 2,
so a corpus-wide wrong emit passes this test**. The suite pays ~155 s per run
to execute the whole-corpus differential and then discards its verdict by
construction — which also means **the corpus-wide byte gate is not in
`cargo test` at all**; it lives only in `gate.sh`'s rows. The fix is not to
narrow the roster (its contract — every invocation `scripts/` makes — is
right, and suitecost §8.3 correctly rejects `--fixtures add3.cpp`): it is to
**add a second, separately-named assertion to the same execution**: when
`Toolchain::locate()` is `Some`, additionally
`assert_eq!(code, Some(0), "corpus-wide oracle self-test went red — read the
per-fixture lines in this test's captured output")`. Zero added wall time;
155 s of existing spend converts from warmth into warranty. (On a
toolchain-less box the commands print `SKIP` and exit 0 via
`Args::toolchain` at `main.rs:419`, so the portable lane stays green.)

**0.2 `C2RS_REQUIRE_TOOLCHAIN` is armed and nothing fires it.** The
instrument exists — `crates/c2-harness/tests/require_toolchain.rs:59`, landed
2026-08-18, whose own header records the motivating measurement: `cargo test
--workspace --release` reads **1,660 / 0 / 43 in a provisioned worktree and
byte-identically 1,660 / 0 / 43 in one with no `compilers/`**
(`require_toolchain.rs:1-27`, boards #3226/#3247). Board **#3247** closes
with, verbatim: *"STILL OPEN … NOTHING SETS THE VARIABLE."* 132 of ~179
integration tests contain a skip-and-return path (124 of them the
`Toolchain::locate() else { eprintln!("SKIP…"); return; }` form), so a fresh
worktree's suite is green with the right target count and the right exit code
while grading nothing. The closure is one line and a convention:
`scripts/partest.sh` (now the canonical runner) exports
`C2RS_REQUIRE_TOOLCHAIN=1` by default when `compilers/` resolves, with
`--portable` to opt out; the merge-funnel checklist quotes the suite only
from a run with the variable set. This closes the family's front door at its
root instead of test-by-test.

**0.3 A standing gate row has not executed in any observed run and nothing
measures for how long.** Board **#3219**: `hatch-red` reads `REFUSED
HATCH-STALE 0/14 arms` on every run in both trees including clean master —
14 arms of a standing row not executing, duration **unmeasured**, and the
gate cannot distinguish "REFUSED this run for a local reason" from "has been
REFUSED for a month". `--selftest` proves each row *can* go red; nothing
proves any row *has run recently*. Two halves, both small: repair
`work/w-front3/hatch.py`'s needles (#1389, open since 2026-08-08 — not this
review's to design), and the generalizing half, which **is** in scope: a
per-row consecutive-non-executing-verdict counter. `gate.sh` already owns a
run directory per run; persist each row's verdict word and print
`hatch-red: REFUSED for N consecutive runs (first seen <date>)` in the table.
~40 lines of shell, no new dependency, and it would have surfaced #3219
without anyone looking.

These outrank everything else in this document because the project's own
record says so: every retraction in STATUS.md came from an instrument
widening, never from a gate going red. The warranty is the product; these are
holes in the warranty that are already known, cheap, and open.

---

## 1. Verdict on ARCHITECTURE_PROPOSAL_2026-08-20.md: **believed, with corrections**

I re-derived its claims against the tree rather than accepting them.

**What checks out (verified directly):**

* `IlBundle::functions()` does couple framing, binding and whole-TU admission
  in one verdict — read at its current location,
  `crates/c2-il/src/func/bundle.rs:1939` (one `Bindings::selective` failure,
  one varargs record, one unmodeled `.drectve` line each refuse the whole
  TU). The proposal's §1.2 item 1 is accurate in substance.
* `IlFunction` is ~15 parallel `Option` shape fields at
  `crates/c2-il/src/func/mod.rs:2980` — verified, field for field.
* `passes/mod.rs` is a 30-line placeholder — verified (`wc -l` = 30).
* The evidence for "the conjunction is the binding constraint" is real and
  strong: board **#3093** (lifting the entire `.gl` binding walk measured
  `match +0 / fnbyte-exact −65`), **#3104/#3106** (49 per-token de-accept
  scans; the widening cost saturates at a floor, 20 of 22 ladder tokens
  contribute 0), the frontier at **2** with both TUs priced ~20 refusals and
  declined, and `w-871`'s decomposition. H3's refutation ("the reader is the
  constraint" — direction refuted) is the best-supported paragraph in the
  proposal.
* H4 ("the harness is instrument mass a cleaner core would not need" —
  mostly no) — I agree, with the composition correction in §2 below.

**Corrections, none fatal:**

1. **Stale line numbers.** `bundle.rs:699` is mid-doc-comment today;
   `functions()` is at `bundle.rs:1939`. `select.rs:127` → `select_function`
   is at `crates/c2-core/src/codegen/select.rs:275`. The files have grown
   under the proposal. Cosmetic, but the proposal is a standing document and
   its citations will be followed.
2. **The "26 of the 27" gloss is off by at least one.** The commissioning
   framing — "26 of the 27 TUs that satisfy every factor are already
   converted" — does not survive the frontier's own definition.
   `factor_frontier` (`crates/c2-harness/src/gap/factors.rs:787`) counts TUs
   *inside* `A∧B∧C` that are not `match` and have no `D∨E` acceptance path;
   it reads **2**. So at most **25** of the 27 in-conjunction TUs are
   matched, and at least **1 of the 26 matches lies outside the
   conjunction** (consistent with the whole-TU `??__E` path and with
   `A∧B∧C∧D = 22 < 26` in the generated block). The proposal's own text
   never states the 26-of-27 form; the summary that circulated does. The
   conclusion — the remainder fails several factors at once and every
   single-stage fix scores ~zero — is unchanged and independently measured.
3. **H4's arithmetic composes wrong by about a third.** "31.8k src" of
   harness includes **5,878 lines of `#[cfg(test)]` test code inside
   `src/`** (`gap/tests.rs` 4,722 + `search/tests.rs` 1,098 +
   `splitter_predicate_guard.rs` 58) — non-test src is ~25.9k. And ~5.2k of
   *that* is dormant research prototype, not warranty: `retrieval.rs` (451,
   last touched **2026-07-11**), `corpus.rs` (1,323, 2026-07-20),
   `search/` (3,390 non-test, 2026-08-04). "The record says the instruments
   found every defect" is true of the **gap/ stack**, not of these. The
   right disposition is still *leave them* (§7) — but the proposal's
   "mostly no" should be read as "the gap/ stack is warranty; a fifth of
   the crate is parked research".
4. **Step 2 (the W8 sum type) has a scheduling risk the proposal does not
   own.** It has been "scheduled, needs its quiesce window" since
   ARCHITECTURE_SEAMS §2.3b (2026-07-30) and has not happened in three
   weeks precisely because the repo's operating mode is 6–9 concurrent
   lanes and nobody owns the window. A step whose precondition is "everyone
   stops" needs a named owner and a named date in the plan, or it will
   still be "step 2" in October. Same applies to step 1's re-expression of
   `functions()`.
5. **Step 3 will grow the harness, and the harness has a growth pathology
   ready to receive it** (§2.3): `TuResult` at 86 `pub` fields
   (`gap/mod.rs:130`) and `GapReport` accessors split across two parallel
   `impl` blocks (`factors.rs`, `report.rs`). The manifest instrument
   should land as its own module with its own denominators, registry-keyed
   like `NAMED_SETS` (`gap/sets.rs`), not as more fields and accessors.

No flaw was found in the staged-IR design itself, in the claims-ledger
refusal boundary, or in the stage-oracle-first ordering. The migration
identity protocol (census numerator, per-key histogram, JSONL rows,
disagreement counters, byte-identical or accounted) is the correct grading
for every step, including every step proposed below.

---

## 2. `crates/c2-harness` — what is warranty, what is mass

Composition, measured (`wc -l`, `#[cfg(test)]` separated):

| layer | lines | disposition |
|---|---:|---|
| differential oracle spine: `lib.rs` 514 + `capture_cache.rs` 1,374 + `fixture_profile.rs` 419 + `provenance.rs` 572 + `cli/reference.rs` 669 | ~3.5k | **warranty. Preserve whole.** |
| gap/ instrument stack (scan, fnbytes, fndiff, factors, report, render, sets, witness, classify) | ~12.6k (+4.7k cfg(test)) | **warranty's instrument layer.** Every retraction on STATUS came from here or from probe grids. Preserve; stop the growth pattern (§2.3). |
| CLI (`main.rs` 484 + `cli/` 4,263) | ~4.7k | keep; the argv parser is already centralized (`main.rs:187-432`, boards #194/#195) — the *former* per-handler duplication is deleted and documented. |
| perf (`perf.rs` 505 + `cli/perf.rs` 302) | ~0.8k | keep — ~~it is the thesis metric~~ **it measures a real property, and the disposition is unchanged (2026-08-21: throughput is no longer the thesis — `GOAL_DECISION_2026-08-21.md`). `perf` is REPORTED, never GATED (#3336), so demoting the metric costs this row nothing: keep it, and keep quoting the ratio with its population.** |
| research prototypes: `corpus.rs`, `retrieval.rs`, `search/`, `listing.rs`, `prefilter.rs` | ~6.3k | **dormant, not warranty** — and still *leave them* (§7.3). |

**Is it one crate doing five jobs?** It is one crate doing five jobs behind
per-concern module boundaries, with no measured contention (SEAMS §1.1 called
the harness "already fine" and nothing since contradicts that). The jobs
share the two things that matter — the capture cache and the `Toolchain`
seam — so a crate split would either duplicate those or add a sixth crate
for them, buying nothing the module boundaries don't already give. **No
split.** The factoring problems are smaller and real:

**2.1 One rule, two implementations, on the warranty path.** The
cache-vs-direct capture arm — `match cache { Some(c) => c.capture(…),
None => tc.capture_reference_with(…) }` — exists in both `lib.rs:174-209`
and `gap/scan.rs:75-78`. This is exactly the shape GAPS §6 keeps recording
(mis-emit #11's class), sitting in the capture path both the fixture gate
and the workload scan depend on. **Funnel it into one function** on
`CaptureCache` or a free `capture_via` in `lib.rs`; construct-rung graded
(scan JSONL byte-identical, cache hit/miss counters unchanged).

**2.2 ~15 hand-rolled toolchain-skip sites, each its own strings.** The
`if !tc.has_strace() { … return }` / `has_mingw` early-return is
re-implemented across `cli/gap.rs:165,169`, `cli/listing.rs:43,177`,
`cli/perf.rs:53,248`, `cli/search.rs:129,203,333,470`, `cli/corpus.rs:53`,
`cli/reference.rs:341,345,405,409`, `lib.rs:152,158,321,326`. Beyond the
duplication, the defect-family angle: any future demand-mode (0.2) has to be
honored at every one of these independently. **One helper** —
`fn toolchain_ready(tc: &Toolchain, need: &[Cap]) -> Result<(), SkipReason>`
— that (a) prints the one canonical skip line, (b) checks
`C2RS_REQUIRE_TOOLCHAIN` and hard-fails instead of skipping when it is set.
Export it so integration tests can use the same funnel (§3.2). This is the
refactor that makes 0.2's guarantee *total* instead of one-test-deep.

**2.3 Stop the report-layer growth pattern; don't churn it.** `TuResult`
(86 `pub` fields, `gap/mod.rs:130`) and the `GapReport` accessor sprawl split
across `factors.rs`/`report.rs` are ugly and **load-bearing** — dozens of
tests and both renderers name those fields, and restructuring them buys
zero warranty. The cheap correct move is a **growth rule**, not a rewrite:
new per-TU facts land in the existing keyed maps (`r.emit` pattern), new
metrics land as `gap-metric` registry keys, and the step-3 manifest
instrument gets its own `gap/manifest.rs` with its own denominators printed
both sides of every change (trap-0/#961 discipline). When the proposal's
step 3 lands, that is the natural moment to fold `factors.rs`/`report.rs`
accessors behind one `impl` — not before.

**2.4 Dead code.** `CaptureCache::header_closure_warning`
(`capture_cache.rs:413`) has no caller in src or tests. Delete it (or wire
it — its module doc still advertises it). One-line rung.

**2.5 `cmd_bench` is `cmd_selftest` wearing a summary line.** Both loop
`oracle_selftest` over `all_fixtures()` and print `selftest_row`
(`cli/reference.rs:288` vs `:638`); suitecost §3 confirms they "differ in
their report format and in nothing else", and the suite therefore pays the
corpus twice. Merge the engines (one function, two renderers), keeping each
command's stdout byte-identical. This buys maintenance, not wall time — the
wall-time lever is suitecost §8.1 (make the loop concurrent with
order-preserved output, `cli_flags` floor 77.7 → ~41 s), which this review
endorses as specified there, including its **"do not thread `c2rs perf`"**
caveat — `perf` is a latency benchmark and threading it corrupts the number
it exists to produce.

---

## 3. Test architecture — 48 targets, 1,690 tests

**The shape is sound and the cost model is now measured** (suitecost): 89 %
of the count is unit tests in `src/` (1,492: c2-il 627, c2-core 584,
c2-harness 220, c2-obj 46, c2-reference 15) costing ~nothing; ~100 % of the
wall is three integration binaries running six serial passes over one
386-fixture corpus. `scripts/partest.sh` already runs binaries in parallel
with by-name identity proof (10 pairs, 1,682 results, including a RED
carried identically). Do not re-derive any of that.

**3.1 Is `cli_flags` a symptom of something structural? Yes — two things.**
First, §0.1: the expensive tests' execution is real and their assertion is
not; the structural statement is that **the CLI has no attested seam between
"argv accepted" and "work performed"**, so the only way to test the bare
invocation is to run it, and the only thing asserted about the run is its
usage-error bit. §0.1's second assertion closes the gap without narrowing
the roster. Second, §2.5: `bench`≡`selftest` means the corpus is paid twice
per suite for one fact.

**3.2 Duplicated setup is real, small, and one piece of it is hazardous.**
Measured across `crates/*/tests/`: ~1,100 duplicated lines (~7 %): 24 copies
of `fn work(tag)`, 124 copies of the locate-else-skip, 40 `has_strace` +
35 `has_mingw` guards, 7 `fn fixture()`. Most of this is benign lane-shaped
duplication the repo has already judged (four `grade_cell` copies were
deliberately left unmerged; board #1094 carries the migration —
`tests/cellgrade/mod.rs:1-22` states the reasoning; leave that judgment
alone). **The hazardous one is the flags constant: 14 identical copies of
`["/nologo","/wd4355","/wd4164","/c","/GR","/O1","/Oi","/EHsc"]`**
(`reloc_identity.rs:58`, `fence_count.rs:48`, `pool_cells.rs:65`,
`pool2_cells.rs:93`, `gate_cause.rs:37`, `inline_fence.rs:41`,
`noinline_boundary.rs:83`, `nonformal_sites.rs:85`, `strlit_fence.rs:66`,
`call_targets.rs:42`, `cellgrade/mod.rs:52`, `empty_elision.rs:36`,
`dead_temp_elision.rs:44`, `census_key_routing.rs:63`). If the workload's
profile ever moves, a missed copy keeps grading the old mode and **reads
green** — the absence family wearing a flags list. Export one
`pub const WORKLOAD_FLAGS` from the harness lib (or a `testsupport` module)
plus the skip funnel from §2.2 and the `work()` helper; convert the 14 sites
mechanically. Leave `grade_cell` to #1094.

**3.3 Failure localization is two-tier and the lower tier is §0.1's hole.**
Named per-fixture differential tests localize perfectly (~40 fixtures,
`differential.rs`); the other ~346 fixtures are graded only by corpus-wide
loops whose verdicts today reach no assertion (§0.1) — after §0.1 they fail
one named test whose captured stdout names the fixture per line
(`cli/reference.rs:605`). That is acceptable localization for a regression
class that gate.sh will also catch; do not build more.

**3.4 One correction to suitecost's numbers going forward:** commit
`5f37a27bc` (post-measurement) added
`census_gate::the_emitter_cfg_class_registry_agrees_with_select_function_and_the_census`
(`census_gate.rs:896`), whose `registry_scan` (`:800`) makes **772 more
serial captures** (386 fixtures × 2 profiles, `:795`). The suite floor has
likely already moved; re-time before quoting suitecost's 67.4 s
`census_gate` figure or sizing §8.1's payoff.

**Deliberately not proposed:** merging test targets to cut link time (loses
file-level lane isolation and invites the #3231 shape), test-impact
selection of any kind, and any change to `census_gate`'s pinned disagreement
counts (`KNOWN_DISAGREEMENTS_PACKED = 1` / `_GY = 12` with named causes,
`census_gate.rs:122,145`) — that pinning *is* the anti-divergence warranty.

---

## 4. `scripts/` — what belongs in shell, what belongs under `cargo test`

`gate.sh` is 5,585 lines: ~2,318 comment, and of the code, **1,767 lines
(32 %) is `--selftest`** — a test suite written in POSIX sh with a
hand-rolled assertion framework (`run_case`/`check_that`, `gate.sh:3372-3419`),
a hand-rolled truncation floor (`-lt 183`, `:5081-5086`), and a documented
subshell-counter hazard (`:3331-3333`) that exists only because it is bash.
The verdict layer it tests — `lane_verdict` (`:683-760`), `sweep_verdict`
(`:761-868`), `debug_verdict` (`:928-1022`), `hatch_red_verdict` /
`ladder_red_verdict` (`:1305-1360`, `:1498-1538`), and `decide`'s ruling
logic (~340 of `:2649-3188`) — is, by the file's own comments, a set of
**pure functions of log text plus numbers** (`:1302`, `:1495`). Everything
else — lane fan-out (`:5344-5360`), env plumbing, disk/inode preflight,
pid-liveness, run-tree reaping, `graded_tree_hash`, the synthetic-checkout
integrity arms (`:1609-2033`) — is process/filesystem/git semantics for
which shell is the honest medium.

**4.1 The move: lift the verdict layer into the workspace; keep the
orchestration in shell.** A small bin target in c2-harness (std-only —
it is line-splitting and integer compares), `gate-verdict <kind> <log>
<exit> [<expected>]`, emitting exactly today's tuple line; `gate.sh` calls
it where it calls the shell functions today. The ~1,100 selftest lines that
fabricate a log string and assert a verdict become `#[test]` functions over
string literals — no `mkdir`, no subshell counters, no truncation floor.
What this buys, in the units the repo prices things: **the log-format
contract stops being an unversioned interface between five scripts** (the
`sed -n 's/.*field=\([0-9]*\).*/\1/p'` patterns at `gate.sh:697-700`,
`mode_lane.sh:95-99`, `debug_lane.sh:189-192` break silently on a format
change today, caught only by hand-written fabricated logs) — the parser and
its tests live in one place, and the same parser can be tested against
`c2rs`'s *actual* output in the same test file. Precedent already in-tree:
`tests/lane_registry.rs` lifted `parse_registry`'s rules exactly this way
and kept the gate's `--selftest` case as *"a deliberately strictly weaker
subset"* (`lane_registry.rs:16-20`) — apply the same pattern: `--selftest`
keeps a ~dozen-case smoke subset so the gate stays self-contained without
cargo, labelled in-file as a subset.

Three properties that must survive the move, stated because each is a
recorded lesson: **NO-RESULT stays a distinct fourth outcome** (an
unparseable line is a lane that did not report, not a lane that reported
zeros — `gate.sh:704-708`); **verdict words are re-derived from counts,
never believed** (`:713`); and the shell side keeps its **count-based
cross-checks** (table rows vs registry length before any verdict —
trap 5's mitigation), because the Rust tool's own test count is a libtest
number and trap 5's newest instance was the test runner itself. The
pin-circularity objection — the verdict tool is built from the tree under
test — is real but weak: the whole chain (mode_lane scraping `c2rs gap`)
already depends on the graded tree's binary, a miscompiled tool fails
loudly at `cargo build`, and the shell cross-checks remain.

**4.2 The `gt_*.py` estate is the sharpest #1406 exposure and should be
frozen, not ported.** BOARD #1406's rule: anything whose output is quoted
as evidence must run under `cargo test` or `gate.sh`. Measured against it:
`gt_inline_decline.py` (3,618 lines — the largest script after gate.sh,
source of `LABEL_COUNTER.md` §6), `gt_label_inline.py` (2,040),
`gt_eh.py`/`gt_eh_cod.py` (1,863, source of `EH_RECORDS.md`), and ~10 more
produce doc-quoted numbers and are executed by nothing standing; only
`gt_dump.py` has a `--selftest` wired as a gate arm (`gate.sh:4042`).
Porting 14k lines of Python characterization into the std-only workspace
is negative value — they are measurement campaigns, not standing gates.
The honest cheap closure: **a dated provenance banner in each consuming
doc** ("measured by `scripts/gt_X.py` at tree `<sha>`; the script is not a
standing instrument; re-run before re-quoting against a newer tree"), plus
a `--selftest` wired as a gate arm *only* for any gt_* script still being
actively re-run (the `gt_dump.py` pattern). The `slotarg_*`, `arms_*`,
`w_tu_*` families (0–3 doc hits) are lane scratch and should say so in a
one-line header, or move to `work/`.

**4.3 Leave alone in scripts/, with reasons:** the intentional
second-opinion redundancy where `expr_sweep.sh`/`mode_cross.sh` keep their
own accumulators *and* gate.sh re-derives from the printed line
(`gate.sh:773` states it); `sweep_verdict` ruling both sweep rows (one
rule, deliberately not two implementations, `:776-780`); `status.sh` whole
(1,110 lines, 42 % self-check — its NO-RESULT discipline, `val_or_missing`
/ registry-walk rendering / `INCOMPLETE — the report is not a measurement`,
is the best absence-defense in the repo and a model for §4.1's port);
`lanes.txt` + `lane_registry.rs` dual (weaker-subset pattern, working as
designed); `debug_lane.sh`'s duplication of `mode_lane.sh` — it already
caused #3134 once and was fixed; unifying them through one script would put
the debug profile's cp/rebuild into every green path for a ~100-line
saving. Note it as accepted debt with #3134 as the fence.

---

## 5. Code quality across `crates/` — spot findings

* **Error handling is disciplined where it matters.** The refusal paths are
  `Option`/`Result` fail-closed; `bundle.rs` and `coff/function.rs` contain
  zero `unwrap()`/`panic!` outside tests. The 790 `unwrap()` sites
  workspace-wide concentrate in `#[cfg(test)]` modules and instrument code
  where a panic is an honest instrument failure. `unreachable!` is rare (14)
  and `deadsites` (#3278) has already classified the dead-code kinds — no
  action beyond its own.
* **Module boundaries are narrow and directional.** Harness → core is
  `Backend`/`PortC2` plus three deliberate instrument reaches
  (`select_function` for the anti-divergence check, `encode` for the
  frontier-bytes instrument, `PORT_WRITER_SECTIONS` for factor C) — each is
  the census/gate symmetry working, not a leak. Core → il is the per-shape
  struct surface (`XteaEncryptLoop`, `PoolFreeList`, …) — this *is* the
  parallel-Option signature the proposal's step 2 dissolves; it is not
  fixable at the boundary and should not be "cleaned" ahead of step 2.
* **Naming is consistent** (try_parse_* recognizers, *_text lowerings,
  gap-metric kebab keys, w-slug lanes). The ~45 % comment density in
  `shapes/`/`codegen/` is measured behavior with fenced negative cells —
  per the commission and per my own reading of `func/mod.rs`'s field docs,
  **it is the cheapest part of the whitebox record and must not be
  stripped**.
* The genuine quality debts found are all named above: §2.1 (duplicated
  capture arm), §2.2 (skip sites), §2.4 (one dead fn), §3.2 (14 flag-list
  copies), §4.1 (the log-format contract).

---

## 6. Crate boundaries — agree: five crates, no new ones, split for contention

Stress-tested rather than accepted: (a) the import surfaces measured in §5
are narrow, so crate seams would formalize boundaries that already hold at
module level; (b) the workspace's compile cost is dominated by **37
integration binaries each linking the full workspace**, which a crate split
does not reduce (and target consolidation is rejected on localization
grounds, §3); (c) the one plausible new crate — pulling the oracle spine
out of c2-harness so instruments can't reach into it — fails its own test:
nothing has ever regressed across that line, and the instruments *should*
reach the oracle. The proposal's §3.2 layout (module-boundary-first, crate
splits available later if contention demands) is right. The only amendment:
when step 0's stage-oracle capture lands in c2-harness, keep it under
`gap/`-style per-concern modules with its own denominators from day one
(§2.3's growth rule), so the crate's next 10k lines don't arrive as
accessor sprawl.

---

## 7. The leave-it-alone list (things that look wrong and are load-bearing)

1. **The 1,767-line `--selftest` in gate.sh — until §4.1 lands, and a smoke
   subset of it forever.** It is the only thing standing between the gate
   and silent format drift today. Port it; never simply delete it.
2. **`census_gate.rs`'s pinned exact counts and named causes**
   (`:122,145,487-507`) and `assert_population_can_fail`'s ordering
   (`:360-399`) — the anti-divergence warranty and the population-can-fail
   discipline; both encode paid-for lessons (#1304, the sixteenth
   absence instance at `:585-593` happened *inside the fix for the
   fifteenth*).
3. **The dormant research modules** (`retrieval.rs`, `corpus.rs`,
   `search/`): deleting ~6k lines buys 0 toward 870, their judge is the
   real oracle (no classifier hazard), `corpus.rs` still feeds the sweep's
   committed sample, and T-A/P1.3 are decided angles that may resume.
   Cheap to keep; expensive to be wrong about.
4. **`TuResult`'s 86 fields and the dual `GapReport` impls** — output
   records read by renderers and dozens of tests; restructuring is churn on
   the instrument layer during active lanes. Growth rule only (§2.3).
5. **Four `grade_cell` copies in tests** — deliberately unmerged, reasoning
   recorded at `cellgrade/mod.rs:1-22`, migration tracked (#1094). Not this
   review's to override.
6. **`mcall.rs` (5.4k), `coff.rs`, the 46-file `codegen/` shape catalogue,
   the ~24 name-paired shape files** — all adjudicated by SEAMS §1.2/§9 and
   the proposal (witnesses, demoted-not-deleted). Re-adjudicating them here
   would be redoing the first review.
7. **The intentional redundancies in scripts** (§4.3): sweep accumulators +
   gate re-derivation; `lanes.txt` + `lane_registry.rs`; `status.sh`'s
   self-check mass.
8. **`fixture_profiles`' "there is no skip lane and there must not be one"**
   (`fixture_profiles.rs:30-34`) — the one test that refuses the skip
   pattern on purpose; it is the template §0.2 generalizes, not an
   inconsistency to normalize.
9. **Docs-as-ledger** (STATUS banners, BOARD corrections-in-place, frozen
   rung docs) — history stays as written; nothing here proposes rewriting
   any of it.

---

## 8. Migration order — the 26 and the gate stay green at every step

Nothing below touches `crates/{c2-il,c2-core,c2-obj,c2-reference}`, so
`match 26 / mismatch 0 / census` are unmoved **by construction**; each step
is still landed under the full gate per the standing protocol, and every
step where stdout is quoted by a parser is graded byte-identical.

| # | step | files | graded by |
|---|---|---|---|
| 1 | §0.2: `partest.sh` exports `C2RS_REQUIRE_TOOLCHAIN=1` by default (opt-out `--portable`); funnel convention documented | scripts/ only | provisioned suite green with the demand; toolchain-less run goes RED on exactly `require_toolchain::…` (the #3247 D6b demonstration re-run) |
| 2 | §0.1: second assertion in `accepted_group` (exit == 0 when toolchain present), named separately from the roster assertion | `tests/cli_flags.rs` | suite green; 0 wall delta (same executions); mutation check: a planted fixture mismatch must redden exactly this test |
| 3 | §0.3(b): per-row consecutive-non-executing-verdict counter in gate.sh | scripts/ | `--selftest` cases for the counter; gate table gains one line per non-executing row; hatch-red's N starts printing |
| 4 | §2.4 delete dead fn; §2.1 capture-arm funnel; §2.2 skip funnel (CLI sites) | c2-harness src | identity protocol: scan JSONL rows byte-identical, cache counters unchanged, all skip lines byte-identical |
| 5 | §3.2: `WORKLOAD_FLAGS` + `work()` + skip funnel exported; 14 flag copies + 24 helpers converted mechanically | harness lib + tests/ | suite green; grep count of the literal flag list goes 14 → 1; per-target test counts unchanged by name (partest by-name diff) |
| 6 | §2.5 bench/selftest engine merge + suitecost §8.1 concurrent loop (ordered output) | c2-harness src | stdout of `c2rs bench` and `c2rs selftest` byte-identical vs before (the suitecost-specified criterion); then re-time the suite floor (§3.4) |
| 7 | §4.1: `gate-verdict` bin + ported selftest cases; gate.sh switched classifier-by-classifier; shell smoke subset kept | c2-harness + scripts/ | per classifier: gate table byte-identical on the same tree before/after the switch; the fabricated-log cases pass in both hosts during the overlap |
| 8 | §4.2: provenance banners on gt_*-fed docs; scratch headers | docs/ only | prose |

Steps 1–3 are independent and can land today, in any order, between lane
waves — none conflicts with `wt-w-stageoracle`/`wt-w-ir0`/`wt-w-objplan`
(which own crates/c2-il, crates/c2-core seams, not the harness/test/script
layer). Step 6 should wait for a quiet box to re-time. Step 7 is the only
multi-day item and should be one lane with the overlap protocol stated.

---

## 9. Honest cost, and what each buys — in the goal's units

First the honest denominator: **nothing in this review converts a TU.**
`match` stays 26 through all of §8; anyone pricing these against a
conversion should decline them all. The goal's own doctrine prices the other
side: the warranty is the product's correctness argument, "widening the
instruments is the work" (STATUS), and a wrong emit that instruments miss
costs more than any lane (a live wrong emit survived **255 commits** of
green gates once). Priced two-sided against doing nothing:

* Steps 1–3: **~1 lane-day total.** Buys: the front door of the defect
  family closed (fresh-worktree-green becomes impossible under the demand),
  155 s/run of already-paid execution converted into a corpus-wide byte
  gate inside `cargo test`, and standing-row liveness measured (#3219's
  named missing instrument). Refusing them keeps three known, cheap,
  recorded holes open in the layer every rung quotes.
* Steps 4–5: **~1 lane-day.** Buys: three "one rule, two implementations"
  hazards removed from the capture/skip/flags paths — the exact defect
  shape (#11, #3134) the repo has paid for repeatedly.
* Step 6: **~1 lane-day + a quiet-box timing session.** Buys: suite floor
  ~199 s serial / ~117 s parallel → ~110 s serial / ~70 s parallel
  (re-time per §3.4), collected several times per merge across a 6–9-lane
  operating mode. Pure iteration speed; declining it costs ~an hour of
  wall per working day at current merge rates.
* Step 7: **~3–5 lane-days, the only real spend.** Buys: the gate's verdict
  logic and its ~180-case test suite move from an unversioned sh string
  contract to typed, unit-tested code — drift immunity for the instrument
  the merge decision rests on. The two-sided price: if it is botched, the
  gate itself is what's at risk — which is why §8 grades every classifier
  switch by byte-identical gate output and keeps the shell smoke subset.
  Defensible to defer; not defensible to do casually.
* Step 8: **hours.** Buys: #1406's rule stops being silently violated by
  14k lines of Python; every doc-quoted gt_* number carries its tree.

The largest thing this review recommends **not** spending on: any
restructuring of the gap/ instrument stack, the research modules, the test
target layout, or the crate set. The first review's conjunction finding
means the program's scarce resource is lanes on the staged-IR path; this
scope's correct contribution is to make the warranty layer airtight and
cheap, and otherwise stay out of the way.
