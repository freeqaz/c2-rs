# w-empty — PREREGISTRATION

    Lane:    w-empty
    Base:    master `9827bcf` (merge w-inline)
    Written: 2026-08-06, BEFORE any probe, any cell and any compile exists in
             this worktree. `work/w-empty/` contains this file and nothing else
             at the commit that introduces it.
    Mission: ship **mechanism E** — the front end drops a call to a source-empty
             same-TU callee — into the port, graded by the FBM partition.

The spec being consumed is `docs/INLINE_PREDICATE.md` (lane `w-inline`). This
lane does **not** rewrite it, does not re-derive `INLINE-P`, and does not touch
mechanism I.

---

## 0. What is already established, and is therefore not a claim of this lane

* **E exists and is not `/Ob`-governed.** `INLINE_PREDICATE.md` §1: `void g(){}`
  / `void f(){ g(); }` emits `?f` as a bare `blr` with 0 relocations at `/O1`
  **and at `/Ob0`**, while `int g(int a){return a;}` — the identical single-`blr`
  callee — keeps `bl ?g` at `/Ob0`. Twelve more boundary probes in §1.1.
* **Family A is 1,886 of the 4,711 `fnbyte-differs`**, all shape `tail`, and it
  is exactly two witness signatures (`work/w-fnbyte/differ_taxonomy.txt`):
  `1516 tail|w1/1/eq0|first@0:port=48000000,ref=4e800020` and
  `370 tail|w2/1/eq0|first@0:port=38a00000,ref=4e800020`.
* **`IlBundle::functions()` refuses any TU that defines one of its own callees**
  (`crates/c2-il/src/func/bundle.rs`), so no obj has ever been emitted for a TU
  where E could fire. `mismatch` is 0 for that reason and this lane does not
  change it.

---

## 1. The registered claims

### P1 — **the claim I most expect to lose.** The caller's whole body collapses

> When E fires, c2's caller body is a bare `blr` **and nothing else** — the
> argument setup for the dropped call is gone too, for every argument expression
> that is free of side effects.

It can lose in a way I would have to ship: if any graded caller cell keeps its
argument-setup words and *then* returns (`li r5,0 ; blr`), the port rule is not
"emit `blr`" but "emit the setup, then `blr`", and the two families
(`w1/1` = 1,516 and `w2/1` = 370) would need different treatment. The `w2/1`
signature — port `38a00000 ; b`, ref a single `4e800020` — is the workload's own
evidence for the collapsing reading, but it is evidence from a *differ witness*
and not from a probe, so it is registered as a claim and not as a premise.

**LOSS CONDITION**: any caller cell in the grid whose reference `.text` is longer
than one word while its callee is graded E.

### P2 — the discriminator holds in the IL, not just in the obj

> `int g(int a){ return a; }` (INLINE_PREDICATE §1.1 `e3`, w-inline `p6`) is
> **not** E. The port's shipped predicate must refuse it, and it must refuse it
> because the IL body is not empty — not because of a size, a linkage or a
> return type.

Loses if the port's `IlFunction::empty_body` (or whatever the shipped predicate
reads) is true for `e3`, or false for a cell the grid grades E. **Either
direction is a loss and both are reported.**

### P3 — the differs delta, registered as a RANGE before the change exists

> `fnbyte-differs` falls from **4,711** into the interval **[1,300, 1,886]** of
> functions removed — i.e. it lands in **[2,825, 3,411]** — and `fnbyte-exact`
> rises by **exactly** the number `differs` fell by.

The upper end is family A entire. The lower end allows for family-A callers
whose callee's IL body the decoder cannot establish as empty (a `census_functions`
row that came back `Err`, or an `empty_body` that the parser spells some other
way), which the conservative rule must refuse rather than guess.

**A result above 1,886 is also a loss**, because it would mean the predicate is
firing outside family A — see P4.

### P4 — **CONTROL, and the one that can go red in the most likely failure
direction.** The previously-exact population does not shrink

> `fnbyte-exact` is **34,466** at the base, of which **4,051 of 7,098 `tail`
> bodies are exact**. Every one of those is a `tail` whose `setup + b callee`
> already equals c2's bytes — so c2 **did** emit that branch. If the shipped
> predicate fires on any of them the port replaces a byte-exact body with a
> `blr` and `fnbyte-exact` falls.

Registered target: `exact_after >= 34,466`, and the per-shape breakdown
(`fnbyte-shape|tail|fnbyte-exact` etc.) printed at both ends. **This is a
precision test on 4,051 real cases and it is the strongest control this lane
has.** #232's shape — a widening that turns honest bytes into wrong bytes —
would appear here first.

### P5 — CONTROL: nothing else moves

> `fnbyte-partition-broken` 0 · `fnbyte-match-tu-differs` 0 ·
> `fnbyte-census-disagree` 0 · `census/gate disagreement` 0 · scan `mismatch` 0
> · `codegen-gap` 0 · TU match **10** · `vocab-gap` 861 · `capture-fail` 7 ·
> factors A/B/C/D/E and `B∧C`/`A∧B∧C`/FRONTIER unchanged, at both ends.

`IlBundle::functions()` is **not** widened by this lane, so `mismatch` cannot
move for the reason `w-fnbyte` §5.3 gives. If it does, the lane is wrong.

### P6 — must-fail mutations, three of them, each with a distinct message

> Dropping the **same-TU** condition (elide a call to any empty-*named* callee,
> including one this TU does not define); dropping the **emptiness** condition
> (elide every same-TU call); and applying E to a **non-empty** same-TU callee —
> each goes RED, in the FBM partition or in a committed unit test, with a
> failure message that names which condition was dropped.

### P7 — the grid is frozen before it is compiled

> Every cell's `.cpp` is `sha256`-stamped and the stamp file is committed in the
> same commit as the cells, **before** the first `cl.exe` invocation. The
> predicate that ships is exactly the one the grid measured, and it **refuses at
> every cell the grid did not grade** — a cell graded `NOT MODELLED` produces a
> refusal in the port, never a guess.

### P8 — spot-verification by hand

> At least **20** functions that move `differs → exact` are printed word-for-word
> against the reference COMDAT and checked by eye, and the count is reported. A
> function that moved must have moved because its bytes now equal c2's, not
> because the instrument stopped grading it — the partition identity (P5) is the
> structural half of this and the hand check is the other half.

---

## 2. What this lane will NOT do

* **No `IlBundle::functions()` widening.** The accept boundary for whole TUs is
  untouched; the FBM partition is the grader. A TU that defines its own callee
  stays refused.
* **No mechanism I.** No size, no `index`, no `N_max`, no cost model.
* **No new "neutrality" or "behaviour-preserving" classifier** as a gate
  (CLAUDE.md).
* **No fitting.** No constant, threshold or exception is introduced to repair a
  grid cell after it is measured. A cell that disagrees with the rule is
  published as a disagreement and the rule refuses there.

---

## 3. The decline floor

If **either** of these holds at the end, the port change is **reverted** and the
lane ships the grid and the rung with a negative result:

1. `fnbyte-exact` falls below 34,466 (P4 red), **or** the FBM partition identity
   breaks, **or** scan `mismatch` moves off 0.
2. The differs delta lands **outside** [1,300, 1,886] (P3), *unless* the reason
   is itself measured and published — an over-fire above 1,886 is a decline in
   every case, because it means the predicate reaches functions family A does
   not contain.

A revert is committed with its reasoning, not silently dropped.

---

## 4. Addenda

New grids and new claims go in dated addenda **below this line, in their own
commit, before the cells they describe are compiled**. Nothing above this line
is edited after the commit that introduces it.
