# PREREG — lane `w-instr`, the INSTRUMENT DEBT rung

Written before the first line under `crates/` and before `hatch.py` was edited.
Base: `56912b72`. Worktree `.claude/worktrees/w-instr`, branch `wt-w-instr`.

This lane ships no widening of the port's accepted class. Every item is a defect
in something that **decides where the project works**. Items are independent;
each is shipped or declined on its own evidence.

## §0 What I already hold at prereg time

Stated up front, because hiding it would make the predictions look better than
they are.

* **Item 1** — board **#1322** already states the defect and its cause:
  `store-run-bind-mixed-kind` was PAID between `503f8937` and `04727f37`, that
  edit is **last** in `EDITS`, so the other five clauses are already written to
  disk when `apply()` raises. I confirmed by grep before predicting: the string
  `addr_producer` **does not occur anywhere in `leaf_store.rs` on this master**,
  so the needle cannot match.
* **Item 2** — `w-front3`'s `ladder.py` §SKIP comment already names the site and
  the message: `calls.rs:71`, `index out of bounds: the len is 2 but the index
  is 2`. `calls.rs:485` is the sole caller of `permutation_cycles` (grepped:
  4 hits, 3 of them doc-comment references). The guard at `calls.rs:468` is the
  only thing that proves the input in range.
* **Item 3** — `w-cflowlabel` published the two-sided error and named the
  concrete gap positively: `0x05`/`0x06` (`/`, `%`) call `off_class()` on the
  same arm whose comment describes `div_mod_leaf`, shipped since and graded
  185/185. Base scan reproduces its five keys to the digit
  (192,495 / 518,991 / 83,776 / 38,227 / 9).
* **Item 4** — board **#1318** declined `0x4C` on the population. The standard
  is `w-divsplit`/#820: two independent confirmations from captures, one on the
  workload.

## §1 What I expect to find, per item

### Item 1 — hatch.py fails open

**Expect:** exactly ONE of the six clause edits fails to apply
(`store-run-bind-mixed-kind`), and it is last, so five are written before the
raise. The tree is left partially hatched with no record of which clauses are
live, and `revert` must be run by hand to clean it.

**The direction I expect to be wrong in:** I expect the drift to be **exactly
one** edit. I am more likely wrong toward MORE — `assign.rs`, `expr.rs`,
`sy.rs` and `calls.rs` have all been edited by lanes since `w-front3`, and a
second silently-dead needle would be the same defect twice. I will report the
per-edit verdict for all eight `EDITS` entries rather than the count.

**What I will ship:** two-phase apply — validate **every** edit first, write
**nothing** until all validate, refuse by name on the first failure, and print
the clause set positively on success. Plus a `verify` mode that says which
clauses are live in the tree right now.

### Item 2 — the `call-arg-outer-formal` panic

**Expect:** `permutation_cycles` panics because it indexes `seen[at]` with
`at = sources[at]`, an entry the guard would have proved in range. Making the
walk total (return `None` on an out-of-range entry) turns the crash into a
refusal, so the clause becomes liftable.

**The direction I expect to be wrong in:** I expect the lift to buy the ladder
**one** rung and immediately stop at the successor refusal I am adding — i.e.
the honest answer to *"what is the refusal now free to do"* is "very little,
the shape is genuinely not a permutation". I am more likely wrong toward the
lift revealing a *different* production key on `keygen_xbox.cpp` entirely,
because the panic was the first-blocker for one body and the TU has many.
**Registered before the lift:** if the successor key is my own new refusal,
that is a NEGATIVE result and I will report it as one, not dress it up.

**Explicitly NOT expected:** any change to a production count. The new refusal
is reachable only when `W_FRONT3_LIFT` names the clause, which never happens on
a committed tree, so `fn_blockers` and `emit_blockers` must move by **0 keys**.
That is a prediction, not an assumption — it is checked at both ends.

### Item 3 — `CfResidue` owes a PAIR

**Expect:** the 518,991 off-class side is dominated by a small number of
`off_class()` call sites, and the largest of them are shapes the port has since
learned. A bare widening is forbidden by #1345; what I will ship is the
**decomposition first** — which site sent each in-class body off-class — and
only then a widening of the sites the decomposition proves are the port's class,
published with the re-measured two-sided error beside it.

**The direction I expect to be wrong in:** I expect `0x05`/`0x06` (div/mod) to
be a **small** share of the 518,991 — under 5 % — because `div_mod_leaf` is one
leaf shape and the in-class population is 711,486 bodies dominated by tail
calls. The named gap is the one somebody already noticed, which is exactly the
kind of gap that is small. I expect the real mass to be somewhere nobody named.

**And the second direction I expect to be wrong in:** narrowing one side of a
two-sided error can WIDEN the other. `cflow-residue-straight-modeled-blocked`
(83,776) is the over-claim; every body I move into `Modeled` that the port
refuses lands there. I predict the over-claim **rises**, and if it rises by more
than the under-claim falls, the widening is a regression and I will say so and
not ship it.

**Every consumer of the `cflow-*` key family, named before the change**
(the shared-semantics rule — a git-silent collision):

| consumer | what it reads |
|---|---|
| `crates/c2-il/src/func/census.rs:324` `cflow_key` | produces the key |
| `crates/c2-il/.../control_flow.rs` `CfBody::key` | the string itself |
| `crates/c2-harness/src/gap/report.rs:82` `cflow_residue_control` | `+expr-modeled` ∩ in-class |
| `crates/c2-harness/src/gap/report.rs:112` `cflow_residue_overclaim` | `cflow-straight+expr-modeled` ∩ blocked |
| `crates/c2-harness/src/gap/report.rs:129` `cflow_emitted_counterfactual` | the emitted 38,227 / 9 |
| `crates/c2-harness/src/gap/factors.rs:1199-1221` | the five `gap-metric` keys |
| `crates/c2-harness/src/cli/gap.rs:661,694` | the printed block |
| `crates/c2-il/src/func/census.rs:989` | the `cflow-loop` in-class invariant |
| `docs/PHASE6_RANKING.md`, `docs/ROADMAP.md` §8.6, `docs/BOARD.md` #1343-#1347 | quote the numbers |

`peerkeys.py` is run at both ends and any key family that moved is reported.

### Item 4 — `0x4C` CALL-END

**Expect to DECLINE.** #1318's own decline is correct and the standard for
overturning it is two independent capture confirmations on the argument-bearing
population, one of them on the workload. That is a capture-grid rung, not a
tail-end-of-the-lane rung. **If I do not have the two confirmations, I decline
with the reason and do not ship a width.**

## §2 The predictions I will be graded on

| # | quantity | prediction | how it is settled |
|---|---|---|---|
| P1 | `hatch.py apply` edits that fail on this master | **exactly 1 of 8**, and it is `leaf_store.rs` | run the applier's own validator |
| P2 | files written before the raise, today | **≥ 1** (fail-open confirmed) | `git diff --stat -- crates/` after a failed apply |
| P3 | `fn_blockers` / `emit_blockers` keys moved by item 2 | **0 keys, 0 sum** | both-ends jsonl diff |
| P4 | div/mod's share of the 518,991 off-class in-class bodies | **< 5 %** | the new decomposition |
| P5 | the largest single `off_class()` site's share | **> 25 %**, and it is NOT div/mod | the new decomposition |
| P6 | `cflow-residue-straight-modeled-blocked` after any widening | **rises** | both-ends `gap-metric` |
| P7 | TU match / mismatch at tip | **11 / 0** | `c2rs gap`, both ends |
| P8 | `gate.sh --require-graded` at tip | **18/18 PASS, 0 mismatch** | the run |
| P9 | item 4 | **DECLINED** | this document |

## §3 The decline clause

Any item whose repair would require widening the port's accepted class is
**out of scope for this lane** and is declined with a measurement. This lane
ships instruments and one totality fix; it converts no TU and is expected to
move TU match by **0**.

## §4 Guards I will make go red on purpose

Each of these must be seen firing, with its verbatim message recorded, and each
refusal must **lead with its own word** so a later gate's refusal cannot satisfy
an earlier case's expectation (`w-throughput` had two of six mutations silently
pass for exactly that reason):

1. `hatch.py apply` with a drifted needle — must refuse naming the clause, and
   must leave `git diff -- crates/` EMPTY.
2. `hatch.py apply` with a needle that matches **twice** — a distinct message
   from (1), because "not found" and "ambiguous" are different defects.
3. `permutation_cycles` on an out-of-range source — must return a refusal whose
   key is its own word, and the test must fail if the walk panics.
4. The `CfResidue` decomposition's totality identity — the per-site counts must
   sum to the off-class total, and the control must fire when they do not.

An early guard can make a later assertion unreachable, so every mutation holds
the earlier quantities fixed.
