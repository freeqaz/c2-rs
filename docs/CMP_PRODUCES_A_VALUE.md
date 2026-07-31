# The comparison that produces a value — DECLINED on measurement, and the +9,071 is Class B

Measured 2026-07-31 on `c2c0c37`. **Not a rung** — the acceptance change was
built, graded byte-exact against real `c2` over 552 generated cases in four mode
lanes, measured at **+0 census functions on the 878-TU workload, and reverted.**
It ships one instrument change (an operator grant that is diagnostic-only) and
four byte-level readings that the next rung into this family needs.

Tag W42. Estimate written before any scan: `work/W42/ESTIMATE.md`.

## The headline

`docs/OPERATOR_GRANTS.md` ranked `expr-call-in-expr-recv-*-then-cmp-*` at
**+9,071 whole bodies** (7,528 of them in the `cmp` rows themselves) and named
the trap correctly — the compare alone is
worth 0, so compare and `bool`-result are one rung. Both halves were built. The
row still converts **0**.

**The row is not `return p->m() <rel> k;`. It is `return a->m() <rel> b->m();`**
— a comparator — and 89.8 % of it has **two calls**, with the first result live
across the second. That is **Class B**, the frame class W41's own "found and not
taken" put at #1 and the port has no callee-saved-register model at all. Nearly
all of the remainder needs a *receiver* production (the virtual-base upcast),
which is not a comparison feature either.

| | functions |
|---|---:|
| the granted `…-then-cmp-*` row, all of it | **17,147** |
| … `-and-branch-more` ⇒ the condition of an `if`; needs **basic blocks** | 9,490 |
| … `-more` on something else | 129 |
| … **whole** — the ceiling this rung was ranked on | **7,528** |
| of the 7,528: `calls-2plus`, a value live across a call ⇒ **Class B** | **6,760 (89.8 %)** |
| of the 7,528: `calls-1` but *still* Class B — measured, then captured | **66** |
| of the 7,528: `calls-1`, all `recv-intrinsic-vbase-upcast` ⇒ a **receiver** rung | **699** |
| of the 7,528: `calls-1`, anything else | **3** |
| of the 7,528: `calls-0` | **0** |
| `return p->m() <rel> <literal>;` — the shape actually built | **0** |

**`calls-2plus` undercounts Class B and the 66 are how we know.** A body with
*one* call is Class B whenever the comparison's other operand is a formal:
`bool f(const S* p, int k){ return p->m() == k; }` is `std 31 · mr 31,4 · bl ·
sub 11,31,3 · … · ld 31`. Those 66 were found by decoding the loaded right-hand
side and refusing it by name (`cmp-rhs-live-across-call`), and the refusal's
count is exactly the row's drop — then the shape was captured to confirm the
frame. So the real Class B share of the ceiling is **6,826 of 7,528 (90.7 %)**,
and the frame-class column is a lower bound on it, not a measure of it.

The frame axis is the whole answer and it was free: `calls-0 / calls-1 /
calls-2plus` is already in every scan.

## Estimate vs outcome

`work/W42/ESTIMATE.md`, written before any scan, with the pre-filter named.

| | estimate | outcome | bias |
|---|---|---|---|
| realized census | **+900**, range 150–3,000 | **+0** | HIGH; the range **missed**, and its floor was 150× the truth |

**The estimate's reasoning was right and its arithmetic was still wrong.** It
named the correct pre-filter — "all 9,071 sit under an `expr-call-in-expr` first
blocker, so realizing them requires the *call* production too, and the port's
accepted call classes are far narrower" — and then multiplied by a **guessed
rate**: "I am guessing 10–15 % and I know that is the exact move (borrowing a
rate across populations) that cost W36 2.99× and cost the relational measurement
two orders of magnitude." It was written down as the named risk and committed
anyway.

The correct move was available and cost one second: **cross the row with the
frame class before estimating.** `calls-2plus` is 6,760 of 7,528 and it is
printed in every scan already. W41's estimate did apply the frame axis, said so
("it removes 0.3 % of the row rather than 99 %"), and was the first estimate in
six rungs to land inside its range. Here it removes 89.8 % and nobody looked.

So the new entry for the ranking table is not a ratio, it is a rule:

> **A `-whole` count is an upper bound on the GRAMMAR, and the frame class is not
> in the grammar.** `body_matches` has no notion of what is live across a call,
> so a body that needs `std r31`/`ld r31` reads `-whole` exactly like one that
> does not. Cross every `-whole` row with `calls-0|calls-1|calls-2plus` before
> quoting it. Rows measured this way so far: W41's `recv-load-whole` **0.3 %**
> `calls-2plus`, realized 2.62× below the row; this row **89.8 %**, realized 0.
> And the column is a *lower* bound — see the 66 above.

## What was built, graded and reverted

The new recognizer is kept verbatim in `work/W42/framed_compare.rs.declined`;
the rest was a set of mechanical edits and is **not** kept, so the list below is
the spec rather than a pointer. Everything hard about rebuilding it is in "the
four readings" — that is the part that took the measurements.

* `shapes/framed_compare.rs` — the relational post-op after a call's `4C`, shared
  by the free-function and member-call heads (the seam W41 had to repair), with
  `CmpRhs::Load` decoded and refused **by name** as `cmp-rhs-live-across-call`.
* `try_parse_compare` — the `2C` convert made optional, which is the
  `bool`-returning comparison leaf.
* `CompareLeaf` — two new fields, `bool_result` and `from_call`, because the
  spine is a function of both (see below).
* `compare_spine` split out of `compare_leaf_text`; `framed_call_text` taking a
  post-call *text* instead of an `i32`; `plan_labels` taking a per-function
  leading-slot count.
* Census keys `framed-compare` and `compare-leaf-bool`.

**Graded green everywhere it was asked**: `cargo test --workspace` all pass
(464 baseline plus this file's three); a
generated 552-case acceptance grid (relation × signedness × `k ∈ {0, ±3, ±32767,
−32768}` × result ∈ {`int`,`bool`,`unsigned`} × head ∈ {leaf, member call, free
call, receiver in a non-zero slot}) **byte-exact in `/Ox`, `/O1`, `/O2` and
`/Ox /Gy`**; 878-TU scan mismatch 0, disagreement 0.

**Reverted anyway.** A production with **zero witnesses on the workload** is
graded only by its own fixtures, which is the configuration W41's "found and not
taken" #7 names as "the same configuration that let the original wrong gate stand
for seventeen rungs". Paying that for +0 is the wrong trade, and the four
readings below are the part worth keeping.

## The four readings, which are the durable result

Every word read off a reference obj, `/Ox` and `/O1`, `/Gy` and packed. The
full 48-cell table is one command — `scripts/gt_cmp_spine.py`, added by this
measurement — and it prints `.` for a cell that depends only on the three axes
the port already models and names the axis for a cell that does not.

### 1. The `bool` result is a different spine, and nothing in the operator says so

`bool f(int a){ return a >= 3; }` and `int f(int a){ return a >= 3; }` have the
same operator, the same operand types and the same literal. The IL differs by
the presence of the `2C <int> 00` convert; the **obj** differs by two words:

```text
  int  … >= 3   39600003 li 11,3 · 7c6afe70 srawi 10,3,31 · 55690ffe srwi 9,11,31
                7d0b1810 subc 8,3,11 · 7c695114 adde 3,9,10
  bool … >= 3   39600003 li 11,3 · 7c6afe70 srawi 10,3,31 · 7d0b1810 subc 8,3,11
                55690ffe srwi 9,11,31 · 7d695114 adde 11,9,10 · 5563063e clrlwi 3,11,24
```

Two of the 24 (relation × signedness × zero-or-not) cells behave this way —
signed `>=` and signed `<=` against a **non-zero** literal, the two sign-sum
spines whose result is not provably 0/1. The other 22 are byte-identical. Under
`/Ox` the `bool` form also **schedules** the dead `subc` ahead of the second
shift; under `/O1` it does not, with identical register numbers either way.

**This is a live trap for the next rung, not a curiosity.** `a->m() > b->m()`
returns `bool`, so the Class B rung this row is really waiting on will hit
exactly these two cells, and a spine borrowed from `compare_leaf_text` as it
stands today emits the `int` form: 2 words short, `.pdata FuncLen` and both `$M`
values wrong to match. It is `GAPS.md` §6's recurring shape — *two facts sharing
one field until some construct pulls them apart* — with the field being
"the comparison's result" and the construct being a `bool` return.

### 2. Leaf and framed part company in exactly one cell

Signed `<=` against a non-zero literal, in both modes, reproduced in two
independently generated TUs:

```text
  int f(int a)      { return a      <= 3; }   li 11,3 · srwi 10,3,31 · srawi  9,11,31 · subc 8,11,3 · adde 3,10,9
  int f(const S* p) { return p->C() <= 3; }   li 11,3 · srwi  9,3,31 · srawi 10,11,31 · subc 8,11,3 · adde 3,9,10
```

The two sign temps swap numbers. Instruction order, operand roles, the `subc`
and every other register are identical. 47 of the 48 cells do not care, which is
why a spine shared between the leaf and a framed body is *almost* free — and why
the one exception has to be a field on the record rather than an assumption.

### 3. The framed comparison's label stride is `4/5 + (leaf stride − 1)`, pre-allocated

Measured seed-free in one TU by `scripts/gt_label_stride.py`'s method (two plain
framed anchors around the probe; `first(a1) − first(a0) − 5` for the stride,
`first(P) − first(a0) − 5` for the part taken *before* the probe's own `$M`
pair), 26 rows per mode over `/Ox /Gy`, `/O1 /Gy`, `/O2 /Gy` and packed `/Ox`,
with the in-TU `a2` control holding on every row
(`scripts/gt_cmp_spine.py --stride`):

```text
                                        /Gy      packed
  framed, arithmetic post-op          5    0     4    0
  framed, cmp ==/!=, any literal      5    0     4    0
  framed, cmp unsigned, any relation  5    0     4    0
  framed, cmp signed < / >=, k == 0   5    0     4    0
  framed, cmp signed, anything else   7    2     6    2
                                    stride lead stride lead
```

i.e. exactly `CompareLeaf::label_slots`'s existing 1-or-3 table, re-expressed as
a surcharge, and **allocated ahead of the function's own triple** — the same
placement `docs/CODEGEN_FRAMED_CALLS.md` §4.4 records for the
`__savegprlr_N`/`__restgprlr_N` pair. `plan_labels` has no per-function leading
count today; when Class B lands it needs one, and it needs it for both reasons at
once.

**The `bool` result does NOT enter the stride** — measured on the same grid, both
results give the same number. This is the one place in this family where the two
axes that move the *bytes* do not both move the counter, so the stride cannot be
used as a proxy for the spine.

### 4. `return p->m() == k;` is Class B, and its spine is not a spine at all

```text
  bool f(const S* p, int k) { return p->m() == k; }
    7d8802a6 mflr 12 · 9181fff8 stw 12,-8(1) · fbe1fff0 std 31,-16(1)
    9421ffa0 stwu 1,-96(1) · 7c9f2378 mr 31,4 · 4bffffed bl ?m
    7d63f850 sub 11,31,3 · 7d6a0034 cntlzw 10,11 · 5543dffe rlwinm 3,10,27,31,31
    38210060 addi 1,1,96 · … · ebe1fff0 ld 31,-16(1) · 4e800020 blr
```

The difference is `sub r11,r31,r3` — register-register — where the literal form
is `addi r11,r3,-k`. So Class B does not merely add a save/restore pair around
the shipped spine: **it needs a second spine family**, one operand of which is a
callee-saved register. Sizing the Class B rung by "the frame plus what we
already emit" would understate it.

## The instrument change that IS kept

`BARE_BINARY_OPS` in `crates/c2-il/src/func/body/mcall.rs` now contains the
relational family `1F`–`24`. `docs/OPERATOR_GRANTS.md` supplied both pieces of
evidence the set demands and recommended the grant; this lands it, with the
capture witnesses and the **compound-assign control** pinned as tests, so `19`
carrying a TYPE is re-read from a capture rather than remembered.

Diagnostic only — `blocker_is_measured` feeds the completeness matcher and
nothing there can accept a function. Re-measured on the 878-TU workload:

| | base (`c2c0c37`) | granted |
|---|---:|---:|
| census numerator | 655,245 (26.61 %) | **655,245 (26.61 %)** |
| census/gate disagreement | 0 | **0** |
| total blocked functions | 1,807,326 | **1,807,326** |
| distinct blocker keys | 697 | 723 |
| functions re-keyed | | 18,690 |
| **sum of all key deltas** | | **exactly 0** |

Without it the row carries no `-whole` bit and cannot be ranked at all; with it,
the row splits into the table at the top of this document. That split is the
only reason this measurement could be made.

## Found and not taken

Ranked, frame axis applied first because it is free.

1. **Class B is now the *only* thing between this port and 6,760 complete bodies
   in this row alone**, on top of W41's 6,463 — and reading 4 above, it is a
   bigger rung than "a frame plus a save pair": the comparison spines it needs
   take a callee-saved register as an operand and are a family this port has
   never emitted. It should be sized with the register-register spines counted.
2. **`recv-intrinsic-vbase-upcast` — 699, all `calls-1`, all `!=`.** The only
   part of this row that is *not* Class B, and it is a **receiver** widening, not
   a comparison one: `try_parse_member_tail_call`'s `eat_receiver_this` requires
   a plain `B9 <tok> <ptr4>` and declines the upcast intrinsic before the post-op
   is ever read. Whether it then lands depends on its right-hand side, which
   nobody has decomposed. First candidate for a cheap next rung in this family.
3. **A pointer-typed literal right-hand side.** `bool f(const S* p){ return
   p->s() == 0; }` is Class A and its spine is the ordinary `== 0` fold
   (`cntlzw 11,3 · rlwinm 3,11,27,31,31`) — captured. The declined production
   refused it only because
   `eat_cmp_postop` requires the literal's TYPE to be exactly `int`/`unsigned`
   (signedness has to be resolved to pick a spine *and* a stride). Its size on
   the workload is **not measured**, and it is the one variant that might not be
   Class B.
4. **`-and-branch-more`, 9,490 of this row.** `if (p->m() != x)`. Same
   conclusion as W37's `&`: basic blocks before operators.
5. **The riskiest thing left unmeasured.** The declined production's *free
   function* head (`return g(a) > 0;`) has **0** witnesses on the workload — the
   same asymmetry W41 recorded for `eat_call_postop`, one rung later and on the
   same region. If Class B revives this production, the free head will still have
   no workload grading and its only evidence will be fixtures. Either grade it
   with a generated axis from the start or do not admit it.

## Reproduction

Both measurements are one command each, and both carry their own controls —
the spine grid compiles leaf and framed in the **same TU**, the stride rows read
the anchor's own stride out of the object rather than from the flags string.

```sh
# readings 1 and 2 — the 48-cell spine grid, /Ox and /O1, one TU each.
# Prints `.` for a cell that depends only on (relation, signedness, k==0) and
# names the axis for a cell that does not. Expect exactly 2 of 24 per mode.
scripts/gt_cmp_spine.py

# reading 3 — the label-counter surcharge, seed-free and in-TU.
scripts/gt_cmp_spine.py --stride --mode '/Ox /GS- /Gy /c'
scripts/gt_cmp_spine.py --stride --mode '/Ox /GS- /c'

# reading 4 — the Class B body, read rather than inferred
printf 'struct S { int m() const; };\nbool f(const S* p, int k){ return p->m() == k; }\n' > /tmp/b.cpp
scripts/gt_capture.sh /tmp/b.cpp /Ox /GS- /Gy /c && scripts/gt_dump.py /tmp/b.obj --text-only

# the decomposition: the frame axis crossed with the granted row
c2rs gap --list work/dc3-workload/files.txt --flags-file work/dc3-workload/flags.txt \
    --cwd ../dc3-decomp --jsonl work/W42/scan.jsonl --jobs 16
#   then sum `fn_frames` over the keys matching `-then-cmp-*whole*`; the three
#   `calls-N|<key>` buckets are already in every scan record.
```
