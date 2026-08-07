# w-seam2 — PREREG

Board **#844**, the composition seam. Committed **before the first probe obj of
this lane exists**. Lane `w-seam2`, branch `wt-w-seam2`, off master `e9605bd0`
with `wt-w-f23` (`f782ec5d`) merged in, because this lane's *input* is the
production that lane landed and then deliberately refused.

Everything below is registered ahead of measurement. Scored in
`docs/rungs/2026-08-08-w-seam2.md` §8, hits and misses both.

---

## 0. Corrections to the brief, registered before probing

The brief was written before `w-f23` finished and two of its statements are
already superseded by that lane's own measurement. Recorded here rather than
silently worked around:

1. **The brief says `xboxheap.cpp` prices at 5 and that this lane pays the
   emitter one of them.** `w-f23` §5 re-prices it at **3** — F2 paid, F3's
   reader half paid — and shows that **`xboxheap.cpp` itself is not reachable at
   all**: its shipped spelling binds `auto& listHead = mListHead;`, which c1xx
   spells as a store into a **local** whose token then stands in two later
   stores' **base** position, and `parse_store_stmt` requires a formal there.
   That is refusal **5** (#839) and it is `crates/c2-il`'s. **So this lane
   cannot convert `xboxheap.cpp` whatever it emits**, and it registers TU match
   **10 → 10** in advance rather than discovering it.
2. **The brief says the seam is an emitter.** `w-f23` §4.2 measured that it is a
   **carrier**: `IlFunction` has no way to spell "a store run *and* a call", and
   `codegen::select` tries the alternatives in a fixed order, so a function
   carrying both emits one and silently drops the other. The rung is therefore
   *give the model a composition carrier and dispatch on it*, not *add an
   emitter arm*.

## 0.1 The acceptance signal, named in advance

`fn_blockers` / `emit_blockers` key **`store-run-call-no-emitter-carrier:eof`**,
which `w-f23` landed reading **8 IL bodies, 3 of them emitted**.

**`codegen-gap` is NOT this lane's acceptance metric** (board #1164): the scan's
partition is per TU and every vocab-gap TU carries another undecodable body, so
it stayed 0 through a real reader payment. The signals are the `:eof` key, the
frozen grid's byte compares, and the 1,576 generated sweep cases.

---

## 1. Board #871, clause by clause — the argument this lane is testing

#871: *"#322 is a prerequisite of #844, not a neighbour of it — every
composition #844 names is INVISIBLE to `fnbyte-differs`."* Registered position,
before this lane measures anything:

| clause | as #871 states it | registered verdict |
|---|---|---|
| **C1** | `fnbytes.rs` maps `Selected::{Tail, Framed, Seq, CondPair}` to `FnByte::Partial` **by construction** | **DISCHARGED.** #322 closed by `w-fnbyte`; `fnbyte-partial` is 0 on master |
| **C2** | so a seam's whole population lands in a blind bucket and the alarm reads 0 whether the seam is right or wrong | **DISCHARGED** *if and only if* the seam's selection is one of the four now-graded shapes. Registered: it will be `Selected::Seq`, which is graded |
| **C3** | that is #232's shape with the alarm REMOVED | **DISCHARGED for `fnbyte`, and REPLACED by a different missing alarm** — `w-heap` §6.3: the 878-TU scan cannot *generate* a store-run-before-a-call. That successor clause was live until `w-gen` landed `88-store-run-call.py` (1,576 cases) and is **now also discharged** |
| **C4** | #322 *"is the only thing on the #844 path that widens the warranty without widening the accept surface"* | **still TRUE as written, and no longer blocking** — it is a statement about #322's value, not a live precondition |

**Registered conclusion: #871's blocking argument no longer holds, and its
successor clause does not either.** The direction this could lose: if the seam's
selection turns out **not** to be one of the four graded shapes — e.g. if it
needs a fifth `Selected` variant that `fnbytes.rs` maps to `Partial` — then C2
comes straight back, and the lane must either grade the new variant or decline.
**That is the registered loss and it is checked before any emitter is wired.**

---

## 2. What the seam will emit, and what will refuse

### 2.1 The carrier

A composition field on `IlFunction`, set by exactly one parser production
(`try_parse_store_run_call`), dispatched **first** in `select_function` so the
alternatives cannot race. Registered as the structural claim: *the fix for #232's
mechanism is not "try the composition earlier", it is "the composition is its own
dispatch key and no arm below can see its operands".*

### 2.2 The emitted body

```text
  prologue(saved_gprs = 1)
  <the scheduled store run, order::schedule + alloc::allocate, no blr>
      with `mr r31,r3` inserted at its measured slot
  bl <callee>                    one REL24, the offset encoded from `off`
  mr r3,r31
  epilogue
```

Registered: this is `Selected::Seq` with the run prepended to `setups[0]` and
`mr r3,r31` as the tail, i.e. **the existing framed-sequence path**, not a new
obj shape. `codegen::frame`, `call_seq_text`, `coff::pdata` and `coff::label` are
consumed and not re-derived (#842).

### 2.3 What REFUSES, registered ahead of the grid

| refuses | why |
|---|---|
| any run `scheduled_gpr_run_text` does not answer | including every **F2 `AddrOf`** run — `parse_simple_gpr_run` declines the four-op group, and the seam must **refuse**, never fall through to `store_leaf_text`'s source-order walk |
| a call whose argument setup is not empty | the reader already gates it (#1129, stricter than the prose); the emitter restates it as a backstop |
| every non-constructor form | `fvoid` / `fretcall` / `fdiscard` are frame words 0 and tail-call (#869/#1131) |
| a reference-bind spelling | #839, `crates/c2-il`'s, and not paid here |
| two callee-saved GPRs or more | outside the frame class |

### 2.4 The one modeled fact this lane would be SHIPPING

The **slot of `mr r31,r3` inside the run**. `w-seam`'s #867 fitted
`stores_before_mr = nprod − 1 + min(u, 2)` on 24 cells and held it on an 18/18
fresh holdout, and **it has never shipped**. Everything else in §2.2 is already
shipped and graded somewhere.

---

## 3. The decline floor, registered against the INCUMBENT

Today's behaviour is a refusal that is **right 100 % of the time on what it
refuses**. A seam that is *mostly* right is strictly worse. So:

* **F-1.** Ship only if **every** cell the seam accepts in the frozen grid is
  **byte-exact** against real `c2.dll`. One accepted-and-wrong cell and the seam
  does not ship at all — not narrowed around the cell, **not shipped**.
* **F-2.** The 1,576 `88-store-run-call.py` cases stay at **0 mismatch** and
  `sweep graded` does not fall. Run on **every** iteration, not at the end.
* **F-3.** `mismatch` 0, `fnbyte-exact` unshrunk, `differs` ungrown,
  `reloc-differs` 861 ungrown, `match-tu-differs` / `-reloc-differs` 0.
* **F-4.** If the `mr` slot rule (§2.4) misses on any grid cell, the seam is
  narrowed to the sub-population the cells prove, or it does not ship. A rule at
  98 % is board #621's refused trade.
* **F-5.** No fitting to `xboxheap`. Its body is **unreachable** in this lane
  (§0), so it cannot even be the target; the grid is structural and its cells
  are shaped to reach productions, not to reproduce one obj.

---

## 4. Predictions, to be scored

| # | registered |
|---|---|
| **P0** | *the headline* — #871's argument is fully discharged and the seam's selection is `Selected::Seq`, one of the four #322 grades. **This is the claim I most expect to lose**, and the direction is C2: that the composition needs its own `Selected` variant, which `fnbytes.rs` would map to `Partial` and put the alarm back out |
| **P1** | the carrier is small — one `IlFunction` field, one `select_function` arm dispatched first, ≤ 120 lines of new emitter |
| **P2** | `store-run-call-no-emitter-carrier` does **not** go to 0. Registered at **≥ 4 of the 8 remaining**, because F2's `AddrOf` runs are refused by `parse_simple_gpr_run` one group earlier and `xboxheap`'s own mix is exactly that |
| **P3** | TU match **10 → 10**. `xboxheap` is unreachable (§0) and the 8 bodies live in TUs that carry other undecodable bodies (#1164) |
| **P4** | the `mr r31,r3` slot is #867's `nprod − 1 + min(u, 2)` on every cell this lane accepts, and `nprod = 0` evaluates to slot **1** (which is `w-heap` §3.2's own `f3_a3_c0` cell, `stw ; mr 31,3 ; stw ; stw ; bl`) |
| **P5** | the leaf control `cnone` is byte-identical to what master already emits for the same run — the seam adds no drift to the shipped store-run emitter |
| **P6** | at least one cell in the grid that the *reader* accepts will be one the *emitter* must refuse, and the refusal will be `alloc`/`parse_simple_gpr_run`'s, not a new one |
| **P7** | the sweep's `88-store-run-call` port split moves off `44 Match · 1,532 NotImplemented` in the Match direction, by **≥ 30** cases |

---

## 5. What this lane will NOT do

* Not touch `crates/c2-il`'s parser productions — `w-f23` owns them and has
  finished. The **carrier** (`IlFunction` + `shape_to_function`'s refusing arm)
  is the declared hand-off point and is the only `c2-il` surface this lane
  edits.
* Not touch `crates/c2-core/src/coff/` — `w-order3` owns it (#174).
* Not lift `alloc::allocate`'s mixed-kind refusal (#836/#868). It is now
  *reachable*, which is exactly why it must not be lifted by a lane whose grid
  was not built to test it: `w-heap` §4.1.1 refuted clause 1 on this mix and six
  keys are already on record as refuted.
* Not pay #839, the reference bind.
* Not re-derive `codegen::frame`, `coff::pdata` or `coff::label` (#842).
