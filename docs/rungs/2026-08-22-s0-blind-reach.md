# w-s0 — the blind-reach instrument: the unmeasurable half is measured, and it is 0.342 % reached

    Tag:       w-s0
    Slug:      s0-blind-reach
    Date:      2026-08-22
    Kind:      construct
    Outcome:   instrument
    Fixtures:  none — construct rung: the S0 blind-reach instrument (`gap::blind`)
               + a named relaxation parameter on the census (`c2_il::Relax`)
    Census:    +0 (no acceptance predicate moved; `IlBundle::functions` untouched)
    Base:      master `9b9530791`
    Prereg:    `docs/rungs/_2026-08-22-w-s0-prereg.md`, the FIRST commit on `wt-w-s0` (`7bf821252`)
    Board:     #3392–#3396
    Spec:      `docs/ROADMAP_SLICING_2026-08-21.md` §5 row S0
    Funded by: `docs/DECISIONS_2026-08-22.md` decision 5 (owner — branch (c), the 8-week Phase 0)

---

## 1. Result

**The population `gap/factors.rs:267` calls "the unmeasurable half" is measured.**
`c2_core::codegen::select_function` had never been called on any of the 113,557
parse-refused functions in the dc3 workload; it has now been called on all of
them, and the output byte-compared against real c2's own COMDAT bytes.

One full 878-TU scan, ladder depth 1, at this branch's tip
(`work/w-s0/tip_scan.log`):

| key | value | of |
|---|---:|---|
| `fnbyte-blind-attempted` | **113,557** | — the denominator, and it **equals `fnbyte-refused-parse` exactly** |
| `fnbyte-blind-exact` | **15** | 0.013 % of 113,557 · **3.9 % of the 388 reached** |
| `fnbyte-blind-differs` | **373** | 0.329 % of 113,557 · **96.1 % of the 388 reached** |
| `fnbyte-blind-unlowerable` | **113,169** | 99.66 % of 113,557 |
| — `no-decode` | 113,165 | the relaxed decode produced nothing |
| — `no-compose` | 4 | selected, and the `/Gy` composition declined |
| — `no-select` · `no-refbytes` | 0 · 0 | |
| **REACH** | **388 of 113,557 = 0.342 %** | the sub-population actually delivered to the lowering |

**Controls, all at their known answer of 0**: `fnbyte-blind-partition-broken` 0
· `fnbyte-blind-population-broken` 0 · `fnbyte-blind-census-desync` 0.

**The required-zero identity holds.** 482 `gap-metric` lines, **0 differ**
against the base scan at `9b9530791`; the 16 added lines are all
`fnbyte-blind-*`. `match` **26** · `mismatch` **0** · `fnbyte-exact` **35,891**
· `fnbyte-differs` **1,963** · `fnbyte-reloc-differs` **530** — every one
unmoved.

### 1.1 The two sharp numbers

**(a) The differing bodies are not near-misses.** Over the 373 `blind-differs`:
port **1,120** words, reference **3,727** words, **equal words: 4**. The port's
relaxed bodies average 3.0 words against c2's 10.0, and across all 373 bodies
exactly **four words** agree anywhere. This is not a lowering that is close; it
is a lowering answering a different question. It corroborates
`docs/DIFF_STRUCTURE.md`'s shape (0 pure reorderings, 94.3 % wrong at word 0)
from a population that doc has never seen.

**(b) Inside the reach the outcome is bimodal by refusal class**, and the
instrument prints the classes rather than leaving them to be inferred:

| strict reader's refusal class | exact | differs | exact rate |
|---|---:|---:|---:|
| `data-sym-not-extern:eof` | **15** | 4 | **78.9 %** (15/19) |
| `data-sym-unresolved:eof` | 0 | 369 | **0.0 %** (0/369) |

**Read this with §2 before reading it as anything.** Two classes is not a
sample of the 594+ distinct blocking features in the population, and the
bimodality is a fact about these two classes only.

---

## 2. WHAT THIS DOES NOT SAY — the prereg's §6 applied, and it binds

`ROADMAP_SLICING_2026-08-21.md` §5 registers three readings of S0. **This lane
declines all three**, under the rule frozen in its own prereg §6 *before* any
blind number existed:

> A small `blind-exact` at a shallow ladder depth is **NOT** §5's "`S0 ≈ 0` →
> the halves are not separable, 15–45 months is optimistic" reading. That
> outcome is a statement about **a general decode**. This lane's ladder is
> explicitly not one: it relaxes an admission gate and leaves all ten of §3's
> constructs unimplemented. If reach < 5 %, **the instrument has not asked §5's
> question yet.**

Reach is **0.342 %**, so P3 held and the clause fires. Concretely:

* **`blind-exact 15` is not evidence that the catalogue does not generalise.**
  99.66 % of the population never reached the lowering at all — `no-decode`
  113,165 — so for those functions the lowering **was never asked**. An `exact`
  rate computed against 113,557 would be a ratio whose denominator is not the
  population that was tested, which is the exact defect
  `calc_fuzzy_match_percent` has and FBM was built to avoid.
* **The reached sub-population contains exactly TWO refusal classes**, both
  `data-sym-*`. It contains **none** of the ten constructs of §3 — no `off-add`
  (22,310), no `intrinsic` (≈ 20,000 across its rows), no `bind` (2,211), no
  `load-type`, no `call-in-expr` (≈ 14,000). A frozen holdout is only as good as
  the classes it happens to contain, and this one contains two.
* **Nothing here re-prices 4a, 4b, or the 15–45 month figure**, in either
  direction.

### 2.1 What IS licensed, at this depth, and it is the sharpest of the three

Prereg §6 clause 4: `blind-differs` is a statement about **the bodies actually
graded**, so it is readable at whatever depth is reached, with its denominator.

> **Of the 388 parse-refused functions the relaxed decode delivered to the
> lowering, 373 (96.1 %) are byte-wrong and 15 (3.9 %) are byte-exact.**

That is a **two-sided price** in the units `ROADMAP_SLICING` §6 rule 2 requires,
on a population that had none: a `functions()` widening that admitted this
sub-population on the strength of the decode alone would ship **373 wrong
emits to buy 15 right ones**, and under `docs/PROGRESS_METRIC.md` §5.2 each
wrong emit scores strictly below the refusal it replaced. The direction agrees
with board **#3363**'s 2,490-wrong measurement on the *accepted* side, and it is
now measured on the refused side too.

**This is not a fence and it licenses nothing.** It is a price, published so
that the next widening is argued against a number instead of an intuition.

---

## 3. Estimate vs outcome — all six predictions scored

Frozen in `_2026-08-22-w-s0-prereg.md` §4 before the instrument existed.

| # | prediction | P | outcome | |
|---|---|---:|---|---|
| P1 | `attempted == 113,557` and `population-broken == 0` | 0.85 | 113,557 / 0 | **HIT** |
| P2 | L0 reproduces `exact 0 · differs 0 · unlowerable == attempted`, all `no-decode` | 0.90 | exactly, on 878 TUs | **HIT** |
| P3 | ladder reach **< 5 %** of 113,557 | 0.80 | **0.342 %** — 14.6× under | **HIT** |
| P4 | `blind-differs` > `blind-exact` among the reached | 0.75 | 373 vs 15 (24.9:1) | **HIT** |
| P5 | `blind-exact > 0` | 0.60 | **15** | **HIT** |
| P6 | required-zero identity holds | 0.95 | 0 of 482 lines | **HIT** |

**Six for six, and the registered bias was correct too**: the prereg said in
advance that P3 holding would make P4 and P5 weak evidence, and §2 above is that
warning being honoured rather than discovered. **The prediction that was
genuinely at risk was P5 at 0.60** — whether the catalogue would reach even one
function it was never admitted to. It reached fifteen.

**Registered bias for the next lane**: six-for-six is a signal that the
thresholds were set too safely, not that the lane was insightful. P3's threshold
(5 %) was 14.6× looser than the outcome; a future ladder-depth lane should
register a *tighter* reach band and be willing to miss it.

---

## 4. What was built

### 4.1 `c2_il::Relax` — the relaxation as a NAMED, ENUMERABLE parameter

`crates/c2-il/src/func/census.rs`. `census_functions()` becomes
`census_functions_relaxed(Relax::STRICT)`; the public signature every existing
reader calls is **untouched**, which is the whole warrant for having widened a
shared predicate at all.

Per `docs/GOAL_DECISION_2026-08-21.md` § "AMENDED" and `ROADMAP_SLICING` §6 rule
7, it is a **struct of independent switches, not an integer scale** — an integer
cannot say *which* gate a measurement crossed, and the one thing this instrument
must report is which refusal classes its reached sub-population contains
(§1.1(b) is that requirement paying off immediately). Selected at runtime by
`C2RS_BLIND_LEVEL`; an unparseable value is **refused loudly** rather than
silently defaulted.

| level | name | relaxes |
|---:|---|---|
| 0 | `strict` | nothing — the identity control |
| 1 | `name-from-gl` | the post-parse **symbol-resolution** gates: a placeholder is supplied where a callee or data symbol did not resolve through `.gl` |

**Level 1 changes no instruction byte**, and that is why it is sound: under
`/Gy` a call word carries a placeholder displacement and a data address is an
`addis`/`addi` pair of zero immediates, so the name lives in the **relocation
record** and not in the word (lane `w-drop3`, boards #984–#989). The consequence
is written at the seam rather than left to be inferred:

> **Blind grades BYTES ONLY and publishes no relocation verdict.**
> `fnbyte-blind-exact` requires bytes alone; `fnbyte-exact` requires bytes **and**
> relocation identity. **They are not the same predicate and must never be
> summed.** A unit test pins the namespace.

### 4.2 `gap::blind` — the instrument

`crates/c2-harness/src/gap/blind.rs`, run from the same denominator-driven walk
FBM uses, immediately after it, over the parse-refused subset only. It calls
`c2_core::comdat::comdat_function_body` — **called, never copied**, for
`fnbytes.rs:98`'s reason, which is sharper here than there: an instrument that
is confident about bytes the port does not emit is worse than the blind spot it
replaced.

It adopts all five of `docs/FUNCTION_BYTE_MATCH.md` §0's properties verbatim —
§0's own 2026-08-22 banner names S0 as one of the two extensions on the books.
**Never in `scripts/gate.sh`; licenses no emit.**

### 4.3 The controls, and why they are shaped this way

* **`population-broken`** compares `attempted` against the *sibling FBM walk's*
  `fnbyte-refused-parse`, computed in the same TU iteration that filed it —
  never by subtracting two published totals (#1464's rule). This is the check
  that says the instrument is pointed at the population it claims, and it is the
  difference between a measurement and a coincidence.
* **`partition-broken`** — the three buckets sum to `attempted`.
* **`census-desync`** — the strict and relaxed passes are two walks over one
  segmentation; row *i* must be the same function in both. Checked, not assumed
  (#918's shape).
* **The identity control is the instrument itself.** At `C2RS_BLIND_LEVEL=0`
  the relaxed census *is* the strict census, so the required answer is
  `exact 0 · differs 0 · unlowerable == attempted`, all `no-decode`. Measured on
  the full workload: **attempted 113,557 == `fnbyte-refused-parse` 113,557,
  unlowerable 113,557 all `no-decode`, all three controls 0.**

  Both halves matter and the second is the one instruments here usually skip:
  `the_relaxation_is_actually_wired_to_something` asserts level 1 really flips
  the switch. A pin that only shows `STRICT` is inert is equally consistent with
  the parameter being wired to nothing at all — which is the shape of the eight
  instruments on this project that reported green from an absence.

---

## 5. Found and not taken

### 5.1 A defect in this lane's own printed block — an absence that read as a zero

The bucket is filed per TU under a **pipe** key
(`fnbyte-blind-unlowerable|no-decode`) and republished as a flat **dash**
metric. The first version of the printed block looked the *metric* spelling up
in the *per-TU* map, found nothing, and printed:

```
no-decode          0     ← on a scan whose own gap-metric line read 113165
ladder depth: 0 (strict — THE IDENTITY CONTROL)   ← on a scan that ran at level 1
```

Both lines were wrong in the direction this project fails most often, and the
second would have **mislabelled the central measurement of this lane as its own
control**. Nothing detected it but the two outputs of one scan disagreeing.

The repair is not the four edited lookups — it is the **locator**.
`Why::emit_key()` / `Why::metric_key()` are now the single source of both
spellings, and `the_per_tu_key_and_the_metric_key_agree_on_the_reason` pins them
equal up to the separator. `docs/GAPS.md` §6: one fact, one locator.

### 5.2 Three filed figures were wrong on this tree, and one of them is LIVE for a peer lane

Re-measured at base `9b9530791` before anything was registered against them
(prereg §1):

| key | filed | measured here |
|---|---:|---:|
| `fnbyte-exact` | 35,894 (slicing §5, the dispatch brief) · 35,886 (`STATUS.md`, tree `977827d78`) | **35,891** |
| `fnbyte-refused-parse` | 113,565 (`w-restim` tip scan) | **113,557** |
| `fnbyte-differs` | 1,960 / 1,958 | **1,963** |

**The cause is named, and it is not a defect**: the workload tree moved. This
scan's provenance reads workload `e5aef017d456`; `w-restim`'s reads
`2f666acc8aa2`; `STATUS.md`'s reads `49ad7cfd5`. Three dc3-decomp states, three
function populations. The denominator is c2's own output and is *allowed* to
move.

> **What is live, and it is not this lane's to fix — board #3396.** Board
> **#3391** and `DECISIONS_2026-08-22.md` decision 5 both register a
> **program-stop condition** for lane `w-s1` in these words: *"S1's delta holds
> but workload `fnbyte-exact 35,894` moves at all"* → the byte-exactness is a
> fit, the pricing basis is void, **and the program should stop.** On this tree
> that number is **35,891**. A lane that measured 35,891, compared it to the
> registered 35,894 and applied the rule as written would **trigger the
> program-stop condition on a workload-tree difference.** The stop condition
> needs re-anchoring to a number measured on the tree the lane runs on, together
> with its workload stamp — which is `ROADMAP_SLICING` §6 rule 4 (*publish the
> denominator in the sentence that states a null*) one level up. Routed to the
> coordinator, not edited here.

### 5.3 Not attempted, and named so absence is not read as coverage

* **A general decode.** The ten constructs C1–C10 are Phase 1 slices, each
  priced 2–4 wk raw on its own. A ladder level for any of them is a Phase 1
  slice wearing an instrument's clothes, and pricing one inside S0 would repeat
  the error §5's encoder row corrects (three jobs priced as one).
* **A `formals-width` ladder level** (drafted in prereg §3 as level 2, not
  built). It needs a flag threaded through `parse_segment_detail` /
  `parse_segment_shape` in `crates/c2-il/src/func/body/mod.rs`. Contained, but
  unbuilt and therefore unmeasured — its reach is **unknown**, not zero.
* **Any relocation verdict.** Structurally out of scope at level 1 (§4.1).
* **`crates/c2-core/src/codegen/`** (peer lane `w-s1`) and
  `crates/c2-core/src/coff/label.rs` (peer lane `w-seedgap`) — untouched. No
  collision occurred; the relaxed decode did not force an edit into either.

---

## 6. Gate evidence

| lane | result |
|---|---|
| `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast` | see §6.1 |
| `scripts/gate.sh --jobs 4` | see §6.1 |
| 878-TU workload scan | `match` 26 · `mismatch` 0 · identity 0 of 482 lines |
| `#[test]` count, merge-base → tip | see §6.1 |

*(filled from the tip run; see §6.1)*
