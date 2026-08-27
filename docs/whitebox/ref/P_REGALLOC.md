# `P_REGALLOC` — `color.c`: Chow–Hennessy priority colouring

> **Reference page.** **`[R]`** read from the disassembly, *not* obj-checked —
> a hypothesis. **`[O]`** confirmed against a real obj or `/FAsc` listing, with
> the witness named. **`[I]`** an interpretive step. Navigation only; nothing
> here may enter `crates/` without a [`DISCLOSURE.md`](../DISCLOSURE.md) row.
> Index: [`ADDR.tsv`](ADDR.tsv) · front door: [`README.md`](README.md)

> ## ⛔ CORRECTED 2026-08-22 by read **R1** — `DAT_10c400d4` is FUNCTION-SCOPED
>
> This page said **"compilation-global"** of the candidate-id counter in three
> places (`:62`, `:86`, `:188`) and built **consequence 3** (`:160-166`) on it.
> Lane `w-read-r1` enumerated the counter's **complete** reference set — four
> instructions — and found the **sole writer** at **`0x10b57676`**, storing the
> constant **1**, inside `FUN_10b57633` (the `globregs.c` phase) which the
> per-function driver `FUN_10b7dc51` calls at `0x10b7dcb7`. Every mint site is
> downstream of it. Full read + the behavioural control:
> [`../WB_CANDID_FINDINGS.md`](../WB_CANDID_FINDINGS.md).
>
> `WB_LIVE_FINDINGS.md:258-260`'s *"dense and function-scoped"* was right and
> this page was wrong. **The three claims are struck in place below** (per
> [`README.md`](README.md) §2.1 corrections are amended beside, never silently
> rewritten), and consequence 3 carries its own revision box.

**Coverage: 18 code entries + 15 data entries against a denominator of 70** —
Ghidra functions whose entry lies in `color.c`'s span
(`0x10b2c21d`–`0x10b3219f`, anchor plus its following gap). The register
environment (`0x10bfb0fa`, `0x10c04faf`) and the liveness solver
(`globregs.c`, `0x10b54848`…) are outside that denominator and are listed
anyway because the allocator cannot be read without them. Not covered: the
spiller's rewrite (`0x10b31544`), `regasg.c` in any detail, and the cost model
of item F5.

> **The identification.** c2's allocator is **priority-based colouring
> (Chow–Hennessy), not Chaitin** — there is no interference graph and no
> simplify/select stack. `wb-live`'s *"the neighbour set is recomputed"* already
> implied it and **nobody had stated it**; `w-dagorder` stated it and confirmed
> the priority list from the obj (`#3239`). The missing statement is why the
> repo's black-box attempts kept reaching for a *traversal* rule.

---

## 1. The pipeline

```
FUN_10b31c9a  (0x10b31c9a)   per function, register classes 7 -> 0
  FUN_10b2ceb7   pressure reduction / copy coalescing      (pre-spill)
  FUN_10b2d630   narrow every candidate's ALLOWED set; accumulate cand+0x0c
  FUN_10b315df   seed: mint / merge candidates
  FUN_10b316b1   BUILD THE PRIORITY WORKLIST  ->  DAT_10c43b7c
  while (DAT_10c43b7c) {
      pop head                                 (0x10b31e97)
      FUN_10b30517   recompute this candidate's neighbour set
      FUN_10b2e7f8   THE SELECTOR: pick the register
  }
```

`0x10b31c9a` has **exactly one caller**, `0x10b7dc51` — the phase driver, which
also runs the instruction scheduler three times around it `[R]`.

---

## 2. Entries

| addr | size | callers | callees | TU | cites | what |
|---|---:|---:|---:|---|---:|---|
| `0x10b31c9a` | 826 | 1 | 31 | `color.c` gap | 50 | **the allocator driver.** Classes 7 → 0 over `DAT_10c400d8[class]`; interference walk → selector → `0x10b31ac9` `[R]` |
| `0x10b31e97` | *(in `0x10b31c9a`)* | — | — | `color.c` gap | 1 | the head-first pop of the worklist `[R]` |
| `0x10b2e7f8` | 582 | 1 | 13 | **`color.c`** | 17 | **THE SELECTOR.** Min cost over the interference-allowed candidates; cost = penalties − copy preferences; **strict `<`, so ties go to the EARLIEST entry of the per-class ordered list**. The only reader of `0x10c385c4` `[R]` · order **`[O]` 6/6** (`WB_REGALLOC_FINDINGS.md` §7.1) |
| `0x10b30517` | 2214 | 1 | 24 | **`color.c`** | 13 | **the interference "graph" — which is not one.** Recomputes, per candidate, the set of candidates simultaneously live with it, by walking blocks forward from `cand+0x28` and tuples backward within each `[R]` |
| `0x10b2d630` | 2034 | 1 | 28 | **`color.c`** | 34 | **the allowed-set narrowing AND the priority accumulation** — it does both, which is the correction that closed `WB_LIVE` §10. `cand[0x0c] += cand[0x18] * n_live`, `-= n_live` when not live, and a POGO-only third term `[R]` |
| `0x10b2ceb7` | 1913 | 1 | 28 | **`color.c`** | 9 | the copy/`mr` coalescer; the copy opcode set is `{0x270, 0x272, 0x293, 0x7b}` `[R]` |
| `0x10b3032a` | 493 | 1 | 13 | **`color.c`** | 5 | **the spiller / live-range splitter.** ~~Saves and restores `+0x40` **and `+0x44`** across a split (`iVar7[0x40] = …[3]; iVar7[0x44] = …[4]`)~~ ⛔ **R4: that is the BASIC BLOCK record, not the candidate** — the list is `**(proc+8)`, the same one `FUN_10b2b3f0` initialises, and the same base carries `+0x48`/`+0x4c`/`+0x50`, which a `0x48`-byte candidate cannot have; the saved values are **bitset pointers**. **This function never writes candidate `+0x44` at all**; its callees `0x10b2dfe2`/`0x10b2e4ae` *inherit* the parent's key onto a split child (§4.1). Uses `+0x18` as a **bitfield** (`&= 0xfffffffe`) `[R]`. `wb-live` listed this function as *"named and not opened"* |
| `0x10b55dbe` | 240 | 1 | 6 | *(gap)* | — | ⭑ **R4: WHERE CANDIDATE IDS ARE ACTUALLY ASSIGNED.** Symbol-arena order × version-list order; the mint call is `0x10b55e66`. No document named it before 2026-08-23. [`P_GLOBREGS.md`](P_GLOBREGS.md) §6 `[R]` |
| `0x10b55eae` | 1468 | 1 | 18 | **`globregs.c`** | 5 | ⭑ **R4: THE SOLE ORIGINATING WRITER OF `cand+0x44`**, at `0x10b55fac` — a tuple-visit ordinal. Previously listed here only as *"rebuilds the per-block bitsets"*. [`P_GLOBREGS.md`](P_GLOBREGS.md) §7 `[R]` |
| `0x10b550e5` | 490 | 1 | 5 | *(gap)* | — | ⭑ **R4: the promotion policy — item F1**, which §7 below calls unread. It is here, not in `0x10b55732`. [`P_GLOBREGS.md`](P_GLOBREGS.md) §3 `[R]` |
| `0x10b316b1` | 164 | 1 | 4 | `color.c` gap | 14 | **builds the priority worklist.** Walks the 1024-bucket candidate hash `DAT_10c43b80` bucket by bucket, follows each chain at `cand+0x30`, filters by class through `&DAT_10b022cc`, and accumulates through `0x10b2b82d`; ends `DAT_10c43b7c = list` `[R]` |
| `0x10b315df` | 210 | 1 | 8 | `color.c` gap | 2 | seed: mint / merge candidates `[R]` |
| `0x10b2b82d` | 126 | 3 | 0 | *(gap after `coffemit.c`)* | 13 | **THE PRIORITY COMPARATOR** — a sorted insert into a doubly-linked list (`+0x14` next, `+0x18` prev). §4 `[R]` · order **`[O]`** on 20 cells at two profiles |
| `0x10b2c21d` | — | — | — | **`color.c` anchor** | — | candidate lookup by id: 1024-bucket hash at `0x10c43b80`, chain `cand+0x30`, ICE if absent `[R]` |
| `0x10b54d32` | 130 | 6 | 5 | *(gap after `globopt.c`)* | 17 | **the candidate constructor.** `alloc(0x48)`; `kind = 2`; `id = DAT_10c400d4++` (~~compilation-global monotonic~~ ⛔ **R1: FUNCTION-SCOPED** — reset to `1` at `0x10b57676`; and the id is stamped **only on a fresh `alloc`**, the free-list path at `0x10b54d48` jumps past both the read and the increment, so a recycled record keeps its old id); `allowed = copy(DAT_10c3d024[class])` — **every candidate starts with the whole class allowed**, and allocation only ever removes `[R]` |
| `0x10b55732` | 1676 | 1 | 18 | *(gap after `globopt.c`)* | 16 | ~~`globregs.c`'s mint/merge; **its promotion policy is item F1 and is unread**~~ ⛔ **R4: it is the RENAMER, and it mints nothing** — 18 callees, none of them `0x10b54d32`. It builds the dense per-function **live-range version** numbering by a **backward** walk (blocks via `B->[0x04]`, tuples via `T->[0x10]`) and returns the count. The promotion policy is `0x10b550e5`; the mint is `0x10b55dbe`. [`P_GLOBREGS.md`](P_GLOBREGS.md) §1, §4 `[R]` |
| `0x10b55eae` | 1468 | 1 | 18 | **`globregs.c` anchor** | 5 | rebuilds the per-block bitsets `[R]` |
| `0x10b54904` | — | — | — | *(gap)* | — | the backward liveness fixpoint `[R]` |
| `0x10b54848` | — | — | — | *(gap)* | — | the forward availability fixpoint, same shape `[R]` |
| `0x10c20f79` | — | — | — | *(gap)* | — | the liveness driver the allocator calls first: runs the pair twice, then binds each candidate's first/last block `[R]` |
| `0x10bfb0fa` | 117 | 1 | 3 | `code.c` gap | 5 | builds the class-0 (GPR) allocatable set **and its ordered list**; then drops the frame-pointer register or adds `r31` `[R]` |
| `0x10c04faf` | 392 | 2 | 3 | *(gap)* | 10 | the per-function register environment: `DAT_10c6fd9c` = **the frame-pointer register**, `DAT_10c6fda8` = the volatile set, `DAT_10c6fda4` = the reserved set `[R]` |
| `0x10bfc02f` | 44 | 1 | 0 | `code.c` gap | 4 | the machine-dependent pre-select hook — **class 5 (VMX) only, and only under `-QVMX128`**, which is what makes the GPR/FPR order a rule rather than a snapshot `[R]` |

### 2.1 Data

| addr | what |
|---|---|
| `0x10b181c0` | **the PPC register-NAME table, and its index IS c2's register number**: `0` noreg, `1` r0, `2` sp, `3` toc, `4…13` r3…r12, `14` r13, `15…32` r14…r31, `33` d0, `34…65` fp0…fp31, `66` cr, `67…74` cr0…cr7, `83` **lr**, `84` ctr, `229…356` vr0…vr127 `[R]` |
| `0x10c37de0` | **the default GPR allocation order**, zero-terminated: `12,11,10,9,8,7,6,5,4, 32,31,…,15` = **`r11,r10,…,r3` then `r31,r30,…,r14`** `[R]` · **`[O]` on cells G1–G4 and P1 with no disassembly** |
| `0x10c37e50` | the `-QGPRReserve` variant — drops `r14`,`r15`,`r16` `[R]` |
| `0x10c37eb8` | the POGO-instrumented variant `[R]` |
| `0x10c37f20` | the FPR order: `fp0, fp13, fp12, …, fp1, fp31, fp30, …, fp14` — ~~**read, never obj-checked**, no cell uses floating point~~ ⭑ **OBJ-CHECKED 2026-08-27 by `w-regcells`: `[O]` on 20 of 20 graded cells at two profiles, 29 of the 32 entries witnessed in position, four rivals refuted (three by ≥18 cells). `fp1`, `fp15`, `fp14` remain `[R]` — no cell reached them.** [`../WB_REGCELLS_FINDINGS.md`](../WB_REGCELLS_FINDINGS.md) §2 |
| `0x10c385c4` | the 8-entry per-class ordered-list array. **Only class 0 (GPR) and class 1 (FPR) are image-initialised**; class 5 (VMX) is filled at run time; classes 2,3,4,6,7 are NULL — **there is no CR register class** `[R]` · ⭑ **`[O]` 2026-08-27**: `ctr` (register 84) and `r12` (register 13) are in **no** list, and neither displaces a value with nine candidates live across it (`pd_ctr_p`, `pd_lr`) — **a physical def only narrows if the register is in the class's list** |
| `0x10b022cc` | operand-type nibble → register class: nibbles `1..4`,`6` → 0 (GPR), `5` → 1 (FPR), `12` → 5 (VMX), everything else `-1` `[R]` |
| `0x10c435e8` | the selector's cost array, `0x594` bytes = **357 ints, one per register number** (`0…356`) — an independent confirmation of the register table's size `[R]` |
| `0x10c43b7c` | **the priority worklist head** `[R]` |
| `0x10c43b80` | the 1024-bucket candidate hash (`0x10c44b80 − 0x10c43b80 = 0x1000`) `[R]` |
| `0x10c400d4` | ~~the compilation-global monotonic candidate-id counter~~ ⛔ **R1: the PER-FUNCTION monotonic candidate-id counter, dense from 1.** Sole write `0x10c400d4 = 1` at **`0x10b57676`**; sole increment `0x10b54d5f`; read at `0x10b54d57` (the stamp) and `0x10b54db5` (the partial clear's occupied-bucket bound). Four references in the image, no more `[R]` · `[O]` by control C1 |
| `0x10c400d8` | per-class candidate id sets `[R]` — **rebuilt per function** at `0x10b57658` |
| `0x10c2e3e0` | the candidate **free list** head — **emptied per function** at `0x10b5767c`, which is what makes ids dense (§4 note) `[R]` |
| `0x10c6fd9c` | **the frame-pointer register number** — written only at `0x10c04fd1`: `2` (`sp`) when the function needs none, `0x20` (`r31`) when it does `[R]` |

---

## 3. The selector — `0x10b2e7f8`

1. `memset(&DAT_10c435e8, 0, 0x594)` — the cost array.
2. **Penalties**: for each interfering neighbour, `cost[reg] += weight`; for a
   neighbour reduced to a single register, `cost[that reg] += 100 × …`.
3. **Preferences**: for each entry on this range's own preference list
   (`cand+0x38`, the copy/coalesce list), `cost[reg] -= weight`. **Negative cost
   = preferred**, and this is what keeps an argument in its argument register.
4. `0x10bfc02f` runs here — VMX only.
5. The selection loop, walking the ordered array from index 0 forward:
   ```
   best = none;
   for (i = 0; list[i] != 0; i++)
       if (candidate_set has list[i] && (best == none || cost[list[i]] < best_cost))
           { best = list[i]; best_cost = cost[best]; }
   ```
   **strict `<` ⇒ ties go to the EARLIEST register in list order** `[R]`.

> **c2 picks the minimum-cost register among those the interference relation
> still allows, where cost is (interference and constraint penalties) minus
> (copy preferences), and every tie is broken by the fixed order
> `r11, r10, …, r3, r31, r30, …, r14`.**

`[O]` on 6 frozen cells (G1–G4, L3, P1); three rivals refuted — first-free
ascending from `r3` (6 cells), first-free descending from `r12` (9 cells,
`r12` never holds a value in any of 15), and no-preference (4 cells).

> ### ⚠ Two corrections to board `#1821`, filed by `wb-live` and carried here
>
> * The second penalty's multiplier is **not the degree**. `[iVar8 + 0x40]` is
>   the neighbour's benefit accumulator; **there is no degree counter anywhere
>   in the candidate record**.
> * The first clause is *"do not take a register a live neighbour **wants**"*,
>   not *"has"* — an already-coloured neighbour has `allowed == NULL` (freed by
>   the selector itself) and contributes **nothing**.
>
> **Neither correction is obj-separable on any shape this project can build**,
> stated so absence does not read as coverage: on all 10 cells of `wb-live`'s
> grid and all 15 of `wb-regalloc`'s, every cost array is uniformly zero over
> its allowed set and the answer is decided entirely by list order.

---

## 4. The tie-break — usability question **U3**, answered here

**`0x10b2b82d`**, the sorted insert that builds the worklist:

```
insert new before n  iff  n->[0x0c] <  new->[0x0c]                            /* signed   */
                     or  (n->[0x0c] == new->[0x0c] && n->[0x44] <= new->[0x44])  /* unsigned */
```

> **Primary key `cand+0x0c` DESC (signed); tie-break `cand+0x44` DESC
> (unsigned); and the tie tier compares `<=`, not `<`, so an exact tie in BOTH
> keys puts the NEWLY inserted candidate FIRST.** `[R]`, order **`[O]`** on 20
> cells at two profiles.

Three consequences a port has to carry:

1. **`cand+0x0c` is the priority**, accumulated by `0x10b2d630` over the
   candidate's live range: `+= cand[0x18] * n_live` where live, `-= n_live`
   where not, scaled by a block weight. It is a **spill cost/benefit measure**,
   which is why a black-box reading came out looking like a use count and is
   **not** one `[I]`.
2. **A spilled candidate re-enters by priority, not at the head** — the driver
   calls the same comparator (`DAT_10c43b7c = FUN_10b2b82d(cand, DAT_10c43b7c)`).
   **A port modelling the worklist as a stack or a queue is wrong in both
   directions.**
3. **On an exact tie the order is a hash-bucket walk over a ~~compilation-global~~
   PER-FUNCTION counter, ~~not a source property~~.** The finished list is the reverse of
   `0x10b316b1`'s accumulation order, and that order is buckets `0…1023` of
   `DAT_10c43b80` keyed on `cand+0x1c = DAT_10c400d4++` mod 1024. ~~**This is the
   most direct available explanation for why source-level fitted sorts keep
   being refuted**~~ (`codegen::alloc` clause 2, `#836`, 7 of 56 fresh-holdout
   cells).

   > ## ⛔ REVISION 2026-08-22 — the mechanism survives, the inference does not
   >
   > **Read R1** (`../WB_CANDID_FINDINGS.md`) settled the counter's scope
   > against this row: `DAT_10c400d4` is reset to `1` at **`0x10b57676`** once
   > per function, and every mint site is downstream of that reset. The bucket
   > walk is real and unchanged; what is withdrawn is the **"so"**.
   >
   > A **per-function counter that is dense from 1** means a candidate's bucket
   > *is* its mint index within the function — which is exactly the kind of
   > quantity that **can** track source order. So this row no longer explains
   > why source-level fitted sorts keep being refuted; it hands the whole
   > question to the **mint order**, `FUN_10b55732` (item **F1**, read **R4**,
   > 1,676 B, still unread). The `n=3` divergence the lane fenced itself with
   > below is now the only live evidence on that axis, and it points at R4.
   >
   > **Consequence for `crates/c2-core/src/codegen/alloc.rs:103-539`: the ten
   > refuted allocation keys are back to UNEXPLAINED.** `READ_PLAN` §3 row R1
   > named that as the thing R1 decides, and this is the decision. Board
   > **#3374**.

   > ## ⛔ REVISION 2026-08-23 by read **R4** — THIS IS THE **THIRD** TIER, NOT THE SECOND
   >
   > The row above describes the bucket walk correctly and **puts it one tier
   > too high.** `cand+0x44` — the comparator's *second* key, which §4.1 below
   > records as *"the unenumerated field that decides every tie"* with no
   > writer named — **is written**, and it holds a **program-position
   > quantity**:
   >
   > > **Sole origination site `0x10b55fac`, in `FUN_10b55eae`**: a tuple-visit
   > > counter zeroed once per function at `0x10b55eb7` and incremented once
   > > per real tuple at `0x10b55f77`. Three further writes exist and all three
   > > are verbatim **inheritance** onto a split child (`0x10b2e159`,
   > > `0x10b2e665`, `0x10b2e73f`); a fifth is the destructor's
   > > `memset(cand,0,0x48)` at `0x10b2f03b`. Enumerated two independent ways.
   > > The **sole reader** is the comparator, six reads.
   >
   > So the bucket walk is reached only when two candidates tie on `+0x0c`
   > **and** on `+0x44`. The tie tier proper is a **sort on a traversal
   > ordinal**. [`P_GLOBREGS.md`](P_GLOBREGS.md) §7,
   > [`../WB_GLOBREGS_FINDINGS.md`](../WB_GLOBREGS_FINDINGS.md). Board
   > **#3411**.
   >
   > **And this answers what R1 handed on.** `../WB_CANDID_FINDINGS.md` §5.2
   > left the ten refuted `codegen::alloc` keys unexplained on this mechanism.
   > They are explained now — the key is a lowered-program-position ordinal, so
   > every source-level variable property was the wrong variable, and a
   > one-candidate-per-variable key is wrong *in kind* because a symbol with
   > *k* versions mints *k* candidates. **The 52,416-config null was
   > structurally guaranteed**: that search ranged over *priority functions*
   > and the tie tier is not one. `P_GLOBREGS.md` §8. Board **#3413**.
   >
   > **What R4 did NOT settle**, said here so the row is not over-read: what
   > the ordinal means in *program* terms is `[R]` only. A separator cell that
   > reaches the `+0x44` tier while holding `+0x0c` fixed was **not** built —
   > the one that was (`G-block`) moved the live range and so never reached the
   > tier. Board **#3414**.

   > **The fence on that claim, from the lane that made it.** Pure descending id
   > predicts `n=2`'s `b a` and `n=8`'s leading `h g f`, but **`n=3` is `b a c`
   > where it predicts `c b a`**. Either the mint order is not source order
   > (`0x10b55732`, unread) or the `n=3` keys are not tied — **that grid does not
   > separate the two**. The claim is *"the tie tier is bucket-walk order"*, not
   > *"reverse source order"*.

### 4.1 The `0x48`-byte candidate record — with `#3243`'s two corrections beside

`wb-live`'s enumeration, **kept as written**, with the corrections appended:

| offset | `wb-live`'s reading | correction |
|---|---|---|
| `+0x00` | type | |
| `+0x04` | kind (`= 2`) | |
| `+0x05`,`+0x06` | flags | |
| `+0x0c` | cost accumulator | **it is THE PRIORITY** — the worklist comparator's primary key (`#3239`) |
| `+0x10` | assigned register descriptor | |
| `+0x14` | live-list link | also the worklist **next** pointer |
| `+0x18` | cost accumulator | ⛔ **CORRECTION 1 (`#3243`).** At worklist time it is the priority list's **`prev` pointer** (`0x10b2b82d` writes `n->[0x18] = new`), and `0x10b3032a` uses it as a **bitfield** (`&= 0xfffffffe`). The field is **phase-overloaded**: a weight during `0x10b2d630`, a back-pointer during `0x10b31c9a`'s loop. **A flat field table cannot express that, and a port reading it as one thing is wrong in one of the two phases.** |
| `+0x1c` | id | `DAT_10c400d4++`, ~~compilation-global~~ ⛔ **R1: per-function, dense from 1** (`0x10b57676`) |
| `+0x20` | allowed set | empty ⇒ spill |
| `+0x24` | ref count | |
| `+0x28`,`+0x2c` | first / last block | |
| `+0x30` | hash chain | |
| `+0x34` | a byte counter | |
| `+0x38` | preference list | the selector's negative term |
| `+0x3c` | last defining tuple | |
| `+0x40` | benefit | saved/restored across a split |
| **`+0x44`** | **— absent —** | ⛔ **CORRECTION 2 (`#3243`).** The record is `0x48` bytes and the enumeration stops at `+0x40`. `+0x44` is the **worklist comparator's tie-break**, ~~saved and restored beside `+0x40` by the spiller `0x10b3032a`~~. **The unenumerated field is the one that decides every tie — and at `/O1` most cells are ties.** <br>⛔ **R4, 2026-08-23 — the field is READ.** It is a **tuple-visit ordinal**, written at exactly one origination site **`0x10b55fac`** (`FUN_10b55eae`), counter zeroed at `0x10b55eb7` and `inc`'d at `0x10b55f77`. **The spiller does not write it** — that clause was the *block* record (see §2). The only other writes are three verbatim split inheritances and the destructor's `memset`. **Its default is a hard 0**, twice guaranteed: the arena `memset`s every chunk (`0x10c2022a`) and the destructor `memset`s the whole `0x48` restoring only `+0x1c`, so the free-list path in `0x10b54d32` cannot leak a stale key. [`P_GLOBREGS.md`](P_GLOBREGS.md) §7 |

> **Two different `+0x44`s exist and they are not the same field.** This one is
> the *allocator candidate's*. The **DAG node's** `+0x44` is the original tuple
> index, minted `node+0x44 = DAT_10c435cc++` at `0x10b327cd`, and it is the
> *scheduler's* ready-list tie-break — [`P_DAG.md`](P_DAG.md) §3. They are
> different records in different passes and conflating them is easy.

---

## 5. The order is downstream of the scheduler — and the profile decides it

**`#3240`.** Four rival readings were refuted from the obj with no address —
source order (dies on `cnd_a2`), reverse source order (`cnd_a4` is `d b a c`,
not `d c b a`), arrival-register order, and use-count order. The fifth was
confirmed by a **height swap**: `cnd_h2`/`cnd_h2r` hold the formal list, the
declaration order and the live set **fixed** and move only which formal carries
the taller producer — **the assignment flips** `[O]`.

**`#3241`, and it is the one to carry.** `/Ox` and `/O1` disagree on **6 of 20
cells**, and the six are exactly the six that carry the signal; the `/O1` ↔
`/Ox` relation is **exact reversal on all six**. The mechanism is in the bytes:
`/O1` lowers `*3` as one `mulli` (multiplicand read **once**); `/Ox`
strength-reduces to `slwi`+`add` (read **twice**).

> **The fixture corpus is captured at `/Ox` and the workload compiles `/O1`**
> (`STATUS.md` trap 7). A characterization of candidate order taken at the
> fixture profile publishes the **reversed** rule. State the profile or the
> finding is wrong almost everywhere.

**And the transferable edge**: the **source** use count and the **machine** use
count are different numbers. `a+b+b+b` folds to `3*b + a`, so `b` is read
**once** in the emitted code. A port fitting candidate order against
source-level use counts is fitting the wrong variable.

---

## 6. Corrections this page carries forward

* ⛔ **`#1823` — "THIS `c2.dll` HAS NO INSTRUCTION SCHEDULER" is REFUTED.**
  `WB_REGALLOC_FINDINGS.md` §1's bold paragraph stands as written and is wrong;
  see [`P_DAG.md`](P_DAG.md) §1. Its load-bearing evidence was *"there is no
  `sched.c` in the TU table"* — a true statement about the **instrument**
  (`c2_tus.tsv` is built from ICE sites, and a TU with no ICE site is invisible
  to it) read as a statement about the **image**. Absence read as coverage, in
  the whitebox record itself. **Every "order" claim in that document is
  therefore a statement about a *scheduled* tuple list.**
* ⛔ **`WB_FRAME_FINDINGS.md` §2.4's "frame-establish pseudo-register" is
  wrong.** Register `0x53` is **`lr`** on the `0x10b181c0` table. The corollary
  that `DAT_10c6fd9c` is "the LR pseudo-register" is wrong too — it is the
  **frame-pointer register number**.
* ⛔ **`F3`/`P2.5` retracted**: `cr6`, not `cr0`, is what an explicit integer
  compare uses. `cr6` is a **lowering constant** assigned literally
  (`0x10c00445`, `0x10bf0882`, `0x10bf522f`, `0x10c195a2`), not an allocator
  choice; `cr0` is reached only through record-form instructions. The
  instruction reading (*which* CR fields are volatile) was right and the
  inference about behaviour was wrong — the shape of error the method doc warns
  about.

---

## 7. What is NOT known here

* **Item F is NOT unblocked** (`#3243`), and F5's published price of 2 lanes is
  wrong in kind: F5's input is `cand+0x0c`, accumulated over the code **the
  scheduler produced**, and F0 — priced at 8 — is what produces it. **F5 is not
  separable from F0.**
* ~~**F1**, `globregs.c`'s promotion policy at `0x10b55732`: unread.~~
  ⛔ **READ 2026-08-23 by R4, and it is at a different address**:
  `FUN_10b550e5` @ `0x10b550e5`. Two gates — a **kind** switch (every arm
  addressed) and a **type-class** table at `0x10b18b28` (30 entries, exactly
  five classes not promotable). **No numeric threshold and no mode flag**, so
  a port needs no fitted constant for F1. [`P_GLOBREGS.md`](P_GLOBREGS.md) §3.
  The *candidate set* is still not buildable, but for a different reason: a
  candidate is a **(symbol, live-range version)** pair, and the versions need
  the backward walk over the **lowered** tuple list.
* ~~**F4**'s non-call physical def: **still no obj cell in existence.**~~
  ⛔ **REFUTED 2026-08-27 by `w-regcells`, and it was wrong when it was
  written.** [`../WB_REGCELLS_FINDINGS.md`](../WB_REGCELLS_FINDINGS.md) §4.
  `scripts/gt_argperm.py --pure` has been compiling the shape since 2026-07 —
  **213 cells** ([`../../CODEGEN_ARG_PERM.md`](../../CODEGEN_ARG_PERM.md) §2,
  §5), each a **tail call** whose body is *"no frame, no saved registers,
  nothing but the moves"*: no `bl` anywhere, a bare physical def of every
  argument register, candidates live across it. **Two documents eleven
  directories apart, neither citing the other** — the same failure shape as
  `#1823`, a claim about the *index* read as a claim about the corpus.
  The state now, decomposed:
  * **(a)** a non-call physical def of an allocatable GPR and **(b)** a
    candidate live across it: **`[O]` on 216 cells** (213 + `pd_tail`,
    `pd_perm6`, `pd_perm8`).
  * **(c)** the narrowing separated from ordinary pressure: **`[R]`, and
    unreachable by construction** — on every shape this front end can express,
    a register a bare def makes unavailable is *simultaneously held by a live
    candidate*. Registered as the ceiling **before** the deciding cell was
    compiled.
  * **New and separable**: `pd_perm8` (σ = four 2-cycles, eight formals, one
    free volatile) hands its four scratches out **`r11`, `r31`, `r30`, `r29`**
    — this list, continuing into the callee-saved tail, **framing a function
    with no call tuple ahead of it**. So the permutation scratch is an
    **allocated candidate**, not a hardwired temp, and
    `CODEGEN_FRAMED_CALLS.md` §3.2's *"broken with `r11`"* is a consequence of
    `r11` being **entry 0**. Board **#3708**.
  * **Price**: `WB_ITEMF_FINDINGS.md` §6.1's *"1 grid lane to obtain the first
    obj cell"* is **spent and was already paid**; F4's remaining price is
    **1, not 2**. And F4's fail-closed boundary *"refuse on any bare physical
    def"* was priced free and **is not** — it withdraws every permuted call,
    a class `crates/c2-core/src/codegen/calls.rs` emits today. Board **#3710**.
* **The cost model itself**: this record has the comparator, not the cost
  function's derivation.
* ~~The FPR order at `0x10c37f20` is **read and never obj-checked** — no cell in
  any grid uses floating point.~~ ⭑ **CLOSED 2026-08-27 by `w-regcells`:
  `[O]`, 20 of 20 graded cells, 0 unscoreable, at both `/O1` and `/Ox`.** The
  selector's *"ties go to the earliest entry of the per-class ordered list"*
  (§3) is now confirmed on a **second class**. Residue: `fp1`, `fp15`, `fp14`
  unwitnessed, and the **cost arithmetic is untouched and stays `[R]`** — every
  cost array in this grid is uniformly zero over its allowed set, exactly as
  §3's correction box says of the earlier grids. Board **#3706**, **#3707**.

> ### ⭑ 2026-08-27 — the two class lists are ONE rule, and it is now `[O]` on both
>
> Predicted in `work/w-regcells/PREREG.md` §0 **before the grid was compiled**:
> each class's list is **the class's scratch register, then the class's
> ARGUMENT registers descending, then the class's non-volatiles descending.**
> Class 0: `r11`, `r10…r3`, `r31…r14`. Class 1: `fp0`, `fp13…fp1`, `fp31…fp14`.
> **A port needs one list generator, not two tables**, and this is the shape to
> check class 5 (VMX) against when `FUN_10bfb00d`'s run-time fill is read.
> Board **#3709**.
