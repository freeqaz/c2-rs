# w-mmioclose — PREREG

Frozen **before the first change to `crates/`** and before the first fixture
this lane authors. Everything in §0 is this lane's own scan or its own compiled
cells; nothing is quoted from `docs/STATUS.md`, which is board **#2360**'s trap
and which `w-ifn` walked into inside the document that opened by saying it would
not.

## 0. The base, re-derived

`work/w-mmioclose/base.out` / `base.jsonl`, one 878-TU scan.

    provenance   c2-rs 981efd7f5335 (clean)   binary 04b4962558edb693
    WORKLOAD     dc3-decomp 1e0215e753c2 (clean)      <- the stamp
    wibo 1.2.0-c2rs.1 · cl.exe/c2.dll/c1xx.dll 16.00.11886.00

| row | this lane's base scan |
|---|---:|
| TU **match** | **19** |
| mismatch · codegen-gap · port-error | 0 · 0 · 0 |
| vocab-gap · capture-fail | 852 · 7 |
| **`fnbyte-exact`** | **35,776** |
| `fnbyte-differs` | **1,879** |
| `fnbyte-denominator` | **162,092** |
| `fnbyte-refused` | 114,685 |
| `fnbyte-reloc-differs` | 532 |
| `fnbyte-census-disagree` / `-expressible` | 1,003 / **0** |
| `fnbyte-spliced` · `-spliced-exact` | see §0.1 |
| `gap-metric` keys | counted at scan time, §5 |

**The workload has NOT advanced since `w-ifn`.** `fnbyte-denominator` reads
162,092 at both ends and `fnbyte-exact` 35,776 is `w-ifn`'s tip to the unit, so
board **#2392**'s hazard did not fire on this lane. The stamp is registered
anyway, because the point of #2360 is that a level is only meaningful beside the
tree it was taken at.

### 0.1 The target TU, re-derived from its own row

`base.jsonl`, `src/xdk/nuispeech/mmio.cpp`:

    class = vocab-gap    fn_total = 11    fn_in_class = 10
    detail = ".ex 5153 B, 1 .gl names — c2_il::functions() and dyninit_tu() both None"

**`1 .gl names`.** That number is registered here, before any code, because it
decides this lane: the commission's premise — *"factors A, B and C already hold,
so function bytes are the entire remaining distance"* and *"what is left is
`mmioClose`'s 124 bytes"* — is a claim about the FBM instrument, and the GATE
that decides TU conversion binds **one** of the eleven names.

## 1. What this lane will try, in order

1. **Answer the architectural question** (`w-ifn`'s C6 / board #139), in code
   rather than in prose.
2. **Decode the `.gl` function record's attribute field** and establish whether
   `__declspec(noinline)` is visible to the port. This is board **#1039**'s
   *undecoded field* and `w-inlfence2` #2155's *missing input*.
3. If it is: **ship it as a narrowing of both inline fences' ACCEPT side**, and
   close the pinned wrong emit `crates/c2-harness/tests/noinline_boundary.rs`
   cell `w10` records (`Differs`, shipped, demonstrated, uncovered by the
   corpus).
4. **Price `mmio.cpp`** against what the gate actually requires, and either
   convert it or decline it at a re-derived N.

## 2. The conversion call

| outcome | p |
|---|---:|
| (A) `match` **19 → 20** — `mmio.cpp` converts | **0.04** |
| **(B)** 19 → 19, `mmioClose` declined, and the FIRST blocker is shown to be the `.gl` name binding rather than codegen | **0.62** |
| (C) 19 → 19, declined for the reasons `w-ifn` gave, nothing added | 0.10 |
| (D) 19 → 19, some other TU converts as a side effect of the fence narrowing | 0.06 |
| (E) something else | 0.18 |

| # | p | call |
|---|---:|---|
| T1 | **0.03** | `mmioClose` ships byte-exact |
| T2 | **0.04** | the TU converts |
| T3 | **0.88** | `mmio.cpp`'s first gate blocker is the `.gl` NAME binding (10 of 11 names carry no `@@`), **not** `mmioClose`'s bytes |
| T4 | **0.90** | the six `w-ifn` named are re-derived and **at least one more** is added that `w-ifn` did not count |
| T5 | **0.75** | `w-ifn`'s C6 ("there is nowhere in the port to ask a sibling-body question") is **refuted** by `IlBundle::functions`' own existing clauses |

## 3. The `fnbyte-exact` delta — the calibrated metric

The conversion is called at 0.04, so this lane's `fnbyte-exact` movement comes
from the fence narrowing, not from a new class.

| # | p | call |
|---|---:|---|
| F1 | 0.34 | **+0** exactly |
| **F2** | **0.40** | **+1 … +8** |
| F3 | 0.16 | +9 … +40 |
| F4 | 0.06 | > +40 |
| F5 | 0.04 | **negative** — a byte-exact function lost |
| F6 | 0.80 | `fnbyte-differs` moves by **0 or downward**, never up |
| F7 | 0.95 | `mismatch` **0** everywhere, both modes, all lanes |
| F8 | 0.70 | the per-function and emitted censuses move by **0** — this lane ships no reader clause and no lowering |
| F9 | 0.60 | `fnbyte-decline-inlined-callee` **falls** (the fence stops over-refusing) |

Point estimate **+2**.

## 4. The attribute field

| # | p | call |
|---|---:|---|
| **G1** | **0.90** | the field separating a `noinline` record from a plain one is a **single bit** |
| **G2** | **0.70** | it is **bit 6 (0x40)** — `WB_INLINE_FINDINGS` §1 read *"requires bit 6 of `[sym+0x4c]`"* off c2's own disassembly, and this would be that field arriving from the other side |
| G3 | 0.65 | `inline` and `__forceinline` move a **different** bit from `noinline` |
| G4 | 0.80 | the bit reproduces on `mmio.cpp`'s own `.gl` with the dc3 source as ground truth, on **2 of 11** records |
| G5 | 0.55 | `static` (internal linkage) moves a third bit, distinct from both |

## 5. Neutrality

| # | p | call |
|---|---:|---|
| U1 | 0.80 | ≤ 1 TU arrives, **0** depart, **0** into `mismatch`/`codegen-gap`, compared over all 878 **by name** |
| U2 | 0.70 | 0 `gap-metric` keys vanish; ≤ 2 appear and each is this lane's own counter |
| U3 | 0.85 | no fixture moves but this lane's, list regenerated **after** the last fixture and `wc -l`-checked, at `/O1` **and** `/Ox` |
| U4 | 0.90 | `c2rs selftest` stays green |
| U5 | 0.60 | the first-blocker maps move on **0** keys (no reader clause ships) |
| U6 | 0.95 | `board_audit.sh` 0/0/0/0/0 and `rung_registry` passes |

## 6. The test-count DELTA

**Three consecutive lanes have over-estimated this in the same direction**
(`w-bdnz` +16→+12, `w-blockir` +16→+10, `w-ifn` +22→+7), and `w-ifn` §10.6
diagnosed the mechanism: a boundary asserted by CELLS is not asserted by unit
tests. This lane's boundary is a **grid**, so it calibrates down harder than the
correction alone would suggest.

| # | p | call |
|---|---:|---|
| **N1** | **0.46** | **+1 … +10** |
| N2 | 0.30 | +11 … +18 |
| N3 | 0.14 | +19 … +30 |
| N4 | 0.10 | outside all |

Point estimate **+7**.

## 7. Decline clauses, with sizes

* **D1 — `mmioClose` is NOT attempted unless the gate blocker is paid first.**
  Writing a reader and an emitter for a body whose TU cannot bind is byte work
  with no grade behind it, which is `D5`'s shape one level up. Size: the reader
  alone is `w-ifn`'s 570 lines plus three statement forms it does not have.
* **D2 — the attribute bit ships as a NARROWING of an accept side and never as
  a widening of one.** Concretely: `None` (the bit unreadable) must leave every
  consumer's behaviour **byte-identical** to today. If any consumer needs
  `None` to mean "assume inlinable", the clause fires and the bit is published
  and not shipped. Size: 1,004 currently-declined functions are the population
  a wrong widening would mis-emit.
* **D3 — the `.gl` NAME binding (`looks_mangled` / `INLINE_NAME_MAX`) is NOT
  widened.** Board **#1721** already declined it with its reason — a TU with no
  mangled name anywhere comes back with `Bindings::unclaimed` EMPTY, so
  `IlBundle::functions`' unclaimed-symbol gate goes **vacuous rather than
  satisfied** — and `mmio.cpp` is exactly such a TU. Doing it here would trade a
  refusal for a fail-open gate on the one TU this lane is measuring. Size: 10 of
  11 names on the target TU; two of them (`mmioSeek`, `mmioRead`) are exactly 8
  characters and hit `INLINE_NAME_MAX` as well.
* **D4 — a `mismatch` anywhere ⇒ revert to the last committed known-good tree**
  (board **#1380**: commit BEFORE any revert).
* **D5 — no bytes without a grade.** Every cell that changes emitted bytes is
  graded by real `c2` at `/O1` **and** `/Ox`.
* **D6 — `PORT_CFG_CLASSES` is not widened.**
* **D7 — one unnamed refusal is budgeted.** Everything else must be named here
  or declined.
* **D8 — fence order: the new clause goes LAST in its arm**, and every `_neg`
  cell's clause key is read off a scratch print rather than predicted, with a
  must-fail mutation for each. `_neg` cells have been confounded or inert in
  five of the last seven lanes.

## 8. Pre-armed places this is expected to go wrong

1. **`fnbyte-census-disagree-expressible` must stay 0.** `w-inlfence2` P26 named
   this site and it fired there. A narrowing that un-declines functions moves the
   disagreement the other way.
2. **`splice` losing a byte-exact function.** 723 spliced, all exact today. If
   the bit refuses one of those 723, F5 fires and D2 requires the revert.
3. **The stride/lead channel.** This lane ships no new emitted class, so
   `plan_labels` should not move at all; if it does, the change is not what this
   PREREG describes.
4. **A `None` that is not the status quo.** The one way D2 can be violated
   quietly is a consumer that reads `Option<bool>` and treats `None` as `false`.
