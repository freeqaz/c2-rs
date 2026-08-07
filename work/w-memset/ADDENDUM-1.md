# w-memset ADDENDUM-1 — GRID-L, and every prediction, registered per cell

Committed **before the first `cl.exe`** of any cell. Manifest
`work/w-memset/CELLS.sha256`.

Grading route: `w-empty`/`w-inl0`'s — the real toolchain at the workload's own
profile, twice (once plain, once with `/Ob0` appended, because at `/O1` alone an
absent REL24 cannot tell **E** from **I**), with a prepended ANCHOR whose callee
the TU does not define and a five-level TAIL PAD (`w-inl0` §4 measured both, and
without either the cells grade nothing).

**A cell whose ANCHOR is not `Exact` is REFUSED, not scored.**

| cell | the structural axis it moves | registered prediction |
|---|---|---|
| **l01** | THE SHAPE — all five levels, class element | c2 emits **one `4e800020`, zero relocations** for `?use`'s chain at both flag settings; the port converts **nothing**, and the stop is level 5's seed |
| **l02** | the loop's callee is `empty_body` | **THE POSITIVE.** The chain closes through the loop LINK alone: `?destroy_range` grades `Exact` with c2's `blr`, and `elide` admits `aux`, `dr`, `destroy_range` |
| **l03** | the loop's callee **keeps bytes** (a store to a global) | nothing is admitted; `?destroy_range` is an honest `Differs` |
| **l04** | CONTROL — the loop's callee is **external** | c2 keeps a REL24 in the chain; nothing is admitted |
| **l05** | **two statements** in the loop body | the reader refuses; nothing is admitted |
| **l06** | the induction step is a **call** | the reader refuses |
| **l07** | the condition reads a **global**, not a formal | the reader refuses |
| **l08** | the loop's statement is a **dead-temporary call** (the workload's own level 3→4 edge) | the two readers COMPOSE and the chain closes |
| **l09** | THE RESIDUE — the loop's leaf is a **pseudo-destructor** | c2 emits one `4e800020`; the port converts **nothing**, and the reason is the missing **seed**, not a missing production |
| **l10** | CONTROL — a loop whose body is a **store** and has no call | the reader refuses; c2 emits real code |
| **l11** | THE CYCLE, through a loop | neither member admitted, `overflowed()` false |
| **l12** | stride **2** and a **`<`** test instead of `++`/`!=` | fires exactly as l02 — the reader is not keyed on the literal `4` or on the `!=` opcode |

## The predictions most expected to lose

1. **l12.** The workload only ever shows `++`/`!=` with a stride equal to
   `sizeof(T)`. If the reader has to be widened to take `<` or a stride of 2,
   that is a value being smuggled into a grammar (#644), and the honest answer
   is to refuse `<` and say so rather than to special-case it.
2. **l09's `/Ob0` row.** `w-inl0`'s m06 lost the analogous prediction — c2 kept
   emitting `4e800020` at `/Ob0`, so the erasure is its own dead-code
   elimination and not its inliner. l09 is registered to reproduce that, and if
   it does **not** the residue is mechanism I and this whole rung is priced
   wrong.
3. **l01/l09 converting nothing.** If either converts, `PREREG.md` P1's point
   prediction of **0** is a LOSS and the seed analysis in `PREREG.md` §0.3 is
   wrong — which would be the better outcome and must be published as such.

## What a cell may NOT be used for

No cell licenses a widening of `parse_segment`, of `IlBundle::functions()`, or
of `crates/c2-core/src/elide.rs`. Every one of them grades a **decode-only**
fact whose single consumer is the fixpoint that already ships.
