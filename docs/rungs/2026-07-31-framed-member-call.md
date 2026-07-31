# W41 — the row was not the construct it was named for, and a refusal justified by an argument

    Tag:       W41
    Slug:      framed-member-call
    Date:      2026-07-31
    Fixtures:  w41_framed_member_call.cpp w41_framed_member_call_neg.cpp
    Census:    639,387 → 643,385 (25.96 % → 26.13 %), +3,998
    Record:    this document

The brief's target was `expr-call-in-expr-recv-load-whole` — **10,494 functions,
10,463 of them `calls-1`** — scheduled, on W36's own handoff, as *"a member call
preceded by assignment statements … the single cheapest thing on this list"*.

**The row contains zero of those.** Not few: none. A member-call *production*
first-blocker histogram over the 878-TU workload decomposes it 1:1, and the whole
of it is one statement long.

## The decomposition, and it is exact

| | functions | what it actually is |
|---|---:|---|
| `t-classb-load-lit-scale` | **6,463** | `return p->m() + n*k;` — a value **live across the call** ⇒ Class B |
| `t-postop-sub-lit` | **3,559** | `return p->m() - k;` — a **framed** call with a literal post-op |
| `g-OK-after-recv-cast` | **440** | `p->m();` whose receiver carries a `2C` **pointer conversion** — the production accepted everything else about it |
| residue | 32 | two member calls in one body (28), and three one-off tails |
| | **10,494** | |

The shipped rung takes the second and third rows. Predicted from the instrument:
**3,999**. Realized: **3,999**, of which one is refused by a new gate, so
**+3,998**. The census delta is 1:1 — the only key that falls is the row
(−3,999) and the only key that rises is the new gate (+1). No pre-existing bucket
moves at all.

### The C++ behind it, because the row's name hides it completely

The 10,022 single-statement bodies are **container iterator arithmetic**, inlined
from headers into ~1 in 3 TUs and therefore counted once per TU:

```text
  b9 …  a6 43 …          the receiver
  99 …  bd … 4c          p->end()
  33 86 41 12 14 03      … - 20        (sizeof(element) == 20)
  41 86 43 …             … and returned
```

`end() - 1` and `begin() + i` — the first is a constant fold into one `addi`, the
second needs the index to survive a `bl`. That split is the whole row, and
nothing in the key `…-recv-load-whole` says so.

## Two widenings, and neither needs any codegen

**The framed member call.** `this` is argument zero exactly as it is for the tail
form, so `return p->m() - k;` is the shipped 0x24-byte `BodyShape::FramedCall`
with a negative immediate: `select_text` computes the receiver's `mr r3,rN` from
the same `params`/`ops` pair the free-function form uses, and the frame,
`.pdata`, `/Gy` COMDAT and REL24 are the code that already grades. MEASURED,
every word read off the reference obj before any of it was written
(`work/w41/probe/p1.cpp`, `p5.cpp`, `/O1 /GS- /c`):

```text
  int f(A* p)             { return p->gi() - 20; }    bl ; 3863ffec addi r3,r3,-20
  int f(A* p)             { return p->gi() + 20; }    bl ; 38630014 addi r3,r3,20
  int f(int k, A* p)      { return p->gi() - 20; }    7c832378 mr r3,r4 ; bl ; addi
  int f(int j,int k,A* p) { return p->gi() - 20; }    7ca32b78 mr r3,r5 ; bl ; addi
  E*  f(A* p)             { return p->ge() - 1;  }    bl ; addi r3,r3,-20  (sizeof E)
  int f(A* p)             { return p->gi() + 0;  }    48000000 b ?gi    — the FOLD
  int f(A* p)             { return p->gi() - 40000;}  3c63ffff addis + addi — REFUSED
  int f(A* p,int a,int b) { return p->ga(b) - 20; }   mr r4,r5 ; bl ; addi — REFUSED
```

**The receiver's pointer conversion.** `2C <ptr4 TYPE> 00` between the receiver
LOAD and the `99` bind. It moves no register. *Which* C++ produces one was
measured rather than assumed (`p2.cpp`, `p4.cpp`): a C-style or `static_cast` to
the receiver's own type is folded away and emits no `2C` at all; cv-qualification
on the pointer, the pointee or the method emits none either; and a base-class
adjustment is `intrinsic 2113`, a different production with a different lowering.
What is left is a cast that changes the pointee without changing the address —
`const_cast<S*>(p)->m()` and `((S*)v)->m()` from a `void*`.

## The refusal that was an argument rather than a byte

`mvp_call_submod.cpp` — `int f(int a){ return g(a) - 1; }` — sat in the
**honest-rejection** lane from W4b2 until today, on this stated ground:

> c2 does **not** canonicalize `-1` to `+(-1)`. Subtraction is non-commutative
> and off the verified 0x24-byte ADD frame.

That is false. `- k` and `+ k` are the **same instruction** with a different
immediate: `3863ffec` against `38630014`. The fixture now grades in the
byte-exact lane and its pinned segment moved from
`parse_segment_rejects_all_out_of_class_call_shapes` to
`parse_segment_accepts_framed_call`, with nothing changed but the measurement.

**Why nothing ever contradicted it: the refusal costs 0 free-function bodies on
the 878-TU workload.** The row that pays for it is 3,559 *member* calls, because
a container's `end() - 1` is written with a subtraction and a free function's
`+ k` is not. So the locator's only consumer never saw the case, and the case's
only population never reached the locator. That is `GAPS.md` §6's "one fact, one
locator" in its third form this session — after W35's *private copy that refuses
more than its siblings* and W38's *shared locator nobody else asks*, this is **a
shared locator with one consumer, whose gate was fitted to that consumer's
population**. The repair is the same shape: the post-op region is now
[`eat_call_postop`], both call productions read it, and `03` is in it.

The neighbour it was grouped with is genuinely out of class and stays refused:
`* k` strength-reduces to a shift/add sequence and is not one `addi`
(`n_mul`).

## Refused, with the measured cost of each refusal

| refusal | why | cost on the 878-TU workload |
|---|---|---|
| a value **live across the call** (`return p->m() + n*k;`) | c2 saves the formal in r31 — `std 31,-16(1) ; mr 31,4 ; bl ; mulli 11,31,20 ; add 3,3,11 ; ld 31` — which is **Class B**, a frame class this port does not have | **6,463**, and it is 62 % of the row |
| an explicit **argument** beside the receiver, under a frame | `FramedCall` carries one operand stream, so it can spell "put this formal in r3" and nothing else; c2 does emit a permutation under a frame | **1** — and see the under-claiming check, which is the whole point of this row |
| a literal past the signed-16-bit `addi` immediate | `± 40000` is `addis` + `addi`, a second instruction and a longer body | **0** on this row; shared with the free-function form |
| a `* k` post-op, or two post-ops | strength-reduced / more than one literal | **0** on this row |
| a `float`/`double` result with a post-op | `fadds`, not `addi`, and the TU carries `_fltused` | **0**; refused at the shared `41` result type |
| a `volatile` receiver | a memory object: c2 homes the parameter and reloads it | **0**; inherited from the shared operand-type locator (§6 #13) |
| a receiver that is not one of this function's formals | the framed path emits a register *move*, and a global is a load | **0** |
| nine formals | past the eighth the parameter is stack-homed | **0**; the same key the free-function framed call raises |
| a conversion that does not preserve the pointer class | a value change, not an address reinterpretation | **0** |

`w41_framed_member_call_neg.cpp` carries one case per row and censuses **0/15**.

### The gate checked in the UNDER-claiming direction, and the workload cannot see it

The direction nothing here tests, and this time the answer is the interesting one.

Lifting `mcall-framed-args` in the parser alone gains **exactly 1** census
function over 2,462,571 bodies and produces **0** census/gate disagreement — the
single body in that population is an *identity* permutation, which the port
happens to emit correctly. Read from the workload alone, the gate looks like pure
conservatism worth deleting.

It is not. One source token away,
`int f(S* p, int a, int b) { return p->ga(b) - 20; }` becomes
**`Port=Mismatch @ offset 8`** with the gate lifted, because the port drops the
`mr r4,r5` the reference emits. **A 2.4-million-function scan's disagreement
counter is blind to this gate**, and it took a two-line probe to see it. That
body is now `n_argperm` in the negative fixture.

## Estimate vs outcome

Written to `work/w41/ESTIMATE.md` **before** any scan or counterfactual, with the
pre-filter named.

**What the bucket had already been filtered by.** To carry the key
`expr-call-in-expr-recv-load-whole` a body must have bound `.sy`, passed
`formals_are_one_register_each`, had its first blocker classified as a member call
with a plain `B9 <tok> <ptr4>` receiver, **and been accepted end to end by
`body_matches`** with the call admitted as a value and nothing else new. So every
TYPE in the body is int-like or ptr4; there are no branches, no compares, no `30`
indirect loads, no `27` offset adds, no free calls, no intrinsics. And the frame
axis — W37's free cross, already in the brief — leaves 10,463 of 10,494
`calls-1`, so it removes 0.3 % of the row rather than 99 %. What was **not**
filtered is where the call sits in the statement list, and everything from the
`4C` rightwards.

| | estimate | outcome | bias |
|---|---|---|---|
| counterfactual A (parse ceiling for the widenings) | **+7,000**, range 4,000–10,200 | **+3,999** | direction **right**; HIGH by 1.75× |
| the shippable rung | **+5,000**, range 2,000–12,000, "expect the outcome ABOVE this" | **+3,998** | HIGH by 1.25×, **inside the range** |

**The first estimate in six rungs that is inside its own range**, and the reason
is not skill at enumeration — the enumeration was wrong in exactly the way W35
and W38 were wrong, and is recorded as such below. It is that the point estimate
came from anchor (B), source-language reasoning, which is the same anchor W36
found "closest of the three" and then did not trust.

**The stated bias direction was wrong and the reason it was stated is worth
keeping.** The estimate argued the outcome would land *above* +5,000 because "a
`-whole` row is a bound on the row, not on the widening" — W38's +36,684 came out
of two keys neither of which was the row it was scheduled against. That mechanism
is real and it did not fire here: **the widening reached exactly zero functions
outside the named key.** Measured, and it is a new number for the ranking:

| row kind | row → realized |
|---|---|
| first-blocker row (`expr-op-*`) | 67× · 67.8× · 13.4× |
| counterfactual successor (W38) | 1.45× |
| **`-whole` first-blocker key (W41)** | **2.62×**, and counterfactual → realized is **1.0002×** |

A `-whole` key is the tightest bound this project has, and it is tight in *both*
directions: it did not overstate the reachable population by much, and it did not
understate the widening's reach at all. The reason is structural — `-whole` is
"the entire segment parses with this one form admitted", so a body outside the key
is outside it for a reason that no widening of the form can remove.

**The sub-shape enumeration was wrong again, and this is the third rung running.**
The estimate's anchor (B) guessed the two dominant shapes as
`int r = p->Get(); return r;` (worth 0) and `p->Do(); return <literal;>` (worth
0), and named the *prefix* case it was scheduled for as "probably under a third
of the row" (worth 0). The two shapes that carry the whole row —
`return p->m() - k;` and a `2C` on the receiver — appear nowhere in the estimate.
The point estimate was right by coincidence of magnitude, not of content.

What did work, and it is repeatable: **do not enumerate sub-shapes at all —
instrument the production.** W38 built a store-production first-blocker histogram
and called it "the whole method of this rung". The same instrument here, keyed on
the *production* rather than on the byte, predicted the realized number to within
one function before a line of the rung was written (§Reproduction). The estimate
discipline's remaining value is in naming the pre-filter, which it did correctly.

## The alarm, and it was the instrument rather than the port

The first baseline scan of this worktree reported **6 mismatching TUs**, which
outranks all widening work. It reproduced on re-run, and `--no-cache` made it
vanish.

**The capture cache is not relocatable, and its key does not say so.** A cached
reference obj embeds the cache directory's absolute path (the `-Fo` argument c2
is driven with), while the cache *key* is content + flags + toolchain + workload
identity. Reflink-copying a warm `work/capture-cache` from the main repo into a
worktree — cheap, obvious, and 4 GB of otherwise-wasted captures — therefore
serves reference bytes captured under a different path, and the port's fresh emit
differs from them by exactly the path-length difference: **`ref 740 B, port 768 B`,
first divergence at offset 8**, the COFF `PointerToSymbolTable`. It looks exactly
like a port regression on six unrelated TUs.

`--validate-cache` catches it and names it precisely (`c2 argv differs`, with both
paths printed), but it is **off by default**, so the default reading of a
copied-cache scan is a false alarm. Re-capturing in place restores the baseline
exactly: 639,387 / 2,462,571 = 25.96 %, mismatch 0, disagreement 0. Nothing in the
repo did this — `scripts/configure_existing_worktree.sh` deliberately does not
copy the cache — but nothing warns against it either, and the brief describes the
cache as SHARED. Booked in "Found and not taken".

## Gate evidence

Corpus `dc3-decomp` at `05ca6d09` (dirty, as recorded in every provenance line);
baseline re-taken in this worktree with a locally-captured cache and reproducing
master `62ade68` exactly.

| lane | baseline (`62ade68`) | W41 |
|---|---|---|
| `cargo test --workspace --release` | 460 pass / 0 fail | **463 pass / 0 fail** |
| `c2rs bench` | 172 pass / 0 fail / 0 error | **174 / 0 / 0** |
| `scripts/mode_lane.sh /Ox` | 80 match, 0 mismatch | **82 match, 0 mismatch, 0 codegen-gap** |
| `/O1` · `/O2` · `/Ox /Gy` | 78 match, 0 mismatch | **80 match, 0 mismatch, 2 codegen-gap** each |
| `scripts/expr_sweep.sh` | 10,996 cases, 0 mismatches | **11,242 cases (+246), 0 mismatches** |
| `scripts/cross_sweep.sh` | 11,761 × 4, 0 mismatches | **11,761 × 4, 0 mismatches**, family set and configuration count bit-identical |
| 878-TU scan | 639,387 / 2,462,571 (25.96 %), mismatch 0, disagreement 0 | **643,385 / 2,462,571 (26.13 %)**, mismatch 0, **disagreement 0** |
| `census fixtures/cpp/w41_framed_member_call.cpp` | — | **25/25 in class**, `Port=Match` |
| `census fixtures/cpp/w41_framed_member_call_neg.cpp` | — | **0/15 in class**, `Port=NotImplemented` |

Final scan on the committed tree with `--validate-cache 50 --replay-every 25`:
17 entries re-captured through the real toolchain and agreed, **0 POISONED**;
replay soundness 36 checked, **0 diverged**.

The three new tests are `mcall_tail`'s, each pinned to a verbatim capture: the
framed shape with its negative immediate, the `± 0` fold in both spellings
against the `* k` refusal, and the receiver conversion. Two further changes are
invisible in the count because they are *moves* — `mvp_call_submod.cpp` from the
honest-rejection differential lane to the byte-exact one, and its pinned segment
`GA_SUBMOD` from `parse_segment_rejects_all_out_of_class_call_shapes` to
`parse_segment_accepts_framed_call`. Both are called out here rather than left to
look like nothing happened.

W41 adds **no new `census.rs` key and no new shape family** — it produces
`framed-call` and the tail-call families that already existed — so
`cross_sweep.sh` has nothing new to discover and its configuration count is
unchanged, which is the correct outcome rather than a missed one. (W36 recorded
the same, for the same reason.)

### The generated axis, and it found nothing

`scripts/sweep.d/73-framed-member-call.py`, **246 cases**, one file per axis so it
cannot conflict with a peer's fragment. It varies the *product* of
`70-framed.py` and `72-member-call.py`, which neither can reach: the post-op
operator crossed with the sign and magnitude of the literal at both signed-16-bit
boundaries and both spellings of zero; the receiver's formal position (to nine,
one past the register file) crossed with the post-op, because `mr r3,rN` and
`addi r3,r3,k` both write r3 and only a non-zero position orders them; the
result's type crossed with the literal, because a pointer result scales it by a
pointee size that is nowhere in the source; the receiver's `2C` conversion, in the
three spellings that actually produce one, crossed with the post-op and the
receiver's position; every argument permutation at each arity crossed with the
post-op, which is the boundary between the two productions in this file; and the
caller being a member function, where the receiver's index and its register are
different numbers.

**It found no mis-emit.** Six live mis-emits have come out of generated axes this
session and hand-written fixtures found none, so a green new axis is worth stating
plainly rather than omitting: on this construct the grid agreed with the port
everywhere it reached. The one wrong-bytes emit W41 did find came from lifting a
gate on purpose (§the under-claiming check), not from the sweep.

## Found and not taken

Ranked, with the frame axis applied first because it is free.

1. **Class B — 6,463 in this row alone, and it is now the binding constraint on
   the whole member-call family.** `return p->m() + n*k;`: one value live across
   one call, `std r31`/`ld r31`, and a `mulli` the pointer scale decides. The port
   has no callee-saved-register model at all. It is 62 % of this row, it is the
   reason `t-classb-*` is the only thing left in it, and every other row that
   mentions a formal after a call lands in the same place. `parse_call_sequence`
   already refuses it **by name** ("Class B, a later rung"), so the boundary is
   drawn and the population is now sized on a real row rather than argued about.
2. **`expr-call-in-expr-recv-load-then-bit-and-and-branch-more` — 102,374, still
   the largest key on the board**, and still UNMEASURED in W36's sense.
   `cflow-if-1` accounts for 102,370 of it, which W37's cross already established
   means it needs basic blocks before it needs the operator. Nobody has yet got a
   completeness figure for it; the ranking should not schedule against the size.
3. **The capture-cache relocation hazard.** A one-line fix exists — put the cache
   root in the context hash, so a relocated cache misses instead of lying — and it
   is in `c2-harness`, which this rung deliberately did not touch. Until then, a
   copied cache is a false-alarm generator that costs an agent an hour and looks
   exactly like a port regression. This is the highest-value thing on this list
   per line of code and it is not a coverage item.
4. **`expr-call-in-expr-recv-load-then-type-ptr-and-off-add-more` — 22,570** and
   **`-and-op-more` — 18,340.** The two largest measured `recv-load` rows left.
   Both are `-more`, so both carry a third construct; neither has been
   decomposed with the production histogram §Reproduction now provides, and that
   instrument is 60 lines and reusable as written.
5. **`recv-load-then-call-data-addr-whole` — 10,540, all `calls-1`.** Unchanged
   from W36's list, and its warning stands: `GAPS.md` §6 records `data-addr`
   realizing **0** against an 11,000 estimate because c2 derives every address
   after the first from a whole-TU pool layout.
6. **The `-whole` measure's own looseness on the statement list.** `body_matches`
   applies no formal/`.sy` membership test to a `26` assignment destination, so a
   **global** store reads `-whole` and is flatly unreachable. It cost nothing here
   because the row had no statement lists at all, but the next `-whole` row that
   does will have this in it and no one has sized it.
7. **The riskiest thing this rung leaves unmeasured.** `eat_call_postop` now
   accepts `03` at *both* its consumers, and the free-function one has **zero**
   witnesses on the workload — the entire evidence for `return g(a) - k;` is
   `mvp_call_submod.cpp`, one probe TU and 18 sweep cases. If the free and member
   forms ever diverge at this position, nothing on the real workload will notice,
   because the population that would is empty there. That is the same
   configuration that let the original wrong gate stand for seventeen rungs, with
   the sign flipped.
8. **`t-postop-mul-lit` and the wide literal fall through to `Err(None)`** rather
   than getting their own keys, so they keep the row's key instead of naming the
   gate that refused them. Correct under the non-committal contract (the body is
   not this production until the plumbing parses) but it means the ~32-function
   residue of this row is attributed to the row and not to its gates.

## Reproduction

```sh
# the lowering, read off the reference obj rather than inferred:
scripts/gt_capture.sh work/w41/probe/p1.cpp /O1 /GS- /c   # ±k, receiver slot, ptr scale, Class B
scripts/gt_capture.sh work/w41/probe/p5.cpp /O1 /GS- /c   # the ±0 FOLD, wide k, arg permutation
python3 scripts/gt_dump.py work/w41/probe/p1.obj
./target/release/c2rs census work/w41/probe/p4.cpp        # which C++ emits a receiver `2C`

# the SIZING INSTRUMENT (scratch, reverted; nothing is claimed in class — it only
# renames a refusal, the numerator is unchanged in every build). It is a
# member-call PRODUCTION first-blocker histogram, and it has two halves:
#
#   1. `mcall.rs::body_matches` gains a `Shape` out-param recording, per
#      statement, whether the statement opened `26 <dst>`, whether it consumed a
#      `form` value, and how it terminated. `mark_whole` renders that as the
#      census key for `CallForm::RecvLoad` whole bodies. THIS ALONE REFUTED THE
#      BRIEF: 10,022 of 10,494 are ONE statement, the prefix count is 0.
#   2. `mcall_tail.rs::w41_diagnose` mirrors `try_parse_member_tail_call` step
#      for step and names the first refusal instead of collapsing every one of
#      them to `Err(None)`, plus a `w41_tail_shape` that names the residual token
#      run after the call's `4C`. That is what splits the 10,022.
#
# The anchor is the whole design: keying on the `-whole` bit alone gives a row
# with no structure, and keying on the byte after the call gives `B9` vs `33`
# without saying which is reachable. Keying on the PRODUCTION gives both.
#
#   6,463 t-classb-load-lit-scale   value live across the call
#   3,559 t-postop-sub-lit          -> shipped
#     440 g-OK-after-recv-cast      -> shipped
#      32 residue
#  10,494 exactly
#
# the per-gate counterfactual (parser-only lift, one warm scan each):
#   mcall-framed-args -> +1 census, 0 disagreement, and a probe one token away
#                        mis-emits (`c2rs diff work/w41/probe/p7.cpp`)
```
