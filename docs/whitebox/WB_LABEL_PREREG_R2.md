# WB-J `wb-label` — PREREG round 2, frozen before the first `cl.exe` of any cell

Lane `wb-label`, 2026-08-09. Companion to `WB_LABEL_PREREG.md` (round 1, frozen
before the first export grep). Precedent for the two-stage freeze:
`WB_READER_PREREG_R2.md`.

> **WHAT HAS HAPPENED BETWEEN THE TWO FREEZES.** The flat export at
> `~/ghidra-projects/export/c2/` was read (`grep`/`sed`/`awk` only, no Ghidra
> project opened), against image sha256 `c80981…6258`. Round 1's mechanism
> predictions are **scored in `WB_LABEL_FINDINGS.md` §1** against that reading
> and are not re-registered here.
>
> **DISCLOSURE — one `cl.exe` ran before this freeze and it was not a cell of
> this lane.** `scripts/configure_existing_worktree.sh` compiles
> `fixtures/cpp/w5_chain.cpp` as its toolchain-resolution assertion (*"OK:
> fixtures/cpp/w5_chain.cpp -> 4/4 functions in class"*), deliberately, so a
> worktree with an absent toolchain cannot silently grade nothing. It is a
> shipped fixture, it is not in any grid below, and no output of it was read
> beyond that one line.

---

## 0. The model, stated so the predictions below are consequences of it

From the binary (findings §1, VAs verified against `objdump_intel.asm` at the
sha above):

* the counter is **`DAT_10c2edd0`**, a single 32-bit TU-global;
* it is post-incremented at **exactly one instruction**, `inc DWORD PTR ds:0x10c2edd0`
  at **`0x10b97de5`**, inside **`FUN_10b97dd0` @ `0x10b97dd0`** (28 bytes), which
  faults with internal-error `0x37` if the counter is still 0;
* `FUN_10b97dd0` has **31 direct call sites**, one of which is the generic label
  constructor **`FUN_10b9a455` @ `0x10b9a455`**, itself called **132 times from
  86 distinct functions**;
* so **the charge is the number of times c2 called a label constructor while
  compiling the function** — a property of c2's internal passes, not of the
  source construct and not of the emitted bytes.

**M — the model in one line:** `stride(f) = |{label objects c2 constructs while
compiling f}|`, of which a **minting** subset reaches the COFF symbol table and
an **internal** subset does not.

## 1. P7 — the `/FAsc` listing seam, sharpened by the binary

The formatter `FUN_10b99dfe` prints label names from **two different fields**:
`$M`/`$T`/`$S`/`$E`/`__catch$`/`__unwind$`/`__annotation$` take their number from
**`sym[+0x28]`** (the global counter), and **`$LC`/`$LL`/`$LN`** take theirs from
**`sym[+0x3f]`**, filled from a *second*, **per-function** counter
`DAT_10c2e918`, reset to 1 in `FUN_10b7e113` @ `0x10b7e113`. `FUN_10b9a455`
bumps **both** on every construction.

That makes the listing an instrument, and these are the falsifiable
consequences:

| # | prediction |
|---|---|
| **P7.1** | The `$LN<k>` indices in a `/FAsc` listing are **per-function and start at 1** — confirming `CFG_SHAPE.md` §7's *"listing-local"* — **but the claim that "nothing should be derived from the numbers" is WRONG**, because `k` is a dense allocation ordinal of the same constructor that moves the global counter. Registered as a correction this lane makes to a shipped document. |
| **P7.2** | **`stride(f) ≥ max($LN<k> printed inside f)` on every cell.** The listing can lose a label (allocated then folded) but can never invent one. A single violation refutes the whole model. |
| **P7.3** | **The printed `$LN` indices are NOT 1..max on at least one loop cell** — there are gaps, and the gaps are the folded labels. This is the direct visual evidence for `LABEL_COUNTER.md` §4.2.2's byte-identical triple. |
| **P7.4** | **THE KILLER CELL.** §4.2.2's triple — `do/while` (+1), `for(;;)+break` (+3), `goto` (+1) — emits 24 byte-identical `.text` bytes. Predicted: the three listings differ, in `max($LN)` or in the printed index set or both. If all three listings are identical too, **P7.4 is a MISS and the listing is not the instrument for the internal population** — say so and do not rescue it. |
| **P7.5** | The `for (i=0;i<10;i++) r=r+a;` body that emits `mulli`+`blr` with no branch at all charges **+2** and prints **0** `$LN` labels, so the listing **under-reports by 2** on it. Registered as the cell that bounds P7.2 from the useful side: the listing is a **lower** bound, never an equality. |
| **P7.6** | The listing does **not** perturb the obj on any cell (board #132). Any perturbation is lane-stopping and reported as such. |

## 2. P8 — the once-per-TU contributors and `LABEL_SEED_GAP`

`crates/c2-core/src/coff/label.rs` has carried `LABEL_SEED_GAP = 9` since the
MVP as a **fitted constant** — *"the first label of a TU is `.gl` counter + 9"* —
with no account of what the nine are.

| # | prediction |
|---|---|
| **P8.1** | **The 9 is nine label-constructor calls made once per TU, before the first function**, not an offset. Predicted mechanism: the TU-preamble section/symbol objects (`FUN_10b9a4a7` @ `0x10b9a4a7` constructs a **named, kind-1** object and takes an upward id at `0x10b9a4c0`). |
| **P8.2** | The `/Gy` **"+3 per function up front"** the port charges is likewise **three section-object constructions per function** in a pre-pass, which is why it is *"three slots per function, whatever kind"* and why it is invisible to any in-TU stride. |
| **P8.3** | `_fltused` (+1) and `w-ifn`'s first-`memcpy` (+1) are **the same mechanism**: the first construction of a compiler-minted named symbol takes an upward id. Predicted: **a third instance exists in the workload's reach** and is findable by the same in-the-middle probe. Registered so the pair is not treated as a closed list. |
| **P8.4** | Rival to P8.1, registered so P8.1 is falsifiable: the 9 is a deliberate reserved gap (a constant added to the seed, e.g. for fixed well-known labels), in which case the binary shows an `add 9` / `+ 9` on the seed path and **not** nine constructor calls. |

## 3. P9 — the frozen obj-check (deliverable 4)

**Construction, frozen:** `scripts/gt_label_stride.py`'s form, **subject in the
middle** — `a0 · P · a1 · a2`, `base = first(a2) − first(a1)` measured in the
same obj, `stride(P) = first(a1) − first(a0) − base`. This is `w-ifn`'s banner
applied: the subject is never first and never last. Flags `/O1 /GS- /c` and
`/Ox /GS- /c`. `minted` is read for every row, per §4's *"read the `minted`
column"* box.

Numbers are **committed**, not ranges. Misses are retractions.

| cell | shape | predicted `stride` `/O1` | predicted `stride` `/Ox` |
|---|---|---:|---:|
| **X1** | framed body + `switch` on an `int` with **12 sparse arms** (values 3, 9, 14, 21, 30, 44, 57, 68, 79, 91, 104, 120) and a `default`, forcing a real jump table | **20** | **19** |
| **X2** | framed body + `for` loop whose body contains an `if` | **13** | **12** |
| **X3** | framed body + `while` loop with an early `return` out of the body | **13** | **12** |
| **X4** | framed body + `try` with **two** `catch` handlers (`int`, `...`), `/EHsc` added to both modes | **28** | **25** |
| **X5** | framed body + a `switch` (X1's arms) **inside** a `for` (X2's loop) | **26** | **24** |
| **X6** | framed body + a loop c2 fully unrolls at `/Ox` and not at `/O1` (`for (i=0;i<4;i++) s += a*i;`) | **12** | **≥13**, i.e. `/Ox` lead **strictly greater** than `/O1`'s |

Reasoning, recorded so a miss is diagnosable rather than a shrug:

* **X1** = framed base 5 + 13 case/`default` target labels + 1 join + 1 table.
* **X2, X3** = framed base 5 + **8**, where 8 is `w-bdnz`'s `/Ox` loop reading and
  the loop handler at `0x10be4f28` constructs **6 unconditionally, +1 if
  `[1]==1`, +1 if `[1]==2`** — a 6/7/8 ladder that is the direct binary
  counterpart of `w-bdnz`'s +7/+8.
* **X4** from `EH_RECORDS.md` §9.8's `11 + 5·S + E`, with `S = 2`, `E = 2`,
  minus the `/Ox` EH deltas `w-main` measured.
* **X5** is registered as **26 against an additive 28**: the sharpest cell, and
  the one where a per-construct table must break if the model is right.
* **X6**: the direction, not the magnitude, is the claim.

| # | prediction |
|---|---|
| **P9.1** | **≥2 of the six will MISS.** Registered first and registered pessimistic, because §0's model says there is no closed-form construct→charge function; **6/6 would be evidence that something was fitted, not that the model is good.** |
| **P9.2** | The **minting** component is exactly right on all six; every miss is in the **internal** component. |
| **P9.3** | **X5 is not additive** — `stride(X5) ≠ stride(X1) + stride(X2) − 5`. This holds even if both magnitudes miss, and it is scored separately. |
| **P9.4** | **X6's mode gap is strictly positive.** Scored separately from its magnitude. |

## 4. P10 — the procedure, and the held-out grade of it (deliverable 5)

The rule/procedure is written **after** X1–X6 are graded. It is then graded on a
**held-out** set frozen in `WB_LABEL_PREREG_R3.md` *before* those cells are
compiled. A procedure that is only ever scored on the cells it was written from
is not a deliverable and this lane will not present one.

| # | prediction |
|---|---|
| **P10.1** | The deliverable is a **procedure, not a closed-form rule** (round 1 P5.3), and it costs **one** compile if P7.4 lands and **two** if it does not. |
| **P10.2** | `IlFunction::label_slots` needs **neither** a mode parameter nor a sub-shape parameter: `None` outside measured classes stays the only correct value, and `w-bdnz`'s argument (#1983) **survives** — strengthened, because the parameter it would need is a count of c2-internal constructor calls. |
| **P10.3** | Registered as what would make P10.2 **wrong**: if the listing exposes the count exactly (P7.4 lands **and** P7.5's under-report turns out to be bounded and predictable), then a **capture-time** channel exists, §4.1's *"any rung that wants a charging shape owes a new IL channel first"* becomes satisfiable, and the honest answer changes from *refuse* to *measure at capture*. Registered so the lane cannot claim credit either way. |
