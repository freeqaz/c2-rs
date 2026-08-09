# w-memfit — PRE-REGISTRATION

    Lane:    w-memfit
    Base:    master `8dd1a577`
    Branch:  worktree `worktree-agent-a797863184ba241db`
    Rows:    #2060–#2079
    Written: 2026-08-09, **before the first score, the first `cl.exe` and the
             first read of any `measured.json` payload beyond one example cell
             of each grid** (structure only: field names, not verdicts).

The commission has two deliverables and only the first is unconditional:

1. **Reconcile** `w-memcpy` (black-box, "no rule fits", best rival 114/232 for
   the id-keyed rule and 182/232 for the best threshold, one unanimous
   sub-class refuted at 114/176) with `wb-memcpy` (whitebox, a decision
   function READ out of `c2.dll`: `align = max(1, BYTE[node+0x38])`,
   non-constant size ⇒ CALL, `n = size/align` truncating, `T = 5` / `10` under
   favor-speed, `n <= T` ⇒ INLINE; plus E-DEADDST for the elimination) — by
   scoring the read function **cell by cell against w-memcpy's own frozen
   cells, on w-memcpy's own denominators**.
2. **Conditional**: convert `src/xdk/nuispeech/mmio.cpp` if a rule survives at
   a rate that would make an emit *correct* rather than plausible.

---

## 1. The denominator, stated before it is scored

w-memcpy froze **1,155** cells in three grids:

| grid | cells | question |
|---|---:|---|
| GRID-L | 747 | where a **literal argument** is materialised at a call site (R-DESC et al.) |
| GRID-M | 232 | the memcpy/memset **expansion decision**, four pointer types |
| GRID-M2 | 176 | the same, four *other* pointer types + an operand-kind axis |

**D0 (registered as a structural claim, to be checked and not assumed).**
`wb-memcpy`'s reading is a statement about the block-move expansion only. It is
therefore **defined on GRID-M's 232 and GRID-M2's 176 — 408 cells — and
undefined on GRID-L's 747**, which contain no intrinsic at all. wb §3.1/Q7
registers that decline in advance. I predict **p = 0.95** that a mechanical
check finds **zero** `memcpy`/`memset`/block-assign cells in GRID-L, and I will
report the check rather than the prediction. If GRID-L cells *do* contain an
intrinsic, the reading is scored on them too and the headline denominator
becomes 1,155.

**The headline number will be quoted on 408**, with 232 and 176 separately, so
it is directly comparable to `114/232`, `182/232` and `114/176`.

---

## 2. The reconciliation — registered predictions

`R-WB` is the reading as written in `WB_MEMCPY_FINDINGS.md` §2, made total over
the three-valued verdict (`none` / `inline` / `call`) exactly as w-memcpy's own
corrected verdict function is:

```
  align = max(1, alignment hint of the pointee type)
  if the destination is a dead, non-escaping local  -> none      (E-DEADDST)
  if size == 0                                      -> none
  if the size is not a compile-time constant        -> call
  n = size / align            (truncating)
  T = 5 at /O1  (10 under favor-speed; both grids are /O1 only)
  n <= T  -> inline   else -> call
```

| # | registered | p |
|---|---|---:|
| **P1** | `R-WB` **beats every frozen rival on both grids** — strictly more than 182/232 and strictly more than 114/176 | **0.93** |
| **P2** | `R-WB` scores **≥ 228/232** on GRID-M | 0.80 |
| **P3** | `R-WB` scores **≥ 172/176** on GRID-M2 | 0.75 |
| **P4** | `R-WB` scores **232/232 and 176/176 — perfect on all 408** | **0.55** |
| **P5** | **The axis w-memcpy's rule space lacked is the DIVISOR, not favor-speed.** Every one of its six GRID-M rivals is a predicate on `size` (or on the id, or on constancy); none divides by the alignment hint. Favor-speed **cannot** explain a single miss on these 408 cells, because both grids compiled at the workload's `/O1` only, where `T = 5` in the reading and 5 is what w-memcpy measured. I register that varying favor-speed changes **zero** of the 408 predictions | **0.90** |
| **P6** | w-memcpy's **second** missing axis is *destination liveness* — GRID-M2's 44 `ll` cells — and it is **orthogonal to the divisor**: `R-WB` **without** the E-DEADDST arm still beats 182/232 on GRID-M (which has no `ll` axis) but loses ≥ 40 cells on GRID-M2 | 0.85 |
| **P7** | The verdict is **outcome (a)** of the three the commission names: the reading explains the cells, and w-memcpy's "no rule" was a **rule-space** limitation. **No `W-MEMCPY-*` address is RETRACTED** | **0.85** |
| **P8** | A **rival that is not the reading** and not one of w-memcpy's six — `n <= 5` with `align` taken from the **C type's natural alignment** rather than from the IL hint byte — is **indistinguishable from `R-WB` on all 408 cells**, because every cell's pointee is a natural-alignment type. So these cells **cannot** discriminate "the divisor is the IL hint byte" from "the divisor is the source type's alignment", and the commission's third outcome (the cells cannot discriminate) holds **for that sub-claim** even if P4 lands | 0.80 |
| **P9** | Manufacturing a cell that *does* discriminate P8's two readings costs ≤ 1 hour and ≤ 12 cells (`#pragma pack`, or an over-aligned/under-aligned pointer cast, so the IL hint byte and the C type's alignment disagree) — and I will manufacture it rather than only naming it | 0.70 |

**Decline clauses for deliverable 1 — registered with sizes.**

* If `R-WB` scores **< 182/232 or < 114/176** it has failed to beat the black
  box on the black box's own cells: outcome (b). I then **RETRACT
  `W-MEMCPY-1`** by address per method doc §7 and correct wb-memcpy's row —
  the reading is "read correctly, not what c2 does".
* If `R-WB` beats the rivals but scores **< 408/408**, I publish the residue
  **cell by cell**, and the port's predicate is the **confident core** (§3),
  never the whole rule.
* If the 408 cannot separate `R-WB` from a rival I write down (P8), I say so
  and **manufacture the separating cells** — the toolchain is available.

---

## 3. The confident core, registered before it is measured

**A wrong emit is strictly worse than a gap** (board #232, 241 commits). So
whatever `R-WB` scores, the port's predicate is not `R-WB`; it is the subset of
`R-WB` on which the measured exactness is **1.000 with no residue at all**,
with every other case refusing by name.

| # | registered | p |
|---|---|---:|
| **P10** | The confident core I will state is **narrower than `R-WB`** on at least two axes (favor-speed unmeasured in these grids ⇒ the core is `/O1`-only or reads the option word; destination liveness unmodelled in the port's IL ⇒ the core refuses any cell whose destination could be a dead local) | 0.90 |
| **P11** | The core's measured exactness on the 408 is **1.000** by construction — I will not state a core with a known counterexample | 0.95 |

---

## 4. Deliverable 2 — the conversion call

`src/xdk/nuispeech/mmio.cpp` is the frontier's top byte-fraction row, **64 of
380 bytes accepted**, `8/11` functions in class, `?mmioGetInfo` **84 B**
remaining. `w-park` priced mmio's chain at **12** across three bodies at three
different first refusals (`callseq-tail-lit`, `call-token-0xB9`,
`call-token-0x26`); `w-memcpy` priced `?mmioGetInfo`'s memcpy clause alone at
**5** independent refusals with `call-arg-lit-permuted` in front of all of
them. **Inherited prices have been wrong six times this week, so both are
re-derived at base before any call is made.**

| # | registered | p |
|---|---|---:|
| **P12** | `mmio.cpp` **does not convert this lane** — TU match unchanged, `mmio.cpp` bytes accepted unchanged at 64/380 | **0.90** |
| **P13** | The re-derived price of `?mmioGetInfo` alone is **≥ 4 independent refusals**, and **≥ 3 of them are outside the expansion rule** (the mint, the argument-slot reduction, `call-arg-lit-permuted`) — i.e. even a *perfect* expansion rule converts **zero bytes** of this function | **0.88** |
| **P14** | The re-derived price of the whole TU is **≥ 8**, and `w-park`'s 12 is within ±4 of it | 0.70 |
| **P15** | The census/bytes numbers at base match `w-memcpy`'s recorded 64/380 and 8/11 **to the byte** | 0.85 |

**Decline clauses for deliverable 2 — registered with sizes.**

* **A priced decline is acceptable and is the registered expectation.** If
  `?mmioGetInfo` re-derives at ≥ 4 independent refusals, or if ≥ 1 refusal is
  the callee **mint** (a symbol with no `.gl` token, which `bundle::resolve`
  structurally cannot produce), the conversion is **declined with N named** and
  the lane ships the reconciliation only.
* If a `crates/` change is made at all, it ships **only** the confident core,
  behind the existing mode gates **in the parser** (#1638), acceptance **in the
  parser** (#139), with fences in **both** directions and a probe-verified
  distinct clause key per `_neg` cell.
* If the only shippable thing is a **module doc** recording the rule, that is
  what ships, and it needs no `DISCLOSURE` row.

---

## 5. Test-count DELTA and neutrality, registered

| # | registered | p |
|---|---|---:|
| **P16** | `#[test]` bodies under `crates/` move by **+0** (decline path) or **+4 … +12** (core-ships path). I register **+0** as the modal outcome | 0.75 |
| **P17** | **Three-level verdict neutrality holds**: 878 TUs by name (0 only-in-base, 0 only-in-tip, 0 changed), every `gap-metric` key accounted, and all 312 fixtures at **/O1 AND /Ox** | 0.92 |
| **P18** | Full gate at the shipping tree: **18/18 lanes, 0 mismatch**, `cargo test --workspace --release` green, `board_audit.sh` and `rung_registry` clean | 0.90 |
| **P19** | The frontier re-survey at tip is **identical** to base | 0.92 |

## 6. DISCLOSURE, registered in advance

`WB_MEMCPY_FINDINGS.md` §9 pre-drafts `W-MEMCPY-1` (adoption: the predicate and
its two constants) and `W-MEMCPY-2`/`-3` (route). **This lane registers, in
advance, that it prefers the black-box derivation**: if the same predicate is
derivable from the 408 frozen cells alone — and P1–P4 are exactly the claim
that it is — then **no row is carried**, and the rung says so explicitly.

| # | registered | p |
|---|---|---:|
| **P20** | **No `W-MEMCPY-*` row is carried into `crates/`** this lane — either because nothing is adopted (the decline path), or because the predicate is black-box derivable from the 408 and does not need one | **0.85** |
| **P21** | If a row *is* carried it is `W-MEMCPY-1` and only `W-MEMCPY-1`; `-2` and `-3` are not needed by any predicate this lane could ship, and `-4` does not exist | 0.90 |

## 7. Direction — board #770

Registered **PESSIMISTIC on the conversion, OPTIMISTIC on the reconciliation**,
and I state the asymmetry rather than hiding it: #770's streak is ~12
optimistic / 2 pessimistic / 2 hits, and #2031 is the *mirror* — a lane whose
four registered-pessimistic predictions all missed optimistically. So the
pessimistic half of this freeze (P12–P15) is the half most likely to be wrong,
and the way it would be wrong is the conversion turning out **cheaper** than
priced. I have registered P13 at 0.88 rather than 0.95 for that reason.

The optimistic half (P1–P4) is optimistic on a specific and unusual ground:
**the reading has already been graded at 180/180 and 36/36 against real
`c2.dll` on 216 cells it was not fitted to** (GRID-W). This is not a lane
trusting a disassembly; it is a lane checking whether a rule that already
passed one held-out grid also explains an *older* grid that predates it.

## 8. One unnamed refusal

Budgeted: **one**. If a second unnamed refusal appears, the lane stops and
reports it rather than absorbing it.

## 9. Pre-armed traps

* **FENCE ORDER / clause reachability** (streak 9/13): every `_neg` fixture
  must be probe-verified to die on *its own* clause key and not on an earlier
  one. `w-bdnz` found **two confounded `_neg` cells**, and a confounded cell
  passes the fixture gate exactly like a correct one.
* **Signedness** (#1788): `__alldiv` is **signed** 64-bit and the size is a
  64-bit constant. Any port arithmetic is signed, and the sign is asserted.
* **Vanishing tests** (#1710a): `#[test]` bodies counted by `git grep -c` at
  both commits, never by a runner's `ok`.
* **#1638 / #139**: mode gates in the parser, acceptance in the parser.
* **Ranking artifacts** (memory: four for four): no lane is dispatched off a
  blocked-key size ranking in this rung.
* **Check the board before dispatching**: any row I propose is grepped against
  `BOARD.md` first.
