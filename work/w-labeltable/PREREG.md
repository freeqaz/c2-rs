# w-labeltable — PREREG

**Frozen and committed BEFORE the first `cl.exe` of this lane.** Nothing below
was written after a capture. The lane is `Kind: characterization`, and its
deliverable is *numbers re-measured against the oracle*, not a lift.

## 0. The question

`docs/LABEL_COUNTER.md` §4.2.1 publishes a 17-row leaf-loop surcharge table.
Four separate lanes have measured that table wrong, always in the direction that
makes a fence look **dearer to lift than it is** (`#3091`/`#3126`, `#3148`).
`w-slots` left the re-audit as its top found-and-not-taken, because
`work/w-slots/lead.py` is now a generic **seed-cancelling** instrument and the
measurement is one command per row.

**Re-measure every row against the oracle. Where the published number and the
obj disagree, the obj wins and the row says so.**

## 1. The two instruments, and why they should agree

| | `gt_label_stride.py` (what §4.2.1 was measured with) | `work/w-slots/lead.py` (what settled `#3091`) |
|---|---|---|
| TU shape | `a0 · P · a1 · a2`, anchors framed | `[P …, z9]`, z9 framed and LAST |
| the readout | `stride(P) = first(a1) − first(a0) − base` | `real $M(z9) − (counter + 9 + 3·segs + nleaf)` |
| seed | cancels — a **difference inside one obj** | cancels — each TU's **own** `.gl` counter is subtracted |
| published as | §4.2.1's `stride`, and `surcharge = stride − stride(leaf-none)` | a `label_lead` |

`coff::plan_labels` charges a leaf `label_lead + 1`, so `stride(P) = lead(P) + 1`
and **§4.2.1's `surcharge` column and `lead.py`'s `LEAD` are the same quantity.**
Both are seed-free. **They should agree row for row, and if they do not, one of
them carries a shape-dependent term and this lane reports which rather than
picking.**

This is the reason the per-row priors below are *high*: unlike `LABEL_LEAD.md`
(`#3148`) and `work/w-bdnz/LABEL_LEAD.md`, §4.2.1 was **not** differenced across
two TUs. The registered expectation is that most of the table reproduces.

## 2. The method correction this lane is required to apply (`#3147`)

`w-slots` followed *"read the charge out of the fixture's own obj"* and **the
objs read 3; shipping 3 would have been a wrong obj.** The series over 1/2/3
loops is `2n+1`, not `3n`.

> **A single cell's obj is a measurement OF THAT CELL. Only a SERIES separates a
> per-function charge from a per-TU constant.**

**So every number this lane publishes is a series.** For every row, `n` copies of
the probe body precede the same framed `z9`, for `n = 1, 2, 3`, and the row
reports the triple `L(1), L(2), L(3)` and the fit `L(n) = k·n + c`. A row is
published as a charge `k` **only if `c = 0`**; a non-zero `c` is a per-TU slot
and is named, not folded in.

At each row this lane states **whether the cells share a `.gl` counter**
(`#3148`: a lead differenced across two TUs is not a lead).

## 3. Populations, and the denominator for every number

**`match` has three meanings in this repo (`#3125`).** This lane publishes no
`match` movement of any kind; it is docs + `work/` only. Denominators:

| number this lane will publish | population / denominator |
|---|---|
| rows re-measured | **17** — `docs/LABEL_COUNTER.md` §4.2.1's leaf rows |
| framed rows cross-checked | **6** — §4.2.1's "§4's FRAMED row" column, i.e. §4's `if`/`while`/`do-while`/`for`/`fornest`/`goto-back` |
| cross-TU rows re-differenced | **8** — `work/w-bdnz/LABEL_LEAD.md`'s table, quoted verbatim into the shipped `IlFunction::label_slots` doc comment |
| series cells | **17 rows × 3 `n`** = **51**, plus controls |
| discriminating cells | printed by the instrument, see §6 |
| `match`, fixture gate | **381 fixtures × 18 mode lanes** — registered **+0** |
| `match`, `c2rs perf` | **381 fixtures at the `/Ox` default** — registered **+0** |
| `match`, workload scan | **878 dc3 TUs** — registered **25 → 25, +0** |

## 4. Registered claims — CEILING, no discount factor

### 4a. Per-row: does the published §4.2.1 number hold under the seed-free lead at `n = 1`?

| row | published surcharge | P(holds) |
|---|---:|---:|
| `leaf-none` | +0 | 0.97 |
| `leaf-if` | +0 | 0.90 |
| `leaf-while` | +2 | 0.85 |
| `leaf-dowhile` | +1 | 0.85 |
| `leaf-for` | +2 | 0.85 |
| `leaf-for-k` | +2 | 0.80 |
| `leaf-for-stride` | +2 | 0.85 |
| `leaf-for-down` | +2 | 0.85 |
| `leaf-for-cont` | +2 | 0.80 |
| `leaf-for-live` | +2 | 0.80 |
| `leaf-idxload` | +2 | 0.80 |
| `leaf-forever` | +3 | 0.80 |
| `leaf-for-break` | +3 | 0.80 |
| **`leaf-ptrwalk`** | **+3** | **0.55** — the contested row (`#3091`, `#3126`, `#3148`) |
| `leaf-for2` | +4 | 0.75 |
| `leaf-fornest` | +4 | 0.75 |
| `leaf-goto-back` | +1 | 0.85 |

**Aggregate, registered as a ceiling with no discount:** rows where measured ≠
published — **point estimate 1, ceiling 4 of 17**.

### 4b. The bridge cell — the pivotal control

`?HashString`'s exact body (`fixtures/cpp/whash_ptr_walk_loop.cpp`) is the **one
leaf loop in this repo whose charge has been graded by the oracle**: `w-fenceb`
settled it at **2** with three mutants red and a separating control green, and
the shipped `label_lead` emits it.

| # | claim | P |
|---|---|---:|
| **B1** | `lead.py` on `[HashString, z9]` reproduces **exactly 2** — an independent re-derivation of the one oracle-graded number | **0.90** |
| **B2** | `HashString`'s body run through **§4.2.1's own instrument** (`a0 · P · a1 · a2`) reads `stride 3`, i.e. surcharge **+2** — so the two instruments agree and §4.2.1 carries **no systematic offset** | 0.70 |
| **B3** | if B2 misses at `stride 4` / surcharge +3, then **§4.2.1's whole column is one high as a rule** and the `leaf-ptrwalk` row is a symptom, not the disease | 0.25 |
| **B4** | `leaf-ptrwalk` (§4.2.1's row) and `HashString` (the shipped class) are **different shapes** and may legitimately charge differently; the row is settled by measuring it, not by inheriting `#3091` | — stated, not scored |

**B1 is a falsifier as well as a claim.** If this lane's instrument cannot
reproduce the one number the oracle has already settled, every other row it
prints is suspect and the lane reports `FAILED` rather than a table.

### 4c. The series

| # | claim | P |
|---|---|---:|
| **S1** | every row fits `L(n) = k·n` with `c = 0` — no per-TU constant, these being `int` probes with no `_fltused` | 0.70 |
| **S2** | at least one row has `c ≠ 0` | 0.30 |
| **S3** | every row's `k` equals its `n = 1` lead (i.e. reading one cell would have been right here, unlike `w-slots`) | 0.65 |
| **S4** | residual of the fit is 0 on every row | 0.75 |

### 4d. `work/w-bdnz/LABEL_LEAD.md` — the table quoted into shipped code

Its instrument is **cross-TU** (*"two TUs differ in exactly one function body"*),
which is exactly the construction `#3148` refuted. Its `lab_goto` row is
`?HashString`'s pointer walk and reads **+8** where the oracle says **2**.

| # | claim | P |
|---|---|---:|
| **W1** | `lab_goto` re-differenced seed-free reads **2**, not 8 — so the table is a counter-gap artifact of the same kind as `#3148` | **0.85** |
| **W2** | every one of its 8 rows moves, and the gap is exactly each cell's own `.gl` counter gap | 0.75 |
| **W3** | `lab_loop`'s **`+7`** — the number quoted into `IlFunction::label_slots`' shipped doc comment as reading 1 where §4.2.1 says `+1` — re-reads at **≤ 4** | 0.80 |
| **W4** | §4.2.1's `for` row and `lab_loop` are **different bodies**, so "the table says +1, the obj says +7" was never a like-for-like comparison even before the artifact | 0.70 |

### 4e. Reach — what a corrected number would unblock. **STATED AND NOT TAKEN.**

This lane settles numbers. It lifts nothing. `#3147`'s standard for a lift is a
**closed recognizer PLUS a series**, and a recognizer is a `crates/` change this
lane is forbidden. Every "this gets cheaper" line in the report is a claim about
a *fence's price*, with the fence named, and is followed by a full stop.

## 5. Falsifiers, registered in advance

| # | falsifier | what it would mean |
|---|---|---|
| **F1** | any row's in-TU anchor control (`first(a2) − first(a1) ≠ base`) fails | `gt_label_stride` is invalid on that row; the row is dropped, not fitted |
| **F2** | `lead.py`'s zero-controls (`leafnone`, `straight`) do not read exactly **0** | the base formula is wrong in this tree; the lane reports `FAILED` |
| **F3** | **B1 misses** — the instrument cannot reproduce the oracle-graded 2 | the lane reports `FAILED`; a table nobody can validate is worse than no table |
| **F4** | the two instruments disagree on some rows by a **non-constant** amount | one carries a shape-dependent term; **report both columns, pick neither** |
| **F5** | any byte under `crates/` changes | this is a docs + `work/` lane; a non-zero byte delta is a `FAILED` construct rung |
| **F6** | fewer than **10** of the 17 rows produce a *discriminating* series (a series is discriminating iff `L(n)` is not constant in `n`) | the grid is vacuous — a table of "0 disagree" nothing could have disagreed with |

## 6. Absence is not success — the counters that must be printed

Every instrument in this lane prints, and the report quotes:

1. **`discriminating cells`** — rows whose series varies with `n`. Rows published
   at `+0` (`leaf-none`, `leaf-if`) are **structurally non-discriminating** for
   the per-TU/per-function split and are counted separately, loudly.
2. **`controls held`** — the `a2`/`a1` anchor pair per row, and `lead.py`'s two
   zero-controls.
3. **`reds`** — for each row, the count of neighbouring charges `k−1`, `k+1` (and
   `0`) whose predicted `$M` triple **disagrees with the reference obj's bytes**,
   against a **separating control** (the same body in a leaf-only TU, which mints
   no labels — board #742 — and is green under every charge).
4. **`rows where measured ≠ published`**, with the direction of each.

**On the strength of the mutants.** `w-fenceb`/`w-slots` graded their mutants by
routing a wrong charge through the shipped emitter and reading `c2rs gap`. That
is available only for a class the port **emits**, and of these 17 rows at most
one is. For the rest the mutation is on the **predicted `$M`** and the judge is
the reference obj's own symbol-table bytes — the same six bytes, one layer
earlier. **This lane states that as a weaker construction rather than claiming
the stronger one**, and runs the stronger one wherever a row maps to a shipped
class.

## 7. Registered outcome numbers — CEILING, no discount

| quantity | population | registered | note |
|---|---|---|---|
| `mismatch` | every gate lane, sweep, cross, 878-TU scan | **0** | |
| fixture-gate `match` | 381 × 18 | **+0** | no `crates/` change |
| `c2rs perf` `Match` | 381 at `/Ox` | **+0** | |
| workload `match` | 878 dc3 TUs | **25 → 25** | |
| `codegen-gap` / `vocab-gap` / `capture-fail` / `frontier` | 878-TU scan | **0 / 845 / 8 / 2** | |
| `gap-metric` keys | — | **372** | |
| verdict lines | — | **878** | |
| `fnbyte-exact` | 878-TU scan | **35734** | |
| workspace tests | — | **1610 passed, 0 failed, 42 targets** | unchanged; no test added |
| `graded tree` | `crates fixtures scripts` | **`04e3500f07b7`, 730 files**, both ends | **F5** |
| census | — | **+0** | |
| new fixtures | — | **0** | a fixture would move the graded tree |
| board rows | — | **UNNUMBERED**; next free is `#3151`, coordinator serializes | |

## 8. Reproduction

```sh
work/w-labeltable/table.py            # the 17 rows, the series, both instruments
work/w-labeltable/table.py --bdnz     # the 8 cross-TU rows, re-differenced
work/w-labeltable/table.py --framed   # §4's 6 framed rows
```
