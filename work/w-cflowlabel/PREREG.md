# PREREG — lane `w-cflowlabel`, rung `body-cflow-label`

Written before the first line under `crates/` and before the first probe obj.
Base: `f49fe5e1`. Worktree `.claude/worktrees/w-cflowlabel`, branch
`wt-w-cflowlabel`.

## §0 What was already known before I predicted anything

Stated up front because it is evidence I hold at prereg time and hiding it would
make every prediction below look better than it is.

* `body-cflow-label` is **not a codegen key**. It is an **IL-decode** key emitted
  by `Block::feature()` (`crates/c2-il/src/func/body/mod.rs:1491`) when a
  straight-line production meets byte `0x29` (a label definition). It is the
  1:1 rename of `body-0x29`.
* This row **has already been re-priced once**, by lane `WCF`
  (`docs/rungs/2026-07-31-cflow-decode.md`): 48,102 bodies → **718 blocked on
  control flow alone** (67×), and on the emitted column 14,947 → **10** (5
  `cflow-if-1`, 5 `cflow-switch`). That is the re-ranking my brief cites as the
  project's most valuable single output. My rung is the same row.
* The base scan reproduces the counterfactual **unchanged to the function**:
  `cflow-if-1+expr-modeled` **713** + `cflow-switch+expr-modeled` **5** =
  **718**, eight days and a +45 % census (491,013 → 711,486) later.
* The instrument that produces it is `CfResidue`
  (`crates/c2-il/src/func/body/shapes/control_flow.rs:131`), whose `Modeled` arm
  is a **hand-written vocabulary** — `is_int4_type`/`is_ptr_to_4`, the `+ - *`
  chain, `26`, the call quadruple — mirroring the port's class *as of
  2026-07-31*. `0x05`/`0x06` (`/`, `%`) call `off_class()` although
  `div_mod_leaf` has since shipped and is graded 185/185.

## §1 Which of the five categories I expect

**Primary: (1) — a private limit inside a recognizer that already exists**, at
one level up from where the brief expects it. Not a limit in an *emitter*; a
limit in the **counterfactual instrument** that prices this rung. `CfResidue`
is that recognizer and its vocabulary is frozen.

**Secondary: (3) — real but far smaller than its size.** The 67× / 1,495×
deflates are real and I expect them to survive re-derivation in kind, if not in
magnitude.

Explicitly **not** expected: (2) misfiled production — the byte is `0x29` and
four shipped productions (`cond_tail`, `early_return`, `guarded_seq`,
`ptr_walk_loop`) already eat it, so it is not misfiled. Not (5) mis-described —
`body-cflow-label` is exactly a label definition met by a straight-line parser.

## §2 The prediction I will be graded on

The control the rung turns on: **for what fraction of the port's own in-class
bodies does the residue predicate say `+expr-modeled`?** If `Modeled` really
were "the port's class", it would be ~100 %.

| # | quantity | prediction | how it is settled |
|---|---|---|---|
| P1 | in-class bodies whose cflow key **lacks** `+expr-modeled` | **> 0, and I estimate 55–70 %** of 711,486 | base scan cross rows |
| P2 | corrected counterfactual (bodies), if the vocabulary were widened to today's class | **2×–6× of 718**, i.e. 1,400–4,300 | bracket, §3 |
| P3 | corrected counterfactual as a share of the 14,990 emitted row | **still < 2 %** | bracket, §3 |
| P4 | TUs this rung converts at any width | **0** | `c2rs gap`, both ends |
| P5 | frontier TUs made CFG-**reachable** by a `cflow-loop` class | ≤ 7 of 16, and **0 conversions** | scan's own CFG-reachability block |
| P6 | `mismatch` at tip | **0** | `gate.sh --jobs 16 --require-graded` |

## §3 The ceiling, taken neat

The rule is: count the **independent** refusals between the ceiling (14,990) and
a converted TU, and take the ceiling neat rather than discounting it.

Refusals for the `cflow-loop` half of this row (the 36,871-body
`cflow-loop|body-cflow-label` cell):

| # | refusal | independent of the others? |
|---|---|---|
| R1 | **No loop representation.** `Selected` has no variant with a back edge; no IL production accepts a general loop (`ptr_walk_loop` is a twenty-word transcription of one class). | yes |
| R2 | **The compiler-label counter.** `coff::plan_labels` charges 0 where c2 charges +1..+4 for a back edge (17 seed-free cells); `labels.rs` invariant 4 refuses every backward branch and `IlFunction::label_slots` returns `None` for the loop shape. | **ONE refusal, not three** — the three sites are one variable (the control-flow surcharge) at three thresholds. |
| R3 | **Register allocation across a back edge.** Behind the frame/liveness spine. | yes |
| R4 | Each frontier TU's own price: `w-front2`/`w-heap` measured **min 5, second-cheapest ≥ 7** independent refusals *per TU*, and the standing decline clause (≥ 4) fires on every one. | yes |

**Four independent refusals. The decline clause fires.** So P4 = 0 and I take
14,990 neat rather than discounting it to a smaller "realizable" number.

## §4 The direction I expect to be wrong in

**I expect to under-estimate the staleness**, i.e. P1 to come in *above* my
70 % ceiling and P2 to come in *above* 6×. The reason is structural and I would
rather record it than be credited for it later: every widening of the port since
2026-07-31 (floats, member designators, intrinsics, `div_mod_leaf`, rotate,
ptr-walk, compares) sets `off_class()` in this file, and I have not counted them,
only sampled two. Ten consecutive optimistic misses (board #770) argue the other
way — that I am over-estimating what widening buys — and P2/P3 are deliberately
written so that being wrong in *either* direction still leaves P4 = 0. **P4 is
the prediction the rung stands or falls on and it is the pessimistic one.**

## §5 What I intend to ship, and what I will not

**Will not:** widen `CfResidue`'s vocabulary. `crates/c2-il/src/func/body/shapes/`
is lane **w-op27**'s file. Widening it is the correct repair and it is not mine
to make; I will file it with the measurement attached.

**Will not:** open `crates/c2-core/src/codegen/coff.rs`. Hard stop per brief.

**Will not:** relax `labels.rs` invariant 4 (forward-only). R2 above is live and
the module's own header already states where a loop rung's relaxation belongs
(`IlBundle::functions`' gate), which is not here.

**Intend to ship:** the **control that would have caught this**, in the harness —
a standing count of in-class bodies the residue predicate calls off-class,
printed as a `gap-metric` beside the counterfactual, with a portable unit test.
This is "publish the denominator, do not adjust the control" (STATUS trap 0,
lane `w-inread`). If the numbers below do not support it, I ship nothing and say
so.

## §6 Shared predicates I touch

* `CfResidue::Modeled` / `CfBody::key()` — **read only**, never widened.
* `FnVerdict::in_class()` / `FnVerdict::key()` — read only; the cross already
  exists at `scan.rs:131`.
* `GapResult::fn_cflow` — **widened, not narrowed**: new keys added, no existing
  key renamed, merged or removed. Any lane reading `cflow-*` or `cflow-*|*` sees
  byte-identical values.
* `work/w-splice/peerkeys.py` at both ends; any key family that moved is
  reported.
