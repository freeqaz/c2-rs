# WCL — `return p->a()->b(k);`, and the marshalling that runs the other way

    Tag:       WCL
    Slug:      chain-link-arg
    Date:      2026-07-31
    Fixtures:  wcl_chain_link_arg.cpp wcl_chain_link_arg_neg.cpp
    Census:    673006 → 685098 (27.33 % → 27.82 %), +12,092
    Record:    this file

WCH's direct successor, and its whole residue. WCH shipped `p->a()->b()` — the
chain with every link nullary, Class A, **no codegen at all** — and refused an
argument on a later link under two keys it had split and sized on the way past:
`mcall-chain-link-args` **12,090** and `mcall-chain-link-arg-lit` **2**. This
rung is both, and the realized figure is **12,092**.

## The three facts, and none of them follows from the others

Read off `work/WCL/probe/p1.cpp`–`p3.cpp`, `/O1 /GS- /c`:

```text
  int f(O* p,int k)       { return p->Next()->gia(k); }     52 B — CLASS B
    mflr r12 ; stw r12,-8(r1) ; std r31,-16(r1) ; stwu r1,-96(r1)
    mr r31,r4 ; bl ?Next ; mr r4,r31 ; bl ?gia
  int f(O* p)             { return p->Next()->gia(7); }     40 B — CLASS A
    …                      bl ?Next ; li r4,7 ; bl ?gia
  int f(O* p,int j,int k) { return p->Next()->gia2(j,k); }  68 B — two saved
    mr r31,r4 ; mr r30,r5 ; bl ?Next ; mr r4,r31 ; mr r5,r30 ; bl ?gia2
```

### 1. The argument goes to r4

Slot 0 is the receiver a `bl` has just left in r3, so a link's explicit
arguments start at slot **1** (`c2_il::LINK_FIRST_SLOT`). Emitting them at slot
0 instead is **156 of the sweep fragment's 183 cases** wrong.

### 2. The formal cell is Class B and the literal cell is not

A formal is live across the previous `bl` and takes a callee-saved GPR; a
literal costs no register, so an all-literal link keeps WCH's three-word
prologue. `plan_saved_gprs` computes this correctly and always did — WCH's claim
held — but only once it is **shown** the link's arguments. That is one loop in
one function, and it is the difference between a `std r31` prologue and a wrong
one.

### 3. The emission order is ASCENDING, and it is the opposite of the port's

This is the rung's real content and it was nearly missed. Every other call in
the port marshals **highest destination first** (`moves_descending`, shipped
since #35 rung 2). A chain link marshals **lowest first**. Both sides measured
in one probe TU:

| body | moves |
|---|---|
| `void f(int a,int b,int c){ v1(a); g2(c,b); }` | `mr r4,r31 ; mr r3,r30` |
| `void f(int a,int b,int c){ v1(a); g2(b,c); }` | `mr r4,r30 ; mr r3,r31` |
| `void f(int a,int b){ v1(a); g3(a,b,5); }` | `li r5,5 ; mr r4,r30 ; mr r3,r31` |
| `int f(O* p,int j){ v1(j); return p->gia(j); }` | `mr r4,r30 ; mr r3,r31` |
| `int f(O* p,int j,int k){ return p->Next()->gia2(j,k); }` | `mr r4,r31 ; mr r5,r30` |
| `int f(O* p,int j,int k){ return p->Next()->gia2(k,j); }` | `mr r4,r30 ; mr r5,r31` |
| `int f(O* p,int j,int k){ return p->Next()->gia3(j,5,k); }` | `mr r4,r31 ; li r5,5 ; mr r6,r30` |

**The fourth row is the one that matters.** It is a *member* call, its `this` is
saved, and it comes out descending with the free functions — so the axis is not
"member calls go the other way". It is whether the argument list starts at slot
0. Reusing `moves_descending` for the link — which is what "it is the same
marshalling, share the locator" produces — is **72 of 183** wrong.

Literals interleave in the same order rather than being grouped, in both
families, which is why the emitter walks the slots once instead of emitting the
moves and then the constants.

### The locator: separate on purpose, and the check in both directions

`link_arg_slots` is a sibling of `tail_call_shape`, not a parameterization of
it. The two disagree about **every** rule they both have an opinion about: the
slot base; whether a permutation is possible (a link's sources are the
callee-saved file and its destinations the argument file — two disjoint sets, so
there is no cycle to break); a repeated argument (`p->Next()->gia2(j,j)` is
`mr r4,r31 ; mr r5,r31`, two ordinary moves, not the dead `mr r11`
`call-arg-duplicated` refuses); and the emission order. Sharing the code would
have been sharing a name with a rule.

The other direction — a shared locator nobody asks — does not apply:
`link_arg_slots` has one caller and `LINK_FIRST_SLOT` has two, in **two crates**,
and that is deliberate. The slot base is what the IL parser bounds the list with
and what the emitter picks a register with, and those agreeing is the whole
reason the census cannot claim a body the gate declines. Grepping the other
readers of a call's argument region finds `tail_call_shape` (above),
`leaf_fp_tail` (its own argument grammar, integer vocabulary cannot spell an FP
value, unreachable from here) and `eat_call_args` itself, which this production
already shares.

## Estimate vs outcome

Written to `work/WCL/ESTIMATE.md` before any scan.

| | |
|---|---|
| estimate | **+12,090** (the ceiling), range 9,000–12,090, bias *at* the ceiling |
| realized | **+12,092** |
| ratio | **1.0002× low** — the closest a rung estimate has come on this board |

Against the ceiling actually built — both cells, 12,090 + 2 — it is **1.0000×**.

**The pre-filter, named.** `mcall-chain-link-args` is not a first-blocker *row*
of the completeness walk; it is a key `try_parse_member_chain_call` raises at
`Err(Some(..))`, i.e. **after the body has parsed to the end of the segment**.
So it had already been filtered by: being a chain of depth ≤ 8; an innermost
receiver that is a plain `B9 <formal>` load; every `99` bind's value a width-4
pointer; the body **ending** at the chain (`4B` / `41`); and a non-`float`
discarded result. It had **not** been filtered by argument count, argument kind,
the innermost call's arity, the formals count, or argument type.

**Seven independent refusals were counted between the ceiling and the emitter,
and every one of them cost ZERO.** Measured, not argued: the baseline and the
post-change blocker histograms differ in exactly two entries.

```text
  -12090  mcall-chain-link-args:eof
      -2  mcall-chain-link-arg-lit:eof
  total blocked delta -12,092;  census delta +12,092
```

Not one function moved to another key in either direction. `-computed`,
`-nonformal`, `-lit-wide`, `-overflow`, `callseq-three-plus-saved`,
`callseq-over-eight-formals` and `callseq-saved-with-first-call-setup` are each
worth **0 functions** on this row — every gate this rung declares is exercised
only by `wcl_chain_link_arg_neg.cpp` and by the sweep. The row was completely
homogeneous, which is why discounting the ceiling for those seven would have
been the WCB error a second time.

## Which of §6n's five categories this row was

**None of them, and that is the finding.** It was not a private limit nobody had
noticed (1), not misfiled under an opcode (2), not smaller than its size (3),
not unmeasurable (4), and not mis-described (5) — it was **declared, sized,
split and named by the previous rung on its way past**, with the counterfactual
already run.

That is a sixth category, and it is the only one in which an estimate is
cheap: §6n's five are all things a first-blocker histogram cannot tell apart,
and this row was never diagnosed from a histogram at all. The generalization is
not about this row but about WCH: **a rung that instruments its own boundary
hands its successor an exact ceiling**, and two consecutive estimates have now
landed (1.061×, 1.0002×) after eight consecutive misses. The cost was one
predicate and one rescan in WCH.

## The sweep axis, and its separation counts

`scripts/sweep.d/98-chain-link-arg.py`, **183 cases**. Three mutations of the
shipped rule, each graded on the whole fragment:

| mutation | mismatches |
|---|---|
| emit the link's marshalling **descending** (reuse `moves_descending`) | **72** |
| put the link's arguments at slot **0** | **156** |
| read the link's argument region **forwards** instead of reversed | **68** |

All three separate. The third is the one a naive grid cannot reach: it needs two
**distinct** formals in a **transposed** pair, because `gb2(j,j)` and `gb2(j,k)`
with the identity ordering agree under it. The first needs two or more link
arguments at all. The second needs a case where the argument's source register
is not also its destination — which is why the fragment pads with leading
formals: with one parameter, r4 is both "argument slot 1" and "where `params[1]`
arrives", and the two facts are indistinguishable.

## What was refuted

**`calls-2plus` is not a frame class, now shown from the OTHER side.** WCH's row
was 100 % `calls-2plus` and 100 % **Class A**. This row is 100 % `calls-2plus`
and 99.98 % **Class B** — 12,090 of 12,092 save a GPR. Two adjacent rows of one
production, one axis, opposite answers. The axis carries no information about
the frame class whatsoever, and this is the third measurement saying so.

**Crossing the row with `calls-N` and `cflow-*` killed nothing, again.** Both
axes are degenerate on it — 12,092 = `calls-2plus` = `cflow-straight`, exactly,
same as WCH. Run it (it is free, and it eliminated 134,763 functions once), but
a row surviving it is still not thereby ranked.

**"Reuse the shipped marshalling locator" was wrong, and only a capture said
so.** Nothing in the shape suggests two orders. `moves_descending`'s own doc
comment carries a capture that is *still correct*, and reading it is what makes
the wrong answer look obviously right. It took compiling the two families side
by side in one TU.

## Gate evidence

| lane | result |
|---|---|
| `cargo test --workspace` | **480 pass / 0 fail** (was 475) |
| `c2rs bench` | **182 pass / 0 fail / 0 error** (was 180) |
| `scripts/mode_lane.sh` `/Ox` / `/O1` / `/O2` / `/Ox /Gy` | **86 / 84 / 84 / 84**, 0 mismatch in all four (was 85/83/83/83) |
| `scripts/expr_sweep.sh` | **12,612 cases, 0 mismatches** (12,429 before) |
| `scripts/cross_sweep.sh` | **14,184 × 4 configurations, 0 mismatches** — count unchanged, because the rung declares no new shape family: a chain link is still `BodyShape::CallSeq` |
| 878-TU workload scan | mismatch **0**, census **685,098 / 2,462,571 = 27.82 %**, disagreement **0** |
| fixtures, `c2rs census` | `wcl_chain_link_arg.cpp` **27/27**, `wcl_chain_link_arg_neg.cpp` **0/13** |

An **instrument defect of my own making** is worth recording, because it is
§6n's fifth shape exactly: the first cross-sweep run died with a JSON decode
error, and the cause was two cross sweeps sharing one output directory — I
started one in the background and then `rm -rf`'d its outdir out from under it.
The re-run in a fresh directory is the 14,184 × 4 above. The lane does not fail
closed against this; `expr_sweep.sh` refuses a shared outdir and
`cross_sweep.sh` does not.

## Found and not taken

Ranked, and **with the obj read** rather than the key name trusted — which is
item 1's entire lesson from WCH, where the second-largest key turned out to name
an argument region rather than an operator.

1. **The chain result with ONE instruction after it — the `-off-add` family,
   10,568 + 3,676.** `expr-call-in-expr-chained-then-type-ptr-and-off-add-more`
   is **10,568** and unchanged; `-then-type-ptr-and-op-more` fell from 15,049 to
   **3,676** when this rung absorbed the 11,373 WCH had identified inside it.
   Probed (`work/WCL/probe/p4.cpp`), the construct is one word:

   ```text
     int  f(O* p) { return p->Next()->gf()->m; }    bl ; bl ; lwz r3,4(r3)
     int* f(O* p) { return &p->Next()->gf()->m; }   bl ; bl ; addi r3,r3,4
   ```

   The **second is already emitted**: `addi r3,r3,k` is exactly
   `SeqTail::CallValue { add_k }`, which has shipped since #35 rung 1, so that
   cell is a recognizer and nothing else. The first needs one new tail variant,
   `lwz r3,k(r3)`, and the offset-0 case emits nothing at all. This is the
   cheapest large thing on the board and the first successor that is *two*
   already-built emitters wearing an unfamiliar key.
2. **The innermost receiver's designator — 5,188, unchanged from WCH's
   ranking**, all `-whole`, one production each:
   `-recv-field-off0-then-chain-bind-whole` **2,666**,
   `-recv-intrinsic-this-adjust-then-chain-bind-whole` **1,686**,
   `-recv-field-then-chain-bind-whole` **836**. Each already has a lowering
   elsewhere in the port; the `-off0` one emits nothing for the designator and
   is both the cheapest and the biggest.
3. **`-chained-then-call-recv-intrinsic-this-adjust-and-off-add-more` — 1,829**,
   and **`-then-type-int1-and-type-aggregate-whole2/3` — 2,211**. The second's
   `-whole2`/`-whole3` means two or three constructs must be admitted together,
   so its ceiling is not its size.
4. **The riskiest thing left unmeasured, and it is specific.** *The ascending
   order has no explanation, only a boundary.* Nine captures separate the two
   families cleanly, and no rule was found that produces both — the port draws
   the line at "does this call's argument list start at slot 0", which is a
   **description of the two probe families and not a mechanism**. Inside the
   accepted class the two candidate readings are inseparable: "the list is based
   at slot 1" and "slot 0 needs no instruction" are the same set of bodies,
   because every slot-0-based call in the class does need one (its value comes
   out of a callee-saved register). The first widening that pulls them apart —
   a link whose receiver needs a `this`-adjust, say, item 2 above — would be
   graded against a rule fitted to a confound, and neither reading is more
   likely than the other from here. **Measure that before widening the receiver
   side**, not after.

   Two lesser ones, both inherited: every one of the 12,092 realized functions
   lives in a TU the port never emits (`vocab-gap` ~99 %), so not one has been
   compared byte for byte; and the widest *graded* link carries two saved GPRs
   plus literals, because three formals on a link is the `__savegprlr_29` helper
   class and refuses — so slots r7…r10 are graded by generated all-literal cases
   alone.
