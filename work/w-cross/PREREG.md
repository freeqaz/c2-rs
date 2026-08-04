# Pre-registered estimate — lane w-cross

    Committed BEFORE the implementation. Nothing under `fixtures/` or `crates/`
    has been touched at the time of this commit. Frozen; scored verbatim in the
    rung doc.

Lane **w-cross**, branched at master `ed99bdf`. Charter: build the **framed ×
branching** cell that `work/w-frame/RANKING.md` §4 measured as the port's one
un-witnessed cross-product cell — *105 functions emitted byte-exact, 28 framed,
2 branching, **zero both***, with 10 of the 17 FRONTIER TUs needing the product
— and convert `src/system/negate_test.cpp` **if the estimate supports it**.

Provenance for everything below: `cl.exe` 16.00.11886.00 / `c2.dll` under wibo
**1.0.1-23-g4a9dd6f** at the workload's own flags (`work/dc3-workload/flags.txt`,
cwd `../dc3-decomp`), never `c2rs compile` (board #195). dc3-decomp HEAD
`940d07dcb0960964ad61aa5f025658f993eb46b2` at the start of the measurement.
Probes: `work/w-cross/p/probe{1,2,3}.cpp` via `work/w-cross/p/mk.sh`.

---

## 0. Three corrections to the brief and to w-frame, before the count

These change the count, so they come first.

1. **A framed body with two `bl` sites is already shipped.** w-frame's refusal
   **4** reads *"two `bl` sites in one body — board #35, still PARTIAL and
   explicitly blocked on '>1 call per body'; `CallSeq` admits one"*. It does
   not. `c2_il::CallSeq` carries `calls: Vec<SeqCall>` with *"at least two, or
   one with a non-void tail"*, `call_seq_text` emits one REL24 site per call,
   and `fixtures/cpp/mvp_call_seq.cpp` ships `void two(){ v0(); v1(); }` and
   `void four(){ v0(); v1(); v0(); v1(); }` as graded 36 B / 44 B objs.
   **Refusal 4 is not a refusal.**

2. **The cheapest framed-and-branching function is not in `negate_test.cpp`.**
   `void f(int a){ if (a != 0) g(); h(); }` compiles to 44 B —
   `mflr/stw/stwu · cmpwi cr6,r3,0 · bt 26,+8 · bl g · bl h · addi/lwz/mtlr/blr`
   — which is `mvp_call_seq.cpp`'s `two()` with a compare and a branch inserted.
   That is the cross-product cell at **one** refusal beyond code that ships,
   and it is the shape this lane builds.

3. **w-frame's refusal 1 — "a framed body containing basic blocks" — is a
   bucket, not a refusal.** Read against the disassembly it is at least four
   independent facts (§1 below, rows 1, 3, 6, 7). w-frame's own §2.2 says its
   key "does not rank the head"; this is the same failure one level down, in a
   hand-count rather than in a classifier. **The five must not be inherited.**

---

## 1. `negate_test.cpp`, re-derived off the disassembly

`?FindNodeA@@YAPBUCharGraphNode@@W4PlayBlend@@PAXM@Z`, 80 B, and
`?FindNodeB` byte-identical to it:

```text
0000  7d8802a6  mflr  r12
0004  9181fff8  stw   r12,-8(r1)
0008  9421ffa0  stwu  r1,-96(r1)
000c  7c6a1b78  mr    r10,r3        <- blendMode PARKED IN r10
0010  7c832378  mr    r3,r4         <- clip hoisted to r3, above the branch
0014  39600000  li    r11,0         <- the local `n`, in r11
0018  2f0a0001  cmpwi cr6,r10,1
001c  41980020  bt    24,+32        -> 0x3c
0020  419a001c  bt    26,+28        -> 0x3c   <- INVERTED (see row 6)
0024  2f0a0002  cmpwi cr6,r10,2
0028  4198000c  bt    24,+12        -> 0x34
002c  4bffffd5  bl    ?FindLast     REL24
0030  48000008  b     +8            -> 0x38   <- intra-section, NO relocation
0034  4bffffcd  bl    ?FindFirst    REL24
0038  7c6b1b78  mr    r11,r3        <- ONE copy, shared by both arms
003c  7d635b78  mr    r3,r11
0040  38210060  addi  r1,r1,96
0044  8181fff8  lwz   r12,-8(r1)
0048  7d8803a6  mtlr  r12
004c  4e800020  blr
```

Counting **independent** refusals, per the project's rule — *"if one quantity
governs several boundaries, that is one refusal"* — with the question **"what
varies between these?"** answered on every row:

| # | refusal | what varies, i.e. why it is its own row |
|---|---|---|
| 1 | a **frame containing a block boundary** at all | the presence of a branch inside a frame. Separable: §2's cell A has this and nothing else on this list |
| 2 | the **intra-section unconditional `b`** (`48000008` @0x30) — true displacement, **no** relocation (board **#191**) | the branch's *target kind*. The port's only `b` today is a tail call with a section-start placeholder + REL24. Separable: cell A has row 1 without row 2 |
| 3 | **three distinct targets, two branches naming the same one**, resolved only after layout | the number of labels and whether one is named twice. Rows 1 and 2 are satisfiable by *displacement arithmetic* (`cond_tail.rs` does exactly that today, and its module doc says why it needs no fixup list); this row is what forces a real label→offset map. Separable: cells A and B have 1 and 2 targets and no shared label |
| 4 | a **literal-initialised local living in r11** across the branch, with `mr r11,r3` after the call and `mr r3,r11` at the join | which register holds a *local*. Separable: probe2 `u0`/`u1` and probe3 `P1` have this with no park |
| 5 | the scrutinee **parked in r10**, because r11 is taken by row 4's local | which register a *park* descends to. Separable the other way: probe2 `s4` parks in **r11** with no local. Two quantities, two rows |
| 6 | the **empty-arm inversion** at 0x20 | whether a block emits any instruction, and the branch sense as a function of that. The IL is `Rel::Ne` consumed by `39` (brTRUE) → the naive emission is `bne` to the else entry; `n = 0` is dead (r11 already 0), the then-block empties to a bare jump, and c2 inverts to `bt EQ` straight to the join. Separable from every row above |
| 7 | the **tail-merge** of the two arms' identical `mr r11,r3` into one instruction at 0x38 | whether identical trailing runs in converging blocks are merged. Board **#193**'s shape, here *inside* a body rather than across two tail calls. Separable from row 6: row 6 is a branch sense, row 7 is instruction identity |
| 8 | the compare operand is an **`enum`** (`86 41 84 20`, with a `2c` convert to `int`) | the operand's declared type. `eat_cmp_operand_type` admits `int`/`long`/`unsigned`/`unsigned long`/pointer-to-4 and nothing else, deliberately, because the spelling is what says *signed* |
| 9 | a **`float` formal** passed through to both callees in `fp1` with no instruction, plus the `_fltused` symbol the TU is thereby obliged to declare | the register *file* of a passthrough formal. Every argument the guarded-call class marshals is a GPR |

**Nine independent refusals. Ceiling taken neat, no discount** — the rule that
fixed this project's estimate series after eight consecutive misses, and every
discount ever applied here has been wrong across five confirmations.

w-frame counted five and this lane counts nine. The delta is: **−1** (its row 4
is already shipped, §0.1), **−1** (its row 2, closed by W9), **+5** (its row 1
was a bucket covering rows 1, 3, 6, 7 here, and rows 8 and 9 are new). The
direction is the one that matters: **a hand-count off a disassembly went UP, not
down, when it was re-done against the code rather than against the byte listing
alone.**

### 1.1 The other frames × blocks frontier TUs are further, checked not assumed

Read at the workload's flags, same session:

| TU | why it is not cheaper than `negate_test.cpp` |
|---|---|
| `undname.cpp` (gap **1**, 1 blocked fn) | 140 B; `std r30`/`std r31` + a 112 B frame, **two REFHI/REFLO data-symbol pairs**, five stores at computed displacements, and a `cmplwi` into **cr0** |
| `vswprnc.cpp` (gap 2) | 156 B; three chained guards, a REFHI/REFLO pair, an 8-formal descending shuffle, a `cmpwi` into **cr0**, `sth`, and a 3-way join |
| `vsnprnc.cpp` (gap 2) | 152 B; the same body plus a second function |

All three are ≥ 6 independent facts by inspection. `negate_test.cpp` remains the
cheapest, exactly as w-frame reported; it is simply not cheap.

### 1.2 A new finding that removes `cr6` from the "settled" list

`docs/CFG_SHAPE.md` §3.2 states *"an explicit compare feeding a branch writes
**cr6**"*, with a record-form `addic.` writing cr0 as the only exception. **Three
frontier TUs refute the rule as stated**: `undname` @0x38 `28030000 cmplwi
cr0,r3,0`, `vswprnc` @0x50 and `vsnprnc` @0x4c `2c030000 cmpwi cr0,r3,0`. All
three are an **explicit compare of a call's result in r3, immediately after the
`bl`**, and each of the same bodies later compares r3 into **cr6**
(`2b030000`, `2f03fffe`). The discriminator is unmeasured. Recorded, not
modelled: the accepted class below compares a **formal**, never a call result,
so it cannot reach the cell — and an emitter that hard-codes cr6 for a
result-compare emits `409a…` where the obj has `4082…`, the exact
fuzzy-invisible shape §3.2 warns about.

---

## 2. What IS built — the class, and its ladder

**W10 — a guarded call in a framed call sequence.** `CallSeq` is the shipped
Class A/B many-call body; this rung admits one of its calls being **guarded by a
conditional branch**, with an optional else-arm.

| cell | source shape | what it adds over shipped code | probe witness |
|---|---|---|---|
| **A** | `if (x rel k) g(…); h(…);` | row 1 only: a `cmp`+`bc` between the prologue and the sequence, and a displacement over one block | `probe2 s0 s1 s3`, `probe3 L1` |
| **B** | `if (x rel k) g(…); else h(…); j(…);` | row 2: the **intra-section `b`** with a true displacement and no relocation (board #191) | `probe2 t0`, `probe3 L2` |

Rows 3–9 are **declined by name** and must census as gaps:

* row 3 (a shared label / a real fixup list) — cells A and B have 1 and 2
  targets, each named once, so both are displacement arithmetic;
* row 4 (the r11 local) — `probe2 u0`/`u1`, `probe3 P1`, refused;
* row 5 (the entry-block park) — `probe2 s4`, `probe3 P0`/`P2`, refused. **This
  is decline clause 2 firing**: the hoisting rule that puts `mr r3,r4` in the
  entry block for `s4`/`P2` and leaves `mr r4,r5` in the arm for `P0` fits three
  cells and is *tested by* none of them;
* rows 6 and 7 (the empty-arm inversion, the tail-merge) — `probe2 t1`/`u2`
  are the negative controls, and they carry a **third** unmodelled fact this
  lane found and is not building: **c2 propagates the branch condition's value
  range.** `void t1(int a,int b){ if(a!=0) a1(b); else a1(a); v1(); }` emits
  `li r3,0` for `a` on the else path, because the branch proved `a == 0`. Not
  recorded anywhere in `docs/` today;
* rows 8 and 9 (the `enum` operand, the FP passthrough formal) — not touched.

### 2.1 The label counter, measured before the code was written

`coff.rs` holds the compiler-label counter and both six-wrong-bytes defects came
from it, so the stride was measured rather than assumed. `probe3` holds the
whole body shape fixed and varies **only** the branch count:

| function | branch targets | first `$M` | stride to the next |
|---|---:|---:|---:|
| `L0` `void L0(){v0();v1();}` | 0 | 2640 | **5** |
| `L1` `if(a)v0(); v1();` | 1 | 2645 | **5** |
| `L2` `if(a)v0();else v1(); v2();` | 2 | 2650 | **5** |
| `L3` two if/elses | 4 | 2655 | **5** |
| `L4` = `L0` again | 0 | 2660 | **5** |

**The stride does not track branch targets**, which is `docs/CFG_SHAPE.md`
§3.6's `$M`-count rule holding for the *counter* as well as for the symbol
count. So this rung's `label_lead` is **0** and it needs no `coff.rs` edit —
registered as prediction **X4** rather than assumed, because the alternative is
six wrong bytes per label.

---

## 3. The estimate

**The unit.** The payoff metric is **TU match out of 878** — whole objs
byte-exact at the workload's own flags. This change quantifies over **function
shapes admitted**, not over TUs, and a TU converts only when *every* function in
it converts. So the TU-level number and the shape-level number are registered
**separately**, and the decline clause keys on the TU-level one. (A lane this
session scored 3 of 4 and its one miss was wrong in *kind* — per-function
quantities registered for a per-TU gate change, both actuals necessarily 0.)

**Point estimate: TU match = 8. Interval: [8, 8].** The decline clause keys on
the **point estimate**; the interval is degenerate on purpose and is not a hedge
— §1 counts nine independent refusals for the cheapest frontier TU and this rung
ships two of them, so there is no arithmetic under which a TU converts.

**Bias, in writing.** I want the cross-product cell to convert something, and I
want `negate_test.cpp` specifically because the brief names it and because a
lane that converts a frontier TU would be the first in five. That bias pushes
the estimate **up**. Every number registered here is therefore against my own
preference, and the one prediction I would most like to lose is **X3**.

### 3.1 Predictions, each with a named rival

| # | prediction | rival |
|---|---|---|
| **X1** | **TU match = 8.** Zero frontier TUs converted. | X1 fails at 9+ — some frontier TU's remaining blockers were fewer than the disassembly says |
| **X2** | **Census delta > 0** on the 878-TU workload. Unlike W9 this rung genuinely widens the accepted class, so some workload function should change class even though no whole TU does | R-X2: the delta is **0**, i.e. the guarded-call shape is a fixture artefact with no workload instance at all — which would make cells A and B worth exactly one witness each and nothing else |
| **X3** | **At least one new fixture cell does NOT come out `Port=Match` on the first differential run.** Ceiling neat: the block layout inside a frame, the branch displacement over a `bl`, and the intra-section `b` encoding are all first witnesses | R-X3: every cell passes first time and `docs/CFG_SHAPE.md` §3.1/§3.3 transfers whole for the third consecutive rung. **I want R-X3** |
| **X4** | **The compiler-label counter stride is 5** for a branching framed body — `label_lead` 0, no `coff.rs` edit | R-X4: the stride tracks branch targets and this rung is six wrong bytes per label. Measured in §2.1 *before* the code; registered so the measurement is scored rather than trusted |
| **X5** | **mismatch 0** in the 878-TU scan and in all 12 `gate.sh` lanes. An alarm, not a metric | — |
| **X6** | **`codegen-gap` stays 0** — census/gate agreement holds across the widening, i.e. every body the IL parser newly accepts, `select_function` also emits | R-X6: the gate refuses something the census claims, which is `docs/GAPS.md` §6's one-fact-two-implementations drift |
| **X7** | The `#[test]` count rises. A rung that adds a code path and no portable assertion is board #137's shape | — |

### 3.2 Decline clauses, stated before the numbers

1. **A frontier TU whose re-derived independent-refusal count is ≥ 4 is not a
   target** — write the measurement instead of building it. Fires on
   `negate_test.cpp` at **9**, on `undname`/`vswprnc`/`vsnprnc` at ≥ 6.
2. **An allocation or schedule decision with fewer than three witnesses is
   refused, never fitted.** Fires on §1 rows 5, 6 and 7, and on the entry-block
   hoisting rule of §2 (three cells, zero tests).
3. **A code path this rung adds that has no coverage under `sweep.py`'s GRADED
   profile is a first witness and must say so** (w-frame row **F-c**, adopted
   here as a standing rule). If a path cannot be graded, the class narrows until
   it can.

### 3.3 What a failing cell means

If **X3** comes true the response is *not* to fit the layout to the new byte. It
is to name the quantity the rule got wrong and, if that cannot be settled from
witnesses in hand, to **narrow the accepted class until the unwitnessed case is
refused** — exactly as `plan_cond_pair` already refuses a schedule its rules
cannot deliver. A shape the rules mis-handle must come out as a **gap**, never
as a plausible-looking wrong branch.

---

## 4. Baselines, measured on master `ed99bdf`

| | value |
|---|---|
| `git grep -c '#[test]'` over `crates/` | **691** |
| dc3-decomp HEAD before | `940d07dcb0960964ad61aa5f025658f993eb46b2` |
| wibo | `1.0.1-23-g4a9dd6f` |
| TU match / mismatch / codegen-gap / vocab-gap / capture-fail | 8 / 0 / 0 / 863 / 7 (from `docs/STATUS.md`, to be re-measured on both refs) |
