# GRID-L — the inserted literal, eighteen cells

Every cell compiled with the real `c2.dll` under wibo at the workload's own
flags and cwd; C++ linkage and long names throughout, so GRID-T's 8-byte
inline-name fence cannot decide any of them. The list under test is the formals
**in order** with one literal spliced in at slot `j`.

| family | n | j | moves | emitted |
|---|---|---|---:|---|
| **l1** | 2…7 | n−1 | 1 | `mr r<j+4>,r<j+3>` · `li r<j+3>,k` · `b callee` |
| **l2** | 3…7 | n−2 | 2 | `mr` · `mr` · `li` · `b` |
| **l3** | 2…5 | n | 0 | `li` · `b` |
| **l4** | 3…5 | 0 | n | `mr` ×n · `li r3,k` · `b` |

`l1_n4` is `vsnprnc.cpp::vsprintf_s` word for word:
`mr r7,r6 ; li r6,0 ; b _vsprintf_s_l`.

## The lowering, 18 of 18, no exceptions

**The moves run in DESCENDING destination order and the `li` is last.**

## What this grid does NOT settle, stated because the temptation is real

In this family the two candidate rules **cannot be told apart**: the literal's
destination is the lowest of all the destinations (so a plain descending walk
puts it last) *and* the highest move's chain reads it (so a dependence rule puts
it last too). The cell that separates them is the shipped WLB one, `g2(b,7)` →
`mr r3,r4 ; li r4,7`, where descending-destination over the union is **wrong**.

So **GRID-L is not the second grid board #1484 is waiting for.** #1484 was
declined for being fitted to the cells that refuted its predecessor; a grid that
agrees with it only where nothing could disagree would be the same error with a
larger n. The class shipped here is stated in its own axes — *the formals in
order with one literal spliced in* — and it is 18 of 18 on those.

## Disjointness from the shipped WLB cell, by construction

`one_moved_at_two` admits `[Formal(1), Lit]`, which **drops** formal 0; no
splice produces that. Its own measured counterexample `g3(c,a,7)` =
`[Formal(2), Formal(0), Lit]` moves one formal up and another down; no splice
produces that either. Both are still refused, and there is a `#[test]` asserting
the second one is.

## STRUCTURAL BLIND SPOT

The grid varies arity, the literal's slot and therefore the move count. It holds
fixed: **one** literal, all-`int` formals, the literal's value (0), a tail call
rather than a framed one, and an external callee. It cannot see a rule that
depends on any of those.
