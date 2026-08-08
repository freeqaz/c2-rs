# WB-INLINE — the inliner's decision function, read out of the binary and graded by objs

> **PROVENANCE — DISASSEMBLY-DERIVED.** Every address below is an absolute VA in
> the exact image pinned in [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0 —
> `sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`,
> verified at the head of this lane. This is **navigation** until a row lands in
> [`DISCLOSURE.md`](DISCLOSURE.md). **The obj is the sole judge** (method doc §7):
> a reading an obj refutes is retracted, not narrowed. This lane adopts
> **nothing** into `crates/`.

Lane **WB-E** of
[`CAMPAIGN_2026-08-08_GENERATORS.md`](CAMPAIGN_2026-08-08_GENERATORS.md).
PREREG in [`WB_INLINE_PREREG.md`](WB_INLINE_PREREG.md), committed at `531ac130`
before the first grep of the flat export. Scored in §9.

---

## 0. The headline, stated first

> ### **`INLINE-P` is EXACTLY RIGHT inside the class it was fitted on and wrong outside it in TWO measured directions — a flag axis it does not have, and a LOOP axis nothing has ever had. On the anchor it predicts six inlines and gets one.**

Three numbers carry the lane:

1. **GRID-I, 264 frozen cells against real `c2.dll`.** `INLINE-P` scores
   **218/264**; this lane's disassembly reading scores **226/264**; neither
   survives. **The size ceiling moves with the favor-speed bit** —
   `(300,308] → (212,252]` for a STATIC callee — which no published rival has
   an input for, and which is the *same* option-word bit `0x10c2e310` that
   `wb-memcpy` found moving the `memcpy` threshold.
2. **GRID-J, 56 frozen cells.** A callee whose body is a **counted loop**
   declines at `(56,80]` of emitted `.text` where a straight-line callee inlines
   to `(96,120]` — **identically at the workload's `/GR /O1 /Oi /EHsc` and at
   `/O1 /GS- /c`**, so it is the loop and not the flags.
3. **The anchor, measured on the real `keygen_xbox.cpp` obj.** The six shuffles
   are **104 / 60 / 84 / 84 / 88 / 88** bytes and **only the 60-byte one is
   inlined**. Every one of the six is under `INLINE-P`'s published 112-byte
   EXTERNAL ceiling.

**The clause that fires for `?supershuffle` is the LOOP-CLASS SIZE CEILING at
`(60,80]`, and the answer to "why `?shuffle2`" is that it is the only one of six
small enough** (§6).

---

## 1. Where the decision lives

The inline pass is `p2\inline.c` (`c2_tus.tsv` anchors `10b5cfd4`…`10b5f048`;
the file string is at `0x10b024f8`). The chain, top down:

| VA | what |
|---|---|
| **`0x10b62675`** | **the pass entry, per function.** Skipped wholesale when `DAT_10c40ec4 == 0`. |
| **`0x10b626d8`** | `DAT_10c3f5cc = (ushort)[fn+0x50]` — the caller's own **instruction count**, the running growth total |
| **`0x10b626f4`–`0x10b62710`** | **the budget**: `B = 2 × instrs`, floored at **1000**, capped at **35000** |
| **`0x10b6276a`** | `FUN_10b61ee1(fn, /*level*/1, /*budget*/B, 0, 100000000, 0)` |
| **`0x10b61ee1`** | **the driver**: collects the sites, loops over them, returns *budget consumed* (`iVar6 - param_3`) |
| **`0x10b600e6`** | **the site collector**: one linear scan of the instruction list; instruction kind **`0x0f`** is a call site (the same kind `wb-frame` found at `10bff565`). Tracks EH-region nesting through opcodes `0x2ee/0x2f0/0x2f1/0x2f4/0x2f6/0x2ff/0x300` and stamps a **conditional/EH flag** into bit 1 of the candidate record |
| **`0x10b5fb5f`** | **candidacy**, per callee — where the size ceiling is |
| **`0x10b5c06b`** | **legality**: refuses on flags `0x400 / 0x1000 / 0x40 / 0x100` at `[sym+0x20]` and `0x80000 / 0x200` at `[sym+0x4c]`; requires bit 6 of `[sym+0x4c]` |
| **`0x10b61d2c`** | per-site driver |
| **`0x10b60930`** | **the accept/decline predicate** — depth, budget, POGO |
| **`0x10b6242a`** | **the charge**, and the second copy of the 40-instruction test |
| **`0x10b620fc`** | **the expansion**, and the recursion back into `0x10b61ee1` for the inlined body |
| **`0x10b5fcd8`** | **the profitability model — POGO ONLY.** Reached from `0x10b60930` only when a profile record exists (`[sym+0x80]` with `DAT_10c2e2fc` set) |

## 2. The decision function, read off the disassembly

### 2.1 Candidacy — the size ceiling, and the flag that switches it off

`0x10b5fb5f`, the arm that returns 1:

```
   0x10b5fdfd   cmp DWORD [0x10c2e310], 0      <- the FAVOR-SPEED bit
                if non-zero: the size test is SKIPPED entirely
   0x10b5fe0c   movzx eax, WORD [sym+0x50]     <- the callee's INSTRUCTION COUNT
   0x10b5fe14   cmp eax, DWORD [0x10c46318]    <- the ceiling; `jl` = candidate
   0x10b5fe1e   test DWORD [sym+0x4c], 0x2000  <- __forceinline: bypass
```

and the ceiling itself, at `0x10b5e4cc`:

```
   0x10b5e4d1   cmp DWORD [0x10c2ea98], 7
   0x10b5e4d7   DAT_10c46318 = 0x10 << DAT_10c2ea98     (16 instructions << k)
   0x10b5e4e8   DAT_10c46318 = 1000                     (k >= 7)
```

`0x10c2ea98`'s image value is **3**. `0x10c2e310` is `wb-memcpy` §2.1's
favor-speed bit, written from bit 23 of the option word at `0x10b8238d`.

**Two things this says that the incumbent could not see.** The quantity is a
**count c2 holds before codegen** (`WORD [sym+0x50]`, and the diagnostic string
at `0x10b025ec` is literally `"INL:\tInlining %s (%d instrs) into "`), not a
byte count; and **the ceiling is switched off by one option bit**, which is why
no grid compiled at a single flag set could ever see it move.

### 2.2 The budget — `B = clamp(2 × caller_instrs, 1000, 35000)`

`0x10b62675`:

```
   0x10b626d8   DAT_10c3f5cc = (ushort)[fn+0x50]          the caller's instrs
   0x10b626f4   uVar7 = 1000
   0x10b626fb   if (2*DAT_10c3f5cc > 1000) uVar7 = 2*DAT_10c3f5cc
   0x10b62708   if (uVar7 > 34999)         uVar7 = 35000
```

charged at `0x10b624a2` and tested at `0x10b60a0e`, **both behind the same
constant**:

```
   0x10b6249b   cmp WORD [callee+0x50], 0x28      <- 40 instructions
   0x10b624a2   *budget -= WORD [callee+0x50]     <- charged only if > 40
   0x10b624ae   DAT_10c3f5cc += WORD [callee+0x50]

   0x10b60a04   if (budget < instrs && instrs > 0x28) return DECLINE
```

> **A callee of 40 instructions or fewer is never charged against the budget and
> is never declined for affordability.** The budget is a growth cap for *large*
> callees only.

### 2.3 Depth, and the categorical arms

```
   0x10b609ae   0x10 < level - DAT_10c3f50c        -> decline   (16 levels)
   0x10b609bd   maxlevel != 0xff && maxlevel < level -> decline
   0x10b609d3   test [sym+0x4c], 0x2000            -> __forceinline bypasses
                                                      every size and budget test
   0x10b609ee   35000 < DAT_10c3f5cc               -> decline (the function
                                                      itself is already huge)
```

### 2.4 The profitability model is POGO-only, and that matters

`0x10b5fcd8` is a full cost/benefit model — a savings vector from
`0x10b5e6a5`, a per-site-count discount `cost -= (K + cost)/n_sites`
(`0x10b600c8`), and ~20 tunable weights copied from one of **two 46-dword
parameter tables** (`0x10b5b86d` / `0x10b5e4cc`: `DAT_10c45e18` when
`DAT_10c6f1c8 == 0`, `DAT_10c45ed0` otherwise) into `DAT_10c3f510…`.

**It is unreachable on a non-PGO build**: `0x10b60930` calls it only when the
callee has a profile record. The 46-dword tables live above the image's raw
`.data` (`0x10c3cc00`), so they are **zero at load and written at run time** —
none of their values is quotable from the image and this lane does not quote
them.

**This is method doc §7 case 1 in advance**: the most model-like code in the
inliner is not the code the workload takes, and a lane that read it and stopped
would have published a cost model c2 never runs.

---

## 3. GRID-I — 264 cells, frozen before the first `cl.exe`

Sources and the frozen table: [`grids/wb-inline/`](grids/wb-inline/) — `grid.py`,
`frozen.json`, `separation.json`, `calib.json`, `measured.json`. Minimum
separation over all **10** rival pairs: **39 cells** (asserted floor 4).
sha256 of the concatenated cell sources
`37d787df7339945b50111070c3ec0d9216aaca88594d953e9b1f0b292cc6bb2b`.

**The verdict function is three-valued** — `inlined` / `called` / `absent` — with
`absent` reported as ASSUMPTION UNMET and excluded, per `wb-frame` §5.2's rule.
0 of 264 came back `absent`.

### 3.1 GRID-I v1 was REFUTED by its own cells, and that is recorded rather than repaired quietly

v1's ladder was `a = a*3 + i` repeated *k* times. **c2 folds the whole chain to
two words at every k**, so the size axis did not occur: 159 cells that all
measured the same 28-byte callee. The rebuild uses `a += tbl[i]` (8 bytes per
rung, opaque to constant propagation) and, more importantly, **stops guessing
`s` at all**: `grid.py cal` measures the callee's own emitted `.text` at `/O1`
for all 31 rungs, and the frozen per-cell predictions are computed from those
measurements. Calibration measures an **input** (`INLINE_PREDICATE.md` §2
defines `index` on exactly this quantity), never a verdict, and was committed
at `256faaca` before one verdict cell was compiled.

### 3.2 The rivals

| id | predicate |
|---|---|
| **R1-INCUMBENT** | `INLINE-P` exactly as `INLINE_PREDICATE.md` §2 publishes it |
| **R2-CEILING** | this lane's reading: a favor-speed-selected instruction ceiling, `<= 40` free of the budget, `__forceinline` bypassing, `/Ob0` categorical |
| **R3-SIZE64** | the strawman: inline iff `s <= 64 B` |
| **R4-OBLEVEL** | inline everything except at `/Ob0` |
| **R5-NOSITES** | `INLINE-P` with SCHEDULE D deleted — the ceiling only |

### 3.3 Scores

| rival | score |
|---|---:|
| **R2-CEILING** | **226 / 264** |
| **R1-INCUMBENT** | **218 / 264** |
| R5-NOSITES | 195 / 264 |
| R3-SIZE64 | 168 / 264 |
| R4-OBLEVEL | 144 / 264 |

> **No rival survives, and R2 winning by 8 is not a result worth having.** R2's
> 38 misses are all *parameter* errors — its registered ceilings (65 words at
> favor-size, 40 at favor-speed) are wrong; R1's 46 are all *structural* — it
> has no flag axis at all. Both are refuted as written. What survives is the
> list of measured facts in §4, each of which is a statement about c2 rather
> than about a rival.

### 3.4 The measured boundaries, `n = 1`

`s` = the callee's own emitted `.text`, measured. The bracket is
*(last inlined, first called]*.

| family | `/O1` | `/O2` | `/O1 /Ot` | `/O2 /Os` | `/O1 /Ob0` |
|---|---|---|---|---|---|
| **STATIC**, straight-line | **(300, 308]** | **(212, 252]** | **(212, 252]** | **(300, 308]** | nothing inlines |
| **EXTERNAL**, straight-line | **(100, 116]** | **(156, 164]** | — | — | — |

**The threshold follows FAVOR-SPEED, not the `/O<n>` level**, on the same two
mixed cells that decided `wb-memcpy`'s GRID-W: `/O1 /Ot` behaves as `/O2`, and
`/O2 /Os` behaves as `/O1`. `wb-memcpy`'s option-word bit 23 is the second
mechanism now shown to hang off it.

**And `INLINE-P` is exactly right where it was fitted.** Its EXTERNAL clause is
`index = s − 48·[leaf] ≤ 64`, i.e. `s ≤ 112` — inside the measured `(100,116]`.
Its STATIC hard cap is `i ≥ 65`, i.e. `s ≥ 308` — the measured boundary is
`(300,308]`, **to the word**. Every one of its misses is at a flag set its
corpus never contained.

---

## 4. What the grid established, stated as facts about c2

| # | fact | cells |
|---|---|---:|
| **F1** | the STATIC ceiling is `(300,308]` at favor-size and `(212,252]` at favor-speed | 120 |
| **F2** | the EXTERNAL ceiling is `(100,116]` at `/O1` and `(156,164]` at `/O2` | 60 |
| **F3** | `/Ob0` declines **everything**, including `__forceinline` | 34 |
| **F4** | `__forceinline` inlines a **980-byte** callee, at `/O1` and at `/O2` | 2 |
| **F5** | varargs and direct recursion decline categorically at every flag set | 6 |
| **F6** | **SCHEDULE D reproduces**: at `s = 212` static, `n=1` inlines and `n=3`, `n=9` decline; at `s = 92` all of `n∈{1,3,9}` inline; at `s = 420` none. `INLINE-P` is **9 of 9**, identically at `/O1` and `/O2` | 18 |
| **F7** | **the caller's own size is NOT an input.** A 48-byte caller and a 5,640-byte caller give identical verdicts at every size and both flag sets | 12 |
| **F8** | a **control-dependent** site at `s = 212` declines at `/O1` where the unguarded one inlines; at `/O2` it does not | 6 |
| **F9** | (GRID-J) a **loop-bodied** callee declines at `(56,80]` where a straight-line one inlines to `(96,120]`, **identically at the workload flags and at `/O1 /GS- /c`** | 56 |

### 4.1 F7 refutes this lane's own budget reading as a *practical* input

§2.2's budget is read correctly — the instructions are there — but the D family
moves the caller from 48 B to 5,640 B, i.e. `B` from 1000 to ~2,820, and
**nothing at all changes on 12 cells**. That is consistent with §2.2 (every
callee tested at `k ≤ 40` instructions is free of the budget, and the ones above
it are already refused by the ceiling), and it means **the budget is not
reachable from the flag/size space this lane swept**. It is recorded as
**READ, NOT CONFIRMED**, and no DISCLOSURE row proposes it.

### 4.2 What is NOT covered, so absence does not read as coverage

* The **POGO cost model** (`0x10b5fcd8`) and both 46-dword parameter tables:
  read, not reachable on this workload, **never obj-checked**, no row.
* The **depth cap of 16** (`0x10b609ae`): no cell nests 16 deep.
* **`0x10c2ea98 = 3`** would give a ceiling of `16 << 3 = 128` *instructions*;
  the measured straight-line ceilings are 25–29 and 37–41 emitted words for
  EXTERNAL and 53–65 / 75–77 for STATIC. **The reading does not compose into the
  measured numbers**, so `16 << k` is named and **not** claimed as the boundary
  the workload takes. Something between the two — most plausibly the linkage
  arm and the `[sym+0x50]`/emitted-size gap of §5 — is unread.
* **Nothing here is a total statement about c2.** It is 320 cells.

---

## 5. F9 is the new finding: the index is a COUNT, and emitted bytes over-credit a loop

GRID-J ([`grids/wb-inline/gridJ.py`](grids/wb-inline/gridJ.py), frozen at
`f7207801`) sweeps two families of `void cg(char*)` — a counted `for` loop and a
straight-line body — at the workload's `/GR /O1 /Oi /EHsc` and at `/O1 /GS- /c`.

```text
              workload flags      /O1 /GS- /c
   loop        (56,  80]           (56,  80]
   line        (96, 120]           (96, 120]
```

**Identical across the two flag sets**, so `R7-FLAGS` — *"it is `/Oi /EHsc /GR`
and not the loop"* — is **REFUTED**, and `R6-LOOP` survives: the boundary in
emitted bytes is strictly lower for a loop body, and `(56,80]` brackets the
anchor's own `(60,84]`.

**The mechanism §2.1 names is exactly this.** The ceiling is applied to
`WORD [sym+0x50]`, a count c2 holds **before codegen**; emitted bytes are a
proxy. A straight-line body's tuple count tracks its word count; a loop's does
not — the induction variables, the compare and the branch collapse into one
`bdnz`, and strength reduction removes more. So a loop body priced by its
*emitted* size is over-credited by roughly `112/72 ≈ 1.55`, and that gap is
`INLINE-P`'s residual class showing up as a boundary rather than as a
percentage.

**This is a reading, and only the BOUNDARY is obj-established.** That the
quantity is a tuple count is what `0x10b5fe0c` and the `"%d instrs"` string say;
that it explains the 1.55 is not measured, and this lane did not measure a
tuple count of anything.

---

## 6. `?supershuffle@@YAXPAD@Z` — the specific clause, and the priced remedies

### 6.1 The measurement

`work/wb-inline/anchor.sh` compiles the real
`dc3-decomp/src/keygen_xbox.cpp` at the workload's own flags. Its obj:

| symbol | own `.text` | at the site |
|---|---:|---|
| `?shuffle1@@YAXPAD@Z` | 104 B | **`bl`** |
| **`?shuffle2@@YAXPAD@Z`** | **60 B** | **INLINED** |
| `?shuffle3@@YAXPAD@Z` | 84 B | **`bl`** |
| `?shuffle4@@YAXPAD@Z` | 84 B | **`bl`** |
| `?shuffle5@@YAXPAD@Z` | 88 B | **`bl`** |
| `?shuffle6@@YAXPAD@Z` | 88 B | **`bl`** |
| `?supershuffle@@YAXPAD@Z` | 104 B = **26 words** | — |

which reproduces `wb-frame` §1's 26 words and its five surviving `bl`s exactly,
by relocation name.

**All six are EXTERNAL, non-`inline`, one `char*` parameter, leaf, and a single
counted `bdnz` byte loop.** They differ in nothing but size.

### 6.2 The clause

> **The LOOP-CLASS SIZE CEILING at `(60,80]` of emitted `.text` — c2's
> `cmp [sym+0x50], [0x10c46318]` at `0x10b5fe14` — fires for `?shuffle2` and for
> nothing else in the TU.** `?shuffle2` is 60 B; the next smallest is 84 B.

`INLINE-P` reads `index = s − 48 ≤ 64` and puts **all six** under its 112-byte
ceiling. It predicts six inlines and gets one — a **1 of 6** on the single
function the whole `keygen_xbox` frontier row turns on.

**Registered prediction P3.1/P3.2 said the clause was the EXTERNAL `index ≤ 64`
arm and that `?shuffle2` is under it while the others are over.** The first half
is **WRONG** — the arm is the loop-class ceiling, three times tighter — and the
second half is **RIGHT** for a reason the prediction did not have. Scored as a
miss in §9.

### 6.3 The two remedies, priced

| | **A. a real inliner pass** | **B. an inlined-body transcription licence** |
|---|---|---|
| what ships | the port lowers `?shuffle2` and splices it into `?supershuffle`'s tuple stream before its own codegen | a recognizer for this body shape plus 26 hand-derived words |
| the decision | needs the loop-class ceiling of §6.2, which is a **bracket** `(60,80]`, not a number — the port cannot ask "is this callee 60 bytes" without lowering it first, and `w-splice` already paid that ordering cost (`INLINE_PREDICATE.md` §6 item 2) | none — the decision is baked into the recognizer |
| the bytes | **the blocker.** c2's inlined `?shuffle2` is 14 words that are *not* `?shuffle2`'s own 15-word COMDAT: the copy is frameless, re-allocated into the caller's registers, and its base pointers are folded against the caller's `r3`. That is WB-D's register-choice question, unsolved | 26 words, transcribed |
| the interprocedural fact | still needed on top: c2 keeps `c` in the **volatile `r3`** across four `bl`s because the in-TU callees provably do not write it (`wb-frame` §7). Without it the port emits `std r31` / `ld r31` / five `mr r3,r31` — 7 of the 21 words it currently gets wrong | none |
| what it converts | **0 TUs today.** `keygen_xbox.cpp` is 1 exact of 20 emitted functions (#1474); fixing the 1 measured wrong emit leaves 18 reader-refused | **0 TUs**, same arithmetic |
| what it costs | a splice pass, a lowering for arbitrary loop bodies, a clobber-set analysis | one recognizer, one word table, and a class of exactly one |

> **Neither remedy converts a TU, and the honest recommendation is that
> `?supershuffle` is not worth either.** Board #1477 should be closed rather
> than re-pointed: `wb-frame` retracted its frame diagnosis, and this lane
> retracts its value. The inliner is real, its decision is now partly readable,
> and **the anchor is the wrong place to spend it.**

### 6.4 What IS worth spending it on

`DIFF_STRUCTURE.md` §2 counts **2,801 of 3,195** differing bodies as mechanism I,
and `w-splice` converted **723** of them by shipping the *bytes* (SPLICE-0) while
explicitly refusing the *decision*. The region where the decision is categorical
is the one thing this lane can widen safely, and §7 states it.

---

## 7. What a code lane could take, and what it must not

**MUST NOT: the cost model.** `INLINE-P`'s 2.84 % residual is a wrong emit if it
lands on the accept side (`INLINE_PREDICATE.md` §6 item 4), and this lane has now
measured **two further classes it is wrong on** — every non-`/O1` flag set, and
every loop-bodied callee. The residual is larger than published, not smaller.

**MAY, and only these — the categorical arms, each obj-established here:**

| clause | evidence | port-side use |
|---|---|---|
| `/Ob0` ⇒ **no** expansion, `__forceinline` included | F3, 34 cells | a mode the port can refuse cleanly |
| varargs callee ⇒ never inlined | F5 | narrows `IlBundle::functions()`' wholesale refusal |
| directly recursive callee ⇒ never inlined | F5 | same |
| a callee whose emitted body is **> 308 bytes** ⇒ never inlined at `/O1` | F1, 30 cells | the safe *decline* side: the port may keep the call |
| a **loop-bodied** callee **> 80 bytes** ⇒ never inlined at `/O1` | F9 + the anchor, 62 cells | ditto, three times tighter |

Every one of these is a **decline** rule. `IlBundle::functions()` refuses any TU
where a callee is also defined; each row above lets the port *keep its own call*
and be right, which is the direction that cannot produce a wrong emit from a
mis-prediction. **The accept side is not offered.**

---

## 8. Reproducing

```sh
sha256sum ~/ghidra-projects/bin/c2dll        # must equal c80981…6258

# the readings — flat export only, never the Ghidra project (method doc §4)
grep -n "inline\|InlBadCandidate" ~/ghidra-projects/export/c2/strings.tsv
awk '/^10b62675:/,/^10b62845:/' ~/ghidra-projects/export/c2/objdump_intel.asm
awk '/^10b5fb5f:/,/^10b5fcd8:/' ~/ghidra-projects/export/c2/objdump_intel.asm

# GRID-I  (calibration first — it measures s, never a verdict)
python3 work/wb-inline/grid.py cal   work/wb-inline/gridI <repo>
python3 work/wb-inline/grid.py gen   work/wb-inline/gridI
python3 work/wb-inline/grid.py run   work/wb-inline/gridI <repo>
python3 work/wb-inline/grid.py score work/wb-inline/gridI

# GRID-J and the anchor
python3 work/wb-inline/gridJ.py work/wb-inline/gridJ
work/wb-inline/anchor.sh <dc3-root> <out.obj>
python3 scripts/gt_dump.py <out.obj> | grep -i shuffle
```

---

## 9. PREREG, scored

| # | registered | outcome |
|---|---|---|
| **P1.1** | the decision is inside `c2.dll`, IL carries a call tuple | **RIGHT, and not separately measured this lane** — the pass is `p2\inline.c` and reads the tuple list at `0x10b600e6`. No IL capture was taken, so this is a reading, not an obj check |
| **P1.2** | one cost function, findable by the incumbent's constants | **WRONG, and in the optimistic direction.** None of `64 / 48 / 19 / 260 / 164` appears in `inline.c`. The pass was found by a **string** (`InlBadCandidate`, `"%d instrs"`), and the one cost function that exists is **POGO-only** |
| **P1.3** | `/Ob` and favor-speed reach it through `.data` globals in `0x10c2e310`'s family | **RIGHT, and it is `0x10c2e310` ITSELF** — the same bit `wb-memcpy` disclosed, read at `0x10b5fdfd`. Obj-confirmed on 180 cells by F1/F2 |
| **P2.1** | the graduated middle is a **budget loop**, not a division | **WRONG.** The only site-count arithmetic in the image is a **division** — `cost -= (K + cost)/n_sites` at `0x10b600c8` — and it is in the POGO-only model. The real budget (§2.2) exists but F7 shows it is not the graduated middle. Registered as this PREREG's sharpest claim; it is a miss, in the optimistic direction |
| **P2.2** | 16 / 64 / `i<=16` are one constant | **NOT ESTABLISHED.** `16 << k` is one constant at `0x10b5e4d7`, but §4.2 records that it does not compose into the measured boundaries. Filed `unknown` |
| **P2.3** | 48 is a call-overhead charge, a single immediate | **WRONG.** No 48 appears. §5 offers a different account — that 48 is an artifact of measuring bytes where c2 counts tuples — and does not claim it |
| **P2.4** | 260 and 164 are one quantity at two thresholds; the site-side input is control-dependence | **HALF RIGHT, and the measured half is the one I registered PESSIMISTIC.** F8 confirms control-dependence is a real site-side input at `/O1` (6 cells) and `0x10b600e6`'s EH/region counters are where it is computed; the "one quantity" half is unestablished |
| **P2.5** | varargs and recursion are early categorical refusals | **RIGHT on the outcome** (F5, 6 cells). The *location* claim is not separately confirmed — the same 6 cells are produced by `0x10b5c06b`'s flags and by the front end alike |
| **P2.6** | all-or-nothing because the count is per callee | **NOT MEASURED.** F6 confirms all-or-nothing behaviour at 9 of 9; no cell distinguishes where the count lives |
| **P2.7** | the residual is NOT closed; ≥1 unmodelled input is named but not obj-confirmed | **RIGHT, and better than registered.** Two unmodelled inputs were named **and both were obj-confirmed**: the favor-speed flag (F1/F2) and the loop class (F9). Registered pessimistic; the miss is in the pessimistic direction |
| **P3.1** | the clause is the EXTERNAL `index ≤ 64` arm; the asymmetry is size | **WRONG on the clause, RIGHT on "it is size".** The arm is the loop-class ceiling at `(60,80]`; `INLINE-P`'s 112-byte arm puts all six under it |
| **P3.2** | `?shuffle2` is a leaf with `s ≤ 112`; ≥1 of the other five is over | **WRONG.** `?shuffle2` is 60 B and **all five others are also ≤ 104 B**, i.e. all six satisfy the registered condition. The prediction was satisfiable by the wrong reason and the obj says so |
| **P3.3** | `n_sites(?shuffle2) = 1` | **RIGHT** — one call site in the TU |
| **P3.4** | the inlined 14 words are not `?shuffle2`'s COMDAT spliced verbatim | **RIGHT** — `?shuffle2`'s own body is 15 words including its `blr`; the copy inside `?supershuffle` is 14 and is re-allocated (§6.3) |
| **P4.1** | the grid separates size-of-callee from caller-side budget on ≥4 cells, size wins | **RIGHT** — F7, 12 cells, and the caller-side axis measured **exactly zero** effect |
| **P4.2** | `/Ob0` is categorical, `__forceinline` included | **RIGHT, 34 of 34** |
| **P4.3** | the threshold moves with favor-speed on ≥2 cells | **RIGHT, and it is 60 cells** — `/O1 /Ot` behaves as `/O2` and `/O2 /Os` as `/O1`, in both linkage classes |
| **P4.4** | `__forceinline` overrides the size ceiling at every optimizing flag set | **RIGHT** — 980 bytes inlined at `/O1` and `/O2` (F4) |
| **P5.1** | a correct inline DECISION converts zero frontier TUs on its own | **RIGHT** (§6.3) |
| **P5.2** | the anchor's honest price is > 1 TU of work for 0 TUs of movement | **RIGHT, and sharpened**: neither remedy is worth taking, and #1477 should close rather than re-point |
| **P5.3** | the shippable thing is a narrowing of `functions()`' refusal via the categorical arms | **RIGHT** (§7) — and the offered rules are **decline-side only**, which is narrower than registered |

### 9.1 Direction, and board #770

Registered **PESSIMISTIC**. The direction was right: the incumbent was not
displaced, no TU moved, and the lane's own reading scored 226/264 and is
retracted as written. **The three misses that matter — P1.2, P2.1, P2.3 — are
all in the OPTIMISTIC direction and are all the same error**: a lane that had
just read a clean call chain assumed the numbers it was hunting would be *in*
it. They are not; the workload's decision is a **single unsigned compare against
a runtime-initialised global**, and every constant the incumbent fitted is an
artifact of measuring bytes on the outside.

**#770 goes to ~11 optimistic / 2 pessimistic / 2 hits** by this lane's own
accounting — the optimistic tally moves because P1.2/P2.1/P2.3 are one
over-read, not three.

---

## 10. Pre-drafted DISCLOSURE rows

**Nothing below is adopted.** These are drafted so a later code lane can carry
them *in the same commit* as the code change.

| # | Kind | What would be adopted | Address in `c2.dll` | Adopted into | Commit | Notes |
|---|---|---|---|---|---|---|
| **W-INLINE-1** | **route** | **The inline size ceiling is switched off by the favor-speed option bit, and is applied to a pre-codegen instruction COUNT rather than to emitted bytes.** A port needs only the *behaviour* — two ceilings selected by favor-size/favor-speed, and a loop-bodied callee priced far above its emitted size — all of which GRID-I/GRID-J measure directly. No constant, table or bit layout need be copied. | `0x10b5fdfd` (`cmp [0x10c2e310],0`), **`0x10b5fe0c`** (`movzx eax, WORD [sym+0x50]`), **`0x10b5fe14`** (`cmp eax,[0x10c46318]`), `0x10b5e4d7` (`0x10 << [0x10c2ea98]`), `0x10b025ec` (the `"%d instrs"` string that names the unit) | *(not adopted — module docs only)* | — | Logged `route:` under the grey-zone rule. **The black-box alternative was tried FIRST and is on record**: `INLINE_PREDICATE.md` §2's `INLINE-P` is a 0.9716 hold-out model built with no disassembly, and it is **exactly right at `/O1` on straight-line callees** (§3.4). The disassembly's contribution is to say *where it stops*, and the 320 cells then measured that directly. **`0x10c2ea98 = 3` is named and NOT decoded** — §4.2 records that `16 << 3` does not compose into the measured boundaries |
| **W-INLINE-2** | **route** | **`__forceinline` is a flag bit that bypasses every size and budget test, and `/Ob0` overrides even it.** | `0x10b609d3` / `0x10b5fe1e` (`test [sym+0x4c],0x2000`), `0x10b626a0`-ish (`DAT_10c40ec4` gating the whole pass) | *(not adopted)* | — | The fact is **obj-established without any address**: F3 (34 cells) and F4 (2 cells). This row exists only so a future reader knows the search was not blind |
| **W-INLINE-3** | *(no row — deliberately)* | The budget `B = clamp(2 × caller_instrs, 1000, 35000)` and the 40-instruction free threshold. | `0x10b626f4`–`0x10b62710`, `0x10b6249b`, `0x10b624a2`, `0x10b60a04` — **read and NOT confirmed** | — | — | §4.1: the D family moved the caller from 48 B to 5,640 B and **nothing changed on 12 cells**. The instructions are certainly there; that they are reachable on this workload is not established. It stays `unknown` |
| **W-INLINE-4** | *(no row — deliberately)* | The POGO profitability model and its two 46-dword parameter tables. | `0x10b5fcd8`, `0x10b5b86d`, `0x10b5e4cc`, `DAT_10c45e18` / `DAT_10c45ed0` — **read, unreachable, NOT quoted** | — | — | Method doc §7 case 1, caught before publication: the most model-like code in the inliner is not the code the workload runs. The tables are above the image's raw `.data` and are zero on disc, so no value of theirs is quotable at all |

**If any of these is ever carried**, `README.md`'s clean-room wording must change
in the same commit (ledger step 4), and the code comment must name this file.
