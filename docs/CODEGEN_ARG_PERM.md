# Argument marshalling — the permutation lowering, over complete grids

`docs/CODEGEN_FRAMED_CALLS.md` §3.2 states the rule as *"non-conflicting moves
are emitted highest destination first; a permutation is broken with **r11** as
the scratch"*, with three witnesses. `docs/CODEGEN_FP_ARGS.md` §1.1 adds that the
FP file uses **f0** the same way. §6 records that when a permuted value is also
callee-saved the cycle is broken through the callee-saved register instead, with
"which saved register is the temp when several are saved" explicitly **refused,
not fitted**.

This document measures the same thing over the **complete permutation grid** at
each arity — 2, 6, 24 and 120 cells at n = 2…5, in two families, 304 objects —
so that a candidate model is scored on every cell instead of the three that were
lying around. `scripts/gt_argperm.py` generates, compiles and scores it.

Bytes out of `cl.exe` 16.00.11886.00 under wibo 1.0.1-23, `/O1 /GS- /c`.

---

## 1. The two families

```c
  --pure    void f(int a1..an){ gn(a_p1, ..., a_pn); }
```
A tail call: no frame, no saved registers, nothing in the body but the moves.

```c
  --saved   void f(int a1..an){ gn(a_p1,...,a_pn); v1(a1); ... vn(an); }
```
The same first call, then one single-argument call per formal so that **every**
formal is live across the first call and must be callee-saved.

Notation throughout: destination slot *k* is `r(2+k)` and wants formal
`perm[k]`, which lives in `r(2+perm[k])`. Write **σ(d)** for that source; σ is a
permutation of the argument registers and decomposes into cycles. Write a cycle
as the cyclic sequence `c0, c1 = σ(c0), …`; `ci` is a **local minimum** when
`c(i-1) > ci < c(i+1)`, cyclically.

---

## 2. `--pure` — the scratch count, ESTABLISHED

> **The number of scratch registers is the total number of local minima of the
> cycles of σ.** They are handed out **r11, then r10**, and for each local
> minimum `ci` the register parked is `σ(ci) = c(i+1)`. The parks are emitted
> first, in **ascending order of the parked source register** (not in cycle
> order). Each local minimum then reads its value back at the very end, in
> **descending order of the minimum**.

Scored over all 152 cells at n = 2…5: **the move multiset is exact in 152/152,
the park prefix is exact in 152/152, and the read-back suffix is exact in
152/152.** (`scripts/gt_argperm.py --pure --model`.)

Why the published rule survived to length 3 and no further: a cycle of length
≤ 3 has exactly one local minimum, because three elements cannot form a valley
after the anchor. **"One r11 breaks the cycle" is not a rule about permutations
— it is a rule about cycles short enough to be unimodal.** The first
counterexamples appear at n = 4, and they are the two 4-cycles whose sequence
after the minimum descends and then ascends:

```
  (2,3,4,1)  cycle r3 r4 r5 r6   seq 4,5,6  one minimum   ->  r11<-r4 ; r4<-r5 ; r5<-r6 ; r6<-r3 ; r3<-r11
  (3,4,2,1)  cycle r3 r5 r4 r6   seq 5,4,6  TWO minima    ->  r11<-r5 ; r10<-r6 ; r6<-r3 ; r5<-r4 ; r4<-r10 ; r3<-r11
```

Both are 4-cycles; the first needs one scratch, the second two, and nothing
about "the length of the cycle" separates them. The grid census:

| cycle structure of σ | 1 scratch | 2 scratches |
|---|---:|---:|
| one 2-cycle | 10 | 0 |
| one 3-cycle | 20 | 0 |
| one 4-cycle | 20 | 10 |
| one 5-cycle | 8 | 16 |
| 2-cycle + 2-cycle | 0 | 15 |
| 2-cycle + 3-cycle | 0 | 20 |

(n = 5 rows; the local-minimum count predicts every cell, 120/120.) **No cell at
n ≤ 5 ever used r9** — a third scratch is predicted by the rule (a 6-element
cycle with two valleys, or three cycles) and is **not captured**.
*(2026-07-31: captured at n = 6, and the reason it could not appear below that
is that n ≤ 5 admits at most two local minima. See §5.)*

### 2.1 What is still uncharacterized: the interleaving

The parks and the read-backs are exact; the **order in which independent chains
are interleaved between them** is not. 26 of the 120 cells at n = 5 differ from
"chains in descending order of their first destination", 0 of the 32 cells at
n ≤ 4 do. Every one of the 26 is a reordering of the *same* instructions:

```
  (4,5,1,2,3)  got  r11<-r6 ; r10<-r7 ; r7<-r5 ; r6<-r4 ; r5<-r3 ; r4<-r10 ; r3<-r11
               pred r11<-r6 ; r10<-r7 ; r7<-r5 ; r5<-r3 ; r6<-r4 ; r4<-r10 ; r3<-r11
  (4,3,5,1,2)  got  r11<-r5 ; r10<-r6 ; r5<-r7 ; r7<-r4 ; r6<-r3 ; r4<-r11 ; r3<-r10
               pred r11<-r5 ; r10<-r6 ; r6<-r3 ; r5<-r7 ; r7<-r4 ; r4<-r11 ; r3<-r10
```

Three orderings were tried and each is refuted by one of these two cells:
descending destination globally (refuted by every 1-scratch chain, where the
dependencies force ascending), greedy "highest legal destination" (refuted by
`(4,3,5,1,2)`, which takes r5 while r6 is legal), and chain order by park order
(refuted by `(4,5,1,2,3)`). **Stated as uncharacterized rather than fitted a
fourth time.** *(2026-07-31: §5.3 raises the corpus from 26 refuting cells to
370 across two arities, and adds an inversion any candidate rule must explain —
the cells needing THREE scratches are the ones that come out exact.)*

An emitter can still use this: the multiset is exact, so a body whose σ has one
non-trivial cycle is fully determined (0/32 refutations at n ≤ 4, and every
1-scratch cell at n = 5 is a single chain and therefore forced).

---

## 3. `--saved` — the question §6 refused, ANSWERED

> **There is no temp.** Across all 152 cells of the saved family, **`r11` is
> never emitted, at any arity, for any permutation** (152/152). Each permuted
> value is read from **its own** callee-saved register. The question "which
> saved register is the temp when two are saved and both permuted" has no
> answer because c2 does not pick one.

The complete rule, exact on 152/152 cells:

1. **Assignment.** Every formal live across the call is copied to a
   callee-saved register; the live formals take **r31, r30, r29, … in parameter
   order**. Save set exact in 152/152.
2. **Sources.** The permutation writes read the **original** argument register
   when it still holds the value, and the value's **callee-saved home** when the
   original has already been overwritten. Exact in 152/152.
3. **Order.** The permutation writes go in **descending destination order** —
   120/152; the 32 exceptions are the same independent-chain interleaving
   residue as §2.1, and the source rule (2) holds in all of them regardless.

```
  n=4, perm (2,3,4,1), every formal live
     r30<-r4 ; r29<-r5 ; r28<-r6 ; r31<-r3      the saves
     r6<-r3                                     a1, still in r3
     r5<-r28                                    a4 — r6 already overwritten
     r4<-r29                                    a3 — r5 already overwritten
     r3<-r30                                    a2 — r4 already overwritten
```

### 3.1 The boundary §6's phrasing does not have

§6 says *"when a permuted argument's value is also callee-saved, c2 breaks the
cycle through the callee-saved register and emits **no r11 at all**"*. That is
true when **every** permuted value is saved. With a *partial* live set both
scratches come back, alongside the saved registers:

```
  void g4(int,int,int,int); void f(int a,int b,int c,int d){ g4(d,c,b,a); ... }

  only a3 live:      r11<-r4 ; r10<-r6 ; r31<-r5 ; r6<-r3 ; r4<-r5 ; r5<-r11 ; r3<-r10
  a3 and a4 live:    r11<-r3 ; r10<-r4 ; r31<-r5 ; r30<-r6 ; r4<-r5 ; r3<-r6 ; r6<-r11 ; r5<-r10
  a1,a3,a4 live:     r11<-r4 ; r29<-r6 ; r31<-r3 ; r30<-r5 ; r6<-r3 ; r4<-r5 ; r3<-r29 ; r5<-r11
```

So the saved registers do not *replace* the scratch mechanism; they remove the
need for it only for the values that happen to be saved. **An implementation
that reads "callee-saved ⇒ no r11" as a property of the function rather than of
the individual value emits the wrong instruction the first time a call permutes
a mix of live and dead formals** — which is the common case, not the corner.

---

## 4. The register-assignment order, ESTABLISHED past n=2

`docs/CODEGEN_FRAMED_CALLS.md` §3.1 and §6 establish the assignment for
`nGPRsaved ∈ {1,2}` and record it as unmodelled at n ≥ 3. Measured here over
every subset of a 4-formal list and over the full 152-cell grid to 5 saved
registers:

> **The formals live across the call take r31, r30, r29, r28, r27 in parameter
> order.** It *is* monotone at n ≥ 3, and the assignment is over the **live
> subset**, not over the formal index — `live = {a2, a4}` gives
> `r31<-r4 ; r30<-r6`, not `r30/r28`.

What is not monotone, and is what made this look unmodelled, is a **different
question**: the order in which the save `mr`s are *emitted*. `live = {a1,a3,a4}`
emits `r11<-r4 ; r29<-r6 ; r31<-r3 ; r30<-r5` — the assignment r31/r30/r29 is in
parameter order, the emission is not. Separating the two is the whole content of
this section: **assignment order and emission order are two facts that were
sharing one name.** Only the emission order is open, and it is open in exactly
the same way as §2.1 and §3.3 — it is one residue, not three.

---

## 5. n = 6 — **r9 observed**, and the residue is 20× bigger than it looked (2026-07-31)

§2 predicted a third scratch and recorded it as *"a third scratch is predicted by
the rule … and is **not captured**"*. It is captured now, and the reason it never
was is arithmetic rather than luck: **n ≤ 5 cannot produce three local minima at
all.** The histogram of the predicted scratch count over the full grid —

| n | 0 minima | 1 | 2 | 3 |
|---:|---:|---:|---:|---:|
| 5 | 1 | 58 | 61 | **0** |
| 6 | 1 | 179 | 479 | **61** |
| 7 | 1 | 543 | 3111 | **1385** |

`scripts/gt_argperm.py` grew a `--minima K` filter so the 61 deciding cells can
be compiled without the 659 that decide nothing.

### 5.1 The prediction holds, verbatim

```
  void f(int a1,int a2,int a3,int a4,int a5,int a6){ g6(a4,a5,a6,a1,a2,a3); }

    mr r11,r6 ; mr r10,r7 ; mr r9,r8            <- three parks, r11 then r10 then r9
    mr r8,r5  ; mr r7,r4  ; mr r6,r3            <- the three chains
    mr r5,r9  ; mr r4,r10 ; mr r3,r11           <- three read-backs, descending minimum
    b ?g6@@YAXHHHHHH@Z
```

σ is `(r3 r6)(r4 r7)(r5 r8)` — three 2-cycles, three local minima, three
scratches. **r9 appears in 61 of 61 three-minima cells and in 0 of the other
479**, exactly where the rule puts it. The scratch registers are handed out
`r11, r10, r9` in ascending order of the parked source, as §2 says.

Reproduce: `scripts/gt_argperm.py --pure --n 6 --minima 3 --model`.

### 5.2 …and on those 61 cells the model is exact, interleaving included

**0 refutations / 61.** Every one of the 61 three-minima cells matches the
predicted instruction *sequence*, not just its multiset.

### 5.3 The interleaving residue at n = 6 — 344 cells, not 26

That makes the two-minima cells the interesting ones, and there the picture is
much worse than §2.1's 26:

| grid | cells | refutations | |
|---|---:|---:|---|
| n = 5, ≥ 2 minima | 61 | 26 | 43 % |
| n = 6, 3 minima | 61 | **0** | 0 % |
| n = 6, 2 minima | 479 | **344** | 72 % |

**In all 344 the move multiset is identical to the prediction** (checked
mechanically: sorted move lists compare equal in 344/344), so this is still
purely the chain-interleaving order of §2.1 and §3.3 — the scratch count, the
parks and the read-backs remain exact everywhere.

Two things follow, and neither is a new ordering rule:

* **The residue is not "more scratches ⇒ more reordering."** The three-minima
  cells, which have the most chains to interleave, are the ones that come out
  perfect. Any candidate rule has to explain that inversion, and the two
  orderings §2.1 already refuted do not.
* **There is now a 344-cell corpus to test a candidate against instead of two
  hand-picked cells.** §2.1 was refuted by exactly two cells, which is enough to
  kill a rule and nowhere near enough to confirm one. A fourth ordering fitted
  to 26 cells and passing would still have been a coin toss; one that survives
  344 across two arities would not be.

Still **stated as uncharacterized rather than fitted a fifth time.** The
emitter-usable boundary is unchanged and now has a much larger warrant: every
cell with a single non-trivial chain is forced by its dependencies and is exact,
at n = 6 as at n ≤ 5.

---

## 6. Reproduction

```sh
export C2RS_WIBO=<the repo's resolved wibo>       # NOT ../wibo/build/wibo
scripts/gt_argperm.py --pure  --n 2,3,4,5         # the grid, as emitted
scripts/gt_argperm.py --pure  --model --n 2,3,4,5 # score §2's model, print refutations only
scripts/gt_argperm.py --saved --n 2,3,4,5         # the callee-saved family
scripts/gt_argperm.py --one 3,1,2                 # one permutation, disassembled
scripts/gt_argperm.py --pure --model --n 6 --minima 3   # §5: r9, 61 cells, 0 refutations
scripts/gt_argperm.py --pure --model --n 6 --minima 2   # §5.3: the 344-cell residue
```

`--minima K` filters the grid to the cells whose predicted scratch count is at
least K. It is what makes n = 6 tractable: the full grid is 720 objects and 659
of them repeat what n ≤ 5 already said.
