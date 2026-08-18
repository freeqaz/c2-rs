# `P_INLINE` — `inline.c`: the inline decision function

> **Reference page.** **`[R]`** read from the disassembly, *not* obj-checked —
> a hypothesis. **`[O]`** confirmed against a real obj or `/FAsc` listing, with
> the witness named. **`[I]`** an interpretive step. Navigation only; nothing
> here may enter `crates/` without a [`DISCLOSURE.md`](../DISCLOSURE.md) row.
> Index: [`ADDR.tsv`](ADDR.tsv) · front door: [`README.md`](README.md)

**Coverage: 16 entries against a denominator of 93** — Ghidra functions in the
inliner band `0x10b5b86d`–`0x10b62b00` (`inline.c`'s anchor span plus the gaps
on both sides that hold the parameter tables and the legality check). Not
covered: `ptinl.c` entirely, the expansion's own body rewrite beyond its entry,
and both 46-dword POGO parameter tables (read, unreachable, deliberately not
quoted — §5).

> ### The headline, and it is a warning about fitted rules
>
> **`INLINE-P` — the project's incumbent predicate — is EXACTLY RIGHT inside
> the class it was fitted on and wrong outside it in two measured directions:
> a flag axis it does not have, and a LOOP axis nothing has ever had.** On the
> `keygen_xbox.cpp` anchor it predicts **six** inlines and gets **one** `[O]`.
> Its EXTERNAL clause `s ≤ 112` sits inside the measured `(100,116]`, and its
> STATIC cap `s ≥ 308` matches the measured `(300,308]` **to the word** — every
> one of its misses is at a flag set its corpus never contained.

---

## 1. The chain, top down

| addr | size | callers | callees | TU | cites | what |
|---|---:|---:|---:|---|---:|---|
| `0x10b62675` | 464 | 1 | 11 | `inline.c` gap | 4 | **the pass entry, per function.** Skipped wholesale when `DAT_10c40ec4 == 0` `[R]` |
| `0x10b626d8` | *(in `0x10b62675`)* | — | — | `inline.c` gap | 2 | `DAT_10c3f5cc = (ushort)[fn+0x50]` — the caller's **instruction count**, the running growth total `[R]` |
| `0x10b6276a` | *(in `0x10b62675`)* | — | — | `inline.c` gap | 1 | `FUN_10b61ee1(fn, level=1, budget=B, 0, 100000000, 0)` `[R]` |
| `0x10b61ee1` | 539 | 2 | 16 | `inline.c` gap | 3 | **the driver** — collects the sites, loops over them, returns *budget consumed* `[R]` |
| `0x10b600e6` | 1062 | 1 | 8 | `inline.c` gap | 4 | **the site collector.** One linear scan; instruction kind **`0x0f`** is a call site. Tracks EH-region nesting through opcodes `0x2ee/0x2f0/0x2f1/0x2f4/0x2f6/0x2ff/0x300` and stamps a conditional/EH flag into bit 1 of the candidate `[R]` |
| `0x10b5fb5f` | 377 | 3 | 5 | `inline.c` gap | 3 | **candidacy — where the size ceiling is.** §2 `[R]` |
| `0x10b5c06b` | 60 | 5 | 0 | `hash.c` gap | 5 | **legality.** Refuses on `[sym+0x20] & {0x400, 0x1000, 0x40, 0x100}` and `[sym+0x4c] & {0x80000, 0x200}`; requires bit 6 of `[sym+0x4c]` `[R]` |
| `0x10b61d2c` | 437 | 1 | 10 | `inline.c` gap | 1 | per-site driver `[R]` |
| `0x10b60930` | 358 | 1 | 7 | `inline.c` gap | 4 | **the accept/decline predicate** — depth, budget, POGO `[R]` |
| `0x10b6242a` | 587 | 1 | 6 | `inline.c` gap | 1 | **the charge**, and the second copy of the 40-instruction test `[R]` |
| `0x10b620fc` | 814 | 1 | 24 | `inline.c` gap | 1 | **the expansion**, recursing back into `0x10b61ee1` for the inlined body `[R]` |
| `0x10b5fcd8` | 1038 | 1 | 6 | `inline.c` gap | 7 | **the profitability model — POGO ONLY.** Reached from `0x10b60930` only when a profile record exists `[R]`. §5 |
| `0x10b600c8` | *(in `0x10b5fcd8`)* | — | — | `inline.c` gap | 3 | the per-site-count discount `cost -= (K + cost) / n_sites` `[R]` |
| `0x10b5e4cc` | 101 | 1 | 3 | **`inline.c` anchor** | 4 | **the ceiling itself**: `DAT_10c46318 = 0x10 << DAT_10c2ea98` (16 instructions << k), or `1000` when `k ≥ 7` `[R]` |
| `0x10b5e6a5` | 768 | 3 | 8 | **`inline.c` anchor** | 2 | the savings vector for the POGO model `[R]` |
| `0x10b5b86d` | 34 | 1 | 0 | `hash.c` gap | 3 | selects between the two 46-dword parameter tables (`DAT_10c45e18` / `DAT_10c45ed0`) `[R]` |

Diagnostics: `"INL:\tInlining %s (%d instrs) into "` at `0x10b025ec`, and the
`-optref` pruner's `"INF:\t%s not allowed to be inlined (globally
unreferenced)"` — **the quantity is an instruction count c2 holds before
codegen, not a byte count** `[R]`.

---

## 2. The decision function `[R]`

### 2.1 Candidacy — and the switch that turns the size test off

`0x10b5fb5f`, the arm that returns 1:

```
0x10b5fdfd   cmp DWORD [0x10c2e310], 0      <- THE FAVOR-SPEED BIT
                                                if non-zero the size test is SKIPPED
0x10b5fe0c   movzx eax, WORD [sym+0x50]     <- the callee's INSTRUCTION COUNT
0x10b5fe14   cmp eax, DWORD [0x10c46318]    <- the ceiling; `jl` = candidate
0x10b5fe1e   test DWORD [sym+0x4c], 0x2000  <- __forceinline: bypass
```

`0x10c2e310` is the same option-word bit 23 (written at `0x10b8238d`) that moves
`memcpy`'s inline threshold — **two mechanisms now shown to hang off one bit**.
This is why no grid compiled at a single flag set could ever see the ceiling
move.

### 2.2 The budget — `B = clamp(2 × caller_instrs, 1000, 35000)`

```
0x10b626f4   uVar7 = 1000
0x10b626fb   if (2*caller_instrs > 1000) uVar7 = 2*caller_instrs
0x10b62708   if (uVar7 > 34999)          uVar7 = 35000

0x10b6249b   cmp WORD [callee+0x50], 0x28      <- 40 instructions
0x10b624a2   *budget -= WORD [callee+0x50]     <- charged only if > 40
0x10b60a04   if (budget < instrs && instrs > 0x28) return DECLINE
```

> **A callee of 40 instructions or fewer is never charged against the budget and
> is never declined for affordability.** The budget is a growth cap for *large*
> callees only. `[R]`

### 2.3 Depth and the categorical arms `[R]`

```
0x10b609ae   0x10 < level - DAT_10c3f50c            -> decline   (16 levels)
0x10b609bd   maxlevel != 0xff && maxlevel < level   -> decline
0x10b609d3   test [sym+0x4c], 0x2000                -> __forceinline bypasses
                                                       every size and budget test
0x10b609ee   35000 < DAT_10c3f5cc                   -> decline
```

---

## 3. The measured boundaries `[O]` — GRID-I, 264 frozen cells

`s` = the callee's own emitted `.text`, measured. The bracket is
*(last inlined, first called]*.

| family | `/O1` | `/O2` | `/O1 /Ot` | `/O2 /Os` | `/O1 /Ob0` |
|---|---|---|---|---|---|
| **STATIC**, straight-line | **(300, 308]** | **(212, 252]** | **(212, 252]** | **(300, 308]** | nothing inlines |
| **EXTERNAL**, straight-line | **(100, 116]** | **(156, 164]** | — | — | — |
| **loop-bodied** (GRID-J, 56 cells) | **(56, 80]** | — | — | — | — |

> **The threshold follows FAVOR-SPEED, not the `/O<n>` level.** `/O1 /Ot`
> behaves as `/O2`; `/O2 /Os` behaves as `/O1`. Same two mixed cells that
> decided `wb-memcpy`'s GRID-W.

Facts, each a statement about c2 rather than about a rival:

| # | fact | cells |
|---|---|---:|
| F3 | `/Ob0` declines **everything**, including `__forceinline` | 34 |
| F4 | `__forceinline` inlines a **980-byte** callee, at `/O1` and `/O2` | 2 |
| F5 | varargs and direct recursion decline categorically at every flag set | 6 |
| F7 | **the caller's own size is NOT an input** — a 48-byte caller and a 5 640-byte caller give identical verdicts at every size and both flag sets | 12 |
| F8 | a **control-dependent** site at `s = 212` declines at `/O1` where the unguarded one inlines; at `/O2` it does not | 6 |
| F9 | a **loop-bodied** callee declines at `(56,80]` where a straight-line one inlines to `(96,120]`, **identically at the workload flags and at `/O1 /GS- /c`** — so it is the loop and not the flags | 56 |

**No rival survives.** Scores: R2-CEILING 226/264, R1-INCUMBENT 218/264,
R5-NOSITES 195, R3-SIZE64 168, R4-OBLEVEL 144. R2's 38 misses are all
*parameter* errors; R1's 46 are all *structural* — it has no flag axis at all.

### 3.1 F7 refutes this page's own §2.2 as a *practical* input

The budget is read correctly — the instructions are there — but moving the
caller from 48 B to 5 640 B (i.e. `B` from 1 000 to ~2 820) changes **nothing on
12 cells** `[O]`. Consistent with §2.2 (everything at `k ≤ 40` is free of the
budget, everything above is already refused by the ceiling), and it means the
budget is **not reachable from the flag/size space anyone has swept**. Recorded
as **READ, NOT CONFIRMED**; no `DISCLOSURE` row proposes it.

---

## 4. `?supershuffle` — the clause that actually fires

On the real `keygen_xbox.cpp` obj the six shuffles are **104 / 60 / 84 / 84 /
88 / 88** bytes and **only the 60-byte one is inlined** `[O]`. All six are under
`INLINE-P`'s published 112-byte EXTERNAL ceiling.

> **The clause is the LOOP-CLASS size ceiling at `(60,80]`** — `cmp [sym+0x50],
> [0x10c46318]` at `0x10b5fe14`. `?shuffle2` is 60 B; the next smallest is 84 B.
> That is why it, and nothing else in the TU.

The registered prediction said the clause was the EXTERNAL `index ≤ 64` arm.
**The first half is wrong** — the real arm is three times tighter — and the
second half is right for a reason the prediction did not have. Scored a miss.

---

## 5. What is NOT known here — and one thing deliberately not quoted

* **The POGO cost model (`0x10b5fcd8`) is unreachable on this workload.** It is
  a full cost/benefit model with ~20 tunable weights copied from one of two
  **46-dword parameter tables** (`DAT_10c45e18` / `DAT_10c45ed0`, selected at
  `0x10b5b86d`). Those tables live above the image's raw `.data`
  (`0x10c3cc00`), so they are **zero at load and written at run time** — none of
  their values is quotable from the image and **this page does not quote them**.
  `0x10b60930` reaches the model only when the callee has a profile record.

  > **This is `C2_MAP_METHOD.md` §7 case 1 in advance: the most model-like code
  > in the inliner is not the code the workload takes**, and a lane that read it
  > and stopped would have published a cost model c2 never runs.

* **The `16 << k` ceiling does not compose into the measured numbers.**
  `0x10c2ea98`'s image value is `3`, giving `16 << 3 = 128` **instructions**;
  the measured straight-line ceilings are 25–29 and 37–41 emitted words
  (EXTERNAL) and 53–65 / 75–77 (STATIC). The reading is **named and not claimed
  as the boundary the workload takes** — something between the two, most
  plausibly the linkage arm and the `[sym+0x50]`-vs-emitted-size gap, is unread.
* The **depth cap of 16** (`0x10b609ae`): no cell nests 16 deep.
* `ptinl.c`: not opened.
* 320 cells is not a total statement about c2.
