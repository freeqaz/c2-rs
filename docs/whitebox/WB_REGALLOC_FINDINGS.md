# WB_REGALLOC — the register-choice policy and the instruction-order policy

> **PROVENANCE — DISASSEMBLY-DERIVED.** Lane WB-D of
> [`CAMPAIGN_2026-08-08_GENERATORS.md`](CAMPAIGN_2026-08-08_GENERATORS.md).
> Every address below is an absolute VA in the exact image pinned in
> [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0 —
> `sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`,
> verified at the top of this lane. This is **navigation** until a row lands in
> [`DISCLOSURE.md`](DISCLOSURE.md). **The obj is the sole judge** (method doc §7).

PREREG: [`WB_REGALLOC_PREREG.md`](WB_REGALLOC_PREREG.md), committed **before the
first grep of `~/ghidra-projects/export/c2/`**. Scored in §8.

---

## 1. The stages, located (deliverable 1)

`c2_tus.tsv` names the backend's own translation units. The relevant band is
the classic Microsoft UTC `p2` back end:

| TU | anchor | what it is |
|---|---|---|
| `fg.c` | `10b36133` | flow graph |
| `dag.c` | `10b3219f` | expression DAG |
| `tuple.c` | `10bd398a` | the tuple IR (c2's IR is a **tuple list**, not a tree) |
| **`color.c`** | **`10b2c21d`…`10b30517`** | **the register allocator — graph colouring, ~7 100 lines** |
| `globregs.c` | `10b55eae` | global (cross-block) register promotion |
| `regasg.c` | `10bc58d5` | register assignment (reader-side) |
| `stack.c` | `10bd0c77` | stack-slot allocation |
| `lower.c` / `lowersmd.c` | `10c053e7` / `10c23539` | lowering; machine-dependent lowering |
| `code.c` | `10bf9f15` | prologue/epilogue and the per-function register environment |
| `mdlist.c` / `list.c` | `10c11060` / `10b709b8` | the `/FAsc` listing writers |
| `factor.c` | `10b34a89` | tail merging (a *block*-level reorder) |

**There is no `sched.c`, and no instruction scheduler.** The whole
stall-modelling apparatus in the image (`Dependency stall`, `Vector dependency
stall`, `Stalled for microcode instruction`, `Stalled for non-pipelined
instruction`, at `0x10b12af8`–`0x10b12bd4`) is reached only from `0x10b6fedd`
and `0x10b71d8f`, which are the **`/QXSTALLS` listing-comment writers**. The
`-schdat#` option (`0x10b139d0`) writes `0x10c2eb40`, and **`0x10c2eb40` has
zero readers in the image** — it is a dead switch in this build.

The per-function phase pipeline is `FUN_10b7d85e` @ `0x10b7d85e` (each phase
bracketed by the timer `FUN_10bec297`), driven from `compile-one-function`
`0x10b7ef55`.

---

## 2. The register NUMBERING — the table that makes everything else readable

`0x10b181c0` is a `char*` array of PPC register names. **Its index is c2's
register number**, and every register-valued field in the machine IR is one of
these indices.

| index | register |
|---:|---|
| `0` | `noreg` |
| `1` | `r0` |
| `2` | `sp` (r1) |
| `3` | `toc` (r2) |
| `4`…`13` | **`r3`…`r12`** |
| `14` | `r13` |
| `15`…`32` (`0x0f`…`0x20`) | **`r14`…`r31`** |
| `33` | `d0` |
| `34`…`65` (`0x22`…`0x41`) | `fp0`…`fp31` |
| `66` (`0x42`) | `cr` (whole) |
| `67`…`74` (`0x43`…`0x4a`) | **`cr0`…`cr7`** |
| `75`…`80` | `fpscr`, `mq`, `xer`, `so`, `ov`, `ca` |
| `81`, `82` | `rtcu`, `rtcl` |
| **`83` (`0x53`)** | **`lr`** |
| `84` (`0x54`) | `ctr` |
| `85` | `msr` |
| `229`…`356` (`0xe5`…`0x164`) | `vr0`…`vr127` |

Two independent confirmations that the index really is the register number:

* `FUN_10bfebf7` (`WB_FRAME_FINDINGS.md` §2.3) sets a callee-saved bit for
  register numbers `0x0f..0x20`, and glosses that window as `r14..r31`. It is
  `r14..r31` on this table **exactly**.
* `FUN_10c04faf` (§4) walks the per-register descriptor array with stride
  `0x60` from `0x10c2f088`, starting at descriptor byte-offset `0x1920`, and
  the loop counter it keeps in parallel starts at `0x43`. `0x1920 / 0x60 =
  0x43`, and the four indices it keeps (`0x43`, `0x44`, `0x49`, `0x4a`) are
  `cr0`, `cr1`, `cr6`, `cr7` — the four volatile CR fields of the PowerPC ABI.
  A wrong table would not land on the ABI.

### 2.1 A CORRECTION to a campaign-1 reading

`WB_FRAME_FINDINGS.md` §2.4 records:

> `10bfed27` | `call 0x10c07910` with register `0x53` | **the frame-establish
> pseudo-register** — emits no PPC instruction

`0x53` is **`lr`**, not a frame-establish pseudo-register. §4 below shows
`0x10c04faf` reading `lr` out of the descriptor table into the volatile set,
and `0x10b1830c` is its name-table slot. The *behaviour* wb-frame observed (no
PPC instruction is emitted for that call) may well be right; the **name is
wrong**, and the corollary — that `DAT_10c6fd9c` is "the LR pseudo-register"
(`W-FRAME.tsv`, `WB_FRAME_FINDINGS.md` §2.2) — is wrong too. §3.2 shows
`DAT_10c6fd9c` is the **frame-pointer register number**.

---

## 3. The register-CHOICE policy (deliverable 2)

### 3.1 The allocatable set and its ORDER — `0x10c37de0`

`FUN_10bfb0fa` @ **`0x10bfb0fa`** builds the class-0 (GPR) allocatable set from
a **zero-terminated, ordered array of register numbers**, choosing one of three
arrays:

| condition | array | contents (register numbers → registers) |
|---|---|---|
| POGO instrumented (`DAT_10c3de20 == 1`) | **`0x10c37eb8`** | `12…4, 32…17` → `r11…r3, r31…r16` |
| default | **`0x10c37de0`** | `12,11,10,9,8,7,6,5,4, 32,31,30,29,28,27,26,25,24,23,22,21,20,19,18,17,16,15` |
| `-QGPRReserve` (`DAT_10c2e980 != 0`) | **`0x10c37e50`** | `12…4, 32…18` → `r11…r3, r31…r17` |

Decoded through §2, the **default allocation order is**

> **`r11, r10, r9, r8, r7, r6, r5, r4, r3, r31, r30, r29, r28, r27, r26, r25,
> r24, r23, r22, r21, r20, r19, r18, r17, r16, r15, r14`**

— volatiles first, **descending from r11**; then callee-saved, **descending
from r31**. `r12`, `r13`, `r0`, `sp`, `toc` are in **no** list and are never
allocatable. The ordered array is kept alongside the bitset in
`PTR_DAT_10c385c4 + 4*class` (class 0 = GPR at `0x10c385c4`, class 1 = FPR at
`0x10c385c8`, class 5 = VMX at `0x10c385d8`; the class map is `0x10b022cc`,
which sends operand-type nibbles `1..4,6 → 0`, `5 → 1`, `0xc → 5` and
everything else to `-1` = not register-allocated).

**`DAT_10c2e980` is `-QGPRReserve`**, and this settles the item
`WB_FRAME_FINDINGS.md` §6 held as *"that flag's meaning is unknown"*. The
option-descriptor table built at `0x10c2a038` pairs the name string
`0x10b13c58` (`"-QGPRReserve"`) with the variable `0x10c2e980`. What it
reserves is exactly **`r14`, `r15`, `r16`**: the alternate list drops them, and
`FUN_10c04faf` independently adds register numbers `0x0f..0x11` to a "reserved"
bitset (`DAT_10c6fda4`) when the flag is set. `-QVMXReserve` (`0x10c2e97c`)
reserves `0x125..0x144` = `vr64..vr95`.

### 3.2 The frame-pointer interaction — `0x10c04faf`

```
if ((fn[0x25] & 0x500400) == 0 && (fn->sym[0x20] & 0x1000) == 0)
        DAT_10c6fd9c = 2;        /* sp  — NO frame pointer */
else    DAT_10c6fd9c = 0x20;     /* r31 — frame pointer     */
```

and the tail of `FUN_10bfb0fa`:

```
if (DAT_10c6fd9c != 2) clear_bit(gpr_set, DAT_10c6fd9c);  /* FP is not allocatable */
else                   set_bit(gpr_set, 0x20);            /* r31 IS allocatable    */
```

So **`r31` drops out of the allocatable set exactly when the function needs a
frame pointer** (the `0x500400` bits are the EH / `alloca` family, the same
family `WB_FRAME_FINDINGS.md` saw at `10bff573`).

### 3.3 The volatile (call-clobbered) set — `0x10c04faf`, `DAT_10c6fda8`

Built by literal range, and it is the PowerPC ABI:

| range added | decoded |
|---|---|
| `4 .. 0xd` | **`r3`…`r12`** |
| `0x22 .. 0x2f` (with `-QFPU`) | `fp0`…`fp13` |
| `0xe5 .. 0xf2` (default) | `vr0`…`vr13` |
| `0x43`, `0x44`, `0x49`, `0x4a` | **`cr0`, `cr1`, `cr6`, `cr7`** |
| `0x54` + `PTR_DAT_10c30e90` | `ctr`, and one more special (`lr`) |

`cr2`…`cr5` are therefore callee-saved, which is why a c2 obj that uses a
non-`cr0` field for an ordinary compare uses `cr6`, not `cr2`.

### 3.4 THE SELECTOR — `FUN_10b2e7f8` @ **`0x10b2e7f8`**

This is the whole policy, and it is short enough to state exactly. It is the
only reader of the ordered list `0x10c385c4`.

1. `memset(&DAT_10c435e8, 0, 0x594)` — a **cost array, one `int` per register
   number**. `0x594 = 1428 = 357 × 4`, and 357 is exactly `|register table|`
   (`0`…`356`, `noreg`…`vr127`). Independent confirmation of §2.
2. **Penalties.** For each already-coloured interfering neighbour, `cost[reg]
   += weight`. For a neighbour whose candidate set has been reduced to a
   **single** register, `cost[that register] += 100 × degree` — a large "do not
   take the only register a constrained neighbour can still use" term.
3. **Preferences.** For each entry on *this* live range's own preference list
   (`param_1 + 0x38` — the copy/coalesce list), `cost[reg] -= weight`.
   **Negative cost = preferred**, and this is what makes an argument stay in
   the argument register.
4. `FUN_10bfc02f` (`0x10bfc02f`) is a machine-dependent hook that runs here. It
   touches **class 5 (VMX) only**, and only under `-QVMX128`. **For GPRs and
   FPRs the ordered preference list is the fixed image-resident array**, which
   is what makes §3.1 a rule rather than a snapshot.
5. **The selection loop**, walking the ordered array from index 0 forward:

   ```
   best = none;
   for (i = 0; list[i] != 0; i++)
       if (candidate_set has list[i] && (best == none || cost[list[i]] < best_cost))
           { best = list[i]; best_cost = cost[best]; }
   ```

   The comparison is a **strict `<`**, so **ties go to the EARLIEST register in
   the list order**.

Stated as one sentence:

> **c2 picks the minimum-cost register among those the interference graph still
> allows, where cost is (interference and constraint penalties) minus (copy
> preferences), and every tie is broken by the fixed order
> `r11, r10, …, r3, r31, r30, …, r14`.**

### 3.5 What this says about the w-osfinfo `r11`-vs-`r10` fact

`_free_osfhnd` materialises two global addresses. Neither has an incoming
argument, so neither has a copy preference, so both cost 0 and **both are
decided purely by list order**. The first takes the head of the list, `r11`.
The second is formed while the first is **still live** — `lis r11,0` / `lwz
r11,0(r11)` at `+0x14`/`+0x18`, and the pair for `<table>` opens at `+0x28`
with the limit still in `r11` — so `r11` is out of the candidate set and the
next entry in the list, **`r10`**, wins.

This is PREREG **P2.2** as written, and it makes the shipped walk's re-key
(w-osfinfo #1762, "the open slot carries that register") a **consequence of a
rule** rather than a patch.

---

## 4. The ORDERING policy (deliverable 3)

**Claim (O).** Emitted word order is a **linear walk of one per-function
machine-instruction list**; it is the order the lowering built, and **no pass
in this c2.dll reorders instructions within a block for latency**.

Evidence, in decreasing strength:

1. **No scheduler exists.** No `sched.c` TU (§1); `-schdat#`'s variable
   `0x10c2eb40` has zero readers; every stall string in the image is reached
   only from the two `/QXSTALLS` listing writers.
2. **The list is walked linearly and expanded in place.** `FUN_10bff507`
   (`WB_FRAME_FINDINGS.md` §2.2) computes the whole prologue flag word in *one
   linear scan of the instruction list*. The final expansion pass is a giant
   `switch` on `instr->opcode` (`param_2[1]`) rewriting each pseudo-op **in
   situ** — `0x2f4` and `0x2f0` call the prologue driver `0x10bff95c` via
   `0x10c216f5` / `0x10c21719` without moving anything.
3. **All reordering in the image is at block granularity or is pattern
   replacement**: `factor.c` (tail merge), and POGO's `pogoopt.c` whose own
   narration says so (`PEEP:\tConditional branches reordered near line %lu`,
   `PEEP:\tReordered conditional/unconditional branch pair`, `0x10b170a4` /
   `0x10b170d8`) — and those three messages are POGO-only.

**Named traversal**: the emitted order is a **pre-order walk of the block list
in flow-graph construction order, and within a block the tuple list in
lowering order** — where "lowering order" is the order `lower.c`'s expansion
switch leaves behind, which is the order `dag.c`/`tuple.c` built. Registers do
not participate: **selection → order → registers**, so a port can reproduce
order without reproducing allocation (PREREG P3.5).

**What is NOT claimed, so absence does not read as coverage.** This lane did
*not* read `dag.c`'s tree-to-tuple walk and cannot say whether it is
left-to-right, Sethi–Ullman, or something else. "The emitter does not reorder"
is a much weaker claim than "the port can predict the order", and §6/§7 grade
only the former.

---

## 5. The class map and the comparison signedness (#1788)

`0x10b022cc` is a 13-entry `int` array indexed by the operand-type nibble
(`(*(u16*)(operand + 10)) >> 12`):

```
idx:  0   1  2  3  4  5  6   7   8   9  10  11  12
val: -1   0  0  0  0  1  0  -1  -1  -1  -1  -1   5
```

So **the type nibble reaches the allocator as a register class** (0 = GPR,
1 = FPR, 5 = VMX; `-1` = not register-allocated). This is the same nibble
`#1788` says selects `cmpwi` vs `cmplwi`, read at a different consumer — it is
carried on **every operand**, not recomputed. Signedness is therefore fixed at
selection time and neither the allocator nor any later pass revisits it
(PREREG P2.6). This is navigation; §6 cell C1/C2 is its obj check.

---

## 6. THE OBJ-CHECK — FROZEN BEFORE THE FIRST `cl.exe` OF THIS LANE

Source: [`grids/wb-regalloc/regorder_grid.cpp`](grids/wb-regalloc/regorder_grid.cpp),
one COMDAT per cell. To be compiled with the real `cl.exe` 16.00.11886.00 under
wibo at the workload mode `/nologo /c /GR /O1 /Oi /EHsc` and read with
`scripts/gt_dump.py`.

**Every cell is outside every shipped port class** (`c2-core` ships
straight-line int add-chains, tail calls, one framed non-leaf call, and four
transcribed body shapes: `if_call_join`, `guard_chain_shared_tail`,
`alloc_init_or_fail`, `osf_handle_guard`, `xlrc_create_guard`). No cell is a
loop-free single-call body and none is a transcribed shape.

### 6.1 The rivals

| id | register-choice rival |
|---|---|
| **R0** | **(this lane's §3.4 reading)** min-cost over the candidate set; cost = interference penalties − copy preferences; **ties broken by the fixed order `r11,r10,…,r3,r31,…,r14`** |
| R1 | first-free **ascending** from `r3` (the naive ABI-order allocator) |
| R2 | first-free **descending** from `r12` (i.e. `r12` IS allocatable) |
| R3 | linear-scan with no preference term — argument values get moved out of `r3..r10` immediately |

| id | ordering rival |
|---|---|
| **O0** | **(this lane's §4 reading)** no within-block reordering: emitted order = lowering order = source order for independent statements |
| O1 | a list scheduler interleaves independent load-use chains |

### 6.2 Frozen predictions

**These are committed before the grid is compiled.** `—` means the cell does
not discriminate that rival.

| cell | shape | R0 (this lane) | R1 | R2 | R3 |
|---|---|---|---|---|---|
| **G1** | one global, one temp | **`r11`** | `r3` | `r12` | `r11` |
| **G2** | two globals live at once | **`r11` and `r10`** | `r3`,`r4` | `r12`,`r11` | `r11`,`r10` |
| **G3** | three globals live at once | **`r11`,`r10`,`r9`** | `r3`,`r4`,`r5` | `r12`,`r11`,`r10` | same as R0 |
| **G4** | four globals live at once | **`r11`,`r10`,`r9`,`r8`** | `r3`…`r6` | `r12`…`r9` | same as R0 |
| **N1** | `p[0]+1`, arg in `r3` | **`r3` only** | `r3` | `r12` | **not `r3`** |
| **N2** | two pointer args | **`r3`,`r4` only** | `r3`,`r4` | `r12`,`r11` | not `r3`/`r4` |
| **N3** | three pointer args | **`r3`,`r4`,`r5` only** | same | `r12`… | not |
| **N4** | four pointer args | **`r3`…`r6` only** | same | `r12`… | not |
| **L3** | loop with a call in the body | **first callee-saved taken is `r31`, second `r30`, third `r29`** | `r14`,`r15`,`r16` | `r31`,`r30`,`r29` | — |
| **P1** | 12 live ints, > 9 volatiles | **all of `r11`…`r3` appear, then `r31`,`r30`,… — and `r12` NEVER appears** | `r3`…`r12` then `r14`… | `r12` appears | — |

| cell | shape | O0 (this lane) | O1 |
|---|---|---|---|
| **S1** | two independent load-use chains | **`lwz p` … `stw out[0]` … `lwz q` … `stw out[1]`, not interleaved** | the two `lwz` adjacent at the top |

Non-rival predictions registered at the same time (scored, no rival column):

| # | cell | prediction |
|---|---|---|
| **F1** | C1 `int x < 10` | `cmpwi` (signed) |
| **F2** | C2 `unsigned x < 10u` | `cmplwi` (unsigned) |
| **F3** | M1 multi-way if | every compare uses **`cr0`** |
| **F4** | M2 dense switch | a jump table via `mtctr`/`bctr`, and the index register is **`r11`** (head of the list; it is a materialised temp, not an argument) |
| **F5** | L1 counted loop | the loop is a `mtctr` / `bdnz` countdown, **not** an `addi`+`cmpw`+`bne` |
| **F6** | L3 | a frame is opened (`stwu r1,−F(r1)`) — wb-frame §5.5 R1, a non-tail call |
| **F7** | every cell | **`r12` appears only as the LR shuttle** (`mflr r12` / `mtlr r12`), never as a value |
| **F8** | every cell | **`r13` never appears at all** |

### 6.3 The three named functions for deliverable 4

Per the contract, ≥3 functions outside every shipped class with a **full**
register-and-order prediction, frozen here, graded in §7:

* **`wbr_glob3`** (three simultaneously-live values with no argument
  preference). Predicted words, in order:
  `lis rA,<g0>@ha` / `lwz r11,<g0>@l(rA)` / `lis rB,<g1>@ha` /
  `lwz r10,<g1>@l(rB)` / `lis rC,<g2>@ha` / `lwz r9,<g2>@l(rC)` /
  `addi r11,r11,1` / `addi r10,r10,2` / `mullw r11,r11,r10` /
  `mulli r9,r9,3` / `add r3,r11,r9` / `blr`.
  **The graded part is the register set `{r11, r10, r9}` and no others besides
  `r3` and the address bases**, plus "no reordering across the three loads".
* **`wbr_loop_call`** (a loop whose body calls). Predicted: frame opened;
  loop-carried accumulator in **`r31`**, the array pointer in **`r30`**, the
  count in **`r29`** or in `ctr`; `bl wbr_extf`; the prologue saves exactly the
  callee-saved registers it uses, highest first.
  **The graded part is that the callee-saved registers taken are the TOP of the
  file (`r31`, `r30`, `r29`) and not the bottom (`r14`, `r15`, `r16`).**
* **`wbr_pressure`** (12 live ints). Predicted: the volatile half of the list is
  exhausted **`r11` → `r3`** before any callee-saved register is touched, and
  the callee-saved ones are then taken **`r31` → downward**; **`r12` and `r13`
  never appear.**

### 6.4 Separation assertion, before the run

| pair | discriminating cells | n |
|---|---|---|
| R0–R1 | G1, G2, G3, G4, L3, P1 | 6 |
| R0–R2 | G1, G2, G3, G4, N1…N4, P1 | 9 |
| R0–R3 | N1, N2, N3, N4 | 4 |
| R1–R2 | G1…G4, N1…N4, L3, P1 | 10 |
| R1–R3 | N1…N4, L3 | 5 |
| R2–R3 | G1…G4, N1…N4 | 8 |
| O0–O1 | S1 | **1** |

Minimum over the register pairs = **4**. **O0–O1 has ONE cell and that is
declared as insufficient in advance**: a single cell cannot separate "there is
no scheduler" from "there is a scheduler that agreed here". §4's claim rests on
the disassembly (no `sched.c`, `-schdat#` dead), and S1 is a *consistency*
check, not a discriminator. Stated here so that a green S1 is not later read as
coverage.

---

## 7. Results

Compiled with real `cl.exe` 16.00.11886.00 under wibo,
`/nologo /c /GR /O1 /Oi /EHsc`, one obj, 23 sections, read with
`scripts/gt_dump.py`. Word streams are in
`work/wb-regalloc/run/grid.txt` (not committed — it is a dump of an obj).

### 7.1 The register-choice rivals

| cell | emitted register set | R0 | R1 | R2 | R3 |
|---|---|---|---|---|---|
| G1 | **`r11`** | ✅ | ✗ | ✗ | ✅ |
| G2 | **`r11`, `r10`** | ✅ | ✗ | ✗ | ✅ |
| G3 | **`r11`, `r10`, `r9`** (+ `r8` for a 4th temp) | ✅ | ✗ | ✗ | ✅ |
| G4 | **`r11`, `r10`, `r9`, `r8`** (+ `r7`) | ✅ | ✗ | ✗ | ✅ |
| N1 | `r11` (temp), `r3` (arg + return) | ✗ *(see 7.3)* | ✗ | ✗ | ✗ |
| N2 | `r11`, `r10`, `r3`, `r4` | ✗ | ✗ | ✗ | ✗ |
| N3 | `r11`, `r10`, `r9`, `r3`…`r5` | ✗ | ✗ | ✗ | ✗ |
| N4 | `r11`, `r10`, `r9`, `r8`, `r3`…`r6` | ✗ | ✗ | ✗ | ✗ |
| L3 | callee-saved taken = **`r31`, `r30`, `r29`** (`bl __savegprlr_29`) | ✅ | ✗ | ✅ | — |
| P1 | **`r11`…`r3`** all used, then **`r31`, `r30`, `r29`, `r28`, `r27`** (`bl __savegprlr_27`); `r12` only as `mflr r12`; **no `r13`** | ✅ | ✗ | ✗ | — |

| rival | refuted by | verdict |
|---|---|---|
| **R0** — min-cost, ties by the fixed order `r11…r3, r31…r14` | N1–N4 refute the *cells as written* (§7.3), not the rule | **SURVIVES on G1–G4, L3, P1 (6/6); its N-cells are a scoring MISS** |
| R1 — first-free ascending from `r3` | G1, G2, G3, G4, L3, P1 | **REFUTED, 6 cells** |
| R2 — first-free descending from `r12` | G1–G4, N1–N4, P1 | **REFUTED, 9 cells** — `r12` never holds a value in any of the 15 cells |
| R3 — no preference term | N1–N4 (arguments *do* stay in `r3`…`r6`) | **REFUTED, 4 cells** |

### 7.2 The ordering rival, and the non-rival predictions

| # | prediction | emitted | verdict |
|---|---|---|---|
| **O0** | S1's two chains not interleaved | `lwz r11,0(r4)` / `mulli` / `stw` / `lwz r11,0(r5)` / `mulli` / `stw` | **HIT** (1 cell, declared insufficient in §6.4) |
| F1 | C1 → `cmpwi` | `cmpwi 6,r3,10` | **HIT** |
| F2 | C2 → `cmplwi` | **no compare at all** — `li r11,10` / `subc` / `subfe` / `addi` | **MISS** (premise failed; scored as a miss, see §7.4) |
| F3 | M1 compares use **`cr0`** | every compare uses **`cr6`** | **MISS — retraction, §7.5** |
| F4 | M2 → jump table via `mtctr`/`bctr`, index in `r11` | **no jump table**: a 3-level binary decision tree of `cmplwi cr6` + `bt` | **MISS — §7.6** |
| F5 | L1 is `mtctr`/`bdnz` | `mtctr r4` / `lwzu` / `add` / `bdnz` | **HIT** |
| F6 | L3 opens a frame | `stwu r1,-112(r1)` | **HIT** |
| F7 | `r12` only as the LR shuttle | true in all 15 cells | **HIT** |
| F8 | `r13` never appears | true in all 15 cells | **HIT** |

### 7.3 The N-series MISS, stated as a miss

§6.2 predicted N1 as "**`r3` only**". The obj is
`lwz r11,0(r3)` / `addi r3,r11,1` / `blr` — **`r11` is there**. Same for
N2 (`r11`,`r10`), N3 (`r11`,`r10`,`r9`), N4 (`r11`,`r10`,`r9`,`r8`).

That is a miss on **my prediction sheet**, and it is scored as one. What went
wrong is worth naming precisely, because it is the failure mode this lane is
most likely to repeat: I applied a copy preference **that the rule does not
give**. The *pointer* `p` has a preference to `r3` (it arrives there, and it
does stay there — `lwz …,0(r3)`). The **loaded value `p[0]` is a brand-new live
range with no copy relation to anything**, so its cost is 0 everywhere and
§3.4's tie-break gives it the head of the list, `r11`. The rule predicted `r11`;
**I** predicted `r3`.

So: **R0 the rule is not refuted by N1–N4; R0-as-I-applied-it is.** The
distinction is recorded rather than used to rescue the cell — the cell scores
MISS, and R0 keeps only the six cells it was actually right about in advance
(G1–G4, L3, P1). Per method doc §7 this lane does **not** re-score the N cells
in R0's favour after the fact.

### 7.4 C2 — the premise failed, and the cell is still scored a miss

The cell was designed as "an unsigned compare". c2 emitted **no compare**: it
lowered `x < 10u` to a carry trick (`subc` / `subfe`). By the `assumption
unmet` rule this lane registered in §6 (inherited from wb-frame §5.2) the cell
could be excluded. It is scored **MISS** instead, because the prediction as
written names a word that is not in the obj. **#1788 gets no support and no
refutation from this grid** — stated so absence does not read as coverage.

### 7.5 RETRACTION: the CR-field claim, and what replaces it

§3.3 read `cr0`, `cr1`, `cr6`, `cr7` as the volatile CR fields (correct — the
`0x43`/`0x44`/`0x49`/`0x4a` loop at `0x10c04faf` says so), and F3 predicted
**`cr0`** for an ordinary compare. **Every explicit integer compare in the grid
uses `cr6`** — `cmpwi 6,…` and `cmplwi 6,…` in L1, L2, L3, M1, M2, C1. `cr0`
appears only as the implicit result of a **record-form** instruction
(`addic. r31,r31,-1` in L3, branched on with `bf 2`).

**F3 is retracted.** The replacement is read from the binary and is *not* an
allocation fact at all:

* `0x10c385c4` is an 8-entry array of per-class ordered lists. **Only class 0
  (GPR, `0x10c37de0`) and class 1 (FPR, `0x10c37f20`) are image-initialised;
  class 5 (VMX) is filled by `FUN_10bfb00d`; classes 2, 3, 4, 6, 7 are NULL.**
  There is **no CR register class**.
* `cr6`'s per-register descriptor is `0x10c2f088 + 0x60·0x49 = 0x10c30be8`, and
  the lowering **assigns that descriptor as a literal** — e.g.
  `*(instr->dst->sym) = &DAT_10c30be8` at `0x10c00445`, `0x10bf0882`,
  `0x10bf522f`, `0x10c195a2`.

> **`cr6` is a lowering constant, not an allocator choice.** `cr0` is reached
> only through record-form instructions.

This is exactly the shape of error §7 of the method doc warns about: the
instruction reading (`which CR fields are volatile`) was right and the
*inference* about behaviour was wrong.

### 7.6 M2 — c2 does not build a jump table here

Six dense `case`s, `0..5`, and c2 emitted **no jump table**: three `cmplwi cr6`
comparisons in a binary search over the value, then seven `li`/`blr` leaves.
The switch value is compared **unsigned** even though the C type is `int`.

Two further order facts this cell gives away for free, both **against** a naive
reading of §4:

1. The seven leaf blocks come out in **reverse case order** — `default`, `66`,
   `55`, `44`, `33`, `22`, `11`. **Block order is not source order.** §4's
   phrase *"flow-graph construction order"* survives only because construction
   order for a switch is not source order; as a rule a port could use, it is
   **not established**, and this lane does not claim it.
2. M1's four arms *do* come out in source order. So the two cases differ, and
   nothing in this grid tells a port which one it is looking at.

### 7.7 Both flag modes, and they are byte-identical

The grid was compiled twice — at the workload mode `/nologo /c /GR /O1 /Oi
/EHsc` and at `/nologo /O1 /GS- /c`, wb-frame §5.4's second mode. The two objs
are **identical in every byte of every code section**, 23 sections, 81 symbols,
4 261 bytes, same section sizes and same disassembly on all 15 cells.

That is worth stating because it bounds a whole family of "but the flags"
objections: on these shapes neither `/Oi`, `/EHsc`, `/GR` nor `/GS-` moves a
single register or a single word. It does **not** extend to `/O2` or to POGO,
neither of which this lane compiled.

### 7.8 The three named functions of deliverable 4

| function | frozen prediction (the graded part) | emitted | verdict |
|---|---|---|---|
| **`wbr_glob3`** | register set is exactly `{r11, r10, r9}` besides `r3` and the address bases; no reordering across the three loads | `lis r11 / lis r10 / lis r9 / lwz r11 / lwz r10 / lwz r9 / addi r8,r11,2 / addi r10,r10,1 / mulli r11,r9,3 / mullw r10,r8,r10 / add r3,r10,r11 / blr` | **HIT on the graded part** — `{r11,r10,r9}` are the loaded values and the address bases are the *same three registers*; a fourth temp took `r8`, the next list entry. The three loads are in one run, unreordered. My literal word sequence differed (c2 hoisted all three `lis` above all three `lwz`, and `+1`/`+2` landed on different temps) — **the word-for-word half is a MISS.** |
| **`wbr_loop_call`** | callee-saved taken are the **TOP** of the file (`r31`,`r30`,`r29`), **not** the bottom (`r14`,`r15`,`r16`) | `bl __savegprlr_29` — `r29`, `r30`, `r31` and LR | **HIT** |
| **`wbr_pressure`** | volatiles exhausted `r11`→`r3` before any callee-saved; callee-saved then `r31` downward; `r12`/`r13` never appear | all of `r3`…`r11` used; `bl __savegprlr_27` = `r27`…`r31`; `r12` only in `mflr r12`; no `r13` | **HIT** |

**2 of 3 named functions are clean hits; the third hits on its graded claim and
misses on its word-for-word claim.** The success floor —
*one policy reading surviving a frozen check on ≥1 function outside every
shipped class* — is **CLEARED** by `wbr_loop_call` and `wbr_pressure`
independently.

---

## 8. PREREG score

`H` hit · `M` miss · `U` unscoreable (premise did not occur).

| # | prediction | verdict | note |
|---|---|---|---|
| P0.1 | the floor is cleared | **H** | §7.8 |
| P0.2 | ordering survives, register does **not** | **M** *(optimistic in reverse)* | the opposite happened — the **register** policy is the one that survived a 6-cell frozen check; ordering has one consistency cell and two counter-facts (§7.6) |
| P1.1 | a separately-named regalloc TU exists | **H** | `color.c`, `10b2c21d`…`10b30517` |
| P1.2 | a real instruction scheduler exists | **M** *(registered optimistic)* | none: no `sched.c`, `-schdat#`'s var has 0 readers |
| P1.3 | selection is fused into lowering, not its own band | **M** *(registered pessimistic)* | `lower.c` **and** `lowersmd.c` are separate named TUs |
| P1.4 | the allocator is global, not per-tree | **H** | interference bitsets, cost array, `color.c` |
| P2.1 | temps descend from `r11`, `r12` reserved | **H** | `0x10c37de0`, and G1–G4 / P1 |
| P2.2 | the osfinfo `r10` is "`r11` still live", not a per-form rule | **H** | §3.5, and G2 reproduces it |
| P2.3 | argument registers pre-coloured, positional | **H** | N1–N4: arguments stay in `r3`…`r6` |
| P2.4 | first callee-saved taken is `r31`, then `r30` | **H** | L3, P1 |
| P2.5 | `cr0` by default; `cr6` not preferred | **M** | **the reverse** — §7.5, retracted |
| P2.6 | signedness fixed at selection, type nibble carried on the operand | **H** *(navigation only)* | `0x10b022cc`; **no obj support**, C2 gave none |
| P2.7 | no allocator-invented spill at `/O1` on these shapes | **H** | P1 has 12 live values and spills nothing — it saves 5 callee-saved instead |
| P3.1 | one per-function instruction list, linear walk | **H** | `0x10bff507`, the in-place expansion switch |
| P3.2 | block order = construction order, not recomputed | **M** | M2's leaves come out in **reverse** case order (§7.6) |
| P3.3 | a list scheduler reorders within a block | **M** *(registered optimistic)* | there is no scheduler |
| P3.4 | the scheduler is off at `/O1` | **U** | premise (P3.3) did not occur |
| P3.5 | order does not depend on register identity | **H** | selection → order → registers; nothing in `color.c` moves an instruction |
| P4.1 | ≥3 functions graded | **H** | §7.8 |
| P4.2 | order: ≥2 of 3 hit | **H** | all three preserve their emitted order under the "no within-block reordering" reading |
| P4.3 | registers: ≤1 of 3 hit | **M** *(registered pessimistic)* | **3 of 3** hit on the graded register claim |
| P4.4 | at least one miss forces a retraction | **H** | F3 / P2.5, §7.5 |
| P5.1–P5.3 | the judgment | scored in §9 | |
| P5.4 | the binding constraint is the reader, not the emitter | **H** | §9.3 |

**Score: 15 H · 7 M · 1 U.** Five of the seven misses are in the **registered**
direction (P0.2, P1.2, P1.3, P3.3, P4.3); two are not (P2.5, P3.2) and both are
retractions in §7.5 / §7.6.

Board #770's streak was ~10 optimistic / 2 pessimistic / 1 hit. This lane adds
**3 optimistic (P1.2, P3.3, and P0.2's inversion), 2 pessimistic (P1.3, P4.3)**.

---

## 9. THE JUDGMENT — can a general lowering be derived for a class? (deliverable 5)

### 9.1 The answer

> **Yes for register assignment. No for instruction selection and block order —
> and those, not registers, are what a class lowering actually needs.**

The register half is *done*, in the strong sense: §3.4 is a complete, short,
deterministic policy that took six frozen cells without a scratch, and the two
inputs it needs (an interference relation and a copy-preference set) are
computable from the port's own IR. A port that implements

```
candidates = allocatable(class) ∖ conflicts(range)
cost[r]    = Σ interference penalties − Σ copy preferences
pick        the first r in [r11,r10,…,r3,r31,r30,…,r14] minimising cost
```

reproduces c2's register choice on every cell in this grid **without a
per-function transcription**. That is a genuine derivation, and it is the first
one this project has for any emitter question.

The other half is where the answer turns negative, and the grid says so
loudly:

* **M2**: a six-case dense `switch` is not a jump table, it is a **binary
  decision tree** whose fan-out and pivot choices this lane did not read and
  cannot predict, whose leaf blocks come out in **reverse** source order, and
  whose comparisons are **unsigned** for a signed C type.
* **C2**: `x < 10u` producing `1` or `2` is not a compare-and-branch at all, it
  is a four-word branchless carry idiom. There is a **pattern library** in
  `cgintrin.c` / `lowersmd.c` that this lane did not enumerate.
* **L1**: the counted loop was rewritten into a `lwzu`-walked pointer with the
  trip count in `ctr`, `cmpwi cr6` + `bf 25` as a zero-trip guard, and the
  index variable **gone**. That is `lur.c` (15 115 lines) plus `globlopt.c`
  (13 477), neither of which this lane opened.

**The register policy is the *last* thing a class lowering needs, not the
first.** Once selection and block order are right, registers follow from a
30-line rule. Until they are, a correct register policy assigns correct
registers to the wrong instructions.

### 9.2 What a class lowering needs, concretely

For the first class to be a *derivation* rather than a transcription, in order:

1. **The pattern set for the class's operators** — which C constructs c2 turns
   into which word idioms. `wbr_cmp_u` shows this is not optional: one
   unremarkable comparison produced an idiom the port has never emitted.
2. **The loop normal form** — `lur.c`'s output shape: induction-variable
   elimination into `lwzu`, trip count into `ctr`, the `cmpwi cr6` zero-trip
   guard, `bdnz`. This is one *shape*, and the grid shows it is stable across
   L1/L2/L3, so it is derivable — but it must be read, and it was not read
   here.
3. **Block emission order** — M1 says source order, M2 says reverse. Both from
   the same compiler on the same day. Until a rule covers both, a port cannot
   place labels, and the label counter is load-bearing for the COFF symbol
   records (`LABEL_COUNTER.md`, w-xlr §3).
4. **Then** §3.4, which is free.

### 9.3 Predicted reach, and the first class to attempt

**First class: the counted `for` loop over one array with one accumulator** —
L1's shape. It is chosen not because it is the biggest but because it is the
only class in this grid whose normal form was **identical across three
different bodies** (L1, L2, L3 share `cmpwi cr6` guard / `addi ptr,-4` /
`mtctr` / `lwzu` / `bdnz`, differing only in the body between `lwzu` and
`bdnz`). A shape that is stable across three witnesses is the definition of a
class rather than a transcription.

**Predicted reach over the 124-TU reach-pool: `≤ 6`, and most likely `0`.**
PREREG P5.3 registered `≤ 6` pessimistically; the grid moved this lane
*further* down, and the reason is P5.4, which scored a hit:

> **A class lowering converts nothing until the port's IL READER accepts the
> class's constructs.** WB-A's finding stands — 48 of the frontier's 59
> functions die at the reader, before any emitter question is reachable, and
> `w-xlr` re-priced its own TU at **thirteen** refusals of which **ten were
> outside the frame/emitter entirely** (reader arms, `.sy` handling, COFF
> symbol order). A loop-class emitter with a perfect register policy adds a
> **capability**, not a conversion, and the conversion is gated on work this
> campaign did not touch.

So the honest form of the judgment is: **"yes, and it is worth building — but
it will measure zero on the first scan, and a lane that expects otherwise will
report a failure."** The register rule's value is that it makes every *future*
class cheaper, not that it converts one now. That is the same shape as the
`.bss` reversal rule and the label-counter table: infrastructure, priced as
infrastructure.

**Explicitly declined**: multi-way `if`s (M1/M2) as a class. Two cells, two
different block orders, and the switch lowering is an unread algorithm. #1767's
rule refuses a two-point fit and it should refuse this one.

---

## 10. Pre-drafted DISCLOSURE rows

Per `DISCLOSURE.md` step 5 the black-box alternative is preferred, and here it
is **partly available but not sufficient**: the *order* `r11,r10,…,r3,
r31,…,r14` can be re-derived from `grids/wb-regalloc/regorder_grid.cpp` alone
(G1–G4 and P1 exhibit it without any disassembly), but the **cost function**
and the **strict-`<` tie-break** cannot — no obj distinguishes "ties to the
earliest list entry" from "the only candidate". A code lane that ships the full
selector needs the row; a lane that ships only the order does not.

| # | Kind | What was adopted | Address in `c2.dll` | Adopted into | Commit | Notes |
|---|---|---|---|---|---|---|
| **W-REGALLOC-1** | **adoption-ready** | **The GPR allocation order `r11,r10,r9,r8,r7,r6,r5,r4,r3,r31,r30,…,r15,r14`, and that `r12`/`r13`/`r0`/`sp`/`toc` are never allocatable.** | **`0x10c37de0`** (the ordered array), `0x10c37e50` (`-QGPRReserve` variant, drops `r14`–`r16`), `0x10c37eb8` (POGO variant, drops `r14`–`r15`), `0x10c385c4` (the per-class list array), `0x10b181c0` (the register-name table that decodes the numbers) | *(nothing — this lane adopts no code)* | *(pending)* | **A black-box re-derivation exists** and should be preferred: cells G1–G4 and P1 of `grids/wb-regalloc/regorder_grid.cpp` exhibit the order against real `c2.dll` with no address. Carry this row only if the *numbers* or the *variant arrays* are copied. |
| **W-REGALLOC-2** | **route** | **Register choice is minimum-cost selection over the interference-allowed candidates, cost = interference/constraint penalties minus copy preferences, ties broken by a fixed per-class order.** The cost array is one `int` per register number and the comparison is strict `<`. | **`0x10b2e7f8`** (the selector), `0x10c435e8` (the `0x594`-byte cost array), `0x10b30517` (the interference walk that produces the candidate set), `0x10bfb0fa` (builds the class-0 set), `0x10c04faf` (volatile/reserved sets and the frame-pointer register) | *(nothing)* | *(pending)* | No obj in this project separates the tie-break from "only one candidate", so this **cannot** be established black-box today. Grey-zone rule applies. |
| **W-REGALLOC-3** | **route** | **`cr6` is a lowering constant for explicit integer compares — there is no CR register class.** `cr0` is reached only through record-form instructions. | `0x10c385c4` (classes 2,3,4,6,7 are NULL), `0x10c30be8` (`cr6`'s descriptor = `0x10c2f088 + 0x60·0x49`), and the sites that assign it literally: `0x10bf0882`, `0x10bf522f`, `0x10c00445`, `0x10c195a2` | *(nothing)* | *(pending)* | **The black-box alternative is complete and should be used instead**: every compare in the grid is `cr6`. The row exists only because §7.5 *retracted* a `cr0` prediction and the reason is in the binary. |

**Held, not proposed.** The FPR order at `0x10c37f20` decodes to
`fp0, fp13, fp12, …, fp1, fp31, fp30, …, fp14` — read, **never obj-checked**
(no cell in this grid uses floating point). It is navigation. So is the
operand-nibble → class map `0x10b022cc`, and so is the identity of the
`0x500400` frame-pointer flag family.

**Not claimed.** This lane did not read `dag.c`, `lur.c`, `globlopt.c`,
`cgintrin.c` or the switch lowering, and §9 is explicit that those are where a
class lowering actually lives.
