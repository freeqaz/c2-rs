# w-5c2 — PREREGISTRATION

Committed **before the first scan of this lane**. Lane `w-5c2`, worktree branch
`wt-w-5c2` off master **`851938df`**.

The row: **`expr-call-in-expr-*-op-0x5C`** — 1,212 functions in 810 TUs on the
default 878-TU scan (`w-5c` §6, board **#1428**), the *second* `0x5C` rung, the
one `chain_skip_form(0x5C) == TypeVarint` moved by **0**.

---

## 1. What the row IS — established by reading, before any measurement

Three findings from the tree, all citable, none of them new measurement:

1. **The key is raised by `mcall`'s completeness matcher, which accepts
   nothing.** `mcall.rs`'s module header: *"It **accepts nothing**: every entry
   point returns a `Block`, the gate is byte-for-byte unchanged, and only the
   census key moves."* The `-then-op-0x5C` half is minted in `mark_whole`
   (`crates/c2-il/src/func/body/mcall.rs:966`, *"Diagnostic only: the `Err`
   stays an `Err`"*) from `Fail::blocker` → `FailKind::Value` → `Blocker::Op(b)`
   at `mcall.rs:1589`.

2. **The refusal is a VALUE WHITELIST, not a missing width.** `body_matches`
   calls `eat_dtor_stmt_trailer` (`mcall.rs:1773`, its **only** call site),
   which requires `5C <int-like TYPE> <flag>` with the flag byte in
   `TRAILER_FLAGS = [(0x11, …), (0x01, …)]` — **two** measured values. `w-5c`
   anchored the token's real width as `5C <TYPE> <varint>` on **335,716 sites,
   0 desyncs, two independent anchors**, and recorded state values `01 02 03 04
   41 43 0101` plus a 9,645-site escape (`80 01 01 00 00`, state 257). So the
   classifier is **narrower than the tree's own reader of the same byte** —
   `control_flow.rs::operand()` and, since `w-5c`, `chain_skip_form` itself.
   `docs/IL_CALL_IN_EXPR.md` §16.2 already names the mechanism in one cell:
   *"`op-0x5C` | 890 | 0 | a destructor statement trailer whose flag is neither
   measured value"*.

3. **The `0` beside it in that table is a RENDERING, not a measurement.**
   `blocker_is_measured(Blocker::Op(b))` is `BARE_BINARY_OPS.contains(&b)`
   (`mcall.rs:1602`, `1689`) and `0x5C` is not in that set, so `mark_whole`'s
   greedy chain **breaks on the first iteration** with `need =
   NEED_UNMEASURED`. The published "whole within 4 = 0" for `op-0x5C` is
   therefore **0 by construction**: the instrument cannot produce any other
   number for this row. `mcall.rs:1600` says so out loud — *"a pair whose second
   half is `op-0x5C` … is reported as UNMEASURED at the pair level"*.

**So the row is (a) + (c), and only partly (d).** (a) a diagnostic classifier
narrower than the tree's own reader; (c) whose published zero is a rendering of
the honesty gate rather than a fact about the bodies. (d) applies to the *width*
question only — `w-5c`/#1428/#1357 measured a width at zero and that is settled
— and **not** to the whitelist question, which no lane has touched
(`git grep eat_dtor_stmt_trailer docs/` returns nothing).

## 2. What is being built

`C2RS_SINK_MCALL_TRAILER`, **OFF by default**, in `eat_dtor_stmt_trailer` only.
Two **nested** arms, on `C2RS_SINK_BRANCH`'s pattern (board #440):

* `flag`  — keep `5C <int-like TYPE> <one byte>`, drop the two-value whitelist.
* `varint` — the full `w-5c`-anchored width `5C <TYPE> <varint>`: any TYPE
  `read_type` reads, any varint state including the escape. Strictly wider than
  `flag`.

It **cannot** change acceptance: `eat_dtor_stmt_trailer` has one call site, in
`body_matches`, which is reached only from `mark_whole`, which is diagnostic.
The sink ships permanently and tested, because the recurrence this session is
lanes that build a counterfactual and then revert it.

## 3. Registered predictions

| # | prediction | direction of error if wrong |
|---|---|---|
| **P-STRUCT** | Every acceptance-side number is **identical** at both ends and in both arms: TU match **11**, mismatch **0**, codegen-gap **0**, vocab-gap **860**, capture-fail **7**; census numerator unchanged; `gap-metric` diff EMPTY with the sink OFF. | none — structural, not empirical. If this moves, the sink is misplaced and the lane is void. |
| **P-KAC** | The known-answer control: the base default scan reads `expr-call-in-expr-*-op-0x5C` = **1,212 functions / 810 TUs**, reproducing `w-5c` §6 to the unit, and the `fn_blockers` sum reads **1,751,957** (`w-5c` §9). A different number indicts the instrument before any new number is read. | — |
| **P-REC** | **Recovered = 0 of 1,212.** No function leaves the blocked set; every one that moves is **RENAMED**. `fn_blockers` sum **identical to the unit** in both arms. | none — structural. |
| **P-MOVE** | ≥ **90 %** of the 1,212 relabel under `varint`. | DOWNWARD (I expect more, not fewer). |
| **P-WHOLE** | **The open number.** Of the 1,212, those landing on a key carrying a `-whole`/`-whole<k>` suffix under `varint`: point estimate **360 (30 %)**, band **10 %–60 %**. | **DOWNWARD** — I expect to be too low. `EH_RECORDS.md` §7.2's own witnesses for this key are `void onlylocal(){ MemA s; }` and `void twolocals(){ MemA s; MemB t; }`, bodies whose *entire* content is the statement the trailer terminates; if the trailer is eaten those bodies end. Board #770's streak is ten optimistic misses, so the band is registered wide in the direction that streak does **not** point. |
| **P-DEST** | The renamed land in **15–60** distinct successor keys, and the single largest successor is a `-then-call-*`, `-then-plain-call`, `-then-chain-bind` or `-then-type-ptr` — i.e. one of `IL_CALL_IN_EXPR.md` §16.2's already-ranked second blockers — **not** another `op-0xNN`. | UPWARD on the key count. |
| **P-LADDER** | **0 of 17** ladders extend, **+0** rungs, on the hatched instrument. `Main.cpp` stays `net=3 STUCK`. The sink is not a `ladder.py` token and `ladder.py` cannot set it; this is registered so the row is not silently priced off a climb. | none. |
| **P-TESTS** | `#[test]` bodies under `crates/`: **+2 to +5**. | — |
| **P-TU** | **0 TUs convert**, in either arm, at either end. | none — structural. |

## 4. THE STOPPING RULE — frozen here

**This lane ships no acceptance change and no `mcall` production, whatever the
counterfactual says.** The deliverable is the sink, the number, and the
correction to `IL_CALL_IN_EXPR.md` §16.2's rendered zero.

The number that decides how the row is FILED, and the decline threshold:

* **< 100 of 1,212 whole within 4** (< 8.3 %) → the row is **DECLINED and filed
  as confirmed worthless**: its published zero was a rendering, the real number
  is also ~zero, and the next lane is told not to rank it. A decline with a
  table is a full result and on this row's history it is the likelier one.
* **≥ 100** → the row is filed as a **live ranking row** with its size, the
  named pair, and an explicit statement that it still converts **0 TUs** — a
  ranking correction, never a rung.

Either way the lane does **not** widen `eat_dtor_stmt_trailer` by default. The
whitelist is a *gate on a field*, and `GAPS.md` §6's "skipped field" hazard is
exactly what the two-value gate exists to prevent; removing it by default would
trade a labelled approximation for an unlabelled one in the default census.

**Decline conditions that void the result rather than answer it:**

* **D1** — the two scans are not of the same binary. Both provenance records'
  `binary_sha` are printed and compared; a difference voids the table.
* **D2** — P-STRUCT moves. Any acceptance-side movement means the sink is not
  where this prereg says it is.
* **D3** — the `fn_blockers` sum moves. A relabelling that loses functions is a
  bug, not a finding.
* **D4** — P-KAC misses. If the base does not reproduce 1,212 / 810, nothing
  downstream is read.

## 5. What this cannot settle, said in advance

* **The operand-position `5C`** (`w-5c` §2.4: 18.05 % of 335,716 sites) is not
  reachable from `eat_dtor_stmt_trailer`, which is only tried at the
  statement-terminal position. A body blocked on one of those is not in this
  population and this lane says nothing about it.
* **The escape's value range.** `w-5c` §3.2's bound carries over verbatim: the
  corpus witnesses the escape's *width* at 9,645 sites and its *value* at
  exactly one point (state 257).
* **`Main.cpp`.** Board #1428 says it needs an `mcall` production, not a width.
  This lane does not build one and does not price the TU.
