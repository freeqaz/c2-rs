# WB_CANDID — read **R1**: `DAT_10c400d4` is **FUNCTION-SCOPED**

> **PROVENANCE — DISASSEMBLY-DERIVED.** Every address is an absolute VA in
> `compilers/X360/16.00.11886.00/c2.dll`, sha256
> `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`, verified
> by this lane against both the repo copy and `~/ghidra-projects/bin/c2dll`
> before any address was read. See [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0,
> [`ref/README.md`](ref/README.md) §2 for the `[R]`/`[O]`/`[I]` legend.

Lane `w-read-r1`, 2026-08-22. Prereg: [`WB_CANDID_PREREG.md`](WB_CANDID_PREREG.md).
Spec: [`READ_PLAN_2026-08-21.md`](READ_PLAN_2026-08-21.md) §3 row R1, §5.2.
Funded by [`../DECISIONS_2026-08-22.md`](../DECISIONS_2026-08-22.md) decision 1.

---

## 0. The answer, in the one sentence R1 asked for

> **Candidate ids are PER-FUNCTION.** `DAT_10c400d4` is set to **1** at
> **`0x10b57676`**, the fourth store in `FUN_10b57633` — the global-register
> allocation phase — which the per-function back-end driver `FUN_10b7dc51`
> calls at `0x10b7dcb7`; **all four** of the counter's references in the image
> execute inside a single invocation of that driver, and the write is the first
> of them in program order. `[R]`, with a behavioural control at `[O]` (§4).

`WB_LIVE_FINDINGS.md:258-260` is **right**. `ref/P_REGALLOC.md:62,86,188` is
**wrong** in three places and is corrected in the same commit as this file.

---

## 1. The complete reference set — four sites, not three

The brief cited three sites; two of them (`0x10b2c1f1`, `0x10b2c21d`) do not
touch this global at all — they touch the **hash table** `0x10c43b80` that the
id keys. The counter's actual reference set is exactly four instructions, and
that is the whole reason this read is closed rather than suggestive.

Enumerated two independent ways, which agree exactly (`C2_MAP_METHOD.md` §1's
rule that no claim rests on Ghidra alone):

```
$ awk -F'\t' '$2=="10c400d4"' xrefs.tsv          # Ghidra flat export
10b54d57  READ         from_func 10b54d32
10b54d5f  READ_WRITE   from_func 10b54d32
10b54db5  READ         from_func 10b54db4
10b57676  WRITE        from_func 10b57633

$ grep '10c400d4' objdump_intel.asm               # GNU binutils, independent
10b54d57:  a1 d4 00 c4 10        mov  eax, ds:0x10c400d4
10b54d5f:  ff 05 d4 00 c4 10     inc  DWORD PTR ds:0x10c400d4
10b54db5:  8b 35 d4 00 c4 10     mov  esi, DWORD PTR ds:0x10c400d4
10b57676:  89 3d d4 00 c4 10     mov  DWORD PTR ds:0x10c400d4, edi
```

| VA | kind | routine | what it does |
|---|---|---|---|
| **`0x10b57676`** | **WRITE** | `FUN_10b57633` (541 B) | **`DAT_10c400d4 = 1`** — `edi` is 1 from `xor edi,edi; inc edi` at `0x10b57665`/`0x10b57669` `[R]` |
| `0x10b54d57` | READ | `FUN_10b54d32` (130 B) | `cand->[0x1c] = DAT_10c400d4` — stamps the new candidate `[R]` |
| `0x10b54d5f` | READ_WRITE | `FUN_10b54d32` | `DAT_10c400d4++` `[R]` |
| `0x10b54db5` | READ | `FUN_10b54db4` (60 B) | `esi = DAT_10c400d4 - 1` — the **occupied-bucket bound** of a partial table clear (§3) `[R]` |

**There is exactly one writer. It writes a constant. It is not an
initialiser-once guard** — there is no `if (already_done)` around it; the
guarded one-time init in the same routine is a *different* global
(`DAT_10c2e450` at `0x10b57682`, which does carry a `cmp/jne` and gates
`0x10bfb00d`). That contrast is on adjacent lines and is the cheapest available
proof that the counter's reset is deliberate per-call state, not lazy init.

---

## 2. Why the write is per-function — the containment argument

The strong form of the result does **not** depend on identifying the driver's
iteration granularity first. It is a containment fact:

> **Every reference to `DAT_10c400d4` lies inside one dynamic invocation of
> `FUN_10b7dc51`, and the write is the first of them in program order.**

`FUN_10b7dc51` calls, in this order (`0x10b7dcb7`, then `0x10b7dcf6`):

* **`FUN_10b57633`** @ `0x10b57633` — the `globregs.c` phase, whose *first* acts
  are the four resets in §2.1; and
* **`FUN_10b31c9a`** @ `0x10b31c9a` — `color.c`, the allocator proper.

Every one of the mint routine's **seven** call sites is inside one of those two
subtrees. Traced hop by hop from the flat export's `calls`/`xrefs`:

| mint call site | containing routine | reached from |
|---|---|---|
| `0x10b55e66` | `FUN_10b55dbe` | ← `0x10b577f2` **in `FUN_10b57633`** |
| `0x10b56839` | `FUN_10b5673e` | ← `0x10bfd827` in `FUN_10bfd665` ← `0x10b576b6` **in `FUN_10b57633`** |
| (via `0x10b55732`) | `FUN_10b55732`, the renamer | ← `0x10b577cb` **in `FUN_10b57633`** |
| `0x10bfd98b` | `FUN_10bfcf7c` / `FUN_10bfde2d` | ← `0x10b57802`, `0x10b57830` **in `FUN_10b57633`** |
| `0x10b2e151` | `FUN_10b2dfe2` | ← `0x10b30483` in `FUN_10b3032a` ← `0x10b31e15` **in `FUN_10b31c9a`** |
| `0x10b2e655`, `0x10b2e732` | `FUN_10b2e4ae` | ← `0x10b304e7` in `FUN_10b3032a` ← `0x10b31e15` **in `FUN_10b31c9a`** |
| `0x10c2073c` | `FUN_10c205fd` | ← `0x10c207d1` in `FUN_10c2075d` ← `0x10c20f97` in `FUN_10c20f79` ← `0x10b31d22` **in `FUN_10b31c9a`** |

and the fourth reference, the partial-clear read at `0x10b54db5`, sits in
`FUN_10b54db4` ← `0x10c20fda` in `FUN_10c20f79` ← `0x10b31d22` **in
`FUN_10b31c9a`**.

**So no candidate can be minted without a preceding reset**, which is
`WB_CANDID_PREREG.md` §5's third falsifier, checked and not triggered. `[R]`

### 2.1 Four pieces of state are reset together, and that is the scope boundary

`WB_CANDID_PREREG.md` §4 step 6 required checking the siblings, because a
boundary that resets the counter but not the table (or vice versa) is a
*different* answer — it is exactly `READ_PLAN` §5.2's reconciliation trap. All
of them are reset within `0x5C` bytes of each other near the top of
`FUN_10b57633`, and **all are unconditionally reached**:

* `0x10b57665`–`0x10b5767c` is **straight-line**: `xor edi,edi; xor ebx,ebx;
  inc edi` then four stores, no branch. The counter reset and the free-list
  reset are two of those four.
* The only branch before the hash memset is the **diamond** at
  `0x10b57688` (`jne 0x10b57695`), which guards the one-time init of
  `DAT_10c2e450` and **reconverges at `0x10b57695`**. Both arms fall into
  `0x10b5769c`. Stated precisely rather than as "no branch", because the
  branch is there and it does not matter — the reason it does not matter is
  the reconvergence, and a reader should be able to check that claim.
* The per-class-set rebuild at `0x10b57658` sits in a bounded fill loop
  (`0x10b57651`–`0x10b57663`, `jl`) that runs the fixed range
  `0x10c400d8 … 0x10c400f7`.

| VA | global | reset to | meaning |
|---|---|---|---|
| `0x10b57658` (loop `0x10b57651`–`0x10b57663`) | `0x10c400d8 … 0x10c400f7` | fresh sets | the **8 per-class candidate-id sets**, rebuilt one per class |
| `0x10b5766a` | `0x10c3ffcc` | `1` | companion phase counter |
| `0x10b57670` | `0x10c3ffc8` | `0` | companion phase counter |
| **`0x10b57676`** | **`0x10c400d4`** | **`1`** | **the candidate-id counter** |
| `0x10b5767c` | `0x10c2e3e0` | `0` | the candidate **free list** head |
| `0x10b5769c` → `FUN_10b2c1f1` | `0x10c43b80` | `memset(…, 0, 0x1000)` | the **1024-bucket candidate hash** |
| `0x10b576a7` | `0x10c3ffd0` | `memset(…, 0, 0x100)` | companion array |

The free-list reset is the load-bearing one for *density*. The mint at
`0x10b54d32` **only stamps a fresh id when it allocates**:

```
FUN_10b54d32:
  esi = DAT_10c2e3e0                     /* 0x10b54d33  free-list head       */
  if (esi) { DAT_10c2e3e0 = esi->[0x30]; goto init; }   /* 0x10b54d3e/43/48  */
  esi = alloc(arena 0x0e, 0x48)          /* 0x10b54d50                       */
  esi->[0x1c] = DAT_10c400d4             /* 0x10b54d5c   <-- id stamped HERE */
  DAT_10c400d4++                         /* 0x10b54d5f                       */
init:
  esi->[5] |= 1;  esi->[8] = esi;  esi->[4] = 2 /* kind 2 */
  ...
  hash_insert(esi)                       /* 0x10b54daa -> FUN_10b2c206       */
```

**A recycled record keeps its old id** — the `jmp` at `0x10b54d48` skips both
the read and the increment. That is only sound if the free list cannot outlive
the id space, and `0x10b5767c` is what guarantees it: the free list is emptied
in the same breath as the counter and the hash. `[R]`

### 2.2 `FUN_10b7dc51` runs once per function

Two independent lines, one static and one behavioural.

> **This half was already established, 18 days before this lane, and R1 did
> not need to re-derive it.** `rungs/_2026-08-04-w-mark-findings.md` §1c names
> `0x10b7f022` **"the p2 driver"** and transcribes the same loop, including the
> restart. Board **#3256** (`w-c2map2`) is the row establishing that
> `0x10b7f022` is a real function Ghidra never created — reached by tail jump
> `jmp 0x10b7f022` at `0x10b7f362`, `functions.tsv` carrying no entry in
> `0x10b7f022`–`0x10b7f1fe` — which this lane re-confirmed (`grep -c
> '^10b7f022' functions.tsv` → **0**). `READ_PLAN` §5.4's structural trap is
> exactly this address. **The reading below is a re-derivation that agrees, not
> a new claim**, and it is recorded because #3256's row is about Ghidra's
> coverage while what R1 needs is the loop's *granularity*.

**Static.** `FUN_10b7dc51`'s sole caller is `0x10b7e6ce` in `FUN_10b7e6af`;
`FUN_10b7e6af`'s sole call site is `0x10b7f1c5`, which is inside the routine at
**`0x10b7f022`**. That routine's tail is a work-list loop over the linked list
at `DAT_10c4630c`:

```
0x10b7f15f:  eax = DAT_10c4630c;  ecx = &DAT_10c4630c
0x10b7f16b:  edx = eax->[0x4c]
             if ((edx & 0x20) && !(edx & 0x02)) goto 0x10b7f199   /* unprocessed */
0x10b7f178:  ecx = &eax->[0x78];  eax = *ecx                      /* next        */
             ... loop
0x10b7f199:  eax->[0x4c] |= 2                                     /* mark done   */
0x10b7f1b1:  esi = FUN_10b7ef55()          /* pick + set up the function       */
0x10b7f1be:  FUN_10b7f000(esi)
0x10b7f1c5:  FUN_10b7e6af(esi)             /* <-- the back-end phase group     */
0x10b7f1d5:  FUN_10bda2ac(); FUN_10b7e1c4()
0x10b7f1f0:  jmp 0x10b7f15f                /* rescan from the head            */
```

Each pass claims one unprocessed entry, runs the phases on it, and rescans.
`FUN_10b7ef55` stores the picked record to `DAT_10c2e2f4` (via `FUN_10b7e719`
at `0x10b7e750`) and the driver publishes `entry->[0x4]` to `DAT_10c2e2f8`,
which is the current-function name the timing and dump hooks read. `[R]`

`w-mark` §1c reads the same loop as *"`compile s`"* at `0x10b7f199 … 0x10b7f1c5`
with a **`RESTART`** at `0x10b7f1e5` — a second back-edge to `0x10b7f15f`
beside the `jmp` at `0x10b7f1f0`. Two independent readings, same granularity:
**one iteration per function body compiled.** That lane's own emphasis is worth
carrying, because it is the reason the loop is a `goto` and not a `for`: the
emit set is a **worklist run to a fixpoint during codegen** — compiling one
function can mark another, which is then compiled. It does not disturb R1
(each admitted body still gets exactly one `FUN_10b7e6af`), but it means "once
per function" here is *per compiled body*, not per source declaration.

**Behavioural.** §4.

### 2.3 The one guard on the path, and it is not a leak

`FUN_10b7e6af` reaches `FUN_10b7dc51` only when `DAT_10c2e2fc != 0`
(`0x10b7e6be`/`0x10b7e6c5`), and returns immediately when
`fn->[0x94] & 0x0C000000` (`0x10b7e6b2`). `DAT_10c2e2fc` is **recomputed per
function** in `FUN_10b7e719` — cleared at `0x10b7e776` when `fn->[0x94] & 0x8000`,
at `0x10b7e867` when `fn->[0x94] & 0x20 && !(fn->[0x1c] & 2)`, and at
`0x10b7e89b` when **`DAT_10c40f18 >= 0x9c40` (40 000)**, a size bail-out. `[R]`

**This is not a hole in the scope claim.** When the guard is closed the counter
is not reset — but neither is any candidate minted, because §2's table shows
every mint site is *downstream of the reset*. The counter simply retains a
stale value across a function that never allocates. A stale value nobody reads
is not a carried scope.

> ⚠️ **What this guard *does* mean, and it is a separate finding: at `/Od`, and
> on any function over the 40 000 bail-out, the global register allocator does
> not run at all.** Anything in the port or the docs that models allocation as
> unconditional is wrong for those functions. Not R1's question; recorded
> because it was on the path. `[R]`

---

## 3. What the partial clear at `0x10b54db4` says on its own

```
FUN_10b54db4:
  esi = DAT_10c400d4 - 1                  /* highest id minted               */
  for (edx = 0; edx <= min(esi, 0x3ff); edx++) {
      for (p = DAT_10c43b80[edx & 0x3ff]; p; p = p->[0x30]) {
          p->[0x28] = 0;                  /* first block                     */
          p->[0x2c] = 0;                  /* last block                      */
      }
  }
```

`min(count-1, 1023)` is an **occupied-bucket bound**: it is only a saving if
the ids in the table are dense from a small base. Under a compilation-global
counter the bound saturates at 1023 after the first thousand-odd candidates in
the compiland and the `min` becomes dead code for the rest of the TU. The
instruction is therefore corroboration of density — **but it is corroboration
only**: a global counter would still be *correct* here, just pointless. The
load-bearing evidence is §1 + §2, not this. `[I]` on `[R]`.

---

## 4. Control C1 — `[O]`, and the positive control went red first

Preregistered in [`WB_CANDID_PREREG.md`](WB_CANDID_PREREG.md) §6, committed at
`7781147ab` **before** a single obj was compiled. `READ_PLAN` §5.3 is why it
exists: `[R]` means the instructions were read correctly, and the `.bss` bump
rule was read correctly and was wrong about c2.

Real `cl.exe` 16.00.11886.00 under wibo, `/O1 /GS- /c` — the workload's own
profile. Driver: **`scripts/candid_c1.py`**, committed and re-run from
`scripts/` rather than left in gitignored `work/`, because #1406 binds anything
whose output is quoted as evidence. It degrades to `SKIP: toolchain absent`
(exit 2) with no `compilers/`, verified against `C2RS_COMPILERS=/nonexistent`.

**C1** — a tie-sensitive probe `P` (10 independent single-use producers feeding
10 globals, 132 B emitted) compiled alone and again after 120 filler functions:

```
P bytes solo      : 132
P bytes after     : 132
solo[0:32]        : fbe1fff83d4000007d6322143d0000003ce000003fe00000916a00007d453214
after[0:32]       : fbe1fff83d4000007d6322143d0000003ce000003fe00000916a00007d453214
C1-pos            : DIFFERENT -> instrument LIVE
C1                : IDENTICAL
RESULT            : GREEN
```

**C1-pos, the positive control, is what makes the green mean anything.** The
same extractor on a `P` differing by one operand (`a * 3` → `a * 5`) reports
DIFFERENT. Had it not, the green would have been discarded as an instrument
failure, not published — `STATUS.md`'s standing trap that `mismatch 0` is not
evidence of correctness.

**C1b, the sharper half.** Fillers `F0 … F(N-1)` are **character-identical
bodies** distinguished only by name, each carrying ten simultaneously-live
values across a loop. Under function-scoping every `Fi` must equal `F0`; under
a compilation-global counter `Fi`'s ids are shifted by ≈`i·k` and wrap the
1024-bucket hash, splitting its candidates across `0x10b316b1`'s bucket walk:

| N | `F0` size | fillers differing from `F0` | `P` == solo |
|---:|---:|---:|---|
| 2 | 204 B | **0** | yes |
| 40 | 204 B | **0** | yes |
| 120 | 204 B | **0** | yes |
| **400** | 204 B | **0** | yes |

At N = 400 a compilation-global counter would have wrapped 1024 roughly four
times over. **558 filler bodies compared against their own `F0` across the four
TU sizes (1 + 39 + 119 + 399), 0 positional differences**, plus 4 `P`-vs-solo
comparisons.

### 4.1 The limit of the green, as fixed in the prereg before it ran

A green corroborates; it does not prove. Identical bytes are also consistent
with "no tie in these bodies actually decided anything", which is the ambiguity
board **#3363** left behind — its 4 shapes × 8 TU contexts came back 32/32
byte-identical and the row read that, correctly, as *"the tie tier does not
reach the emitted bytes on these shapes"*, agnostic between the two hypotheses.
C1b narrows it (four wraps, ten live values per body, 400 positions) but does
not close it. **The closing evidence is §1's exhaustive reference set and §2's
containment; C1 is the check that could have overturned them and did not.**

---

## 5. What this decides

### 5.1 `P_REGALLOC.md` consequence 3 loses its force — corrected in place beside

`ref/P_REGALLOC.md:160-166` reads, and board **#3242** repeats:

> *On an exact tie the order is a hash-bucket walk over a **compilation-global**
> counter, not a source property. … **This is the most direct available
> explanation for why source-level fitted sorts keep being refuted.***

The premise is false. The walk is over a **per-function counter that is dense
from 1**, so within one function a candidate's bucket **is** its mint index —
and a dense per-function mint index is precisely the kind of quantity that
*can* track source order. **"Not a source property at all" does not follow.**

What survives, and it is not nothing:

* The **mechanism** is untouched: the tie tier really is `0x10b316b1`'s bucket
  walk over `cand+0x1c`, reversed by `0x10b2b82d`'s `<=`. Only the claim that
  the key is *acausal with respect to the source* is withdrawn.
* The question is handed **whole** to the **mint order** — `FUN_10b55732`, the
  globregs renamer, item **F1**, read **R4** (3–5 d), still unread. The n=3
  divergence `#3242` fenced itself with (`b a c` where descending id predicts
  `c b a`) is now the *only* live evidence on that axis, and it points at R4.

### 5.2 The ten refuted allocation keys are back to UNEXPLAINED

`crates/c2-core/src/codegen/alloc.rs:103-539` catalogues ten fitted-then-refuted
keys — clause 2 refuted on 7 of 56 fresh-holdout cells (#836), `H-self` refuted
on 11 of 72 (#857), the 52 416-configuration preregistered search returning a
negative with the residual *exactly* the tie tier. `READ_PLAN` §3 row R1 named
"whether the ten refuted alloc keys have an explanation at all" as the thing R1
decides.

**They do not, on this mechanism.** R1 removes the standing explanation rather
than supplying one. That is a real deliverable and it changes what to fund
next: a source-level key is no longer *a priori* doomed, and the missing input
is the mint order, not a better sort. **R4 is now the read that would settle
it** — and it is the read `alloc.rs:40-43` already points at.

### 5.3 `select_function`'s no-TU-context signature is SOUND on this axis

Nothing about a function's register allocation can depend on how many
candidates its predecessors minted. §4's 400-position ladder is the behavioural
form of the same statement. This does **not** revive per-function composition:
board **#3363** refuted it on three *other* grounds (nothing left to buy; the
label plan is not derivable; anti-safe under `PROGRESS_METRIC.md`), and #3363's
own framing already assumed per-function independence held.

### 5.4 Two board rows must be reconciled, and #3056 was right

**#3056** (`wb-live`) says *"function-scoped"*. **#3242** (`w-dagorder`) says
*"a compilation-global counter"*. #3056 is right. #3242's headline —
*"THE TIE TIER IS A HASH-BUCKET WALK OVER A COMPILATION-GLOBAL COUNTER, SO THE
CANDIDATE ORDER IS NOT A SOURCE PROPERTY AT ALL"* — is half right: the walk is
real, the scope is not, and the **"so"** does not carry. Amended at #3374.

### 5.5 No `DISCLOSURE.md` row is owed, and that is not an oversight

`READ_PLAN` §3 row R1's "spec produced" column asks for one. `DISCLOSURE.md` is
the ledger of **findings adopted into `crates/`** ("Adopted findings", and its
§"If you are about to add the first row" step 1: *add the row before or with the
code change*). R1's result is a **negative** — it withdraws an explanation and
supplies no constant, table or rule for the port to adopt. There is nothing to
disclose until a lane adopts something, and manufacturing a row for a fact that
never enters `crates/` would make the ledger stop meaning what it says. Flagged
as a small `READ_PLAN` spec mismatch rather than silently satisfied.

---

## 6. Corrections filed by this read

| doc | site | was | now |
|---|---|---|---|
| `ref/P_REGALLOC.md` | `:62` | *"`id = DAT_10c400d4++` (compilation-global monotonic)"* | struck; function-scoped, reset `= 1` at `0x10b57676` |
| `ref/P_REGALLOC.md` | `:86` | *"the compilation-global monotonic candidate-id counter"* | struck; per-function, dense from 1 |
| `ref/P_REGALLOC.md` | `:188` | *"`DAT_10c400d4++`, compilation-global"* | struck; per-function |
| `ref/P_REGALLOC.md` | `:160-166` | consequence 3's *"so … not a source property at all"* | fenced with a revision box; the mechanism stands, the inference is withdrawn |
| `READ_PLAN_2026-08-21.md` | `§5.2` | live caveat | marked **RESOLVED by R1**, function-scoped |
| `WB_LIVE_FINDINGS.md` | `:258-260` | correct, but evidenced only by the table clear | left as written (dated record); the missing address `0x10b57676` is added as a revision box |

`ref/ADDR.tsv:1209` carries `10c400d4 … unknown` in its label column. **It is
generated** (`scripts/build_ref.py`, "do not hand-edit") and is not touched
here; it will pick the label up when the reference is next regenerated.

## 7. What was NOT read, stated so absence does not read as coverage

* **`FUN_10b55732`, the mint order** (1 676 B) — read **R4**. §5.1 hands it the
  whole remaining question and this lane did not open it.
* **What `fn->[0x94]` bits `0x8000`, `0x20`, `0x0C000000` and `fn->[0x1c]` bit 1
  mean** — §2.3 names the guard's shape, not its source-language trigger.
* **`DAT_10c40f18`** — the quantity compared against 40 000. Read as a size
  proxy from context only; not traced. `[I]`
* **Whether `0x10b57633` can run twice for one function.** No path was found,
  and none is needed for the result (a second run would reset again), but the
  negative was not exhaustively established.
* **The recycled-record duplicate-key question.** A record taken off the free
  list keeps its id and is re-inserted at `0x10b54daa`, so a bucket can hold two
  nodes with the same `+0x1c` and `0x10b2c21d` returns the first. Within a
  function this is reachable in principle; whether it happens was not measured.
