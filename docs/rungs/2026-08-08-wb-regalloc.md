# wb-regalloc — the register-choice policy, read off c2 and graded by objs; `cr0` retracted

    Tag:       WB-D
    Slug:      wb-regalloc
    Date:      2026-08-08
    Fixtures:  none — grid sources live in docs/whitebox/grids/wb-regalloc/, deliberately NOT in fixtures/cpp/
    Census:    +0 — WHITEBOX/navigation lane, adopts nothing into crates/
    Record:    docs/whitebox/WB_REGALLOC_FINDINGS.md

---

## PREREG

[`docs/whitebox/WB_REGALLOC_PREREG.md`](../whitebox/WB_REGALLOC_PREREG.md),
committed at **`a02c3e04`** — before the first grep of
`~/ghidra-projects/export/c2/` and before the first `cl.exe` of this lane.
Twenty-three rows, each with its direction if wrong. Scored in
`WB_REGALLOC_FINDINGS.md` §8: **15 hit · 7 miss · 1 unscoreable**, five of the
seven misses in their registered direction.

The obj-check predictions are separately frozen: `WB_REGALLOC_FINDINGS.md` §6
was committed at **`0a6de90f`**, before the grid was compiled.

## The result

> **The success floor is CLEARED.** The register-choice policy read off
> `color.c` survived a frozen check on **six** cells, including two functions
> outside every shipped class (`wbr_loop_call`, `wbr_pressure`) that the lane
> got right in advance. The ordering policy did **not** survive as a rule: it
> has one consistency cell and two counter-facts.

`sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`
verified at the top of the lane.

| | |
|---|---|
| register-choice rivals refuted | **3 of 4** (on 6, 9 and 4 cells) |
| register-choice rival surviving | **R0** — min-cost, ties by `r11,r10,…,r3,r31,…,r14` |
| named functions graded (deliverable 4) | **3**, of which **2 clean hits** and 1 hit-on-the-graded-claim |
| retractions | **2** — the `cr0` prediction (#1824) and block-order-is-source-order (#1825) |
| campaign-1 corrections | **3** — `0x53` is `lr`, `DAT_10c6fd9c` is the frame pointer, `DAT_10c2e980` is `-QGPRReserve` |
| board rows minted | **#1820**–**#1830**; **#1831**–**#1839** explicitly unminted |
| `crates/` changed | **none** |

## What it admits, and what it refuses

**Admits**, obj-confirmed: the GPR allocation order
`r11,r10,r9,r8,r7,r6,r5,r4,r3, r31,r30,…,r15,r14`; that `r12`/`r13`/`r0`/`sp`/
`toc` are never allocatable; that the first callee-saved register taken is the
**top** of the file, not the bottom; that `cr6` is what an ordinary integer
compare uses.

**Admits**, read but not obj-checked (navigation, and named so absence does not
read as coverage): the cost function and the strict-`<` tie-break (no obj in
this project separates it from "only one candidate was legal"); the FPR order
`fp0, fp13, …, fp1, fp31, …, fp14` (no cell uses floating point); the
operand-nibble→class map `0x10b022cc`; the `0x500400` frame-pointer flag
family; the `-QGPRReserve` and POGO variant arrays.

**Refuses**: any claim about block emission order (§7.6 has two cells with two
different answers); any claim about instruction *selection* (`dag.c`, `lur.c`,
`globlopt.c`, `cgintrin.c` and the switch lowering were not opened); and any
claim that this lane's reading converts a TU — see #1829.

## Estimate vs outcome

| registered | outcome |
|---|---|
| P0.2: **ordering** survives, **register** does not | **inverted** — the register policy is the one that took a frozen check |
| P1.2 / P3.3: a real instruction scheduler exists | **no scheduler at all** (#1823) |
| P2.5 / F3: `cr0` for an ordinary compare | **`cr6`, every time** — retracted (#1824) |
| P3.2: block order = construction order | **refuted by one obj** (#1825) |
| P4.3: registers, ≤ 1 of 3 named functions hit | **3 of 3** hit on the graded register claim |
| P5.3: first class reaches ≤ 6 of the reach-pool | **held, and revised down to "most likely 0"** (#1829) |

Board #770's streak gains **3 optimistic** (P1.2, P3.3, P0.2's inversion) and
**2 pessimistic** (P1.3, P4.3).

## What the follow-on code lane should do

1. **Do not ship a class lowering expecting a conversion.** #1829 registers
   `≤ 6, most likely 0` over the 124-TU reach-pool, and the reason is the
   reader, not the emitter. A lane that measures 0 and reports failure has
   mis-read this rung.
2. **The cheapest real adoption is the order alone**, and it needs **no
   `DISCLOSURE` row**: cells G1–G4 and P1 of
   `docs/whitebox/grids/wb-regalloc/regorder_grid.cpp` establish
   `r11,r10,…,r3,r31,…,r14` against real `c2.dll` with no address. Ship it as a
   black-box result, per `DISCLOSURE.md` step 5.
3. **Shipping the full selector needs W-REGALLOC-2** (`WB_REGALLOC_FINDINGS.md`
   §10), because the tie-break is not establishable black-box today.
4. **`cr6` should be adopted black-box** (W-REGALLOC-3 exists only to record
   why a `cr0` prediction was retracted).
5. **The next whitebox rung is `lur.c`/`cgintrin.c`, not `color.c`.** §9.2 lists
   what a class lowering actually needs and registers are item 4 of 4.

## Housekeeping

`docs/whitebox/c2_functions.tsv` is **not** regenerated in this lane. The new
labels are in `docs/whitebox/labels/W-REGALLOC.tsv` (19 rows, 8 of them
function addresses that `build_map.py` picks up; verified by regenerating to a
scratch path) and two corrections are in `labels/W-FRAME.tsv`. Regeneration is
left to the coordinator so that four concurrent campaign lanes do not each
rewrite the same generated table.
