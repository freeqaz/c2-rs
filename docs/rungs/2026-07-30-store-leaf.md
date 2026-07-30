# W25 — the store leaf

    Tag:       W25
    Slug:      store-leaf
    Date:      2026-07-30
    Fixtures:  w25_store_leaf.cpp w25_store_leaf_neg.cpp
    Census:    418,628 → 442,273 (17.00 % → 17.96 %), +23,645
    Record:    docs/IL_STORE_LEAF.md; docs/ROADMAP.md §6i

`void f(S* s, int v) { s->m = v; }` is one `stb`/`sth`/`stw`/`std` at a folded
displacement, and it is the **third** consumer of the sub-object designator the
indirect-load leaf (`lwz`) and the address leaf (`addi`) already share — which
is why that fact now has one locator, `c2-il/src/func/body/shapes/designator.rs`.

Three candidates were measured before anything was implemented. Estimate
+22,821, biased LOW; the +824 residual is the class-preserving `2C` on the
stored value. Mismatch 0, no TU changing class, **0 new census keys**,
disagreement 0, and the sum of every blocker key's delta is exactly −23,645 —
the bucket drop equalling the gain, for the sixth rung running. All 23,645
admitted bodies read `calls-0`.

Swept by `scripts/sweep.d/80-store-leaf.py`.
