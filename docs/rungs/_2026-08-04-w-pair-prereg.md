# Pre-registration — lane w-pair: both sides of the seam, aimed at TU match

Committed **before the first line of implementation code** and **before any
probe obj is compiled**. Base: `wt-w-pair` branched from master `3457c9f`.

The lane owns `crates/c2-il/` **and** `crates/c2-core/` together, because lane
w-front demonstrated that a shape widening is a paired change and a crate-split
lane converts zero TUs by construction: all 17 frontier TUs report
`il function decode failed`, so `select_function` is never reached and
`fn_gate_refusals` is empty across all 878 TUs.

---

## 0. What I have already measured, stated so it is not mistaken for a result

Disclosed because this prereg is not written from a blank page. Everything in
this section is **orientation**, taken before any implementation decision, and
none of it is a prediction:

* Baseline `c2rs gap` over the 878-TU workload, reproduced in this worktree:
  `match 8, mismatch 0, codegen-gap 0, vocab-gap 863, port-error 0,
  capture-fail 7`, `FRONTIER 17`, factors `A 28 / B 338 / C 114 / D 8`,
  control `A 0 B 0 C 0 D 2` (red on purpose, board #179).
* The frontier re-derived from that run rather than transcribed, and
  `c2rs census` run on **each of the 17** TUs. Exactly two are straight-line
  only: `src/Main.cpp` and `src/xdk/nuispeech/xboxheap.cpp`.
* `xboxheap.cpp` under the `C2RS_SINK_OFF_ADD_ARG=expr` counterfactual: the
  blocker moves `expr-op-0x27` → **`expr-op-0x32`**. It is not one change away.
* The `.ex` segment of `??0CXboxHeap@NUISPEECH@@QAA@II@Z` (404 B, one segment)
  hand-decoded, and the `/FAsc` listings of **both** `xboxheap.cpp` and
  `Main.cpp` at the workload's flags.

The listings are the reason this prereg is shaped the way it is, so the two
facts they establish are stated here as **read**, not as predicted:

* `Main.cpp` needs the **full C++ EH apparatus** — `__CxxFrameHandler` and
  `__ehfuncinfo$main` in `.rdata`, an `__unwind$2585` funclet with its own
  prologue and its own `.pdata` COMDAT (two `.pdata` COMDATs in one obj), a
  stack-homed local object, and three framed `bl`s. That is the EH critical
  path, not a shape widening.
* `xboxheap.cpp`'s constructor is a **scheduled** body. The six stores come
  back in source order, but the three value-producing instructions are
  interleaved between them at slots 0, 2 and 5 of a nine-instruction run.

---

## 1. Bias, declared

**My bias is to convert `xboxheap.cpp`.** It is the lane's assigned cheapest
target, it is one 404-byte segment, its EH class is `eh-none`, and a lane that
returns "1" on the project's payoff metric is worth more than one that returns
"0". That wanting will push me to (a) read the interleave above as decoration
rather than as the emission, (b) treat "the stores are in source order" as
"the body is in source order", and (c) accept a placement rule fitted to the
one function I have read.

Mitigations committed now: every placement rule is tested against probe cells
**other than** `xboxheap.cpp` before it is believed; the known-answer control
P4 below runs first and voids the grid if it fails; and the decline clause in
§4 is priced before any cell is compiled.

---

## 2. The one question this lane turns on

Everything else `xboxheap.cpp` needs — a `27` designator in a store statement,
a mixed literal/formal store run, a `26` local-reference bind, a trailing
framed call in a body that also stores, a constructor `return this` out of r31
— is an ordinary paired rung. The **placement of the setup instructions** is
not: `docs/…/leaf_store.rs` already records four allocation rules fitted to
this family and **each refuted by another cell** of the same grid, under
`GAPS.md` §6 instance #10 ("measure at the edge, do not fit the scheduler").

So the lane's question is: **is the placement derivable, or is it a scheduler?**

## 3. Frozen predictions

Incumbents named, not thresholds.

| # | prediction | refuted by |
|---|---|---|
| **P1** | In every probe cell, the **stores** come back in source order. The setup instructions move; the stores do not. | any cell whose store offsets are emitted out of source order |
| **P2** | In at least one cell that is **not** `xboxheap.cpp`, a setup instruction is hoisted **more than one slot** above its consuming store. (I.e. the interleave is general, not an artefact of one function.) | every non-xboxheap cell emitting each producer immediately before its consumer |
| **P3** | **No** single position rule reproduces every cell. The three rivals registered now, all of which reproduce `xboxheap.cpp` itself: (R1) all producers first, in first-use order; (R2) each producer immediately before its first consuming store; (R3) producers and stores strictly alternate while producers remain. | any of R1/R2/R3 reproducing **every** cell — in which case P3 is refuted, the decline in §4 is void, and I implement it |
| **P4** | **Known-answer control.** A pure formal-valued store run of length 3 — the shape `try_parse_store_run` already accepts and the port already grades byte-exact — emits three stores in source order and **no** setup instruction at all. | anything else; and if it fails, the whole grid is void and I report the harness as broken rather than reporting cells |
| **P5** | **Incumbents hold, unchanged, at end of lane**: `cargo test --workspace --release` = 677 passed / 0 failed / 25 targets; `scripts/gate.sh --jobs 6` = 12/12 PASS, 2,592 verdicts, 0 mismatch; `c2rs selftest` = 216 PASS 0 FAIL; `c2rs gap` = match 8 (or higher), mismatch 0, codegen-gap 0, vocab-gap 863 (or lower), capture-fail 7, FRONTIER 17; `cargo build` 0 warnings. | any of them moving in the wrong direction |
| **P6** | `Main.cpp` does **not** convert in this lane and I do not attempt it. Recorded as a prediction rather than omitted, so its absence cannot later read as an untried option. | — |

**P1 and P2 are not evidence for the decline on their own.** Only P3 is. A count
of cells that "look scheduled" is a count about my eyes, not about a predicate;
the rivals R1/R2/R3 are registered *by name* so that the grid tests them rather
than my impression of them.

## 4. Priced decline clause

**If P3 holds — no registered rival reproduces every cell — I stop.** I land no
codegen, I convert zero TUs, and I say "zero" plainly.

What I deliver instead, and it is priced as the lane's output rather than as an
excuse:

1. The frontier re-derived from my own scan, with a per-TU census of all 17 and
   the *second* blocker for every TU I unblocked counterfactually.
2. The byte-level boundary for both straight-line-only TUs, from the real
   `/FAsc` listings, naming every production each still needs.
3. The probe grid, its cells, and which registered rival each cell refutes —
   as data, so the next lane does not re-fit the same four rules a fifth time.
4. Proposed board rows (I do not edit `BOARD.md`, `ROADMAP.md` or
   `rungs/INDEX.md` this session).

**If P3 is refuted**, the decline is void: I implement the surviving rule, and
the conversion is graded only by the differential — byte-exact against real
`c2.dll` under wibo with `TimeDateStamp` zeroed, and nothing else counts.

## 5. What I will not do

* Build a CFG/block IR. Lane w-cfg owns that spec; 15 of the 17 frontier TUs
  sit behind it and pre-empting it would be a second w-front.
* Widen the gate to make anything pass. Outside the class `PortC2` returns
  `NotImplemented`; that boundary is the open gate, not a defect.
* Add any neutrality / behavior-preserving classifier. The compiler is the sole
  judge.
* Grade anything on `Pool.cpp` (its `if-1` bodies emit no branch and its
  constructor is `cf-expr-0x05`, which is a DIV width refusal and not control
  flow at all).
