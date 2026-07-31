# W35 — the RUN of byte-offset adds, at both of the sites that implement it

    Tag:       W35
    Slug:      offset-run
    Date:      2026-07-31
    Fixtures:  w35_offset_run.cpp w35_offset_run_neg.cpp
    Census:    575,284 → 581,791 (23.36 % → 23.63 %), +6,507  [on the merged tree]
    Record:    this document

`expr-op-0x27` was the head of the blocking histogram at **461,786 functions,
18.8 % of the denominator** and the largest key by a factor of 1.65 over the
next. This rung takes **5,161** of it, plus **1,346** from
`expr-intrinsic-base-member-addr`, and the far more useful output is the
**map of where the other 455,279 go** — measured, not sampled, in §"The row,
mapped" below.

## What it admits, and what it refuses

`p->mid.in.b` is not one byte-offset add, it is a **chain** of them:

```text
  b9 <p> 86 43 8b 20     LOAD p                (Outer *)
  33 86 41 74 08         LITERAL int 8         offsetof(Outer, mid)
  27 86 43 86 20         byte-offset add       -> Mid *
  33 86 41 74 04         LITERAL int 4         offsetof(Mid, in)
  27 86 43 8b 20         byte-offset add       -> Inner *
  33 86 41 74 04         LITERAL int 4         offsetof(Inner, b)
  27 86 43 f4 08         byte-offset add       -> int *
  30 86 41 74            indirect load         -> int
                                               => lwz r3,16(r3) ; blr
```

c2 folds the whole chain into the single `lwz` displacement. The indirect-load
leaf admitted **exactly one** add and refused the rest.

**That limit was neither measured nor shared, and it is `GAPS.md` §6's recurring
shape in its coverage-costing form.** The *address* leaf has folded an arbitrary
run since it was written — its own comment records the capture, `&s->arr[2]` is
`LIT(40) 27 · LIT(8) 28` and emits one `addi r3,r3,48` — and the *store* leaf
inherited that same walk when it was built on the shared designator. Only the
load leaf kept a private single-add copy. One rule, three implementations, and
the third was missing a widening the other two already had.

The fix is a one-locator repair rather than a fourth copy:
[`designator::eat_offset_adds`] is the walk both readings now use, and
`eat_addr_offset_adds` is a two-line wrapper over it. The load side needs one
thing the address side does not — the **last** `27`'s TYPE, because `27`
re-types the address and so states a second time what width the following `30`
will load — so that is the walk's extra return value and nothing else changes.

**Only the last `27` may be asked, and that is a measurement, not a
simplification.** An intermediate one re-types to a pointer to the *enclosing
sub-object*, and an aggregate pointer's tag width nibble is the pointer's own
alignment rather than the aggregate's size: a pointer to a 24,004-byte struct
carries `86 43`. Pinned by
`only_the_last_offset_add_announces_the_pointee_width`, which retypes the first
`27` to `char *` and requires the parse to be unchanged, then retypes the last
one and requires it to refuse.

### The second site — "estimate the fix, not the finding", applied in advance

The run rule has **two** call sites in `leaf_load.rs`. A member inherited from a
non-virtual base is not a `27` at all — c1xx computes its address with intrinsic
2117 `base-member-addr` — and a further `.sub` chain then follows it as ordinary
offset adds. `try_parse_base_member_load` folded none of them.

`GAPS.md` §6 records that the `66` class-pair descriptor fix realized **2.4×**
its estimate because a second site implementing the same rule was found only
after the fact. Here the second site was measured as its own counterfactual
before either was shipped: **+1,346**, and the two are exactly additive
(5,161 + 1,346 = 6,507, and the two blocker rows fall by exactly those amounts
with no third key moving).

### Refused, with the measured cost of each refusal

| refusal | why | cost on the 878-TU workload |
|---|---|---|
| the SUM outside the signed 16-bit displacement | c2 emits `lis`/`addi` or `lwzx`; each individual add still fits, which is why the gate is on the total | **16 functions**, measured by lifting the gate to 32 bits and rescanning |
| arithmetic after the folded load (`p->mid.in.b + 1`) | the load lands in the SCRATCH register (`lwz r11,k(r3) ; addi r3,r11,1`) and `* 3` is strength-reduced | **214 functions** (198 with one indirect load, 16 with two) |
| a `#pragma pack(4)` 8-byte member behind a 4-byte tag | the tag carries ALIGNMENT and the kind carries SIZE — `GAPS.md` §6's third live mis-emit, which folded an `lwz` at the wrong offset | not separately measured; the pair is simply absent from `SIZED_PTEE` |
| a second DEREF (`p->m->in.b`) | a second `30`, not a longer run: the intermediate pointer must be materialized | the walk stops at the first non-offset-add token, so this is structural |
| a VARIABLE index in the chain | not a `33 <literal>` at all, so it never enters the walk; really is `slwi`/`lwzx` | structural |
| a `volatile` base pointer | the pointer is a volatile object and c2 homes it in the frame; `GAPS.md` §6's thirteenth live mis-emit, gate predates W35 | 0 (W32 measured it) |
| a base member reached in the MIDDLE of a chain (`p->d.b0`) | a run BEFORE the 2117 designator does not compose — the intrinsic re-reads the object pointer from inside its own argument list, so it is a different production | **UNMEASURED** — see "Found and not taken" |

`w35_offset_run_neg.cpp` carries one case per row and censuses **0/8**.

**The displacement gate was checked in the under-claiming direction too**, which
is the direction nothing in this project tests. Lifting it in the parser alone
and rescanning gains 16 census functions **and produces a census/gate over-claim
of exactly 16** — the port's own `select_text` still refuses, because a signed
16-bit displacement field cannot encode 32768 at all. So the parser gate is not
conservatism layered on top of a codegen rule; it is the same rule stated where
census and `PortC2` cannot disagree about it, which is what keeps disagreement
at 0. Verified on a one-function TU (`work/w34/probe/p8.cpp`): with the parser
gate lifted, `c2rs census` reads **1/1 in class** against `Port=NotImplemented`.

## Estimate vs outcome

The estimate was written to `work/w34/ESTIMATE.md` **before** any scan, together
with the answer to the question W31 skipped.

**What the bucket had already been filtered by.** `expr-op-0x27` is a
first-blocker key from `parse_expr`'s fall-through. For a body to be filed there,
`.sy` binding, one-register-each formals, `LO`/`SS`/scopes, and — decisively —
**every operand consumed before the `0x27` already being in the modeled
int4/ptr4/int1u class** had all already succeeded. `eat_operand_type` admits
`86/96/A6/B6 × 43/44`, so `A643` const-pointer bases pass. Any deduction of the
form "the base pointer's type might not be modeled" was therefore **already
applied by the bucket** and could not be taken again. What was *not* filtered is
everything downstream of the `0x27`, which is where the whole uncertainty lives.

| | estimate | outcome | bias |
|---|---|---|---|
| counterfactual A (parse ceiling: admit the whole indirect-load OPERAND production in `parse_expr`, no codegen, no arithmetic guard) | **+9,000**, range 4,000–20,000, **biased HIGH** | **+6,816** | called **correctly**; HIGH by **1.32×** |
| the shippable rung | **+3,500**, range 1,000–8,000, **biased HIGH** | **+6,507** | **wrong direction** — LOW by **1.86×** |

The row-to-counterfactual gap is **67.8×** (461,786 → 6,816), against the
control-flow lane's 67× (48,102 → 718). That prior held almost exactly, and the
prediction of "~50×" was the part of the estimate that was closest.

**Where the shippable estimate went wrong is worth more than the number.** The
estimate named the wrong sub-shape. It predicted the winner would be *an
indirect load as a CALL ARGUMENT* (`return g(s->m);`), reasoning that the
argument register makes the lowering local. That sub-shape is worth **at most 7
functions on the entire workload** — measured, because counterfactual A run
*without* the chain-gate bypass gained exactly 7, and every other one of the
6,816 came through the straight-line arm. The real winner was a shape the
estimate did not consider at all: bodies that *look exactly like the leaf the
port already accepts* and were refused by a private limit inside it. The general
form: **before sizing a row's sub-shapes, check what the recognizer that already
covers the obvious one is refusing** — the largest sub-population in a big
blocker row can be a shape the port thinks it already has.

## The row, mapped

Deleting the gate and rescanning does not only size the rung; it says where the
whole row goes. `expr-op-0x27`'s 461,786 redistribute (deltas against the
baseline scan, one warm rescan):

| lands in | Δ | what it is |
|---|---|---|
| `expr-op-0x99` | **+84,407** | member/temporary bind — now the largest single successor |
| `expr-op-0x32` | **+78,983** | the STORE token: a statement store the store leaf does not match |
| `cf30-loadtype-A645` / `-8645` / `-A646` | **+53,000** | a FLOATING-POINT member at the tail of the chain |
| `expr-cmp-eq` / `-ne` / `-lt` / `-le` / `-ge` / `-gt` | **+43,000** | the load feeds a comparison |
| `expr-brfalse` / `-brtrue` | **+21,000** | the load feeds a branch |
| `expr-call-in-expr-*-off-add-*` | **+37,000** | a member call on or after the designator |
| `assign-dst-not-formal` | **+13,350** | assignment to a non-formal destination |
| **whole-body complete** | **+6,816** | this rung's ceiling |

Two things follow for the next ranking. The successors are **not** one shape —
this row is genuinely heterogeneous, exactly as `GAPS.md` §6 warns a big bucket
can be. And `expr-op-0x99` at 280,283 today would become **364,690** if `0x27`
were fully cleared, which makes it the real head of the board rather than a
distant second.

## Gate evidence

Corpus `dc3-decomp` at `05ca6d09`; baseline re-taken in this worktree and
reproducing master `6548a4e` to the function (549,148 / 2,462,571, 461,786 in
`expr-op-0x27`, mismatch 0, disagreement 0).

| lane | baseline | W35 |
|---|---|---|
| `cargo test --workspace --release` | 438 pass / 0 fail | **444 pass / 0 fail** |
| `c2rs bench` | 163 pass / 0 fail / 0 error | **165 pass / 0 fail / 0 error** |
| `scripts/mode_lane.sh /Ox` | 76 match, 0 mismatch | **77 match, 0 mismatch, 0 codegen-gap** |
| `/O1` · `/O2` · `/Ox /Gy` | 74 match, 0 mismatch | **75 match, 0 mismatch, 2 codegen-gap** each |
| `scripts/expr_sweep.sh` | 7,673 cases, 0 mismatches | **7,881 cases (+208), 0 mismatches** |
| `scripts/cross_sweep.sh` | 7,545 × 4, 0 mismatches | **7,545 × 4, 0 mismatches**, family set and configuration count bit-identical |
| 878-TU scan (branch, vs master `6548a4e`) | 549,148 / 2,462,571 (22.30 %), mismatch 0, disagreement 0 | **555,655 / 2,462,571 (22.56 %)**, mismatch 0, **disagreement 0** |
| 878-TU scan (**merged tree**, vs master `d69d6b1` = W34 landed) | 575,284 / 2,462,571 (23.36 %) | **581,791 / 2,462,571 (23.63 %)**, mismatch 0, **disagreement 0** |

> **The two rungs are exactly additive, and that was measured rather than
> assumed.** W34 (the multi-argument FP tail call, +26,136) and W35 (+6,507)
> were each scanned against the same baseline of 549,148 in their own
> worktrees, so their figures could not be added without checking: a merge of
> two independently-green branches is a new corpus. The merged-tree scan gives
> **581,791 = 575,284 + 6,507 to the function**, and the cross-product lane
> grades the combination separately (11,341 configurations × 4 mode lanes, 0
> mismatches). Additive here because the families are disjoint — W34 moves
> `calls-1` FP tail rows and W35 moves `expr-op-0x27` and
> `expr-intrinsic-base-member-addr` — but that is the conclusion, not the
> premise.
| `census fixtures/cpp/w35_offset_run.cpp` | — | **22/22 in class**, `Port=Match` |
| `census fixtures/cpp/w35_offset_run_neg.cpp` | — | **0/8 in class**, `Port=NotImplemented` |

The census delta is **1:1**: the only two keys that move are `expr-op-0x27`
(−5,161) and `expr-intrinsic-base-member-addr` (−1,346), and their sum is exactly
the gain. No bucket rises.

The new sweep axis is `scripts/sweep.d/45-offset-run.py`, **208 cases**, one file
per axis so it cannot conflict with a peer's fragment. It varies what nobody
varies: chain depth 1–6 crossed with fourteen tail types; **cv-qualification at
every level of the chain independently** (the axis `GAPS.md` §6's thirteenth
mis-emit hid behind, because cv changes no operator and no shape and *does*
change the `27` tags the width cross-check reads); `27`/`28` interleaved in every
order; the displacement crossed by the SUM rather than by any one add; the
DS-form `ld` reached by a sum; intrinsic 2117 at each position in the chain; and
the chain behind `this` at every argument position. 0 mismatches.

W35 adds **no new `census.rs` key and no new shape family** — it widens
`indirect-load-leaf`, which already existed — so `cross_sweep.sh` has nothing
new to discover and its configuration count is unchanged, which is the correct
outcome rather than a missed one. It did pick the new fragment up: the lane's
chosen representative for `indirect-load-leaf` in one slot is
`45-offset-run-0171.cpp`.

The final scan was taken on the **clean committed tree** (`36b99bb`) with
`--validate-cache 50`: 17 entries re-captured through the real toolchain and
agreed, **0 POISONED**.

The port needed **no codegen change at all** — `IlOp::LoadInd { off }` already
carries a folded displacement and `select_text` already emits it — which is why
`census/gate disagreement` stayed 0 through every counterfactual.

## Found and not taken

Ranked, with the frame axis applied (`calls-2plus | expr-op-0x27` is 116,118, so
a quarter of the residual row needs a frame before it needs anything else).

1. **`expr-op-0x99` — the real head of the board, 280,283 today and 364,690
   behind a cleared `0x27`.** Member/temporary bind. Nothing here sizes it; it
   needs its own counterfactual, and it is now the largest key by a wide margin.
2. **`expr-op-0x32` at 80,797 behind the lift** (1,814 today). A statement-level
   store that the store leaf does not match. The store leaf already folds the run
   and already handles the designator, so the gap is in what is being *stored*,
   not in where — this is the closest thing to a cheap next rung that this
   measurement found.
3. **A floating-point member at the tail of a chain: ~53,000** across
   `cf30-loadtype-A645`/`-8645`/`-A646`. `lfs`/`lfd` from the other register
   file, and `wt-fp-multiarg` owns that seam.
4. **A run of offset adds BEFORE the intrinsic-2117 designator — UNMEASURED.**
   `p->d.b0` where `d` is a member and `b0` is inherited. It does not compose
   with the current decoder because `parse_base_member_designator` re-reads the
   object pointer from inside its own argument list, so this needs the intrinsic
   to accept a pre-computed base rather than a token. **This is the riskiest
   thing this rung left unmeasured** — it is the one place where the fix has a
   *third* site and I did not size it, which is precisely the shape §6 says
   recurs.
5. **Arithmetic over an indirect load: 214.** Below the point where the r11
   scratch rule is worth characterizing, and `IlOp::LoadInd`'s own comment
   records that `*p * 3` is strength-reduced, so the rule is not one instruction.
6. **The sum outside the 16-bit displacement: 16.** Noise floor; the refusal is
   correct.
