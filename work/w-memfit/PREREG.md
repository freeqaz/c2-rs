# w-memfit — PRE-REGISTRATION

Committed **before the first scoring run of this lane and before the first
`cl.exe`**. Base `8dd1a577`. Branch `wt-w-memfit`.

The commission is a **reconciliation**: two landed lanes measured the same
decision and published opposite conclusions.

* `w-memcpy` (black box, `docs/rungs/2026-08-08-w-memcpy.md` §6) — *"no rule
  fits"*: `M-ALWAYSCALL` **114/232**, best frozen threshold `M-THRESH-32`
  **182/232**, and the one unanimous sub-class refuted by GRID-M2 at **114/176**.
* `wb-memcpy` (whitebox, `docs/whitebox/WB_MEMCPY_FINDINGS.md` §2) — a decision
  function read out of the binary, graded **180/180** on its own new GRID-W.

## 0. What I have already read, and therefore am not blind to

Honesty about the information state, because a prereg written after reading the
answer is worth less than one written before and pretending otherwise is worse
than either:

* I have read `WB_MEMCPY_FINDINGS.md` in full, including §3's informal table of
  **13 measured boundaries** the reading recomputes, and §5.1's GRID-W scores.
* I have read `w-memcpy`'s rung in full, including §6.1's three per-alignment
  boundaries and §6.2's elimination.
* I have **not** run any per-cell scoring. No score below has been computed. The
  numbers registered here are guesses conditioned on the two documents' prose.
* I have **not** captured any IL, and in particular **the alignment hint for the
  four struct / `void` pointee types in the two grids is unmeasured** (§2 R3).

## 1. The rival under test, stated completely before it is scored

**R-N5** — `WB_MEMCPY_FINDINGS.md` §2's decision function, restated with no free
parameter and with the alignment source made explicit:

```
  verdict(cell) =
     none    if the size operand is the constant 0
     call    if the size operand is not a compile-time constant
     call    if floor(size / align) > T
     inline  otherwise

  align = the front end's alignment hint for the pointee type
  T     = 5   at the dc3 workload's flags (/O1, favor-speed clear)
```

and the composed rule

**R-N5+DEAD** — `none` if the destination is a non-escaping local never read
afterwards (`wb-memcpy` §5.2's `E-DEADDST`), else `R-N5`.

Scored on **the same denominators `w-memcpy` used**: GRID-M's **232** and
GRID-M2's **176** — 408 of the 1,155 frozen cells. The other 747 (GRID-L) carry
**no intrinsic at all** and are not on this question; that is stated here so the
denominator cannot be quietly widened later to flatter a score.

Rescored on the same cells for comparability, exactly as frozen:
`M-ALWAYSCALL`, `M-THRESH-{8,16,32,64}`, `M-VARCALL`, `F-48`, `F-ALL`, plus two
new controls **R-N10** (the same rule with `T = 10`) and **R-SIZE5** (`inline`
iff `size <= 5`, i.e. the same threshold applied to the *size* rather than to
the element count) so that "the quantity is `size/align`" is scored against "the
quantity is `size`" on the grid rather than asserted.

## 2. Registered predictions — probabilities, frozen

| # | registered | P |
|---|---|---:|
| **R1** | `R-N5` scores **232 of 232** on GRID-M, exactly | 0.72 |
| **R1a** | `R-N5` scores **≥ 228 of 232** on GRID-M | 0.93 |
| **R2** | `R-N5+DEAD` scores **176 of 176** on GRID-M2, exactly | 0.62 |
| **R2a** | `R-N5+DEAD` scores **≥ 170 of 176** on GRID-M2 | 0.90 |
| **R2b** | `R-N5` *alone* scores **exactly 132 of 176** on GRID-M2 — every miss is one of the 44 `ll` cells and no other | 0.55 |
| **R3** | the alignment used by the rule is **`alignof(pointee)`**, not `sizeof(pointee)`: measured from the IL hint byte it is **1 / 4 / 8 / 8** for `char` / `int` / `double` / `S16{double;double;}` and **1 / 8 / 4 / 8** for `void` / `long long` / `S4{int;}` / `S32{double[4];}`. GRID-M's manifest records `align = 16` for `S16` and that field is **wrong for this rule** | 0.85 |
| **R4** | `R-N5` **beats every rival `w-memcpy` froze**, on both grids, by ≥ 40 cells on GRID-M | 0.95 |
| **R5** | **R-N10 loses on GRID-M's own cells** (i.e. these already-paid-for cells discriminate `T = 5` from `T = 10` without GRID-W), scoring ≤ 200/232 | 0.88 |
| **R6** | **R-SIZE5 loses badly** (≤ 160/232) — the quantity is the quotient, not the size | 0.90 |
| **R7** | The verdict I will publish is **"the reading explains the cells"**, i.e. NOT a retraction of any `0x10bf65…` address and NOT "the cells cannot discriminate" | 0.88 |
| **R8** | `w-memcpy`'s "no rule fits" is a **rule-space limitation and the missing axis is the DIVISION, not favor-speed**: its grids never varied the optimization flags (`gridm2.py`'s docstring lists optimization as axis D and `build_cells` never crosses it), but that costs it only the value of `T`, and at `/O1` `T = 5` was right. The thing no rival it froze could express is `floor(size/align)` | 0.80 |

### 2.1 Conversion — `src/xdk/nuispeech/mmio.cpp`

| # | registered | P |
|---|---|---:|
| **C1** | `mmio.cpp` **does not convert** this lane | 0.94 |
| **C2** | the re-derived chain at base `8dd1a577` **differs from `w-park`'s 12** in at least one entry (inherited prices have been wrong six times this week) | 0.60 |
| **C3** | `?mmioGetInfo`'s remaining distance at base is still **84 B**, `mmio.cpp` still **64 / 380** bytes accepted | 0.80 |
| **C4** | the decline is priced at **≥ 4 independent refusals** for the intrinsic clause alone, and `call-arg-lit-permuted` still sits in front of all of them | 0.85 |

### 2.2 Ship / neutrality

| # | registered | P |
|---|---|---:|
| **S1** | **no emitter change ships** — nothing this lane writes causes the port to emit a byte it did not emit at base | 0.90 |
| **S2** | `#[test]` bodies under `crates/` move by **DELTA 0** (not a total: the delta) | 0.55 |
| **S2a** | the delta is in **[0, +10]** | 0.92 |
| **S3** | TU match **18 → 18**, mismatch **0 → 0**, codegen-gap **0 → 0**, vocab-gap **853 → 853**, capture-fail **7 → 7**; every `gap-metric` line byte-identical; 0 `fn_blockers` / `emit_blockers` keys moved | 0.92 |
| **S4** | `scripts/gate.sh --require-graded` is **18/18 PASS, 0 mismatch** | 0.95 |

## 3. Decline clauses, with sizes, registered in advance

* **The reconciliation is not optional and has no decline clause.** It is scored
  on already-compiled cells; there is no outcome in which it is not published.
* **If `R-N5` scores below 95 % on either grid** (< 221/232 or < 168/176), the
  reading is **not** carried as a predicate anywhere and the rung's verdict is a
  partial refutation naming the miss class. Nothing is fitted to the misses —
  board **#260**, and `w-mmio` §3's three-fits-two-refutations.
* **If `R-N5` scores 100 % on both grids**, the confident core is still stated
  **separately** from the score, and it is the intersection of the two grids'
  own axes: constant size, a pointee type whose alignment the front end emits as
  1/4/8, favor-speed clear. Anything outside that refuses. A rule right on 95 %
  of cells is a rule that emits wrong bytes on 5 % (board **#232**, 241 commits).
* **The conversion declines** unless, after re-deriving `mmio.cpp`'s chain at
  base with scripts, the residual for **some** `mmio.cpp` function is ≤ 1
  independent refusal *and* that refusal is inside the memcpy expansion. If the
  chain re-derives at ≥ 2, this lane ships **no emitter change** and prices the
  decline with `N` named.
* **`c2rs gap` must be run at both ends** even if no `crates/` line changes, and
  a `mismatch` anywhere is an alarm and stops the lane.

## 4. Direction — board #770

Registered **PESSIMISTIC**, against a base rate of twelve-of-fourteen
optimistic. The argument: this lane's headline number is a *rescoring of cells
somebody else already paid for*, which is the cheapest kind of result there is,
and the expensive half — a conversion — sits behind a chain `w-memcpy` already
priced at five refusals and `w-park` at twelve. The pessimistic bet is that the
reconciliation lands cleanly and moves **zero** bytes of obj, and that the
temptation to be found here is to let a 100 % score on 408 cells read as a
licence to emit.

## 5. What would make this lane wrong in a way I would not notice

Registered because it is the failure this exact shape has: **the committed
`probeM2/measured.json` was written by `gridm2.py`'s TWO-VALUED verdict function
— the one `w-memcpy` §6.2 records as the bug that nearly produced a false
refutation.** It has no `none` arm; all 176 rows read `call` or `inline`. If I
score against that field I will grade 44 eliminated bodies as `inline` and
manufacture a refutation of the reading out of a known-bad label. The corrected
three-valued verdict **must** be recomputed from the recorded `nbytes` (4 ⇒
`none`) before anything is scored, and both scores reported. Board **#984**.
