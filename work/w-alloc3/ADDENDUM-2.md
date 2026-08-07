# w-alloc3 — PREREG ADDENDUM 2: the two GRID-A refinements

    Written:   after GRID-A was compiled and graded (`gridA.tsv` at commit
               below), and **before GRID-H's predictions were frozen and before
               the first `cl.exe` ran on GRID-H**. `PREREG.md` §4 step 2 is the
               step this is.
    GRID-A:    38 cells, 38 graded, **35 HIT / 3 MISS**, 0 out of domain.

---

## A2.1 The three misses, printed

```text
  A-two-SUM-10    g = [7c632214 4e800020]      add r3,r3,r4     ; blr
     RULE BIND     7c641a14 4e800020           add r3,r4,r3
     c2            7c632214 4e800020           add r3,r3,r4

  A-twoa-SUM-10   g = [7c632214 4e800020]
     RULE BIND     7d641a14 386b0007 4e800020  add r11,r4,r3 ; addi r3,r11,7
     c2            7d632214 386b0007 4e800020  add r11,r3,r4 ; addi r3,r11,7

  A-arith-add1    g = [38630001 4e800020]      addi r3,r3,1     ; blr
     RULE BIND     39630001 386b0005 4e800020  addi r11,r3,1 ; addi r3,r11,5
     c2            38630006 4e800020           addi r3,r3,6
```

**Neither family is an allocation decision, and both are already named
elsewhere in the record.**

* The first two are **commutative canonicalisation**. `a + b` compiles to
  `add r3,r3,r4`; binding the actuals in the other order leaves the *same*
  instruction, because `add` does not care. RULE BIND's TEMP clause is right in
  both — `r11` in `A-twoa-SUM-10` is exactly where RULE BIND puts it — and its
  BIND clause writes a semantically equal word with the two source fields
  swapped. The matching non-commutative cell is a **HIT**:
  `A-two-SUB-10`, `subf r3,r4,r3` → `subf r3,r3,r4` (`7c641850` →
  `7c632050`), which is `w-seq`'s `s11` reproduced at the byte.
* The third is **constant folding**, `w-seq` §4.2's third field family and its
  `s04`/`s05` cells: `+1` inside the callee and `+5` at the site become
  `addi r3,r3,6`, and the inlined value never needs a register at all.

## A2.2 The refinement is a NARROWING, in both cases

Both are handled by **refusing**, not by modelling:

> **D10 (COMMUTATIVE).** If any instruction of the callee's body has two
> register source fields that hold two **different** live formals and its
> operator is commutative in those two operands — `add addc adde and or xor
> nand nor eqv mullw mulhw mulhwu`, and the indexed forms `lwzx lbzx lhzx lhax
> stwx stbx sthx` whose effective address is the sum `RA+RB` — the cell is
> **OUT OF DOMAIN**, clause `D10-commutative`.
>
> **D11 (CONSTANT FOLD).** In `arith` mode, if the instruction producing the
> callee's return value is a D-form immediate add (`addi`/`addis`), the cell is
> **OUT OF DOMAIN**, clause `D11-const-fold`.

**A narrowing can turn a miss into a refusal and can never turn a refusal into
a wrong emit.** That is the incumbent's own discipline — `codegen::alloc`
refuses a mixed run, a multiply and a fourth producer for exactly this reason —
applied one level up, and it is why these two are refusals rather than the
seventh and eighth fitted clauses.

**D10 is stated with no free parameter.** The obvious cheaper form — *"refuse
only when the substitution REORDERS the pair"* — would have kept
`A-two-SUM-01` as a hit, and it would have been fitted on the direction c2
happened to canonicalise in one cell. The clause as written refuses the
identity binding too, costs two GRID-A hits, and asks nothing of the data. The
indexed forms are in the list **before any cell containing one has been
compiled**, on the a-priori ground that `RA+RB` is a sum: that is the one place
this addendum is allowed to be predictive, and it is registered here so it
cannot be read as a post-hoc rescue.

## A2.3 D11 is a rung a successor can take, and it is priced here

The fold is **mechanically predictable** — `addi r3,rA,c1` at the site `± K`
is `addi r3,rA,c1+K`, and that reproduces `A-arith-add1`'s `38630006` exactly.
It is not taken, for two reasons: it is a *constant* fold and this lane's
hypothesis is about *registers*, and it would be a branch of the rule fitted on
the single cell that produced it — which is the standing every one of the six
dead keys had. The grid that would decide it is named in the rung.

## A2.4 What the refinement does NOT touch

BIND, TEMP, `POOL_TOP = r11`, D1–D9, the six rivals, the nine registered
predictions and the decline floor are unchanged. In particular **P1 is
unchanged**: RULE BIND must be **0 wrong** on GRID-H's in-domain cells, and
D10/D11 shrink that population rather than protecting it — every cell they
remove is *counted and printed* as a refusal, so the shrinkage is visible.

## A2.5 GRID-A after the refinement

Re-graded in the same commit. The expected reading is **36 HIT / 0 MISS /
2 refused by D10 / 1 refused by D11** — registered here before the re-grade
ran, so that a different number is a finding and not a silent correction.
