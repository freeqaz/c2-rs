# w-fltret — PREREG

**Frozen before the first workload scan, before the first probe is compiled and
before the first line of `crates/` is written.** Lane `w-fltret`, worktree
branch `wt-w-fltret`, off master `05d743f7` (the w-memfit merge).

This is a **CODE** lane. Its commission is **R2** of
[`docs/rungs/2026-08-09-w-callprice.md`](../../docs/rungs/2026-08-09-w-callprice.md)
§7 — *the float value tail of a statement-position member-call sequence*, priced
there at **544 emitted over 9 constructs** and recommended as *"the largest
priced conversion rung on the board"*, **78× w-mcall's realized 7**.

The commission is explicit that the price must be **re-derived before anything
is built**, and that a failure to re-derive is a **full result**:

> Inherited prices have been wrong SIX times this week and seven blocked-key
> rankings were artifacts … Re-derive the 544 and the 9 constructs at base with
> a script, with the CONSTRUCTS column (distinct mangled names) beside the
> emitted column … **If it does not survive, say so and decline — that is a full
> result.**

---

## 0. What is inherited, quoted rather than paraphrased

Every number here is read out of a committed document **before** this lane runs
anything, so §4's P1–P4 are predictions and not observations.

| source | quoted |
|---|---|
| `rungs/2026-08-09-w-callprice.md` §1 | family at `c5ff9953`: **423,905 bodies / 35,576 emitted**, 27.25 % of the blocked emitted column |
| same, §3.1 row 21 | `recv-load-then-type-real-whole` — **439 emitted / 5 constructs / 436 TUs / 933 bodies**, `em/1k` 470.5 |
| same, §4.4 | *"`float Timer::SplitMs() { Split(); return Ms(); }` — `src/system/os/Timer.h:137`, ×434. **`-whole`: the census's own grammar walk says granting the real type finishes the body**"* |
| same, §7-R2 | *"Population **544 emitted over 9 constructs**: `recv-load-then-type-real-whole` **439 / 5** and `chained-then-type-real-whole` **105 / 4**. On the `prod` axis the same population is `msc-result-not-discarded-value-tail` **447 / 13**."* |
| same, §5.2 | the `call-token` clause split: *"the result is **not discarded** — the value tail — **447 emitted / 13 constructs / 727 bodies / 614.9 em/1k**"*, and *"434 of its 447 emitted are `?SplitMs@Timer@@QAAMXZ`"* |
| same, §7-R2 | *"**In port terms: a READER ADMISSION plus one named gate.** `BodyShape::CallSeq` already lowers the statement half …; `SeqTail::CallValue` already exists for the free-function spelling. What is new is the **member** value tail and `CallRet::discarded`'s `_fltused` obligation … on the *returned* rather than the discarded side."* |
| same, §7-R2 "Risk, stated" | *"the class is nine constructs, so a single unmodelled detail in `Timer::SplitMs` takes 434 of the 544 with it."* |
| `rungs/2026-08-09-w-callprice.md` §6.2 / board **#2026** | the standing hazard: an **address-taken stack local wears the same `26 <sym>` designator a relocation does**; `recv-object-*` is 10,144 emitted, 28.5 % of the family; `seq_call_arg_slots`' blanket refusal *"is what has been holding this, and it should be **kept** until the split has a census key"* |
| `rungs/2026-08-08-w-mcall.md` §4.3 | the class the workload had was **20 bodies / 7 emitted** against the **1,505 emitted** its first-blocker key carried — **0.46 %** |
| `fixtures/cpp/wmcall_seq_neg.cpp` N6 | *"the VALUE TAIL: `s->a(); return s->get();`. `SeqTail::CallValue` marshals a receiver into slot 0 *and* a post-op region, and **the two have never been graded together**."* — so the member value tail is declined **today even for `int`** |
| `crates/c2-il/src/func/body/expr.rs` `eat_return_head` | the `41` result gate is `eat_int_like_or_ptr4`, *"deliberately not widened to the FP classes"* — an FP value tail therefore blocks at `result-type` |
| `crates/c2-il/src/func/mod.rs` `touches_floating_point` | `SeqTail::CallLoadFp` is already the **fourth** `_fltused` producer and *"the first FRAMED one"*, with the symbol placed after the first FP-touching function's **complete** framed group and **the label stride not moving** |
| `git diff c5ff9953..HEAD -- crates/` at this base | **a comment-only change** in `expr.rs` (w-memfit's `memcpy` note), 14 insertions, no behaviour |

**The last row is why P1 is registered at a high probability**: the two merges
between w-callprice's tip and this lane's base changed no parser or emitter
behaviour at all.

---

## 1. The instrument, described before it is written

### 1.1 The re-derivation (the gate on whether anything is built)

`work/w-fltret2/pop.py`, over **one un-instrumented 878-TU base scan** at this
lane's own base, printing for every `expr-call-in-expr-*` key:

    emitted | constructs (distinct mangled names) | TUs | bodies | em/1k

`constructs` is the **distinct mangled name** count over the emitted column —
w-callprice's `cons`, re-derived rather than copied — with the per-key name
histogram printed for the two target keys so that *"434 of the 439 are
`?SplitMs@Timer@@QAAMXZ`"* is a measurement and not a quotation. The script
**asserts** that the per-key emitted counts sum to the family total taken
independently from the same scan, on w-callprice's P2 pattern.

No compound-key instrument is needed for the re-derivation: both target keys are
ordinary census keys and the name column is already in the scan's emitted
records. If it is not, that is the first unnamed refusal (§5).

### 1.2 The ladder, if the population survives

Three rungs, each graded against real `c2.dll` **before** the next is written.
The order is chosen so that each step's fence can be measured with the previous
step as the control:

* **L1 — the MEMBER value tail, integer.** `int f(S*s){ s->a(); return s->get(); }`
  — `fixtures/cpp/wmcall_seq_neg.cpp` cell **N6** exactly. This is a pure reader
  admission: `SeqTail::CallValue` and `plan_saved_gprs` already lower it for the
  free spelling, and the receiver is already appended to slot 0 by
  `eat_member_stmt_call`'s own convention.
* **L2 — the FREE FP value tail.** `float f(){ g(); return hf(); }` — the `41`
  result gate widened to the real classes at the **sequence value tail only**,
  with `_fltused` following from a new `SeqTail` variant.
* **L3 — the MEMBER FP value tail.** `float T::m(){ a(); return b(); }` — the
  composition of L1 and L2, and `Timer::SplitMs`'s exact shape.

**L3 is the commission's target and L1/L2 are not optional detours**: L3 cannot
be reached without both, and a lane that shipped only L3 would have two
untested clauses under one cell.

### 1.3 What is deliberately NOT built

`SeqTail::CallLoadFp`'s precedent says the FP tail wants its **own variant**, not
a flag, *"for the reason `CallLoad` is not a flag on `CallValue`: it is a
**different register file**"*. This lane follows that precedent
(`SeqTail::CallValueFp`) rather than adding a `double: bool` to `CallValue`.

---

## 2. DECLINE CLAUSES — named, sized, frozen

Each of these fires **without further argument** and each is quoted in the rung
whether it fires or not.

* **D1 — no `IlOp::Call` variant, and no operand-position call lowering.**
  Inherited from w-mcall #1961 through w-callprice's own PREREG D1. Size: the
  populations that would want one are `-then-call-recv-load-and-deref-load-more`
  **2,183 / 974** and `recv-field-off0-then-call-nested-call-and-type-real-more`
  **419 / 1** (w-callprice §3.1). **Neither is touched.**
* **D2 — `seq_call_arg_slots`' blanket `SlotArg::SymAddr` refusal is KEPT and
  FENCED, not widened.** Board **#2026**: an address-taken stack local wears the
  same `26 <sym>` designator a relocation does, `recv-object-*` is **10,144
  emitted / 28.5 %** of the family, and there is **no census key** for the split.
  If any cell of this lane's ladder approaches that boundary the refusal stays
  and the cell becomes a `_neg` cell. *A wrong emit is strictly worse than a
  gap* — #232 sat on master 241 commits.
* **D3 — the CHAINED receiver is declined.** `chained-then-type-real-whole` is
  **105 emitted / 4 constructs** of the commissioned 544. It is w-callprice's
  **R4** — *"R1's sibling, reached through the same `eat_member_stmt_call` arm …
  price it by building it"* — and this lane does not build it. **So the honest
  ceiling this lane is aiming at is 439, not 544**, and that is registered
  before the first scan.
* **D4 — no first-blocker population is quoted as a price.** #2025: R1's was
  2,188 emitted and its conversion was 0. Every number this lane publishes as a
  *conversion* comes from a counterfactual scan diffed **per TU and per key**,
  never from a key's size.
* **D5 — no `Some(k)` label charge is guessed.** If the measured label lead is
  mode-dependent, or if the measurement cannot separate this class from its
  controls, the class carries the same charge as the `CallSeq` it already is and
  nothing is added. w-bdnz reading 2: *"`None` is not conservatism here; it is
  the only value that can be right."*
* **D6 — a result CONVERSION is refused in both directions.** `41 <TYPE>` must
  be **byte-identical** to the callee's own CALL-token return TYPE. `float
  f(){…; return d();}` (`frsp`) and `double f(){…; return f32();}` (free, but
  ungraded) both refuse. Precedent: `leaf_fp_tail`'s *"a conversion applied to
  the **result** is refused whichever direction it goes"*.
* **D7 — no census key may be re-keyed.** The member value reader is
  **non-committal** and placed **last**; on decline the cursor is restored and
  the block the body already reported is re-raised. w-mcall clause D7, and the
  property §5 pre-arms a refusal on.
* **D8 — the population gate.** If the re-derivation of `recv-load-then-type-real-whole`
  comes back **below 300 emitted**, or if its construct column shows it is not
  the one-function class #2023 claims, this lane **declines and ships no
  `crates/` change**, per the commission.
* **D9 — a small conversion is not a reason to withhold a byte-exact widening.**
  If the ladder is byte-exact on the fixtures but the workload conversion is
  small, the class **ships** and the number is published as w-mcall published
  its 7. What would make it not ship is a wrong emit or an unfenced clause, not
  a small number.

---

## 3. FENCE ORDER, pre-armed

Frozen here so that §5's budgeted refusal has a place to fire:

1. The free-function `eat_call_head` probe runs **first**, on a scratch cursor
   (unchanged).
2. `eat_member_stmt_call` (the statement half, w-mcall) runs **second**
   (unchanged).
3. The **member value** reader runs **third and last**, non-committally.
4. Every clause inside it restores the cursor and returns `None`.

The FP widening of the `41` gate lives **only** at the sequence value tail —
`eat_return_head`'s own gate is **not** touched, because it is shared with every
other production in the crate and widening it there would admit an FP result in
positions no obj has graded.

---

## 4. PREDICTIONS

| # | p | prediction |
|---|---:|---|
| **P1** | 0.90 | The family re-derives at this base to **exactly 423,905 bodies / 35,576 emitted** — w-callprice's figures, unmoved, because the only `crates/` change since is a comment. |
| **P2** | 0.80 | `expr-call-in-expr-recv-load-then-type-real-whole` re-derives to **exactly 439 emitted / 933 bodies**, and its **constructs column is exactly 5**. |
| **P3** | 0.75 | `?SplitMs@Timer@@QAAMXZ` is **434** of those 439, in 434 distinct TUs — i.e. `emitted == TUs` for that name, the one-function-class shape #2023 claims. |
| **P4** | 0.70 | `chained-then-type-real-whole` re-derives to **105 / 4**, so the commissioned 544 / 9 survives as a *population*. **The population SURVIVES** (P2 ∧ P4). |
| **P5** | 0.55 | **`Timer::SplitMs` is not two calls and a return.** Registered because every hand-read construct on this board has had one more fact in it than the reading recorded (#1128, #2067), and because w-callprice read this body from a **header line number**, not from the IL. If it holds, the 434 does not convert and D8/§6 decide the lane. |
| **P6** | 0.60 | **L1 (the member INT value tail) is byte-exact** against real `c2.dll` at `/O1` and `/Ox` with **no `crates/c2-core` change at all** — the same "the lowering seam was already open" result w-mcall got one position over. |
| **P7** | 0.50 | **L2/L3 need exactly one new `SeqTail` variant and no new emitted instruction** — the FP value tail emits **nothing**, exactly as `CallValue { add_k: 0 }` does, and the whole difference in the obj is the `_fltused` symbol. |
| **P8** | 0.45 | The workload conversion of the whole ladder is **≥ 400 emitted functions**. (The commission's 544 less D3's 105 is 439; P8 is the claim that the reader actually reaches nearly all of it — which #2025 and w-mcall's 7-of-1,505 both say is the way these predictions fail.) |
| **P9** | 0.35 | The conversion is **≥ 100 emitted** but **< 400** — the "the class is real but the reader reaches part of it" branch. |
| **P10** | 0.20 | The conversion is **< 25 emitted** — the w-mcall branch, in which the class ships byte-exact and converts almost nothing. |
| **P11** | 0.70 | **`fnbyte-exact` moves by the same number as the emitted census, and `fnbyte-differs` moves by ZERO.** This is the strong instrument here: every function this lane adds is in a TU whose obj exists, so FBM grades the bytes rather than trusting the parser (trap 2). A non-zero `fnbyte-differs` delta is a **failure**, not a finding. |
| **P12** | 0.85 | **TU match 18 → 18** and **mismatch 0 → 0**. This rung converts functions, not TUs: `Timer.h` is a header inline, so its 434 TUs each still contain hundreds of other blocked bodies. |
| **P13** | 0.60 | The **label lead is 0** for this class — the FP value tail takes no slot the `CallSeq` it already is does not take, and the `_fltused` slot is charged **once per TU** by `coff::label`'s existing `fltused_slot_taken`. Measured against the obj by w-json's counterfactual method at **both** `/O1` and `/Ox`, never read off `LABEL_COUNTER.md`. |
| **P14** | 0.55 | The `_fltused` **symbol placement** for this class needs **no new rule**: `writer.rs`'s `funcs.iter().position(|f| f.is_float)` puts it after the first FP-touching function's complete group and WFL already proved that for a **framed** `CallSeq`. |
| **P15** | 0.65 | **≥ 1 unnamed refusal fires.** Pre-armed at two places, per the standing budget: **(i) FENCE ORDER** — a member value reader that runs before the free probe, or that commits its cursor on decline, re-keys a census key and shows up as a non-zero first-blocker key delta on a scan that should be neutral outside the target keys; **(ii) CLAUSE REACHABILITY** — a `_neg` cell that fires on an earlier clause than the one it is written for (w-bdnz §5.1 caught two, and *"a confounded cell passes the fixture gate exactly like a correct one"*). |
| **P16** | 0.50 | **The `-whole` suffix is a stronger signal than a first-blocker count and this lane demonstrates it.** Concretely: the ratio (converted emitted) / (key's emitted) for this `-whole` key is **> 0.5**, against R1's 0.0 on a `-more`/first-blocker population. If P8 and P9 both miss low, this is refuted with them and the `-whole` signal joins the ranking artifacts. |
| **P17** | 0.75 | **Verdict neutrality holds at all three levels** — 878 TUs by name with **0 changed** outside the intended conversions, all `gap-metric` keys accounted, all pre-existing fixtures byte-identical at `/O1` **and** `/Ox`. |

### 4.1 Registered direction, stated

Board **#2031** is w-callprice's own lesson: *"a prior calibrated on seven
instances of one mechanism misfires on the eighth"* — all four of its
registered-pessimistic predictions missed **optimistic**. Board **#770**'s
streak is the mirror: optimistic predictions missing.

**This lane therefore registers a distribution rather than a direction**: P8
(≥ 400), P9 (100–400) and P10 (< 25) are the three branches, they are mutually
exclusive, and their probabilities are written down before the scan. Exactly one
can hit. That is the honest form for a quantity this board has now got wrong in
both directions.

### 4.2 Metric predictions

| metric | base (predicted) | tip (predicted) |
|---|---|---|
| function census | 711,514 | 711,514 **+ (bodies converted)**, predicted **+900 ± 400** |
| emitted census | 39,200 | 39,200 **+ 400 ± 250** |
| TU match | 18 | **18** |
| mismatch | 0 | **0** |
| `fnbyte-differs` | (read at base) | **unchanged, exactly** |
| `fnbyte-exact` | (read at base) | **+ the emitted delta, exactly** |
| `factor-c` / `b-and-c` / `frontier` | (read at base) | **unchanged** |
| `#[test]` count **DELTA** | — | **+8 ± 5** (a DELTA, not a total — trap #1710a) |
| workspace tests | 1,347 passed / 0 failed / 36 targets | **1,347 + the delta, 0 failed**, target count **+0** unless a fixture adds one |

---

## 5. The budgeted unnamed refusal

**One**, pre-armed at FENCE ORDER and CLAUSE REACHABILITY (§4 P15). If something
else fires instead it is reported as a **miss of the budget** and w-park's streak
takes it, per the standing precedent — it is not absorbed into the narrative.

---

## 6. What "decline" looks like, written before it can be embarrassing

If D8 fires, or if P5 holds and `Timer::SplitMs` is not the body w-callprice read
it as, this lane ships:

* the re-derivation with its constructs column, as a committed script and its
  output;
* the rung, saying **which** of the inherited numbers did not survive and by how
  much;
* **zero `crates/` lines**, and `git diff master -- crates/` empty at the tip.

That is a full result and it is the second-best outcome on the board, not a
failure. The worst outcome is a class that ships on an inherited number nobody
re-derived.
