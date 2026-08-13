# WB_LIVE — the liveness and interference construction that feeds the selector

> **PROVENANCE — DISASSEMBLY-DERIVED.** Every address below is an absolute VA in
> the exact image pinned in [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0 —
> `sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`,
> verified at the top of this lane against
> `compilers/X360/16.00.11886.00/c2.dll`. This is **navigation** until a row
> lands in [`DISCLOSURE.md`](DISCLOSURE.md). **The obj is the sole judge**
> (method doc §7). **This lane adopts nothing into `crates/` and adds no
> `DISCLOSURE.md` row.**

PREREG: [`WB_LIVE_PREREG.md`](WB_LIVE_PREREG.md), committed at `c0b952d5`
**before the first grep of `~/ghidra-projects/export/c2/`**;
[`WB_LIVE_PREREG_R2.md`](WB_LIVE_PREREG_R2.md) (the grid) committed at
`4fc56f7c` **before the first `cl.exe` of this lane**, freezing the grid by
**content hash**. Scored in §8.

**Why this lane exists.** `wb-regalloc` read the *selector* (board **#1821**):
minimum cost over the **interference-allowed** candidates. It did not read what
makes a candidate interference-allowed. `docs/CFG_SHAPE.md` §6.2 item **F** —
values live across block boundaries — is the one item of the specified
block/instruction IR whose mechanism is uncharacterized, and §6.2 says so in
those words. This lane reads it.

---

## 1. Where it lives

| what | address | notes |
|---|---|---|
| **the allocator driver** | **`0x10b31c9a`** | `color.c`. Per class 7→0: build → colour → spill → rebuild |
| pressure reduction (pre-spill) | `0x10b2ceb7` | backward per-block tuple walk; spills when local pressure > `k` |
| **the interference build** | **`0x10b2d630`** | forward per-block tuple walk; narrows each candidate's allowed-register set |
| **the on-demand neighbour set** | **`0x10b30517`** | re-walks one candidate's live range and returns the *set of candidates simultaneously live with it* |
| the selector | `0x10b2e7f8` | #1821; §5 corrects one clause of it |
| **the backward liveness fixpoint** | **`0x10b54904`** | `globregs.c` |
| the forward availability fixpoint | `0x10b54848` | same shape, opposite direction |
| liveness ∩ availability | `0x10b549b5` | `live_in &= avail_in`, `live_out &= avail_out` |
| **the gen/kill builder** | **`0x10c207ec`** | backward per-block tuple walk |
| **the liveness driver the allocator calls first** | **`0x10c20f79`** | runs the pair twice, then binds each candidate's first/last block |
| **the candidate constructor** | **`0x10b54d32`** | mints the id, allocates the allowed-register set |
| candidate lookup by id | `0x10b2c21d` | 1024-bucket hash at `0x10c43b80`, chain at `cand+0x30`, ICE if absent |
| candidate table insert / clear | `0x10b2c206` / `0x10b2c1f1` | |
| per-block set allocation | `0x10b3f454` | two parallel 8-slot dataflow groups per block |
| per-block allocator sets | `0x10b2b3f0` | allocates `blk+0x40`, `+0x44`, `+0x50`, `+0x54` |

### 1.1 The sparse bitset library — everything above is built on it

`0x10b26763`…`0x10b275d8`. A set is a **sorted singly-linked list of 64-bit
chunks**, node `{base:u32, w0:u32, w1:u32, next:ptr}` (16 bytes). It is sparse
by construction — a set with one member costs one node — which is why c2 can
afford a bitset per candidate over a 357-entry register space and a bitset per
block over an unbounded candidate space.

| address | operation |
|---|---|
| `0x10b26ecd(nchunks)` | new set (arg is a chunk-count hint stored at `set+4`) |
| `0x10b26eda(s,i)` / `0x10b26efb(s,i)` / `0x10b26f37(s,i)` | set / clear / test bit `i` |
| `0x10b27290(s)` | iterate next member; `0xffffffff` ends. **Keeps its cursor in globals** (`0x10c2e1f0/f8/fc`) — pass `NULL` to continue, so it is not re-entrant |
| `0x10b2712f(d,s)` | `d := s` |
| `0x10b27091(d,s)` | `d &= s` |
| `0x10b27474(d,s)` | `d \|= s` |
| `0x10b271a5(d,s)` | `d &= ~s` |
| `0x10b271f7(d,s,k)` | `d := s ∖ k` |
| **`0x10b275d8(d,s,k,g)`** | **`d := g ∪ (s ∖ k)`** — the dataflow transfer, one call |
| `0x10b270fc(a,b)` | `a == b` (exact, node by node) |
| `0x10b2733d(a,b)` | `a ∩ b ≠ ∅` |
| `0x10b26ca4(s)` | cardinality |

---

## 2. Question 1 — what is the interference representation?

**There are two structures, and only one of them is persistent.**

### 2.1 The persistent one: a per-candidate ALLOWED-REGISTER bitset, `cand+0x20`

`FUN_10b54d32` @ **`0x10b54d32`**, the candidate constructor:

```
cand              = alloc(0x48)                      /* 72 bytes            */
cand->kind        = 2                                /* byte at cand+4      */
cand->id          = DAT_10c400d4++                   /* cand+0x1c, monotonic */
class             = DAT_10b022cc[ type_nibble ]      /* the #1788 nibble    */
cand->allowed     = new_set();  copy(cand->allowed, DAT_10c3d024[class])
add(DAT_10c400d8[class], cand->id)
hash_insert(cand)                                    /* 0x10b2c206          */
```

So every candidate starts with **the whole register class allowed** —
`DAT_10c3d024[class]` is the class's physical-register bitset — and allocation
is a process of *removing* registers, never of adding them. `FUN_10b2d630` @
**`0x10b2d630`** does the removing, in one **forward** walk of the blocks and,
inside each block, of the tuple list:

* it seeds a running **live list** from the block's live-in set, threading
  candidates through `cand+0x14`;
* for every **physical-register** operand (symbol kind 1) it clears that
  register from `cand->allowed` for **every candidate currently on the live
  list**;
* for every **register-set operand** (operand kind `0x0b`) it does
  `cand->allowed &= ~operand->set` for every live candidate — one call to
  `0x10b271a5` per live candidate. **This is the call-clobber path** (§6);
* it also unions in the registers already taken by previously-coloured
  candidates (seeded from `blk+0x40`, the block's live-in *register* set).

The granularity is therefore **per candidate**, and the alphabet of the bitset
is **physical register numbers** (`0`…`356`, the `0x10b181c0` table
`wb-regalloc` §2 decoded).

**`cand->allowed` empty is the spill trigger.** The driver's `*(int *)piVar7[8]
== 0` test at `0x10b31c9a` routes such a candidate to `FUN_10b31544` instead of
the selector. `FUN_10b2e7f8` frees `cand->allowed` and nulls it the moment a
colour is chosen (`0x10b2e7f8`, after the selection loop) — the allowed set is
the *uncoloured* state, and its absence is the coloured state.

### 2.2 The non-persistent one: the neighbour set is RECOMPUTED, per candidate

There is **no adjacency matrix and no persistent neighbour list**. The set of
candidates that interfere with the one about to be coloured is produced fresh,
by `FUN_10b30517` @ **`0x10b30517`**, immediately before each call to the
selector:

```
puVar5 = new_set();
puVar6 = FUN_10b30517(fn, cand, class, class_cands, puVar5, coloured, nregs);
FUN_10b2e7f8(cand, class, puVar5);          /* the selector, #1821          */
```

`FUN_10b30517` walks the blocks from `cand+0x28` (the candidate's **first**
block) forward, and inside each block walks the tuple list **backwards** from
`blk+0x20`, maintaining a live set seeded from that block's live-out; every
other candidate it finds live at a point where `cand` is live goes into
`param_5`. `param_5` is the selector's `param_3`.

**This is the answer to "what is the interference representation": a live-range
recomputation, not a graph.** It is `O(live range)` per colouring rather than
`O(V²)` storage, and it is why `color.c` can be 7 100 lines without an
adjacency structure anywhere in it.

> **Consequence for a port.** A port does **not** need to build an interference
> graph to reproduce c2's choice. It needs (a) a per-value set of still-allowed
> physical registers, narrowed by every physical def and every clobber list its
> value spans, and (b) the ability to enumerate the values live at the points a
> given value is live. (b) is only needed for the *cost* term, and §5 shows the
> cost term is inert on every shape this project has graded.

---

## 3. Question 2 — how is liveness computed?

**A dataflow fixpoint over basic blocks. Not a linear scan.**

`FUN_10b54904` @ **`0x10b54904`**, in full:

```
for each block b (walking blk+4, the REVERSE layout chain):
      b->live_in  = dup(b->use)          /* b+0x48 := dup(b+0x30) */
      b->live_out = new empty set        /* b+0x4c                */

do {
    changed = false;
    for each block b, starting at the SECOND entry of the reverse chain:
        t = {}
        for each edge e in b->succs:             /* b+0x0c, chained e+0    */
            t |= target(e)->live_in              /* e+0x0c -> blk+0x48     */
        if (t != b->live_out) {                  /* 0x10b270fc, exact      */
            b->live_out = t
            b->live_in  = b->use ∪ (t ∖ b->def)  /* 0x10b275d8, one call   */
            changed = true
        }
} while (changed);
```

* **Direction: backward.** `live_out(b) = ∪_{s ∈ succ(b)} live_in(s)`.
* **Transfer: the textbook one.** `live_in(b) = use(b) ∪ (live_out(b) ∖ def(b))`
  — `0x10b275d8` is literally `d := g ∪ (s ∖ k)` and is called with
  `g = b+0x30` (upward-exposed uses) and `k = b+0x34` (defs).
* **Iteration order: reverse layout order.** Blocks carry two chains —
  `blk+0` = next in layout order (what `FUN_10b2d630` and the selector's
  projection loop walk) and `blk+4` = previous. This fixpoint walks `blk+4`,
  i.e. **exit to entry**, which is the right order for a backward problem.
* **Iteration strategy: round-robin until stable. THERE IS NO WORKLIST.** The
  outer `do { … } while (changed)` re-sweeps every block whenever any block
  moved. This was registered at `p = 0.45` *against* my instinct (P2.3) and it
  is the pessimistic answer that is correct.
* **The exit block is skipped** — the sweep starts at the second entry of the
  reverse chain, whose `live_out` is `∅` by construction.
* **Convergence test is exact set equality on `live_out`** (`0x10b270fc`
  compares node base and both words), not a dirty bit.

### 3.1 `gen`/`kill`, and the second fixpoint

`use`/`def` are built by `FUN_10c207ec` @ **`0x10c207ec`**, a **backward** walk
of each block's tuple list (`blk+0x20` back to `blk+0x1c` via `tuple+0x10`),
maintaining three sets per block: seen, upward-exposed-use, and killed. It
handles partially-overlapping symbols explicitly — the `sym+0x20`/`sym+0x24`
size-and-offset overlap test at `0x10c20a05` — so a store to one field of an
aggregate does not kill a neighbouring one.

`FUN_10b54848` @ **`0x10b54848`** is the *same code shape in the other
direction*: edges from `blk+8` (predecessors), layout order via `blk+0`,
`in(b) = ∪_{p} out(p)`, `out(b) = gen ∪ (in ∖ kill)`, round-robin until stable.
It is an **availability** problem, and `FUN_10b549b5` @ `0x10b549b5` then does

```
b->live_in  &= b->avail_in
b->live_out &= b->avail_out
```

so **a value is live only where it is also available** — c2 will not carry a
range backwards past the point where nothing has defined it. A port that
implements liveness alone gets ranges c2 does not have.

`FUN_10c20f79` @ **`0x10c20f79`**, the first call of the allocator driver, runs
`{ gen/kill; backward; forward; intersect }` **twice** (with `FUN_10c20b3f`
between the two runs), then binds `cand+0x28` / `cand+0x2c` — the first and
last block in which each candidate is live — by iterating
`blk+0x50 = blk+0x48 ∪ blk+0x4c`.

### 3.2 The block's five allocator sets

Read off the selector's projection loop at `0x10b2e7f8` (which, on colouring a
candidate, copies the candidate fact into the register world):

| offset | alphabet | meaning |
|---|---|---|
| `blk+0x48` | candidate ids | **live-in** |
| `blk+0x4c` | candidate ids | **live-out** |
| `blk+0x50` | candidate ids | live somewhere in the block |
| `blk+0x40` | register numbers | registers live-in — the projection of `+0x48` |
| `blk+0x44` | register numbers | registers live-out — the projection of `+0x4c` |
| `blk+0x54` | register numbers | registers used in the block — the projection of `+0x50` |

**The two worlds are kept in lockstep, and that is the whole mechanism by which
an already-coloured candidate becomes interference for a later one.** The
selector's last act is:

```
for blk from cand->first_block to cand->last_block:
    if (cand->id ∈ blk+0x50) {
        blk+0x54 |= {reg}
        if (cand->id ∈ blk+0x48) blk+0x40 |= {reg}
        if (cand->id ∈ blk+0x4c) blk+0x44 |= {reg}
    }
```

---

## 4. Question 3 — what is a "value" to this allocator?

**A symbol of kind 2, minted by c2, carrying a globally unique integer id.**

* `*(char*)(sym+4)` is the symbol kind. **1 = a physical register** (its
  `*(sym+8)` is the per-register descriptor at `0x10c2f088 + 0x60·n`, and
  `descriptor+0x1c` is the register number). **2 = an allocation candidate.**
  **3 = a memory symbol.** Anything else is an ICE at `0x10b2d5ff`.
* A candidate's id is `cand+0x1c`, taken from the monotonic counter
  `DAT_10c400d4` at `0x10b54d32`. Ids are **dense and function-scoped** — the
  hash `id & 0x3ff` at `0x10b2c21d` and the table clear at `0x10b2c1f1` are
  per-function.
* **Candidates are not IL tokens and not `.sy` locals.** They are created in
  `globregs.c` (`FUN_10b55dbe`, `FUN_10b5673e` → `0x10b54d32`) by the
  global-register-promotion renamer `FUN_10b55732` @ `0x10b55732`, which walks
  the blocks, renames each definition, and **inserts merge candidates at join
  points** (`FUN_10b54c07` on the join path). Two more sites create them
  *inside* the allocator — `FUN_10b2dfe2` and `FUN_10b2e4ae`, the split/spill
  paths — so the candidate set is not fixed before colouring.
* Physical registers and candidates **do not share an id space**; they share a
  *representation* (the same bitset library) and are related only by the
  projection in §3.2.

> **What a port-side IR must therefore be able to name.** Not "the IL token".
> A value with (i) an identity independent of any IL token, because c2 mints
> merge values at joins and split values during allocation; (ii) a live range
> expressed over an ordered block list, with a first and last block; and (iii)
> a set of physical registers it may still take. **Machine registers must be
> nameable as operands of the same instruction stream** — the clobber list is
> an *operand* (kind `0x0b`), not a side table.

---

## 5. Question 4 — what feeds the cost penalties, and a CORRECTION to #1821

Read off `0x10b2e7f8` and `0x10b2d630` together. Writing `cost[]` for the
`0x594`-byte array at `0x10c435e8`:

```
memset(cost, 0, 0x594)

/* PENALTIES — over the interference set param_3 */
for id in param_3:
    nb = cand_by_id(id)                              /* 0x10b2c21d          */
    if (nb->benefit > 0):                            /* nb+0x40             */
        for p in nb->preferences:                    /* nb+0x38, {next,sym,w} */
            if (p->sym->id ∈ nb->allowed):
                cost[p->sym->id] += p->weight
        if (|nb->allowed| == 1):
            cost[the single register] += 100 * nb->benefit

/* PREFERENCES — over this candidate's own list */
for p in this->preferences:
    cost[p->sym->id] -= p->weight

FUN_10bfc02f(class, this)      /* MD hook; class 5 (VMX) only, -QVMX128 only */

/* SELECT — the ordered image array, strict < */
best = none
for (i = 0; list[i] != 0; i++)
    if (list[i] ∈ this->allowed && (best == none || cost[list[i]] < best_cost))
        best = list[i], best_cost = cost[best]
this->reg = &DAT_10c2f088 + best * 0x60
```

**Four terms, and their sources:**

1. **`cand+0x40` — the benefit (a.k.a. spill cost).** Accumulated in
   `FUN_10b2d630`: for every reference of the candidate,
   `benefit += access_cost(operand) × weight(block)`. A reference that is
   *itself* a store to memory (opcode `0x2b8`) **subtracts** — keeping it in a
   register buys nothing there.
2. **`weight(block)` is an execution-frequency estimate and it is
   `1 << block->depth`** — `local_18 = 1 << (*(byte *)(blk + 0xba) & 0x1f)` at
   `0x10b2d630`. So a reference at loop depth 3 counts 8×. **Under POGO
   (`DAT_10c3de20 == 2`) it is a real profile count** from
   `FUN_10ba56b3(blk+0xc8, blk+0xcc)`; with `DAT_10c2e310 == 0` it is a flat 1.
3. **The preference list `cand+0x38`** is a list of `{next, symbol, weight}`
   naming **physical registers**. It is where the calling convention enters as
   a *preference* — it is what makes an argument stay in its argument register
   (`wb-regalloc` N1–N4).
4. **The calling convention also enters as hard interference**, not as cost —
   see §6. Those two are different mechanisms and only one of them is in
   `cost[]`.

> ### ⚠ CORRECTION TO BOARD #1821
>
> #1821 records the second penalty as **"`cost[that register] += 100 × degree`
> — a large *do not take the only register a constrained neighbour can still
> use* term"**. The *reading of the term's purpose is right*; the **multiplier
> is not the degree**. `*(int *)(iVar8 + 0x40)` is the neighbour's
> **benefit/spill-cost accumulator** from §5.1, not a neighbour count — there
> is no degree counter anywhere in the candidate record (`0x48` bytes,
> enumerated: `+0x00` type, `+0x04` kind, `+0x05`/`+0x06` flags, `+0x0c` and
> `+0x18` cost accumulators, `+0x10` assigned register descriptor, `+0x14`
> live-list link, `+0x1c` id, `+0x20` allowed set, `+0x24` ref count,
> `+0x28`/`+0x2c` first/last block, `+0x30` hash chain, `+0x34` a byte counter,
> `+0x38` preference list, `+0x3c` last defining tuple, `+0x40` benefit).
>
> #1821's first clause — *"for each already-coloured interfering neighbour,
> `cost[reg] += weight`"* — is also imprecise in a way that matters to a port.
> The weight comes from the **neighbour's own preference list**, and is added
> only for registers the neighbour **can still take**. The rule is *"do not
> take a register a live neighbour wants"*, not *"do not take a register a live
> neighbour has"* — a neighbour that is already coloured has `allowed == NULL`
> (freed at `0x10b2e7f8`) and contributes **nothing**.
>
> **Neither correction is obj-separable on any shape this project can build**,
> and that is stated so absence does not read as coverage: on all ten cells of
> this lane's grid and all fifteen of `wb-regalloc`'s, every candidate's cost
> array is uniformly zero over its allowed set and **the answer is decided
> entirely by the list order**. The cost function is *inert* at these widths.
> The grey-zone rule of `DISCLOSURE.md` applies and no row is proposed.

---

## 6. Question 5 — where callee-saved registers enter, and §6.2 item F's two cases

**There is no callee-saved preference, no callee-saved policy, and no "this
function needs a frame" decision that precedes allocation.** Both of item F's
measured cases fall out of §2 plus the fixed order.

### 6.1 The mechanism: a call's clobber list is an OPERAND

A call tuple carries a source operand of **kind `0x0b`** whose payload
(`operand+0x18`) is a **bitset of physical registers**. Two consumers, and they
are the whole story:

* `0x10b2d630`: `for each live candidate c: c->allowed &= ~operand->set`
  (one `0x10b271a5` per live candidate);
* `0x10b2ceb7`: the same set is counted as pressure and unioned into
  `blk+0x54`.

So a candidate live across a call loses **every volatile at once**. Its
`allowed` set becomes exactly the callee-saved tail of the class, and the
selector — walking `r11, r10, …, r3, r31, r30, …, r14` and taking the first
allowed entry at equal cost — returns **`r31`**, then `r30`, then `r29`.
`#1820`'s order is doing all the work; nothing prefers callee-saved registers.

**And the frame is a consequence, not a cause.** The driver runs
`FUN_10bfc477` and `FUN_10bfdf47` *after* the whole per-class colouring loop
(`0x10b31c9a`), and `wb-frame`'s prologue-flag scan `0x10bff507` computes the
save set by scanning the finished instruction list. A body is framed **because**
the allocator took a callee-saved register.

> **The grid proves the causal direction on a body with no call in it at all.**
> `wbl_v3` is a leaf. It takes `r31`, `r30`, `r29`, `r28` — purely because nine
> volatiles were already occupied — and is framed with `bl __savegprlr_28`.
> Register *pressure*, with no calling convention involved, produces a frame.
> A port that decides framing from the call graph has the dependency backwards.

### 6.2 Item F case 1 — the entry-block copy (`MemFree`'s `v2` r4 → r11)

`CFG_SHAPE.md` §6.2 F: *"`MemFree` copies `v2` from r4 to **r11** in the entry
block because both successors need it after clobbering r4."*

The mechanism: `v2` arrives precoloured in `r4`; its range is live out of the
entry block; something on the far side defines `r4`; `0x10b2d630` clears `r4`
from `v2`'s allowed set while `v2` is on the live list; the selector then takes
the head of the order at cost 0, which is `r11`; and the copy exists because
the value's *arrival* register and its *chosen* register differ. **Nothing is
special about the entry block** — the copy lands wherever the arrival is.

`wbl_x4` is this case with the clobber supplied by a call rather than a bare
`r4` def, and it reproduces the *shape* exactly (§7): `mr 31,3` at offset
`0x10`, **before** the `cmpwi`/`bf`, i.e. in the entry block, with both
successors clobbering the volatiles and joining at the return. What the grid
does **not** establish is the `r11` half — no cell of this grid produces a bare
physical-register def with no call, so *"a non-call physical def narrows the
allowed set"* is **disassembly-only** and is recorded as such.

### 6.3 Item F case 2 — formals held in `r31`/`r30` across calls, and why those bodies are framed

Directly §6.1, and the grid grades it on four cells (`wbl_x1`, `wbl_x2`,
`wbl_x5`, `wbl_x6`) against a negative control (`wbl_x3`) that could have gone
red and did not. See §7.

---

## 7. THE OBJ CHECK

Compiled with real `cl.exe` 16.00.11886.00 under wibo,
`/nologo /c /GR /O1 /Oi /EHsc`, one obj, 22 sections, 102 symbols, 5 466 bytes.
Source: [`grids/wb-live/live_grid.cpp`](grids/wb-live/live_grid.cpp),
`sha256 fc1d42d9…` as frozen in `WB_LIVE_PREREG_R2.md`. The dump is
`work/wb-live/run/grid_wl.txt` (not committed — it is a dump of an obj);
reproduce with `scripts/gt_capture.sh` + `scripts/gt_dump.py --text-only`.

### 7.1 Results against the two models

| cell | emitted | L0 (this lane) | I0 (incumbent) |
|---|---|---|---|
| **V1** | `r11,r10,r9,r8,r7,r6` — six distinct, **no reuse** | **✗ MISS** (§7.3) | ✅ |
| **V3** | `r11…r3` **each reused up to 3×**, then `r31,r30,r29,r28`, `bl __savegprlr_28`, framed | **✗ MISS** on the prediction; ✅ on reuse (§7.3) | **✗ REFUTED** — I0 forbids reuse and r11 holds three values |
| **P2** *(positive control)* | `r11,r10,r9` | ✅ | ✅ *(by design)* |
| **X1** | `mr 31,3`; `std 31,-16(1)`; framed | ✅ | **✗ REFUTED** |
| **X2** | `mr 30,3` / `mr 31,4`; `std 30`,`std 31` | ✅ *(set)* | **✗ REFUTED** |
| **X5** | `mr 30,3` / `mr 31,4` / `mr 29,5`; `bl __savegprlr_29` | ✅ | **✗ REFUTED** |
| **X3** *(negative control)* | `mflr 12`/`stw 12,-8(1)`/`stwu` and **no GPR saved** | ✅ | — |
| **X4** | `mr 31,3` at `+0x10`, **before** the `cmpwi`/`bf` | ✅ | **✗ REFUTED** |
| **X6** | `mr 31,3` in the entry block; `r31` taken | ✅ | **✗ REFUTED** |
| **R1** | `lis 11` … `bl wbl_void` … **`lis 11` again**; no GPR saved | ✅ | **✗ REFUTED** |

| model | verdict |
|---|---|
| **I0 — positional, temps descend from `r11` in emission order, never reused** | **REFUTED on 7 cells** (V3, X1, X2, X5, X4, X6, R1). `r11` demonstrably holds three different values inside one straight-line block with no call between them (`wbl_v3` `+0x44`/`+0x68`/`+0x94`), and a formal demonstrably leaves `r3` for `r31` when a call clobbers it |
| **L0 — allowed-set narrowing + fixed order** | **SURVIVES on 7 cells** (X1, X2, X5, X3, X4, X6, R1) and on V3's reuse; **its V1 and V3 predictions are MISSES and are scored as such** |

### 7.2 The non-rival predictions

| # | prediction | emitted | verdict |
|---|---|---|---|
| F1 | `r12` only as the LR shuttle | true in all 10 cells; no other `r12` reference exists in the obj | **HIT** |
| F2 | `r13` never appears | true in all 10 cells | **HIT** |
| F3 | callee-saved taken from the **top** | `r31`,`r30`,`r29`,`r28`; `r14`–`r16` never | **HIT** |
| F4 | X1/X2/X5 save exactly 1/2/3 GPRs | `{r31}`, `{r30,r31}`, `{r29,r30,r31}` | **HIT** |
| F5 | both flag modes byte-identical | every code section byte-identical (obj sizes 5 466 vs 5 470 — a non-code difference) | **HIT** |

### 7.3 THE MISS, stated as a miss

**V1 and V3's predictions are wrong, and the reason is not the allocator.** I
predicted that three (and eight) statements with independent sources and sinks
give three (and eight) *sequential* live ranges, so one register would serve
them all. c2 emitted six live-at-once address registers for V1 and exhausted
the volatiles for V3.

The cause is visible in the obj: **c2 hoists every `lis <sym>@ha` to the top of
the block**, ahead of every `lwz`. In `wbl_v1` all six `lis` come out at
`+0x00`…`+0x18` before the first `lwz`. That makes all six address values
simultaneously live, so they interfere, so they get six registers — *by the
same rule*. My model of the allocator was not what failed; my model of the
**live ranges** was.

This is `wb-regalloc` §7.3's failure mode repeating one level down, and it is
scored the same way: **L0 the rule is not refuted by V1/V3; L0-as-I-applied-it
is.** Per method doc §7 the cells are **not** re-scored in L0's favour. They
keep their MISS and L0 keeps only the seven cells it was right about in
advance.

**What the miss buys, and it is the most useful thing in this section.** The
liveness a port must compute is over **c2's emission order, not the source's
statement order**, and c2's emission order hoists. `R1` is the control that
isolates it: the *same* two-statement shape with a call between the statements
**does** reuse `r11`, because nothing can be hoisted across the call. So:

> **Live ranges are a property of the lowered instruction order, and the
> lowered order is not the source order.** `wb-regalloc` §4 already said the
> emitted order is the lowering order and that it did **not** read `dag.c`'s
> tree-to-tuple walk. That unread walk is now on the critical path for item F,
> because it determines the ranges the allocator sees.

### 7.4 The controls, and whether they could have gone red

| control | could it have gone red? | did it? |
|---|---|---|
| **P2** — three values live at once must show three registers | yes: if the dumper or the mode were wrong it would show one | no |
| **X3** — a non-leaf with nothing crossing the call must save **no** GPR | yes, and this is the one that makes X1 mean anything: if X3 had saved `r31`, "framed non-leaf bodies save GPRs" would explain X1 with no liveness at all | no |
| **I0 on V3** — a leaf body with eight independent statements | yes, and **it went red**: I0 predicts no reuse, and `r11` holds three values | **yes** |
| **L0 on V1/V3** | yes — **and it went red** (§7.3) | **yes** |
| **R1 vs V1** — registered in `PREREG_R2` §5 as the discriminator that would show the reuse is not live-range-driven | yes | fired, and §7.3 is the resolution |

**Two of the five controls fired.** Neither was adjusted after the fact.

---

## 8. PREREG score

`H` hit · `M` miss · `U` unscoreable · `N` not established by this lane
(navigation only, stated so absence does not read as coverage).

| # | p | prediction | verdict | note |
|---|---:|---|---|---|
| P0.1 | 0.75 | floor: representation named with an address + ≥1 surviving obj claim | **H** | §2, §7 |
| P0.2 | — | decline floor: ≥3 of 5 questions address-backed with a red-capable cell | **cleared** | 5 of 5 address-backed; Q1/Q3/Q5 have red-capable cells, Q2/Q4 do not — declared in §10 |
| P0.3 | — | ≥3 discriminating cells or declare insufficient | **cleared** | 7 cells separate L0 from I0 |
| P0.4 | 0.85 | no `crates/` change, no DISCLOSURE row | **H** | |
| P1.1 | 0.70 | bitset per node, not an edge list / triangular matrix | **H** | `cand+0x20`, `0x10b54d32` |
| P1.2 | 0.80 | a degree count is kept alongside | **M** | **there is no degree anywhere**; §5's correction to #1821 |
| P1.3 | 0.65 | nodes are synthesized live ranges, not IL tokens or `.sy` locals | **H** | §4 |
| P1.4 | 0.85 | per-function arena bitsets, not a fixed image array | **H** | sparse chunk lists, §1.1 |
| P1.5 | 0.75 | machine registers are nodes in the same graph | **M** *(partially)* | they are in the same *representation* and reach the candidate through `cand->allowed`, but they are **not** nodes in a shared id space; the projection in §3.2 is the link |
| P2.1 | 0.80 | dataflow fixpoint over blocks, not a linear scan | **H** | `0x10b54904` |
| P2.2 | 0.70 | `live_in = use ∪ (live_out ∖ def)`, `live_out = ∪ live_in(succ)` | **H** | `0x10b275d8` is that formula in one call |
| P2.3 | 0.45 | a **worklist**, not round-robin | **M** *(registered pessimistic, and the pessimistic answer is right)* | round-robin `do{}while(changed)` |
| P2.4 | 0.55 | iteration in reverse block-construction order | **H** | the `blk+4` chain, exit-to-entry |
| P2.5 | 0.75 | interference added by a **second** walk maintaining a running live set | **H** | `0x10b2d630` forward, `0x10b30517` backward |
| P2.6 | 0.60 | the construction lives in `color.c`, no `live.c` TU | **M** | **no `live.c`** (right) but the fixpoint is in **`globregs.c`** and gen/kill in the `0x10c20f79` band — not `color.c` |
| P2.7 | 0.70 | build → colour → spill → rebuild outer loop | **H** | `0x10b31c9a`'s `while (DAT_10c43b7c)` with the `local_8` spill restart |
| P3.1 | 0.70 | keyed on a symbol-table entry, not a tuple index | **H** | kind-2 symbol, `0x10b54d32` |
| P3.2 | 0.85 | a superset of the `.sy` locals — c2 mints temporaries | **H** | §4 |
| P3.3 | 0.80 | a port IR must name its own temporaries as values with live ranges | **H** | §4, §9 |
| P3.4 | 0.50 | splitting exists but does not fire at `/O1` on graded bodies | **U** | premise not exercised — no cell reaches the split path; **not established** |
| P4.1 | 0.65 | the per-neighbour weight is not the constant 1 | **H** | it is the neighbour's preference weight |
| P4.2 | 0.70 | a frequency / loop-depth term exists | **H** | `1 << blk+0xba`, and a POGO profile count |
| P4.3 | 0.85 | the preference list is the ABI-argument mechanism | **H** | `cand+0x38` |
| P4.4 | 0.80 | the calling convention enters as **interference**, not as a constraint list | **H** | the kind-`0x0b` clobber operand, §6.1 |
| P4.5 | 0.60 | a separate spill-cost field, distinct from the selector's cost array | **H** | `cand+0x40` vs `0x10c435e8` |
| P5.1 | 0.85 | `r31`/`r30` fall out of "volatiles excluded" + list order; no callee-saved preference | **H** | §6.1, X1/X2/X5 |
| P5.2 | 0.80 | framing is a consequence of allocation, not a cause | **H** | and `wbl_v3` proves it on a **leaf** |
| P5.3 | 0.70 | the `r4 → r11` case needs no new mechanism | **N** | the reading is §6.2; **no cell of this grid produces a non-call physical def**, so it is disassembly-only |
| P5.4 | 0.55 | the copy is emitted by lowering, not inserted by a coalescer | **N** | not read |
| P5.5 | 0.70 | nothing is specific to entry blocks | **H** *(weakly)* | `0x10b2d630` has no entry-block case; X4 puts the copy in the entry block because that is where the arrival is |
| P6.1 | 0.85 | three non-overlapping temps take `r11` three times | **M** | §7.3 — they were not non-overlapping |
| P6.2 | 0.90 | a value live across a call takes `r31` | **H** | X1 |
| P6.3 | 0.80 | overlap count, not temp count, sets the high-water mark | **H** *(in the direction that hurt me)* | V1/V3 confirm it by having **more** overlap than I predicted |
| P6.4 | 0.85 | I0 goes red on ≥3 cells | **H** | 7 cells |
| P6.5 | 0.70 | the grid finds ≥1 fact the disassembly did not predict, reported as a miss | **H** | the `lis` hoist, §7.3 |
| P7.1 | 0.80 | item F becomes specifiable but not buildable; workload reach 0 | **H** | §9 |
| P7.2 | 0.75 | the IR requirement is live intervals + nameable machine registers | **H** | §4, §9 |
| P7.3 | 0.85 | the binding constraint remains the IL reader | **H** | unchanged; §9.3 |
| P7.4 | 0.65 | the spiller is out of scope and is declared so | **H** | §10 |

**Score: 29 H · 5 M · 1 U · 2 N** (of 42).

**Calibration by registered band** — this is the line board #770's streak is
about:

| registered `p` | n | hits | note |
|---|---:|---:|---|
| `p ≥ 0.80` | 16 | 14 | misses: P1.2 (0.80), P6.1 (0.85) |
| `0.60 ≤ p < 0.80` | 17 | 13 | misses: P2.6, P5.3(N), P3.4(U), P5.4(N) |
| `p ≤ 0.55` | 4 | 2 | misses: P2.3 (0.45), and P5.4 (0.55) unread |

The two misses above `0.80` are the informative ones: **P1.2's degree counter
does not exist** (and that is what produced §5's correction to a published
board row), and **P6.1's reuse prediction was right about the allocator and
wrong about the live ranges**. Both are optimistic. Adding to #770's tally:
**2 optimistic (P1.2, P6.1), 1 pessimistic-and-correct (P2.3).**

---

## 9. THE JUDGMENT — is `CFG_SHAPE.md` §6.2 item F buildable now?

### 9.1 The answer

> **The dependency §6.2 named is discharged. Item F is now SPECIFIABLE and is
> still not BUILDABLE, and the thing standing between them moved: it is no
> longer "nobody has characterized the allocator", it is "nobody has
> characterized the lowering order that fixes the live ranges."**

§6.2 says item F *"depends on work nobody has done, not merely on work nobody
has scheduled"*, citing `CODEGEN_W6_COMPARE.md` §6's *"demonstrably richer than
a descending counter and **not** characterized"*. That sentence can be
retired. The allocator is characterized: §2 and §3 are a complete, short,
deterministic mechanism, obj-confirmed on seven cells, and the incumbent
positional model is refuted on those same seven.

**What a port must compute, in full:**

```
1.  per block:  use, def                    (backward tuple walk)
2.  backward fixpoint:  live_out = U live_in(succ);  live_in = use U (live_out \ def)
    round-robin over blocks in reverse layout order until live_out stops moving
3.  forward fixpoint on the same shape for availability;  live &= avail
4.  per value:  allowed := all registers of its class
5.  forward walk;  for each physical def or clobber-set operand:
        for every value live at that point:  allowed &= ~{those registers}
6.  colour in the driver's order:  first r in [r11,r10,...,r3,r31,r30,...,r14]
    that is in allowed
7.  after colouring:  a value that arrived in a register other than the one it
    was given needs a copy at its arrival; the set of callee-saved registers
    coloured is the prologue's save set
```

Steps 4–6 are ~30 lines. Steps 1–3 are the textbook. **Step 0 — producing the
instruction order the ranges are measured over — is the unread one**, and §7.3
is the receipt: a three-statement body whose statements look independent has
six simultaneously-live values because c2 hoists all six address
materializations to the top of the block.

### 9.2 What still blocks item F, ranked

1. **The lowering order (`dag.c`'s tree-to-tuple walk, `0x10b3219f`).** Unread
   by `wb-regalloc` (its §4 says so) and unread here. Without it a port cannot
   compute the live ranges, and with the wrong ranges a *perfect* allocator
   gives wrong registers — V1 is that failure in miniature, committed in
   advance.
2. **Items A–E and G of §6.2** — blocks, terminators, labels, fixups, branch
   forms, condition codes. Item F is the *seventh* thing the new IR needs and
   the other six are still absent. A live-range model with nothing to hang it
   on converts nothing.
3. **The reader.** `wb-regalloc` P5.4 and this lane's P7.3: 48 of the
   frontier's 59 functions die in the IL parser before any emitter question is
   asked. Unchanged.

**What is NOT blocking, and should not be re-priced as if it were:** the
interference graph (there isn't one — §2.2), the cost function (inert at these
widths — §5), the spiller (`wb-regalloc` P2.7 and this lane's V3: c2 takes four
callee-saved registers rather than spill one value), and a callee-saved policy
(there is none — §6.1).

### 9.3 Predicted reach

**Zero, as registered.** Nothing in this lane is a `crates/` change and nothing
in it can be one until items A–E exist. This is `#1829`/`#1921`'s shape and it
is priced as infrastructure, which is what a characterization lane is.

---

## 10. What this lane did NOT establish

Stated so absence does not read as coverage.

* **The cost function has no obj support at all.** §5's four terms and both
  corrections to #1821 are disassembly-only; the cost array is uniformly zero
  over the allowed set on all 25 cells this project has now compiled for this
  question. A lane that ships the cost function inherits this paragraph.
* **The non-call physical-register def** (§6.2's `r4` case, P5.3) has no cell.
* **The spiller and the live-range splitter** were not read. `FUN_10b31544`,
  `FUN_10b2dfe2`, `FUN_10b2e4ae`, `FUN_10b3032a` are named and not opened.
* **`dag.c`'s lowering order** was not read, and §9.2 says it is now the
  binding item.
* **The globregs renamer `0x10b55732`** was read only far enough to establish
  that it mints candidates and inserts merge candidates at joins. Its promotion
  policy — *which* symbols become candidates — is not characterized, and a port
  that gets that wrong has the wrong value set before any of §9.1 applies.
* **The `wbl_x2` assignment order is unexplained**: `a` took `r30` and `b` took
  `r31`, i.e. the *second* formal got the head of the callee-saved run. #1821's
  tie-break predicts the first candidate coloured takes `r31`; which candidate
  is coloured first is set by the driver's worklist order, which this lane did
  not read. Recorded as an open fact, not smoothed over.

## 11. Pre-drafted DISCLOSURE rows — NONE

Per `DISCLOSURE.md` step 5 the black-box alternative is preferred. For the two
facts a port would actually adopt, it **exists and is sufficient**:

* *values whose live ranges do not overlap share a register, and a value live
  across a call is excluded from every volatile* is re-derivable from
  `grids/wb-live/live_grid.cpp` alone — cells V3, X1, X2, X5, X3, R1 exhibit it
  against real `c2.dll` with no address;
* *the order `r11,…,r3,r31,…,r14`* already has `W-REGALLOC-1`'s black-box
  re-derivation and this grid adds `r28` to its witnessed span.

What **cannot** be established black-box — the recomputed-rather-than-stored
neighbour set, the transfer function's exact form, the round-robin iteration,
the four cost terms — is also **not adopted by anything**, so no row is
proposed. If a future lane ships the cost function or the availability
intersection, it needs rows naming `0x10b2e7f8`, `0x10b54904`, `0x10b54848`,
`0x10b549b5` and `0x10b2d630` in the same commit.
