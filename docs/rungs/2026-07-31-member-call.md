# W36 — the member call as a whole body, and the head of the board was a misfiled name

    Tag:       W36
    Slug:      member-call
    Date:      2026-07-31
    Fixtures:  w36_member_call.cpp w36_member_call_neg.cpp
    Census:    581,791 → 602,703 (23.63 % → 24.47 %), +20,912
    Record:    this document

`expr-op-0x99` was the largest single key on the board — **280,283 functions,
11.4 % of everything blocked**, and 364,690 behind a cleared `expr-op-0x27`. It
was never a missing token. It is `expr-call-in-expr-recv-*` under a second name,
reached by the one route that does not call `mcall::classify`.

## What it admits, and what it refuses

```text
  4c 4f 11 53                    LO, SS
  26 <method>                    push the METHOD symbol   — the callee
  b9 <recv> 86 43 88 20          LOAD the receiver        — `this`
  99 86 43 89 20 00              bind it as argument zero
  bd 82 12 30 00 80 05 10 00 00  CALL, void result
  4c 4b                          apply, statement end
                                                          => b ?v0@Obj@@QAAXXZ
```

`p->m(x)` is `m(p, x)` on this ABI: **`this` is argument zero, in r3**, and
nothing else about the call differs from a free-function one. So the whole
production is the tail call the port has emitted since the MVP with one extra
argument slot. The receiver is appended to the argument list and
[`calls::tail_call_shape`] does the rest — the identity case that emits nothing,
the single `mr r3,rN`, the permutation walk with its measured 3-cycle limit, the
`.gl` callee resolution, `.pdata` and `/Gy`. **The port needed no codegen change
at all**, which is why census/gate disagreement stayed 0 through every
configuration.

The receiver goes on the **end** of the argument list, not the front: the list
is in stream order (rightmost source argument first, slot `i` is
`args[len-1-i]`). `member_tail_call_puts_this_in_slot_zero` pins that against a
capture where the two readings give different permutations — `o->v2(b, a)` is
`[0, 2, 1]`, one 2-cycle over r4/r5, and pushing `this` on the front would give
`[1, 2, 0]`, a 3-cycle and three wrong `mr`s.

### Why the row was invisible, and why that is the reusable part

`mod.rs`'s body dispatch tells a call from an assignment by asking whether a
`BD` follows the statement-head `26 <tok>`. **For a member call it does not** —
the receiver sits between the method push and the CALL token — so the statement
went to the assignment parser, which read the receiver as an ordinary LOAD and
stopped dead on the `99` bind under `parse_expr`'s generic `expr` fall-through.
The construct was therefore filed as an **opcode**, while the identical
production one byte different (`x = p->m();`, which has a real destination
push) reached [`mcall::classify`] and was filed as a member call all along.

That is `GAPS.md` §6's unstable-*attribution* hazard in the form that costs
coverage rather than correctness, and it is worse than the sharded-key form the
document already records: a sharded key at least splits one construct across
several names that *look* like the construct. This one filed 280,283 functions
under a name that describes a byte, in a bucket carrying **no whole-body
completeness bit at all** — so every ranking taken from it for weeks could see
the row's size and nothing about what was complete behind it.

[`mcall::reanchor_chain`] already repaired exactly this for the *chained* case
(§18.3, a measured 4.4× undercount). Generalizing it to any receiver form —
same three conditions, same [`walk_detail`], no second tokenizer — de-conflates
the row **1:1**: `expr-op-0x99` 280,283 → 0, the `expr-call-in-expr-recv-*`
family rises by exactly 280,283, no other key moves, census unchanged, and the
`-whole` counts appear for free. **The de-conflation is the counterfactual**; no
scratch build was needed to size the rung.

### Refused, with the measured cost of each refusal

| refusal | why | cost on the 878-TU workload |
|---|---|---|
| a **computed** argument beside the receiver (`o->gk(k+1)`, `o->v1(7)`) | adding `this` makes every such call multi-argument, and the multi-argument tail call models only a pure register permutation — which register a computed argument is evaluated into interacts with the permutation temp and no capture covers it | **1,097 functions** (`call-arg-computed`) |
| the caller's own `this` binding undetermined | shared gate; a `this` that cannot be positively established never silently means "absent" | **208** (`this-undetermined`) |
| an argument that is not one of the caller's formals | a global read is a relocation and a load, not a register move | **7** (`call-arg-nonformal`) |
| a **discarded** `float`/`double` result | the TU carries `_fltused` and the port has no model of it — see §"the fourteenth" below | **0**, and it closes a live mis-emit |
| a permutation with a cycle > 3, or more than one cycle | `permute_args_text` is measured wrong past 3 (`GAPS.md` §6 #10); `this` makes each cycle one element longer than the equivalent free-function call, so the gate had to be read over `n+1` slots | structural, shared gate |
| nine argument registers (`this` + eight) | past the eighth a parameter is stack-homed and the setup is a store | **0**, and it is a real gate — see the under-claiming check |
| a **non-zero** `99` bind offset | `IL_EXPR_LAYER.md` §7 records the field as UNKNOWN and zero at every observation; required literally rather than skipped | **0** — lifting it in the parser alone gains exactly 0 over 2,462,571 bodies, which is a much stronger statement about that field than the eleven observations the doc had |
| a receiver that is not a plain `B9 <tok> <ptr4>` — a member, a deref, a named object, another call's result, an adjusted base, a chain | each is a *different* receiver production with a different lowering, and the census names every one of them | the residual `recv-field` / `-deref` / `-object` / `-call` / `-intrinsic-*` / `chained` rows |
| a body that does not **end** at the call | the Class A statement sequence with a member call in it — a further rung; it falls through and keeps its measured second-blocker key rather than being claimed here | **12,158 of the `-whole` bodies**, measured — see below |

`w36_member_call_neg.cpp` carries one case per row and censuses **0/17**.

### The `-whole` measure over-counts realized yield by 1.67×, and all of it is named

This is worth recording because the whole-body-completeness bit is the
instrument this project ranks with, and its own documentation says only "expect
the realized yield to be below it".

| | functions |
|---|---:|
| `recv-load-whole` + `recv-load-then-type-ptr-whole` (the grammar ceiling) | **34,825** |
| realized in class | **20,912** (60.0 %) |
| refused by a codegen gate *after* the whole-body parse, under its own key | 1,312 |
| `-whole` bodies that are **not terminal**, or whose argument the port's own operand vocabulary refuses | **12,158** (measured by a scratch that sinks that refusal under its own key and diffing the two `-whole` rows) |
| residual `-whole` after both | 443 |
| | 34,825 exactly |

The cause is structural and applies to every future reading of the measure:
`body_matches` admits `adm.form` at **every** value position and iterates
`stmt*`, so a body with *two* member calls, or with an assignment statement
before the call, reads `-whole` — and both are excluded by any production that
requires the call to be terminal. `n_two_stmts` in the negative fixture is that
case, censusing `recv-load-whole` and refusing.

## Estimate vs outcome

Written to `work/w36/ESTIMATE.md` **before** any scan, together with the
pre-filter analysis the last three rungs got wrong.

**What the bucket had already been filtered by.** For a body to be filed
`expr-op-0x99`: `.sy` binding, one-register-each formals, `LO`/`SS`/scopes; the
first statement opens `26 <tok>` with **neither** a `BD` nor another `26` after
the token; and `parse_expr` consumed `B9 <tok> <TYPE>`, i.e. **the receiver's
type is already in the modeled 4-byte pointer class**. Two deductions were
therefore unavailable and taking them again would have been the exact mistake
W34 and W35 made: *"the receiver's type might not be modeled"* and *"the body
might not open on a `26`-headed statement"* are already priced in. What was
**not** filtered is everything from the `99` rightward — the CALL token, the
whole argument region, whether a second statement follows, the plumbing, and the
receiver's register. The row is the *first* statement's blocker.

| | estimate | outcome | bias |
|---|---|---|---|
| counterfactual A (parse ceiling — admit the head as a prefix that re-dispatches to the existing call grammar with `this` as slot 0) | **+12,000**, range 4,000–35,000, **biased HIGH** | **+13,877** (`recv-load-whole`), or **34,825** counting the pointer-argument row the completeness matcher refuses and `parse_expr` does not | direction **wrong**; LOW by **1.16×** against the like-for-like figure |
| the shippable rung | **+7,000**, range 2,000–20,000, **biased HIGH** | **+20,912** | direction **wrong**; LOW by **2.99×**, and outside the stated range |

**Three anchors were written down and the wrong one was trusted.** (A) the
row-to-counterfactual prior, measured at 67× and 67.8×, gives 4,183 — and the
estimate *explicitly discounted it*, on the argument that both prior
measurements were of rows whose token is a value annotation the rest of the body
has to be understood anyway, while `0x99` sits at one fixed position in exactly
one production and 40.7 % of its row was already `calls-1`. **That argument was
right**: the realized ratio is 13.4×, not 67×. (B) the sibling family's own
`-whole` rate on `calls-1` (3,849 / 59,346 = 6.5 %) applied to the 114,059
`calls-1` sub-row gives 7,414 — that is where the +7,000 came from, and it is
the anchor that was wrong, by 2.8×. (C) source-language reasoning gave 28,500
and was the **closest of the three**, off by 1.36×.

The lesson is specific and it is not "estimate higher". **Anchor B was a rate
measured on a population the row is not.** The sibling `recv-load` family is
25,308 functions with *two* `calls-1` in the whole of it, because a member call
in a value position or an assignment RHS is nearly always in a body that makes
more than one call. The statement-position population is the opposite —
114,059 `calls-1` — and its `-whole` rate is 30 %, not 6.5 %. The asymmetry was
visible in the existing scan before any estimate was written, and it was read as
context rather than as the reason the sibling's rate did not transfer.
Generalizing: **before borrowing a rate from a sibling bucket, check that the
two agree on the axis that decides the rate** — here the frame axis, which was
already tabulated for both.

The stated LOW hazard was named and it was the right one: *"the shape that could
blow the estimate upward is `void f(A* p){ p->m(); }` — bodies that are
byte-for-byte the shape the port already accepts once the `this` group is
removed"*. That is exactly what happened. It was left unbounded, and `GAPS.md`
§6 already says an unbounded bias direction is an excuse.

## The fourteenth live wrong-bytes emit — pre-existing, on mainline

`float gf(); void f() { gf(); }` is a bare `b ?gf@@YAMXZ`. It touches no
floating-point register at all, and its obj still carries the undefined external
`_fltused`. The port emitted one symbol too few: **`Port=Mismatch @ offset 12`,
the COFF header's `NumberOfSymbols`** — reproduced on **clean master `e24e27f`**
for both the nullary and the one-argument form, so it is not this rung's.

It is `GAPS.md` §6 instance #11's field one producer further out.
`touches_floating_point` enumerates the shapes whose own *body* does FP work
(the float leaf, the FP tail call, the FP store); a body that merely **calls** an
FP-returning function does none of them and still needs the hook.

Bounded by probe rather than guessed: `float`, `double` and `long double`
results mis-emit; `float*` does not; an FP *argument* does not (the FP tail call
marks the function itself); and merely declaring the callee without calling it
does not. So the trigger is the **call**, and specifically its result class.

**Refused rather than modeled**, under the census key `call-ret-fp`, at the one
locator every call shape goes through, and applied only where the value is
discarded — the value-consuming sites are already gated by their own `41`
annotation and the FP tail call is deliberately untouched. Modeling it would
mean claiming that `_fltused`'s measured placement rule ("after the first
FP-touching function's symbol group") and the per-TU label-counter surcharge it
also drives (`LABEL_COUNTER.md` §1.1) extend to a new kind of FP-touching
function, and neither has been captured. It costs **0 functions** on the
workload. This is `GAPS.md` §6 #13's "a gate that hides a wrong rule is a debt",
and it is booked as one in "Found and not taken".

**How it was found is the point.** It came from the axis this rung added to the
generated sweep — *the callee's return type, crossed with discarded and
returned* — on its first run. Nothing had ever varied it: every call in the
fixture corpus, in all 10,194 pre-existing sweep cases, in four mode lanes and
in the 878-TU scan returns `void`, `int` or a pointer. Three instruments and a
2.4-million-function scan were green over it for as long as the void tail call
has existed.

## Gate evidence

Corpus `dc3-decomp` at `05ca6d09`; final scan on the **clean committed tree**
`0b1a0b7` with `--validate-cache 50` and `--replay-every 25`: 17 entries
re-captured through the real toolchain and agreed, **0 POISONED**; replay
soundness 36 checked, 0 diverged.

| lane | baseline (master `e24e27f`) | W36 |
|---|---|---|
| `cargo test --workspace --release` | 452 pass / 0 fail | **457 pass / 0 fail** |
| `c2rs bench` | 167 pass / 0 fail / 0 error | **169 pass / 0 fail / 0 error** |
| `scripts/mode_lane.sh /Ox` | 77 match, 0 mismatch | **79 match, 0 mismatch, 0 codegen-gap** |
| `/O1` · `/O2` · `/Ox /Gy` | 75 match, 0 mismatch | **77 match, 0 mismatch, 2 codegen-gap** each |
| `scripts/expr_sweep.sh` | 10,194 cases, 0 mismatches | **10,359 cases (+165), 0 mismatches** |
| `scripts/cross_sweep.sh` | 11,341 × 4, 0 mismatches | **11,341 × 4, 0 mismatches**, family set and configuration count bit-identical |
| 878-TU scan | 581,791 / 2,462,571 (23.63 %), mismatch 0, disagreement 0 | **602,703 / 2,462,571 (24.47 %)**, mismatch 0, **disagreement 0** |
| `census fixtures/cpp/w36_member_call.cpp` | — | **25/25 in class**, `Port=Match` |
| `census fixtures/cpp/w36_member_call_neg.cpp` | — | **0/17 in class**, `Port=NotImplemented` |

The census delta is **exact**: the only key that falls is `expr-op-0x99`
(−280,283), the rise into the de-conflated `expr-call-in-expr-recv-*` family is
**259,371**, and the difference is **20,912 = the census gain**, to the function.
No pre-existing bucket rises.

W36 adds **no new `census.rs` key and no new shape family** — it produces the
`void-tail-call` / `int-tail-call` / `multiarg-tail-call` families that already
existed, because a member tail call and a free tail call emit the same thing —
so `cross_sweep.sh` has nothing new to discover and its configuration count is
unchanged, which is the correct outcome rather than a missed one.

### The gates checked in the UNDER-claiming direction

The direction nothing in this project tests, and both answers are informative.

* **The `99` bind offset.** Lifting the "must be `00`" requirement in the parser
  alone gains **exactly 0** over 2,462,571 bodies and moves no key. The field is
  not merely "zero in every observation" (eleven of them, in
  `IL_EXPR_LAYER.md` §7) — it is zero everywhere the port can reach on a
  real workload, so requiring it literally costs nothing at all.
* **The nine-argument-register gate.** Lifting it in the parser alone puts
  `n_nine` in class (the negative fixture reads 1/17) while `PortC2` still
  returns `NotImplemented` — a census/gate **over-claim of exactly 1** on the
  probe TU, and 0 on the workload only because that population is empty there.
  So the parser gate mirrors a real codegen limit (`select_text` refuses past
  eight formals) rather than adding conservatism, which is what keeps
  disagreement at 0.

### The generated axis

`scripts/sweep.d/72-member-call.py`, **165 cases**, one file per axis so it
cannot conflict with a peer's fragment. It varies what nobody varies: the
**complete permutation grid** at each arity crossed with the receiver at every
position in the caller's own formals (`this` makes each cycle one element longer
than the equivalent free-function call, so a call the free grid graded as a
3-cycle is a 4-cycle here — the region `permute_args_text` is measured *wrong*
in); **the caller being a member function itself**, where the receiver's index in
the formals list and its argument register are different numbers for the first
time — the single most repeated defect in this project (§6 #4, #5, #6, #8);
cv-qualification of pointee, pointer and method crossed; an argument list that
is a strict subset of the caller's formals (§6 #5's panic, with a slot added to
one side of it); the receiver passed again as an argument; the argument-register
boundary at nine; and the callee's return type across every width and class,
discarded and returned — which is the one that fired.

## Found and not taken

Ranked, with the frame axis applied first because it is free.

1. **A member call preceded by assignment statements — ~10,000, and the single
   cheapest thing on this list.** The dispatch only offers this production the
   *first* statement; a body whose member call follows `int x = a;` goes to the
   assignment parser, which folds the assignments into an expression and then
   has nowhere to hand the call. Those bodies are the bulk of the 12,158
   "not terminal" `-whole` residue above and they are `calls-1`. The assignment
   parser already resolves the statement list by substitution; what is missing is
   the hand-off, and the risk to price is that a folded local reaching an
   argument is *argument setup*, not a permutation.
2. **`expr-call-in-expr-recv-load-then-bit-and` — 102,382 functions**, and it
   appeared out of nowhere in the de-conflation: it is the single largest key on
   the board today. It is UNMEASURED (`bit-and` has no production, so the pair
   carries neither `-whole` nor `-more`), and its size makes it the first thing
   the next ranking should look at. The obvious reading is `if (p->Flags() & k)`
   — a member call whose result is masked — which would also make it a
   control-flow row, and control flow was declined at 718 earlier today. **Get a
   completeness figure for it before scheduling against the number.**
3. **A discarded FP result, modeled rather than refused.** Booked as a debt above.
   Worth 0 census functions and one live mis-emit closed; the reason to do it is
   that `touches_floating_point` now has four producers and its *fourth* was
   found by a sweep rather than by reading the field, which says the next one
   will be too.
4. **`recv-load-then-call-data-addr-whole` — 10,540, all `calls-1`.** A member
   call one of whose arguments is a data symbol's address (a string literal).
   `GAPS.md` §6 records `data-addr` realizing **0** against an 11,000 estimate
   because c2 derives every address after the first from a *whole-TU pool
   layout*, which no per-body grammar can express. Do not schedule this without
   re-reading that entry.
5. **`recv-load-then-call-recv-load-whole` — 3,197, all `calls-2plus`.** Two
   member calls in one body: needs a frame before it needs anything else, and
   Class C was declined at 0 today.
6. **A computed argument beside the receiver — 1,097.** The rule is "which
   register does a computed argument get evaluated into, given the permutation
   temp", and it is one capture grid away rather than one instruction.
7. **The riskiest thing this rung leaves unmeasured: what `_fltused` and the
   label counter do for a body that CALLS floating point.** The gate above turns
   the mis-emit into a refusal and costs 0 today, but `touches_floating_point`
   now has four producers and the fourth was found by a generated sweep rather
   than by anyone reading the field — which says the fifth will be too. §6 #13
   is explicit that a gate hiding a wrong rule is a debt, and this is the debt.
8. **Second-riskiest, and it is a *binding* rather than a lowering.** 280,283
   functions now route a **method** token through `gl_symbol_index`, and
   `GAPS.md` §6 records that same index missing 12,505 of 33,059 `?`-mangled
   names because its anchor was a byte value rather than a field — a failure
   that presented as a stubborn census bucket, not as an alarm. A wrong callee
   name is a relocation against the wrong symbol, so `Port=Match` on 25 fixture
   functions, 165 sweep cases and 9 probes IS evidence for the manglings those
   contain: the positive fixture now pins a namespaced class, a nested class, an
   overload set, an `operator[]` and a member of a class template, all
   byte-exact. What is **not** covered is the invariant D14 used for generated
   destructors — *a population whose answer the source language fixes* — asked
   of member calls: every accepted one must resolve to a **method** mangling,
   and nothing checks that. The instrument exists (`census` already prints
   "N bound, 0 to a NON-destructor" for the destructor family) and this rung did
   not add its equivalent.
