# w-magic — PREREGISTRATION

Lane `w-magic`, worktree branch off master **`f128b21`**.
**Committed before the first grid cell was compiled and before `kgrid.py`
existed.** Anything below that is graded against real `c2.dll` output was
written down first.

## 0. What was already known when this was written

Read, in this order, before a single claim below was drafted: `docs/STATUS.md`;
`docs/rungs/2026-08-05-w-divsplit.md` in full; board rows #780–#784 and
#816–#824; `crates/c2-core/src/codegen/div_mod_leaf.rs`;
`docs/rungs/2026-08-05-w-divmod.md` §3–§4; `docs/rungs/2026-08-05-w-hash.md`
§5.1; `work/w-hash/divgrid.py`.

Three published facts are **inputs**, not findings of this lane:

* `w-hash` §5.1, at `/O1`: `s-mod-k7` (`a % 7`) reads **`li divw mulli subf`**
  and `s-mod-k2` (`a % 2`) reads **`srawi addze rlwinm subf`**.
* `w-divmod` §3: *"`addi ; divw ; mulli ; subf` for `%7` … `addis ; ori ; divw ;
  mullw ; subf` outside `simm16`"*, and **no non-zero literal divisor emits any
  trap** over 24 cells; `k = 0` emits `twi 7, r0, 0` and no division; `a / -1` is
  a bare `neg`.
* `#822` states **3,950 of 4,674 sites "need a magic-number multiply"**. That
  figure is an **inference from the divisor's value** (non-power-of-two), not a
  measurement of any emitted byte. `w-divsplit` never disassembled a
  constant-divisor division.

**The two published bullet points and the third are in direct tension**, and
noticing that is the whole reason this lane exists. R1 is that tension made
falsifiable.

## 1. The registered claims

Each is graded against real `c2.dll` under wibo. `HIT`/`MISS` is decided by a
count with a denominator, never by a status line.

### R1 — the headline, and it can lose

> **At `/O1`, a non-power-of-two constant divisor is lowered to a REAL
> hardware divide (`divw`/`divwu`) with the constant materialized into a
> register — there is NO magic-number multiply, and `mulhw`/`mulhwu` does not
> appear at all.**

Threshold: `mulhw`/`mulhwu` occurs in **0** of the `/O1` non-power-of-two cells.
**MISS if it occurs in ≥ 1.**

If R1 holds, **#822's "3,950 need a magic-number multiply" is refuted at the
workload's own optimization mode** and the embedded-division rung gets
*cheaper*, not dearer — the first time that has happened on this project.

### R2 — the mode cliff, registered in the opposite direction

> **At `/Ox` (and `/O2`), c2 DOES emit a magic-number multiply for at least one
> non-power-of-two constant divisor.**

Threshold: `mulhw`/`mulhwu` occurs in **≥ 1** `/Ox` cell. **MISS if 0.**

R1 and R2 are registered together on purpose: they cannot both be vacuous, and
whichever way each falls, the pair is the deliverable. If **both** hold, the
statement is *"the magic multiply is an `/Ox` phenomenon and the workload never
sees one"*. If R1 misses, the workload does see one and the map is the
deliverable. If R2 misses, `c2` has no magic-multiply path at all at any mode
graded here.

### R3 — the number of regimes in the `k` axis

> **At `/O1`, signed `/`, the number of distinct instruction-sequence shapes
> (mnemonic sequence, ignoring register fields and immediate values) over the
> whole `k` axis is 7.**

Registered as a point estimate with an accepted band of **6 to 9**. The seven
predicted, each with the `k` that forces it:

| # | regime | predicted shape |
|---|---|---|
| 1 | `k = 0` | `twi 7,r0,0` — no division (published, a control) |
| 2 | `k = 1` | identity, `mr`/nothing |
| 3 | `k = -1` | `neg` (published, a control) |
| 4 | `k = +2^n`, `n ≥ 1` | `srawi ; addze` |
| 5 | `k = -2^n`, `n ≥ 1` | `srawi ; addze ; neg` |
| 6 | non-pow2, fits `simm16` | `li ; divw` |
| 7 | non-pow2, outside `simm16` | `addis ; ori ; divw` |

**MISS if the count is outside [6, 9].** Regimes 1, 3 and 6/7's boundary are
published; 2, 4, 5 and the `-2^n` split are this lane's predictions.

### R4 — the `k → magic` map, IF a magic multiply exists

> **Where a magic multiply is emitted, the multiplier `M` and shift `s` are
> exactly those of the standard Granlund–Montgomery algorithm** (Hacker's
> Delight `magic()` / `magicu()`), and the fixup is: add the dividend back iff
> `k > 0 ∧ M < 0`, subtract iff `k < 0 ∧ M > 0`, then `srawi s` then `addze`.

Graded by **generation, not prediction**: `kgrid.py` computes `(M, s)` from `k`
alone and the grader compares against the immediate fields decoded out of the
emitted words. Threshold: agreement on **≥ 90 %** of magic-bearing cells, and
the disagreeing cells named individually. **Vacuous if R2 misses**, and that is
stated here so it cannot be quietly claimed later.

### R5 — #644, aimed at this lane

> **In at least one cell, the `lis`/`ori` pair that materializes a wide constant
> is SPLIT by at least one intervening instruction.**

Threshold: ≥ 1 cell where the `ori` is not at `lis`'s index + 1. **MISS if the
pair is contiguous in every cell.** This is registered as a claim that can lose:
`w-divmod` published `addis ; ori ; divw ; mullw ; subf` as contiguous, so the
straightforward reading is that it MISSES. It is registered anyway because #644
has fired three times and the population here is wider than w-divmod's.

### R6 — the `mulli` boundary, carried over from w-hash's `a * k`

> **In the `%` fixup, the multiply-back is `mulli` iff `k` fits `simm16`
> (`-32768 ≤ k ≤ 32767`) and `mullw` otherwise** — the same cliff w-hash found
> for `a * k`, on a different instruction's operand.

Threshold: the predicate is exact over the `%` cells — **0 counterexamples in
either direction**. MISS on ≥ 1.

### R7 — conservation, and the bar

> `fnbyte-differs` is **0** at both ends of this lane; TU match **10 → 10**;
> mismatch **0**; census/gate disagreement **0**; per-function census and
> emitted census **unmoved unless this lane ships an accept path**, in which
> case both move by exactly the amount the accept path's own gate reports.

Verified by re-running the 878-TU scan at **both** ends, not inferred from "no
code changed".

### R8 — the honest expectation about conversions

> **Any emit this lane ships converts ZERO workload functions.**

Registered because #782 shipped a leaf graded 185/185 that converts zero, and
#822 measured the overlap between that leaf and this population at **0 sites**.
The straight-line constant-divisor population is **4 of 4,674** (`w-divsplit`
§8). Four bodies, each of which needs the rest of its body modeled too. **MISS
if any emit this lane ships converts ≥ 1 function** — and a MISS here is the
best possible outcome, which is why it is registered in this direction.

### R9 — the `unsigned` axis is cheaper than the signed one

> **For `unsigned`, a power-of-two divisor is a bare `rlwinm` (a shift/mask)
> with no `addze` fixup, and the signed `srawi ; addze` pair has no unsigned
> counterpart** — because the round-toward-zero correction only exists for
> negative dividends.

Threshold: **0** `addze` in any unsigned power-of-two cell. MISS on ≥ 1.

## 2. What this lane will NOT do

* It will **not** ship a loop lowering. `w-divsplit` §8 measured the population
  at **4,649 of 4,674 `cflow-loop`** and this lane's brief says plainly that
  building that is a later rung's job.
* It will **not** fit a `twi` placement rule. #780 refuted three readings,
  `w-pair` §4 accounts for six more and `leaf_store.rs` four; a rule fitted here
  would be the eleventh. If R1 holds, non-zero constant divisors emit **no
  trap at all**, so the question does not arise for this population.
* It will **not** add a neutrality or behavior-preserving classifier as a gate.
* It will ship **no emit at all** if the graded rule does not reach an accept
  path cleanly. An honest negative result is the accepted deliverable.

## 3. The controls, named before they are run

1. **`plain-add`** (`a+b` → two words) — the capture seam works.
2. **`s-mod-var`** (`a%b`) — must reproduce `div_mod_leaf`'s published nine
   words exactly. This is a **cross-lane** control: it grades this lane's
   capture path against a body another lane already shipped byte-exact.
3. **`k = 0`** must read `twi 7,r0,0` and **`k = -1` signed `/`** must read a
   bare `neg` — two published cells re-derived here. If either disagrees, the
   capture path is wrong and no other row in this lane is readable.
4. **Cross-instrument**: every instruction word is decoded twice — once by the
   grid's own decoder and once by `scripts/gt_dump.py` — and the two must agree
   on the mnemonic sequence for every cell. #823 is the reason: a lane whose
   only control is that its buckets sum has no control.
5. **Held-out `k`.** P3 (`floor((N-1)/2)` fitting three cells and dying at N=5)
   says: do not validate on the cells fitted. The regime map is derived on a
   **fitting set** of `k` and then predicted, before compiling, on a
   **disjoint held-out set**, and the held-out prediction is graded as a
   separate number with its own denominator.
6. **A must-fail mutation** for any accept path that ships, with the bar
   w-varloop set: real mutations that produce *wrong bytes*, not refusals.

## 4. The `k` axis

Fitting set and held-out set are fixed **now**, before any compile:

* **FIT**: `0, 1, -1, 2, -2, 3, -3, 4, 7, -7, 8, 16, 10, 100, 1000, 32767,
  32768, -32768, -32769, 65536, 100000, 2147483647, -2147483648`
* **HELD OUT** (not compiled until the map is written down): `5, -5, 6, 9, -9,
  12, 20, 24, 25, -25, 64, 1024, 4096, 30000, -30000, 40000, -40000, 65535,
  131072, 1000000, 2147483646, 732`
* The workload's own divisors (`w-divsplit` §9: 20, 2, 24, 12, 6, 40, 60, 56,
  28, 84, 44, 100, 48, 96, 72, 88, 36, 732, 3, 76) are graded as a **third,
  separate** set with its own denominator, because they are the only ones whose
  count is load-bearing.

Crossed with: `signed`/`unsigned` × `/`/`%` × `/O1`/`/Ox`. Unsigned cells take
only `k ≥ 0`.

---

*Nothing below this line was known when the file was committed.*
