# `P_BLOCKORDER.md` — block emission order

> **PROVENANCE — DISASSEMBLY-DERIVED.** Every address is an absolute VA in
> `compilers/X360/16.00.11886.00/c2.dll`, sha256
> `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`, verified
> by this lane before the first byte was read. See
> [`../DISCLOSURE.md`](../DISCLOSURE.md). Marks: **`[R]`** read only (a
> hypothesis), **`[O]`** obj-confirmed against real c2 under wibo, **`[I]`**
> inferred. See [`README.md`](README.md) §2.

**Produced by** lane `w-read-r8`, read **R8** of
[`../READ_PLAN_2026-08-21.md`](../READ_PLAN_2026-08-21.md) §3 — the row the
plan flagged as *"the only row with no known address for the rule it seeks"*,
priced 5–10 days uncertain, with a priced decline pre-authorized. Grade and
prereg score: [`../WB_BLOCKORDER_FINDINGS.md`](../WB_BLOCKORDER_FINDINGS.md).
Board **#3437**–**#3441**.

**This is the other half of [`P_LABEL.md`](P_LABEL.md).** R3 gave the *charge* —
what makes c2's label counter move — and its §8 open #1 says plainly: *"The
ORDER. Which block a `$M` lands on is R8's. A charge rule alone cannot place a
label."* §5 below closes that seam.

---

## 0. The one-paragraph answer

**c2 has no block-ordering pass, and no block-order key.** Blocks are not
records that get sorted; they are spans of a **single flat doubly-linked tuple
list**, and the emitter walks that list once, linearly, following `tuple+0`.
Emission order *is* list order, and list order is whatever the lowering left —
so every question of the form *"what order do blocks come out in?"* is really a
question about **which order some pass spliced tuples in**, and there are five
splice primitives and hundreds of callers. `M1` (source order) and `M2`
(reverse case order) were never two rival rules about blocks. They are two
different **traversals** inside the `switch` lowering, which has **three**
lowerings and not two, and the decisive fact is that the decision-tree form
traverses a **value-sorted** structure while the other two traverse the
**source** list. Every prior cell in this repo wrote its cases in ascending
source order, where those two are the same sequence — which is why the record
has called it *"reverse **source** order"* for a year, and it is not.

---

## 1. The emit walk — `FUN_10b338f5` @ `0x10b338f5`, 426 B `[R]`

The single consumer of block order in the whole compiler.

```c
puVar4 = *(undefined4 **)(*(int *)param_1[2] + 0x1c);   /* the list head */
do {
    if (puVar4 == NULL) { /* finish the section, return */ }
    ...
    bVar2 = *(byte *)(puVar4 + 2);            /* tuple+8, the KIND byte */
    if (0xc < bVar2) {
        if (bVar2 < 0x17) {
            local_1c = FUN_10bf9f15((int)puVar4, local_10, local_14, 1);  /* ENCODE */
            local_14 = local_14 + local_1c;               /* the running offset */
            *(int *)(iVar5 + 0x18) += local_1c;           /* the section's size */
        }
        else if (bVar2 == 0x17) { ... }       /* section switch */
        else if (bVar2 == 0x18) { ... }       /* section switch + memset padding */
        else if (bVar2 == 0x1b) { iVar3 = puVar4[9]; goto LAB_10b339c3; }  /* LABEL */
    }
    puVar4 = (undefined4 *)*puVar4;           /* <== the whole of "block order" */
} while (true);
```

| fact | evidence |
|---|---|
| the advance is `tuple = *(tuple+0)` — one flat singly-linked traversal | **`0x10b33a21`**, `8b 36` = `mov esi,[esi]` — **two bytes**, and the loop back-edge is `jmp 0x10b33a21` at `0x10b33a9d` |
| **no sort, no comparator, no ordering key, no block loop, no recursion** | the entire 426-byte body. Its **whole direct callee set is five**: `memset`, `__security_check_cookie`, the encoder `FUN_10bf9f15`, `FUN_10bd456b` and `FUN_10c205f1` — there is nothing present that could sort |
| a running offset (`local_14`, seeded `-1`) accumulates encoded lengths | `0x10b33997` |
| the encoder called is `FUN_10bf9f15` — **R2's**, [`P_ENCODE.md`](P_ENCODE.md) | `0x10b33990` |
| a tuple is a real instruction iff kind `∈ [0x0d, 0x16]` | `0x10b33978` |

> **This independently confirms a black-box conclusion the repo already had and
> could not explain.** Board **#2352** (`w-ifn`, nine cells) concluded *"the
> emitter therefore needs a running offset and **no ordering pass at all**"*.
> `local_14` **is** that running offset, and "no ordering pass at all" is now a
> read fact rather than an inference from nine objs.

### 1.1 The four kinds the walk distinguishes `[R]`

| `tuple+8` | meaning | what the walk does |
|---|---|---|
| `0x0d`–`0x16` | a machine instruction | encode, advance the offset |
| `0x17` (with `tuple+4 == 0x318`) | section start | reload the offset from `sect+0x18` |
| `0x18` | section switch | reload, and `memset` pad to the section's current size |
| **`0x1b`** | **a label** | reload the offset from `tuple+0x24` |

---

## 2. The tuple record, and the five splice primitives `[R]`

The fields, established by the primitives themselves and agreeing with the
independently-read `tuple+0 next, +0x10 prev` in [`P_DAG.md`](P_DAG.md):

| offset | field |
|---|---|
| `+0x00` | **next** — the only thing the emit walk follows |
| `+0x04` | opcode (`0x308` label, `0x318` section start, `0x2e8` multiway, …) |
| `+0x08` | kind byte, stamped by the allocator |
| `+0x09` | flags; bit 0 = is a real instruction ([`P_ENCODE.md`](P_ENCODE.md)) |
| `+0x10` | **prev** |

| VA | size | what it is | body |
|---|---:|---|---|
| **`0x10bd3815`** | 15 | **INSERT AFTER** `at` | `new->next=at->next; at->next=new; new->prev=at; new->next->prev=new` |
| **`0x10bd3824`** | 17 | **INSERT BEFORE** `at` | `new->prev=at->prev; at->prev=new; new->next=at; new->prev->next=new` |
| **`0x10bd3835`** | 29 | **SPLICE A CHAIN AFTER** `at` | links the chain in, walks it to its end, reattaches `at`'s old successor |
| **`0x10bd3852`** | 31 | **UNLINK** `t` | `t->prev->next=t->next; t->next->prev=t->prev;` then nulls both |
| **`0x10bd38d0`** | 50 | **MOVE A RANGE** | unlink around `b`, relink the range in front of `a` |
| `0x10bd3750` | 83 | the tuple allocator | size from `DAT_10b18910[kind]`, free-list `DAT_10c6f848`, stamps `tuple+8` |

### 2.1 The direction of a splice is a RUNTIME PARAMETER, not a property of the builder `[R]`

This is the structural fact that makes a single "block order rule" impossible
to state, and it is why the question stayed open.

Measured by [`../scripts/dump_tuple_splice.py`](../scripts/dump_tuple_splice.py)
directly from the image, and independently cross-checked against the Ghidra
objdump — **both routes agree to the digit**:

| primitive | direct `E8` call sites | address taken as data |
|---|---:|---:|
| `0x10bd3815` INSERT AFTER | **131** | **201** |
| `0x10bd3824` INSERT BEFORE | **207** | **506** |
| `0x10bd3835` SPLICE CHAIN | **90** | **77** |

The address-takes are `push imm32` (503) and `mov r32, imm32` (204) feeding the
shared tuple builders — `FUN_10bd79b9`, `FUN_10bd76e6`, `FUN_10bd72b0`,
`FUN_10bd7780`, `FUN_10bd59aa` — **every one of which takes an inserter as an
argument**. **75 functions load both inserters as immediates**, i.e. they
choose direction at runtime. Verified by eye rather than trusted from a count:

```
10b2bf69:  68 24 38 bd 10     push  0x10bd3824          ; passed as a parameter
10b37ed6:  bf 24 38 bd 10     mov   edi,0x10bd3824      ; under a `test ebx,ebx / je`
```

> **The R3 closure argument does not transfer, and the failure is the finding.**
> R3 closed the label-allocator population because `FUN_10b97dd0`'s VA occurs
> **zero** times as data ([`P_LABEL.md`](P_LABEL.md) §2). Run the same test here
> and it fails loudly: the inserters' VAs occur 707 times as immediates. **The
> splice-site population of this compiler is not closed and cannot be closed by
> that argument** — so any claim of the form *"pass X is the author of block
> order"* is unfounded unless it also says why no other splice site can reach
> the same tuples. This page makes no such claim.

---

## 3. Where a block order comes from — the pipeline `[R]`

Per-function driver **`FUN_10b7e6af`** @ `0x10b7e6af`, in order:

| # | pass | gate |
|---:|---|---|
| 1 | `FUN_10b7dbf6` | `DAT_10c2e2fc` (optimizer on) |
| 2 | `FUN_10b7dc51` | ditto — **the list scheduler, mode 1, ×3** ([`P_DAG.md`](P_DAG.md)) |
| 3 | `FUN_10b7dd2c` | |
| 4 | `FUN_10b7ddff` | |
| 5 | `FUN_10b7de4a` | |
| 6 | `FUN_10b7ded5` | `DAT_10c2e2fc` |
| 7 | `FUN_10b7df57` | **the list scheduler, mode 0, ×1** |
| 8 | **`FUN_10b7e032`** | the emit tail |

`FUN_10b7e032` @ `0x10b7e032` runs, in order: `FUN_10c21b03`, `FUN_10be46f0`,
`FUN_10b3c6e5`, **`FUN_10b35c78`**, `FUN_10b9d6be`, **`FUN_10b36169`**,
`FUN_10c12099`, `FUN_10b821c3`, `FUN_10c275a7`, and finally **`FUN_10b3421b`**
→ `FUN_10b338f5`, the emit walk of §1.

The scheduler re-links the list in scheduled order through `FUN_10be626c` @
`0x10be626c` (`P_DAG.md`), so it is **a second author of order** and it runs
four times per function. `WB_DAGCLIENTS_FINDINGS.md` names two more in
`factor.c` (`0x10b3b167`, `0x10b3b41b`). **This page does not claim the list of
authors is complete** — see §2.1.

---

## 4. The `switch` lowering — the module that produces `M1` and `M2` `[R]`

No address for this existed anywhere in the repo before this lane;
`WB_REGALLOC_FINDINGS.md:708` called it *"an unread algorithm"*. It is a
self-contained module: `0x10bd0f55`–`0x10bd22a7` (target-independent) plus
`0x10c1cf09`–`0x10c1e40b` (PPC-specific).

| VA | role |
|---|---|
| `0x10bc2d7a` | `reader.c`'s 189-arm dispatch ([`P_ILRECORD.md`](P_ILRECORD.md), R5's). Arms `0x3b`/`0x3c`/`0x3d` build the case list |
| `0x10bd22a7` | **the switch-lowering entry point** |
| `0x10bd13f5` | case-list normalize/merge; returns the count |
| `0x10bd1634` | statistics: value count, test count, distinct labels, 64-bit range |
| **`0x10bd1373`** | **the table-vs-tree decider** (§4.1) |
| `0x10bd1f1a` | the **recursive** driver; picks a builder, performs the binary split |
| `0x10bd1801` | the split-point chooser |
| `0x10bd16eb` | the split's compare+branch pair |
| `0x10bd19a3` | the leaf builder (linear compare chain) |
| **`0x10bd1c85`** | **the jump-table builder** |
| `0x10c1de62` | the paired `blt`/`beq` ladder — the form that emits three `cmplwi` for six cases |
| `0x10c1dc58` | the late `0x2e8` → jump-table expansion, reached from `FUN_10b36169` |
| `0x10c1da6f` | the deferred table emit; chooses the entry width |

### 4.1 The two thresholds — read from the image, then reproduced from objs `[O]`

**Threshold record at `0x10b2417c`**, a 2 × 4-dword table, the only two
references to it in the image:

| record | dwords | selected when |
|---|---|---|
| `0x10b2417c` | `4, 4, 3, 0xff` | `DAT_10c2e310 != 0` (optimize for **size**) |
| **`0x10b2418c`** | **`8`, `4`, `3`, `0`** | `DAT_10c2e310 == 0` (optimize for **speed**) |

Slot `+0` — **8** at speed — is the minimum test count before c2 will consider a
table or a split at all. Read at **`0x10bd1388`** (`cmp edx, [ecx+0x10b2417c]`)
and again by the split-point chooser at `0x10bd18e0`.

**And a second, later threshold**: `0x10c1dc7b` tests the value **range > 9**
before the compact byte-indexed (`lbzx`) table is used.

| measured from objs (this lane's grid, dense cases) | shape |
|---|---|
| n = 2 … 7 | **decision tree** |
| n = 8, 9 | **CTR ladder** (`mtctr` + `bdz` chain, no index table) |
| n ≥ 10 | **jump table** (`lbzx` byte index + `bctr`) |

Both boundaries fall exactly where the two read constants put them: **8** at
`0x10bd1388`, and **range > 9** at `0x10c1dc7b`. The disassembly and the objs
converge on the same two integers from opposite directions.

> **There are THREE lowerings, not two, and the record has only ever named
> two.** The CTR ladder at n = 8, 9 is `mtctr` + a chain of `bdz`/`bdzf` with
> **no index table at all**. A classifier keyed on `mtctr` calls it a jump
> table — **this lane's own first classifier did**, and only reading the emitted
> words caught it.

### 4.1a The case list is built by APPEND and is never sorted `[R]`

Load-bearing for §5, so it is quoted rather than summarised. In
`FUN_10bc2d7a` (`reader.c`'s dispatch):

```c
case 0x3c:                                    /* IL 0x3C — switch table header */
    local_4c = FUN_10c2022a(7,0x48);          /* the head node, 0x48 bytes     */
    DAT_10c6f2a4 = local_4c;                  /* the global case-list head     */
    local_4c[6] = *(int *)(local_18 + 0x33);  /* +0x18 = the DEFAULT label     */
    break;
case 0x3d:                                    /* IL 0x3D — one case entry      */
    piVar7 = FUN_10c2022a(7,0x48);
    *local_4c = (int)piVar7;                  /* tail->next = new  <== APPEND  */
    piVar7[4] = piVar10[6];  piVar7[5] = piVar10[7];   /* hi */
    piVar7[2] = piVar7[4];   piVar7[3] = piVar7[5];    /* lo */
    piVar7[6] = *(int *)(local_18 + 0x33);    /* the case's label symbol       */
    local_4c = piVar7;                        /* advance the tail cursor       */
    break;
```

**A running tail cursor and no sort** — so the case list is in **source order**,
and the lowerings that walk it produce source order for free. There is no
`qsort` anywhere on this path; the image's only two `qsort` sites are in the
PGO database code. Three separate places *assume* ascending order instead
(`FUN_10bd1634`'s span, `FUN_10bd13f5`'s `hi == next.lo - 1` merge,
`FUN_10c1de06`'s running counter), which is what makes the decision tree's
value-ordered traversal a **different** traversal rather than a re-sort.

### 4.2 The `switch` value is compared UNSIGNED because the type is hard-coded `[R]`

`WB_REGALLOC_FINDINGS.md` §7.6 measured *"the switch value is compared
**unsigned** even though the C type is `int`"* and could not say why. The
reason is a literal: `FUN_10c1de62` passes the type constant **`0x2004`** to
`FUN_10bd575d` and `FUN_10bd79b9`, and `FUN_10bd1c85` forms
`*(ushort*)(op+10) & 0xfff | 0x2000`. `0x2000` is the unsigned type class
(`reader.c` tests `(type & 0xf000) == 0x1000` for signed). **The source type is
never consulted on this path.**

---

## 5. THE RULE — `M1` and `M2` reconciled `[O]`

**Stated once, in the form a port can implement.**

> Blocks are emitted in **tuple-list order**, and nothing reorders them at emit
> time. For a `switch`, the arms' order is fixed by **which traversal the chosen
> lowering used**:
>
> * **jump table** and **CTR ladder** — the lowering walks the **case list**,
>   which `reader.c` built by **append** in source order and **never sorts**.
>   Arms come out in **SOURCE order**.
> * **decision tree** — the lowering walks a **value-ordered** structure. Arms
>   come out in **recursive-pivot order over the case VALUES**, and **source
>   order does not enter into it at all**:
>
> ```
> emit(V):                        # V = the case values, sorted ASCENDING
>     n = |V|
>     if n < 8:  return reverse(V)          # descending
>     p = n // 2
>     return emit(V[:p]) ++ [V[p]] ++ emit(V[p+1:])
> ```
>
> The bottom-out constant **8** is the one at `0x10b2418c` read by
> `0x10bd1388`/`0x10bd18e0` — it is **read, not fitted**.
>
> **Default arm placement:** decision tree → **first**; ladder and table →
> **last**.

**Scored 22 HIT / 0 MISS** over every decision-tree cell of both grids,
including **six out-of-sample holdout cells whose predictions were frozen and
committed before the first `cl.exe`**
([`../grids/wb-blockorder/`](../grids/wb-blockorder/)).

**And 239 HIT / 1 MISS over a 240-cell randomized corpus** `[O]` — case count
(2–24), value set, density (dense / sparse / clustered / wide) and **source
order** all randomized independently, seed-deterministic
([`../grids/wb-blockorder/gen_random.py`](../grids/wb-blockorder/gen_random.py)).
A marker too wide for a `li` immediate makes a cell **unreadable**, and
unreadable cells are reported as such and never scored as passes.

### 5.0a The one miss is a HYBRID lowering, and the record has never described one `[O]`

`sw_rc065`, 19 clustered values
`{26–29, 31–33, 36–38, 52, 53, 119–122, 173–175}`:

```
emitted, as values:  36 37 52 27 26 53 32 38 28 29 33 31 | 119 | 175 174 173 122 121 120
                     `------------ SOURCE order ------------'   `--- the tree rule ---'
                        the 12 low values, as a JUMP TABLE        pivot 119, then reverse
```

c2 **partitioned the case set** and lowered the parts differently: a jump table
for the dense low cluster (12 values spanning 26–53) and a decision tree for the
seven outliers. **§5's rule holds on each part; what it omits is the partition
step**, and that step is already read: `FUN_10bd1801` @ `0x10bd1801` chooses a
split using the **max-gap parameter `3`** at `[0x10b24184 + mode*0x10]`, loaded
at **`0x10bd1844`**. The cluster boundaries here are exactly the gaps > 3
(38→52, 53→119, 122→173).

So the honest scope of §5 is **per contiguous cluster**, and the clustering rule
is named but not modelled. It is 1 cell in 240, it is explained by a constant
read from the image rather than fitted to the cell, and it is the shape the
prereg registered (**P3.3**) as the thing a corpus finds and a hand grid cannot.

### 5.1 The correction this forces on the record `[O]`

**Every published statement of `M2` in this repo says "reverse *source*
order", and it is wrong.** Board **#1906**, `WB_LOOP_FINDINGS.md:449`,
`WB_REGALLOC_FINDINGS.md:541`, `P_DAG.md:259` and `CFG_SHAPE.md:1757` all say
it. It is **descending case VALUE**.

The reason nobody could have known: **every prior block-order cell in this repo
writes its cases in ascending source order**, and in such a cell "reverse
source order" and "descending value" are the *same sequence*. They are
structurally incapable of separating the two. This lane's `scram` family writes
the same case values in a non-ascending source order, so the two predictions
disagree cell by cell, and the objs decide:

| cell | source order | descending value | c2 emitted |
|---|---|---|---|
| `sw_scram04`–`07` (tree) | ✗ | **✓** | descending value |
| `sw_spscram06` (tree) | ✗ | **✓** | descending value |
| `sw_scram08` (ladder) | **✓** | ✗ | source |
| `sw_scram12`/`13`/`16` (table) | **✓** | ✗ | source |

`sw_spscram12` and `sw_sparse12` hold the same case *values* in two different
*source* orders and emit **the identical arm sequence** — an independent
confirmation on the same point.

### 5.2 What this gives `P_LABEL.md` §8 open #1 `[R]`

R3 read the *charge*: `FUN_10b9a455` mints a label symbol. **The placement is
`FUN_10bd415e` @ `0x10bd415e`, 31 B**, which wraps that symbol into a
**kind-`0x1b`, opcode-`0x308` tuple** and back-links it at `labelsym+0x33`. The
tuple is then spliced into the one list by an inserter, and §1's emit walk turns
it into an address at the `0x1b` arm. **85 functions call `FUN_10bd415e`** —
the placement population, next to R3's 86 minting callers.

So a label lands **exactly where its `0x1b` tuple was spliced**, and "which
block does a `$M` land on" is answered: the one whose tuples follow it in the
list. The jump-table path makes the join concrete — the emitted table base is a
`$LN` label and the index table a `$T` symbol (`$LN18`/`$T3128` in cell
`sw_dense12`), both minted through R3's allocator.

---

## 6. What this page does NOT give

| # | open |
|---|---|
| 1 | **Whether the tree's arm order is a REORDER or a MATERIALIZATION.** Every arm in this lane's grid is a `return CONST` leaf (two words), the same shape `WB_REGALLOC_FINDINGS.md` §7.6 used. The read says the switch module splices the switch **body** in unchanged (`FUN_10bd3835` at `reader.c`'s `0x3b` arm) and that its five expanders insert only compare/branch/table tuples — **none of them moves an arm block**. So the tree order may be arms *materialized* in traversal order rather than blocks *moved*. This grid cannot separate those, and a grid of call-bearing arms would. **Named, not resolved.** |
| 2 | **`FUN_10b35c78`** @ `0x10b35c78` (`fg.c` band) runs inside `FUN_10b7e032` **before** `FUN_10b36169`, and is unread. It is the standing candidate for open #1. |
| 3 | **The author population is not closed** (§2.1), and cannot be closed by R3's argument. Any "pass X owns block order" claim needs a different argument. |
| 4 | **Non-`switch` block order is inherited, not re-derived here.** `CFG_SHAPE.md` §3.4 (10 of 11 cells), board **#2352** (`if-2`/`if-n`, 9 cells) and **#1906** (loops) stand; §1 explains *why* they are source order — nothing reorders — but this lane compiled no new `if` or loop cell. |
| 5 | **Code motion still moves blocks.** `CFG_SHAPE.md` §3.4.1's `?d_join` tail-merges and hoists, inverting the layout. §1 says why that is possible at all — order is list order, so any splice moves it — but the *decision* to tail-merge is not read here. |
| 6 | **`/O2` and POGO.** Both grids were compiled at two modes (§7) and are byte-identical at both; neither is `/O2` or POGO. |
| 7 | **The size-mode threshold record** (`0x10b2417c`, `{4,4,3,0xff}`) is read but **not** exercised — no `/Os` cell was compiled. The `4` predicts a much earlier table, untested. |

---

## 6.1 A note on TU attribution — do not quote `ADDR.tsv`'s file names for this page

`ADDR.tsv` assigns a translation unit by **address banding** between the ICE-site
anchors in `c2_tus.tsv`, and every address on this page lands in a **`gap`**,
i.e. a hypothesis rather than a fact (`READ_PLAN` §5.4). The banding calls the
emit walk `dag.c` and the switch-lowering module `stack.c`; **neither is
credible**, and this page claims no TU for either.

The likely reason is stated in `c2_tus.tsv`'s own header: *"a file with no ICE
site is invisible here."* There is **no `swtch.c`** among the 59 recovered
units, and a self-contained switch lowerer spanning `0x10bd0f55`–`0x10bd22a7`
is exactly the shape of a file that raises no `C1001`. The same banding
artifact affects the `page` column: `build_ref.py` assigns a page from an
address band, so these addresses are attributed to `P_DAG.md`. **This page has
no band** — its subject is a data structure and its users — which is the same
status [`P_LABEL.md`](P_LABEL.md) carries and for the same reason.

---

## 7. Reproduce

```sh
sha256sum compilers/X360/16.00.11886.00/c2.dll   # must equal the digest above

# the splice primitives and the closure test, straight from the image
python3 docs/whitebox/scripts/dump_tuple_splice.py \
        compilers/X360/16.00.11886.00/c2.dll

# the grid (frozen: docs/whitebox/grids/wb-blockorder/FROZEN.sha256)
python3 docs/whitebox/grids/wb-blockorder/gen_grid.py
python3 docs/whitebox/grids/wb-blockorder/gen_holdout.py
sh work/w-ifn/probe/cc.sh docs/whitebox/grids/wb-blockorder/switch_grid.cpp \
        /tmp/sw_o1 /nologo /c /GR /O1 /Oi /EHsc > /tmp/sw_o1.dis.txt
python3 docs/whitebox/grids/wb-blockorder/score.py /tmp/sw_o1.dis.txt

# the second mode — every code word is identical
sh work/w-ifn/probe/cc.sh docs/whitebox/grids/wb-blockorder/switch_grid.cpp \
        /tmp/sw_m2 /nologo /O1 /GS- /c > /tmp/sw_m2.dis.txt
```
