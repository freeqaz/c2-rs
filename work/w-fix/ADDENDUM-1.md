# w-fix ADDENDUM-1 — GRID-3b, and the one thing GRID-3 cannot answer

**Committed before one cell of GRID-3b is compiled.** GRID-3's own result
(`grid3.out`, committed at `8d9c153`) is the input to this addendum and is not
re-opened by it.

## Why there is an addendum at all

GRID-3 grades the chain at depths **1, 2, 3, 4** and every edge of all four is
`E` with the caller a bare `blr`. `PREREG.md` P1 registered exactly that range
and it held. But the rule §2 registers — *iterate to a fixpoint* — fires at
**every** depth, and the decline floor's clause 4 says the port may not fire on
a shape no cell graded. Two ways out, and only one of them is a measurement:

* cap the recursion at 4. Arbitrary: nothing in c2's observed behaviour suggests
  a bound, and a cap would be a constant fitted to the grid's own edge.
* **grade deeper.** Depths 5, 6 and 8, and if the pattern survives 8, an
  unbounded iteration is an interpolation between graded points rather than an
  extrapolation past the last one.

This addendum takes the second. It also closes three axes GRID-3 crossed only at
the top of a chain, where a fixpoint has to cross them in the *middle*:

| cell | the axis it moves into the middle of a chain |
|---|---|
| `m4_seq_mixed_mid` | a mid-node with **two** calls, one elidable and one not. `w-empty`'s `f08_mixed` graded this shape as a *caller*; as a *link* it decides whether "reduces to nothing" may be asked of a `Seq` body at all |
| `m5_static_chain` | internal linkage at every link. `INLINE_PREDICATE.md` §1.1 measured linkage-independence for one step |
| `m6_defined_after` | every definition **below** its use — the fixpoint must be order-independent, and an implementation that resolves in stream order might not be |
| `m8_arg_every_link` | an argument computed at **every** link, not just the top: the setup has to be discarded k times, not once |
| `m7_dtor_chain_d4` | board #924's own family — `??1?$_Rb_tree_base@…` over `??1?$_STLP_alloc_proxy@…` — one link deeper than GRID-3's `k18` |

## What can lose

* **A1.** Depth 5, 6 and 8 are `E` at every edge, caller a bare `blr`, at both
  flag settings. *Loses if any depth stops.* A stop at depth d bounds the shipped
  rule at d−1 and that bound ships as a constant with this cell beside it.
* **A2.** `m4`'s `f → g1` is **not** `E`: a mid-node that still emits a branch
  does not reduce to nothing. *Loses if it is `E`* — in which case `Seq`
  mid-nodes are inside c2's fixpoint and the port's refusal of them is an
  under-fire to be sized, not a correctness matter.
* **A3.** `m5` and `m6` are `E` throughout — linkage and definition order do not
  enter. *Loses if either stops the chain.*

Nothing in `crates/` has been written when this file is committed.
