# PREREG — lane `w-mass`

Committed **before the first probe**. Base tree `b027eaad`, worktree
`.claude/worktrees/w-mass`, branch `wt-w-mass`.

## 0. The question

`vocab-gap` is 860 of 878 and `codegen-gap` is 0: every non-matching capturable
TU fails at the IL decoder. The brief is to **rank by emitted-function mass**
(`emit_blockers`, 648 keys summing 130,575) and take **the largest family that
survives its own counterfactual**.

## 1. The trap this lane is built against

A first-blocker key is a **NAME, not a distance**. Five lanes have been burned by
dispatching off a blocked-key size ranking. Therefore:

* the unit ranked is a **production family**, not a key — a key is where the walk
  stopped, a family is the construct;
* **the counterfactual runs before any implementation**, and the family is
  chosen by its *measured* yield, not by its mass;
* a **rename** (the refusal acquires a successor name at the same stream
  position, buying zero decode distance — board #1465) is reported separately
  from a **recovery**, and both are published even if the answer is zero.

## 2. Board check — done before the arms were chosen

| family | emit mass | board verdict | admitted? |
|---|---:|---|---|
| `expr-call-in-expr-*` | **36,751** (28.1 %, 503 keys) | #1428 / #1453 / #1455 / **#1456 "Do not rank this row"** apply to the `…-op-0x5C` sub-row only (1,212 fns). The family as a whole has no decline row. | **YES**, as ARM-A |
| `expr-intrinsic-*` | **25,563** (19.6 %, 22 keys) | #127 DECLINED `this-adjust` at **+472** emitted (any offset) / **+434** (offset 0); #140 *"Schedule it at 434, not at 8,790"*. Both were taken at a far smaller emitted census and **neither is a family-wide counterfactual**. | **YES**, as ARM-B |
| `expr-op-0x27` | 22,373 (17.1 %) | #150 with **eight** confirmations, #441 (a fixed point of the sink operation), #1105, ROADMAP §9.14.5/§9.17. **BANNED.** | **NO — not probed, not ranked** |
| control flow (`body-cflow-label`, `expr-br{false,true}`, `return-scope-close-cflow-label`, `expr-jump`, `expr-ternary`) | **22,217** (17.0 %, 6 keys) | `expr-brfalse` carries #150's seven confirmations at the *key* level; the family's own poison (`expr-branch-sink-poison`) has never been read over the workload's **emitted** population. | **YES**, as ARM-C |

`assign-store-type-8643` (#407, *"Do not rank this row"*) is not in any arm.

## 3. The arms — frozen

All arms are **poisoned, measurement-only sinks that already exist on master**.
None pushes an `IlOp`; a walk that reaches the end having used one refuses anyway.
**No arm can move an obj byte**, so no arm can manufacture a wrong emit.

| arm | env | family |
|---|---|---|
| ARM-0 | *(none)* | baseline, `work/w-mass/base.jsonl` |
| ARM-A | `C2RS_SINK_CHAIN=op:26` | `expr-call-in-expr-*` head token |
| ARM-B | `C2RS_SINK_CHAIN=intrinsic` | the class-layout / intrinsic family, **whole two-token production** |
| ARM-C | `C2RS_SINK_BRANCH=cflow` | the control-flow family |

**ARM-A is registered as a WEAKER instrument than ARM-B and this is written down
before it runs**: `op:26` consumes `26 <tok>`, one token of a multi-token
`26 … 4C` production, so it is a *head-token* sink and not a production sink.
ARM-B's `intrinsic` arm consumes `33 <int> <id> 40 <TYPE>` — the entire unit.
If ARM-A measures near zero, that is **not on its own** a refutation of family A;
it is a refutation of the head token, and the rung must say so.

## 4. Metrics, per arm, on BOTH populations

`emit_blockers` (emitted functions — the ranking column the brief names) **and**
`fn_blockers` (all bodies). Reported side by side; never summed (#1476).

* **`recovered`** — functions that become in-class (`emit-in-class`, census
  numerator). Under a poisoned sink this is **structurally 0**; it is measured
  and published anyway, because a nonzero would mean the sink is unsafe.
* **`recovered-proxy`** — functions whose first-blocker key becomes the arm's
  poison key (`expr-chain-sink-poison` / `expr-branch-sink-poison`). This is an
  **UPPER BOUND of unknown tightness**: the poison fires when *the expression
  walk that used the sink* reaches its stop, and a body may have further
  statements behind it. It is quoted with that caveat every time.
* **`renamed`** — functions that left a family key and landed on a non-poison
  successor. **Successor keys are named**, top 15 by size.
* **partition control (R3)** — Σ(negative key deltas) must equal Σ(positive key
  deltas) to the unit, as w-5c2's identity did. Also `fn_blockers`/`emit_blockers`
  sums must be unchanged (1,751,957 / 130,575), because a poisoned sink relabels
  and never converts.

## 5. Predictions — frozen, one per arm, before the first probe

`recovered-proxy` on the **emitted** population:

| arm | prediction |
|---|---|
| ARM-A (`op:26`) | **< 200** |
| ARM-B (`intrinsic`) | **400 – 1,500** |
| ARM-C (`cflow`) | **800 – 3,000** |

I expect ARM-C to be the largest and ARM-A the smallest. The family ranking by
mass (A > B > C) is predicted **not** to survive its own counterfactual — that
prediction is the lane's thesis and it is registered here so a hit is not
retro-fitted.

## 6. The decision rule — frozen

**R1 — the winner.** The winning family is the one with the largest
`recovered-proxy` on `emit_blockers`. If the winner's `recovered-proxy` is
**< 750 emitted functions AND < 5 % of its own family mass**, then **DECLINE**:
publish the zero, mint the board row, write the rung, stop. No implementation.

**R2 — shipping.** If R1 passes, narrow to the sub-family that carries the proxy
and ship only if all of:
 (a) the acceptance lives in the **IL parser** (#139) — if it needs a
     codegen-side acceptance, say so and do **not** split it;
 (b) `scripts/gate.sh --require-graded` PASSes: 18/18 lanes, 0 mismatch;
 (c) `mismatch` stays **0** and `fnbyte-differs` does not rise. A refusal turned
     into a wrong emit is strictly worse than a gap (#232).

**R3 — instrument honesty.** If the partition control fails on an arm, that
arm's number is **withheld**, not adjusted.

**R4 — no goalpost move.** If ARM-B or ARM-C passes R1 but its yield is behind a
frame class or a block IR (i.e. not expressible in the IL parser), the outcome is
**DECLINE under R2(a)**, and the measured number is published as the price rather
than converted into a smaller rung to claim a ship.

## 7. Scoring

Every prediction in §5 and the §6 outcome is scored in the rung's §1, with
misses published in the direction they were registered (board #770's streak).
