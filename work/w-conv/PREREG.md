# w-conv — pre-registration

Lane **w-conv**, 2026-08-04, branched at master `caff20d`.

**Committed before the first `crates/` or `fixtures/` edit.** Everything below
was written from measurements taken with the real toolchain (`cl.exe`
16.00.11886.00 / `c2.dll` under wibo `1.0.1-23-g4a9dd6f`) and from the port's own
source. Nothing here is transcribed from another lane's document; where I
reproduce another lane's number I say so and say how I re-derived it.

Provenance: dc3-decomp `940d07dcb0960964ad61aa5f025658f993eb46b2` before the
measurements. Baseline reproduced in this worktree, independently of the
coordinator's figure: **match 8 / mismatch 0 / codegen-gap 0 / vocab-gap 863 /
capture-fail 7**, FRONTIER **17**, census **706,555 / 2,463,393 (28.68 %)**,
A/B/C/D **28 / 338 / 114 / 8**.

---

## 0. What this lane concluded before choosing what to build

**No FRONTIER TU is a target.** I compiled all 17 at the workload's own flags
(`work/w-frame/refobj.sh`, which reads `work/dc3-workload/flags.txt` rather than
transcribing it) and disassembled every code section
(`work/w-conv/frontier_dis.txt`, 17 objs). §1 is the hand-count. The minimum over
the 17 is **6** and the cheapest framed-and-branching one is **9**. The standing
decline clause — *a frontier TU at ≥ 4 independent refusals is not a target* —
fires on **all seventeen**.

So this lane's rung is chosen by a different key: **the refusal shared across the
most frontier TUs that is buildable on its own.** §2 identifies it, §3 prices it,
§4 registers the predictions.

---

## 1. The FRONTIER, hand-counted off the disassembly

Counting **independent** refusals, per the project rule: *if one quantity governs
several boundaries, that is one refusal.* Each row answers **"what varies between
these refusals?"** — where the answer is "nothing, it is one variable read at
different thresholds", the refusals are collapsed into one and the collapse is
stated.

### 1.1 `src/system/negate_test.cpp` — re-derived independently at **9**

w-cross published 9. I did not read its table before counting; I counted off
`work/w-conv/ref/negate_test.obj` and arrived at the same number by a route that
differs in two rows (noted). Two byte-identical 80-byte framed functions:

```text
000c  mr   r10,r3      park the scrutinee — r10, NOT r11
0010  mr   r3,r4       hoist the shared argument
0014  li   r11,0       the local `n`, live across every block
0018  cmpwi cr6,r10,1
001c  bt   24,+32  -> 0x3c        three branches, TWO targets, one shared
0020  bt   26,+28  -> 0x3c
0024  cmpwi cr6,r10,2
0028  bt   24,+12  -> 0x34
002c  bl   ?FindLast
0030  b    +8                     INTRA-SECTION b, no relocation
0034  bl   ?FindFirst
0038  mr   r11,r3     ONE copy for both arms — an in-body tail-merge
003c  mr   r3,r11
```

| # | refusal | what varies between this and its neighbours |
|---:|---|---|
| 1 | a real **label → offset map**: 4 transfers, 3 targets, two branches naming one | the *position* of a target. `cond_tail` and `guarded_seq` both compute one displacement from a fixed block count; nothing here is fixed |
| 2 | the **intra-section `b`** at 0x30 (board #191) | the *encoding chosen for a `b`*, not its displacement. #1 answers "where is the target"; this answers "which of the two same-opcode forms is this". They are separable and this lane's §2 separates them |
| 3 | the **entry-block park in r10**, not r11 | *which register a park descends to*. `plan_cond_pair` parks at r11 and only r11; `CODEGEN_W6_COMPARE.md` §6 calls the descent uncharacterized |
| 4 | a **local with a register home across every block** (`li r11,0` … `mr r11,r3` … `mr r3,r11`) | *whether a non-formal value has a home*. #3 is a formal displaced by another formal; this is a value with no incoming register at all |
| 5 | the **in-body tail-merge** — one `mr r11,r3` where both arms produce the value | *how many arms feed one destination*. A single guarded call capturing its result needs #4 and not this |
| 6 | **capturing a call's result at all** — `CallSeq` discards every result it makes | *whether r3 is read after a `bl`*. Distinct from #5: #5 is the merge, this is the capture |
| 7 | an **`enum` compare operand** with its `2c` convert | the operand's *IL type*, not its register |
| 8 | a **`float` passthrough formal** and the `_fltused` it obliges | a *TU-level symbol obligation*, not a body fact |
| 9 | **`cflow-if-n` inside a framed body** as a parse production | the *recognizer*, in a different crate from #1's layout. Both must exist; neither implies the other |

**Two rows where I differ from w-cross and the difference is recorded rather than
reconciled.** Its list carries the redundant `mr r11,r3 ; mr r3,r11` round-trip
inside its branch-bucket; I count it under #4 because one register-home decision
produces both words. It counts the empty-arm branch inversion (`bt EQ` where the
naive emission is `bne`) as its own row; I fold that into #1, because the
inversion is a consequence of which block the layout makes the fall-through.
**Net: still 9**, by a different partition, which is the outcome that makes the
number worth something.

### 1.2 The other sixteen

Read off `work/w-conv/frontier_dis.txt`. Counts are **lower bounds** — I stopped
counting each row at the point the decline clause had already fired.

| TU | independent refusals ≥ | the ones that decide it |
|---|---:|---|
| `xboxmem.cpp` | **6** | `GetXAllocAttributes`: the `!=0` spine is in class, but `lis` of `0x249b0000`, `rlwimi`, the `lis` **scheduled between** `addic` and `subfe`, and the r10 destination are four; `MemAlloc` adds a `rlwinm` inside a `CondStep` list that has only `Move` and `Li`. w-cfgimpl measured 7 with a 10-cell grid; I get 6 and defer to the higher figure |
| `xboxheap.cpp` | **unpriceable** | one function, `gap 0`, every instruction in vocabulary, and it diverges at **instruction 0** on order: six stores with `li r10,0` / `addi r11,r3,8` / `mr r31,r3` interleaved at slots 0, 2 and 5. w-pair killed six scheduling rules on it. Confirmed at the bytes; not a count |
| `Biquad.cpp` | **7** | `fdivs`; the `lis`/`lfs` REFHI/REFLO pair **straddling** the `cmplwi` (schedule); two 4-byte `.rdata` COMDATs; an intra-section `b +84`; a leaf branch with a join; five repeated `lfs/lfs/fdivs/stfs` groups; a dead `mr r10,r3` in the ctor |
| `undname.cpp` | **8** | two saved GPRs (`std 30`/`std 31`, 112-byte frame); two callee-saved formals; two REFHI/REFLO data pairs; `cmplwi cr0,r3,0` on a **call result** (board X-e); four stores into the returned pointer; `stb`; a shared-target fixup; `li r10,-1` scheduled between them |
| `vsnprnc.cpp` / `vswprnc.cpp` | **8** each | three chained guards **plus** a callee-saved r31; a REFHI/REFLO pair used as a *data pointer argument*; a 6–8 formal descending shuffle; `cmpwi cr0,r3,0` after the `bl` then `bf 0` on the **LT** bit; `cmpwi cr6,r3,-2`; two `bl _errno` sites converging on one `stw`; `stb`/`sth`; an intra-section `b` |
| `xlrcimpl.cpp` | **7** | the `__savegprlr_26`/`__restgprlr_26` frame (a whole frame variant, entered by `bl` and left by `b`); a stack local at `80(1)` whose address is taken; `mr. r31,r3` branching on cr0; `lis`/`ori` wide constants; three intra-section `b`s with shared targets; four callee-saved formals |
| `mmio.cpp` | **7** | `mtctr`/`bctrl` (an indirect call); an inlined `memcpy` with `li r5,72`; `cmplw` register-register; callee-saved r31; the entry-block park `mr r11,r3 ; mr r3,r4`; `cmplwi cr0` after a `bl`; two shared-target `b`s per function |
| `osfinfo.cpp` | **8** | two REFHI/REFLO pairs; `srawi`, `mulli`, `slwi`, `lwzx`, `lbz`; `clrlwi.` branching on cr0; `cmpwi cr6,r10,-1`; an intra-section `b`; two `bl` sites; a shared-target fixup |
| `Main.cpp` | **6** | the two-word `__CxxFrameHandler`/`__ehfuncinfo$main` prefix **inside `.text`** with ADDR32 relocations; two `.pdata`; a 64-byte EH `.rdata` group with five relocations; a **funclet** with its own prologue; the `addi r31,r1,-112` frame-pointer form |
| `Sort.cpp` | **7** | a cr0-tested loop with a back edge; `divw`, `mullw`, `mulli`, `rotlwi`, `andc`, `twi` ×2, `lbzu`, `mr.` |
| `Pool.cpp` | **7** | the ctor is a CTR loop (`mtctr`/`bdnz`) with `divw`, `rotlwi`, `twi` ×2; `Alloc`/`Free` are fold-band-2 `bclr` bodies the port has never emitted |
| `IPP_basicmath_xbox.cpp` | **6** | four leaf CTR loops; `bclr` guards; `lfsx`, `stfsx`, `stfsu`, `fadds`, `fmuls`; a `sub`-based index base |
| `jsonwriter.cpp` | **≥ 10** | `__savegprlr_28`; 15 branches over 304 bytes; `cmplw`; `lhzx`, `sth`, `sthu`; `rlwimi` ×5; `lis`/`ori` wide constants; a loop |
| `wordwrap.cpp` | **≥ 12** | 640 bytes, `__savegprlr_29`, `.bss`, two REFHI/REFLO globals, `lhzx`/`lbzx`, `srawi`/`addze`, dozens of shared-target branches, two loops, a recursive-ish call chain |
| `EncryptXTEA.cpp` | **≥ 12** | 64-bit (`std`, `ld`, `stdu`, `stdx`, `rldicl`, `rldimi`, `clrldi`); a CTR loop; `__savegprlr_26`; a tail `b memcpy`; `addic.` on cr0; `addis`+`addi` constant pairs |

**Minimum over the 17: 6.** No row is under the decline clause's 4.

### 1.3 What that says about `work/w-frame/RANKING.md`

Its §5 says the key *"excludes correctly and does not rank the head"*, and this
hand-count is the confirmation it asked for: its three cheapest TUs
(`xboxheap` 0, `xboxmem` 1, `Biquad` 1) price at unpriceable, 6 and 7. **The
ranking is right about everything it excludes and its head is not a head.** It
should be read as a *floor*, never as an ordering.

---

## 2. What this lane builds instead, and why that one

Ranking the frontier by **construct** rather than by TU — the framing the brief
asks for — over all 17 objs:

| missing mechanism | FRONTIER TUs that need it |
|---|---:|
| **a real label → offset map (≥ 2 transfers, ≥ 1 shared target)** | **14** |
| **the intra-section unconditional `b`** (board #191) | **10** |
| a branch on **cr0** (record-form or call-result compare) | 10 |
| callee-saved GPR formals / `savegprlr` | 9 |
| a REFHI/REFLO data-symbol pair | 6 |
| `cmplw` / `cmpw` register-register | 5 |
| a CTR loop | 4 |

The top two are the same rung and **nothing in the port has ever emitted either
of them**. Board #191 has been open since w-cfg, and w-cross closed the only
route to it that had been tried — the `else` arm — because *"the only source
shape that produces one also produces `/Ox`'s duplication"*, with a threshold
bracketed by one cell either side and declined as a c2 cost model.

**There is a second route and it is not that shape.** A **guarded early return**
in a framed call sequence:

```cpp
int f(int a) { if (a) return 5; v0(); return 0; }
```

produces an intra-section `b` whose target is the **epilogue**, not a join block:

```text
/O1  (the workload's own mode)            /Ox and /O2
  mflr/stw/stwu    Class A frame            mflr/stw/stwu
  cmpwi cr6,r3,0                            cmpwi cr6,r3,0
  bt    26,+12                              bt    26,+24
  li    r3,5                                li    r3,5
  b     +12        <- 48000...              addi/lwz/mtlr/blr   <- epilogue DUPLICATED
  bl    ?v0                                 bl    ?v0
  li    r3,0                                li    r3,0
  addi/lwz/mtlr/blr                         addi/lwz/mtlr/blr
```

**The mode split is real and it is X-b's, but it is not a cost model here.**
w-cross's threshold was over *how many bytes the join is worth duplicating* and
varied with join length. Here the duplicated block is the **epilogue**, whose
length is a constant of the frame class, and `/Ox` duplicates it in **every**
measured cell (guard counts 1, 2, 3; both signednesses; six relations; trailing
call counts 1–4; scrutinee at formals 0/1/2 — `work/w-conv/p/probe1.cpp`,
`probe2.cpp`, at `/O1`, `/O2` and `/Ox`). There is no quantity to fit; there are
two measured layouts, one per mode, with ≥ 8 witnesses each.

**This is not a conversion claim.** §1 already says no frontier TU converts. It
is the mechanism 14 of them need, given its first oracle witness.

---

## 3. The refusals this rung closes, priced by the same rule

The estimate rule applies to me too: the ceiling is the estimate, no discount,
and "independent" is load-bearing.

| # | refusal | what varies between this and its neighbours |
|---:|---|---|
| 1 | the **IL production** — an `if` whose then-clause is `RESULT k ; 3A epilogue` (or a bare `3A epilogue`), repeated N times ahead of the shipped call sequence | *which bytes are read*. A parser fact, in `c2-il`, with no emitter content |
| 2 | the **intra-section `b`** — true displacement, **no relocation** (board #191) | *which of two same-opcode encodings*. `encode_b_intra` was written by w-cross and **deleted** rather than shipped ungraded; it comes back here with a byte compare behind it |
| 3 | a **label → offset map** — N branches over up to N+1 targets, where the epilogue's offset is not known until the whole sequence is laid out | *where a target is*, not what a branch is. W8 and W10 each computed one displacement from a fixed block count and both said in their own doc that the first variable-layout shape would need this |
| 4 | **two block layouts from one recognizer**, selected by `OptMode` | *which blocks exist*. Orthogonal to 1–3: the same targets, the same encodings, a different set of blocks |
| 5 | the **exit-merge refusal** — any two exits producing the same value | a *class boundary*, and the one that keeps the other four honest |

**Five, and I checked 2 against 3 specifically** because they are the pair most
likely to be one variable at two thresholds. They are not: w-cross computed a
displacement (3's job) without ever choosing an encoding (2), because its only
`b` was an external tail call carrying a `REL24`; and board #191 is registered
precisely as *"choosing the encoding is distinct from computing the
displacement"*. Two quantities.

### 3.1 The exit-merge refusal, measured — and it moves the label counter

The cell that forces #5 is `m2` (`if(a) return 5; if(b) return 5; v0(); return
0;`). c2 does **not** emit a second arm; it branches **backwards into the first**
and inverts the sense:

```text
  0018  48000014  b   +20        first arm -> epilogue
  001c  2f040000  cmpwi cr6,r4,0
  0020  409afff4  bf  26,-12     <- BACKWARD, into the first arm, sense inverted
```

and `m0` (`if(a) return 0; v0(); return 0;` — a guard whose literal equals the
final one) deletes the arm entirely and branches over the call to the shared
`li r3,0`. Same variable in both: *do two exits produce the same value?* **One
refusal, two witnesses**, and it holds at `/Ox` as well as `/O1`.

**It is also the label-counter boundary, which is the part worth registering.**
Measured over 16 functions in one TU (`work/w-conv/p/p2_o1.txt`, leads computed
as `first($M) − last($T of the previous function) − 1`):

```text
  g1 g2 g3 t2 t4 p3 rv rn r_ne r_lt r_ge r_eq ac c0   lead 2   (1,2,3 guards; void; int)
  m2 m0                                               lead 3   <- the merged cells
```

So every cell **in** the accepted class has the **same** lead and the same 5-slot
stride as the shipped guard-free `c0`, and the two cells the class refuses are
the two that would have cost a sixth slot. A lane that admitted the merge without
noticing emits six wrong bytes per function in the symbol table.

---

## 4. Predictions — registered, with rivals

Bias, stated first: **I want this to convert a TU.** It is the payoff metric, five
lanes have converted zero, and the temptation is to price the frontier optimistically
and call a near-miss progress. Every prediction below is registered *against* that
preference, and the estimate is the ceiling taken neat.

### 4.1 The estimate

> **Point estimate: TU match = 8. Interval: [8, 8]. Unit: TU match, of 878.**

**The decline clause keys on the point estimate.** The interval is degenerate on
purpose and that is a claim, not a hedge: §1 prices every one of the 17 at ≥ 6
independent refusals, this rung closes 5, and none of the 5 is the *last* one for
any TU. For the interval to be wrong, some frontier TU would have to be at ≤ 5 and
have all of them inside this rung — `mmio.cpp` is the nearest and it additionally
wants the entry-block park, an inlined `memcpy`, `mtctr`/`bctrl`, `cmplw` and
callee-saved r31.

**Bias direction: my estimate is biased HIGH if anywhere.** A degenerate interval
at the incumbent cannot be biased low; the failure mode available to it is that I
have talked myself out of a conversion that was reachable. §1 is the evidence
against that and it is on the page in full so it can be checked.

### 4.2 The scored predictions

| # | prediction | registered rival |
|---:|---|---|
| **C1** | **TU match = 8**, mismatch **0**, `codegen-gap` **0**, `capture-fail` **7**, FRONTIER **17** | — |
| **C2** | **census delta > 0** — unlike W10's guarded call, the guarded early return is an ordinary shape and the workload should contain instances | **R-C2: delta = 0**, because a body with early returns almost always also has a callee-saved formal or a non-literal return, and 863 of 878 TUs never reach the emitter at all |
| **C3** | **≥ 1 new cell does not come out `Port=Match` first time** — taken neat. Two block layouts and a displacement that is not a function of a fixed block count | **R-C3: all cells match first time**, as happened to w-frame (E3) and to W10's one-armed family |
| **C4** | **label lead 2, stride 5, no `crates/c2-core/src/coff/` edit** — measured in §3.1 *before* any code was written | **R-C4**: a guard mints a slot and the lead is `2 + N` |
| **C5** | **`scripts/gate.sh` 12/12 with 0 mismatch**, including both `/Od` lanes, where the class must refuse rather than emit | — |
| **C6** | the **`#[test]` count rises**, `git grep -c` measured at merge-base **and** tip and reconciled against the runner | — |
| **C7** | **`work/w-frame/sweep.py` reports no never-executed EMISSION line that this rung adds**, run with `C2RS_SWEEP_KEEP` covering every module I touch (F-c, and Y-f's correction to it) | **R-C7**: it fires, as it did inside w-cross's own rung |
| **C8** | **`m2`/`m0`'s shape is refused, not emitted**, at every lane — and the fixture that says so censuses **0** | — |

### 4.3 Decline clauses

1. **A frontier TU at ≥ 4 independent refusals is not a target.** Fires on all 17
   (§1). Recorded as *already fired*: this rung is not aimed at a TU.
2. **A layout or allocation decision with fewer than three oracle witnesses is
   refused by name, never fitted.** Fires on: the entry-block park (three cells,
   zero tests — W10's boundary, inherited), the exit merge (two cells), a guard
   placed after a call (one cell, `probe1 e7`, which folds branchlessly into r31),
   an arm containing a call (`ac` — one cell each mode).
3. **F-c**: a code path this rung adds with no coverage under the GRADED profile
   is a first witness and must say so in the rung doc.
4. **If the byte compare disagrees with §2's two layouts on any cell, the rung
   ships the refusal and not a third layout.** A wrong emit is strictly worse than
   a refusal, and #232 is the precedent.

---

## 5. What this lane will not do

* **No cost model** — neither #187's fold bands nor X-b's duplication threshold.
  The two layouts here are measured per mode, not fitted to a size.
* **No entry-block park, no callee-saved allocation, no r10 descent.** W10 refused
  them with three cells and nothing has been added since.
* **No `coff/` edit** (C4), and none is expected — but the prediction is
  registered rather than assumed, because the alternative is six wrong bytes.
* **No `BOARD.md` / `STATUS.md` / `ROADMAP.md` edit.** Lettered rows proposed in
  the rung doc; T, U, V, X, Y are taken.
* **No attempt on any frontier TU.** All 17 read, all 17 priced, all 17 declined
  in §1.
