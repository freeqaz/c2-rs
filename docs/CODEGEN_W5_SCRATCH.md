# CODEGEN W5 — multi-scratch expressions (characterization)

Roadmap step **W5**: expression trees deeper than a serial accumulator chain.
Today `c2_core::codegen::select_text` models a single scratch register (`r11`)
and rejects any operand stack deeper than 2 (`stack.len() > 2`), so the
canonical `(a+b)*(c+d)` is `NotImplemented`.

This document is **characterization only** — no Rust was changed. Every register
claim below is backed by instruction bytes read out of an obj produced by the
real toolchain (`cl.exe` 16.00.11886.00 under wibo, `/Ox /GS- /c`), disassembled
by hand from the big-endian `.text` payload. Nothing here comes from `paint/`.

Reference-replay status of the new fixtures (`c2rs diff`):

```
fixtures/cpp/w5_tree2.cpp    -> ReferenceReplay=ByteExact (ref=1024B replay=1024B)  Port=NotImplemented
fixtures/cpp/w5_tree3.cpp    -> ReferenceReplay=ByteExact (ref=1074B replay=1074B)  Port=NotImplemented
fixtures/cpp/w5_chain.cpp    -> ReferenceReplay=ByteExact (ref=1013B replay=1013B)  Port=Mismatch
fixtures/cpp/w5_tree_neg.cpp -> ReferenceReplay=ByteExact (ref=1654B replay=1654B)  Port=NotImplemented
```

`w5_chain.cpp` is **`Port=Mismatch`, not `NotImplemented`** — see §6. The
single-scratch model is already wrong inside the class the port currently
accepts; W5 has to fix that too, and it is the most urgent item in this document.

---

## 0. Ground rules the data establishes up front

**The IL is a plain postfix tree.** `c1xx` does *no* reassociation: the `.ex`
body for `(a+b)*(c+d)` is literally `LOAD a, LOAD b, ADD, LOAD c, LOAD d, ADD,
MUL`, and for `a*(b*(c*d))` it is `LOAD a, LOAD b, LOAD c, LOAD d, MUL, MUL,
MUL`. Every flattening, term-reordering and re-ranking documented below happens
**inside c2** and therefore has to be reproduced by the port.

**Two independent mechanisms** decide the emitted bytes and must not be
conflated:

1. **Canonicalization** — which *values* exist at all (product flattening,
   additive term-collection/reordering). This decides the instruction sequence.
2. **Register assignment** — which physical register each surviving value gets.
   This is a clean, mechanical rule (§2) and is the same in every shape measured,
   including the ones canonicalization mangles.

W5's risk is entirely in (1): a naive "post-order, one register per node"
selector reproduces (2) correctly and still emits wrong bytes, because c2 has
already rewritten the tree.

---

## 1. Fixtures: source, IL, and reference `.text`

The four fixture files are `fixtures/cpp/w5_tree2.cpp`, `w5_tree3.cpp`,
`w5_chain.cpp`, `w5_tree_neg.cpp` (source only — the IL and objs below are
regenerated, never committed). IL streams are the `.ex` operand stream in the
`crates/c2-il/src/func.rs` grammar (`LOAD`/`LIT`/`ADD`/`SUB`/`MUL`, postfix).

### 1.1 `w5_tree2.cpp` — depth-2 trees (the canonical rejected shape)

```c
int t2_mul_add(int a, int b, int c, int d) { return (a + b) * (c + d); }
int t2_mul_sub(int a, int b, int c, int d) { return (a - b) * (c - d); }
int t2_sub_mul(int a, int b, int c, int d) { return (a * b) - (c * d); }
int t2_add_mul(int a, int b, int c, int d) { return (a * b) + (c * d); }
```

IL (`a`=`E309`… by declaration order → `r3`,`r4`,`r5`,`r6`):

```
t2_mul_add: LOAD a LOAD b ADD  LOAD c LOAD d ADD  MUL
t2_mul_sub: LOAD a LOAD b SUB  LOAD c LOAD d SUB  MUL
t2_sub_mul: LOAD a LOAD b MUL  LOAD c LOAD d MUL  SUB
t2_add_mul: LOAD a LOAD b MUL  LOAD c LOAD d MUL  ADD
```

Reference `.text` (0x40 bytes):

```
?t2_mul_add@@YAHHHHH@Z
 0000 7d632214   add   r11,r3,r4      ; left  = a+b   -> r11
 0004 7d453214   add   r10,r5,r6      ; right = c+d   -> r10
 0008 7c6b51d6   mullw r3,r11,r10     ; root  (rA=left, rB=right)
 000c 4e800020   blr
?t2_mul_sub@@YAHHHHH@Z
 0010 7d641850   subf  r11,r4,r3      ; a-b (subf: rD = rB-rA)  -> r11
 0014 7d462850   subf  r10,r6,r5      ; c-d                     -> r10
 0018 7c6b51d6   mullw r3,r11,r10
 001c 4e800020   blr
?t2_sub_mul@@YAHHHHH@Z
 0020 7d6321d6   mullw r11,r3,r4      ; a*b -> r11
 0024 7d4531d6   mullw r10,r5,r6      ; c*d -> r10
 0028 7c6a5850   subf  r3,r10,r11     ; r11-r10 = left-right    (rA=rhs, rB=lhs)
 002c 4e800020   blr
?t2_add_mul@@YAHHHHH@Z
 0030 7d4321d6   mullw r10,r3,r4      ; a*b -> r10   <-- SWAPPED
 0034 7d6531d6   mullw r11,r5,r6      ; c*d -> r11   <-- SWAPPED
 0038 7c6a5a14   add   r3,r10,r11
 003c 4e800020   blr
```

### 1.2 `w5_tree3.cpp` — depth-3 trees (the multi-scratch case proper)

```c
int t3_four (int a,int b,int c,int d)                       { return ((a+b)*(c+d)) - ((a+c)*(b+d)); }
int t3_eight(int a,int b,int c,int d,int e,int f,int g,int h){ return ((a+b)*(c+d)) - ((e+f)*(g+h)); }
int t3_wide (int a,int b,int c,int d,int e,int f)           { return ((a+b)*(c+d)) - (e*f); }
int t3_wide_flip(int a,int b,int c,int d,int e,int f)       { return (e*f) - ((a+b)*(c+d)); }
```

IL:

```
t3_four:      LOAD a LOAD b ADD  LOAD c LOAD d ADD  MUL  LOAD a LOAD c ADD  LOAD b LOAD d ADD  MUL  SUB
t3_eight:     LOAD a LOAD b ADD  LOAD c LOAD d ADD  MUL  LOAD e LOAD f ADD  LOAD g LOAD h ADD  MUL  SUB
t3_wide:      LOAD a LOAD b ADD  LOAD c LOAD d ADD  MUL  LOAD e LOAD f MUL  SUB
t3_wide_flip: LOAD e LOAD f MUL  LOAD a LOAD b ADD  LOAD c LOAD d ADD  MUL  SUB
```

Reference `.text` (0x70 bytes):

```
?t3_four@@YAHHHHH@Z                        ; 4 params -> r7..r11 free
 0000 7d632214   add   r11,r3,r4    ; a+b -> r11
 0004 7d453214   add   r10,r5,r6    ; c+d -> r10
 0008 7d232a14   add   r9,r3,r5     ; a+c -> r9
 000c 7d043214   add   r8,r4,r6     ; b+d -> r8
 0010 7ceb51d6   mullw r7,r11,r10   ; left  product -> r7
 0014 7cc941d6   mullw r6,r9,r8     ; right product -> r6
 0018 7c663850   subf  r3,r6,r7     ; r7-r6 = left-right
 001c 4e800020   blr
?t3_eight@@YAHHHHHHHHH@Z                   ; 8 params -> only r11 free at entry
 0020 7d632214   add   r11,r3,r4    ; a+b -> r11
 0024 7cc53214   add   r6,r5,r6     ; c+d -> r6   (d's reg, d dies here)
 0028 7ca74214   add   r5,r7,r8     ; e+f -> r5   (c's reg, dead)
 002c 7c895214   add   r4,r9,r10    ; g+h -> r4   (b's reg, dead)
 0030 7c6b31d6   mullw r3,r11,r6    ; left  product -> r3 (a's reg, dead)
 0034 7d6521d6   mullw r11,r5,r4    ; right product -> r11 (cursor wrapped)
 0038 7c6b1850   subf  r3,r11,r3
 003c 4e800020   blr
?t3_wide@@YAHHHHHHH@Z
 0040 7d632214   add   r11,r3,r4    ; a+b   -> r11
 0044 7d453214   add   r10,r5,r6    ; c+d   -> r10
 0048 7d2741d6   mullw r9,r7,r8     ; e*f   -> r9
 004c 7d0b51d6   mullw r8,r11,r10   ; (a+b)*(c+d) -> r8
 0050 7c694050   subf  r3,r9,r8     ; r8-r9 = big - small
 0054 4e800020   blr
?t3_wide_flip@@YAHHHHHHH@Z                 ; identical prefix; only the subf swaps
 0058 7d632214   add   r11,r3,r4
 005c 7d453214   add   r10,r5,r6
 0060 7d2741d6   mullw r9,r7,r8
 0064 7d0b51d6   mullw r8,r11,r10
 0068 7c684850   subf  r3,r8,r9     ; r9-r8 = small - big
 006c 4e800020   blr
```

### 1.3 `w5_chain.cpp` — linear chains ≥ 3 ops

```c
int c4_mul(int a,int b,int c,int d)       { return a * b * c * d; }
int c4_sub(int a,int b,int c,int d)       { return a - b - c - d; }
int c4_add(int a,int b,int c,int d)       { return a + b + c + d; }
int c5_mul(int a,int b,int c,int d,int e) { return a * b * c * d * e; }
```

IL: `LOAD a LOAD b MUL LOAD c MUL LOAD d MUL` etc. — plain left-leaning chains.

```
?c4_mul@@YAHHHHH@Z            ?c4_sub@@YAHHHHH@Z            ?c4_add@@YAHHHHH@Z
 0000 7d6321d6 mullw r11,r3,r4  0010 7d641850 subf r11,r4,r3  0020 7d632214 add r11,r3,r4
 0004 7d4b29d6 mullw r10,r11,r5 0014 7d455850 subf r10,r5,r11 0024 7d6b2a14 add r11,r11,r5
 0008 7c6a31d6 mullw r3,r10,r6  0018 7c665050 subf r3,r6,r10  0028 7c6b3214 add r3,r11,r6
 000c 4e800020 blr              001c 4e800020 blr              002c 4e800020 blr
?c5_mul@@YAHHHHHH@Z
 0030 7d6321d6 mullw r11,r3,r4
 0034 7d4b29d6 mullw r10,r11,r5
 0038 7d2a31d6 mullw r9,r10,r6
 003c 7c6939d6 mullw r3,r9,r7
 0040 4e800020 blr
```

### 1.4 `w5_tree_neg.cpp` — the negative neighbours (full listing in §5)

---

## 2. THE KEY ANSWER — the scratch-register allocation order

> **The second scratch is `r10`. The third is `r9`. The fourth is `r8`.**
> c2 allocates temporaries from a **rotating cursor descending
> `r11 → r10 → r9 → r8 → r7 → r6 → r5 → r4 → r3`, wrapping back to `r11`**,
> advancing one slot per emitted value, **skipping any register that still holds
> a live value** (an argument with a later use, or a temp with a pending
> consumer). The function result always lands in `r3`.

Not `r11, r12` (r12 is never allocated — see below), and not a fixed pair. It
*is* argument-register-liveness driven, but only in the sense that live argument
registers are *skipped* and dead ones are *recycled* by the same descending
cursor.

### 2.1 Evidence — the pure descent (`t3_four`, 4 params, r7..r12 free)

Six consecutive temps, six consecutive registers, in emission order:

| # | instruction bytes | dest |
|---|---|---|
| 1 | `7d632214` `add r11,r3,r4` | **r11** |
| 2 | `7d453214` `add r10,r5,r6` | **r10** |
| 3 | `7d232a14` `add r9,r3,r5`  | **r9** |
| 4 | `7d043214` `add r8,r4,r6`  | **r8** |
| 5 | `7ceb51d6` `mullw r7,r11,r10` | **r7** |
| 6 | `7cc941d6` `mullw r6,r9,r8`  | **r6** |
| 7 | `7c663850` `subf r3,r6,r7` | r3 (result) |

`r12` is free throughout and is **never** chosen — the cursor's top is `r11`.
`r0` and `r2` are likewise never chosen. The allocatable pool is exactly
`[r11, r10, r9, r8, r7, r6, r5, r4, r3]`.

### 2.2 Evidence — skipping live argument registers (`q_m6`, scratch probe)

`int q_m6(int a..int f){ return a*b*c*d*e*f; }` (6 params → `r3..r8`):

```
7d6321d6 mullw r11,r3,r4   ; cursor r11 -> free            -> r11
7d4b29d6 mullw r10,r11,r5  ; cursor r10 -> free            -> r10
7d2a31d6 mullw r9,r10,r6   ; cursor r9  -> free            -> r9
7ce939d6 mullw r7,r9,r7    ; cursor r8  -> f LIVE, skip; r7 = e, dies here -> r7
7c6741d6 mullw r3,r7,r8    ; result                        -> r3
```

`r8` is skipped by exactly one slot because `f` is still needed by the last
`mullw`. This is the clean discriminator between "descending" and "descending,
liveness-gated".

### 2.3 Evidence — recycling dead argument registers **and the wrap** (`t3_eight`)

8 params occupy `r3..r10`, leaving only `r11` free at entry. Cursor trace:

| # | bytes | cursor scan | dest |
|---|---|---|---|
| 1 | `7d632214` `add r11,r3,r4` | r11 free | **r11** |
| 2 | `7cc53214` `add r6,r5,r6` | r10=h live, r9=g live, r8=f live, r7=e live, r6=d **dies at this insn** | **r6** |
| 3 | `7ca74214` `add r5,r7,r8` | r5 = c, dead since #2 | **r5** |
| 4 | `7c895214` `add r4,r9,r10` | r4 = b, dead since #1 | **r4** |
| 5 | `7c6b31d6` `mullw r3,r11,r6` | r3 = a, dead since #1 | **r3** |
| 6 | `7d6521d6` `mullw r11,r5,r4` | cursor **wraps** past r3 to r11; `a+b` died at #5 | **r11** |
| 7 | `7c6b1850` `subf r3,r11,r3` | result | r3 |

Two facts are pinned here that nothing else pins: (a) a register whose value's
**last read is the very instruction being allocated** is available as that
instruction's destination (`add r6,r5,r6`); (b) the cursor **wraps** from the
bottom of the pool back to `r11`.

The same trace, independently, on an 8-param chain
(`int q_m8(int a..h){return a*b*c*d*e*f*g*h;}`): dests
`r11, r5, r4, r3, r11, r9, r3` —
`7d6321d6 / 7cab29d6 / 7c8531d6 / 7c6439d6 / 7d6341d6 / 7d2b49d6 / 7c6951d6` —
wrap at the same place, then continue descending from r11 skipping the still-live
`r10` (`h`).

### 2.4 Evidence — mixed skip + recycle (`w_r4` probe, 8 params)

`((a+b)*(c-d)) - ((e*f)*(g+h))`:

```
7d695214 add   r11,r9,r10   -> r11
7d432214 add   r10,r3,r4    -> r10 (h dead)
7d262850 subf  r9,r6,r5     -> r9  (g dead)
7ceb39d6 mullw r7,r11,r7    -> r7  (cursor r8 = f LIVE, skipped; r7 = e dies here)
7cca49d6 mullw r6,r10,r9    -> r6  (d dead)
7ca741d6 mullw r5,r7,r8     -> r5  (c dead)
7c653050 subf  r3,r5,r6     -> r3
```

Nine allocations across four probes, zero deviations.

### 2.5 The one exception found: an `add`-rooted node

When the **root operator is `+`** and both children are values (not leaves), the
two children's registers are **swapped** relative to every other root operator:

```
t2_sub_mul  (a*b)-(c*d):  mullw r11,r3,r4 / mullw r10,r5,r6 / subf r3,r10,r11   left=r11
t2_add_mul  (a*b)+(c*d):  mullw r10,r3,r4 / mullw r11,r5,r6 / add  r3,r10,r11   left=r10
```

This is reproducible and order-independent: `(a*b)+(c*d)` and `(c*d)+(a*b)`
compile to **byte-identical** `.text` (`7d4321d6 7d6531d6 7c6a5a14 4e800020`),
i.e. c2 canonicalizes the commutative `+` by parameter order and then assigns
`r10` to the first term, `r11` to the second. At depth 3 the `+` root does
something different again (`((a+b)*(c+d)) + ((a+c)*(b+d))` →
`add r11 / add r10 / add r9 / add r8 / mullw r11,r11,r10 / mullw r10,r9,r8 /
add r3,r11,r10` — the two products update their left operands **in place**
instead of taking fresh cursor slots). **I cannot explain the `+`-root register
behaviour mechanistically.** It is characterized empirically at depth 2 and must
fail closed above that (§5, N6).

---

## 3. Evaluation order

> **Left-subtree-first, and within a subtree bottom-up by node height** — c2
> emits *all* height-1 nodes first (left to right), then all height-2 nodes,
> then the root. It is a level order from the leaves, **not** a post-order.
> When the two children of a node need different numbers of registers, the
> **heavier child is ranked first** regardless of which side of the operator it
> is written on.

### 3.1 Left-first, by height

`t3_four`: the four height-1 `add`s are emitted first in source order
(`a+b`→r11, `c+d`→r10, `a+c`→r9, `b+d`→r8), *then* the two height-2 `mullw`s,
*then* the root. A post-order walk would have emitted
`a+b, c+d, mullw, a+c, b+d, mullw, subf`; the observed order is
`a+b, c+d, a+c, b+d, mullw, mullw, subf`. The register numbers (r11…r6) follow
the emission order exactly, so "left first" is directly readable off the
registers: the left subtree's leaves own the **higher** numbers.

`t3_wide` makes the level order unambiguous: the small right factor `e*f` is
emitted (`7d2741d6 mullw r9,r7,r8`) **before** the left subtree's own top node
(`7d0b51d6 mullw r8,r11,r10`), because `e*f` is at height 1 and the left product
is at height 2.

### 3.2 The operand that lands in which register

For every root operator except `+`:

* left child → the **earlier** (higher) cursor register,
* right child → the next one,
* the root instruction reads `rA = left`, `rB = right`
  (`mullw r3,r11,r10` in `t2_mul_add`; `subf r3,r6,r7` = `r7 − r6` in `t3_four`,
  keeping the existing `subf rD,rA,rB = rB − rA` mapping).

### 3.3 Source order is NOT the ranking key — register need is

`t3_wide` and `t3_wide_flip` differ only in which operand of `-` the big subtree
is. Their `.text` is **byte-identical except the final `subf`'s two operand
fields**:

```
t3_wide      : 7d632214 7d453214 7d2741d6 7d0b51d6 7c694050   ; subf r3,r9,r8
t3_wide_flip : 7d632214 7d453214 7d2741d6 7d0b51d6 7c684850   ; subf r3,r8,r9
```

In `t3_wide_flip` the IL literally starts `LOAD e LOAD f MUL` — the small factor
is the *left* operand — and c2 still evaluates the heavy `(a+b)*(c+d)` subtree
first. The corroborating probe is `(a+b)*(c+d+1)`, where the *right* child is
heavier and is emitted first:
`7d653214 add r11,r5,r6` (c+d) precedes `7d432214 add r10,r3,r4` (a+b).

So: **rank the two children by register requirement, heavier first; break ties
by source order (left first); then emit level-by-level from the leaves.**

---

## 4. Immediates: materialized vs kept affine

The affine (`base + folded immediate`) model in `select_text`/`combine` survives
W5 **unchanged for parameter leaves**, and must not be extended to interior
nodes.

**Rule (verified).** A `param ± k` leaf of a tree stays affine while it is being
folded and materializes as exactly **one** `addi` (or `addis`+`addi` when wide)
whose destination is the leaf node's ordinary cursor register:

```c
int w_i1(int a,int b){ return (a+1)*(b+2); }
 39630001  addi  r11,r3,1     ; leaf a+1 -> cursor slot 1
 39440002  addi  r10,r4,2     ; leaf b+2 -> cursor slot 2
 7c6b51d6  mullw r3,r11,r10
int w_i2(int a,int b){ return (a+1)*(b-2); }
 39630001  addi  r11,r3,1
 3944fffe  addi  r10,r4,-2    ; `- k` still folds to a negative addi
 7c6b51d6  mullw r3,r11,r10
```

**Adding a constant to a *computed* value is different, and only safe in one
measured form.** When the value being offset is a `*` result, the `addi` is an
**in-place** update that consumes **no** cursor slot:

```c
int w_i7(int a,int b,int c,int d){ return ((a*b)+1)*((c*d)+2); }
 7d6321d6 mullw r11,r3,r4
 7d4531d6 mullw r10,r5,r6
 396b0001 addi  r11,r11,1     ; in place, no new register
 394a0002 addi  r10,r10,2     ; in place
 7c6b51d6 mullw r3,r11,r10
```

but when the value being offset is an **additive** node, the `+k` joins that
node's term list and the result is not an in-place update at all:

```c
int n_imm_sum(int a,int b,int c,int d){ return (a+b+1)*(c+d); }
 7d632214 add   r11,r3,r4
 7d453214 add   r10,r5,r6
 392b0001 addi  r9,r11,1      ; NEW register r9, not in place
 7c6951d6 mullw r3,r9,r10
int n_imm_sum2(int a,int b,int c,int d){ return ((a+b)+1)*((c+d)+2); }
 7d432214 add   r10,r3,r4     ; r10 FIRST — descent order broken
 7d653214 add   r11,r5,r6
 394a0001 addi  r10,r10,1     ; in place
 392b0002 addi  r9,r11,2      ; NOT in place
 7c6a49d6 mullw r3,r10,r9
```

`n_imm_sum2` is the one shape in this study whose register assignment I **cannot
explain** (`r10` before `r11`, and one `addi` in place while its mirror image is
not). It is listed as a hard negative (§5, N5); do not guess a rule for it.

**Implementable statement.** Keep `Operand::RegOff{base, off}` exactly as it is
for `Base::Phys` leaves and materialize with the existing `emit_add_imm` into the
node's cursor register. Allow a pending offset on a **`*`-produced** value only
as an in-place `addi` (dest = src). Reject a pending offset on any additive
value — that is the `n_imm_sum` / `n_imm_sum2` territory.

---

## 5. Negative neighbours — what must still fail closed

These are `fixtures/cpp/w5_tree_neg.cpp` (plus the two chain shapes of §6). All
are *tree-shaped source* that c2 does **not** lower as a tree. A "post-order,
one register per node" selector produces plausible, wrong bytes for every one.

### N1 — product flattening: a `*` node with a `*` child

A `*` with a `*` operand becomes **one n-ary product**, re-linearized into a
chain. The tree disappears; the operand pairings in the obj are not the source's.

```
n_mul_of_mul  (a*b)*(c*d)      7d6321d6 mullw r11,r3,r4    ; = ((a*b)*c)*d
                               7d4b29d6 mullw r10,r11,r5
                               7c6a31d6 mullw r3,r10,r6
n_mul_of_add  (a+b)*(c*d)      7d632214 add   r11,r3,r4    ; = ((a+b)*c)*d
                               7d4b29d6 mullw r10,r11,r5
                               7c6a31d6 mullw r3,r10,r6
n_right_mul   a*(b*(c*d))      7d6321d6 / 7d4b29d6 / 7c6a31d6   ; identical to n_mul_of_mul
```

All three compile to the same chain. A tree selector would emit
`mullw r11,r3,r4 ; mullw r10,r5,r6 ; mullw r3,r11,r10` for `n_mul_of_mul` — a
*different, wrong* instruction sequence. **Gate: reject any `*` node with a `*`
child.**

### N2 — additive canonicalization: an additive node with an additive child

`+`/`-` nodes collect into one n-ary sum whose terms are **reordered**
(subtracted terms first, then added ones) and accumulated in a single register.

```
n_add_of_add  (a+b)+(c+d)      7d632214 add  r11,r3,r4   ; a+b
                               7d6b2a14 add  r11,r11,r5  ; +c   (in place)
                               7c6b3214 add  r3,r11,r6   ; +d
n_sub_of_add  (a+b)-(c+d)      7d651850 subf r11,r5,r3   ; a-c  <-- b deferred
                               7d665850 subf r11,r6,r11  ; -d
                               7c6b2214 add  r3,r11,r4   ; +b
n_right_sub   a-(b-(c-d))      7d641850 subf r11,r4,r3   ; a-b
                               7d665850 subf r11,r6,r11  ; -d
                               7c6b2a14 add  r3,r11,r5   ; +c
n_reorder     a+b-c-d          7d651850 / 7d665850 / 7c6b2214   ; identical to n_sub_of_add
```

Note `n_sub_of_add` and `n_reorder` are byte-identical: the leaf order in the obj
is `a, c, d, b`, nothing like the source. **Gate: reject any `+`/`-` node that
has a `+`/`-` child.**

### N3 — subtree re-ranking

```
n_rerank  (a+b)*(c+d+1)        7d653214 add   r11,r5,r6    ; RIGHT child first
                               7d432214 add   r10,r3,r4
                               392b0001 addi  r9,r11,1
                               7c6a49d6 mullw r3,r10,r9
```

Re-ranking itself is modeled (§3.3) — but only for shapes that survive N1/N2.
Here it combines with an additive node and lands out of class anyway.

### N4 — spilling past the volatile pool

```
n_spill  (((a+b)*(c+d))-((a+c)*(b+d))) * (((a+d)*(b+c))-((a-b)*(c-d)))
 00b8 fbe1fff8   std  r31,-8(r1)      <-- NON-VOLATILE save, no prologue model exists
 00bc 7d642a14   add  r11,r4,r5
 ...
 00d0 7fe53214   add  r31,r5,r6       <-- r31 used as a temp
 ...
 00f8 ebe1fff8   ld   r31,-8(r1)
 00fc 4e800020   blr
```

Eight height-1 nodes with all four parameters live across them → peak live ≈ 10
values against a 9-register pool, so c2 spills into `r31` and grows a
prologue/epilogue the leaf codegen has no model for. **Gate: compute the peak
simultaneous live count (live params + live temps) and reject when it exceeds 9.**
This predicts `n_spill` (peak 10) and clears `t3_eight` (peak 8) correctly.

### N5 — immediates on additive nodes

`n_imm_sum`, `n_imm_sum2` (§4). `n_imm_sum2`'s register order is **unexplained**;
it must stay rejected.

### N6 — `+` root above depth 2

The depth-2 `(x*y)+(z*w)` swap (§2.5) is characterized; the depth-3 `+` root is
not (in-place products, §2.5). Accept only the exact depth-2 form.

---

## 6. The live bug this uncovered: `w5_chain.cpp` is `Port=Mismatch`

`c2rs census fixtures/cpp/w5_chain.cpp` reports **`4/4 functions in class`,
all `ok straight-line`** — and then the port emits wrong bytes. Side by side
(port obj via `c2rs prefilter --emit-obj`):

| function | reference `.text` | port `.text` |
|---|---|---|
| `c4_mul` | `7d6321d6` `7d4b29d6` **`mullw r10,r11,r5`** `7c6a31d6` | `7d6321d6` `7d6b29d6` **`mullw r11,r11,r5`** `7c6b31d6` |
| `c4_sub` | `7d641850` **`7d455850 subf r10,r5,r11`** `7c665050` | `7d641850` **`7d655850 subf r11,r5,r11`** `7c665850` |
| `c4_add` | `7d632214` `7d6b2a14` `7c6b3214` | `7d632214` `7d6b2a14` `7c6b3214` ✅ |
| `c5_mul` | `7d6321d6` `7d4b29d6` `7d2a31d6` `7c6939d6` | `7d6321d6` `7d6b29d6` `7d6b31d6` `7c6b39d6` |

The current single-scratch model is correct **only** for chains whose last
operation is `add` (`c4_add`, and the existing `mvp_two.cpp::add4`), because the
additive canonicalization of §5/N2 collapses an add-chain to a single accumulator
value. For a `*` or `-` chain each intermediate is a distinct value and gets its
own cursor register — `r11, r10, r9, …`, exactly the §2 rule.

The MVP fixtures never contained a `*`/`-` chain longer than two operations
(`mvp_sub.cpp::sub3` and `mul3` are 2-op, where both models coincide), which is
why this survived. **W5 must fix chains and trees in one change**; the chain case
is not a "future" gap, it is a current silent mis-emit gated only by fixture
coverage.

---

## 7. The change required in `select_text`

The current design has three assumptions that all break at once:
`Base::Prev` (a *single* running result), `SCRATCH_REG` (a *single* scratch), and
dest-by-position (`i == last ? r3 : r11`).

### 7.1 Structural changes

1. **Build a tree, not an affine stack of ≤2.** Replace the `Vec<Operand>` +
   `stack.len() > 2` guard with a real postfix→tree reduction:
   `enum Node { Leaf { reg: u8, off: i32 }, Bin { op: IlOp, lhs: Box<Node>, rhs: Box<Node> } }`.
   The stack-depth guard disappears; the *shape* gate of §7.3 replaces it.

2. **`Base::Prev` → an explicit temp identity.** Every interior node becomes a
   distinct value with its own id; `Base::Prev`'s "the running result is always
   r11" invariant is exactly the thing that is wrong.

3. **`PlanOp::Bin` carries a node id, not resolved registers.** Registers are
   assigned in a second pass (§7.2) after the emission order is fixed, because
   the destination depends on liveness of *later* nodes.

4. **Emission order (§3):** compute each node's height and register weight;
   order the children of every node heavier-first (tie → source left); then emit
   level-by-level from height 1 upward, in that ranked traversal order. The root
   is last.

### 7.2 Register assignment (the §2 rule, stated for implementation)

```
POOL = [11, 10, 9, 8, 7, 6, 5, 4, 3]        // r12, r2, r0 are NOT allocatable
cursor = 0                                   // index into POOL, starts at r11

for (i, node) in emission_order.enumerate():
    if i == last:  dest = 3                  // the function result, unconditionally
    else:
        // scan POOL cyclically from `cursor`; a register is available iff every
        // value living in it has its last read at or before instruction i
        // (a source operand dying at THIS instruction is available as its dest)
        k = first j >= cursor (mod POOL.len()) with available(POOL[j], i)
        dest = POOL[k]
        cursor = (k + 1) % POOL.len()        // one slot below the register picked
    assign(node, dest)
```

Liveness inputs: for each parameter, the index of its last `LOAD` in the emission
order; for each temp, the index of the instruction that consumes it.

Operand order at each node is unchanged from today: `rA = lhs`, `rB = rhs` for
`add`/`mullw`; `subf rD, rA=rhs, rB=lhs` (the existing load-bearing reversal in
`encode_subf`). The only deviation is the depth-2 `+`-root swap of §2.5, which
inverts which of the two children gets the earlier cursor slot.

### 7.3 The shape gate (replaces `stack.len() > 2`)

Accept only if **all** of these hold; otherwise `NotImplemented`:

| # | condition | reason (fixture) |
|---|---|---|
| G1 | no `*` node has a `*` child | product flattening — `n_mul_of_mul`, `n_mul_of_add`, `n_right_mul` |
| G2 | no `+`/`-` node has a `+`/`-` child | additive canonicalization + term reordering — `n_add_of_add`, `n_sub_of_add`, `n_right_sub`, `n_reorder` |
| G3 | the root is `*` or `-`; a `+` root is allowed only in the exact form `(x*y) + (z*w)` (both children `*`, leaves are parameters) | `t2_add_mul` is characterized, depth-3 `+` is not |
| G4 | every leaf is a parameter, optionally with a folded immediate; a pending immediate on an interior node is allowed **only** on a `*` node (in-place `addi`) | `w_i1`/`w_i7` accept, `n_imm_sum`/`n_imm_sum2` reject |
| G5 | peak simultaneous live values (live params + live temps over the emission order) ≤ 9 | `n_spill` spills to `r31` |
| G6 | ≤ 8 parameters, all `int` (unchanged) | existing ABI limit |

**A linear chain is just the degenerate tree** and needs no separate path: with
G2 in force, an all-`+` chain is rejected as an additive-of-additive… which is
wrong, since `c4_add` *does* work today. Two options, in order of preference:

* **(a)** special-case the pure additive chain that the current code already gets
  right — an n-ary sum whose terms are all leaves — as a single accumulator value
  in `r11` (exactly today's behaviour, now stated as its own rule rather than as
  a side effect). `c4_add`, `mvp_two::add4`, `mvp_sub::submix` stay green.
* **(b)** reject additive chains entirely in W5 and re-add them in W5b. This is a
  regression against the current fixtures and is not acceptable.

`*`/`-` chains (`c4_mul`, `c4_sub`, `c5_mul`) fall out of the tree path for free
once §7.2 is in place: they are left-leaning trees, their nodes are all at
distinct heights, and the cursor produces `r11, r10, r9, …` — which is what c2
emits.

### 7.4 Regression set for the change

* must become `Match`: `w5_tree2.cpp`, `w5_tree3.cpp`, `w5_chain.cpp`
* must stay `Match`: every existing `mvp_*.cpp`, `add3.cpp`
* must stay `NotImplemented`: `w5_tree_neg.cpp` (all 11 functions)
* `crates/c2-core/src/codegen.rs::select_text_rejects_tree_expression` must be
  **replaced** — `(a+b)*(c+d)` becomes a positive test asserting
  `7d632214 7d453214 7c6b51d6 4e800020` — and new rejection tests added for
  G1–G5, one per row.

---

## 8. Open / unexplained

Stated explicitly rather than guessed at:

1. **Why `+` is special.** Additive chains collapse to one accumulator register
   while `*`/`-` chains do not, the depth-2 `+` root swaps `r10`/`r11`, and the
   depth-3 `+` root updates its products in place. There is clearly an additive
   term-collection pass with its own value-numbering, but its interaction with
   the cursor is not pinned down. Everything `+`-rooted above the two measured
   forms is a hard reject.
2. **`n_imm_sum2`'s register order** (`r10` before `r11`, one `addi` in place and
   its mirror not).
3. **Level-order vs. scheduling in flattened shapes.** In shapes already rejected
   by G1/G2 (e.g. `((a+b)*(c-d)) - ((e*f)*(g+h))`) the emission order is not the
   §3 level order — `(g+h)` is emitted first. The §2 cursor rule still holds
   there byte-for-byte, so the scheduler runs *before* allocation; its ordering
   heuristic is unmodeled and out of W5 scope.
