# PREREG — `w-adjacency`, the carryover-balanced rotation for `scripts/cost_arms.py`

    Tag:       w-adjacency
    Date:      2026-08-24
    Kind:      construct (instrument)
    Base:      67f276409
    Rows:      #3521–#3524 (ninth-wave ledger, `docs/BOARD.md` end of file)
    Status:    REGISTERED BEFORE ANY TIMING MEASUREMENT

**No timing run has been taken when this file is committed.** The rotation code
(`62284ed7c`) is committed first because it is a *deterministic* artefact — its
correctness is a property of arithmetic, verified by `--show-design`, and it
cannot be tuned by looking at a measurement it has not seen. Every numeric
prediction below is registered before the first `c2rs perf` invocation of this
lane. `crates/` is untouched and stays untouched: this lane's fence forbids it.

---

## 0. What this lane was asked to do

`w-permute` (#3495) diagnosed and deliberately did **not** fix the adjacency
defect in `scripts/cost_arms.py`, pricing the fix as *"one `order` line plus a
balance check that counts pairs instead of positions; the re-validation is
three lanes' cost runs"*. This lane builds the fix, validates it against a
known-answer case, and pays what it can of the re-run.

---

## 1. THE BRIEF'S CITATIONS, VERIFIED BEFORE USE

| claim in the brief | checked at `67f276409` | verdict |
|---|---|---|
| `cost_arms.py` rotates as `arms[r%n:] + arms[:r%n]` | `scripts/cost_arms.py:207` at `67f276409`, verbatim | **HOLDS** |
| over 3 arms one cycle gives `base→nulldup ×2`, `nulldup→tip ×2`, `tip→base ×2` | re-derived mechanically by `--show-design 3 --rotation cyclic`: A→B 2, B→C 2, C→A 2, B→A 1, C→B 1, A→C 1, and **A→A B→B C→C all 0** | **HOLDS, and it is worse than stated** — the reverse pairs are not 0, they are 1, so the imbalance is 2-vs-1 per cycle, not 2-vs-0 |
| `#3468`'s fix required the round count to complete the rotation | `if args.rounds % len(arms) != 0: raise SystemExit` present at base | **HOLDS** |
| three balanced `--rounds 9` runs, n = 157, `cmp`-verified null | `docs/rungs/2026-08-24-w-permute.md` §4 table, three rows | **HOLDS** |
| run 1 `base,nulldup,tip` load 4.4 contaminated, null +0.29 % [+0.14,+0.44] split 62 % | rung §4 row 1 | **HOLDS** |
| run 2 `base,nulldup,tip` load 1.4 idle, null +0.46 % [+0.32,+0.61] split 71 % | rung §4 row 2 | **HOLDS** |
| run 3 `base,tip,nulldup` load 2.2, null +0.06 % [−0.05,+0.17] split 54 % | rung §4 row 3 | **HOLDS** |
| `w-permute` predicted the null's sign would flip negative and scored a MISS | rung §4.2, *"The registered prediction is a MISS as worded"* | **HOLDS** |
| `w-s1bc` §4.3, `w-s1c2` §4.1/§4.3 and `w-s1c3` (#3468) each published a cost reading on the unbalanced instrument | all three located and read; §5 below prices each | **HOLDS** |

### 1.1 One citation that needed correcting

The brief says *"for n=3, a Williams design needs 6 sequences (two Latin
squares)"*. Six is right and the reason given is not the one this lane uses. A
Williams design balances adjacency **within a sequence**; it says nothing about
the boundary between one sequence and the next. In this instrument the round
boundary is **not a pause** — the last arm of round `r` and the first of round
`r+1` are two back-to-back `c2rs perf` invocations with nothing between them,
exactly like a within-round pair. A design that balanced only within-round
adjacency would leave `1/n` of all adjacencies (12 of 36 at 12 rounds over 3
arms) unbalanced, which is the same class of defect at a third of the size.
**The design registered here balances the whole flat sequence**, boundaries
included. For n = 3 that is also 6 rounds, so the brief's number is right; the
general rule is not "Williams" but `L = 2n`, and it differs from Williams at
even `n` (Williams needs `n`; this needs `2n`).

---

## 2. THE DESIGN, AND WHY `L = 2n` — registered as arithmetic, not as a hope

For `n` arms the cycle is `L = 2n` rounds, each round a permutation of the arms.
Three exact properties over one cycle, all re-derived from the flat sequence by
`verify_design`:

1. each arm holds each slot of a round exactly **2×**;
2. each of the `n(n-1)` ordered cross pairs `a→b` occurs exactly **2×**;
3. each of the `n` self pairs `a→a` occurs exactly **2×**.

**The counts are forced.** A cycle of `L` rounds has `L·n` adjacencies (circular),
of which `L(n-1)` lie inside rounds and are necessarily cross pairs. Balance
requires `n(n-1) | cross_total` and `n | self_total`. At `L = 2n`:
`cross_total ≥ 2n(n-1)` from the within-round pairs alone, and
`cross_total + self_total = 2n²`, so `cross_each = 2`, `self_each = 2` is the
only solution — which means **all `2n` round boundaries are self-repeats, two
per arm.** That is balanced, not a defect: every arm gets exactly two warm
restarts per cycle, so the min-over-rounds estimator draws from the same mixture
for every arm.

`L = n` admits no such design for `n = 3` or `n = 4` (exhaustive DFS, both
return "no solution" without hitting the step budget), which is why the cycle is
`2n` and not `n`.

**Known residue, registered so it is not discovered later and called a finding.**
The cycle is balanced as a *circle*; a run is a *line*, so the wrap from the last
arm back to the first never happens. `rounds·n − 1` adjacencies cannot divide
evenly among `n²` pair classes, so exactly one class is short by one. At
`--rounds 12` over 3 arms that is **1 adjacency in 36**, and it is the floor, not
a slack.

---

## 3. PREDICTIONS

### P1 — the design verifies for every arm count the script will realistically see

`--show-design N` for `N = 2,3,4,5,6` prints `VERIFY: BALANCED` with
`cross_each == self_each == pos_each == 2` and `rounds == 2N`.

**Score:** PASS if all five verify. MISS if any does not.
**Registered before running:** already run (deterministic, no box involvement) —
this is recorded as a HIT-in-advance, not claimed as evidence of anything about
the box.

### P2 — every guard is watched refusing on deliberately broken input

Per `CLAUDE.md` (*"watch it fail on deliberately broken input before relying on
it"*), four refusals, each required to exit **non-zero**:

1. `--rounds 9` over 3 arms — **legal under `#3468`, illegal now**;
2. `--rounds 8` over 3 arms — illegal under both;
3. `--rounds 0` — a degenerate count;
4. a null arm that is not byte-identical (`#3468`'s existing refusal, re-checked
   because this lane moved code above it).

**Score:** PASS only if all four exit non-zero **and** the message names the
legal count. MISS on any exit 0.

### P3 — THE ACCEPTANCE TEST: a byte-identical arm reads ~0 in EVERY list position

Arms are `w-permute`'s own: base `f6f56df78`, tip `0ff503eb0`, and `nulldup` a
`cp` of the base binary, `cmp`-verified by the script. `--port-iters 2000`,
`--rounds 12` in every run (legal under *both* rotations, so rotation is the
only thing that changes). Four runs, back to back, on a box the coordinator has
held:

| run | rotation | arm list | null's list position |
|---|---|---|---|
| **A** | `cyclic` (control) | `base, nulldup, tip` | 2 |
| **B** | `balanced` | `base, nulldup, tip` | 2 |
| **C** | `balanced` | `base, tip, nulldup` | 3 |
| **D** | `cyclic` (control) | `base, tip, nulldup` | 3 |

**The registered criterion, numeric:**

* **P3a (positive control — the test's POWER).** Run **A** must reproduce the
  artefact: null mean **> +0.15 %** with a sign split **≥ 60 %**. If it does
  not, runs B and C prove nothing, and I will report the acceptance test as
  **UNPOWERED** and say so in exactly those words — a clean null under the new
  rotation is worthless if the old rotation was also clean on the same binaries.
* **P3b (the fix).** Runs **B** and **C** must *both* give a null with
  `|mean| ≤ 0.20 %`, a 95 % CI containing zero, **and** a sign split in
  **[42 %, 58 %]**.
* **P3c (position-independence).** `|null(B) − null(C)| ≤ 0.20` percentage
  points. This is the claim the brief asked for — the reading must not depend on
  where the null sits in the declared list.

**Score:** PASS only if P3a, P3b and P3c all hold. Any one failing is a MISS and
is reported as a MISS.

**What I will conclude if the null does NOT collapse — registered now:**

* If **A** is clean (P3a fails): **UNPOWERED**, no credit claimed for B or C,
  and the fix ships as *built, not validated on this box today*.
* If **A** is dirty but **B** is dirty too: adjacency balance is **not
  sufficient**, the mechanism is not first-order carryover, and the honest
  reading is that #3495 localised the artefact to *the declared list* and not to
  *adjacency*. I will say the fix is **BUILT AND REFUTED as an explanation**,
  keep it (it strictly subsumes #3468's criterion and costs nothing), and name
  the surviving candidates rather than pick one.
* If **B** is clean and **C** is dirty (or vice versa): the fix has **moved** the
  artefact rather than removed it, which is the worst outcome and the one worth
  the loudest headline; I will report it as a MISS on P3c specifically.
* If **D** is also clean (as `w-permute` run 3 found): that is a **replication**
  of #3495's asymmetry — the cyclic rotation happens to be benign in position 3
  — and it is reported as corroboration of #3495, not as evidence for the fix.

### P4 — the re-run of the prior readings

Full re-measurement of `w-s1bc`, `w-s1c2` and `w-s1c3` needs **8 release builds
of historical trees and 6–8 further cost runs**. §5 prices it. **Registered
prediction: I will re-take `w-s1c3`/#3468's reading only** — base `e85253cda`,
tip `4d04ee59e` — and **decline the other two with a price**, because #3468 is
the one whose *conclusion* #3495 amended and the only one whose number is
currently load-bearing.

**Registered numeric prediction for the #3468 re-run (run E):** #3468 published
`tip +0.99 % [+0.63, +1.35]`, median +0.90 %, on a cyclic `--rounds 9`. On the
balanced rotation at `--rounds 12` I predict the tip stays **positive** and lands
in **[+0.5 %, +1.5 %]**, and the null lands within P3b's band.

**Score:** HIT if both hold. MISS if the tip leaves that band or changes sign —
and a MISS here is the more interesting outcome, because it would mean #3468's
*number* moves and not merely its conclusion.

---

## 4. WHAT COULD MAKE THIS LANE'S OWN NUMBERS WRONG — named in advance

* **The box.** Three other lanes are live and any of them can start a gate. The
  coordinator holds the box for the block; load is recorded at both ends of every
  run and a contaminated run is **labelled and kept**, not discarded
  (`w-permute`'s precedent).
* **Rebuilt binaries are not `w-permute`'s binaries.** They are rebuilt from the
  same shas with the same toolchain, in a different directory, so their embedded
  `CARGO_MANIFEST_DIR` differs and the code may not be byte-identical to what
  `w-permute` timed. **Therefore this lane does not claim to reproduce
  `w-permute`'s numbers**, only to compare rotations against each other on one
  set of binaries. Run A is the control that makes that comparison mean
  something.
* **The null's distinct inode**, named by the coordinator for `w-permute`: it is
  identical in all four runs here, so it cannot explain a difference between
  them. It remains a candidate for any residue.
* **#3483's sharpening**: changing `--rotation` and re-running proves
  **reproducibility, never attribution**. The attribution claim here rests on
  the *control*, A vs B, not on B alone.

---

## 5. THE PRICE OF THE FULL RE-RUN — stated before it is declined

| prior reading | arms | binaries needed | new legal `--rounds` | cost |
|---|---|---|---|---|
| `w-s1c3` §4.3 / **#3468** — tip +0.99 % [+0.63,+1.35] | 3 | base `e85253cda`, tip `4d04ee59e` | 12 | 2 release builds + 1 run (~36 `perf` invocations) |
| `w-s1c2` §4.1/§4.3 — two runs disagreeing in sign, −1.08 %/+0.52 % nulls | 3 | base `f53877aa5`, tip `178423b56` | 12 | 2 release builds + 1 run |
| `w-s1bc` §4.3 — null +0.47 %/57 % then −0.08 %/46 %; S1c +1.78 % | **4** | base `4b19cda28` + **two** tips (`s1b`, `s1c`) whose shas the rung does not name in its header | **16** (cycle is 8) | 3 release builds + 1 run of **64** `perf` invocations, plus a sha hunt |

**Declared intent: take the first, decline the other two, and say which of their
readings can and cannot be re-derived.** A prior number that cannot be re-taken
is a finding and will be reported as one.

---

## 6. Gates

`C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast`,
`sh scripts/gate.sh --jobs 4 --require-graded`, `sh scripts/board_audit.sh`.
`crates/` is untouched, so the gate is expected to be identical to base; it is
run to prove that rather than assumed. **Neither is run while a timing
measurement is live** — that is the contamination this lane exists to remove.

---

## 7. AMENDMENT, registered BEFORE the first timing measurement

**Nothing has been timed when this section is committed.** One `c2rs perf
--port-iters 2000` invocation was run once as a **budget probe** — 157 Match
rows, **50.6 s wall on a load-27 box** — and no number from it is used as data.

§3's protocol registered `--rounds 12`. At 50 s per invocation that is 36
invocations = **30 min per run**, and the five runs A–E would hold the box for
**2.5 hours** with three other lanes idle. **The rounds count is amended to 6
for every run**, uniformly, so the runs stay comparable to each other:

* **6 is legal under both rotations** (`6 % 3 == 0` cyclic, `6 % 6 == 0`
  balanced), so rotation remains the only thing that differs between A and B;
* **6 is `#3468`'s own stated minimum** — *"6 over 3 arms is the smallest
  balanced setting worth running"*;
* the sharpest of the three P3b criteria is the **sign split**, which is a
  proportion over 157 paired fixtures and does not weaken with fewer rounds.
  A wider CI makes "CI contains zero" **easier** to satisfy, so that clause
  alone is now a weak test and is not leaned on; the `|mean| ≤ 0.20 %` and
  `split ∈ [42 %, 58 %]` clauses carry P3b.

**No numeric criterion in §3 changes.** P3a, P3b, P3c and P4's band are as
registered. If the box turns out faster than the probe suggests, a 12-round
confirmation of A and B is added **after** the six-round set is complete and is
reported as a separate, later run — never as a replacement for one that already
read.

**Binaries are built and pinned before this is committed** (md5 recorded in the
rung): `c2rs-permute-{base,null,tip}` from `f6f56df78` / `0ff503eb0`, and
`c2rs-s1c3-{base,null,tip}` from `e85253cda` / `4d04ee59e`. In both sets the
null is a `cp` of the base binary and `cmp` returns 0; base and tip differ.

---

## 8. AMENDMENT 2 — RUN F, THE BUILD-TO-BUILD FLOOR. Registered before it runs.

Run A is complete and is a **MISS on P3a**: the *defective* rotation produced a
null of **−0.00 % [−0.11, +0.10], split 43 %**, which lands inside the band §3
registered for a GOOD run. The acceptance test is **UNPOWERED**, B–E were not
run, and nothing about the fix is claimed from timing. That is the branch §3
registered and it needs no amendment.

What needs one is what run A found on the way: its **tip** reads
**+1.27 % [+1.01, +1.52], split 76 %** on the same two commits `w-permute`
measured at **−0.49 / −0.44 / −0.55 %** (splits 27–32 %). Sign flip, ~1.8
points, **with the run's own null certificate clean**.

**Run F is the coordinator's design, and it is better than either option this
lane proposed**, because it isolates *build* from *change* completely instead of
mixing them.

### 8.1 The design

**Three independent builds of ONE commit**, `f6f56df78`, in three directories
of deliberately different name length (`b1`, `b2xx`, `b3yyyyyy`), run as three
arms under `--rotation balanced --rounds 6`. **Every pairwise difference has a
true value of exactly zero by construction** — same sha, same pinned toolchain,
same box, same session, same flags. It is the null-arm logic promoted one level:
instead of a `cp` (byte-identical, layout identical) it is an independent build
(semantically identical, layout possibly different).

### 8.2 The `cmp` precondition — CHECKED FIRST, and the experiment is LIVE

Registered condition: *if the three builds are byte-identical the test
degenerates into the existing null arm, and it is reported as that and stopped,
not presented as new evidence.*

    c2rs-b1        e63eb8bd50e4d97b5bb57c0c346edabd   6,126,264 bytes
    c2rs-b2xx      b9a82a769ecd04df850895f16bd4fdcd   6,126,296 bytes
    c2rs-b3yyyyyy  84268b6129ec827c59a275a14e104017   6,126,312 bytes

**All three differ, and the sizes track the directory-name length** (+32, +16),
so the mechanism is not `rustc` nondeterminism — it is the **embedded
`CARGO_MANIFEST_DIR`**, which `crates/c2-reference` takes with `env!`. That
makes run F a test of **build-directory variation specifically**, which is
exactly the real case: every lane builds in its own worktree at its own path.

### 8.3 There is no `nulldup` arm, ON PURPOSE

The script will warn *"no arm named 'nulldup' — this run has NO noise floor, and
any number it prints is a mean without a scale."* **The warning is correct in
general and inapplicable here**: in run F every arm is a null with respect to
every other, so the numbers this run prints **are** the noise floor. A `nulldup`
could not be added even if wanted — it would have to be `cmp`-identical to the
baseline, and the script would refuse anything else, which is the guard working.

### 8.4 P5 — the registered prediction and the three-way conclusion rule

Let **F** = the largest `|mean|` over the two pairwise readings, with its split.

**Prediction: `F` lands in ±0.2 % to ±0.7 %** — bigger than the byte-identical
null's ±0.11 %, because the embedded strings move `.rodata` and everything
aligned after it, but well short of the 1.8-point swing run A showed.

| outcome | what is concluded |
|---|---|
| **`F` < ±0.2 %** | Build-directory layout is **not** a plausible explanation for the 1.8-point swing. The sign flip needs another cause — round count (6 vs 9) or session — and #3468's +0.99 % and #3495's −0.55 % survive *this* objection. |
| **`F` = ±0.2–0.7 %** | A real build floor exists and is **comparable in size to the effects the COST CLAUSE measures**. Every published cost reading must be quoted beside it, and none of them was. The 1.8-point swing is then only **partly** explained, and is said to be. |
| **`F` > ±0.7 %** | **Every published cost reading in this project is inside the noise of its own build.** #3468's +0.99 %, #3495's −0.55 % and this lane's +1.27 % are then all unresolved, and the COST CLAUSE as practised cannot resolve what it claims to. |

**Scored on the mean AND the split**, per #3495: a true effect of exactly zero
must split near 50 %, and a split far from 50 % on these arms is as much a
finding as the mean.

**What run F canNOT do, registered so it is not overread:** it is one session on
one box with three builds. It establishes a floor **for build-directory
variation on this box today**; it does not establish that the same floor
explains `w-permute`'s numbers, whose binaries were reaped with its worktree and
**cannot be re-run**. That remains a disagreement without an established cause,
and run F narrows the candidate list rather than closing it.
