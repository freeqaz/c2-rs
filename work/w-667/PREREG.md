# w-667 — PREREGISTRATION

    Lane:   w-667
    Base:   master 56912b72 ("merge w-throughput: the verification cycle goes
            605s -> 112s with the SAME verdict, and the skip I hoped for is
            declined at 0 of 40")
    Branch: wt-w-667, worktree .claude/worktrees/w-667
    Written BEFORE: the first probe obj, the first `cl.exe` invocation, and the
            first line changed under `crates/`. Nothing under `crates/` has been
            touched at the time of writing; `git status` in the worktree is
            clean except for `work/w-667/`.

Frozen once the first counterfactual build compiles. Corrections go in the rung
doc as scored misses, never by editing a line above the score table.

One base-evidence scan of the **unmodified** base binary has already run
(`work/w-667/scan/base_evidence.{txt,jsonl}`, `binary_sha
7d03c589ecc5c20fc1a5689538a899a0`). It changed no code and is the "both ends,
base" row the brief requires; every number in §1 and §2 below is read off it.

---

## 0. The rung as briefed

`assign-store-type-8643` — board **#667**'s key, tag `86` kind `43`, a 4-byte
**data pointer** in the destination slot of an assignment statement. Briefed as
the row under which five of `w-lineage`'s cells actually sit (board **#1335**),
the layer that "killed allocation keys ten and eleven, priced #1266 and killed
`H-MIX`".

**Its size at this base, measured, not quoted:**

| axis | count | of | share | rank |
|---|---:|---:|---:|---:|
| blocked **emitted** functions | **1,133** | 130,575 | **0.868 %** | **23** |
| blocked functions (all) | **6,821** | 1,751,957 | 0.389 % | — |
| TUs in which it blocks an emitted function | **313** | 878 | — | — |

Siblings, same production, same base: `assign-store-type-8212` 294 emitted /
1,906 all (rank 66), `assign-store-type-8644` 5 emitted / 1,402 all (rank 236).
Family total **1,432 emitted / 10,129 all**.

**This is not a top-of-board row.** The brief's four predecessors were priced
against rows at 17.1 % and 11.5 %; this one is rank 23 at 0.868 %, and I record
that before measuring anything so the decline is not later read as a big row
collapsing.

## 1. The prior art I read BEFORE writing this, oldest hit read LAST

`grep -ril` over `docs/` and `scripts/` for the key; then a **separate** topic
search of `BOARD.md` for `eat_int_like` / `int_like` / `ptr4` / `Ptr4` /
`pointer store` / `assignment layer`, because the board's rows do not carry a
topic grep's phrasing. Read newest first, as the standing instruction requires:

| where | what it says |
|---|---|
| **#1335** (`w-op27`, newest, 2026-08-08) | five `w-lineage` cells move `expr-op-0x27` → **`assign-store-type-8643`** one-for-one under the sink arm, **converting 0**. It is a *relabelling of where those cells stop*, and it is the reason I was sent |
| **#1295** (`w-lineage`, 2026-08-08) | of the 410 allocation-key cells, **406 report `expr-op-0x27` and 4 report `assign-store-type-8643`** — and **not one of the 410 is in the reader's class** |
| **#1099 / #1128** (`w-front2`, `w-heap`) | the store/bind seam next door: `xboxheap` re-priced 3, then **5**; `codegen::alloc`'s mixed-kind refusal is LIVE on that body |
| **#667** (`w-depth`, the row itself) | already carries **its own correction**: the row first claimed this was *"the single most common round-0 blocker on the frontier"* and that is **FALSE** — the frontier aggregate is `expr-cmp-eq` 11/6, `expr-jump` 10/3, then `assign-store-type-8643` **5 fns / 3 TUs**. Those 3 were `Sort.cpp`, `negate_test.cpp`, `keygen_xbox.cpp`; **`Sort.cpp` has since matched** (board #760) |
| **`w-cmp` §3.1** (2026-08-05) | the greedy blocker ladder, re-ranked after the `C2RS_SINK_REL` substitution: `assign-store-type-8643 **converts 2** appears 3`. **This is the only surviving optimistic claim about the row and it is a DRIVER number** — a ladder credit, not a measurement |
| **#408** (`w-dclass`/C, 2026-08-05) | the key's vocabulary is **CLOSED at three kinds** and the closure is *derived*: `eat_operand_type` admits int4 / ptr4 / int1u, so every other type refuses one token earlier as `expr-load-type-*`. **There is no tail to discover** |
| **#407** (`w-dclass`/C, 2026-08-05) | **THE COUNTERFACTUAL I WAS SENT TO RUN HAS ALREADY BEEN RUN, TWICE.** Counterfactual 1 is *exactly* the one-line widening this seam has: `eat_int_like` → `eat_int_like_or_ptr4` **at the store**. Result: `census 706,555 -> 706,555 (+0) · TU match 8 -> 8 · FRONTIER 19 -> 19`, with `−6,820 assign-store-type-8643 / −1,402 -8644` redistributing to `+3,855 expr-op-0x60 · +3,086 expr-jump · +722 expr-op-0x10 · +451 expr-op-0x27 · 51 rows of 1s and 2s`. Counterfactual 2 lifts the type gate **entirely**: `+0` again. The row's verdict is **"Do not rank this row."** |
| **#406** (`w-dclass`/C) | the key exists at all only because `assign-store-type-0x86` printed the slot **width**; the split into `-8643` / `-8644` / `-8212` is where 8,222 became 6,820 + 1,402 |
| **#269** (`w-conv`) | the standing decline clause — *a frontier TU at ≥ 4 independent refusals is not a target* — and `negate_test.cpp` **priced at 9**, re-derived at 9 **by a different partition** than `w-cross`'s |
| `assign.rs`'s own `dst_not_formal` doc (in-tree) | *"Lifting the gate entirely converts 0; **lifting it and the store-type check below it converts 0 as well**"* — the joint counterfactual, in the source file I own |
| **`2026-07-31-assign-eof.md`** (oldest, read LAST) | the original. Counterfactual B lifted the destination gate **and** the store-type check: `+0`, with the 8,221 landing on `+4,034 expr-jump · +3,855 expr-op-0x60 · +809 recv-load-then-call-other · +722 expr-op-0x10`. #407 reproduced that to **within one row** five days and 27 merges later |

**So #667 has already measured ZERO — three times, at two bases, on the exact
mechanism, and the board says in so many words "Do not rank this row."** This is
the fourth row to re-enter the widening order on size alone after doing so.

**Why I am running it again anyway, rather than citing #407.** The base has
moved a long way: census `706,555 → 711,486` (+4,931), TU match `8 → 11`,
FRONTIER `19 → 16`, emitted census `38,458 → 39,185`, and four lanes have worked
this exact store/bind layer since (`w-front3`, `w-mrslot`, `w-lineage`, `w-bd`).
The row's own emitted count is **1,133 today**; #407's function-axis count was
6,820 and is **6,821** now. `w-rtti` and `w-op27` both re-derived a declined
price at a moved base rather than transcribing it, and both were right to.

## 2. Which of the five categories I expect this to be

Registered before any measurement:

1. **A private limit inside a recognizer that already exists — YES, this is my
   primary registration, and the limit is already located.** `assign.rs:193`
   asks `eat_int_like`. The sibling one production upstream — `eat_operand_type`,
   `readers.rs:794` — asks `eat_int_like_or_ptr4` and **admits the width-4
   pointer**. So the *operand* of `T *x; x = q;` is accepted and its *store* is
   refused, on the same type bytes, two lines apart, in the same statement. That
   is the brief's category 1 in its purest form: the recognizer exists, the
   emitter behind it (`BodyShape::StraightLine`, which folds the store away
   entirely) exists, and one call to a narrower locator manufactures the whole
   row.
2. a production misfiled under an opcode — **NO**
3. **real but far smaller than its size — YES, as the consequence.** #407 and
   `assign-eof` both measured the realizable worth at **0** against a 6,820-row
   stock, which is an ∞ gap, not a 67× one
4. unmeasurable — **NO**, but with a caveat the brief's other lanes did not
   have: **the instrument does not exist yet.** There is no `C2RS_SINK_*` for
   this key (`grep C2RS_ crates/c2-il` gives exactly four, all in the
   *expression* layer). Unlike `w-op27` I cannot run the counterfactual with an
   env var alone — I must **build the sink first**. §4 registers what it is
5. mis-described — **NO.** The key has **one mint site**
   (`assign.rs:194`, `blk_type(seg, p, p, "assign-store-type")`), reached from
   one production, and #408 proved its vocabulary closed. It is *not*
   `expr-op-0x27`'s shape: it is not a fall-through label, and the overloaded-
   `None` question the brief raises against `noform-0xNN` has a negative answer
   here that I will state as a count, not as an argument

**Category 1, with 3 as its consequence and a measured zero already on the
record.**

## 3. What I predict, and the stopping rule — frozen NOW

### 3.1 The ceiling, taken NEAT (board #770's rule)

The rule: *when a row's blocker is a class whose emitter already exists, count
the **independent** refusals between ceiling and emitter and take the ceiling
neat.* The emitter does already exist — a folded pointer local and a folded
`int` local are the same register move, and `StraightLine` is what both come out
as. So:

* **Ceiling, neat: 1,133 emitted functions** (6,821 on the function axis). Not
  discounted, not scaled by a prior.
* **Independent refusals between ceiling and emitter, counted:** #407's measured
  successor set is `expr-op-0x60`, `expr-jump`, `expr-op-0x10`, `expr-op-0x27`
  plus a 51-row tail. Asking the load-bearing question — *what varies between
  these refusals?* — the answer is **not** "nothing, one variable at different
  thresholds": `0x60` and `0x10` are different opcodes in the expression layer,
  `expr-jump` is a control-flow construct in a different layer, and `0x27` is the
  offset-add fall-through in a third. **≥ 3 independent, and they are not one
  variable.** So the ceiling does **not** collapse to the emitter, and 1,133
  stands as the ceiling rather than as a prediction.

### 3.2 The one TU that could move, named in advance

Of today's **16** frontier TUs, exactly **two** name this key, and only one of
them could be closed by it alone:

* **`src/system/negate_test.cpp` — `{'assign-store-type-8643': 2}` and NOTHING
  ELSE.** Its entire emitted blocker set is this key. It is the whole of
  `w-cmp`'s "converts 2" and the only TU on the board for which this rung is a
  complete answer.
* `src/keygen_xbox.cpp` — 2 of 18 blocked emitted, across **10** distinct keys
  (`expr-jump` 8, …). `w-front2` priced it at **≥ 21**. Not a candidate.

And `negate_test.cpp` is **already priced at 9 independent refusals by `w-conv`,
re-derived at 9 by `w-front2`/`w-cross` under a different partition** — the
cross-check that says those 9 are not one variable at nine thresholds. Board
#269's clause fires on it at 9 ≥ 4.

### 3.3 The numbers I predict

* **Point estimate: +0 emitted functions, +0 TU match**, unchanged from #407,
  because #407 ran the identical one-line widening on the identical workload.
* **The 1,133 will be RENAMED, not recovered**, over a successor set whose head
  I predict is `expr-op-0x60` (#407's head at 3,855 on the function axis).
* **The successor key COUNT is my one genuinely new prediction:** #407 reported
  4 named successors + 51 tail rows on the function axis; I predict the
  **emitted** axis lands in **10–60** distinct successor keys.

### 3.4 THE STOPPING RULE — the number below which I decline

**I DECLINE, ship the record, change nothing under `crates/` beyond the sink and
its revert, and mint the board rows, if the counterfactual returns:**

> **fewer than 100 recovered emitted functions AND 0 TU match movement.**

**I RE-OPEN and build** — freeze a grid (sha256 + every rival's predictions,
structural axes enumerated and crossed first, arity varied inside each cell, the
generator asserting its own classes and refusing to write if a class is absent or
two rivals are indistinguishable), grade against real `c2.dll` under wibo at the
workload's own flags, and never by disassembly reading — **only if the
counterfactual returns ≥ 100 recovered emitted functions OR ≥ 1 TU.**

100 of a 1,133 ceiling is 8.8 %. It is chosen above `w-op27`'s realised 0.036 %
and above the ∞-gap zeros of #407 and `assign-eof`, and below any number at
which the ceiling would be worth a grid.

**`negate_test.cpp` gets one extra clause, because it is the row's whole TU
case:** if the counterfactual converts `negate_test.cpp` to `match`, that is
**≥ 1 TU** and I re-open regardless of the emitted count. If it converts
`negate_test`'s 2 functions to *some other blocker key* and the TU stays
`vocab-gap`, that is the decline, and I must say **which key** they land on —
board #269's price of 9 predicts they land on one of the other 8 facts and I
will name it rather than assert it.

## 4. The instrument, and what it is NOT

There is no existing sink for this key, so I build one. Registered in advance:

* **`C2RS_SINK_STORE_TYPE=ptr4`** — at `assign.rs:193` only, swap `eat_int_like`
  for the sibling `eat_int_like_or_ptr4` that the operand gate one production
  upstream already uses. This is #407's counterfactual 1, exactly.
* **`C2RS_SINK_STORE_TYPE=any`** — consume any well-formed TYPE at the same
  site. This is #407's counterfactual 2, and it prices the whole family
  (1,432 emitted) in one number rather than only my row.
* **It is OFF unless the variable is set**, so every gate lane, the sweep, the
  mode cross and `cargo test --workspace` run on the unmodified parser. I will
  say so beside every number it produces, and I will **assert in a unit test
  that the test process does not set it**, as `expr.rs:1875` does for
  `C2RS_SINK_CHAIN`.
* **Registered honestly, per board #661's lesson about
  `C2RS_SINK_OFF_ADD_ARG`:** this sink is *not* obviously measurement-only.
  Folding away a store whose destination is a pointer the class cannot prove
  register-resident is exactly the mis-emit `dst_not_formal` exists to stop. The
  destination gate still runs (it is deferred, not removed), but **I will report
  `mismatch` from both arms as a first-class number** and treat any non-zero as
  an alarm, not a footnote. Nothing in this lane promotes the sink to a default.

## 5. The direction I expect to be wrong in

Board #770: estimates here have missed **optimistically eleven consecutive
times**. So the guard I register is the *pessimistic* one, and there are two
specific reasons it could fire:

1. **My +0 is a three-day-old transcription and the base has moved by 4,931
   census functions and 3 TU matches.** `w-op27` registered exactly this and was
   right to: its row had *shrunk* by 717 between the two bases, so the population
   under the key was not the same population. Mine has moved by **+1** on the
   function axis (6,820 → 6,821) — which is a much stronger stability signal
   than `w-op27` had, and is itself a finding to report either way.
2. **The emitted axis has never been measured for this row.** #407 and
   `assign-eof` both report the **function** census (`706,555 → 706,555`). The
   *emitted* census — 39,185 / 178,977, the one the goal is written in — was not
   printed by either. A row can be +0 on 2.46 M functions and non-zero on the
   178,977 that c2 actually emits. **If the emitted arm comes back > 0, that is
   NEW and I must not report it as "#407 reproduces".**

Third, and pointing the other way: I may be wrong that this is category 1
*rather than* 5. If `assign-store-type-8643` turns out to be reached from more
than one production the way `expr-op-0x27`'s single mint site is worn by many, my
"one mint site" claim understates it. §6.2 registers that count as mandatory and
reportable either way.

## 6. What I will do, in order

1. Base-evidence scan at `56912b72` — **done before this file was frozen**,
   `binary_sha 7d03c589ecc5c20fc1a5689538a899a0`.
2. **The category-1 grep, in BOTH directions, before the sink is written.**
   Every site in `crates/c2-il` that gates a TYPE in a *store* or *destination*
   position, against every site that gates one in an *operand* or *value*
   position: `eat_int_like`, `eat_int_like_or_ptr4`, `eat_operand_type`,
   `eat_value_type`, `is_int4_type`, `is_ptr4_kind`, `is_ptr_any`,
   `eat_reinterpret_type`. Does any copy refuse **more** than its siblings, and
   does any refuse **less**? Report both answers; `w-op27`'s #1334 found a real
   two-directional divergence on the neighbouring byte and it is the model.
3. The mint-site / production count for the key (§5's third direction).
4. **The counterfactual**: two full 878-TU `c2rs gap` runs of the **SAME**
   binary, `C2RS_SINK_STORE_TYPE` unset vs `=ptr4`, `--jobs 16`, `binary_sha`
   verified equal in both JSONL provenance records and quoted. A third arm at
   `=any` prices the family. Report TU match, mismatch, the emitted census, the
   recovered-vs-renamed split, the successor keys **by name**, and
   `negate_test.cpp` by name.
5. Re-run `w-lineage`'s five cells (`work/w-lineage/reach/mk.py`, unmodified)
   under both arms, as #1335 did, to say whether the cells that killed
   allocation keys ten and eleven are recovered or merely move again.
6. `work/w-splice/peerkeys.py` at both ends; report any key family that moved.
7. Apply §3.4. Decline or build.

## 7. What I am NOT going to do

* Not open `crates/c2-core/src/codegen/coff.rs` (never), `codegen/alloc.rs`,
  `crates/c2-harness/src/gap/`, `scripts/gate.sh`, `scripts/status.sh`, the
  *expression*-layer decode under `shapes/`, or `work/w-front3/ladder.py`.
* Not rename `assign-store-type-8643` or any published key spelling.
* Not promote the sink to a default.
* Not grade anything by disassembly — `w-lineage` had a change read 0 wrong of
  30 by disassembly and **11 of 30 `Port=Mismatch`** by the differential.
* Not report a `mismatch` from the sink arm as an expected artifact.

## 8. Board rows

Allotted **#1363–#1372**. Minted with the work; any left unused stay unminted
and are named as such in the rung doc. A row minted to fill a range is a trap
with a number on it.
