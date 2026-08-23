# `P_GLOBREGS` — `globregs.c`: the candidate mint/merge, and the field that decides every tie

> **Reference page.** **`[R]`** read from the disassembly, *not* obj-checked —
> a hypothesis. **`[O]`** confirmed against a real obj or `/FAsc` listing, with
> the witness named. **`[I]`** an interpretive step. Navigation only; nothing
> here may enter `crates/` without a [`DISCLOSURE.md`](../DISCLOSURE.md) row.
> Index: [`ADDR.tsv`](ADDR.tsv) · front door: [`README.md`](README.md) ·
> adjacent: [`P_REGALLOC.md`](P_REGALLOC.md) (the *colouring*; this page is the
> *candidate set and its order*, which is that page's missing input)

Produced by read **R4** (lane `w-read-r4`, 2026-08-23), prereg
[`../WB_GLOBREGS_PREREG.md`](../WB_GLOBREGS_PREREG.md), grade
[`../WB_GLOBREGS_FINDINGS.md`](../WB_GLOBREGS_FINDINGS.md), board
**#3411**–**#3414**. Spec:
[`../READ_PLAN_2026-08-21.md`](../READ_PLAN_2026-08-21.md) §3 row R4 and §5.2.
Raw listings for every function named here, both objdump and Ghidra, are
committed under [`../labels/globregs/`](../labels/globregs/) and regenerate
with `docs/whitebox/scripts/dump_globregs.py` (digest-fenced).

**Coverage: 19 code entries + 9 data entries.** The denominator R4 registered
was *the target plus its 18 callees = 19*; the read went **outside** that
denominator on purpose — twelve of the eighteen callees are the shared bitset
library and carry no policy, while the three functions that actually decide
the order (`0x10b550e5`, `0x10b55dbe`, `0x10b55eae`) are **not** callees of the
target at all. The honest coverage statement is therefore: **6 of 18 callees
read to policy level, plus 7 functions outside the target's subtree that the
target's own deliverable turned out to depend on.** Not covered: the twelve
bitset primitives; `regasg.c`; the POGO variants.

---

## 0. THE HEADLINE — the tie tier is **not** a hash-bucket walk, and the read plan's entry point does not contain the answer

Two corrections, both to load-bearing published claims, and both filed against
this lane's own prereg as MISSES.

> **1. `cand+0x44` — the comparator's tie key — is WRITTEN, and it holds a
> program-position ordinal.** The sole originating writer in the image is
> **`0x10b55fac`** in **`FUN_10b55eae`**, and the value is a **tuple-visit
> counter**: zeroed once per function at `0x10b55eb7`, incremented once per
> *real* tuple at `0x10b55f77`, and stored into `cand+0x44` at every
> encounter, so the surviving value is the counter at the candidate's **last
> visit**. [`P_REGALLOC.md`](P_REGALLOC.md) §4 consequence 3's *"on an exact
> tie the order is a hash-bucket walk over a counter"* describes the **third**
> tier, not the second. The bucket walk is only reached when two candidates
> tie on `+0x0c` **and** on `+0x44`. `[R]`, direction `[O]` — §7.
>
> **2. `FUN_10b55732` is not where candidate ids are assigned.** It is the
> **renamer**: it computes a dense per-function *version* numbering and returns
> the count. The **mint** — `id = DAT_10c400d4++` — is in **`FUN_10b55dbe`**
> (240 B), a *sibling* the read plan never names, called by the same driver
> forty bytes later. `READ_PLAN` §3 row R4's *"the candidate mint order … the
> missing input to the already-read comparator"* is wrong twice over: the mint
> order is in a different function, and it is not the comparator's missing
> input. **The missing input was `+0x44`.** §8.

---

## 1. The pipeline, as the ordered algorithm R4 was asked for

Every step is one function; every address is cited in §2. The phase driver is
`FUN_10b57633`, which the per-function back-end driver `FUN_10b7dc51` calls at
`0x10b7dcb7` (read **R1**, `../WB_CANDID_FINDINGS.md` §2).

```
FUN_10b57633                              /* the globregs phase, per function */
  DAT_10c400d4 = 1                        /* 0x10b57676  candidate-id counter  */
  DAT_10c2e3e0 = 0                        /* 0x10b5767c  candidate free list    */

  /* STEP 1 -- INDEX.  FUN_10b550e5 @ 0x10b550e5                              */
  k = <base>
  for each chunk C in symtab->[0x10], in APPEND order:                /* §6.1 */
      for each symbol s at C+0x20, C+0x20+0x60, ... < C+0x04:
          if !promotable(s):  s->[0x34] = 0;  continue                /* §3   */
          for each sub-symbol t in s, s->[0x0c], ...:
              t->[0x34] = alloc(arena 0x0e, 0x20)     /* the aux record */
              t->[0x34]->[0x00] = k++                 /* the LIVENESS BITSET INDEX */
  /* => bitset index order == symbol-arena order.                             */

  /* STEP 2 -- RENAME.  FUN_10b55732 @ 0x10b55732   <-- the read plan's entry */
  /* NB the driver passes (*(proc+0x08))->[0x04] -- the list header's SECOND  */
  /* end -- and advances via B->[0x04] (0x10b577c8, 0x10b55c98).  This is a   */
  /* BACKWARD block walk; see the note under this listing.                    */
  v = 1
  for each block B, BACKWARD (B = B->[0x04]), from (*(proc+8))->[0x04]:
      for each tuple T in B, from B->[0x20] stepping T->[0x10] to B->[0x1c]:
          if (T->[0x09] & 1) == 0: continue           /* not a real instruction */
          for each operand o in T->[0x2c] then T->[0x28]:
              s = o->[0x18]
              if s->[0x34]->[0x14] == 0:
                  v = FUN_10b54bad(s, v)              /* 0x10b54bad, §4        */
      at the join into the next block:
          FUN_10b54c07(list, mergeset, phiset, &v)    /* 0x10b54c07, §5        */
  return v                                            /* the version COUNT     */

  /* STEP 3 -- MINT.  FUN_10b55dbe @ 0x10b55dbe                                */
  map = alloc(arena 7, v * 4)                         /* version -> candidate  */
  for each chunk C in symtab->[0x10], in APPEND order:
      for each symbol s at C+0x20, stride 0x60, < C+0x04:
          if s->[0x04] == 0x10:            continue   /* 0x10b55e50 */
          if s->[0x34] == 0:               continue   /* 0x10b55e56 */
          for each version record r in s->[0x34]->[0x0c], LIST order:
              cand = FUN_10b54d32(s)                  /* 0x10b55e66: id = DAT_10c400d4++ */
              for each version number n in r->[0x04]:  map[n] = cand
              r->[0x04] = cand
  /* => candidate id ascends with (symbol-arena position ASC, version DESC),   */
  /*    because 0x10b54bad HEAD-INSERTS the version list (§4).                 */

  /* STEP 4 -- STAMP THE TIE KEY.  FUN_10b55eae @ 0x10b55eae                   */
  /* starts at (*(proc+0x08))->[0x00] and advances via B->[0x00] -- the OTHER  */
  /* direction from step 2 (0x10b55eb4/0x10b55ebc).                            */
  n = 0                                               /* 0x10b55eb7, once      */
  for each block B, FORWARD (B = B->[0x00]):
      for each tuple T in B, from B->[0x20] stepping T->[0x10] to B->[0x1c]:
          if (T->[0x09] & 1) == 0: continue
          n = n + 1                                   /* 0x10b55f77            */
          for each pseudo-register operand o of T:
              cand = map[o->[0x10]]
              o->[0x1c] = cand                        /* 0x10b55fa9            */
              cand->[0x44] = n                        /* 0x10b55fac  <-- THE KEY */
  /* => cand+0x44 = the ordinal of the candidate's LAST VISIT in this walk.    */
```

Then `FUN_10bfcf7c`, and afterwards `color.c`'s `FUN_10b31c9a` consumes the
result through `FUN_10b316b1`'s worklist build and `FUN_10b2b82d`'s sorted
insert ([`P_REGALLOC.md`](P_REGALLOC.md) §1, §4).

> ## ⚠ The step-2 and step-4 walks run in OPPOSITE block directions
>
> A first draft of this page said they were the same walk. **They are not**,
> and the difference is one field. `*(proc+0x08)` is a **list header** whose
> `+0x00` and `+0x04` are the two ends of the block list; the tuple list is
> the same shape and [`P_DAG.md`](P_DAG.md):113 already reads it as
> **`+0x00` next, `+0x10` prev**.
>
> | | root | block step | tuple step |
> |---|---|---|---|
> | **step 2** `0x10b55732` | `header->[0x04]` (`0x10b577c8`) | `B->[0x04]` (`0x10b55c98`) — **backward** | `T->[0x10]` — **backward** |
> | **step 4** `0x10b55eae` | `header->[0x00]` (`0x10b55ebc`) | `B->[0x00]` — **forward** | `T->[0x10]` — **backward** |
> | `0x10b568af` | `header->[0x00]` | `B->[0x00]` — forward | `T->[0x10]` — backward |
>
> **Step 2 is a fully backward walk, and that is the correct shape for what it
> does**: it is not forward SSA renaming but a **backward live-range
> construction** — the running list `local_8` is the live set, an operand
> encounter ADDS to it, and the arm where the remaining-use nibble
> `o[0x14] & 0xf` reaches 0 (the definition, reached last going backward)
> REMOVES it. The sibling `FUN_10b54904` is already labelled *"the backward
> liveness fixpoint"* in [`P_REGALLOC.md`](P_REGALLOC.md) §2.
>
> **Consequence for the version numbers**: `v` ascends as the walk proceeds
> **backward**, so version 1 goes to the symbol encountered earliest in the
> backward walk, i.e. **latest in program order**. Any port that models the
> numbering as forward-SSA gets the sequence reversed.

---

## 2. Entries

| addr | size | callers | callees | TU | what |
|---|---:|---:|---:|---|---|
| `0x10b57633` | 541 | 1 | 30 | *(gap)* | **the globregs phase driver.** Resets the id counter and the free list; runs steps 1–4 in order; the four resets are read **R1** `[R]` |
| `0x10b550e5` | 490 | 1 | 5 | *(gap after `globopt.c`)* | **STEP 1, the INDEX and the promotion policy.** Walks the symbol arena; allocates the `0x20`-byte aux record at `sym+0x34`; assigns `aux[0x00] = k++`. §3 `[R]` |
| `0x10b568af` | 1129 | 1 | 14 | *(gap)* | fills `DAT_10c400d0[index] = symbol` and the three per-block sets `block+0x2c/+0x30/+0x34`; same walk shape as steps 2 and 4 `[R]` |
| **`0x10b55732`** | **1676** | **1** | **18** | *(gap after `globopt.c`)* | **STEP 2, the RENAMER — item F1, the read plan's entry point.** Blocks in layout order, tuples via `T->[0x10]`; stamps a dense version number per (symbol, definition); returns the count. **Holds no direct call to the mint** `[R]` |
| `0x10b54bad` | 67 | 1 | 3 | *(gap)* | **the version stamp.** `alloc(arena 7, 8)`; `rec[1] = {v}`; **HEAD-inserts** `rec` on `aux[0x0c]` (`0x10b54bdb`/`0x10b54be0`); `aux[0x14] = v`; **returns `v+1`** `[R]` |
| `0x10b54bf0` | 23 | 1 | 0 | *(gap)* | the candidate-list link: `sym+0x30` next, `aux[0x10]` prev `[R]` |
| `0x10b54c07` | 222 | 2 | 5 | *(gap)* | **STEP 2b, the MERGE at joins.** §5 `[R]` |
| **`0x10b55dbe`** | **240** | **1** | **6** | *(gap)* | **STEP 3, the MINT — where candidate ids are actually assigned.** Symbol-arena order × version-list order; `0x10b55e66` calls the constructor. **Named by no document before R4** `[R]` |
| **`0x10b55eae`** | **1468** | **1** | **18** | **`globregs.c` anchor** | **STEP 4 — the sole originating writer of `cand+0x44`** at `0x10b55fac`. Also re-points every pseudo-register operand at its candidate (`0x10b55fa9`). §7 `[R]` |
| `0x10b54d32` | 130 | 6 | 5 | *(gap)* | the candidate constructor. `alloc(0x0e, 0x48)`; `+0x04 = 2`; `+0x1c = DAT_10c400d4++` on the fresh path only. **Writes `+0x44` never** — verified, 0 references `[R]` |
| `0x10b2efd6` | 130 | ~ | ~ | **`color.c`** | the candidate **destructor**: unhooks the hash chain, `memset(cand, 0, 0x48)` at `0x10b2f03b`, pushes on the free list, **restores only `+0x1c`**. This is why a recycled record has `+0x44 == 0` `[R]` |
| `0x10b2dfe2` | 562 | 1 | ~ | **`color.c`** | split path: mints a child and **copies the parent's `+0x44` verbatim** at `0x10b2e159` `[R]` |
| `0x10b2e4ae` | 794 | 1 | ~ | **`color.c`** | split path: two more verbatim `+0x44` inheritances, `0x10b2e665` and `0x10b2e73f` `[R]` |
| `0x10b316b1` | 164 | 1 | 4 | `color.c` gap | the worklist build. Buckets **ascending** `0x10c43b80 → 0x10c44b80` (`0x10b316cd`/`0x10b3173b`), chain `cand+0x30`, class filter `DAT_10b022cc`. **Contains no `+0x44` reference at all** `[R]` |
| `0x10b2b82d` | 126 | 3 | 0 | *(gap)* | **THE COMPARATOR.** `+0x0c` DESC signed, then **`+0x44` DESC unsigned**, `<=` on the tie. Its **six** `+0x44` reads are the field's only reads in the image `[R]` |
| `0x10bd2343` | 230 | ~ | ~ | *(symtab)* | **the symbol-chunk allocator.** `alloc(0, 0xc20)`; **32 slots** of `0x60` from `+0x20`; stamps `slot+0x1c = symtab[0]++`; **APPENDS** the chunk (`symtab+0x10` head, `symtab+0x14` tail) `[R]` |
| `0x10bd3225` | 394 | ~ | ~ | *(symtab)* | one symbol allocation: bump `chunk+0x04` by `0x60`, or take the free list at `symtab+0x30` — which **`memset`s `0x60` and restores only `+0x1c`** `[R]` |
| `0x10bd7d24` | 13 | 23 | 1 | *(types)* | **the promotable-type gate.** `mov al, [eax*4 + 0x10b18b28]` over the class from `0x10bd7c10` §3 `[R]` |
| `0x10c2022a` | 280 | many | 3 | *(arena)* | the arena allocator. **`memset`s every chunk it takes**, so `alloc` returns zeroed memory — this is what makes `+0x44 == 0` a hard default rather than garbage `[R]` |

### 2.1 Data

| addr | what |
|---|---|
| `DAT_10c6f844` | **the symbol table.** `+0x00` next serial, `+0x10` chunk-chain **head**, `+0x14` tail, `+0x30` free list. Chunk: `+0x00` next, `+0x04` high-water, `+0x20` first slot, stride `0x60`, 32 per chunk `[R]` |
| `sym+0x1c` | **the compilation-global symbol serial**, stamped at chunk creation (`0x10bd2372`) and **preserved across recycling** — so it is a property of the arena *slot*, not of creation time `[R]` |
| `sym+0x34` | the `0x20`-byte **aux record**, allocated per function by step 1 and cleared for every non-promotable symbol `[R]` |
| `aux+0x00` | the liveness **bitset index** (`k++`, step 1) `[R]` |
| `aux+0x0c` | the **version list**, HEAD-inserted, hence **descending version number** `[R]` |
| `aux+0x10` | candidate-list prev `[R]` |
| `aux+0x14` | the **current version number** for this symbol (0 = none yet) `[R]` |
| `aux+0x18` | a partner symbol that is versioned alongside (`0x10b55aa2` in step 2) `[R]` |
| `DAT_10c400d0` | **index → symbol** array, `alloc(7, n*4)` in the driver, filled by `0x10b568af`; read by the merge at `0x10b54c50` `[R]` |
| `0x10b18b28` | **the promotable-type table** — 30 bytes at stride 4, indexed by the type class. §3 `[R]` |
| `0x10bd7cf0` | the 13-entry type-class jump table (top nibble of the type word), immediately followed by `FUN_10bd7d24` — which is what bounds it at 13 `[R]` |

---

## 3. The promotion policy — item **F1**, which three documents call uncharacterized

`WB_LIVE_FINDINGS.md:682`, `WB_ITEMF_FINDINGS.md` F1 and `P_REGALLOC.md` §7 all
say *"which symbols become candidates is not characterized"*. It is decided
entirely inside `FUN_10b550e5`, in two gates, and it is **categorical — there
is no threshold constant anywhere in it.**

**Gate A — the structural gate**, in order:

| addr | test | effect |
|---|---|---|
| `0x10b5511a` | `sym+0x04 == 0x10` | skip the slot entirely |
| `0x10b55125` | — | `sym+0x40 &= ~1` unconditionally |
| `0x10b55129` | `sym+0x08 != sym` | skip — **only a group leader is considered** |
| `0x10b55134` | kind `== 3` | → gate A3 |
| `0x10b55138` | kind `< 3` | **REJECT** (kinds 0, 1, 2 — a physical register is kind 1) |
| `0x10b5513e` | kind `∈ {4,5}` | eligible; `sym+0x05 & 2` set ⇒ also joins the `DAT_10c2e3e8` set |
| `0x10b55142` | kind `== 6` | **REJECT** |
| `0x10b5514a` | kind `∈ {7,8}` | eligible, and always joins the `DAT_10c2e3e8` set |
| `0x10b5514e` | kind `!= 10` | **REJECT** (kind 9) |
| `0x10b55156`–`0x10b5516b` | kind 10 needs `*(sym)+0x37 & 0x400` set **and** `& 0x200000` clear | else **REJECT**; and then only sub-symbols with `t+0x20 == 4` are indexed (`0x10b55173`) |
| **A3** `0x10b551b3` | kind 3 needs `sym+0x14 == 0` … | else **REJECT** |
| `0x10b551bc` | … **and** `sym+0x07 & 0x40` clear | else **REJECT**; then `sym+0x06 &= ~2` |

`0x10b552b8` is the reject tail: it increments the diagnostic counter
`DAT_10c2e454` and clears `+0x34`/`+0x38` on **every** sub-symbol.

**Gate B — the type gate**, `0x10b551d4`: `FUN_10bd7d24(sym+0x10 as u16)`.
`0x10bd7c10` maps the type word's **top nibble** through the 13-entry table at
`0x10bd7cf0`, each arm resolving the low 12 bits to a **type class `0x00…0x1d`**;
`FUN_10bd7d24` then reads the byte at `0x10b18b28 + class*4`. Read out of the
pinned image:

> **Not promotable: classes `0x00`, `0x12`, `0x13`, `0x18`, `0x1d`. The other
> 25 are.** `0x00` is the null type (nibble 0); `0x1d` is **the whole of nibble
> 8**; `0x12`/`0x13` are nibbles 3–4 with low-12 ∉ {4}; `0x18` is nibble 6 with
> low-12 ∉ {1,2,4,8}. `[R]`

A symbol failing gate B gets `sym+0x34 = 0` at `0x10b5524f` and takes no
further part — no index, no version, no candidate.

> **What the policy does NOT contain, stated so absence does not read as
> coverage:** no size threshold, no use-count threshold, no live-range-length
> test, and **no compilation-mode flag**. `DAT_10c2e2cf` is consulted at
> `0x10b551dd` but only to add the index to a side bitset — it does not gate
> eligibility. **The mode-dependence of this phase is entirely at the phase
> level**, not the symbol level: `DAT_10c2e2fc` gates whether `FUN_10b7dc51`
> runs the phase at all, and it is cleared per function including on a
> **40,000 size bail-out** (`WB_CANDID_FINDINGS.md` §2.3, board **#3375**).
> **A port therefore needs no fitted constant for F1** — which is a stronger
> result than the read plan asked for.

---

## 4. The version numbering — what `FUN_10b55732` actually computes

`FUN_10b54bad(s, v)` is 67 bytes and does exactly four things `[R]`:

```
10b54bba  rec = alloc(arena 7, 8)
10b54bc4  rec[1] = new_bitset();  bitset_add(rec[1], v)
10b54bdb  rec[0]      = s->[0x34]->[0x0c]      /* HEAD INSERT ... */
10b54be0  s->[0x34]->[0x0c] = rec              /* ... so the list is DESCENDING v */
10b54be7  s->[0x34]->[0x14] = v
10b54beb  return v + 1
```

* **The counter is a local** (`local_c` in the driver's frame), **starts at 1**,
  and is **returned**; the driver uses the return only to size the
  version→candidate map (`alloc(7, v<<2)` at `0x10b577d8`). It is **not**
  `DAT_10c400d4` and it is **not** the candidate id.
* A symbol is versioned on **first encounter** (`aux[0x14] == 0`), whether that
  encounter is in the `T->[0x2c]` operand list or the `T->[0x28]` one — so a
  value that is live-in to the function is versioned at its first *appearance*,
  not at a definition. `[R]`
* `aux[0x14]` is **cleared** when an operand's `o[0x14] & 0xf` reaches 0
  (`0x10b559c2` region), which ends that version's extent and unlinks the
  symbol from the running list.
* **The head insert is the load-bearing detail for §6**: it means step 3 mints
  a symbol's candidates newest-version-first.

---

## 5. The merge rule — `FUN_10b54c07`

Called at the join into each successor block, with the running candidate list,
the merge bitset, the block's phi bitset, and **a pointer to the version
counter** `[R]`:

```
FUN_10b54c07(list, mergeset, phiset, &v):
  n = v
  for each index i in mergeset, via FUN_10b27290       /* ASCENDING index */
      s = DAT_10c400d0[i]                              /* 0x10b54c50 */
      link s onto the candidate list (sym+0x30 / aux+0x10)
      if phiset is non-empty:
          search s's version list for a record whose bitset meets phiset
          (FUN_10b273d3); if one is found, REUSE its number
      otherwise:
          bitset_add(phiset, n)
          rec = alloc(arena 7, 8); rec[1] = {n}
          HEAD-insert rec on aux[0x0c];  aux[0x14] = n;  n = n + 1
  *(&v) = n
```

Three things a port has to carry:

1. **The merge is keyed on the symbol**, reached through `DAT_10c400d0[i]` —
   two definitions of the same symbol merge, two different symbols never do.
2. **A merge either REUSES an existing version number or mints a fresh one**,
   and which it does depends on whether any existing version's bitset already
   intersects the join's phi set. So merge candidates are *not* always new.
3. **The iteration order is the bitset index order**, which by step 1 is the
   **symbol-arena order** — not the block order and not the source order.
   `FUN_10b27290` is a **stateful global iterator** (`DAT_10c2e1f0/f8/fc`):
   started with the set, continued with `NULL`. Nesting two walks of different
   sets is therefore impossible, which is why step 3 caches before it iterates.

---

## 6. The mint order — `FUN_10b55dbe`, and what "symbol-arena order" means

### 6.1 The walk is forward over the arena, and the arena serial is a real field

`0x10b55e37`–`0x10b55ea7` walks `symtab->[0x10]` chunk by chunk (`chunk->[0x00]`)
and, within each chunk, `chunk+0x20` upward at **stride `0x60`** to the
high-water mark `chunk->[0x04]`. `0x10bd2343` **appends** new chunks
(`symtab+0x14` is the tail and is written at `0x10bd23c9`-ish), so the chain is
in allocation order, and each slot was stamped `slot+0x1c = symtab[0]++` when
its chunk was created. **Walking the arena therefore yields ascending
`sym+0x1c`.** `[R]`

### 6.2 The three skips, and the one that matters

`0x10b55e50` skips kind `0x10`; `0x10b55e56` skips `sym+0x34 == 0` (everything
gate A or gate B rejected); **`0x10b55e5d` skips a symbol whose version list is
empty.** The third is the one with a consequence a port can use:

> **A promotable symbol that the renamer never reached mints no candidate and
> consumes no id.** Eligibility is necessary and not sufficient; *appearing in
> a real tuple* is the sufficient condition.

### 6.3 The order, stated

> **`id` ascends with (`sym+0x1c` ASCENDING, version number DESCENDING).**
> The outer key is the symbol's arena serial; the inner key is reversed by
> `0x10b54bad`'s head insert. `[R]`

### 6.4 The wrinkle: the serial is a slot property, not a creation-time property

`0x10bd3225`'s free-list path (`symtab+0x30`) `memset`s the recycled `0x60`
bytes and **restores `+0x1c`**. A symbol created late into a recycled slot
therefore carries an **early** serial and mints early. **"Arena order == symbol
creation order" is true only for a compiland in which nothing was recycled**,
and this page does not establish that. `[I]` on `[R]`.

---

## 7. `cand+0x44` — the field that decides every tie, read at last

`P_REGALLOC.md` §4.1's correction 2 records `+0x44` as *"the unenumerated field
that decides every tie"* and says nothing about what writes it. The complete
picture, enumerated two independent ways (Ghidra xrefs and a displacement scan
of the objdump, with every hit attributed to a containing function and
disqualified when the same base carries a displacement ≥ `0x48`):

| # | addr | function | what it writes |
|---|---|---|---|
| **1** | **`0x10b55fac`** | **`FUN_10b55eae`** | **`cand+0x44 = n`, the tuple-visit counter — the ONLY origination site** |
| 2 | `0x10b2e159` | `FUN_10b2dfe2` | verbatim copy of the **parent's** `+0x44` onto a split child |
| 3 | `0x10b2e665` | `FUN_10b2e4ae` | ditto |
| 4 | `0x10b2e73f` | `FUN_10b2e4ae` | ditto |
| 5 | `0x10b2f03b` | `FUN_10b2efd6` | `memset(cand, 0, 0x48)` — the destructor, a write of **0** |

**Sole reader: the comparator `FUN_10b2b82d`**, six reads
(`0x10b2b84d/850/860/863/87c/87f`). `FUN_10b316b1` and `FUN_10b31c9a`, the
comparator's other two callers, contain no `+0x44` reference at all.

> ## ⛔ CORRECTION to [`P_REGALLOC.md`](P_REGALLOC.md) §2 and §4.1 — the spiller does **not** write `cand+0x44`
>
> That page says `0x10b3032a` *"saves and restores `+0x40` **and `+0x44`**
> across a split"*, and §4.1 repeats it. **The record it saves is the BASIC
> BLOCK, not the candidate.** The list it iterates is `**(proc+8)` — the same
> list `FUN_10b2b3f0` initialises — and the same base is dereferenced at
> `+0x48`, `+0x4c` and `+0x50`, which a `0x48`-byte record cannot have. The
> saved values are **bitset pointers**. What the spiller's *subtree* does with
> the candidate's `+0x44` is rows 2–4 above: `FUN_10b2dfe2` and `FUN_10b2e4ae`
> (both called **only** from `0x10b3032a`) **propagate** the parent's key to a
> split child and never compute one. Board **#3412**.

**The default is a hard 0, not garbage** — two independent guarantees: the
arena `memset`s every chunk it takes (`0x10c202cd`, `0x10c202f4`, and the
large path), and the destructor `memset`s the whole `0x48` before the record
reaches the free list, restoring only `+0x1c`. So the free-list path in
`FUN_10b54d32` — which skips the id stamp — cannot leak a stale tie key. `[R]`

### 7.1 What the ordinal means, and the direction

The counter is monotone over the step-4 walk: blocks in layout order, tuples
from `B->[0x20]` stepping `T->[0x10]` to `B->[0x1c]`, counting only tuples with
`T+0x09 & 1`. `+0x44` is overwritten at every encounter, so it ends at the
candidate's **last visit** in that sequence.

[`P_DAG.md`](P_DAG.md):113 reads the tuple list as **`tuple+0` next, `+0x10`
prev** — already-held and not re-derived here — which makes the `T->[0x10]`
step a **backward** walk within each block, so `B->[0x20]` is the block's last
tuple and `B->[0x1c]` its first-sentinel. With blocks going **forward**
(§1's box), the composed traversal is *blocks forward, tuples backward*, and
the counter is **not reset per block** (`0x10b55eb7` is outside the block
loop). The arithmetic consequence would be:

> *Among candidates of equal priority, the one whose last visit falls latest in
> that traversal is coloured first — so a candidate appearing in a **later
> block** would outrank one confined to an earlier block, regardless of source
> position.*

> ## ⛔ THAT SECOND CLAUSE IS **NOT CONFIRMED**, AND THE CELL BUILT TO TEST IT DID NOT REACH THE TIER
>
> This page will not publish the clause as a rule. `scripts/globregs_c2.py`'s
> **G-block** cell was built precisely to separate it from plain arena order —
> two formals whose *declaration* order is fixed but whose *later* appearance
> is in swapped blocks. It came back **UNCHANGED at both `/O1` and `/Ox`**
> (`a0->r31, a1->r30` in both variants), where the clause predicts a swap.
>
> **The most likely reason is the caveat registered with the cell before it
> ran, and it fired:** moving a formal's later use into a different block
> changes its **live range**, hence `cand+0x0c` — the *primary* key — so the
> comparator very plausibly decided the pair on priority and **never reached
> the `+0x44` tier at all**. The result is therefore **UNGRADED, not a
> refutation**: it neither confirms nor refutes the clause, and the separator
> remains unbuilt. Board **#3414**.
>
> **What IS settled, and what is not:**
> * `[R]`, verified at the bytes twice-sourced — **`cand+0x44` is written, at
>   one origination site, with a monotone tuple-visit ordinal, and it is the
>   comparator's second key.** That much replaces `P_REGALLOC.md` §4
>   consequence 3 outright.
> * `[R]` **only** — what the ordinal means in *program* terms. It rests on an
>   inherited reading of `+0x10` as *prev*, and on a traversal this lane
>   composed rather than confirmed.
> * `[O]` on 118 cells — the *observable*: on every straight-line body built
>   here the map is invariably `a_i -> r(31-i)`, first formal first, at both
>   modes. **That is consistent with the ordinal reading AND with plain
>   arena/mint order, and this lane could not build a cell that separates
>   them.** Prereg §7 item 1 registered exactly this in advance: the observable
>   is a many-to-one image of the claim.

---

## 8. What this decides for `codegen::alloc`'s ten refuted keys

`crates/c2-core/src/codegen/alloc.rs:103-539` catalogues **ten fitted-then-
refuted allocation keys, "wrong on 5 to 42 each"**, and `alloc.rs:29-36`'s
**52,416-configuration** preregistered search returns 179/236 with the residual
*exactly the tie tier*. Read **R1** removed their only standing explanation
(`../WB_CANDID_FINDINGS.md` §5.2) and sent the question to R4.

> ### They now HAVE an explanation, and it is a mechanism with an address.
>
> The tie key is **`cand+0x44`**, written at **`0x10b55fac`**: an ordinal in a
> **blocks-forward × tuples-backward** walk of the whole function, taken at the
> candidate's **last visit**, and sorted **descending**. Every one of the ten
> refuted keys is a function of a **source-level property of a variable** —
> declaration order, use count, live-range length, first-use position. None of
> them is that ordinal, and three structural reasons make it impossible for any
> of them to be:
>
> 1. **The ordinal counts *lowered* tuples, not source constructs.**
>    `P_REGALLOC.md` §5 already measured the consequence — `a+b+b+b` folds to
>    `3*b + a`, so the machine use count and the source use count are different
>    numbers, and `/O1` and `/Ox` disagree on 6 of 20 cells with the relation
>    exact reversal.
> 2. **The counter is not reset per block** (`0x10b55eb7` sits outside the
>    block loop), so the ordinal is global to the function and mixes block
>    position into the key. *(§7.1: the sharper form of this clause —
>    "a later block outranks an earlier one" — is **UNGRADED**; the cell built
>    to test it did not reach the tie tier. It is listed here as a structural
>    property of the counter, which is read, not as a confirmed ordering.)*
> 3. **A variable is not a candidate.** A symbol with *k* versions mints *k*
>    candidates (§1 step 3), each with its own `+0x44`. Every one of the ten
>    keys is one-candidate-per-variable by construction and is therefore wrong
>    in kind, not merely mis-fitted, on any body where a value is redefined.
>
> **And the 52,416-configuration null is explained too, which is the sharper
> half.** That search ranged over *priority functions* — candidates for
> `cand+0x0c`. The residual it could not reach is the **tie tier**, and the tie
> tier is **not a priority function at all**: it is a traversal ordinal stamped
> by a different phase, in a different function, before the priority is ever
> accumulated. **No member of that family could have expressed it**, so the
> null was structurally guaranteed rather than evidence about priority
> functions. Board **#3413**.

**The fence on that, stated in the lane's own voice.** This says the ten keys
were fitting the wrong variable and names the right one. It does **not** say
the port can now compute it: the ordinal is over c2's *lowered, scheduled*
tuple list, which is `CEILING` §6.1's phase 0/1 output and is exactly what the
port does not yet have. **This converts an unexplained residual into a named
dependency on a phase**, which is a different and better thing to own than a
mystery — but it is not a key a port can drop in today.

---

## 9. What is NOT known

* **Whether the arena serial equals symbol creation order** in a real
  compiland — §6.4's recycling path is a live hole and no cell here exercises
  it.
* **Which of `T->[0x28]` and `T->[0x2c]` is the def list** and which the use
  list. The renamer versions on first encounter in either, so the *numbering*
  is settled without it; the *merge* semantics are not.
* **`aux+0x18`, the partner symbol** versioned alongside at `0x10b55aa2` — read
  as a field, not as a rule.
* **The FPR path.** No cell in any grid uses floating point
  (`P_REGALLOC.md` §7), and gate B's classes `0x0d`–`0x0f` (nibble 5, the FPR
  nibble) are promotable but untested. **Blind, and said so.**
* **The `> 1024`-candidate regime.** `id & 0x3ff` wraps and step 4's ordinal
  does not; nothing here tests a body that large.
* **Whether `FUN_10b55eae`'s remaining 1,300 bytes hold a second policy.** Only
  the `+0x44` stamp and the operand re-point were read out of it.
* **The IL-record side.** Which IL constructs create which symbol kinds is read
  **R5**'s subject (`FUN_10bc2d7a`) and is an open cross-reference from here,
  not a claim.
