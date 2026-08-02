# STATUS — where the port is, and how to check for yourself

**This doc is a cache, not a source.** Every number in the generated block below
comes from `scripts/status.sh`; regenerate it with `scripts/status.sh --write`
rather than editing it by hand. If the block and the tree disagree, the tree is
right — rerun the script.

The rest of the doc is the part a script cannot produce: what each number
**means**, what it **does not** mean, and which of them is actually the target.
That distinction is the whole content of ROADMAP §8, and getting it wrong has
cost this project real work more than once.

---

## The numbers

<!-- BEGIN GENERATED: scripts/status.sh — do not hand-edit -->
Collected 2026-08-02 · tree `e342b08-dirty` · binary `cfe2e2b2952f`

| metric | value |
|---|---|
| Workspace tests (cargo test --workspace --release) | 606 passed, 0 failed, 24 targets |
| Oracle self-test (c2rs selftest) | 211 PASS, 0 FAIL |
| Fixture port gate (c2rs perf) | 100 port Match, 0 mismatch, 111 not-implemented (of 211) |
| Port speedup, geomean over matched fixtures | 668x geomean over matched fixtures |
| 878-TU dc3 workload scan (c2rs gap) | match 6, mismatch 0, codegen-gap 0, vocab-gap 865, capture-fail 7 |
| Per-function census (driver, not target) | 706402/2462572 functions in class (28.69%) |
| Emitted-function census | 38456/178969 emitted functions in class (21.49%) |
| Emitted-census residue | residue 9275: 2004 compiler-generated (no IL body), 7271 unexplained  (5.18% of the denominator) |
| TU distance to match, blocked functions | ≤0: 1, ≤1: 10, ≤10: 25, ≤100: 32, ≤1000: 210 |
| TU distance to match, blocked emitted functions | ≤0: 2, ≤1: 19, ≤10: 82, ≤100: 403, ≤1000: 858 |
| Emit-set ceiling (segments == COMDATs) | 25 of 871 graded TUs |
| Emit-set MODEL ceiling (today / repaired / wall) | 324 today / 420 repaired / 451 wall |
| .gl binding invariants (records / arity / conflicts) | 1515161 records, 420 nameless, 2 before the first row, 39371 row-conflicts, 712 name-conflicts, 0 accounting breaks, 0 unreadable objs |

<!-- END GENERATED -->

---

## The one-paragraph answer

The **foundation is proven and fast**: standalone replay of the real `c2.dll` is
byte-exact on all 871 capturable TUs of a real Xbox 360 game, the port is
byte-exact everywhere it accepts, it refuses everywhere else, and **no run has
ever recorded a mismatch**. The **payoff metric is stuck**: TU match has been
6/878 across a per-function census run from 4.45 % to 28.69 %. That is not a
mystery and not a regression — §8.1 measured why, and the emit-set ceilings below
bound how far widening alone can ever take it.

---

## What each number is for

| number | it is | it is NOT |
|---|---|---|
| **TU match** (of 878) | **the payoff metric** — whole objs byte-exact at the workload's real flags | ~~a coverage percentage~~ |
| TU distance ≤1 / ≤10 / ≤100 | the leading indicator for TU match | a promise that the near ones are cheap |
| **emitted-function census** | in-class ∩ *code c2 actually emits* | gradeable by the differential on its own |
| per-function census | **a driver** — it ranks rungs, and does that superbly | the target. "census → 100 %" is **retired** (§8.1) |
| emit-set ceiling (25/871) | TUs where `.ex` segments == obj COMDATs — the most TU match can reach **before** Phase 7 exists | reachable by widening |
| emit-set MODEL ceiling (324/871) | TUs where a segment-driven model binds every emitted symbol | the same thing as the line above (see below) |
| mismatch count | an **alarm**, and it has never fired | evidence of correctness (see the coverage bound) |
| fixture gate | the port's accepted class, graded per fixture | representative of the workload's shape |
| perf geomean | the project's actual thesis — verifier throughput | comparable across versions. **Always quote it with its fixture count** (GAPS §1): the geomean is taken over the *matched* set, which grows as the port widens, so two geomeans are a change of population, not a regression. It is *also* wall-clock — 623×/653×/689× on three consecutive runs of one binary over the same 100 fixtures. Quote the order of magnitude with the count, never the digits alone. |

### The two ceilings are different things, and only one bounds TU match

* **Emit-set ceiling, 25 of 871** — TUs where the number of `.ex` function
  segments already equals the number of `.text` COMDATs in the real obj. For
  these, a port that lowered every body correctly would emit the right *set* of
  functions without modelling anything. **This is the hard bound on TU match
  until Phase 7 (the emit-set model) exists** — and **6 of the 25 are already
  taken**, so every widening rung in the plan, summed, can move the payoff metric
  by at most **19 TUs, ever**. On the other 846, the port emits one `.text`
  COMDAT per `.ex` segment and is wrong about the *set* regardless of how
  correctly it lowers each body (842 spurious, 4 missing).
* **Emit-set MODEL ceiling, 324 today / 420 repaired / 451 wall** — TUs where the
  `.gl` binding can account for every emitted symbol. This bounds *a model*, not
  today's port. It went 111 → 324 in §9.20 from a one-byte reader repair.
  §9.20 then claimed that gain was "unrealisable until the gate learns the same
  rule"; **W-ADOPT taught the gate that rule and the ceiling did not move**
  (§9.21). It is computed on `EmitBinding`, which already had the widened
  reader, so the gate was never the dependency. Realising it needs Phase 7 — an
  emit-set model — and nothing short of that.

Quoting 324 as "where we are" is the most likely misreading of this page.

---

## The traps

Each of these is a mistake the project has already made and paid for. They are
recorded here because the numbers above are individually true and jointly
misleading without them.

1. **`mismatch 0` is not evidence of correctness.** 865 of 878 TUs refuse before
   the emitter is consulted, so the scan *cannot see* a codegen or binding defect
   in them. Zero mismatches means "nothing the scan could grade came out wrong",
   over a population the scan mostly cannot grade. Verification here is
   coverage-bounded differential testing, and a green run is sound only on the IL
   it ran against. (ROADMAP **§7 / §10.8** — the bound has been restated
   independently fourteen times and is now an invariant. Do **not** cite this as
   "#149"; that number denotes the off-add argument slot.)

2. **A per-function census claim for a never-emitted body can never be graded.**
   The differential compares whole objs, and an unemitted body is not in the obj.
   For those, "in class" is a *parser-only* claim with no byte behind it. The
   recorded precedent that this direction can be green-and-wrong is the `.sy`
   positional relaxation: census +2,981, mismatch 0, **wrong on 62 % of
   bindings**.

3. **A residue shrinking is not the thing the residue is a proxy for.** §9.20.3
   raised the `.gl` name-distance bound and watched `records_nameless` fall
   monotonically from 70 → 4 while **not one additional emitted symbol was
   covered**, and past a point it started handing one name to two records. A lane
   grading itself on the residue would have reported steady progress while
   covering nothing and corrupting the binding. (ROADMAP §9.20.3, §9.16.5. The
   prose calls this "#144's shape" as an **echo**; neither registered #144 is
   this rule — see [`BOARD.md`](BOARD.md) on bare-`#N` ambiguity.)

4. **Totality residue 0 is not a control.** `records == bound + residue` is
   satisfied exactly by moving a record from one bucket to another, so it cannot
   distinguish "we found a record" from "we found a name". The arity axis exists
   because of this: record *count* and record *offsets* are published and
   compared, and they were byte-identical (1,515,160) across a change that moved
   152,521 records between buckets.

5. **Absence reads as success unless something forbids it.** ROADMAP §9.18.8
   records this failure mode **twelve times**, and the newest instance was the
   *test runner itself* — a run reporting `ok` for every target with **169 tests
   silently not run**. Two others: a sweep that `sed`-ed a number out of a report
   and read the missing number as `0`, passing a run that graded literally
   nothing (§6s); and a lane registry whose four recorded lanes contained **no
   `/EH` at all** on a workload that is 100 % `/EHsc` (GAPS §7). This is why
   `gate.sh` renders from a registry, why `lanes.txt` is data, and why
   `status.sh` prints `NO-RESULT`. **The mitigation generalizes: compare a count,
   never a status.**

6. **The census names the callee, not the function, for any call-bearing body.**
   In the near-match tables, any row whose body makes a call is labelled with the
   *callee's* name. Known, unfixed. (GAPS §9.6.) **The blocker keys, the counts
   and both class axes are unaffected** — it is a labelling defect, so the
   rankings built on it stand.

7. **The mode caveat is resolved, but know that it existed.** Fixture numbers are
   captured at `/Ox`; the 878-TU workload compiles `/O1`. The port now reads the
   per-function optimization word and refuses anything unmodeled, `/O1` is a
   supported target, and `scripts/gate.sh` runs 12 enumerated lanes crossing the
   optimization and `/EHsc` axes. Numerator and denominator now speak the same
   modes.

---

## Reproducing any of it

```sh
scripts/status.sh                 # everything below, in one pass
scripts/status.sh --check         # prove the collector, no toolchain needed
```

| what | command |
|---|---|
| workspace tests (portable) | `cargo test --workspace --release` |
| oracle self-test | `cargo run --release -p c2-harness --bin c2rs -- selftest` |
| fixture gate + speedup | `cargo run --release -p c2-harness --bin c2rs -- perf` |
| the 878-TU workload scan | `c2rs gap --list work/dc3-workload/files.txt --flags-file work/dc3-workload/flags.txt --cwd ../dc3-decomp --jobs 16` |
| regenerate the workload inputs | `scripts/gen_dc3_workload.sh <dc3-tree>` |
| **the merge gate** (12 lanes) | `scripts/gate.sh --jobs 6` |
| generated expression sweep | `scripts/expr_sweep.sh` |
| cross-product lane | `scripts/cross_sweep.sh` |
| throughput vs concurrency | `c2rs perf-scale --csv docs/perf/perf_scale.csv` |

`status.sh` deliberately does **not** run the merge gate, the sweep, or the
cross-product: those answer *"is this tree safe to land"*, which is a different
question from *"where is this project"*, and they cost minutes rather than
seconds. Run the gate before landing; run `status.sh` to report.

Everything except `cargo test` needs the toolchain (wibo + `compilers/`); all of
it degrades to `SKIP: toolchain absent` rather than failing.

---

## Where the code is

Two files hold the accept/refuse boundary, and a third exists only to stop them
from diverging:

| | |
|---|---|
| parse-time acceptance | `crates/c2-il/src/func/bundle.rs:699` — `IlBundle::functions()` |
| emit-time dispatch | `crates/c2-core/src/codegen/select.rs:127` — `select_function` (ordered match; **order is load-bearing**) |
| the anti-divergence check | `crates/c2-core/src/codegen/select.rs:210` — `function_gate`, why `census/gate disagreement` is 0 |
| gap-key rendering | `crates/c2-il/src/func/body/mod.rs:784` — `Block::feature()` |
| TU-level classes | `crates/c2-harness/src/gap.rs:74` — `TuClass` (`vocab-gap` = IL decode, `codegen-gap` = port refusal) |

Decode is **3.4× the emitter** by line count (`c2-il` 35.5k vs `c2-core` 10.4k)
and holds ~4× the tests. That is the physical signature of `vocab-gap 865`: the
port is not blocked on generating PowerPC, it is blocked on reading IL. The
largest single file in the project is the member-call decode
(`crates/c2-il/src/func/body/mcall.rs`, 4,643 lines), which is exactly the
`tail-recv-not-a-plain-b9-load/*` family at the top of the blocked histogram.

---

## Where to go from here

| question | doc |
|---|---|
| what is open, what was declined, what was refuted | [`BOARD.md`](BOARD.md) |
| the phase plan and why it is ordered that way | [`ROADMAP.md`](ROADMAP.md) §8 |
| what each blocker holds hostage, per rung | [`GAPS.md`](GAPS.md) |
| what landed, when, and for how much census | [`rungs/INDEX.md`](rungs/INDEX.md) |
| the correctness rule and the invariants | [`ROADMAP.md`](ROADMAP.md) §7, `../CLAUDE.md` |
