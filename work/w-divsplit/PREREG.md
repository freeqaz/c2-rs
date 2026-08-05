# w-divsplit — PREREGISTRATION

Written **before** any measurement was taken. Tree at write time: master `8fd79b6`
(worktree `.claude/worktrees/agent-ad194bc5fcb677fd8`). The only numbers read
before writing this are the ones already published in `docs/BOARD.md` #782/#783:
`expr-op-0x05` = **4,670**, `expr-op-0x06` = **4**, over 878 TUs and 2,463,393
bodies.

## 0. The assignment

Board **#783**: the census key `expr-op-0x05` cannot separate integer division
from floating-point division, so 4,670 is an **upper bound** on the integer
population. Split it and report the real number.

## 1. The mechanism I expect to find (stated so it can be wrong)

`expr-op-0x05` is produced at **exactly one site**: the fall-through arm of
`parse_expr_classed`, `crates/c2-il/src/func/body/expr.rs:1487`
(`_ => return Err(blk(seg, *p, "expr"))`), rendered by `Block::feature()` at
`crates/c2-il/src/func/body/mod.rs:1115`. Grepped: `"expr"` as a `ctx` with a
blocking byte occurs nowhere else in `crates/c2-il`.

A census key is the **first** blocker of a body. To reach the `05` byte the walk
must already have consumed both operands, and every operand-producing arm admits
a type only through `eat_operand_type` (`ValueClass::Int4` / `Ptr4` / `Int1u`) or
a class-preserving / width-4-reinterpret `2C`. A `float`/`double` operand refuses
**earlier**, at `expr-load-type-<tagkind>`, `expr-lit-type-<tagkind>` or
`expr-convert-target-<tagkind>` — all of which are *different first-blocker
keys*.

So my expectation is that #783's ambiguity, while real as a statement about the
*byte* `05`, is **not** realisable as a first-blocker key: the FP share of the
4,670 should be **zero**, and the 1000× asymmetry against `expr-op-0x06` has some
other cause.

This is the claim that can lose, and it loses loudly if even one of the 4,670
carries a real-class (`kind & 0x0F == 5`) operand.

## 2. Registered predictions

Each is a number with a denominator, graded in the rung's §1 table.

| # | prediction | loses if |
|---|---|---|
| **R1** | **FP share of the 4,670 is 0.** Integer share ≥ 4,600 of 4,670 (≥ 98.5 %) | any site classifies as float/double |
| **R2** | A **fixed-offset** reader (the byte at `hex_mark - 3` being a tag `86`/`A6`/`96`) misclassifies **≥ 100** of the 4,670 sites — #644's warning is live here, because `2C <TYPE> 00` puts a `00` immediately before the operator and `(a*b)/c` puts an operator byte there | fewer than 100 sites fail the fixed-offset read |
| **R3** | At least one site has a **pointer**-class operand (`p - q` scaled by the element size is a division whose operands are `Ptr4`) | zero pointer-class sites |
| **R4** | The number of distinct `(tag,kind)` operand pairs over all 4,670 sites is **≤ 6** | 7 or more |
| **R5** | `expr-op-0x06` (4 functions) is also 100 % integer | any of the 4 is not |
| **R6** | Conservation: after the key refinement the new `expr-op-0x05-*` buckets sum to **exactly 4,670**, per-function census stays **711,427/2,463,393** and emitted census **39,177/178,975** — the port accepts nothing new and `fnbyte-differs` stays 0 | any of those four numbers moves |
| **R7** | **The prize is much smaller than 4,670 once emission is asked.** The EMITTED share of the 4,670 is between **150 and 800** (the workload-wide emitted rate is 178,975/2,463,393 = 7.3 %, which would give ~340) | outside [150, 800] |
| **R8** | **Most divisions have a CONSTANT divisor.** > 50 % of the 4,670 sites have a `33` LITERAL as the immediately preceding operand token rather than a `B9` LOAD — which matters more than the int/float split, because a constant divisor is a magic-multiply synthesis and never a `divw`, and w-divmod's `div_mod_leaf` refuses every constant divisor (#781) | ≤ 50 % |

**R1 and R7 are the two that decide the lane's headline.** R1 says the 4,670 is
not diluted by FP; R7 says it is diluted by something else entirely.

## 3. Method

Two independent instruments, required to agree row for row:

1. **BYTE-SCAN (the board's own suggestion).** `C2RS_ROW_DUMP=expr-op-0x05,
   expr-op-0x06` over the 878-TU workload dumps every site's hex window with its
   `hex_mark` (`crates/c2-harness/src/gap/witness.rs:161`, read-only over the
   census by construction and asserted so there). A Python probe under
   `work/w-divsplit/` decodes **backwards** from the mark by re-deriving the
   token that ends there, and prints the count of sites it could **not** decode
   rather than defaulting them into a bucket.
2. **PARSE-DERIVED.** The census key itself gains the operand type, recorded by
   `parse_expr_classed` at the moment it *reads* the type — so the offset is the
   parse cursor's own and never an assumed stride (#644).

The two must produce the same partition. Disagreement is a result and gets
printed, not reconciled.

## 4. The must-fail mutation

The byte-scan classifier is a measuring device, so it gets calibrated: a mutated
copy that reads the type one byte to the left (`mark-4` instead of the decoded
token start) must be **caught** by a control that the honest one passes. If the
mutation passes every control, the controls are worthless and the measurement is
withdrawn.

## 5. What this lane will NOT do

- It will not widen the port. `PORT_WRITER_SECTIONS`, `select_function` and the
  codegen classes are untouched; `fnbyte-differs` must stay 0.
- It will not fit a placement rule for `twi` (#780 refuted three of those).
- It will not claim a conversion. A sizing lane converts nothing.
