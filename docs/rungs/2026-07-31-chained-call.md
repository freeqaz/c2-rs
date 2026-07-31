# WCH — `return p->a()->b();`, the largest `-whole` row on the board

    Tag:       WCH
    Slug:      chained-call
    Date:      2026-07-31
    Fixtures:  wch_chained_call.cpp wch_chained_call_neg.cpp
    Census:    661245 → 673006 (26.85 % → 27.33 %), +11,761
    Record:    this file

The chained member call as a whole body. `expr-call-in-expr-chained-whole` was
**12,479 functions, the largest `-whole` row on the board**, and it needed **no
codegen at all**: each call's result lands in r3, which is exactly where the next
call's `this` belongs, so nothing is ever live across a `bl` and the body is
`BodyShape::CallSeq` with `saved` empty — the Class A statement sequence that has
shipped since #35 rung 1. The entire rung is the recognizer.

## What it admits, and what it refuses

```text
  26 <m_outer> … 26 <m_inner>   the method symbols, stacked LIFO
  B9 <recv> <TYPE ptr4> [2C…]   the innermost receiver     — `eat_receiver_this`
  99 <TYPE ptr4> 00             …bound as its argument zero
  BD <ret ptr4> 00 <id> (<arg> 55 <T>)* 4C    the innermost call
  ( 99 <TYPE ptr4> 00           the chain link: bind the RESULT as `this`
    BD <ret> 00 <id> 4C )+      …and call the next method out
    4B | 41 <T>                 statement end, or the result is returned
```

Read off the reference obj (`work/WCH/probe/p1.cpp`, `/O1 /GS- /c`):

```text
  int  c_ret (O* p) { return p->Next()->gi(); }            36 B, F = 96
    mflr r12 ; stw r12,-8(r1) ; stwu r1,-96(r1)
    bl ?Next ; bl ?gi
    addi r1,r1,96 ; lwz r12,-8(r1) ; mtlr r12 ; blr
  void c_void(O* p) { p->Next()->vv(); }                   36 B — the same
  int  c3    (O* p) { return p->Self()->Next()->gi(); }    40 B — one more `bl`
  int  c4    (O* p) { return p->Self()->Self()->Next()->gi(); }  44 B
```

**Three facts, each measured rather than inferred.**

### 1. The emission order is the push run REVERSED

The method symbols stack LIFO, so `p->Next()->gi()` is `26 <gi> 26 <Next> B9 <p>`
and `?Next` is called **first**. An emitter that walked the pushes forwards writes
both REL24 targets the other way round. Two mutations of the shipped rule,
graded on the new sweep fragment's 231 cases:

| mutation | mismatches |
|---|---|
| emit the push run **forwards** | **75** |
| emit it reversed, with the outer **two links swapped** | **71** |

The second is the one a depth-2-only grid cannot see: reverse and swap-two agree
at two links and disagree at every depth above, which is why the grid runs to
four links with all-distinct methods.

### 2. Chain depth is free, and arguments are free on exactly one link

`call_seq_text` takes one setup per call, so a third and a fourth link are one
more `bl` each. The **innermost** call marshals out of the argument registers with
nothing clobbered yet, so `this` appends to its argument list as slot 0 and the
whole thing goes through `tail_call_shape` — the identical trick W36 plays, and
the identical permutation:

```text
  int c_ai3(O* p,int j,int k) { return p->NextB(k, j)->gi(); }   48 B
    mr r11,r5 ; mr r5,r4 ; mr r4,r11 ; bl ?NextB ; bl ?gi
```

A **later** link is a different lowering in both of its cells:

```text
  int c_ao(O* p,int k) { return p->Next()->gia(k); }        52 B — CLASS B
    … std r31,-16(r1) ; stwu ; mr r31,r4 ; bl ?Next ; mr r4,r31 ; bl ?gia
  int c_al(O* p)      { return p->Next()->gia(7); }         40 B — `li r4,7`
    … bl ?Next ; li r4,7 ; bl ?gia
```

The formal cell is Class B *and* needs the save/marshalling interleave
`plan_saved_gprs` refuses by name; the literal cell is Class A but writes **r4**,
and `select_text` computes into r3 and only r3. **Both refuse, under two keys, and
splitting the key is the measurement that mattered** — see below.

### 3. The `99` bind is one locator, and it carries a gate for free

`eat_receiver_this` was split into a receiver-designator half and
`eat_this_bind`; the chain link is the bind with no designator in front of it.
That is not tidiness — the bind's TYPE is the **bound value's** class pointer, so
requiring it to be `ptr4` says the previous call returned a class pointer, and the
intermediate calls need no return-type gate of their own. A private copy of those
three bytes would have had to restate that, and would have re-decided
`mcall-bind-offset` silently (`GAPS.md` §6 #9).

**Refused, each by name:** an argument on a later link
(`mcall-chain-link-args` / `mcall-chain-link-arg-lit`); a computed argument on the
innermost link (`call-arg-computed`, the shared key); an innermost receiver that
is not a plain `B9 <formal>` load — a global, a dereference, a sub-object, a
`this`-adjust — which **declines non-committally** so those bodies keep the
`expr-call-in-expr-chained-…` key that names their own designator; a `float`
result even discarded (`call-ret-fp`); nine formals
(`callseq-over-eight-formals`); and anything after the chain — a post-op, a
dereference, a comparison, a branch, a second statement.

### The locator check, both directions

`eat_this_bind` has **two** consumers the day it lands (the receiver bind and the
chain link), so it is not a shared locator nobody asks. Grepping every other
reader of the `99` bind in the body layer finds two, both pre-existing, both in a
different *position*, and neither reachable from this production:

* `shapes/ctor_dtor.rs` reads the bind itself, and **refuses more** than its new
  sibling — it requires the `2C` cv-strip in front of the bind rather than
  admitting it optionally, and reads the trailing field as a literal byte rather
  than a varint. That is part of the empty-dtor-delegation production's own
  measured shape (`docs/IL_CALL_IN_EXPR.md` §5, §15) and it is a whole-body
  production, so no chain reaches it. Left alone: unifying it needs the dtor
  captures, not this rung's.
* `shapes/this_binding.rs::read_this_group` reads the **function header's** `this`
  group, not an expression's, and **refuses less** — it does not require the bound
  value to be a width-4 pointer at all. Same conclusion: different position,
  unreachable from here, and narrowing it is a measurement somebody else has to
  make.

## Estimate vs outcome

Written to `work/WCH/ESTIMATE.md` before any scan.

| | |
|---|---|
| estimate | **+12,479** (the ceiling), range 9,000–14,760, bias *at* the ceiling |
| realized | **+11,761** |
| ratio | **1.061× high** |

**The pre-filter, named.** `expr-call-in-expr-chained-whole` is
`CallForm::Chained` (`heads.len() >= 2` in the completeness walk) crossed with
`whole_body_is_one_value`. It had **not** been filtered by chain depth, by the
innermost receiver's designator, by arguments on any link, by whether the receiver
is a formal, or by the last link's result type.

**The rule applied, and it held.** WCB came in 3× low for discounting a ceiling
when four of its five "independent things" were already built; the rule it wrote
is *when the blocker is a class whose emitter already exists, the ceiling IS the
estimate — what remains is counting the independent refusals between it and the
emitter*. Three were counted (a later link's argument, a non-`B9` receiver, a
non-int result); none of them is what defines the row, and the ceiling came in at
**1.061×**. That is the closest a row estimate has come on this board (previous
row ratios: 67×, 67.8×, 13.4×, 2.62×, 1.45×).

**Where the 718 went, measured by counterfactual rather than argued.** With the
link-argument refusal turned into a non-committal decline, `chained-whole` reads
**719** instead of 0 and *no other key moves*:

* **11,760 of the 12,479** are accepted — the row is **94.2 %** realized;
* **719** of it carry an argument on a later link;
* **+1** came from `chained-then-type-ptr-whole`.

## Gate evidence

| lane | result |
|---|---|
| `cargo test --workspace` | **475 pass / 0 fail** (was 470) |
| `c2rs bench` | **180 pass / 0 fail / 0 error** (was 178) |
| `scripts/mode_lane.sh` `/Ox` / `/O1` / `/O2` / `/Ox /Gy` | **85 / 83 / 83 / 83**, 0 mismatch in all four (was 84/82/82/82) |
| `scripts/expr_sweep.sh` | **12,429 cases, 0 mismatches** (12,198 before) |
| `scripts/cross_sweep.sh` | **14,184 × 4 configurations, 0 mismatches** — unchanged from the baseline count, because the rung declares no new shape family: a chain is `BodyShape::CallSeq`, which the lane already had a representative for |
| 878-TU workload scan | mismatch **0**, census **673,006 / 2,462,571 = 27.33 %**, disagreement **0** |
| fixtures, `c2rs census` | `wch_chained_call.cpp` **17/17**, `wch_chained_call_neg.cpp` **0/14** |

## What was refuted

**The frame axis over-counts Class B, confirmed a second time and at scale.**
The row is **100 % `calls-2plus`** and **100 % Class A** — a three-word prologue
with nothing saved. WCB refuted the axis on 6,000 functions; this refutes it on
11,761 more.

**Crossing the row with `calls-N` and `cflow-*` killed nothing, and that is a
result.** W37 eliminated 134,763 functions for free that way. Here both axes are
degenerate: 12,479 = `calls-2plus` = `cflow-straight`, exactly. The cross is still
worth running — it is free — but it is not a general-purpose ranking instrument,
and a row that survives it is not thereby ranked.

**The cheap half of the residue is worth 2 functions.** `mcall-chain-link-args`
came out at **12,092** — *larger than the row this rung landed* — and the obvious
next move was its Class A cell, the `li r4,k` literal argument, which needs only a
per-call first-argument-slot in `call_seq_parts`. Splitting the key cost one
predicate and one rescan and settled it: **`mcall-chain-link-arg-lit` is 2
functions and `mcall-chain-link-args` is 12,090.** The cheap repair buys nothing
and the expensive one (Class B chain link) is the whole row. That is the
"enumerating sub-shapes when the winner is not among them" failure caught *before*
it cost a build.

**A sibling row's name describes the wrong construct.**
`expr-call-in-expr-chained-then-type-ptr-and-op-more` (15,049) reads as a chain
with an arithmetic post-op. It is not: **11,373 of it** reaches this production and
refuses on `mcall-chain-link-args`, i.e. its "op" is the outer link's *argument
region*. §6n category (5) one level down, in a `-then-` key rather than a
schedule.

## Which of §6n's five categories this row was

**(1) — a private limit inside a recognizer that already exists**, and the sixth
rung running in that category. The limit was one byte:
`try_parse_member_tail_call` reads its callee push and then demands `B9`, so
*every* chain in the corpus fell through to the assignment parser at the second
`26`. Unlike W35/W38/WSL the limit was in the production's **entry** rather than
in a sub-locator, so the repair is a new file rather than a widened predicate —
but the diagnosis is the same one, and it is the sixth time the cheapest thing on
the board was a recognizer that already existed refusing at its own front door.

The row was otherwise **exactly what it said it was**: correctly named, correctly
sized to 6 %, and with its emitter already shipping. That combination has not
occurred before in §6n, and it is the reason the estimate landed.

## Found and not taken

Ranked, frame axis applied by reading the obj rather than the call count.

1. **`mcall-chain-link-args` — 12,090, and it is the direct successor to this
   rung.** `p->a()->b(k)`: the outer link's formal argument is live across the
   first `bl`, so the body is Class B with one saved GPR and the argument goes to
   **r4**, not r3. Read off the obj: `std r31,-16(r1) ; stwu ; mr r31,r4 ;
   bl ?Next ; mr r4,r31 ; bl ?gia`. Two things are missing and they are
   independent: a per-call **first argument slot** in `call_seq_parts` (slot 0 is
   the `this` already in r3, so the explicit arguments start at r4), and the
   Class B marshalling for a later call at a non-zero slot. `plan_saved_gprs`
   already computes the save correctly — the receiver of the *chain* is not live,
   only the argument is — so the saved-formal half is done. **Bigger than the row
   this rung landed**, and the largest single key the chain family now carries.
   The literal cell beside it is **2** functions: do not build the `li r4,k`
   path for its own sake.
2. **The innermost receiver's designator — 5,188 together**, all `-whole`, all
   one production each:
   `expr-call-in-expr-recv-field-off0-then-chain-bind-whole` **2,666**,
   `expr-call-in-expr-recv-intrinsic-this-adjust-then-chain-bind-whole` **1,686**,
   `expr-call-in-expr-recv-field-then-chain-bind-whole` **836**. Each is this
   rung's body with the `B9 <formal>` swapped for a designator that already has a
   lowering elsewhere in the port (`designator::eat_offset_adds`, the shared
   offset walk, and the `this`-adjust intrinsic). The `-off0` one emits **nothing**
   for the designator at all, so it is the cheapest of the three and it is the
   biggest.
3. **`expr-call-in-expr-chained-then-type-ptr-and-off-add-more` — 10,568**, the
   chain whose result is then offset-added. Unmeasured: no probe here reproduced
   the key, and its name has already been shown (above) to describe the wrong
   construct once in this family. **Read the obj before ranking it** — that is
   the whole lesson of item 1, where the second-largest key turned out to be an
   argument region rather than an operator.
4. **`expr-call-in-expr-chained-then-type-int1-and-type-aggregate-whole2/3` —
   2,211.** A chain whose last link returns a one-byte value into an aggregate
   context. `-whole2`/`-whole3` means two or three constructs would have to be
   admitted together, so its ceiling is not its size.
5. **The riskiest thing left unmeasured.** The rung's byte evidence is 231
   generated cases, 17 fixture functions and one probe TU — and **every one of
   the 11,761 realized functions lives in a TU the port never emits** (`vocab-gap`
   at 98.5 %), so not one of them has ever been compared byte for byte. Two
   asymmetries inside that:
   * **Chain depth in the corpus is unknown.** The grid runs to four links because
     `mcall::MAX_CHAIN`'s sample said four was the deepest seen; the acceptance
     bound is eight. Nothing measured how many of the 11,760 are depth 2 against
     depth 3+, so the depth axis is graded by construction and not by population
     — the exact shape of the token-width asymmetry section 13 of the WCB
     fragment had to close.
   * **The innermost call's argument permutation is graded at depth 2 only in the
     wild sense.** Sections 4 and 2 of the fragment cross arity with depth, but
     the workload's own `NextA`-style links were never counted: if the realized
     population is overwhelmingly nullary, the permutation cells are graded
     against generated cases alone, and W36's `MAX_VERIFIED_PERM_CYCLE` boundary
     is the one place in this path where c2 stops agreeing with the port's walk.
