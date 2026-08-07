# w-order3 — PREREG

    Lane:      w-order3 (`wt-w-order3`), branched at master `e9605bd0`
    Rung:      board #174 / #1152 — the `.bss` / `.XBLD$W` order for a
               `.bss` holding an internal-linkage object
    Seam:      `crates/c2-core/src/coff/` (section ordering, watermarks,
               `emit_data_obj`). Peers: w-f23 owns `crates/c2-il` `.ex`;
               w-seam2 owns `crates/c2-core/src/codegen/`.
    Profile:   the WORKLOAD's — `/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc`
               (board #1112). Second profile for confirmation: `/Ox`, `/O2`,
               `/Od`. `c2rs gap --flags-file` is the only grader that takes a
               profile; `diff` and `census` do not.
    Judge:     real `c2.dll` under wibo + byte-exact obj compare with
               TimeDateStamp (offset 4..8) zeroed. Nothing else.
    Frozen:    `work/w-order3/cells/SHA256SUMS`, committed in the same commit
               as this file and BEFORE the first `cl.exe` on any cell and
               BEFORE one line of `crates/` is edited.

---

## 1. The incumbent, and what it would take to beat it

Board **#1148** (lane `w-align16`, closed `8fa6b119`) turned two live wrong
emits into refusals. `emit_data_obj` now returns `None` for any `.bss` holding
an internal-linkage object:

```rust
if bss.iter().any(|o| !o.external) { return None; }
```

**The incumbent is therefore a refusal that is right 100 % of the time on what
it refuses.** It emits zero wrong bytes on this shape and costs zero matches.

That sets the bar and it is not the usual bar. A rule that is *mostly* right is
**strictly worse than what ships today**, because the only currency it can pay
in is wrong bytes, and a wrong emit is the one direction `CLAUDE.md`'s
correctness rule forbids (#232 sat on master as exactly that for 255 commits).

**Registered decline floor.** This lane SHIPS a reorder only if, on the frozen
grid:

* **F1.** Every cell the widened writer ADMITS is **byte-exact** against real
  c2 — 100 %, not "most" — at the workload profile **and** at one other
  profile. One admitted cell grading `mismatch` at any profile ⇒ **revert to
  the refusal and decline**.
* **F2.** `D01` (align 4), `D02` (align 8) and `A11` (align 16) — the three
  #1148 cells, reproduced here as `O01`, `O10`, `O11` — all convert
  `codegen-gap` → `match`. Fewer than three ⇒ ship only the subclass that
  converts, refuse the rest, and say so.
* **F3.** Any region a cell does not separate from a rival is **refused**, not
  guessed. In particular, if `O04` cannot separate "internal linkage" from
  "functionless" the shipped predicate is conjunctive (the narrower one).
* **F4.** `mismatch` stays **0** at every profile and in the gate;
  `fnbyte-exact` does not shrink; `differs` does not grow; `reloc-differs`
  stays 861; `match-tu-differs` / `match-tu-reloc-differs` stay 0.

**The direction I expect to lose on**, named in advance:

1. **Mixed linkage (`O08`).** #1148 measured the external `.bss` symbol landing
   *after the following section's group*, at offset 4, with the static at 0 —
   neither Rule Y1's order nor Y1's walk. I expect to be unable to fit that
   from this grid and to **refuse mixed linkage** while shipping static-only.
2. **Multi-object static `.bss` (`O09`, `O16`).** Y1's static row says
   declaration order, fitted only on TUs with functions. If the functionless
   walk or symbol order differs I refuse above one object.
3. **`O14` (a `/GF` string literal in the same TU).** Three insertion points
   interacting; if the `.rdata` slot and the moved `.bss` collide I refuse.

I expect the widening to be **worth ~0 matches on the workload** and to be
worth the correctness statement instead. See §5.

---

## 2. The candidate rule, stated before probing

> **Rule S1′ (candidate).** Rule S1's middle clause is conditioned on linkage.
> The non-COMDAT uninitialized section is placed
>
> * **immediately before `.XBLD$W(C2)`** when it holds **at least one
>   internal-linkage (STATIC) object**;
> * **between `.XBLD$W(C2)` and `.XBLD$W(C1)`** otherwise (Rule S1 unchanged).
>
> `.data`'s slot (after `.XBLD$W(C1)`) and the `/GF` `.rdata` slot (before
> `.XBLD$W(C2)`) are **unchanged** by the move.
>
> **Rule Y1′ (candidate, symbol table).** The symbol table still follows
> section order, so a moved `.bss` group is written before `.XBLD$W(C2)`'s
> group. Within the group, the static block is Y1's static block: declaration
> order.

This is a **candidate**, not the block from `OBJ_DATA_BSS_SHAPE.md` §2.2 — that
block is explicitly labelled *"a description of three cells and NOT a rule"* and
*"do NOT encode this from the doc"*. Every clause above is graded by its own
cell below or it does not ship.

---

## 3. The rivals, and the cell that kills each

Board **#259**'s method: a rival dies to its own probe, not to an argument.
Every rival below predicts the same thing as S1′ on all three #1148 cells —
that is exactly why #1148 declined to pick one.

| # | rival | it says `.bss` moves before `C2` iff … | killed by | its prediction there | S1′'s prediction |
|---|---|---|---|---|---|
| **R1** | **reloc** | a `.data` relocation targets an object in the `.bss` | **`O02`** `A g; A* p = &g;` | **moves** | stays between |
| **R2** | **address-taken** | the `.bss` object's address is taken anywhere | **`O02`** (same cell, extern, address taken) | **moves** | stays between |
| **R3** | **functionless** | the TU defines no functions | **`O03`** `A g;` (functionless, extern) | **moves** | stays between |
| **R3b** | **functionless** (other side) | — | **`O04`** `static A g; void f(){g.a=1;}` | **stays between** | moves |
| **R4** | **all-static** | *every* object in the `.bss` is internal-linkage | **`O08`** `A g; static A h; A* p=&h;` | **stays between** | moves |
| **R5** | **`.data` present** | the obj has a non-COMDAT `.data` | **`O03`**, **`O04`** (no `.data`, and `O05`) | `O04` stays between | `O04` moves |
| **R6** | **alignment** | the section's alignment nibble exceeds some threshold | **`O01`/`O12`/`O17`** at align 4 | stays between at 4 | moves at 4 |
| **R7** | **object count** | more than one object in the `.bss` | **`O01`** (one object) | stays between | moves |
| **R8** | **aggregate type** | the object is a class/struct rather than a scalar | **`O12`** `static int g; int* p=&g;` | stays between | moves |

`O04` is the **load-bearing cell of this lane**: it is the only shape in the
grid that has an internal-linkage `.bss` object alive **without** a `.data`
relocation into it — a function reference keeps it alive instead. #1152's own
warning is that *"nothing here separates 'internal linkage moves it' from 'a
`.data` relocation into `.bss` moves it', because every surviving cell has
both"*. `O04` is the cell with only one of them. If it does not separate them,
F3 applies and the shipped predicate is the conjunction.

**A recorded unreachability is a statement about the cells someone thought
of.** `wsect_drop_static.cpp` recorded the static-`.bss` drop and
`wsect_data_linkage.cpp` concluded mixed linkage was unreachable; the route
around it was one line of C++. So this grid deliberately writes **both** routes
around the drop — the `.data` initializer (`O01`) *and* the function reference
(`O04`) — rather than trusting either exclusion.

---

## 4. The frozen grid — structural axes, not values

*"A generated axis is only as good as the axes it varies"* — three lanes here
have been bitten by a grid that varied values exhaustively and structure not at
all. So the axes are structural and the values are held at their least
interesting setting (`int`, size 4, align 4) except where the axis IS the value.

Axes varied: **linkage** (extern / static / mixed) × **liveness route**
(`.data` initializer / function reference / both / none) × **function presence**
(yes / no) × **object count in the `.bss`** (1 / 2 / 3) × **`.data` presence**
(yes / no) × **`/GF` `.rdata` presence** (yes / no) × **alignment** (4 / 8 / 16)
× **object shape** (scalar / aggregate / array) × **`.data` object linkage**
(extern / static).

| cell | source | what it is for |
|---|---|---|
| `O01` | `static A g; A* p=&g;` | **the defect** (`D01`), align 4 |
| `O02` | `A g; A* p=&g;` | extern + reloc — **kills R1, R2** |
| `O03` | `A g;` | extern, functionless, no `.data` — **kills R3, R5** |
| `O04` | `static A g; void f(){g.a=1;}` | **the separator** — static alive with NO reloc |
| `O05` | `A g; void f(){g.a=1;}` | control for `O04` |
| `O06` | `static A g; A* p=&g; void f(){…}` | both routes at once |
| `O07` | `A g; A* p=&g; void f(){…}` | control for `O06` |
| `O08` | `A g; static A h; A* p=&h;` | **mixed** (`D07`) — **kills R4** |
| `O09` | two statics, two `.data` pointers | count 2 — walk + symbol order |
| `O10` | `O01` at **align 8** | `D02` |
| `O11` | `O01` at **align 16** | `A11` |
| `O12` | `static int g; int* p=&g;` | scalar — **kills R8**; align 4 — **kills R6** |
| `O13` | `O01` + `int d=7;` | a second `.data` object with no reloc |
| `O14` | `O01` + `const char* s="hi";` | the `/GF` `.rdata` insertion point |
| `O15` | `static A g; int d=7;` | **the drop control** — expect 5 sections, no `.bss` |
| `O16` | three statics | above `MAX_OBJECTS_PER_SECTION` — measure, do not ship |
| `O17` | `static int g[4]; int* p=g;` | array — size 16 at align 4 |
| `O18` | `static A g; static A* p=&g; A** pp=&p;` | the `.data` object is itself STATIC |

**One directory per cell** for capture and for grading (board #1045 — four
probes once shared a PID-keyed temp dir, the captures raced, and the lane
published a finding that reversed when it was rerun).

---

## 5. Predictions, recorded so they can be wrong

| # | prediction | how it fails |
|---|---|---|
| **P1** | `O02`, `O03`, `O05`, `O07` keep `.bss` **between** the watermarks | any of them before `C2` ⇒ S1′ wrong, R1/R2/R3 live |
| **P2** | `O01`, `O10`, `O11`, `O12`, `O17` put `.bss` **before both** watermarks | any staying between ⇒ S1′ wrong |
| **P3** | `O04` puts `.bss` **before both** watermarks | staying between ⇒ the trigger needs the reloc, F3 fires, predicate becomes conjunctive |
| **P4** | `O08` (mixed) puts `.bss` before both, static at 0, external at 4 | either way it is **refused** in the writer |
| **P5** | `.data` stays after `.XBLD$W(C1)` in every cell that has one | it moving ⇒ the whole model is wrong |
| **P6** | `O15` emits **5** sections with no `.bss` (the drop) | a `.bss` there ⇒ `wsect_drop_static.cpp` is wrong |
| **P7** | `O14`'s `/GF` `.rdata` stays before `C2`, with the moved `.bss` adjacent to it | a collision ⇒ refuse when a literal is present |
| **P8** | the workload has **ZERO** objs with a `.bss` before `.XBLD$W:C2` | non-zero ⇒ this is worth factor-c and the census says so |
| **P9** | `factor-c` is **169 → 169** and the fixture match count moves by exactly the fixtures this lane adds | any other movement is a peer interaction, run peerkeys |

**P8 is checked from `work/w-bss/census/sections.jsonl` — 871 objs, whose
`order` array is exactly this question** — so this lane can answer "cannot"
rather than "did not", the way `w-align16` did for #1150.

## 6. Rule Y1 on functionless TUs — the second, outranking question

The brief carries a standing instruction: if **Rule Y1 is wrong on functionless
TUs** in a way that is *live* — i.e. reachable by `emit_data_obj` and not
already refused — that is a **sixth alarm** and it outranks this rung.

Registered position **before probing**: #1148 already found Y1 misapplied on
the functionless mixed case and **already closed it by refusal**, so the
remaining live use of Y1 in `emit_data_obj` is the extern-only `.bss`. This lane
therefore expects to find Y1 **wrong but not live** — a scope error already
fenced. `O09` (two statics) and `O16` (three) test Y1's *static* row on a
functionless TU, which nothing has measured; `O08` re-measures the mixed row.
**If any of them shows a Y1 misapplication that is still reachable, this lane
stops and reports it as the alarm.**

---

## 7. What this lane will NOT do

* Not touch `crates/c2-il` (w-f23) or `crates/c2-core/src/codegen/` (w-seam2).
* Not raise `MAX_OBJECTS_PER_SECTION` — that is board #184.
* Not encode the #1152 block from the doc; every clause is re-measured here.
* Not re-run `w-align16`'s diag cells as evidence — they are reproduced as
  `O01`/`O10`/`O11`/`O08` inside this lane's own frozen manifest.
* Not trade a refusal for a guess. A registered decline is a result.
