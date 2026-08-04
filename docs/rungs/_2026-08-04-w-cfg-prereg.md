# Pre-registration — lane `w-cfg`, the CFG step's byte-level spec (2026-08-04)

Written and **committed before the first capture**. Scored verbatim in
`docs/CFG_SHAPE.md`; wrong predictions stay on the page, with the rival reading
that beat them.

## What I am measuring

**What `c2` emits for a body with control flow**, at the workload's own flags
(`work/dc3-workload/flags.txt`, i.e. `/O1 /Oi /EHsc /GS- /GR /c` plus includes) —
the PPC branch forms, the condition-register discipline, the branch-target
encoding, and **the order in which blocks land in `.text`**.

The IL half (`38`/`39`/`3A`/`29` and the statement grammar) is already
characterized in `docs/IL_STMT_GRAMMAR.md` §7–§9 and implemented as a decode-only
scanner in `crates/c2-il/src/func/body/shapes/control_flow.rs`. This lane
**confirms** that half against fresh captures and **adds** the emission half,
which nothing in `docs/` currently states at all.

Read-only lane: **no file under `crates/` is touched.** Reference obj + `.cod`
listing + `.ex` IL only.

## My bias, in writing

**I want the answer to be "c2 emits basic blocks in the IL's own order, with the
branch sense inverted at `38`, and the target as a plain self-relative
displacement with no relocation."** That result makes the CFG step a
straightforward extension of the existing emitter: walk the statement stream,
emit, patch displacements at the end.

The failure mode that points at is **fitting the rule to leaf probes**. `docs/
CODEGEN_W6_COMPARE.md` §7 already records one capture where `if (a > 7) return
5; return 9;` emits `cmpwi cr6,r3,7 ; li r3,5 ; bclr 12,25` — a *conditional
return*, with **no label and no displacement anywhere**. If the whole `if-1`
probe family folds like that, I will have "specified" a CFG lowering that the
frontier's real functions never take, and the first implementer to hit a real
`if` with a call in it will find nothing in the document.

Guards, registered in advance:

1. Every claim about block order must be made on a probe **with a call in at
   least one arm**, because a call is what makes a block un-foldable. Leaf
   probes are recorded but never generalized from.
2. Every claim must be checked against **at least one real frontier function**
   (`xboxmem.cpp`, `Pool.cpp`, `mmio.cpp`) and not only against my probes.
3. The honest outcome "block order is decided by a scheduler whose input is not
   the IL's order, and I cannot state the rule" is registered here **in advance
   as a good result**, not a lane failure. See the decline floor below.

## The probe grid (written before compiling; sources in `work/w-cfg/p/`)

Three TUs, 19 functions, all at the workload flags.

**`pa.cpp` — `if-1` leaves, no calls.** Tests whether the shape folds.

```cpp
int  a_eq   (int a,int b){ if(a==b) return 1; return 2; }
int  a_ne   (int a,int b){ if(a!=b) return 1; return 2; }
int  a_lt   (int a,int b){ if(a<b)  return 1; return 2; }
int  a_eqk  (int a)      { if(a==7) return 1; return 2; }
int  a_else (int a,int b){ if(a==b) return 1; else return 2; }
int  a_var  (int a,int b){ int r=2; if(a==b) r=1; return r; }
void a_store(int a,int*p){ if(a==0) *p=1; }
```

**`pb.cpp` — control flow around CALLS.** The load-bearing TU: a call cannot be
folded into a conditional-return idiom, so these are the probes that must show a
real block boundary if one exists.

```cpp
void b_if     (int a){ if(a) g(); }
void b_ifelse (int a){ if(a) g(); else h(); }
int  b_ifval  (int a){ if(a) return gi(1); return gi(2); }
void b_and    (int a,int b){ if(a && b) g(); }
void b_or     (int a,int b){ if(a || b) g(); }
void b_if2    (int a,int b){ if(a) g(); if(b) h(); }
void b_ifn    (int a,int b,int c){ if(a) g(); if(b) h(); if(c) g(); }
```

**`pc.cpp` — loops.**

```cpp
int  c_while   (int n){ int s=0; while(n){ s=s+n; n=n-1; } return s; }
int  c_for     (int n){ int s=0; for(int i=0;i<n;i=i+1) s=s+i; return s; }
int  c_do      (int n){ int s=0; do { s=s+n; n=n-1; } while(n); return s; }
void c_callloop(int n){ while(n){ g(); n=n-1; } }
int  c_forcall (int n){ int s=0; for(int i=0;i<n;i=i+1) s=s+gi(i); return s; }
```

Held-out cells, compiled only **after** the rules below are frozen: the four
`xboxmem.cpp` functions and the three `Pool.cpp` functions from the real
workload.

## Registered predictions

### A — the branch instruction and the condition register

| # | prediction | rival registered | confidence |
|---|---|---|---|
| A1 | Integer compares feeding a branch use **`cr6`**, always, even with several compares live in one function. `cmpw cr6,r3,r4` = `7f 03 20 00`; `cmpwi cr6,r3,7` = `2f 03 00 07` | **R-A1:** c2 allocates CR fields (cr0/cr6/cr7 in turn) when two compares are live, as it allocates GPR temps descending from r11 | medium-high on the single-compare case, **low** on the multi-compare case |
| A2 | A conditional branch is `bc` (op 16) with `AA=0, LK=0`, `BO ∈ {4,12}` (branch-if-CR-bit-false / -true) and `BI = 4*crf + {0=LT,1=GT,2=EQ,3=SO}`, i.e. `BI ∈ {24,25,26,27}` for cr6. No `BO` with the decrement-CTR bits set | — | high |
| A3 | **The branch sense is the negation of the IL relation at `38` (brFALSE) and the relation itself at `39` (brTRUE).** `a==b` + `38 L` → `bne cr6,L` = `40 9a <BD>`; `a==b` + `39 L` → `beq cr6,L` = `41 9a <BD>` | **R-A3:** c2 normalizes to one sense and swaps the successor blocks instead, so `38` and `39` over the same relation emit the *same* branch word and different block orders | medium-high |
| A4 | `cmpwi`/`cmplwi` when the rhs is a literal fitting a signed 16-bit field; `cmpw`/`cmplw` for a register rhs; signedness from the operand type triple exactly as `docs/CODEGEN_W6_COMPARE.md` §1.1 established | — | high |
| A5 | The comparison **is fused into the branch** — there is no branchless 0/1 materialization (`cntlzw`/`subfe`/`rlwinm` spine) anywhere in a body whose comparison feeds a `38`/`39`. The W6 spines appear only when the comparison's value is *used* | **R-A5:** c2 materializes the bool and then tests it with `cmpwi cr6,rt,0` | medium-high |

### B — targets, fixup, relocations

| # | prediction | rival registered | confidence |
|---|---|---|---|
| B1 | An intra-function branch target is a **plain self-relative displacement** in the instruction word: `bc` carries `(target - addr) & 0xFFFC` in bits 16..31 as a signed 14-bit `BD<<2`; `b` carries `(target - addr) & 0x03FFFFFC`. **No relocation record is emitted for it** | **R-B1:** intra-section branches carry a relocation the way the `??__E` thunk's `b` to an external does (`docs/OBJ_DYNINIT_SHAPE.md` §3.3), i.e. the word stores a section-start-relative value and a REL24 fixes it up | high on `bc`, medium on `b` |
| B2 | Consequently `.text` relocation count for `pa.cpp` (7 leaf functions, no calls) is **0**, and for `pb.cpp` it is exactly one REL24 per emitted call site | — | high |
| B3 | Every branch in these probes fits the 14-bit `BD` field; no `bc`-over-`b` inversion ("long branch") is needed or emitted | — | high |
| B4 | Labels produce **no symbol-table records**. `$M`/`$T` label symbols appear per `docs/LABEL_COUNTER.md` for *framed* functions only, and their count does **not** move with the number of basic blocks | **R-B4:** each emitted label mints a `$L`-family symbol, so the label counter is a function of block count | medium-high |

### C — block order (the load-bearing section)

| # | prediction | rival registered | confidence |
|---|---|---|---|
| C1 | **Blocks land in `.text` in the order their statements appear in the `.ex` stream.** For `if`/`else` that is: condition, `bc` to the else entry, then-block, `b` to the join, else-block, join. `b_ifelse` therefore emits `g` before `h` | **R-C1:** c2 reorders — e.g. hoists the un-taken arm below the epilogue, or inverts so the *shorter* arm falls through — and IL order is not recoverable from `.text` order | medium |
| C2 | **The trailing `b` to the join is elided when the join immediately follows** (fall-through), and the epilogue-label `3A` of a `return` at the end of a body emits no branch at all | — | high |
| C3 | For `while`, c2 **rotates**: it emits a top guard, the body, and a **conditional backward branch** at the bottom. The IL's unconditional `3A TOP` back-jump does **not** appear as an unconditional `b` in `.text` | **R-C3:** c2 emits the IL literally — `Ltop: cmp; bc Lexit; body; b Ltop; Lexit:` — with a genuine unconditional backward `b` | medium |
| C4 | For `for`, c2 **straightens** the IL's `init · b COND · INCR: incr · COND: cond · bc EXIT · body · b INCR · EXIT:` (`IL_STMT_GRAMMAR.md` §8.2) into a single-back-edge loop; the `b INCR` disappears and the increment moves back below the body. **A port that emits the IL's block order literally emits wrong bytes here** | **R-C4:** the rotation survives into `.text` verbatim, two branches and all | medium |
| C5 | `&&` / `||` (already branches in the IL, §7) emit one `bc` per operand with both targeting the same label; no `cror`/`crand` CR-logic instruction is emitted | **R-C5:** c2 folds the two CR bits with `crand`/`cror` and emits one `bc` | medium-high |
| C6 | The epilogue is emitted **once**, at the end, and every `return` that is not the last statement becomes a forward `b` to it — *unless* the whole tail folds into a conditional return (`bclr`) | — | medium |

### D — the folding hazard my bias points at

| # | prediction | rival registered | confidence |
|---|---|---|---|
| D1 | **At least three of the seven `pa.cpp` leaf functions emit no label and no displacement at all**, folding to a conditional-return (`bclr`, `BO=12/4`, `BI` in cr6) or to a branchless W6 spine | **R-D1:** the folds in `docs/CODEGEN_W6_COMPARE.md` are specific to a comparison whose *value* is returned, and an `if` statement always emits a real branch | medium |
| D2 | `a_eq` (`if(a==b) return 1; return 2;`) emits exactly, 20 bytes: `7f 03 20 00` `38 60 00 01` `4d 9a 00 20` `38 60 00 02` `4e 80 00 20` (`cmpw cr6,r3,r4 ; li r3,1 ; beqlr cr6 ; li r3,2 ; blr`) | **R-D2:** it emits the label form — `cmpw cr6,r3,r4 ; bne cr6,L ; li r3,1 ; blr ; L: li r3,2 ; blr` (24 bytes, one `bc`) | low-medium (registered so it can be scored wrong) |
| D3 | `b_if` (`void b_if(int a){ if(a) g(); }`) is a **leaf** — `cmpwi cr6,r3,0 ; beqlr cr6 ; b g` — with one REL24, no frame, no `.pdata` | **R-D3:** it is framed, with a forward `bc` over a `bl g` and an inline epilogue | medium |

### E — the listing seam, and instrument controls

| # | prediction | confidence |
|---|---|---|
| E1 | `cl /FAsc` prints intra-function labels (`$LN<k>`) and the **textual order of the listing's blocks equals the obj's byte order** — unlike the section order, which `docs/OBJ_DYNINIT_SHAPE.md` §6 measured as disagreeing | medium |
| E2 | The listing's printed branch word is the **canonical unrelocated** word for an external `b`/`bl` (§6 of that doc) but is the **real** word for an intra-function `bc`, because no relocation is involved | medium |
| E3 | `c2rs census` on every probe TU reports `0 in class` and a `cflow-*` key matching the shape the source has — a control on my own probes, so a probe that silently is not the shape I think it is gets caught | high |

## Decline floor — registered in advance

I will state the document **cannot be built from** and decline the corresponding
section if any of these holds:

1. **Block order is not a function of the IL's statement order** and I cannot
   state the rule it *is* a function of from the grid. Then §2 says so and names
   the probes that separate the readings, rather than shipping a rule fitted to
   `pa.cpp`.
2. **The condition-register assignment is not constant** across multi-compare
   bodies and the allocation rule does not fall out of the grid.
3. **Register allocation across a back edge** turns out to be the thing that
   decides the emitted bytes for loops. `docs/CODEGEN_W6_COMPARE.md` §6 already
   records the allocator as "demonstrably richer than a descending counter and
   not characterized"; if the loop probes need that characterization, the loop
   section is a *statement of the dependency*, not a spec.
4. Anything I can only establish on leaf probes and not on a real frontier
   function is marked **unvaried**, not constant.

## What I will not do

* No file under `crates/`. No edit to `BOARD.md`, `ROADMAP.md`, `STATUS.md`,
  `PHASE7_PLAN.md`. Proposed board rows live in my own document; next free number
  is **186**.
* No optimizer. The port is I/O-behavioral and does not reimplement c2's 35
  passes; where a fold is observable in the bytes it is recorded as a **required
  emission rule for the accepted class**, never as a pass to be reproduced in
  general.
* No merge, no push.
