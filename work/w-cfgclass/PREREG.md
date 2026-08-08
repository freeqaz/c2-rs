# w-cfgclass — PREREGISTRATION

Lane `w-cfgclass`, worktree branch `wt-w-cfgclass` off master **`b234d826`**.
Committed **before the first line under `crates/`** and before any `cl.exe`
invocation that is part of the *build* (see §0 for what was already measured).

## §0 What was already measured, before this file existed — declared, not hidden

The lane's brief instructs it to pick a target off the live frontier, so the
**target-selection survey ran first** and is not covered by the predictions
below. It consisted of exactly:

1. one full 878-TU `c2rs gap` scan at master `b234d826` (`work/w-cfgclass/scan_base.out`);
2. `c2rs compile` + `scripts/gt_dump.py` on the eight smallest frontier TUs
   (`Main`, `osfinfo`, `undname`, `xlrcimpl`, `vswprnc`, `vsnprnc`, `jsonwriter`,
   `negate_test`);
3. one `c2rs census` and one `c2rs capture` on `src/system/negate_test.cpp`.

Nothing in §1–§4 is scored against a number that came out of that survey; every
scored prediction below is about **what building the class costs and whether it
converts**, which the survey did not touch.

## §1 The target

**`src/system/negate_test.cpp`** — a FRONTIER TU, 2 emitted functions, both
blocked, `cflow-if-n`, first blocker `assign-store-type-8643` on both.

Chosen over the other six single-function frontier TUs and over `Primes.cpp`
because, measured in §0:

* its two functions are **80 bytes each and byte-identical to one another**
  (`?FindNodeA@@…` and `?FindNodeB@@…`), so one class converts the whole TU;
* it is the only frontier `cflow-if-n` TU with **2 relocations** and no
  `REFHI`/`REFLO` pair, no `__savegprlr_N`, no `.data`, no callee-saved GPR
  beyond the frame, no 32-bit constant materialisation (board **#410** already
  recorded all six markers absent, and this lane re-confirmed it);
* the port already emits, byte-exact and oracle-graded, **most of its
  skeleton**: the 96-byte Class-A frame, `cmpwi cr6`+`bt/bf` guards between the
  prologue and the sequence (W10), the intra-section `b` and a real label→offset
  map (W11), and the entry-block park on the sub-class where the first early
  return anchors (#1444).

`Primes.cpp` is explicitly **not** the target: `w-loop` measured its first
refusal in `crates/c2-il` (`expr-jump`) and three of its five structural
refusals outside `codegen/`, at **≥ 13** remaining.

## §2 What the port must grow — the registered inventory

Read off the reference obj (§0 item 2). The 20 words of `?FindNodeA`:

```
mflr/stw/stwu -96          the shipped Class A frame
mr   r10,r3                (1) entry park: scrutinee evicted out of r3
mr   r3,r4                 (2) an argument move HOISTED above every branch
li   r11,0                 (3) the result home, initialised in the entry block
cmpwi cr6,r10,1            (4) ONE compare
bt   24,+32                (5)   read at LT ...
bt   26,+28                (6)   ... and at EQ — CSE'd across two source `if`s
cmpwi cr6,r10,2
bt   24,+12
bl   ?FindLast             (7) a call in the TAKEN-FALLTHROUGH arm
b    +8                    (8) an intra-section `b` to a JOIN, not the epilogue
bl   ?FindFirst            (9) a call in the other arm
mr   r11,r3                (10) the join: result home round-trip ...
mr   r3,r11                (11) ... immediately undone
addi/lwz/mtlr/blr
```

## §3 Predictions — scored in the rung, misses in their registered direction

**P1.** `negate_test.cpp` will read `≥ 8` independent refusals when re-derived
on this tree from its own obj and its own IL, i.e. board **#411**'s hand-count
of 10 will reproduce within ±2 and will **not** collapse to the 4 the
construct-reprice claimed. *(Registered direction of my own bias: I expect this
to come back DEARER, per board #770's streak — 11 optimistic estimates wrong.)*

**P2.** The **first** thing that refuses is in `crates/c2-il`, not in
`crates/c2-core`: `assign-store-type-8643` at IL byte offset 16 of body 0, the
`n = 0` store. No IL body reaches `select_function` today.

**P3.** The **intra-section `b` to a join block** (word 8) is the one structural
item the port has never emitted in any graded cell — W10 built the `else` arm,
graded it, and took it back out because `/Ox` tail-duplicates. At `/O1` — the
workload's mode — c2 emits the `b` and does **not** duplicate. So this shape is
`/O1`-only and any class I ship must refuse `/Ox`.

**P4.** The compare **CSE across two source `if`s** (words 5/6: one `cmpwi
cr6,r10,1` consumed at both the LT and the EQ bit) has no representation in
`Selected` and no encoder path in `codegen::calls`' guard emitter, which emits
one compare per guard.

**P5.** `?FindNodeA` and `?FindNodeB` emit **byte-identical** `.text` despite
different source (`!(b != k)` vs `b == k`), so a class that takes one takes
both, and the TU converts on one class or on none. **There is no partial-credit
outcome on this TU.**

**P6 — THE CONVERSION CALL.** I predict TU match **11 → 11**: this lane does
**not** convert `negate_test.cpp`. The registered reason is P1+P2 together — the
work is split across two crates and the reader half is a new statement-layer
production (`if`/`else` with a join and a call in each arm), which is the block
IR `docs/ARCHITECTURE_SEAMS.md` §7 says has never been sized.

**P7 — the success floor.** I predict the lane **does** land a fenced,
oracle-graded deliverable under `crates/`, and that it is a **transcription**
in the `codegen::ptr_walk_loop` tradition (a named function class, `/O1` only,
`NotImplemented` outside), not a general `if-n` lowering.

**P8.** mismatch stays **0**, codegen-gap stays **0**, and `scripts/gate.sh
--require-graded` passes with 18/18 lanes and 0 mismatch at the tip.

**P9 — REGISTERED BIAS.** If P6 is wrong it will be wrong in the *optimistic*
direction on the reader, not the emitter: I expect the emitter half to be
smaller than it looks (the frame, the guards, the label map and the park are all
shipped) and the reader half to be larger.

## §4 Decline clauses, with frozen thresholds

Evaluated in this order; the first that fires ends the build half of the lane
and the lane publishes the measured distance instead.

* **D1.** If the re-derived independent-refusal count for `negate_test.cpp` is
  **≥ 12**, decline the conversion and publish the inventory. (Board #269's
  standing clause is ≥ 4 and fires on every frontier TU; this lane's brief
  overrides it — *"do the work before declining"* — so the threshold is set at
  the point where the work is provably more than one lane, not at the point
  where it is non-trivial.)
* **D2.** If the reader production needed is not expressible as a **single
  pattern-matched token sequence** — i.e. if it requires a general basic-block
  IR with a value merge at a join — decline and publish, because #139 forbids
  splitting a lowering across the two crates and a half-built block IR is
  exactly the wrong-emit risk board #232 cost 241 commits.
* **D3.** If any fixture cell in the grid comes back `Port=Match` on a body
  whose bytes I have not compared word-for-word against real `c2`, stop: that is
  a fence that accepts something it was not graded on.
* **D4.** If `gate.sh --require-graded` reports a single `mismatch`, revert to
  the last green commit and publish the revert with its reasoning. A refusal
  becoming a wrong emit is strictly worse than a gap.

## §5 What this lane will NOT do

* Not adopt anything disassembly-derived. `docs/whitebox/` is navigation; if an
  adoption happens it carries its `DISCLOSURE.md` row in the same commit, and I
  register here that I expect **zero** such rows.
* Not touch `expr-op-0x27` (excluded by #150, eight confirmations) or spend on
  `assign-store-type-8643` as a *recovery* lever (#407/#1363 measured it at
  0 recovered of 1,133) — it is touched here only as the first gate in front of
  one named function class, never as a family.
* Not relax `codegen::labels` invariant 4. `cflow-if-n` is forward-only
  (board #1346), so the back-edge refusal does not fire on this shape at all.
