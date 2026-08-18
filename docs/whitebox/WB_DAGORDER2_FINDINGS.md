# WB_DAGORDER2 — the register allocator's candidate order: a PRIORITY LIST, and it is a consequence of the scheduler

> **PROVENANCE — MIXED, and the split is stated per claim.** The **order itself
> is obj- and listing-confirmed** on a 20-cell grid at two profiles, with no
> address required. The **mechanism** (§4) is disassembly-derived: every address
> is an absolute VA in the image pinned in [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md)
> §0, `sha256 c80981c0…a66258`. Navigation only. **This lane adopts nothing into
> `crates/` and adds no [`DISCLOSURE.md`](DISCLOSURE.md) row.**

PREREG: [`WB_DAGORDER2_PREREG.md`](WB_DAGORDER2_PREREG.md) at `d0d48a21`,
**before the first grep of the export**;
[`WB_DAGORDER2_PREREG_R2.md`](WB_DAGORDER2_PREREG_R2.md) (the grid, frozen by
content hash `b06a05fc…afeee6`) at `393d18c3`, **before the first `cl.exe`**.
Scored in §7.

---

## 0. THE LANE WAS RETARGETED, AND THAT IS THE FIRST FINDING

The brief named `STRATEGY_REVIEW` §4 **lever 3** — read `dag.c`'s lowering order
at `0x10b3219f` — as *"the sole remaining characterization blocker for
`CFG_SHAPE.md` §6.2 item F"*.

**It was already discharged, five days earlier, and is in `master`.** Lane
`wb-dagorder` landed it 2026-08-13 (`rungs/2026-08-13-dagorder.md`,
[`WB_DAGORDER_FINDINGS.md`](WB_DAGORDER_FINDINGS.md), boards **#3067**–**#3071**);
`git rev-list --count master..wt-w-dagorder` is **0**. Two lanes have landed on
top of it: `w-dagclients` (#3099–#3103) and `w-itemf-price` (#3166–#3170).

So the lane took the blocker the record **actually** names — board **#3169**,
item F step **F5**:

> *"The register order `[r11 … r3, r31 … r14]` is settled and cheap. **Which
> candidate is coloured first is not.** … `codegen::alloc`'s fitted sort and
> `0x10b31c9a`'s unread worklist order are the same unknown from two sides."*

and executed the experiment board **#3166** explicitly declined to run:
*"This lane built no cell that could falsify it and does not claim it. **Testing
it is the cheapest experiment the lane found.**"*

---

## 1. THE HEADLINE

1. **The candidate order is a PRIORITY LIST, not a traversal order.**
   `FUN_10b316b1` @ **`0x10b316b1`** builds it and `FUN_10b2b82d` @
   **`0x10b2b82d`** keeps it sorted by a **two-key comparator**: `cand+0x0c`
   **descending**, tie-broken by `cand+0x44` **descending**. The list head is
   `DAT_10c43b7c`, doubly linked through `cand+0x14` (next) / `cand+0x18`
   (prev), and `FUN_10b31c9a`'s colouring loop consumes it head-first. c2's
   allocator is **priority-based colouring** (Chow–Hennessy), which is the
   reading `wb-live`'s *"there is no interference graph"* (#3049) already
   implied and nobody had stated.

2. **Four of the five rival readings are REFUTED by the obj, with no address.**
   Source order, reverse source order, arrival-register order and use-count
   order each predict cells this grid contradicts (§3).

3. **The order is moved by DEPENDENCE HEIGHT ALONE** — formal order,
   declaration order and the live set held fixed. `cnd_h2`/`cnd_h2r` and
   `cnd_h3`/`cnd_h3r` differ only in *which formal carries the taller producer*,
   and the register assignment **flips**. None of the four refuted readings can
   see height at all. **The candidate order is downstream of the scheduler
   `wb-dagorder` found.**

4. **`/Ox` and `/O1` DISAGREE on 6 of the 20 cells — and they are exactly the 6
   that carry the signal.** The eight-cell A series agrees everywhere; every H
   and U cell **inverts**. A characterization of this taken at `/Ox` — *the
   fixture profile* — publishes the reversed rule and is wrong on the workload.
   `w-section`'s seven-of-eight finding, reproduced in the allocator.

5. **THE CONSEQUENCE FOR ITEM F, which is why the lane exists: F5 IS NOT
   SEPARABLE FROM F0.** `WB_ITEMF_FINDINGS.md` §6.1 prices F5 at **2 lanes** as
   its own step. It cannot be built as its own step: its input is a priority
   accumulated over the *scheduled* code, and F0 is what produces that. **Item F
   is NOT unblocked** (§6).

---

## 2. The series — n = 1..8, at the workload's `/O1`

Published as a series over cells because `wbl_x2` alone admits **three**
readings (#3147, `w-slots`; reinforced by `w-bind16`'s `2n+1` and `w-section`'s
R-SEC at n=1..4). Every row is read from the `/FAsc` listing and confirmed
against the obj's `.text` COMDAT.

`cnd_aN(int a, …) { cnd_void(0); return a + b + … ; }` — n formals live across a
call, so the volatiles are all disallowed and the callee-saved run `r31, r30, …`
is handed out in `W-REGALLOC-1`'s fixed order. **Which formal got `r31` is which
candidate was coloured first** — an inference that depends on `W-REGALLOC-1`,
recorded as a dependency rather than assumed.

| n | assignment | **colour order** (first → last) |
|---:|---|---|
| 1 | `a=r31` | `a` |
| 2 | `a=r30 b=r31` | **`b` `a`** |
| 3 | `a=r30 b=r31 c=r29` | `b` `a` `c` |
| 4 | `a=r29 b=r30 c=r28 d=r31` | `d` `b` `a` `c` |
| 5 | `a=r29 b=r30 c=r28 d=r27 e=r31` | `e` `b` `a` `c` `d` |
| 6 | `a=r28 b=r29 c=r27 d=r26 e=r30 f=r31` | `f` `e` `b` `a` `c` `d` |
| 7 | `a=r28 b=r29 c=r27 d=r26 e=r25 f=r30 g=r31` | `g` `f` `b` `a` `c` `d` `e` |
| 8 | `a=r27 b=r28 c=r26 d=r25 e=r24 f=r29 g=r30 h=r31` | `h` `g` `f` `b` `a` `c` `d` `e` |

**The series has two parts and the second one is not explained here.** A
**head** — the last *k* formals in reverse — and a **tail** `b a c d e …`, with
*k* growing `0,0,0,1,1,2,2,3` for n = 1..8. The `b`-before-`a` inversion of the
n=2 cell survives unchanged all the way to n=8. **The head/tail split is the
lane's largest unclosed residue** (§8) — it is reported as measured, not fitted.

**`n=1` is the control that makes the rest readable**: one candidate takes
`r31`, so the run really is handed out from `r31` downward and "got `r31`" means
"was first".

## 3. The four refutations — obj-confirmed, no address needed

| reading | what it predicts | the cell that kills it |
|---|---|---|
| **H-SRC** source/formal order | `a` first, always | `cnd_a2`: **`b`** takes `r31` |
| **H-REV** reverse source order | n=4 → `d c b a` | `cnd_a4` is **`d b a c`** |
| **H-ARR** arrival-register order (either sense) | fixed in n, insensitive to the body | `cnd_h2` vs `cnd_h2r` **flip** with identical formals |
| **H-USE** use count descending (`codegen::alloc` clause 1, `ORDER.md`'s rank) | `cnd_u2` (`a+b+b+b`, `b` used 3×) → **`b`** first | `cnd_u2` gives **`a`** first at `/O1` |

**H-USE's refutation has a sharp edge worth carrying**: the *source* use count
and the *machine* use count are different numbers. `a+b+b+b` folds to `3*b + a`,
so `b` is read **once** in the emitted code, not three times. A port fitting a
candidate order against source-level use counts is fitting the wrong variable —
which is a candidate explanation for why `codegen::alloc`'s **clause 2 is
refuted** (#836, 7 of 56 fresh-holdout cells) and why clauses 3 and 4 *"carry
opposite signs inside one sort"*.

## 4. The discriminator — height alone, and it flips

`cnd_h2` / `cnd_h2r` hold the formal list, the declaration order and the live
set **fixed** and move only which formal carries the taller producer:

```c
cnd_h2 (int a,int b){ int x=a*3+7; int y=b;     cnd_void(0); return x+y; }
cnd_h2r(int a,int b){ int x=a;     int y=b*3+7; cnd_void(0); return x+y; }
```

| cell | `/O1` | `/Ox` |
|---|---|---|
| `cnd_h2` | `a=r30 b=r31` → **`b` first** | `a=r31 b=r30` → **`a` first** |
| `cnd_h2r` | `a=r31 b=r30` → **`a` first** | `a=r30 b=r31` → **`b` first** |
| `cnd_h3` | `b` `c` `a` | `a` `c` `b` |
| `cnd_h3r` | `a` `b` `c` | `c` `b` `a` |
| `cnd_u2` | `a` `b` | `b` `a` |
| `cnd_u2r` | `b` `a` | `a` `b` |

**Six pairs, six flips, and the `/O1`↔`/Ox` relation is exact reversal on all
six.** The A series (no height variation) agrees at both profiles on all eight
cells; `cnd_x2`, `cnd_x2r`, `cnd_x3`, `cnd_x3r`, `cnd_s2`, `cnd_s2r`, `cnd_c0`
agree too. **6 of 20.**

**The `/Ox` mechanism is in the bytes, and it is a second confirmation rather
than a nuisance.** `/O1` lowers `*3` as one `mulli`; `/Ox` strength-reduces it
to `slwi`+`add`, which **reads the multiplicand twice**:

```
/O1  cnd_h2   00024  1d7e0003  mulli r11,r30,3      ; a read ONCE
/Ox  cnd_h2   00394  57eb083c  slwi  r11,r31,1      ; a read TWICE
              00398  7d7f5a14  add   r11,r31,r11
```

A transformation that changes **only the DAG** — not the source, not the
formals, not the live set — inverts the candidate order. The profile axis was
run for discipline and became independent evidence for the headline.

## 5. The mechanism — DISASSEMBLY-DERIVED, and it agrees with §2–§4

`FUN_10b31c9a` @ **`0x10b31c9a`** (one caller, `0x10b7dc51` — #3166) iterates
register classes **7 → 0** over `DAT_10c400d8[class]`. Per class:

```
FUN_10b2ceb7   build                                    /* cand+0x28/+0x2c live range */
FUN_10b2d630   narrow the allowed set   (#3049)         /* AND accumulate cand+0x0c  */
FUN_10b315df   0x10b315df  seed                          /* mint/merge candidates     */
FUN_10b316b1   0x10b316b1  BUILD THE WORKLIST  ->  DAT_10c43b7c
while (DAT_10c43b7c) { pop head; colour it; }            /* head-first, 0x10b31e97   */
```

**`FUN_10b316b1` @ `0x10b316b1`** walks `wb-live`'s 1024-bucket candidate hash
`DAT_10c43b80` bucket by bucket (`0x10c44b80 − 0x10c43b80 = 0x1000` = 1024 × 4,
which independently confirms #3049's bucket count), follows each bucket's chain
at `cand+0x30`, filters to the class under `&DAT_10b022cc`'s class table, and
accumulates every survivor through `FUN_10b2b82d`. It ends
`DAT_10c43b7c = local_c` — the list the colouring loop consumes.

**`FUN_10b2b82d` @ `0x10b2b82d` is the priority comparator**, a sorted insertion
into a doubly-linked list (`+0x14` next, `+0x18` prev):

```
insert new before n  iff  n->[0x0c] <  new->[0x0c]                    /* signed   */
                     or  (n->[0x0c] == new->[0x0c] && n->[0x44] <= new->[0x44])  /* unsigned */
```

i.e. **`cand+0x0c` DESC (signed), then `cand+0x44` DESC (unsigned)** — and the
tie-tier comparison is `<=`, not `<`, so **an exact tie in both keys puts the
NEWLY inserted candidate FIRST**. That last clause is the whole behaviour of the
`/O1` cells in §2, where the benefit keys are equal and the order is decided
entirely by insertion sequence — i.e. by `0x10b316b1`'s hash-bucket walk. The same function is called
from the driver to re-insert a candidate after a spill
(`DAT_10c43b7c = FUN_10b2b82d(cand, DAT_10c43b7c)`), so **a spilled candidate
re-enters the list by priority, not at the head** — a port that models the
worklist as a stack or a queue is wrong in both directions.

### 5.0 The tie tier is a HASH BUCKET WALK — the order is not a source property

This is the part with the sharpest consequence for a port, and it follows from
the `<=` above rather than from any new address.

On an exact tie in both keys the newly inserted candidate goes to the **head**,
so the finished list is the **reverse** of `0x10b316b1`'s accumulation order —
and that accumulation order is *"buckets `0 … 1023` of `DAT_10c43b80`, chain at
`cand+0x30`"*, where a candidate's bucket is its id `cand+0x1c` — a **global
monotonic counter** `DAT_10c400d4++` (`FUN_10b54d32`, #3049) — taken mod 1024.

**So when the benefit keys tie, the candidate order is a function of a hash
bucket index over a compilation-global counter, not of anything in the source
function.** That is a direct explanation for why a source-level fitted sort
keeps being refuted: `codegen::alloc`'s clause 2 is fitting a *source*
permutation to an order whose tie tier is not a source property at all
(#836, 7 of 56 fresh-holdout cells; #3169).

**The honest limit of this sub-finding, stated because it is tempting to
overclaim.** If the tie tier were *purely* "descending candidate id", n=2's
`b a` follows and so does n=8's leading `h g f` — but **n=3 is `b a c`, and
descending id predicts `c b a`**. So the accumulation order is **not** simply id
ascending: either the mint order is not source order (globregs `0x10b55732`
mints and merges, and its promotion policy is item F1's unread step), or the
n=3 keys are not in fact tied. **This grid does not separate those two**, and
the claim is therefore *"the tie tier is bucket-walk order"*, not *"the tie tier
is reverse source order"*.

### 5.1 What the two comparator fields are — and TWO CORRECTIONS to `wb-live`

**`cand+0x0c` is the priority/benefit accumulator**, and it is accumulated in
`FUN_10b2d630` @ `0x10b2d630` — the very function `wb-live` characterized as
*"narrows the allowed set in one forward walk"*. It does both:

```
cand->[0x0c] += cand->[0x18] * iVar10          /* live, in this range   */
cand->[0x0c] -= iVar10                          /* not live             */
cand->[0x0c] -= local_18   if DAT_10c3de20 == 2 /* POGO build, different weight */
```

where `iVar10` counts the candidates on the live list scaled by `local_18`, a
block weight. **This is a spill-cost/benefit measure accumulated over the
candidate's live range** — which is why the black-box reading in §3 came out
looking like a use count, and why it is *not* one.

> **CORRECTION 1 to `WB_LIVE_FINDINGS.md`'s field table.** It enumerates the
> `0x48`-byte candidate record as *"…`+0x0c` and `+0x18` cost accumulators,
> `+0x14` live-list link…"*. **`+0x18` is not a cost accumulator at worklist
> time — it is the priority list's `prev` pointer** (`FUN_10b2b82d` writes
> `n->[0x18] = new`), and `FUN_10b3032a` uses it as a **bitfield**
> (`cand->[0x18] &= 0xfffffffe`). The field is **phase-overloaded**: a weight
> during `0x10b2d630`, a back-pointer during `0x10b31c9a`'s loop. A flat field
> table cannot express that, and a port reading it as one thing is wrong in one
> of the two phases.
>
> **CORRECTION 2: `+0x44` is not in that table at all.** The record is `0x48`
> bytes and the enumeration stops at `+0x40`. `+0x44` is the **worklist
> comparator's tie-break**, saved and restored beside `+0x40` across a live-range
> split by the spiller `FUN_10b3032a` @ `0x10b3032a`
> (`iVar7->[0x40] = …[3]; iVar7->[0x44] = …[4]`) — a function `wb-live` lists as
> *"named and not opened"*. **The unenumerated field is the one that decides
> every tie**, and at `/O1` this grid's cells are mostly ties.

## 6. IS ITEM F UNBLOCKED? — NO, AND THE PRICE MOVES THE WRONG WAY

**Stated plainly, as the brief requires.**

* **F5's question is ANSWERED** — the candidate order is a priority list, the
  comparator is named and address-cited, and the order is obj-confirmed over a
  series at two profiles. `WB_LIVE_FINDINGS.md` §10's *"the `wbl_x2` assignment
  order is unexplained"* is **closed**, and #3169's *"nobody has connected these
  two"* is **connected**: `codegen::alloc`'s fitted sort is fitting a priority
  that is accumulated over **scheduled** code.
* **F5 is NOT BUILDABLE, and it is not buildable for a NEW reason.**
  `WB_ITEMF_FINDINGS.md` §6.1 prices F5 at **2 lanes** as a separable step. It
  is **not separable**: its input is `cand+0x0c`, accumulated by `0x10b2d630`
  over the code the scheduler produced, and F0 — priced at **8** — is what
  produces that. §4's six flips are the measurement: change the DAG and the
  order changes. **F5 cannot be built before F0, and the two are not additive.**
* **Item F is therefore NOT unblocked.** What remains, in order: **F0** (8
  lanes, unchanged — a tuple-level IR *below* item A, which does not exist),
  **F1** (the promotion policy, unread), **F4**'s non-call physical def (still
  no obj cell in existence), and **F5**'s *cost model* — this lane read the
  comparator, not the accumulation weights `local_18`/`iVar10`, and `wb-live`
  already records that the cost array is **uniformly zero on all 25 cells this
  project has compiled**, so the priority's weights have no obj support either.
* **And none of it buys anything.** `w-itemf-price` **#3170** measured item F
  complete at **0** in all four named populations. **This lane converts 0, and
  predicted 0.** It is not an argument for building item F; it is an argument
  that F5's published price is wrong.

## 7. Prereg score — 6 H · 4 M

Misses stay on the page.

| id | p | outcome | note |
|---|---:|---|---|
| **P1** | 0.90 | **HIT** | `cnd_x2` reproduces `wbl_x2` exactly: `a=r30`, `b=r31` |
| **P2** | 0.90 | **HIT** | H-SRC refuted at n=2 |
| **P3** | 0.55 | **MISS** | H-SCHED **is** confirmed — but by the **H** family, not by the test P3 names. `cnd_x2r` (`b+a`) is **byte-identical** to `cnd_x2`: the commutative pair is **inert**, exactly as PREREG R2 registered before the first compile. The prediction is scored on the test it wrote down |
| **P4** | 0.60 | **MISS** | the series does **not** hold one kind across n — the head/tail split appears at n=4 and grows at n=6 |
| **P5** | 0.40 | **HIT** | and it is the complement of P4, so the pair is one bit, not two |
| **P6** | 0.55 | **MISS** | **6 of 20 cells disagree**, and they are the six discriminators. The most consequential miss on the page |
| **P7** | 0.45 | **MISS** | not a head insertion — a **sorted insertion by a two-key comparator**, `0x10b2b82d` |
| **P8** | 0.75 | **HIT** | item F is not unblocked, and F5 is not what is left |
| **P9** | 0.50 | **HIT** | `cnd_s2`/`cnd_s2r` (§8) and the A-series head/tail |
| **P10** | 0.85 | **HIT** | nothing adopted, no `DISCLOSURE.md` row |

## 8. What this lane did NOT establish

Stated so absence does not read as coverage.

1. **The A-series head/tail split is measured, not explained.** `h g f | b a c
   d e` at n=8. Neither the priority comparator alone nor reverse-use order
   predicts the tail. It is a composition of the schedule, the accumulation
   weights and (at n ≥ 6) register pressure, and this grid does not separate
   them.
2. **`cnd_s2` / `cnd_s2r` are insensitive to written operand order.** `a-b` and
   `b-a` both give `a=r31, b=r30`, and both **invert** the `+` cells. So the
   *operator* moves the assignment and the written operand order does not —
   which a canonicalization step upstream of the allocator would explain.
   `wb-dagorder`'s found-and-not-taken item 3 (*"right-first operand order and
   `+`-reassociation are grid facts; the code that produces them is unread"*) is
   the same gap, reached from the allocator side.
3. **The accumulation weights are unread.** `local_18` and `iVar10` in
   `0x10b2d630` are a block weight and a live-list count; neither is
   characterized, and `wb-live` records the cost array as uniformly zero on 25
   cells, so **no obj this project can build separates them.**
4. **`DAT_10c3de20 == 2`** takes a different subtraction (§5.1). That is the
   POGO level (`0x10b848dc`, `w-dagclients`). Not exercised — every cell here is
   a non-POGO build.
5. **The class loop 7 → 0 was not varied.** Every cell is class-int. Float and
   vector candidates are untouched.
6. **`FUN_10b31402`, `FUN_10b31544`, `FUN_10b2f40f`** — the three arms of the
   colouring loop's spill decision — were not opened.

## 9. Pre-drafted `DISCLOSURE.md` rows — NONE

Nothing here is adopted into `crates/`. The **order** (§2–§4) is black-box
re-derivable from `grids/wb-dagorder2/candorder_grid.cpp` against real `c2.dll`
with no address, and a lane adopting it should prefer that derivation. Only the
**comparator** (§5) and the **field identifications** (§5.1) are
disassembly-only; a lane adopting either owes a row naming `0x10b2b82d`,
`0x10b316b1` and `0x10b2d630` **in the same commit**.
