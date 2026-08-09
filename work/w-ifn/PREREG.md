# W-IFN — PREREG, frozen before the first `crates/` change

**Lane** `w-ifn`, branch `wt-w-ifn`, off master **`42fe7cb1`**.
**Frozen at** the commit that adds this file, which is **before** the first edit
to anything under `crates/` and **before** the first fixture this lane authors.
Everything already committed at `029e1fae` is *measurement*: five probe grids,
`work/w-ifn/base.out` (an 878-TU scan at the base binary), the label-lead
counterfactual and the reference obj. No `crates/` file has been touched.

**Commission**: extend the CFG class from `w-blockir`'s loop shape to the branch
shapes and convert `src/xdk/nuispeech/mmio.cpp` — `cflow-if-2` ×1 and
`cflow-if-n` ×2, **316 of the TU's 380 `.text` bytes**, the frontier's top
byte-fraction row, `ABC--` so function bytes are the whole remaining distance.

---

## 0. §0 — the base, re-derived rather than inherited

Nine inherited prices were wrong this week and the commission says ten. Nothing
in this section comes from a rung, a board row or the commission. Every number
was produced by a command in this tree at `42fe7cb1`
(`work/w-ifn/base.out`, `work/w-ifn/base.jsonl`, run before this file existed).

| | base `42fe7cb1` |
|---|---:|
| TU match | **19** |
| mismatch · codegen-gap · port-error | 0 · 0 · 0 |
| vocab-gap · capture-fail | 852 · 7 |
| FRONTIER | **8** |
| `fnbyte-exact` · `fnbyte-differs` | **36,232** · 1,879 |
| function census · emitted census | 712,241 · 39,647 |
| `gap-metric` keys | **255** (a bare `grep -c` gets 256) |
| mmio: blocked / emitted | **3 / 11** |
| mmio: accepted / total `.text` bytes | **64 / 380** (16.8 %), remain **316** |
| mmio `frontier-codegen` row | den 11 · exact 8 · wrong 0 · cg-ref 0 · **reader 3** |

**`w-blockir`'s §2 correction is CONFIRMED on this tree**: `grep -cE '^ *gap-metric
[^ ]+ '` is **255** and a bare `grep -c gap-metric` is **256**, the extra hit
being the prose line at `base.out:1024` that mentions
`gap-metric fnbyte-census-disagree-*`. This lane quotes **255** and compares the
two ends with a key→value map (`work/w-ifn/metricdiff.py`), never a `diff` and
never a count.

**The class's ceiling, registered in advance and AS A CEILING.** All three
blocked bodies block at census key `expr-cmp-eq`, which is the same key
`w-blockir`'s four bodies blocked at and which stood at **19,295** bodies before
that lane and **19,291** after. Seven lanes have now been dispatched off a
blocked-key size ranking and found the ranking an artifact; this is the eighth
to register the ceiling in advance. **Registered expectation: the key moves by
exactly the number of bodies this lane admits, i.e. ≤ 3 + this lane's own
fixtures, three and a half orders of magnitude below the key's size.**

---

## 1. The re-derived price of `mmio.cpp` — and which of `w-blockir`'s eleven are PAID

`w-blockir` priced this TU at **eleven distinct unbuilt mechanisms** in board
#1418's unit ("distinct clauses the port names"). That count was taken by
reading the reference obj; **it was not taken against the port's own source**,
and this lane's re-derivation is. Full table with evidence in
`work/w-ifn/MMIO_PRICE.md`; the summary the PREREG is frozen on:

| # | `w-blockir`'s mechanism | this lane's re-derivation |
|---:|---|---|
| 1 | a framed prologue/epilogue at a 96-byte frame | **PAID** — `codegen::frame::FrameLayout` at `saved_gprs: 0` and `1` emits these exact words and three shipped classes already use them |
| 2 | the materialised common epilogue (a forward join) | **PAID AS A PRIMITIVE, UNPAID AS A PRODUCTION** — `Selected::Framed` indeed has no join, but `guard_chain_shared_tail`, `osf_handle_guard`, `alloc_init_or_fail` and `if_call_join` all emit one. What is missing is a *production*, not a mechanism |
| 3 | forward conditional branches on `cr6` in a framed body | **PAID** — `encode_bc(BO_FALSE, cr_bi(6, CR_BIT_EQ), disp)`, used by every class in row 2 |
| 4 | the `.pdata` flag word, computed | **PAID** — `coff::pdata::pdata_record` computes it from `Frame{prolog_len, func_len}`; it reproduces all three of mmio's words (`40001503`, `40001b04`, `40001f04`) arithmetically |
| 5 | the coalesced two-register park | **UNPAID** — `calls::SeqPark` exists but no production carries a two-register park; probe `p5` shows the words |
| 6 | `memcpy`'s expansion cost model | **RE-DERIVED AND MUCH CHEAPER THAN ITS NAME** — at `/O1 /Oi` the boundary is a step: `n ≤ 5` expands, `n ≥ 6` calls, 25 witnesses, `work/w-ifn/probe/mcpy.cpp`. What is unpaid is a **reader** for the `40`-selector-172 intrinsic, not a cost model |
| 7 | a callee-saved GPR live across a call | **PAID** — `guard_chain_shared_tail` parks `params[0]` in r31 across two `bl`s at `saved_gprs: 1` |
| 8 | a second relational regime (`cmplw` on two loaded values, `bf 24`) | **PAID AT THE ENCODER, UNPAID AT THE READER** — `encode_cmplw` and `CR_BIT_GT`/`CR_BIT_LT` exist |
| 9 | an indirect call (`lwz`/`mtctr`/`bctrl`) | **HALF PAID** — `encode_mtctr` exists; `bctrl` has no encoder and no `Selected` shape reaches an indirect call |
| 10 | `cr0` compares beside `cr6` ones in one body | **PAID** — `guard_chain_shared_tail` reads `r < 0` on cr0 and `r != S` on cr6 in one body |
| 11 | an ELIDED CALL | **EXPLAINED, UNPAID** — §1.1 |
| **12** | *(not counted by `w-blockir`)* the `mmioClose` park into **r5, a volatile, across a `bl`** | **UNPAID, and it is an INTERPROCEDURAL CLOBBER fact** — probe `park.cpp` p1/p2/p4 |
| **13** | *(not counted)* the compiler-label charge | **PAID AND MEASURED AT ZERO COST** — `work/w-ifn/LABEL_LEAD.md`: stride 5 `/Gy`, 4 packed, i.e. exactly the framed constant `plan_labels` already advances |

**Re-derived total: of `w-blockir`'s eleven, SIX are paid outright (1, 3, 4, 7,
10, and 8's encoder half), one is paid as a primitive and unpaid as a production
(2), one is much cheaper than its name (6), and four are genuinely unbuilt (5,
9, 11, and the uncounted 12).** The rung will state this as its own count and
will not claim it refutes `w-blockir`, whose unit was the obj and not the port.

### 1.1 Mechanism 11, the elided call — ANSWERED before the PREREG was frozen

The call **is in the IL** (`work/w-ifn/probe/mmio_ex.txt` decodes mmioClose's
`.ex` segment; the `26 <tok> BD 86 41 12 00 80 12 10 00 00` is at line 60), so
c2 deleted it and c1xx did not. Ten cells at `/O1` and again at `/Ob0`
(`work/w-ifn/probe/elide.cpp`) give the rule:

> **A call whose RESULT IS UNUSED and whose callee is defined in this TU with a
> body that has NO SIDE EFFECT is deleted.** Not `noinline` (e5 elides without
> it), not "constant body" (e6 is `return a+1` and elides), not declaration
> order (e8 elides with the callee below), not tail position (e9 elides mid-body).
> A store (e2), an external callee (e3), a callee that calls an external (e7)
> and a used result (e4) all keep the call. **Every verdict is identical at
> `/Ob0`**, so this is `elide.rs` mechanism **E**'s family and not the inliner.

It is strictly wider than shipped E, which requires the callee to *reduce to
nothing*: e6's callee emits `addi r3,r3,1 ; blr` and the caller still drops the
call.

---

## 2. What this lane intends to ship

**One reader production and one emitter module, carrying three sub-shapes** —
`w-blockir`'s A/B/C form, and for its reason: the three bodies share the frame,
the guard chain, the forward-branch layout and the epilogue, and differ in the
spine. Name: `guard_ret_chain` (`c2_il::GuardRetChain`/`GuardRetChainFn`,
`c2_core::codegen::guard_ret_chain`).

| sub-shape | body | bytes | new mechanisms it needs |
|---|---|---:|---|
| **G** | `mmioGetInfo` — 2 null guards, an intrinsic `memcpy`, `return 0` | 84 | 5 (two-register park), 6 (the `40` reader) |
| **S** | `mmioSetInfo` — 2 null guards, an intrinsic `memcpy`, a member compare + conditional store, `return 0` | 108 | 8 (the reader half) |
| **C** | `mmioClose` — 1 null guard, a same-TU call, a cr0 result test, an indirect member call, a second cr0 test, an ELIDED call, an external void call, `return 0` | 124 | 9 (`bctrl`), 11 (the elision), 12 (the volatile park) |

`/O1` only, asked **first, in the parser** (board #1638) and again in the
emitter. **Accepting these shapes is not a claim about `cflow-if-2` or
`cflow-if-n` as classes** — `PORT_CFG_CLASSES` will not be widened, which would
be the sixth consecutive deliberate exclusion.

---

## 3. THE CONVERSION CALL — in probability form

| outcome | p |
|---|---:|
| **(A)** `match` **19 → 20**: all three bodies byte-exact and `mmio.cpp` converts | **0.30** |
| **(B)** 19 → 19, **two** of the three ship byte-exact (G and S), C declined | **0.34** |
| **(C)** 19 → 19, **one** ships (G) | **0.16** |
| **(D)** 19 → 19, **none** ships — a priced decline with N named | **0.16** |
| **(E)** something else (a different TU converts, or a regression) | **0.04** |

| # | p | call |
|---|---:|---|
| **T1** | **0.80** | `mmioGetInfo` (shape G) ships byte-exact |
| **T2** | **0.64** | `mmioSetInfo` (shape S) ships byte-exact |
| **T3** | **0.32** | `mmioClose` (shape C) ships byte-exact |
| **T4** | 0.30 | the TU converts (= T1∧T2∧T3, and nothing else blocks it) |
| **T5** | 0.85 | **if** all three bodies are byte-exact, nothing ELSE blocks the TU — no `_fltused`-shaped TU-level fact, no section the writer lacks. Registered *because* `w-blockir`'s last blocker was exactly that (board #764, third occurrence) |

## 4. THE `fnbyte-exact` DELTA — the calibrated metric

The commission's calibration: only `fnbyte-exact` and per-TU byte-exactness map
to the goal. This lane's admissions are three workload bodies plus its own
fixtures; fixtures are **not** in the 878-TU population, so the workload delta
is exactly the number of mmio bodies that ship byte-exact.

| # | p | call |
|---|---:|---|
| **F1** | **0.30** | `fnbyte-exact` **+3 exactly** (36,232 → 36,235) |
| **F2** | **0.34** | **+2** |
| **F3** | **0.16** | **+1** |
| **F4** | **0.16** | **+0** |
| **F5** | **0.04** | anything else, including negative |
| **F6** | 0.88 | `fnbyte-differs` **unmoved** at 1,879 — this lane admits no body it does not also emit correctly |
| **F7** | 0.95 | `mismatch` **0** everywhere, at every gate row |
| **F8** | 0.70 | the per-function census moves by **exactly** the same count as `fnbyte-exact` on the workload, plus this lane's fixture bodies |

## 5. The inlined-callee hazard — checked, not asserted

`w-readpx` measured five call-bearing classes at **0.000 over 1,106 emitted
functions** because c2 inlines callees the port keeps as calls; `framed-call` is
0-for-123. **This class is call-bearing**, so unlike `w-blockir`'s the hazard is
NOT structurally absent and the fence is load-bearing.

| # | p | call |
|---|---:|---|
| **H1** | **0.75** | the fence that keeps this class safe is a **callee-side** clause: every call site the class accepts must have a callee that is either **external to the TU** (memcpy, FreeHandle — nothing to inline) or **same-TU and provably left alone**, and shape C's `mmioFlush` edge is the only same-TU one |
| **H2** | 0.60 | shape C's same-TU edge needs the **same** analysis mechanism 12 needs (what does this callee do), so 11 and 12 are one fence and not two |
| **H3** | 0.85 | no cell in this lane produces a `bl` c2 does not emit — checked against the reference obj's own relocation count per COMDAT, never asserted |

## 6. Block order for `if-2`/`if-n` — the commission's deliverable, called in advance

Measured before this file was frozen (`work/w-ifn/probe/blkorder.cpp`, 9 cells),
so these are **results** and are registered as such rather than as predictions:

* **SOURCE ORDER throughout, no sinking.** b6's four-call arm stays where it is
  written; b7 — the same body with the guard's sense inverted — inverts with it.
* **One exception**: a `||`-chained guard's *shared* arm is **sunk** past the
  fall-through and its branch flips `bf` → `bt` (b8). That is
  `guard_chain_shared_tail`'s known shape, reached here independently.
* `wb-loop`'s loop-exit sinking rule does not arise: there is no loop.

| # | p | call |
|---|---:|---|
| **B1** | 0.90 | the three mmio bodies are laid out in **source order** with the epilogue last, and the emitter needs no block-ordering pass — only a running offset |

## 7. The label lead — measured, and the call it licenses

`work/w-ifn/LABEL_LEAD.md`, in `w-json`'s counterfactual form, at both modes,
plus a second independent derivation off the target TU's own obj.

| # | p | call |
|---|---:|---|
| **L1** | **0.85** | `label_lead()` needs **no arm** for this class and `label_slots` needs no arm — the framed stride is the framed constant, 5 under `/Gy` and 4 packed |
| **L2** | 0.75 | the must-fail mutation is therefore the **inverse** of the last four lanes': ADDING a `+1` lead must turn a fixture into a live `Port=Mismatch`, and the `_neg` cell must be ordered so the mutation is **downstream-visible** — `w-blockir` board #2305's cell that could not fail |
| **L3** | 0.60 | `osf_handle_guard`'s shipped `label_lead` of `+1` is **not** explained by "it has an intra-section `b`", because these three bodies have two or three each and charge 0. Recorded as an open discrepancy, not fixed here |

## 8. The test-count DELTA

`cargo test --workspace --release --no-fail-fast`, base measured at the same
tree (#2262: `--no-fail-fast` is mandatory, and the TARGET count is quoted
beside the test count).

| # | p | call |
|---|---:|---|
| **N1** | **0.45** | **+14 … +30** |
| N2 | 0.25 | +31 … +50 |
| N3 | 0.20 | +1 … +13 |
| N4 | 0.10 | outside all |

Registered point estimate **+22**. `w-bdnz` registered +16 and got +12,
`w-blockir` registered +16 and got +10; both over-estimated, and both shipped a
*transcription*. This lane ships three sub-shapes and a wider fence, so the
point estimate is above theirs and the bracket is wide on the low side.

## 9. Neutrality — registered in advance, at three levels

| # | p | call |
|---|---:|---|
| **U1** | 0.80 | the per-TU verdict set over all 878, **by name**: at most **1** arrival, **0** departures, **0** into `mismatch` or `codegen-gap` |
| **U2** | 0.75 | **0** `gap-metric` keys vanish and **0** appear |
| **U3** | 0.85 | no fixture but this lane's own moves, at `/O1` **or** `/Ox`, with the list regenerated **after** the last fixture and `wc -l`-checked |
| **U4** | 0.90 | `c2rs selftest` stays **321 + this lane's fixtures PASS / 0 ERROR** — it was red for part of today and must not be re-redded |
| **U5** | 0.70 | the first-blocker maps move on exactly one key, `expr-cmp-eq`, by exactly the number of admitted bodies |
| **U6** | 0.95 | `scripts/board_audit.sh` 0/0/0/0/0 and `rung_registry` passes |

## 10. Decline clauses, each with its size

| # | clause | size |
|---|---|---|
| **D1** | **A mechanism whose rule cannot be stated from cells is REFUSED IN THE READER, never guessed in the emitter.** In particular mechanism 12: if the volatile-park rule cannot be separated on ≤ 8 cells, shape **C** is declined and the reader refuses it — the emitter grows no arm. | shape C, 124 B |
| **D2** | **The elision (11) is shipped ONLY as a clause of this class's reader**, not as a widening of `elide.rs`. Generalising mechanism E to the purity rule is a rung of its own and this lane will not take it: it would move `fnbyte-exact` over a population this lane has not gridded. | the general mechanism |
| **D3** | **If any body cannot be made byte-exact, it is declined and the other two still ship.** Partial conversion is scored (`fnbyte-exact` is per function), so there is no all-or-nothing. | per body |
| **D4** | **A `mismatch` anywhere at any gate row** ⇒ revert to the last committed known-good tree (board **#1380**: commit BEFORE any revert) and re-derive. | the whole ship |
| **D5** | **No bytes without a grade**: every accepted sub-shape has a fixture, graded at `/O1` **and** `/Ox`. | per sub-shape |
| **D6** | **`PORT_CFG_CLASSES` is not widened.** These are transcriptions of three named functions, not a `cflow-if-n` lowering. | the declaration |
| **D7** | **One unnamed refusal is budgeted.** `w-blockir` spent its on `_fltused`; if a second unnamed TU-level refusal appears, the TU is declined and the refusal is named in the rung rather than chased. | 1 |
| **D8** | **Fence order / clause reachability.** This production goes **LAST** in its `parse_segment_shape` arm unless a probe-verified disjointness argument is written down, and every `_neg` clause key is verified by probe to be the key it claims. Fired in five of the last seven lanes. | the fence |

## 11. What this lane will NOT do

* It will **not** build `docs/CFG_SHAPE.md` §6's block IR. No `Block` type, no
  fixup pass, no terminator enum — the branches are computed from a running
  offset, which is what every shipped framed class does.
* It will **not** widen `elide.rs` (D2), re-price `#2136`'s nine, touch the
  `memcpy` threshold outside the accepted window, or add a row to
  `docs/whitebox/DISCLOSURE.md` unless it actually adopts a whitebox reading —
  every register in the emitter will be read off this class's own objs.
* It will **not** attempt the other seven frontier TUs.
