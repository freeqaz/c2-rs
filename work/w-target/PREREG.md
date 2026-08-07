# w-target — PREREG

    Lane:      w-target, worktree branch `wt-w-target`
    Base:      master `217d4a85`
    Rung:      board #1013 — the 861 `fnbyte-reloc-differs` bodies whose bytes
               are exact and whose relocation names the wrong function
    Written:   BEFORE the counterfactual reach existed, before one line of
               emitter code, and before any GRID-W cell was compiled.

Everything below is registered in advance. Numbers that were *already published
by a prior lane* are marked as such and are NOT claimed as this lane's
predictions; they are the incumbent state this lane must beat.

---

## 0. The incumbent, stated exactly — because a floor without a baseline cannot
## tell an improvement from a regression

Re-derived on this worktree from a full 878-TU scan at the lane's base
(`work/w-target/base_metrics.txt`), not quoted from the brief:

| | |
|---|---:|
| `fnbyte-exact` | **35,986** |
| `fnbyte-reloc-differs` | **861** (all `-target`; `-count`/`-offset`/`-type`/`-section-target` all **0**) |
| `fnbyte-exact-bytes` | **36,847** |
| `fnbyte-differs` | **2,334** |
| `fnbyte-partial` · `-refused` · `-unbound` · denominator | 0 · 130,579 · 9,217 · 178,977 |
| `fnbyte-shape-tail-exact` / `-tail-reloc-differs` | 4,949 / 845 |
| `fnbyte-shape-seq-exact` / `-seq-reloc-differs` | 1,227 / 16 |
| TU match · mismatch · vocab-gap · capture-fail | 10 · 0 · 861 · 7 |
| factor A/B/C/D/E · `B∧C` · `A∧B∧C` · FRONTIER | 28/338/169/10/2 · 151 · 27 · **17** |

**THE INCUMBENT IS "DO NOTHING", AND ITS SCORE IS `(conversions 0, regressions
0)`.** Any rule this lane could ship is scored against that pair and against
nothing else.

> **The FRONTIER is 17 on this base, not 16.** The brief quotes 16 from a commit
> subject; `docs/STATUS.md` line 398 and this lane's own scan both read **17**.
> Recorded here so a later reader does not treat the difference as this lane's
> motion.

---

## 1. THE DECLINE FLOOR, stated against the incumbent

A rule ships only if, verified **per `(TU, emit_name)`** on the full 878-TU scan:

| # | condition | against the incumbent |
|---|---|---|
| **F1** | **conversions ≥ 1** — at least one function moves `reloc-differs → exact` | incumbent scores 0; a rule that converts 0 is strictly not better |
| **F2** | **regressions = 0** — **zero** functions move out of `exact` in any direction, and zero move `exact → reloc-differs` | incumbent scores 0; **any** positive number here is strictly worse |
| **F3** | `fnbyte-differs` does not grow; `mismatch` stays 0; gate 18/18 | incumbent holds all three |
| **F4** | conversions and regressions are counted from the **per-`(TU, emit_name)` join of two scans**, never by subtracting corpus totals | w-splice proved subtraction cannot distinguish "disjoint" from "two lanes fighting over the same functions" |

### 1.1 UNCONDITIONAL STOP — registered exactly as `w-drop3` registered it

> **If `fnbyte-exact` shrinks by one, or `fnbyte-reloc-differs` grows by one, the
> rule does not ship.** No widening of the stop, no "net positive" arithmetic, no
> trading a regression against a conversion. `w-drop3` §6 declined on this stop
> and `w-drop3` was right to.

This stop is *stronger* than F2 needs to be, deliberately: `fnbyte-exact` is the
project's credited count and a rule that moves it down while moving
`reloc-differs` down is not a repair, it is a re-labelling.

### 1.2 The second stop — the ~6,176 control class

`fnbyte-shape-tail-exact` **4,949** + `fnbyte-shape-seq-exact` **1,227** =
**6,176** functions are `exact` **because c2 did not inline their callee**. A
rule that names a different relocation target must fire on **none** of them.
This is the population w-drop3 named as making its escape hatch unavailable, and
it never built the count. **This lane builds the count first and lets it decide.**

---

## 2. THE PARTITION PREDICTION for the 861 under the closure hypothesis

The brief's leading hypothesis is that c2's relocation target is the transitive
closure of the port's target under the elision/inline relation, and that the port
stops at one level — from w-splice's 150 workload witnesses, 145 of them
`??1length_error` → `??1__Named_exception` where c2 writes `??1exception@std@@`.

**`w-relo` §4.1 ALREADY PUBLISHED THIS PARTITION and this lane re-derived it from
its own baseline scan; it reproduces to the digit.** It is therefore recorded
here as *prior* state, not as a prediction:

| n | family | closure-reachable? |
|---:|---|---|
| 529 | `tail\|local->local\|blocked` | **NO** — the port's own target is a parse-refused body; the closure cannot be evaluated at all |
| 169 | `tail\|local->local\|unrelated` | **NO** — edges existed and none reached |
| 73 | `tail\|local->local\|chain2` | yes, depth 2 |
| 69 | `tail\|local->local\|chain1` | yes, depth 1 |
| 16 | `seq\|local->extern\|chain1` | yes, depth 1 |
| 3 | `comdat-only` ends, `blocked-unbound` | no |
| 2 | `tail\|local->comdat-only\|unrelated` | no |

> ### **CLOSURE-REACHABLE = 158 of 861 = 18.4 %. The hypothesis is TRUE OF UNDER A FIFTH OF THE QUEUE, and 532 (61.8 %) are not even answerable.** This is registered here so that a rule which converts 158 is not later read as having solved 861.

**P1 (registered prediction, this lane's own).** The counterfactual reach of a
closure rule on the **exact** population — the number of currently-`fnbyte-exact`
`tail`/`seq` functions whose relocation target a depth-≥1 closure rule would
CHANGE — is **> 158**, point estimate **1,200**, interval **200 … 4,000**.
Reasoning: a tail call to a same-TU callee that itself calls something is the
commonest shape in this corpus, and c2 inlines only some of them; there is no
reason for the firing set to be confined to the ones where c2 happened to inline.

**P2.** If P1 holds, **R-CLOSE as specified in §3 is DECLINED**, because every
firing on an `exact` function is a §1.1 stop.

**P3.** The 158 are dominated by **one or two template roots** (#925/#952) — I
predict the largest single root is ≥ 50 % of the 158.

---

## 3. THE RULES, specified before they are measured

### R-CLOSE — the brief's hypothesis, as an emitter rule

> For an emitted `tail` or `seq` function `F` whose relocation names a same-TU
> callee `P` bound by `FnCensus::emit_name` (#918), if the port's splice walk
> from `P` reaches a link `L` that does not splice, emit `L`'s own call target
> in place of `P`.

Scored by its **counterfactual reach**, measured additively in the harness
*before* any emitter change:

* **`reach-convert`** — currently `reloc-differs` functions where the rule's new
  target equals c2's target. A win.
* **`reach-regress`** — currently `exact` functions where the rule would change
  the target at all. **Every one is a §1.1 stop.**
* **`reach-null`** — the rule does not fire.
* **`reach-wrong`** — currently `reloc-differs`, the rule fires, and the new
  target is still not c2's. Not a win and not a regression, but it is printed,
  because a rule that fires and is still wrong is not "no change".

### R-REFUSE — the alternative w-drop3 §6.1 asked for and did not build

> Refuse to emit any `tail`/`seq` body whose call names a same-TU callee the
> port's parser refuses.

Scored by the **same** counterfactual:

* **`refuse-convert`** — `reloc-differs` functions the refusal removes.
  These leave `exact`'s complement honestly, and `reloc-differs` falls.
* **`refuse-regress`** — **currently `exact` functions the refusal would remove.**
  w-drop3 §6.1: *"If that number is 0 the rule is free … if it is positive the
  rule is forbidden as written."* Registered as a §1.1 stop.

**R-REFUSE is registered as an option and NOT as a preference.** It lowers
`reloc-differs` by removing functions rather than by pointing them correctly,
and if it ever ships it must be described that way.

### 3.1 What this lane will NOT do

1. **No inline decision procedure.** `INLINE_PREDICATE.md`'s `INLINE-P` is 0.9716
   with a 2.84 % residual §7 leaves NOT MODELLED. 2.84 % of a guess is a wrong
   emit. `w-empty` §2.2 and `w-drop3` §6 declined on exactly this ground; so does
   this lane, in advance.
2. **No RTTI machinery.** Seven independent refusals, priced by `w-rdata` and
   re-derived unpaid by `w-rtti`.
3. **No template-name key.** No rule, key or test in this lane may mention a
   template spelling. (`w-drop3` `PREREG.md` §5.1, inherited whole.)
4. **No narrowing of `IlBundle::functions()`** and no widening of it. `mismatch`
   stays 0 by construction.
5. **No edit to `crates/c2-harness/src/gap/fnbytes.rs` SEMANTICS.** The
   counterfactual keys are strictly **additive** — new `emit` entries in the walk
   that already holds every input (the anti-duplication choice: the reader over
   this fact exists and adding a second one in a new file is the failure mode the
   brief names). Every pre-existing key must be byte-identical at both ends,
   checked by `diff` of sorted metric files and by `work/w-splice/peerkeys.py`.

---

## 4. BOUNDARY CELLS — GRID-W, frozen before compiling

A `sha256` manifest of every cell is committed **before** the first cell is
compiled. Cells are compiled by the real toolchain and graded against real
`c2.dll` under wibo; nothing here is hand-written expected output.

| cell | what it is | why |
|---|---|---|
| `w01` | chain depth 1, c2 closes | the `chain1` family's mechanism |
| `w02` | chain depth 2, c2 closes | the `chain2` family's mechanism |
| `w03` | chain depth 3, c2 closes | does the closure keep going, or stop at 2 |
| `w04` | **a chain where c2 does NOT close** — the intermediate is too big / not inlinable | **the control that decides the whole lane.** If this cell does not exist the rule is free; if it does, the rule needs a discriminator the port cannot read |
| `w05` | a callee c2 did **not** inline — the ~6,176 control class in miniature | the rule must leave it alone |
| `w06` | a cycle (`f` → `g` → `f`) | termination |
| `w07` | the next link is **external** | the `local->extern|chain1` family's 16 |
| `w08` | the next link has **no census row** in this TU | w-splice's `S6-chain-open`, which cost it 1 wrong of 4 firings |

**`w04` is registered as the cell most likely to kill the lane, and finding it is
a deliverable in its own right.** If c2 declines to close some chains, then
"close the chain" is not a rule, it is a guess with an unmeasured error rate.

---

## 5. THE MOST-EXPECTED LOSS

That **P1 is wrong in the safe direction** — i.e. `reach-regress` comes out **0**
and R-CLOSE is free. I do not believe it; I am registering it so that if it
happens the surprise is on the record and not retro-fitted. If `reach-regress` is
0, R-CLOSE ships and the lane converts up to 158.

The second most-expected loss is that **`reach-convert` < 158** even where the
closure is reachable — because the port's splice walk and c2's inline walk stop
in different places, which is precisely what `S6-chain-truncated` exists to hedge.

---

## 6. A DECLINE IS A SUCCESSFUL OUTCOME OF THIS LANE

Registered in advance so it cannot be reframed afterwards as a failure. If
`reach-regress > 0` and `refuse-regress > 0`, this lane ships **the two numbers
w-drop3 §6.1 and §10.1 asked for and did not build**, the GRID-W cells, and a
registered decline — and no emitter change. That is the deliverable, and the
numbers are the thing that makes the next lane's decision cheap instead of
speculative.
