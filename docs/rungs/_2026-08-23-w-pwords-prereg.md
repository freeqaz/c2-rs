# PREREG — `w-pwords`: can `.pdata`'s `prolog_words` REPAIR S1's demoted tuple↔word bijection?

**Frozen 2026-08-23, as the first commit on branch `wt-w-pwords`, before any
measurement of the bijection was taken on this tree.** Base `f53877aa5`.

Row: `docs/ROADMAP_SLICING_2026-08-21.md` §5 row **S1**, specifically its
**AMENDED 2026-08-21 block** — the amendment that demoted `w-ildecode`'s
`the_final_tuple_order_reproduces_the_text_words` from an *equality* to a
*per-function ratio*. Funded by `docs/DECISIONS_2026-08-22.md` decision 8
(board **#3446**), priced as a follow-up by `docs/rungs/2026-08-23-w-s1bc.md`
§7, unblocked by board **#3431** (`w-read-r6`).

Lane kind: **construct rung** — a measurement seam. `Fixtures: none`.
`Census: +0`. Required-zero byte delta. Reserved rows **#3456–#3459**.

> **THE AMENDED BLOCK BINDS UNTIL THIS LANE'S MEASUREMENT SAYS OTHERWISE.**
> This lane is *testing* the ratio-not-equality demotion, not assuming its
> repeal. A `declined` outcome — "the demotion stands" — is a full result and
> is the outcome this prereg predicts for the general case (§4).

---

## 0. THE FENCE, STATED FIRST

This is a **measurement seam**. It **licenses no emit**, it is **never a
gate**, and it never stands in for the byte judge (`CLAUDE.md` § "The one
correctness rule"). It reads oracle-side facts and confronts a port-side
prediction with them; nothing in `crates/c2-core` may change as a result of it
within this lane.

**Edit fence**: this lane edits `crates/c2-harness` only. `crates/c2-core` is
peer `w-s1c2`'s; `crates/c2-il` and `crates/c2-reference/tests/` (including
`middle_interfaces.rs`, which *hosts the bijection this lane is about*) are
peer `w-4f01`'s. **This lane does not touch `middle_interfaces.rs`** — it
re-creates the decode layer inside the harness instead. If the seam turns out
to require a `c2-core` change, the lane STOPS and reports the crossing point
priced, as `w-seedgap` did with #3404.

---

## 1. What the tree already says, before this lane measures anything

Recorded so a later disagreement is visible rather than silently overwritten.

| quantity | filed value | source |
|---|---|---|
| bijection population | **3 functions, 9 words**, all leaf/frameless | `ROADMAP_SLICING` §5 AMENDED |
| `prolog_words` population | **12,610 framed functions / 6,000 objs** | **#3431**, `ref/P_EXPAND.md` §4.4 |
| `prolog_words` range | 1..7, seven distinct, 100 % ≤ 8 | same |
| prologue **shape** confirmed | **282 records (2.2 %)** | same |
| epilogue term | **UNGRADED** | #3431 caveat 2 |
| unbounded expansion arms | `retaddr`, `nopalign` ×3, `0x2e5` | `ref/P_EXPAND.md` §3 |

### 1.1 R6's citations, CHECKED before use — all three hold

The coordinator's brief warns that #3431's citations were unverified. This lane
verified them **first**, and records the result here because a prereg that
builds on an unchecked number is the failure mode:

- **`crates/c2-core/src/coff/pdata.rs:71`** — read at source. The unwind word is
  `bits 7..0 PrologLen` **in instructions**, `bits 29..8 FuncLen`, bit 30
  `ThirtyTwoBit`, bit 31 `ExceptionFlag`. The "low 8 bits" claim is **correct**,
  and `PrologLen` is already in *words*, needing no division.
- **The 12,610** — `probe_prolog_words.py --limit 6000` re-run on this tree
  reproduces **every digit**: 12,610 records, the same seven-valued histogram
  (3 words 59.22 %, 4 words 23.30 %, 5 words 14.86 %), the same 282-record
  shape sub-population. **#3431 reproduces.**
- **Two facts #3431 did NOT publish**, measured here while checking it:
  the probe's `prolog == 0` / malformed filters drop **exactly 0** records, so
  12,610 is the *complete* record population with no hidden stratum; and
  **3,608 of 6,000 objs (60.1 %) contain no `.pdata` record at all**. The leaf
  stratum is not a footnote — on the obj corpus it is the **majority**.
  (72 of 12,610 records set the EH bit.)

These are python re-runs of a whitebox script, used **only** to check a cited
number. Every figure this lane offers as *its own* evidence runs under
`cargo test` (#1406); see §3.

---

## 2. The question, and why nobody has answered it

The AMENDED block demoted the equality with a **prediction**, not a
measurement:

> Pointed at the corpus it goes **red on the first framed function** …

Its own text concedes the graded population is *"three functions, nine words,
all leaf, **frameless** … precisely the ones that cannot exhibit the
framed-prologue expansion."* **So the equality has never been evaluated on a
single framed function.** The first deliverable of this lane is therefore not
the repair — it is the *baseline*: what does `T == W` actually do on framed
functions? The demotion may be right, and it may also be that the expansion
happens **upstream of the `after0` tap site**, in which case the tuples are
already expanded and the premise is wrong. Both are live; §4 predicts.

### 2.1 The quantities, defined exactly

Per function `f`, all counted at the **`after0`** tap site (site 8, *after the
final schedule*, `stagetap.c` / `stage.rs:66`):

- `T(f)` — number of funcwalk tuples with `is_instruction()` true (flags bit 0,
  i.e. tuple `+0x9` bit 0; R2's invariant, reproduced from the constructor end
  in `ref/P_EXPAND.md` §2).
- `W(f)` — number of 32-bit words in `f`'s `.text` range.
- `P(f)` — `prolog_words`, the low 8 bits of `f`'s `.pdata` unwind word, or
  **`None`** when `f` has no record (the leaf stratum — *absent from the
  denominator, never counted as zero*).
- `I(f)` — number of tuples at `after0` whose opcode ∈ {`0x2f0`, `0x2f4`} **and**
  which are `is_instruction()`. Measured, not assumed: whether the prologue
  pseudo-op carries the real-instruction bit is **an open question this lane
  answers**, and it decides whether the correction is `+P` or `+P−1`.
- `E(f)` — the **epilogue** term. UNGRADED and **not in the obj**. This lane
  does not fit it; it *measures it as a residual* (§2.2 H2).

### 2.2 The three forms tested, in order

**H0 — the baseline, the demoted equality itself:**

    T(f) == W(f)

Run on framed functions for the first time. Publishing its hold-rate on framed
vs. frameless functions **is** the test of the AMENDED block's premise.

**H1 — the prologue correction, epilogue assumed zero:**

    T(f) - I(f) + P(f) == W(f)

This is the form the brief names and the strongest claim `.pdata` alone can
support. If H1 holds broadly, the demotion is **repairable**.

**H2 — the residual, which is where the honest answer lives:**

    R(f) = W(f) - ( T(f) - I(f) + P(f) )

`R` is everything `.pdata` cannot see: the epilogue expansion, `nopalign`,
`0x2e5`, `retaddr`, and the long-branch `bc`. **This lane publishes `R`'s
distribution; it does not fit a constant to it.** The falsifiable readings,
registered now:

- `R` single-valued ⇒ the epilogue term is a **constant** and the equality
  closes with `.pdata` plus that constant.
- `R` an exact function of `P` (e.g. `R = P + c`, prologue/epilogue symmetry)
  ⇒ the equality closes with **the obj field alone**, and the demotion is
  fully repairable.
- `R` scattered and uncorrelated with `P` ⇒ **the demotion STANDS.**

---

## 3. The instrument, and how it will be shown to FAIL

A new harness test, `crates/c2-harness/tests/pwords_bijection.rs`, run under
`cargo test`. It re-creates in the harness only the *decode* layer
(`Tuple`/row parsing, a COFF `.text` symbol slicer, and a new `.pdata` record
reader), and drives the **existing pub** `c2-reference` API:
`capture_reference_with` + `replay_tapped_probe(STAGE_SITES, ops, funcwalk)`.
No `c2-core` edit; no `c2-obj` edit unless §6 records one.

**Deliberately-broken-input check** (`CLAUDE.md`'s formatter rule generalized —
*a fence never seen refusing is not a fence*). Before any number is trusted the
instrument must be **watched to fail**, and the rung must quote it doing so:

1. Perturb `P` by +1 on every framed function ⇒ H1's hold-rate must **collapse**.
2. Corrupt one `.pdata` unwind word ⇒ attribution or hold-rate must move.
3. Drop the `is_instruction()` filter ⇒ H0 must go red where it was green.

If any of those leaves the numbers unmoved, the instrument is measuring
nothing and the lane reports **FAILED**.

**Attribution** of a `.pdata` record to a function is the one real hazard —
`probe_prolog_words.py` punted on it, which is exactly why its shape
sub-population is 2.2 %. This lane resolves it **self-checkingly**: records lie
in `.text` order, and each record carries `FuncLen` in bits 29..8, so the
sequential match is *verified* against each candidate function's own word
count. A TU whose match does not verify is not guessed at — it is counted into
a named `unattributable` stratum.

**Bounds honesty**: the tap truncates at `WALK_MAX 4096` tuples / `BLK_MAX
4096` / `OPS_MAX 128` / `ARENA 4 MiB`, surfaced as `walk_refusals`. Any TU with
a non-empty `walk_refusals` is **excluded and counted**, never silently
floored.

---

## 4. Predictions, registered BEFORE measuring

| # | prediction | scored in the rung |
|---|---|---|
| **P1** | `H0` holds on frameless/leaf functions at **> 90 %** | |
| **P2** | `H0` holds on framed functions at **< 10 %** — the AMENDED block's premise is CORRECT and the expansion is downstream of `after0` | |
| **P3** | `I(f) == 0` on framed functions — the prologue pseudo-op does **not** carry the real-instruction bit, so the correction is `+P`, not `+P−1` | |
| **P4** | `H1` holds on framed functions at **< 25 %** — because the epilogue term is real, nonzero, and not in the obj | |
| **P5** | `R` is **not** single-valued, but **is** concentrated on ≤ 5 values | |
| **P6** | `R` correlates with `P` (prologue/epilogue symmetry), **without** being an exact function of it | |
| **P7** | the graded denominator is **≥ 100 functions** — two orders of magnitude above the bijection's 3 | |

**A prediction that is scored a MISS is a result, not a failure of the lane.**
P2 in particular: if it misses, the AMENDED block is wrong on its own premise
and that is the most valuable thing this lane could find.

---

## 5. Decline criteria, registered in advance

The lane reports `declined` (the demotion stands) or `FAILED` if:

- **`declined`**: `R` is scattered — no single value covers ≥ 50 % and `R` is
  not an exact function of `P` — on every stratum. The demotion stands and the
  ratio remains the right instrument.
- **`FAILED`**: fewer than 40 functions grade; or the tap cannot be driven over
  the fixture corpus; or the broken-input checks of §3 do not move the numbers.
- **STOP and report priced**: any path that requires editing `crates/c2-core`.

**Partial repair is an expected and publishable outcome**: "repairable on
stratum X, not on Y" with X and Y named and counted. The strata published,
fixed now: `leaf` (no record) · `framed-clean` (record, no unbounded-arm
pseudo-op present) · `framed-unbounded` (record + ≥1 of `nopalign` `0x27b`,
`0x2e5`, `retaddr`) · `walk-refused` (bounds hit) · `unattributable`
(record↔function match unverified).

---

## 6. What this lane will NOT do

- **No source-shape grid.** #3431's rung item 6: c2 saved two GPRs regardless of
  source; framing is a consequence of allocation (**#3052**). The corpus is the
  instrument.
- **No edit to `middle_interfaces.rs`** — peer `w-4f01`'s site.
- **No promotion of the bijection to a gate**, whatever the hold-rate. §0.
- **No `c2-obj` edit** unless a `.pdata` reader proves genuinely unwritable in
  the harness, in which case the reason is recorded in the rung.
