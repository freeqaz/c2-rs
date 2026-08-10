# w-gatefix — PREREG

**Honesty note about registration timing, first, because it is the thing a
reader should distrust.** A tooling lane's beliefs move as the code is read,
and this file is not one frozen block. Each row records **the point in the lane
at which it was written down**, and rows registered after the relevant
measurement are marked `POST` and are **not scored** — a claim written after its
answer is not calibration data. Six of the eight below are `PRE` at a named
point; two are `POST` and say so.

`CEILING.md` §5's standing lesson is that the misses here are on FORWARD COST
and that optimism dominates ~5:1. This lane's forward-cost row is P6.

The two freshest standing lessons are both antecedent failures — *"an antecedent
that only makes your registered clause true is not the antecedent the claim
needs"* (`w-seclayout` P3), and a deciding row that imported a `/Ox` antecedent
for a `/O1` population (`w-seclayout` P5). **The deciding row here is P4**, and
its antecedent is written to be the one the claim needs, with the falsifier
spelled out.

---

## P1 — the gate has no dirty-`crates/` interlock at all

`PRE`, registered after reading the commission and `docs/rungs/2026-08-10-w-seclayout.md`
§10, **before opening `scripts/gate.sh`**.

> **p = 0.75.** The commission describes an *"unconditional
> `git checkout -- crates/`"* reached from `gate.sh`'s first row. The simplest
> code that produces that is a row with no guard in front of it, and the
> deliverable asks me to add one.

**Falsifier**: a guard already present in `hatch_red_run`.

---

## P2 — with the interlock in place, a dirty tree will survive a gate run

`PRE`, registered after reading `hatch_red_run` and seeing
`git diff --name-only HEAD -- crates/` sitting right there, **before running
anything**.

> **p = 0.70.** The interlock is wide (all of `crates/`), reads `HEAD` rather
> than the index, and refuses with its own word. If it fires, nothing invokes
> the script and the edit survives. My residual 0.30 is "then why did
> `w-seclayout` lose their work".

**Falsifier**: a real run at the merge base with a dirty `crates/` that comes
back `REFUSED HATCH-STALE` instead of `DIRTY-TREE`.

---

## P3 — whatever the mechanism is, it is *upstream* of the interlock

`PRE`, registered immediately after P2 failed (`reproA`), **before finding it**.

> **p = 0.85.** The row reported `HATCH-STALE`, which is only produced from the
> script's own log — so the script ran, so the interlock passed, so the tree was
> already clean when the interlock looked. Something between the top of
> `hatch_red_run` and the interlock reverted it. There is exactly one candidate
> shape: an invocation of the subject above its own guard.

**Falsifier**: the tree being cleaned by something outside `hatch_red_run`
entirely (a peer session, a `mode_lane.sh`, the reaper).

---

## P4 — **THE DECIDING ROW.** No landed gate claim on this project is invalidated

`PRE`, registered after the mechanism was known and after `audit.py` part 1 was
written, **before `audit2.py` (the exposure window) or `audit3.py` (the
intersection) were run**.

> **p = 0.80.**
>
> **The antecedent the claim needs — and it is a conjunction of three, not one:**
>
> 1. **`git checkout -- <path>` restores from the INDEX**, so the edits it
>    destroys are exactly the *unstaged* `crates/` modifications, and after it
>    runs `crates/` equals the index's `crates/`, which on every lane here is
>    `HEAD`'s. Therefore **every affected run graded exactly the `crates/` of
>    the commit it named.** The report was never false about what it graded.
> 2. **The row that invokes `hatch_red.py` did not exist before `378a3cae`**
>    (2026-08-08). Runs whose named commit lacks that ancestor are not merely
>    "probably fine" — the code that could destroy anything was not in the file
>    that produced them.
> 3. So the only way a *landed* claim can be wrong is a rung asserting its gate
>    covers `crates/` work that was uncommitted at gate time and therefore
>    eaten. That is visible in git iff the work later landed, i.e. iff the
>    lane's landed `crates/` differs from its transcript's `crates/`.
>
> I register that (3)'s population is small and that **every member of it will
> turn out to be non-emitting** — tests, test fixtures, comments — because a
> lane that lost real codegen would have gone red on its own cells and re-gated.
>
> **What is NOT claimed**: that (1) makes the defect harmless. It cost
> `w-xtea2` a re-applied edit and `w-seclayout` a retracted claim. Those are
> real and neither is a landed *gate* claim.

**Falsifier, written before the measurement**: any in-window transcript whose
lane landed a `crates/` delta that touches a line **outside `#[cfg(test)]` and
outside a comment**. One such row falsifies P4 outright — it means a landed
`GATE: PASS` covers port sources that are not the port sources that shipped, and
the lane must be named and re-gated.

**Second falsifier, for the antecedent rather than the conclusion**: any
in-window transcript showing `REFUSED DIRTY-TREE` **together with** evidence
that an unstaged edit was destroyed. That would break clause (1) — the index
argument — and the whole reading collapses to "unknowable from transcripts".

**Why this row is not unlosable.** It has a population (36 in-window
transcripts), a decision rule that could return "yes" (a non-test, non-comment
line), and both falsifiers are things I could actually have found. The
`w-seclayout` P4 lesson is why they are written down.

---

## P5 — the fix moves nothing measurable in the port

`PRE`, registered before the tip gate.

> **p = 0.97.** The lane changes **0 bytes** under `crates/` and `fixtures/`, so
> all four neutrality levels are unchanged *by construction* and the binary
> should be **bit-identical** to the merge base's.
>
> The 0.03 is not modesty about the argument — it is `w-seclayout`'s retracted
> byte-identical claim, which failed because their tree *did* change (doc
> comments shifting `#[track_caller]` line numbers). Mine does not, so the same
> failure needs `crates/` to embed something outside `crates/` — a build script
> reading `git`, an `env!` of the tree — which I have not exhaustively excluded.

**Falsifier**: the tip gate pinning a binary sha other than the merge base's
`0a252107b376`.

---

## P6 — the forward-cost row: this fix will make the five pre-existing
## `hatch.py` arms newly loud

`PRE`, registered before the tip gate.

> **p = 0.25 that they become newly visible.** `R2/R6/A2/F1/C1` have failed for
> seven lanes because `hatch.py apply` cannot hatch this tree (`HATCH-DRIFT`,
> #1322/#1389). The gate reads `SETUP FAILED` before the arm table and reports
> `REFUSED HATCH-STALE`, which is a tree property and forfeits the unqualified
> headline. Nothing I changed touches that ordering.
>
> Registered at 0.25 and not 0.05 because `CEILING.md` §5 says my forward-cost
> numbers run optimistic 5:1, and "my change is inert over there" is exactly the
> forward-cost claim that keeps being wrong on this project.

**Falsifier**: the tip gate's hatch-red row reading anything other than
`REFUSED HATCH-STALE`, or the arm counts moving.

---

## P7 — `POST`, not scored: the residue-postcondition defect

Recorded for the next lane, **after** it was observed. Both red rows'
postconditions were `git diff HEAD -- crates/` rather than a delta, so the first
`--allow-dirty-crates` run failed the gate on `LADDER-RESIDUE` naming a file no
ladder arm has ever touched. I did not predict this; I found it by exercising
the flag I had just added, which is the only reason it is in the lane at all.

## P8 — `POST`, not scored: `--list` was the destructive step

Recorded because it is the finding, not because it was predicted. P3 registered
"upstream of the interlock"; it did **not** name the arm-count probe, and I
would have guessed a `mode_lane.sh` or a stale `pin_harness` before guessing
that a `--list` whose docstring says *"runs nothing"* runs a `git checkout`.
