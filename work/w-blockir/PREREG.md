# W-BLOCKIR — PREREG

    Lane:   w-blockir
    Branch: wt-w-blockir, off master `2b9f7ffd` (the w-main merge)
    Board:  #2300–#2329
    Frozen: 2026-08-09, **before the first change to `crates/`** and before the
            first fixture this lane authors.

Everything below was written with these things in hand and **nothing else**:
`docs/STATUS.md`; `docs/rungs/2026-08-09-w-band.md`; `2026-08-09-w-readpx.md`;
`2026-08-09-w-bdnz.md`; the prior art on `mmio.cpp` and on `IPP_basicmath_xbox.cpp`
quoted from `docs/BOARD.md` / `docs/rungs/` / `docs/whitebox/WB_LOOP_FINDINGS.md`;
one 878-TU scan at base (`work/w-blockir/base.out`); the reference objs and
disassembly of the two target TUs (`work/w-blockir/ref/`); and the `/O1` and
`/Ox` disassembly of a self-contained reproducer of `IPP_basicmath_xbox.cpp`
(`work/w-blockir/probe/ipp.cpp`, `ipp_o1`, `ipp_ox`).

**No probe beyond those four objs has been compiled, and no `crates/` file has
been read for the purpose of changing it.**

---

## §0 — the base, re-derived rather than inherited

Nine inherited prices were wrong this week, so every number here is from a
command run in **this** tree at `2b9f7ffd` (`work/w-blockir/base.out`).

| key | value at base |
|---|---:|
| `gap-metric match` | **18** |
| `gap-metric mismatch` | 0 |
| `gap-metric frontier` | **9** |
| `gap-metric factor-a / -b / -c` | 28 / 338 / 169 |
| `gap-metric fnbyte-exact` | **36,228** |
| `gap-metric fnbyte-differs` | 1,879 |
| `gap-metric fnbyte-denominator` | 178,977 |
| `gap-metric progress-emitted-in-class` | 39,643 |
| per-function census | 712,237 / 2,463,443 |
| `gap-metric` keys | **256 lines** |
| workspace tests (release) | to be re-run at base; registered as a **DELTA** below (#1749) |

The two target TUs at base, from the scan's own FRONTIER blocks:

    3 blocked | 11 emitted | src/xdk/nuispeech/mmio.cpp                  | labels 9   | 64/380 bytes 16.8%
    4 blocked |  4 emitted | src/system/synth_xbox/IPP_basicmath_xbox.cpp | label-free |  0/184 bytes  0.0%

All seven blocking bodies re-read at base: 4 × `cflow-loop` / `expr-cmp-eq` on
IPP, 3 × (`cflow-if-2`, `cflow-if-n`, `cflow-if-n`) / `expr-cmp-eq` on mmio.

---

## §1 — THE SCOPE CALL, MADE BEFORE THE WORK

**The two TUs need different things and this lane takes ONE of them.** The
commission licenses exactly this (*"If the two TUs need different things, take
the one that converts a TU and price the other"*).

* **TAKEN: `src/system/synth_xbox/IPP_basicmath_xbox.cpp`.** Four leaf bodies,
  no frame, no `.pdata`, no relocation, **label-free**, 4/4 decoded end to end.
  The whole TU is 184 `.text` bytes.
* **DECLINED: `src/xdk/nuispeech/mmio.cpp`**, by clause **D3** below. Its three
  bodies are framed, carry a materialised common epilogue (board #506), a
  `memcpy` expansion-cost model nobody has built (#1925), an indirect
  `mtctr`/`bctrl`, a second `cr0` compare regime, an **elided call** (the source
  calls `mmioSetBuffer(hmmio,0,0,0)` and the obj carries no branch for it), and
  a label charge of 9. It has been priced and declined by eight lanes; this lane
  re-derives the price at base and does not attempt it.

### 1.1 What is to be built, in port terms

A **narrow float array-walk counted loop**, `/O1` only, drawn around the three
sub-shapes the four IPP bodies occupy — the same standard as
`codegen::ptr_walk_loop`, `codegen::static_scan_loop` and
`codegen::json_utf8_copy`, all of which are one-function transcriptions and all
of which read **1.000** byte-exact (`w-readpx` §5.2). It is **not** the block IR
`docs/CFG_SHAPE.md` §6 specifies: no fixup list, no liveness across a block
boundary, no scheduler, no register allocator. The back edge's displacement is
computed in the emitter, which is the escape hatch every shipped loop class has
used (`codegen/labels.rs` invariant 4 is untouched).

The three sub-shapes, read off the reference obj at base:

    A  two arrays, compound assign      Add_InPlace, Mul_InPlace   48 B
       mr r11,<w> · cmplwi cr6,r3,0 · bclr 12,26 · mtctr r3 · sub r10,<o>,<w>
       lfsx f0,r10,r11 · lfs f13,0(r11) · f<OP>s f0,f0,f13 · stfs f0,0(r11)
       addi r11,r11,4 · bdnz .-20 · blr
    B  one array + scalar, compound     MulConstant_InPlace        36 B
       cmplwi cr6,r3,0 · bclr 12,26 · addi r11,<w>,-4 · mtctr r3
       lfs f0,4(r11) · fmuls f0,f0,f1 · stfsu f0,4(r11) · bdnz .-12 · blr
    C  three arrays, plain assign       Mul                        52 B
       cmplwi cr6,r3,0 · bclr 12,26 · mr r11,<w> · mtctr r3
       sub r10,<o1>,<w> · sub r9,<o2>,<w>
       lfsx f0,r10,r11 · lfs f13,0(r11) · f<OP>s f0,f0,f13 · stfsx f0,r9,r11
       addi r11,r11,4 · bdnz .-20 · blr

---

## §2 — THE CONVERSION CALL, PER TU, IN PROBABILITY FORM

A mutually exclusive and exhaustive distribution over the TU-match delta
(#2158's requirement), registered before the first `crates/` line:

| outcome | p |
|---|---:|
| **(A)** `match` **18 → 19** — IPP converts, mmio does not | **0.50** |
| (B) `match` 18 → 18 — IPP declines too, at a named refusal | 0.44 |
| (C) `match` 18 → 20 — both convert | 0.01 |
| (D) `match` 18 → 19 by a TU **other than** IPP | 0.02 |
| (E) something else, including a `match` that goes DOWN | 0.03 |

Per TU, stated separately so the pair can be scored:

| # | p | call |
|---|---:|---|
| **T1** | **0.50** | `src/system/synth_xbox/IPP_basicmath_xbox.cpp` converts |
| **T2** | **0.02** | `src/xdk/nuispeech/mmio.cpp` converts |

`T1` is under 0.5-ish rather than high because **all four bodies must convert
together** — the TU is 4 emitted / 4 blocked, so a single unfenced word anywhere
converts nothing. Four independent sub-goals at ~0.85 apiece is ~0.52.

---

## §3 — THE `fnbyte-exact` DELTA (the calibrated metric, #2292 / CEILING §10)

**Registered as a delta on the fixed denominator 178,977, not as a census
delta.** A census-only prediction is unscored.

| # | p | call |
|---|---:|---|
| **F1** | **0.50** | `fnbyte-exact` **36,228 → 36,232** (exactly **+4**) |
| F2 | 0.30 | `fnbyte-exact` +0 (the class ships nothing, or ships and converts none) |
| F3 | 0.15 | `fnbyte-exact` **+5 … +40** — the class reaches bodies outside IPP and they are exact |
| F4 | 0.05 | `fnbyte-exact` **> +40** or **< 0** |
| **F5** | **0.85** | `fnbyte-differs` **1,879 → 1,879** — this class moves **no** function the wrong way |
| **F6** | **0.90** | `mismatch` **0 → 0** everywhere, on every gate row |

**F7 (p = 0.75)** — the **inlined-callee check does not apply to this class**,
because all four bodies are **leaf** (zero call edges, zero relocations in the
whole 184 B of `.text`). The bimodality `w-readpx` §5.2 measured — five
call-bearing classes at 0.000 over 1,106 functions — cannot reach a class with
no call in it. Registered as a prediction because the *reader* could still admit
a call-bearing body elsewhere: if it does, F7 is a MISS and clause **D7** fires.

---

## §4 — THE TEST-COUNT DELTA (#1749: a DELTA, never a total)

| # | p | call |
|---|---:|---|
| **N1** | **0.45** | workspace tests move by **+12 … +20** |
| N2 | 0.30 | by **+21 … +34** |
| N3 | 0.15 | by **+1 … +11** |
| N4 | 0.10 | outside all of the above |

Registered point estimate: **+16**. (`w-bdnz`, the closest shipped precedent,
was +12 against a registered +16.)

---

## §5 — THE TWO MECHANISMS I HAVE NOT MEASURED, CALLED IN ADVANCE

`WB_LOOP_FINDINGS.md` §4.3 states the base-difference rule and then says of the
walker: *"In all five measured cells the walker is the array whose access is
emitted last, which is circular. `#1767`'s rule against a two-point fit applies;
not claimed."* So this is genuinely open and the calls below are made **before**
the probes that decide them.

### 5.1 Walker selection — which array gets `r11`

| # | p | call |
|---|---:|---|
| **W1** | **0.55** | The walker is the base of the **LAST array LOAD in evaluation order** (RHS operands left to right, then — for a compound assign — the LHS's own load). Decisive probes: `f3[i] = f2[i] * f1[i]` walks **f1**; `f1[i] += f2[i]` walks **f1**. |
| W2 | 0.20 | The walker is the **store destination whenever it is also loaded**, and the last RHS array otherwise (agrees with W1 on all four base cells, disagrees on `f1[i] += f2[i]`) |
| W3 | 0.15 | Neither; the walker tracks a register-number or formal-slot rule |
| W4 | 0.10 | The probes disagree with every rule I can state in ≤ 4 cells |

**W5 (p = 0.70)** — whichever wins, the class ships the walker as a
**per-sub-shape transcription with the rule NAMED and its witness count stated**,
not as a derived allocator. #1767's bar (a two-point fit is refused) is the
reason.

### 5.2 The park's position relative to the guard

Read at base: shape A puts `mr r11,r5` **before** `cmplwi`; shape C puts it
**after** `bclr`; shape B has no `mr` at all and its `addi r11,r4,-4` is after
`bclr`.

| # | p | call |
|---|---:|---|
| **P1** | **0.35** | the park floats above the guard **iff** the walker arrives in the **last** GPR formal register |
| P2 | 0.30 | I will not separate a rule from a coincidence in ≤ 6 probes, and the position ships as a per-sub-shape constant |
| P3 | 0.20 | the rule is about the **count** of preheader instructions |
| P4 | 0.15 | something else |

---

## §6 — THE MODE CALL

**M1 (p = 0.95)** — the class is **`/O1` only**. Measured at base:
`/Ox` unrolls `Add_InPlace` 4× with a `cmpwi cr6,r3,4` pre-test, a remainder
loop, `lfsu`, and a 688 B single `.text` — a body this class must **refuse**,
not approximate.

**M2 (p = 0.80)** — the fixture is graded at `/Ox` as well as `/O1` and reads
`vocab-gap` (an honest refusal) there, not `match` and not `mismatch`.

---

## §7 — THE LABEL LEAD (measured against the obj by counterfactual, never quoted from the table)

`docs/LABEL_COUNTER.md`'s published surcharges have been measured wrong by three
lanes and are mode-dependent. This lane measures its own.

| # | p | call |
|---|---:|---|
| **L1** | **0.75** | `IlFunction::label_slots` must return `None` for this shape (the `w-bdnz` #1983 outcome, for the same reason: the charge is mode-dependent and `label_slots` has no mode parameter) |
| L2 | 0.70 | the measured lead over a `leaf-none` control is **≥ +4** at `/O1`, where `LABEL_COUNTER.md` §4.2.1's `for` row read literally predicts **+1** |
| L3 | 0.60 | the `/O1` and `/Ox` leads **differ** |

**L4 (p = 0.85)** — because IPP is `label-free` (2 of 9 frontier TUs are; the
scan prints it), a wrong label charge **cannot** be what blocks this TU. It can
still block the *fixture* if the fixture pairs the class with a framed function,
which is `whash_loop_then_framed.cpp`'s shape and is why a `_neg` cell exists.

---

## §8 — NEUTRALITY, REGISTERED AS A PREDICTION AND NOT AS A HOPE

| # | p | call |
|---|---:|---|
| **U1** | **0.60** | per-function census moves by **exactly +4** (712,237 → 712,241) |
| U2 | 0.75 | emitted census moves by **+4 … +12** |
| **U3** | **0.80** | the per-TU verdict SET over all 878, compared **BY NAME**, moves in **one** direction only: 0 TUs leave `match`, ≤ 2 arrive |
| **U4** | **0.70** | **0** of the 256 `gap-metric` keys vanish and 0 appear (values may move) |
| U5 | 0.65 | `frontier` **9 → 8** |
| **U6** | **0.85** | every one of the 318+ fixtures at `/O1` **and** `/Ox` keeps its verdict except the ones this lane authors; the list is regenerated **after** the last fixture and `wc -l`-checked |

---

## §9 — DECLINE CLAUSES, EACH WITH A SIZE

Any one of these firing ends the build; the lane then reports a **priced
decline** with N named per TU, which is an acceptable outcome and an unfenced
widening is not.

* **D1 — the update form.** Sub-shape **B** (`MulConstant_InPlace`) needs
  `stfsu` with an `addi r11,<w>,-4` pre-bias. `wb-loop` §4.4/§7.5 put four
  update-form rivals on a frozen ten-cell grid and **elected none**; `w-bdnz`
  declined pass 3 by name. **If shape B cannot be fenced without electing one of
  those four rivals, shape B is not built** — and because IPP is 4-of-4, that
  costs the whole TU. Size if it fires: **1 of 4 bodies, 36 of 184 bytes, and
  the TU.**
* **D2 — the walker rule.** If §5.1's probes leave the rule a two-point fit, the
  class ships **three transcribed sub-shapes** and says so; it does **not** ship
  a derived walker-selection rule. Size: 0 (this is a labelling discipline, not
  a build stop).
* **D3 — mmio.** `src/xdk/nuispeech/mmio.cpp` is **not attempted**. Its price is
  re-derived at base and reported as N per body. Size: **3 bodies, 316 of 380
  bytes, 1 TU.**
* **D4 — a mismatch anywhere.** Any `mismatch` on any gate row, any fixture, any
  sweep row: **revert**, and per board **#1380 commit the known-good state
  BEFORE the revert**, then report.
* **D5 — no bytes without a grade.** No `crates/` change ships without a fixture
  graded against real `c2.dll` at `/O1` **and** `/Ox`.
* **D6 — refuse in the reader, never bend the emitter.** A body that would need
  a different word must be refused by the **reader**. If the emitter grows a
  branch to accommodate a body, that is D6 and the body leaves the class.
* **D7 — the inlined-callee fence.** If the reader admits any **call-bearing**
  body, the class is narrowed until it does not, before anything ships.
  `framed-call` is 0-for-123 and `call-sequence-cmp-eq` 0-for-542 for exactly
  this reason.
* **D8 — one unnamed refusal is budgeted.** Exactly one refusal not on this list
  may be met and worked through. A second unnamed refusal ends the build and the
  lane reports the decline.

---

## §10 — PRE-ARMED INSTRUMENTS

The failure mode this lane is most likely to commit is **fence order / clause
reachability**, which has fired in four of the last six lanes. Pre-armed:

1. **FENCE ORDER.** The new production is placed so that **no body any
   production above it accepts today can move, by construction**, and that claim
   is checked by a per-key census counterfactual over all 878 TUs — 256
   `gap-metric` keys and both first-blocker maps compared as **key→value maps**,
   never as a `diff`. IPP's bodies block at `expr-cmp-eq` inside the
   `disp-expr-load` arm (base scan: `19,295 x disp-expr-load|BLOCKED|expr-cmp-eq`),
   so that arm's ceiling is registered here as a **ceiling and not a size**:
   granting it wholesale would be 19,295 bodies, and this class is a strict and
   much smaller subset. Five lanes dispatched off a blocked-key ranking found the
   ranking was an artifact; I assume mine is.
2. **CLAUSE REACHABILITY.** Every `_neg` fixture cell must have a **distinct,
   probe-verified clause key**, and each cell's base verdict is a
   **counterfactual against a binary built at master** — confounded cells have
   been found in five lanes running.
3. **#1380.** Commit before any revert.
4. **The commission's own calibration.** Seven one-function transcriptions
   bought +7 exact and +7 TU conversions; a 444-wide admission bought +0 and +0.
   This lane is on the transcription side of that line **by construction**, and
   if at any point it stops being, that is D6.
