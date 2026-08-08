# w-ceiling — PRE-REGISTRATION

    Lane:      w-ceiling (docs-only)
    Worktree:  wt-w-ceiling off master `b234d826`
    Written:   2026-08-08, BEFORE the first probe of this lane.
    Scored in: docs/rungs/2026-08-08-w-ceiling.md §7

This lane ships **no code**. It ships `docs/CEILING.md` (the arithmetic between
today's process and TU match 871) and one dated annotation block in
`docs/ROADMAP.md` closing board **#1476**'s open half. Every prediction below is
about a number I have not yet measured at this tree.

---

## §0 What I have already read, and what I am registering as ALREADY-KNOWN

Registering this first, because w-bcgap §2's lesson is that a lane whose brief is
stale can ship a duplicate and report it as a success.

* **`c2rs factors` exists** (`crates/c2-harness/src/cli/factors.rs`, board
  **#1520**, landed by w-bcgap). I do not rebuild set algebra; I *run* it.
* **The per-TU factor listing exists** (`gap --factors-tsv`, board **#352**).
* **The HAND-COUNT / INSTRUMENT convention exists** (`docs/BOARD.md`
  Conventions, board **#1476**, lane `w-column`). I do not redefine it; the open
  half is the `ROADMAP.md` sweep, which #1476's own row says "this lane did not
  sweep".
* Two source facts I read before writing this, from `crates/`, not from a run:
  `crates/c2-harness/src/gap/scan.rs:387-393` computes `today` / `repaired` /
  `wall` as a **three-way partition predicate per TU**, and
  `crates/c2-harness/src/cli/gap.rs:534-537` prints and asserts
  `repaired + wall == graded`.

**Numbers in my brief are not sources.** The brief states a baseline of
`match 11 · mismatch 0 · vocab-gap 860 · capture-fail 7 · FRONTIER 16` and
factor counts `A 28 · B 338 · C 169 · B∧C 151 · A∧B∧C 27`. Those are the
*predictions* below (K1–K7), not inputs.

---

## §1 Known-answer controls — the base must re-derive what master publishes

If any of these disagrees, the disagreement is the lane's headline and CEILING.md
is written against **my** run, not against the disagreeing page.

| # | key | registered |
|---|---|---|
| **K1** | `graded` | exactly **871** |
| **K2** | `factor-a` / `factor-b` / `factor-c` | **28 / 338 / 169** |
| **K3** | `b-and-c` | exactly **151** |
| **K4** | `a-and-b-and-c` | exactly **27** |
| **K5** | `frontier` / `frontier-if-a` | **16 / 138** |
| **K6** | scan headline | `match 11 · mismatch 0 · codegen-gap 0 · vocab-gap 860 · capture-fail 7` |
| **K7** | `c2rs factors --check-metrics` | **13 OK, 0 DISAGREE, 0 ABSENT of 13** |

A concurrent code lane (`w-cfgclass`) owns `crates/`. If K1–K7 move under me I
report the tree hash of the collection and do **not** average two trees.

---

## §2 The 450-wall — does it survive re-derivation?

**Registered prediction: it survives as a NUMBER and DIES AS A CEILING.**

My prediction, from reading `scan.rs`/`gap.rs` and *not* from a run:

* **P1** — `emit-set-ceiling-wall` re-derives at **450** at my base (interval
  [430, 470]; the collector's own self-check text still says 451, which is older).
* **P2** — `emit-set-ceiling-repaired` re-derives at **421** (interval
  [410, 440]).
* **P3** — **`repaired + wall == graded`, i.e. `421 + 450 == 871` exactly.**
  This is the load-bearing one.
* **P4** — Therefore **"the 450 wall" is a COUNT OF BLOCKED TUs, not a ceiling**:
  it is the complement of `repaired` inside the graded population, and the
  ceiling that population implies is **421**, not 450. I register now that I
  expect at least one place in `docs/` to read 450 as though it were a reachable
  figure — `scripts/status.sh`'s own registry label is
  `Emit-set MODEL ceiling (today / repaired / wall)`, which puts the word
  "ceiling" in front of all three.
* **P5** — I expect the phrase to be **superseded rather than wrong**: 450 is a
  real measured quantity and I will publish it under its true name, not delete
  it.

**Decline clause D1:** if `repaired + wall != graded` at my base, I stop, report
the broken invariant as the lane's finding, and do **not** publish a ceiling
ladder built on either number.

---

## §3 Cost per converted TU

The folklore figure is *~5 TUs per ~161 lanes* (≈ 32 lanes/TU), and my brief says
its window was never verified. I will compute it with the denominator stated.

Numerator, from the rung record (I have grepped the transition lines but not yet
verified each): TU match **6 → 11** over four landed conversions —
`w-r1c` 6→8, `w-tu1`/W42-W43 8→9, `w-hash` 9→10, `w-lineage` 10→11.

* **P6** — the numerator is **+5 TUs** and there are **exactly 4** conversion
  events in the rung record between 6 and 11. Interval: 4–6 events.
* **P7** — denominator A, **every landed rung at my base** (non-`_` files in
  `docs/rungs/`, which is exactly what `gen_rung_index.sh` indexes): point
  **185**, interval [150, 230].
* **P8** — denominator B, **rungs landed inside the conversion window**
  (first-conversion date through `w-lineage`): point **120**, interval
  [80, 170].
* **P9** — the headline **cost per converted TU on denominator B** comes out at
  point **24 lanes/TU**, interval **[16, 40]**. I register that this is
  **cheaper** than the folklore 32, and that the reason will be that the
  folklore denominator counted lanes that landed *after* the last conversion.
* **P10** — the *recent* rate is worse than the average: I register that the
  count of landed rungs since the **last** conversion (`w-lineage`) is
  **≥ 10 and still 0 TUs**, so the marginal cost is not the average cost.

**Decline clause D2:** if the rung record cannot date the conversions
unambiguously (e.g. a conversion lands in a merge with no rung), I publish the
window as a **range** and say so, rather than picking the flattering end. The
prior "5 per 161" claim is exactly what an unstated window produces.

---

## §4 The ROADMAP sweep (#1476's open half)

* **P11** — the count of distinct pre-2026-08-08 `ROADMAP.md` claims carrying an
  untagged codegen number: point **35**, interval **[12, 90]**. This is the
  prediction I am least confident in; the interval is wide on purpose.
* **P12** — **every one of them classifies as HAND-COUNT**, not INSTRUMENT. I
  register **0** pre-2026-08-08 codegen numbers in `ROADMAP.md` that name a
  `gap-metric` key, because #1464 proved the codegen column did not exist until
  #1473/#1474 built it on 2026-08-08. If I find even one, that is a *refutation*
  of #1476's default rule and it is the headline.
* **P13** — I annotate with a **single dated block**, and edit **zero** dated
  sections in place. Verified mechanically: `git diff` on `ROADMAP.md` touches
  one contiguous added region plus at most one cross-reference line.

---

## §5 The estimate-streak calibration

* **P14** — board **#770**'s streak at my base reads, from the rows that score
  it, on the order of **ten optimistic · two pessimistic · one-to-two hits**,
  and the most recent scoring row is **#1459**. I register that the streak's
  *shape* (optimism dominant, with the two pessimistic misses both on
  bracket/depth questions) is what CEILING.md must apply, and that the
  application is: **every forward number in CEILING.md is to be read as a lower
  bound on cost.**
* **P15** — I will not publish a forward *point* estimate of when 871 is
  reached. Registering that as a refusal now so that its absence is not read as
  an oversight.

---

## §6 What this lane will NOT do

1. **It will not decide the re-scope.** CEILING.md equips the decision; the
   decision is the user's. If the arithmetic reads badly I publish it as
   arithmetic.
2. **It will not price codegen.** Any codegen number in CEILING.md is either
   `frontier-codegen-*` (INSTRUMENT, #1474) or carries a HAND-COUNT tag and a
   citation. **Never a hand-count and a scan reading in the same sum** (#1476).
3. **It will not touch `crates/`, `scripts/` or `fixtures/`.** Verified by
   `git diff <base> -- crates scripts fixtures` being 0 bytes at the end.
4. **It will not re-run w-bcgap's model intersections.** ALIAS_IN's 110/124 is
   cited from `rungs/2026-08-08-w-bcgap.md` §5 with its date, not re-measured.
5. **It will not claim `mismatch 0` as evidence of anything.**

---

## §7 Gate

`scripts/gate.sh --require-graded` must PASS before I report. Baseline to beat,
from the brief and to be re-derived, not assumed: 18/18 lanes, 5,202
fixture-verdicts, sweep 19,460/19,556, cross 90,424/90,812, hatch-red 14/14,
ladder-red 5/5, **0 mismatch**. A run that grades 0 is a failure, not a pass.

**Decline clause D3:** a docs-only lane cannot break the gate. If it goes red,
the cause is the concurrent code lane or my base, and I report that rather than
claiming a pass.
