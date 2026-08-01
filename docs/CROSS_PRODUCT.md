# CROSS_PRODUCT — grading the combinations, not the axes

`scripts/cross_sweep.sh` (driver: `scripts/cross_sweep.py`). Written 2026-07-31;
converted to the `scripts/lanes.txt` registry and re-keyed on measured emitted
shape 2026-08-01.

## Why

`docs/GAPS.md` §6 #12. Two branches were each fully green — an FP-store rung and
a many-call framed rung — and the **merge** mis-emitted: the compiler-label
counter is a per-TU quantity and it was being read from a per-function method,
so a framed function downstream came out six bytes wrong in an obj that still
links. **Neither branch's corpus could contain the case.** The label counter has
an observable effect only when a framed function follows, and until Class A
many-calls landed there was no framed shape that could share a TU with an FP
store; the FP rung's fixtures have no framed function and the framed rung's have
no floating point. #13 then found that the *repair* was also wrong one row
further out, because a per-function quantity and a per-TU one are
indistinguishable at n = 1.

The rule those two wrote down is:

> A merge of two independently-green branches is a **new corpus**, and the shapes
> only it contains have never been graded by anyone.

Until this lane, applying that rule was manual — it depended on whoever was
merging remembering to compile the cross product. `scripts/expr_sweep.sh` grows
**additively**: each fragment varies its own parameters inside one shape family,
so 24 fragments give 24 independently-swept axes and **zero** graded
combinations beyond whatever a fragment happened to put in one file. This lane
grows the corpus **combinatorially** instead.

## How the families are enumerated (and why not by hand)

A hand-written list of families drifts the moment a rung adds one, and a lane
that silently under-enumerates reports full coverage of a subset — the exact
failure `GAPS.md` §6 keeps recording. So:

1. **The families come from the port.** They are the `FnVerdict::InClass("…")`
   labels in `crates/c2-il/src/func/census.rs`, extracted by a paren-matched
   scan of each call's whole argument. Not a line-wise `grep`: several of the
   twenty-eight (`call-sequence*`, and `float-leaf`/`double-leaf`) live inside a
   nested `match`/`if` and a line pattern misses them.
2. **The representatives come from compiling, and they are keyed on SHAPE, not
   on the family's name.** The whole `scripts/sweep.d/` corpus is generated and
   graded; a *candidate* is a matched TU whose in-class functions are all of one
   family; every candidate's obj is then emitted and its **shape** read out of
   it. A family's representatives are one per *distinct shape*, most-populated
   bucket first, smallest source within a bucket, capped at `C2RS_CROSS_REPS`
   (default 8).

   **Shape here is measured, and it is the masked opcode sequence of the emitted
   `.text`** — primary opcode plus the extended field where the extended field
   is the instruction's identity, with registers and immediates removed. Two
   cases are the same shape iff the same instructions come out in the same
   order. That is the only sense of "same shape" this project judges by
   (`CLAUDE.md`: the obj is the sole judge), it needs no agreement from the port
   about how it labels anything, and it moves the moment a rung emits an
   instruction it did not emit before. Masking the operands is the deliberate
   cut and not an approximation: offsets, widths and operand order are swept
   *within* a shape by the per-axis fragments and are explicitly not crossed
   here, and keying on raw bytes would make every immediate its own "shape".

   See §"Why the key is the shape and not the label" for the measurement that
   forced this.
3. **A family with no representative fails the lane, by name.** That check found
   a real hole on its first run: `call-sequence`, `call-sequence-value` and
   `call-sequence-lit` — the newest accepted class, and *the* class that made
   §6 #12 reachable — had **no single-family case anywhere in the corpus**.
   Every TU that reached them carried a second function, so the class had only
   ever been graded beside something else. `scripts/sweep.d/71-call-sequence.py`
   is what closed it (+303 cases).
4. **The external-bearing predicate is measured, not assumed — and it is a
   heuristic, not a derivation.** A representative is "external-bearing" if its
   own obj carries `_fltused`, `__savegprlr`, `__restgprlr` or a `.pdata` — read
   out of the bytes, never inferred from the family's name.

   What that predicate is *for* is picking tier-C representatives likely to
   disturb the compiler-label counter, which is the mechanism behind every bug
   this lane exists for. It is **not** an instance of "one slot per TU-level
   external": that rule was **refuted** on 2026-07-31 — `docs/LABEL_COUNTER.md`
   §2.1 — in both directions, by a newly pooled FP constant that costs **+2 and
   mints no external at all**, and by a string literal that mints one and costs
   **0**. The rule that fits the measurements is a per-function **surcharge
   table**, `LABEL_COUNTER.md` §1.1: base 1 for a leaf and 4 packed / 5 `/Gy`
   framed, plus `+1` for `_fltused` on the *first* FP-touching function, `+2`
   per distinct GPR/FPR helper width first introduced, `+2` per newly pooled FP
   constant, and `0` for a callee external at any count.

   The predicate still selects well because three of its four markers *are*
   surcharge-bearing (`_fltused` +1, the helper pairs +2 each, `.pdata` marks
   the framed base). What it **misses** is stated under "what it deliberately
   does not grade": the surcharges that mint nothing.

## Why the key is the shape and not the label (2026-08-01)

The lane keyed on the census **label** until this date, and a label is a *name*.
The cost of that was measured rather than argued.

On 2026-07-31 a rung landed **+5,507 functions of genuinely new accepted shape**
— a trailing literal call argument, and the formal that moves beside it. Every
number this lane prints came back **byte-identical to the pre-merge run**: 28
families, 84 representatives, 406 pairs, 388 emitted. The new shapes had been
absorbed into the existing `multiarg-tail-call` label, so they added no family
and no pair; and with three representatives sampled per label, all three drawn
from older fragments, they added no representative either. The contrast is the
tell: when an earlier rung genuinely added *labels*, this counter moved 18 → 20
families and 171 → 210 pairs with no human input. It did not move here.

That is correct behaviour for a label-keyed instrument and a real hole in what it
proves — the new shape was graded *alone* by `expr_sweep`, and never once beside
another family, which is precisely the configuration that produced §6 #12.

Measured on the merged tree, at `/Ox /GS- /c`, over the 8,863 single-family
candidate TUs the sweep corpus produces:

* **433 distinct emitted shapes** across the 28 families.
* The label-keyed selection's **84 representatives covered 41 of them** — one
  shape for 17 of the 28 families, because "smallest first, one per fragment"
  systematically picks the same small emission three times.
* `multiarg-tail-call` alone holds **8** distinct shapes. Four of them are
  contributed *only* by the two fragments the rung added
  (`74-lit-call-arg`, `75-moved-lit-call-arg`), and **none of those four was a
  representative**. The rung's shapes were never crossed against anything.

Shape-keyed, with the cap at 8, the same corpus yields **147 representatives
covering 147 shapes**, and `multiarg-tail-call` is **completely** represented —
its slots now include `74-lit-call-arg-0055`, `74-lit-call-arg-0069`,
`75-moved-lit-call-arg-0001` and `75-moved-lit-call-arg-0003`, i.e. the rung's
own emissions, crossed against every other family in both orders at all twelve
lanes. The cap is 8 for that reason and not by taste: **it is the smallest cap
that fully covers the family the hole was reported against.**

The cap is a **residue, and the run names it**. 433 − 147 = 286 measured shapes
are not crossed, and the output lists them per family with counts:

```
    store-run              165 of 173     compare-leaf            8 of  16
    call-sequence-value     49 of  57     call-sequence-load-fp   6 of  14
    call-sequence           16 of  24     float-leaf              4 of  12
    straight-line           12 of  20     double-leaf             4 of  12
    call-sequence-lit       12 of  20     indirect-load-leaf      1 of   9
    store-leaf               9 of  17
```

That number is the thing that was silent before: a rung that emits an
instruction it did not emit before now moves it whether or not it adds a label.

**What this does not do.** It does not make "family" mean shape in the pair
matrix — pairs, the refusal frontier and the residue are still reported per
census label, so every earlier run's numbers stay comparable. Making the pair
axis itself shape-keyed would put tier A at 433² ≈ 187k configurations per lane
(≈ 2.2 M gradings over the registry), which is roughly **5×** this lane's whole
current cost for a matrix that is 96 % single-family diagonal. What it buys —
crossing the 286 uncrossed shapes against everything — is bounded above by that
286, and that is the number to weigh when the question comes back.

## The mode lanes are `scripts/lanes.txt` (2026-08-01)

This file used to say "four mode lanes throughout: `/Ox` packed, `/Ox /Gy`,
`/O1`, `/O2`", and those four were **hardcoded in `scripts/cross_sweep.py`**.
They compile **no `/EH` at any invocation**.

That was the last surviving instance of the un-enumerated-lane defect, and it was
in the worst possible place. Every TU of the dc3 workload is compiled `/EHsc`;
**35,964 `eh-bare` functions are in class with markers that appear only under
`/EHsc`**; and this is the lane whose entire purpose is finding mis-emits the
hand-written corpus cannot — its record is real, §6 #12 was found in the cross
product of two individually-green branches. Its `/EHsc` intersection was empty
and its green read exactly like a green that had verified those flags.

The lane now reads `scripts/lanes.txt` and grades **every** lane in it, splicing
each row's flags exactly the way `scripts/mode_lane.sh` does
(`<mode> /GS- /c <rest>`), so "the same lane" means the same characters in the
same order as the fixture gate's. Concretely the mode set went **4 → 12**:

| | before | after |
|---|---|---|
| kept | `/Ox`, `/Ox /Gy`, `/O1`, `/O2` | same four, identical flag strings |
| gained | — | `/EHsc` over all six base configurations, `/O1 /Oi`, `/Od` |
| lost | — | **nothing** |

Two consequences worth stating:

* **It inherits the registry's assertions for free.**
  `crates/c2-harness/tests/lane_registry.rs` already requires the shipped list to
  carry an `/EH` lane, to *vary* `/Oi` where `/Ox` does not already imply it, and
  to name `/O1 /EHsc` by flags even though its verdict rows are identical to
  `/O1`'s — verdict-identical is not redundant, the reference obj is a different
  obj. None of that had to be restated in the driver.
* **A new test keeps it converted.** `cross_product_lane_takes_its_modes_from_the_registry`
  asserts, over `scripts/cross_sweep.py` itself, that it mentions `lanes.txt` and
  that it defines no private mode table. Nothing in `cargo test` would otherwise
  notice a "tidy up the sweep driver" commit pasting the four back, and the
  symptom of that regression is silence.

`/Od` in the registry is what forced tier S below: it refuses essentially
everything on purpose, so a wrapping check asserted unconditionally would have
blamed the instrument for the mode.

## What it grades

Twelve mode lanes throughout — `scripts/lanes.txt`.

| tier | what | count at k = 8 |
|---|---|---:|
| S | each representative **alone** — tier W's control | 147 |
| W | each representative **alone inside a namespace** — the wrapping check | 147 |
| A | every **ordered pair** of representatives, both orders, diagonal included | 21,609 |
| B | **arity**: n = 1…4 copies of a family, alone and with a framed observer before and after | 336 |
| C | **ordered triples over the TU-external families**, with and without a stride-1 separator at each position | 20,480 |

42,719 configurations × 12 lanes = **512,628 gradings**, ~19 min cold at
`C2RS_JOBS=24` on a 32-core host. (Against 27,956 × 4 = 111,824 before: ×1.53 in
configurations, from the shape key, and ×3 in lanes, from the registry.)

Three of those tiers need their reason stated, because none is obvious:

* **Tier W exists so the lane cannot lie.** Every half after the first sits in a
  `namespace`, and if a namespace by itself pushed a shape out of class then
  every pair would grade a refusal and the green would mean nothing.
  (Namespaces rather than identifier renaming: they cannot collide, they need no
  tokenizer, and the port reads names out of the IL so the extra mangling is not
  a variable. The **first** half is left unwrapped, so it is byte-identical to
  the standalone case that was graded.)
* **Tier S is tier W's control, and without it the registry conversion would
  have been unsafe.** The old check was "every W must match", which is only a
  statement about the *wrapping* at a lane where the representative matches
  unwrapped. `/Od` is in the registry and refuses on purpose. Asserted
  unconditionally it would have reported "the wrapping is not coverage-neutral"
  for all 147, which is false. So each representative is compiled both ways at
  every lane and the alarm is the **difference**: W refuses where S matched.
* **Tier C is where a per-function and a per-TU counter rule come apart.** #13's
  candidate pair — "one slot per function plus one for the TU if anything
  touches floating point" versus "two slots per FP function" — agree at n = 1
  and disagree at n ≥ 2, which is why a single-FP-function probe could never
  have separated them and why the wrong one looked right. Pairs get n = 2; the
  triples get n = 3 with every ordering and with a separator, because a counter
  error an adjacent function absorbs is invisible without one. (Both of those
  candidates have since been superseded by the measured surcharge table,
  `docs/LABEL_COUNTER.md` §1.1; what tier C grades is unchanged, and *n* is
  still the axis that separates a per-function quantity from a per-TU one.)

## What it deliberately does NOT grade

Stated because a silent cap reads as "covered everything", which §6 forbids.

* **Triples of three *distinct non-external* families.** Tier C is restricted to
  the TU-external families (16 of 28); the full `R³` is not run. A three-way
  interaction among plain leaves would not be caught.
* **The measured shapes beyond the cap — 286 of 433, named per family in the
  run's own output.** A family gets up to 8 of its distinct emitted shapes;
  `store-run` has 173 and gets 8, `call-sequence-value` has 57 and gets 8. **17
  of the 28 families are completely represented; 11 are not**, and the run prints
  which and by how much. Operand order, widths, offsets and source lines *within*
  one shape are swept by the per-axis fragments and are still not crossed here.
  Concretely: the lane now grades "each of `multiarg-tail-call`'s 8 emissions
  beside every other family", not "every addr-leaf beside every framed call".
* **The label surcharges that mint no symbol.** Tier C selects its triples on an
  *external-bearing* predicate, and `docs/LABEL_COUNTER.md` §1.1 measures three
  surcharges that predicate cannot see: a **newly pooled FP constant** (+2), a
  **materialised signed relational** (+2), and a **loop** (+2 to +5, and not
  uniform). A TU built from those would disturb the counter exactly as an
  external-bearing one does, and tier C would not have selected it. Two of the
  three are not reachable today anyway — §2.1 checked each counterexample
  through `c2rs diff` and the TU-level gate refuses all of them — so this
  overlaps the refusal frontier rather than adding to it, but the overlap is
  incidental and will stop holding the moment that gate moves. When it does, the
  predicate should be re-grounded on the surcharge table rather than on symbols.
* **Flags beyond the registry** — `/GS`, `/GR`, `/Zi`, and every combination of
  what `scripts/lanes.txt` already holds. The four-hardcoded-modes hole that used
  to be recorded here was closed on 2026-08-01; see "The mode lanes are
  `scripts/lanes.txt`" above. Adding an axis is now a line in the registry and it
  is inherited by both this lane and `scripts/gate.sh` — but the cost is not
  free here, because this lane multiplies by ~43k configurations rather than by
  197 fixtures. Measure before adding one.
* **Everything on the refusal frontier below.** Those are compiled and counted
  and named, but the port refuses the TU, so no bytes were compared. They are
  **unmeasured**, not green.

## Result, 2026-08-01 (master `97a60bc`)

**0 mismatches over 512,628 gradings, every one of which was graded** — 42,719
configurations × the 12 registry lanes, `C2RS_JOBS=24`, ~24 min end to end.
Stated positively, because `mismatch 0` over `graded 0` is the shape that has
fooled this repo's instruments nine times:

| | before (same tree, label-keyed, 4 hardcoded modes) | after |
|---|---:|---:|
| configurations | 27,956 | **42,719** |
| mode lanes | 4 | **12** |
| gradings submitted | 111,824 | **512,628** |
| gradings graded | 111,824 | **512,628** |
| capture-fail | 0 | **0** |
| families | 28 | 28 |
| representatives | 84 | **147** |
| distinct emitted shapes crossed | 41 of 433 | **147 of 433** |
| families completely represented | — | **17 of 28** |
| pairs reached | 406 | 406 |
| pairs emitted | 406 | **406** |
| refusal-frontier residue | 0 | **0** |
| mismatches | 0 | **0** |

* **No pair regressed.** 406 of 406 before and after; the frontier is empty in
  both. (The 18-pair FP-leaf frontier that stood here until 2026-08-01 was closed
  by the WUNW rung, not by this change.)
* **The `/EHsc` intersection is no longer empty.** Six of the twelve lanes
  compile `/EH`, and each grades all 42,719 configurations. Their verdict rows
  are identical to their plain partners' — which is not the same as redundant:
  the reference obj is a different obj, and the port reproduced it byte-exactly
  512,628 / 12 × 6 times over.
* **`/Od` and `/Od-EHsc` refuse everything, on purpose**: 42,393 `codegen-gap` +
  326 `vocab-gap` each, 0 mismatch. That is the fail-closed boundary lane's whole
  content, and it is why tier S exists — without its own control at each lane,
  those two would have reported "the wrapping is not coverage-neutral" for all
  147 representatives.
* **The residue that is left is named twice**: 286 measured shapes not crossed
  (table above), and 18 ordered tier-A pairs (9 unordered, all `compare-leaf`)
  refused at every lane — the port refuses those TUs, so no bytes were compared
  and they are unmeasured, not green.

## Running it

```sh
scripts/cross_sweep.sh                       # full, 42,719 x 12, ~24 min cold
C2RS_CROSS_REPS=1 scripts/cross_sweep.sh     # 1 shape/family — smoke, not a gate
C2RS_LANES=/path/to/lanes.txt \
  scripts/cross_sweep.sh /abs/work           # a cut-down registry, for iteration
C2RS_JOBS=32 scripts/cross_sweep.sh /abs/x   # more parallelism, chosen workdir
```

**The workdir is made absolute at entry, in both the shell wrapper and the
driver.** A relative one used to die with a bare `KeyError` (the paths written
and the paths graded disagreed), and it is the same shape as the failure that
yields `z:work\…` — which `cl.exe` cannot open, so every case capture-fails and
every count parsed out of a report reads 0 and passes.

Exit codes: `0` clean · `1` **MISMATCH** (an alarm — the port emitted bytes for
a combination and they were wrong) · `2` a declared family has no representative
(a hole in `scripts/sweep.d/`) · `3` the namespace wrapping is not
coverage-neutral at some lane, measured against that lane's own standalone
control (the instrument is lying) · `4` a lane reported `capture-fail`
(configurations submitted and never graded) · `5` a lane graded nothing, or no
pair emitted anywhere. Toolchain absent → `SKIP`, exit 0.
