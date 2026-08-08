# w-park — PRE-REGISTRATION

**Frozen before the first cell this lane authors and before the first change to
`crates/`.** Everything below was written from base-only instruments: the 878-TU
scan at master `b6783688` (`work/w-park/scan_base.out`), the two reference objs
produced by real `c2.dll` under wibo at the workload's own flags
(`work/w-park/ref/{mmio,biquad}/ref.obj`, disassembled to `dis.txt`), and the
reader-ladder sinks `C2RS_SINK_REL` / `C2RS_SINK_BRANCH`, which are
measurement-only by construction and push no `IlOp`.

Lane: `w-park`, worktree branch `wt-w-park` off master **`b6783688`**.
Commission: ROADMAP §10.26 item 1 — ship M-RULE / B-RULE / B′-RULE toward
`src/xdk/nuispeech/mmio.cpp` and/or `src/system/synth_xbox/Biquad.cpp`.

---

## 0. The base, measured and not quoted

| | value at `b6783688` |
|---|---:|
| TU match | **18** |
| mismatch · codegen-gap · port-error | 0 · 0 · 0 |
| vocab-gap · capture-fail | 853 · 7 |
| FRONTIER | **9** |
| `mmio.cpp` accepted `.text` bytes | **64 / 380** (316 remain), 8/11 emitted fns exact |
| `Biquad.cpp` accepted `.text` bytes | **0 / 176**, 0/2 emitted fns exact |
| workspace tests | to be recorded at §5 |

**The brief's `docs/rungs/2026-08-08-w-json.md` exists at master; the base this
lane was handed (`ea60519f`) did not carry it.** Rebased before anything else.

---

## 1. §1.0 — the refusal chain, RE-DERIVED at this base, against the inherited
## description

**The inherited position (WB_CHOOSER_FINDINGS §8) is that these two TUs are
"blocked by three lowering rules the port does not implement" — M-RULE, B-RULE
(+B-RULE-2) and B′-RULE.** That statement is about *lowering*. It is registered
here, before any probe of the emitter, that **it is not the whole price and is
not the head of either chain**, because the scan says so at base:

* the **first** refusal on 4 of the 5 blocked bodies is `expr-cmp-eq` — a
  **reader** refusal, in `c2-il`, not a chooser;
* with `C2RS_SINK_REL=expr` those 4 move to `expr-brfalse`; with
  `C2RS_SINK_BRANCH=stmt` as well, **mmio's three bodies reach the end of the
  walk** (`expr-rel-sink-poison:mid`, decode complete, acceptance still refused)
  while **Biquad's two do not** — `?SetCoefficients` stops at `expr-op-0x27`
  and `??0Biquad` at `expr-call-in-expr-recv-load-then-plumbing-0x3A`.

So on the reader axis **Biquad is strictly deeper than mmio**, which is the
opposite of what the byte-fraction ranking (`mmio` 16.8 % vs `Biquad` 0.0 %)
suggests, and the opposite of what "Biquad has 2 blocked functions and mmio has
3" suggests. Registered as a correction *before* it is used.

The last four survey prices were all wrong **low** (#1760 "9 vs ≥6", #1782 "one
mechanism was thirteen", w-json's "reader 16 and CLEAR was fourteen", and
wb-chooser's own inherited decline was a mis-copy). **P0.0 (pessimistic): both
chains below will price at ≥ 10, and the published three-rule description will
turn out to be 3 of ≥ 10 rather than the list.**

### 1.1 `mmio.cpp` — registered at **TWELVE**

| # | crate | refusal |
|---|---|---|
| M1 | `c2-il` | `expr-cmp-eq` — the relational family. First blocker on all 3 |
| M2 | `c2-il` | `expr-brfalse` — the conditional-branch token |
| M3 | `c2-il` | the rest of the intra-body control-flow set (`29` label, `3A` jump, `4B` stmt-end, `53`/`54` scope) |
| M4 | `c2-il` | `expr-intrinsic-memcpy` — decoded, refused; "decoding is not accepting" |
| M5 | `c2-il` | `callseq-multiarg-lit` — the `li r5,72` literal argument slot; W-MEMCPY's GRID-L fence may or may not already admit this exact shape |
| M6 | `c2-core` | `mmioGetInfo`'s block plan: two guards, shared epilogue, the entry park `mr r11,r3 ; mr r3,r4`. `Selected::Seq` + `SeqPark` exist (w-mmio) — registered as **possibly already PAID** |
| M7 | `c2-core` | `mmioSetInfo`: the **callee-saved** park (`mr r31,r3` + `std/ld r31`, `saved_gprs:1`) — M-RULE's other branch — plus the post-call `lwz/lwz/cmplw/bf/stw` tail |
| M8 | `c2-core` | `mmioClose`: three calls of three kinds (same-TU direct `bl`, indirect `mtctr`/`bctrl` through a loaded member, external `bl`), park r31 **and** the coalesced park `mr r5,r4` (M-RULE's coalescing sub-rule) |
| M9 | `c2-core` | the **elided** call: the source calls `mmioSetBuffer(hmmio,0,0,0)` and the obj contains no such branch. The port must not emit it |
| M10 | `c2-core` | `cr0` vs `cr6`: `mmioClose` compares call results on **cr0** (`cmplwi r3,0` / `bf 2`) and formals on **cr6** |
| M11 | `c2-core/coff` | three framed functions' `$M`/`$M`/`$T` triples and `.pdata`, with the label counter running across all 11 emitted functions |
| M12 | — | the fixtures (positive per class at `/O1`, `_neg` with distinct clause keys) |

### 1.2 `Biquad.cpp` — registered at **FIFTEEN**

| # | crate | refusal |
|---|---|---|
| B1 | `c2-il` | `expr-cmp-eq` |
| B2 | `c2-il` | `expr-brfalse` + the control-flow token set |
| B3 | `c2-il` | `expr-op-0x27` — `?SetCoefficients` stops here with **every** sink on |
| B4 | `c2-il` | `expr-call-in-expr-recv-load-then-plumbing-0x3A` — `??0Biquad`'s own stop |
| B5 | `c2-il` | a recognizer for the if/else FP body (5 constant stores vs 5 divisions, 2 stores after the join) |
| B6 | `c2-il` | a recognizer for the ctor (framed same-TU call, no argument setup, a **dead** park) |
| B7 | `c2-core/coff` | two `.rdata` COMDAT pools emitted **between** the two `.text` COMDATs, with `__real@3f800000` in the *earlier* section though it is used *second* |
| B8 | `c2-core` | **B-RULE** — the dominating-block `lis` placement |
| B9 | `c2-core` | **B-RULE-2** — the compare/branch separation slot. `medium` at exactly 3 witnesses |
| B10 | `c2-core` | **B′-RULE** — the CSE reload order across the 5-division run |
| B11 | `c2-core` | **M-RULE** — `mr r10,3` in the ctor, a park whose value is never read |
| B12 | `c2-core/coff` | `_fltused`, and its position in the symbol table |
| B13 | `c2-core/coff` | the label counter: `$M2574`/`$M2575`/`$T2576` on the ctor, with a 3-label leaf emitted before it |
| B14 | `c2-core` | the 35 words of `?SetCoefficients` and the 9 of `??0Biquad` |
| B15 | — | the fixtures |

---

## 2. The conversion call — in PROBABILITY form

Registered as probabilities, not as a plan, because #770's streak is ten for ten
on optimistic misses and this lane has re-derived both chains at more than three
times the inherited "three rules".

| claim | P |
|---|---:|
| **C1** `src/xdk/nuispeech/mmio.cpp` is a byte-exact `match` at this lane's tip | **0.12** |
| **C2** `src/system/synth_xbox/Biquad.cpp` is a byte-exact `match` at this lane's tip | **0.08** |
| **C3** at least one of the two converts | **0.17** |
| **C4** neither converts and both are declined with a script-counted, named, sized chain | **0.83** |
| **C5** this lane ships at least one of M-RULE / B-RULE / B′-RULE into `crates/` in a form the differential grades | **0.45** |
| **C6** this lane ships **zero** `crates/` behaviour and is a measurement rung | **0.45** |
| **C7** the mmio chain re-prices ≥ 10 (P0.0 applied to §1.1) | **0.80** |
| **C8** the Biquad chain re-prices ≥ 10 | **0.85** |
| **C9** at least one refusal in each chain above turns out **already PAID** at this base (the w-xlr §7.2 / w-json R4 shape — an item priced from a reading rather than from the file) | **0.75** |
| **C10** at least one refusal fires that is in **neither** table above (the budgeted unnamed reader refusal, §4) | **0.60** |

**C4 is the registered headline.** A priced decline on both is the modal outcome
and is registered as such rather than discovered.

---

## 3. Metric predictions

Registered as **deltas** from the base block in §0, per #1749.

| metric | registered delta | confidence |
|---|---|---|
| TU match | **+0** | high (follows C4 at 0.83) |
| mismatch | **+0** | high — any non-zero is a lane failure, not a result |
| codegen-gap · port-error | **+0 · +0** | high |
| vocab-gap | **+0** | medium (−1 or −2 only if C1/C2 fire) |
| capture-fail | **+0** | high |
| FRONTIER | **+0** | medium |
| factor A / B / C | **+0 / +0 / +0** | high — this lane adds no section name |
| `A∧B∧C` · `b-and-c` | **+0 · +0** | high |
| function census | **+0** | medium — a new accepted shape moves it by the number of bodies it accepts |
| emitted census | **+0** | medium |
| `fnbyte-exact` · `fnbyte-differs` | **+0 · +0** | medium |
| `writer-sections` | **+0** | high |
| **workspace tests** | **DELTA +12** (not a total) | low — w-json registered +18 and measured +10; w-xlr's was also over. Registered low on purpose |
| gate fixture-verdicts | **+18 per fixture added**, and 0 if no fixture is added | high — 18 lanes × 1 verdict |
| per-lane `match` counts | **+0 on every lane except the one mode a new positive fixture lands in** | high |

**The verdict-neutrality prediction, registered as a set claim and not a count**
(#D4's evidence shape): over all 878 TUs at base and tip, compared **by name**,
the only verdicts that move are the ones this lane intends; `only-in-base` and
`only-in-tip` are both **0** on the non-intended side.

---

## 4. The budgeted unnamed reader refusal

The streak is **5 of 8 lanes**. Its last location is pre-armed here so that
scoring it is not retrospective:

1. **`IlFunction::callees()`** — w-json's R4 (priced, needed nothing) and
   w-xlr's three. `mmioClose` has *three* callees of three kinds plus one
   **elided** one, which is the shape most likely to break an iterator that
   assumes one call per statement.
2. **the unclaimed-`gl`-symbol gate** — `Biquad.cpp` carries two `__real@…`
   externals and `_fltused`, none of which is a function; `mmio.cpp` carries
   `memcpy` and `?FreeHandle@@YAXPAX@Z`.
3. **board #1704's filing** — `c2rs census` reports only the fall-through
   blocker, so every `_neg` cell will read the same key on an unpatched tree.
   Every clause key below is therefore to be read by **probe-patch-and-revert**,
   never by reading the unpatched census.

**Registered: P(the unnamed refusal fires) = 0.60 (C10), and P(it is item 1 of
the three above) = 0.30.**

---

## 5. Decline clauses, with sizes

Named and sized before the work, so that stopping is a scored outcome and not a
retreat.

* **D1 — the ≥ 4 standing clause.** *A frontier TU at ≥ 4 independent refusals
  is not a target* (#269). It fires on **both** TUs at base, at 12 and 15. This
  lane proceeds past it only on the argument that #269's clause ranks and does
  not forbid, and it is registered that **firing D1 and stopping is a valid
  outcome for either TU**.
* **D2 — the reader-depth clause.** If either TU still refuses in `c2-il` after
  the emitter-side rules are shipped, that TU is DECLINED at the reader and the
  chooser rules ship (if at all) behind a fixture rather than behind the TU.
* **D3 — the B-RULE-2 clause.** B-RULE-2 is `medium` at **exactly 3 witnesses**
  (wb-chooser §3.3, counted by `sep.py`). **This lane will not ship a rule that
  depends on it without first widening it with new grid cells that hoist two or
  more words into a block containing a compare.** If the widening is not run,
  anything that depends on B-RULE-2 is declined by name.
* **D4 — verdict neutrality.** Any widening that moves a previously-emitted obj
  on any of the 18 gate lanes, or moves a TU verdict this lane did not intend,
  is REVERTED, and the revert plus its reasoning is committed.
* **D5 — mismatch.** One `Port=Mismatch` anywhere and the change that caused it
  is reverted before anything else is done. `mismatch` is an alarm, not a
  budget.
* **D6 — the DISCLOSURE clause.** If any §5 constant of
  `WB_CHOOSER_FINDINGS.md` or the narrowed `r17..r31` mode is copied, the
  named row (D-CH-1 / D-CH-2) lands **in the same commit**. If the rules are
  taken from the black-box grid cells only, **no row is owed and the rung says
  which source each rule used**.
* **D7 — budget.** If at the point where the emitter work for one TU would
  start, the remaining unpaid count for that TU is still ≥ 8, that TU is
  declined and the lane spends its remainder making the decline *countable*
  rather than making it smaller.

---

## 6. What this lane registers it will NOT do

* It will **not** adopt WB_REGALLOC's tie-break selector (needs W-REGALLOC-2).
* It will **not** put a mode gate anywhere but the parser (#1638, fired twice).
* It will **not** put a number on the board that a script did not count
  (wb-chooser §8 — its own §3.3 draft was wrong on both halves from memory).
* It will **not** claim `cflow-if-n` or `cflow-loop` as a class.
* It will **not** widen `IlBundle::functions()`.
