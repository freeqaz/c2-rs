
### 10.26.4 w-mcall correction — §10.26.3's ordering rests on a premise that is WRONG for 90.5 % of the family (2026-08-08)

§10.26.3 closed item 2 as a lever with the sentence *"the only thing that moves
those 33,277 is a member-call **lowering** — a call in an expression, which the
emitter has no representation for at all — and no further reader work will touch
them"*. **The first half is right about the wrong population and the second half
is false.**

`w-value` §4.2's own split says the 33,277 are the bodies with **nothing else in
their expression**. There are two readings of that, and the rung took one:

* the call is an **operand** of an enclosing expression whose other operands are
  all consumed — this needs an operand-position lowering the emitter really does
  not have. That is w-value's **1,168 (3.2 %)**, not the 33,277;
* the call **is the statement**. `p->m(a…)` is `m(p, a…)` on this ABI, so it is a
  statement-position call with one more argument slot, and `BodyShape::CallSeq`
  has lowered a *sequence* of statement-position calls byte-exactly since #35
  step 2. **The lowering seam was open; the reader could not reach it.**

Lane `w-mcall` (board **#1960**–**#1966**) shipped the reader route:
`crates/c2-core` is byte-for-byte unchanged, `fixtures/cpp/wmcall_seq.cpp` is a
whole-TU `match` at `/O1` and `/Ox`, and the census moves **711,494 → 711,514 /
39,193 → 39,200** with `fnbyte-differs` unmoved, so every function it adds is
byte-exact against real c2. TU match **18 → 18** and the FRONTIER's nine members
are byte-identical per TU and per key.

**And the number that actually changes the plan is how small it is.** The class
is **20 bodies / 7 emitted** on the workload, against the **1,505 emitted** its
first-blocker key carries — 0.46 % of its own ceiling, and four optimistic PREREG
misses (#1964). So §10.26.3's *"no further reader work will touch them"* is
refuted in kind and confirmed in magnitude: reader work does touch them, and
barely.

**The seam's own next step is priced** (#1963), by a reverted scratch instrument
over the 159,068 bodies whose member call is not the whole body:

| the sequence route refused with | bodies | whose seam |
|---|---:|---|
| `call-ref` — the next statement is not a call at all | **125,458 (78.9 %)** | board **#844**'s COMPOSITION seam (`StoreRunCall` is its first member) |
| `call-token` — a chained / named-object / intrinsic receiver in a later statement | **25,060 (15.8 %)** | **this** seam — decline D2 one position over |
| `this-undetermined` · `expr` · `result-type` · `formals-marker` · tail | 8,550 (5.4 %) | mixed |

Neither is a TU rung, and §10.26.2's correction still stands: these are construct
rungs shipped on manufactured cells. **The re-ordering §10.26 licenses is
otherwise unchanged** — item 4's `loop_guard+bdnz` class and item 3's inline
decline side are still the sequenced next code seams, and item 2 is now spent as
a *reader* lever rather than closed as an unopenable *lowering* one.

[`rungs/2026-08-08-w-mcall.md`](rungs/2026-08-08-w-mcall.md).
