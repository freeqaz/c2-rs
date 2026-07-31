# WREL — the relational family is bare, and it is the first operator worth ranking

    Tag:       WREL
    Slug:      relational-bare
    Date:      2026-07-31
    Fixtures:  (measurement only — no fixture, no widening; see "What to do with it")
    Census:    639,387 unchanged — this is an INSTRUMENT rung, not a widening one
    Record:    this document

This rung ships **no acceptance change**. It replaces an *inference* with a
*capture*, and the inference turns out to have been wrong.

## The claim being retired

`crates/c2-il/src/func/body/mcall.rs` documented the exclusion of the relational
family from `BARE_BINARY_OPS` like this:

> …the relational family `1F`–`24` is excluded because a neighbouring opcode in
> the same numeric range is observed carrying a TYPE
> (`… 33 86 41 74 01 · 19 · 86 42 75 …`), so "one byte" would be an assumption
> there rather than a reading.

Every other member of that set was admitted on **two** pieces of evidence — a
capture witness that the token is bare, and a 1:1 redistribution over the
878-TU workload. `1F`–`24` was excluded on **neither**. It was excluded by
reading a *different byte* and generalising across a numeric neighbourhood,
which is the same move `call-anchor-*`, `expr-cast` and the relational *names*
were each wrong about before — applied across a byte boundary instead of past a
capture set.

## Estimate, written before the capture

`work/gt-relational/ESTIMATE-task1.txt`, recorded before the probe ran:

* **Prediction: BARE, ~60 %.** The IL is stack-typed — every LOAD
  (`b9 <tok> 86 42 75`) and every LITERAL (`33 86 41 74 <n>`) carries its own
  TYPE — so a compare can take signedness off the operands. `09`/`0A`
  (`<<`/`>>`) already prove the point: arithmetic-vs-logical shift is the same
  problem and those bytes are bare.
* **Against:** `1F`–`24` is exactly six bytes for exactly six relations, so
  signedness is not in the opcode and has to come from *somewhere*; and `19`,
  one byte below the family, demonstrably carries a TYPE.
* **Bias named in advance:** I wanted them bare, because bare makes the row
  rankable. So a bare reading was not to be accepted on the hexdump alone — it
  also had to produce a redistribution summing to exactly 0 with the census
  numerator unchanged.
* **Secondary prediction: `-whole` non-zero but small — "low hundreds at most,
  most likely under 50"**, on the reasoning that a compare in a real TU is
  overwhelmingly the condition of an `if` and therefore hits a branch on the
  next token, exactly as `&` does.

## The capture that settles it

`work/gt-relational/rel_probe.cpp` puts all six relations in **value** position
and in **branch** position, signed and unsigned, **and the compound-assign
control in the same TU** — so `19`'s family is read beside the relations rather
than from memory. `c2rs census … --keep-il`, `/Ox`:

```text
  v_lt   4c 4f 11 53 b9 ee 09 86 42 75 · 33 86 42 75 03 · 22 · 2c 86 41 74 00 · 41 …
  v_le   … 33 86 42 75 03 · 21 · 2c 86 41 74 00 · 41 …
  v_gt   … 33 86 42 75 03 · 24 · 2c 86 41 74 00 · 41 …
  v_ge   … 33 86 42 75 03 · 23 · 2c 86 41 74 00 · 41 …
  v_eq   … 33 86 42 75 03 · 1f · 2c 86 41 74 00 · 41 …
  v_ne   … 33 86 42 75 03 · 20 · 2c 86 41 74 00 · 41 …

  b_lt   4c 4f 11 53 53 b9 12 0a 86 42 75 · 33 86 42 75 03 · 22 · 38 15 0a …
  b_ge   4c 4f 11 53 53 b9 16 0a 86 41 74 · 33 86 41 74 03 · 23 · 38 19 0a …

  THE CONTROL, same TU, same capture:
  c_add  4c 4f 11 53 26 1a 0a · 33 86 41 74 03 · 0f · 86 41 74 · 4b …   (`+=`)
  c_shr  4c 4f 11 53 26 1d 0a · 33 86 41 74 03 · 16 · 86 42 75 · 4b …   (`>>=`)
```

**The relational family is bare.** In value position the operator is followed
immediately by `2C` (the class-preserving convert); in branch position by `38`
(the conditional branch). Nothing in between, in twelve leaves and two branch
bodies.

**The compound-assign family is not**, and the control proves the instrument can
tell the difference: `0F` (`+=`) is followed by `86 41 74`, `16` (`>>=`) by
`86 42 75`. `19` is a member of *that* family, not of the relational one — which
`GAPS.md` §6 already recorded and which nobody had put beside a relational
capture. The neighbourhood inference crossed a family boundary that the numeric
order hides.

Two further readings fall out for free, neither of them assumed:

* **Signedness is not in the opcode.** `v_lt` (unsigned) and `s_lt` (signed)
  both emit `22`; the only difference is the operand TYPE, `86 42 75` vs
  `86 41 74`. This is the argument the estimate leaned on, now measured.
* **The map, all six, both signednesses:**
  `1F ==` · `20 !=` · `21 <=` · `22 <` · `23 >=` · `24 >`. Identical to the
  table `expr_opcode_name` already carries, so the *names* were right; it is the
  *width* that was never read.

## The redistribution — the second piece of evidence

Scratch build only (`work/gt-relational/scratch/`, a copy — this lane makes no
`crates/**` change), `BARE_BINARY_OPS` extended by `1F`–`24`, full 878-TU
rescan against the same capture cache:

| | base | granted |
|---|---:|---:|
| census numerator | 639,387 (25.96 %) | **639,387 (25.96 %)** |
| census/gate disagreement | 0 | **0** |
| total blocked functions | 1,823,184 | **1,823,184** |
| distinct blocker keys | 696 | 722 |
| **sum of all key deltas** | | **exactly 0** |

Every `…-then-cmp-*` row empties into its own `-and-<second>-<whole|more>`
children and nothing else moves. The falsifier — a byte that is *not* bare
desyncs the matcher and scatters its row across the hex tail — is the
`sum of all key deltas` column, and it is printed by
`scripts/gt_relational_redist.py` on every run rather than remembered.

## What it is worth: the first non-zero `-whole` an operator grant has produced

| grant | row | `-whole` |
|---|---:|---:|
| `&` (`0B`, W37) | 102,382 | **0** |
| **relational (`1F`–`24`)** | **17,146** | **7,529 (43.9 %)** |

Whole-body completeness over the whole workload rises **85,935 → 95,006, +9,071**
(the 7,529 in the `cmp` rows plus 1,542 in rows where a compare was the *second*
blocker under some other first blocker — `…-then-type-int1-and-op-whole2` +824,
`…-then-type-ptr-and-type-real-lit-whole4` +703, and eight singletons).

**Predicted "under 50, low hundreds at most". Actual 9,071. The estimate was
low by more than two orders of magnitude, and the bias direction is the
interesting part:** I reasoned by analogy from `&`, whose row is 99.99 %
`cflow-if-1`, and assumed the relational family lived in the same place. It does
not — see the split below. Reasoning from the *previous* operator's population
is exactly the unstable-attribution error `GAPS.md` §6 warns about, and W36's
own lesson ("its rate could not be borrowed from the row above") was available
and not applied.

**One qualification that must travel with the 9,071, because it changes what the
rung would be:** the `-whole` count for the compare **alone** is **0**. There is
no `…-then-cmp-*-whole` key with a single admission anywhere in the granted
scan. Every one of the 7,529 is `-whole2` and the partner is almost always
`type-int1` — the **`bool` the comparison produces**. The compare and its result
type are **one rung, not two**; a rung that admits `1F`–`24` and stops will
convert 0 functions, which is precisely the `&` outcome arrived at by a
different route.

## The other half of the family, and it *is* the `&` story

The free-standing `expr-cmp-*` rows (26,627) never reach `mcall`, so
`BARE_BINARY_OPS` cannot measure them — W37's method applies instead: admit the
bytes in `parse_expr` and rescan. Scratch build, arity-preserving stand-in
(`IlOp::Sub`, 2→1 like a compare — the *value* is fiction, so the numerator this
produces would be an over-claim; it is 0, so the point is moot):

| lands in | Δ |
|---|---:|
| `expr-brfalse` | **+19,409** |
| `expr-brtrue` | **+2,955** |
| `expr-intrinsic-memcpy` | +1,622 |
| `expr-or-or` | +958 |
| `expr-call-in-expr-op-0x1F` | +717 |
| `expr-ternary` | +508 |
| `expr-intrinsic-vbase-upcast` | +396 |
| `expr-cmp-{eq,ne,le,lt,ge,gt}` | −26,627 |
| **census numerator** | **+0** (639,387 unchanged, disagreement 0) |

**84.0 % of the free-standing relational population is one token from a
conditional branch.** That is the `&` result reproduced exactly, on a different
operator, in the same lane. The family is not uniform: its *mcall* half carries
7,529 complete bodies and its *parse_expr* half carries none.

## What to do with it — and what this rung deliberately does not claim

* **The exclusion is retired.** `1F`–`24` now has both pieces of evidence
  `BARE_BINARY_OPS` demands. Granting it is an instrument change (the
  completeness matcher is diagnostic-only; nothing here can accept a function).
* **The row that can now be ranked is `compare + bool-result`, 7,529 `-whole`**,
  not "the relational operator", which is worth 0 on its own. Rank it against
  W38's realized +36,684 with the usual discount: `-whole` is an upper bound on
  in-class yield, not a promise of one — no codegen-class gate is applied.
* **`expr-cmp-*` (26,627) is now MEASURED and worth 0**, for the same reason `&`
  is. Declined on measurement, not on argument.
* The `cflow-if-1`/`cflow-loop` cross-tabs the earlier hand-off quoted
  (4,040 / 3,242 on `expr-cmp-eq`) are a *slice* of an 18,249 row, and slicing
  them was what made the row look small. The row is 18,249 and it is worth 0;
  the value is in the *other* family, which the cross-tab did not name.

## Re-running it

```sh
# capture — is the token bare?
./target/release/c2rs census work/gt-relational/rel_probe.cpp \
    --keep-il work/gt-relational/il
python3 scripts/gt_relational_redist.py --il work/gt-relational/il

# redistribution — does it sum to 0?
python3 scripts/gt_relational_redist.py \
    --base work/gt-relational/scan-rel-base.jsonl \
    --grant work/gt-relational/scan-rel-grant.jsonl \
    --parser work/gt-relational/scan-rel-parser.jsonl
```
