# `WB_DAGPRICE` — the `[dag]` band attribution settled, the read plan ranked, and the price with its derivation printed

Lane `w-dagprice`, wave 20 L4, 2026-08-29. Kind **characterization**. Outcome
**built**. Board **#3838**–**#3844**. Prereg (committed before the image was
opened): [`../../work/w-dagprice/PREREG.md`](../../work/w-dagprice/PREREG.md).
Instruments: [`../../work/w-dagprice/`](../../work/w-dagprice/) —
`band_probe.py`, `band_probe2.py`, `tu_signal.py`, `tu_signal2.py`,
`tu_signal3.py`, `rank_artifact.py`, `span2.sh`, `span3.sh`.

> **PROVENANCE — DISASSEMBLY-DERIVED.** Image
> `compilers/X360/16.00.11886.00/c2.dll`, sha256
> `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`,
> **verified in-lane before any address below was read** — it matches the pin at
> [`ref/README.md`](ref/README.md):21. Nothing here may enter `crates/` without
> a [`DISCLOSURE.md`](DISCLOSURE.md) row. **This lane writes no `crates/` code**
> and adds no `DISCLOSURE.md` row; predicted reach **0**, byte delta **0**, both
> shown to have held in the rung.
>
> **Legend** ([`ref/README.md`](ref/README.md) §2): `[R]` read from the
> disassembly, not obj-confirmed — *a hypothesis*; `[O]` confirmed against a
> real obj or `/FAsc` listing; `[I]` inferred. Everything in §1–§4 below is
> `[R]`, and §1's structural facts are read from the **flat Ghidra export**
> (`functions.tsv`, `calls.tsv`, `xrefs.tsv`, dated 2026-08-04) rather than from
> raw bytes — a weaker channel than a byte read and named as such.

---

## 0. The three headlines

1. **The `[dag]` band attribution splits into three claims with three different
   standings, and the repo has been quoting the weakest one's caveat over the
   strongest one's evidence.** The FUNCTIONAL claim (*these functions are the
   scheduler*) never depended on `c2_tus.tsv` at all; the TRANSLATION-UNIT claim
   now has **positive** evidence for the first time; and the EXTENT claim —
   the one nobody had tested — is **wrong at both edges**. §1.
2. **The brief's *"the load-bearing assumption under every `[dag]` number"* is
   false for the headline number.** `[O]` 7 of 50 is `count_marks()` over
   `P_DAG.md`'s prose (`crates/c2-harness/src/subsys.rs:857`) and is invariant
   under any re-attribution of the band. It is load-bearing for exactly two
   figures, the `read` row's denominators **61** and **83**, and **61 counts a
   function that is not the scheduler's**. §2.
3. **The `[dag]` read plan's binding constraint is not reading.** Nine ranked
   targets are priced at **0.6–2.5 agent-lane hours each**, calibrated on nine
   executed read lanes measured from git — and **every one of them lands `[R]`
   and stays `[R]`**, because this repo has no DAG observation channel and a
   corpus whose entire order signal is 8 tuple positions of 3,015 (`#3728`).
   The honest price is a **pair**, and the second element is 0 by construction.
   §5, §6.

---

## 1. The band attribution — SETTLED, and it was three claims

`ref/SUBSYS.md`'s blind-spot box and `docs/WAVE20_BRIEF_2026-08-29.md` §2 both
say the scheduler band `0x10be5cce`–`0x10be663f` is *"a TU with NO ICE SITE, so
even attributing that band to the scheduler is a hypothesis rather than a
fact."* The prereg registered, before measuring, that this sentence conflates
three claims. It does.

### 1.1 Claim A — FUNCTIONAL: *these functions are the scheduler.* **SURVIVES, and it was never a `c2_tus.tsv` claim.**

`c2_tus.tsv` is a **file-name** partition built from C1001 ICE sites. It has no
opinion about what a function *does*. Every ground for calling these functions a
list scheduler is independent of it:

* the bodies are read — `ref/P_DAG.md` §2 pages 12 of them, and §3/§5 give the
  priority function and the latency mechanism from the instructions `[R]`;
* the region rule is graded **1,461 / 1,461** against the live tap over 60
  fixtures (`scripts/grade_regions.py`, R7, `#3434`) — four of its nine clauses
  pinned (`#3726`);
* the **four runs per function** are `[O]` by four separately-patched call sites
  whose counts equal c2's own `/FAsc` `PROC` count and the obj's `.text` COMDAT
  count, 7/7/7 (`w-stageoracle`, `P_DAG.md` §2's correction box);
* and — measured in this lane — the 12 form a **single-entry call component**:

```
  == THE 12 (0x10be5cce .. 0x10be663d) ==
    span 0x10be5cce..0x10be663d = 2416 bytes
    ENTRY 0x10be6382  from 0x10b7dc51, 0x10b7df57
    external entry points: 1 of 12
```

`0x10b7dc51` and `0x10b7df57` are exactly the two phase drivers `P_DAG.md` §1's
table names as the mode-1 and mode-0 sites. **Zero** of the other eleven is
reachable from outside. `work/w-dagprice/band_probe2.py`.

> **The category error, and it is `#1823`'s own, inverted.** `#1823` read a true
> statement about the *instrument* (*"there is no `sched.c` in `c2_tus.tsv`"*) as
> a statement about the *image*. `SUBSYS.md`'s blind-spot box exists to stop
> that — and the box is now being read the same way in the other direction: a
> caveat about the **file-name partition** is being quoted as a caveat about
> **what the code does**. `c2_tus.tsv` cannot weaken claim A because claim A was
> never derived from it.

### 1.2 Claim B — EXTENT: *the band's members are the scheduler's functions.* **REFUTED, at both edges.**

**Top edge.** The published band ends at `0x10be663f`. The last function whose
*start* is inside it is `FUN_10be663e`, **1,197 bytes long**, of which the band
covers **2** (0.17 %). It is in the denominator because a band membership test
compares start addresses. It is not the scheduler:

| evidence | value |
|---|---|
| callers | `0x10be6aeb`, `0x10be6bca` — **no band function calls it, and it calls none** |
| callees | `0x10c28862`, `0x10c28868` (imports) |
| its callers' string refs | **`clui.dll`**, **`PATH`** (`ref/FUNCS.tsv`) |
| its cluster's outside caller | `0x10c1f1bb` |
| stack frame | **1,248 bytes** (`functions.tsv`) |
| exclusive `.data` shared with the 12 | **none** |

The correct top edge is `0x10be663d` — `0x10be6382 + 700 − 1`, the last byte of
the scheduler driver. **The band as published overshoots the driver by two bytes
and admits a 1,197-byte stranger.**

**Bottom edge.** `FUN_10be5cbe` (16 B) ends exactly where the band starts and is
called by **`0x10c1bbaf`** — `mdmisc.c`'s dynamic priority bonus, itself a
scheduler function (`P_DAG.md` §2). `ref/FUNCS.tsv` hands it to
`except.c` / `eh` / `P_EH.md`, an attribution **no call edge supports**: nothing
in `except.c`'s anchored extent calls it.

**And the same mid-function defect is on the `dag.c` half.** The published
`dag.c` band ends at `0x10b3433f`, which is inside `FUN_10b3421b`
(`0x10b3421b`..`0x10b34398`). Both endpoints of the pair `P_DAG.md`:9-10 counts
`61` from are mid-instruction-stream. Eleven further functions
(`0x10b34399`..`0x10b34a88`) sit between that band's end and `factor.c`'s
anchor, unattributed by the band and given to `dag.c` by the gap rule.

### 1.3 Claim C — TRANSLATION UNIT: *the band is one compiland.* **POSITIVE EVIDENCE, for the first time — and the control is watched.**

The prereg refused in advance to turn an absence into a positive. So the lane
built a **positive** test.

**The test.** A compiland's file-scope statics are concatenated into `.data` by
the linker in OBJ order. A real translation unit should therefore own a
**contiguous, exclusively-referenced run** of `.data` words, bounded above and
below by other compilands' runs. Scored over `.data`/`.bss` only
(`0x10c2e000`–`0x10c90000`); `.rdata` constants are shared freely and say
nothing about a compiland, which is why the window is stated rather than
assumed. `work/w-dagprice/tu_signal2.py`.

**The 12 score 14, at `0x10c3d140`..`0x10c3d180`** — a 0x44-byte block, 13 of
whose 14 referenced words have **no referrer anywhere outside the 12**. The
fourteenth is `0x10c3d144`, which `P_DAG.md` §2 already names in the issue-width
rule and which `dag.c` (`0x10b328da`) and `mdmisc.c` (`0x10c1b9da`,
`0x10c1c0e5`) also read. The block is bounded on both sides by other compilands'
blocks — `sizeopt.c` at `0x10c3d0ac`..`0x10c3d138` below, `regasg.c` at
`0x10c3d18c`..`0x10c3d19c` above.

> ### ⛔ THE FIRST CONTROL CAME BACK BROKEN, AND IT IS RECORDED RATHER THAN DROPPED
>
> `work/w-dagprice/tu_signal.py` ran the test with positive controls built from
> `c2_tus.tsv`'s **anchor extents**. Every one scored **below** the subject —
> `dag.c` 5, `color.c` 6, `reader.c` 7, `except.c` 5, `emit.cpp` 4, `inline.c` 0,
> `tuple.c` 0, against the subject's 14. That is a **broken control, not a strong
> subject**: an ICE anchor brackets only the ICE-*bearing* functions, so an
> anchor-extent set is a **subset** of its compiland, and every datum a left-out
> sibling also touches is scored "shared" and discarded. The control is biased
> down by construction. **A test whose positive control loses to its subject has
> not been passed; it has not been run.** (`#3336`, and MEMORY's *"printed is not
> watched"*.)

**Repaired positive control** — the full nearest-anchor partition
(`ref/FUNCS.tsv`'s `tu` column), 48 TUs of ≥ 12 functions:

```
  smdmisc.c 537 · inline.c 75 · hash.c 63 · sizeopt.c 33 · p2symtab.c 19 ·
  misc.c 19 · lur.c 19 · dll.cpp 19 · coffemit.c 19 · pogocg.c 15 ·
  pogoopt.c 14 · [THE SUBJECT] 14 · tuple.c 13 · mod.c 13 · mdmisc.c 13 · …
  48 TUs; median run 6.0, mean 21.08, max 537
```

The subject sits at **rank 12 of 48**, squarely inside the distribution of
things that are known to be compilands, and above `dag.c` (5), `except.c` (5),
`emit.cpp` (7) and `mdmisc.c` (13).

**Negative control — and this is the number that decides whether the signal
means anything.** Every sliding window of 12 consecutive functions in the image,
N = 4,905. **40 (0.82 %) match or beat the subject's run of 14.** Located:

```
    0x10b5b5f7..0x10b5c1f0  best-run=63   tu=hash.c
    0x10b5e531..0x10b6174c  best-run=22   tu=inline.c
    0x10b76491..0x10b76cef  best-run=19   tu=lur.c
    0x10be5cce..0x10be6382  best-run=14   tu=(unnamed TU: no ICE site)   <-- SUBJECT
    0x10c28e0e..0x10c2a3b6  best-run=537  tu=smdmisc.c
    -> 40 overlapping windows collapse to 5 distinct code regions
```

**Four of the five are named compilands. The fifth is the subject.** So a win on
this test is not a coincidence of locality — in the whole image, an exclusive
contiguous `.data` run this long happens in five places and four of them are
translation units.

**And the test discriminates inside the very same gap.** The 29 functions
*above* the band (`0x10be663e`..`0x10be717f`), which `ref/FUNCS.tsv` hands to
`except.c` by the nearest-anchor rule, own **zero** exclusive `.data`, and their
union with `except.c`'s own anchored functions does not beat `except.c` alone
(both 5, identical range). Same gap, same instrument, opposite answer.

**Verdict on C: the 12 are a distinct compiland `[R]`.** What is *not*
recoverable is its **name**: the only naming channel in this image is the ICE
site's `__FILE__` string, and the band has none. The image holds 52 such names
and no scheduler-like one.

> **One inference this lane REFUSES, having measured it.** The subject's `.data`
> neighbours are `sizeopt.c` below and `regasg.c` above; it would be natural to
> read link-order position off that. **It is not available.** Spearman
> ρ(`.text` rank, `.data` rank) = **0.378** over the 29 TUs with a run ≥ 6
> (`work/w-dagprice/tu_signal3.py`): `.data` order is only weakly related to
> `.text` order in this image, so the neighbours say nothing about which
> compilands the scheduler sits beside. The exclusivity argument stands alone
> and is not helped by adjacency.

### 1.4 What `[O]` 7/50 means if the attribution is wrong — the brief's question, answered

**Nothing. It does not move by one.** §2 shows why.

### 1.5 The epistemic standing of `[dag]`, restated correctly

`SUBSYS.md`'s rule is *"in-anchor attributions are facts; gap attributions are
hypotheses."* Applied to `[dag]`'s own denominator of 61, from
`ref/FUNCS.tsv`'s `tu_conf` column over the 83 functions with `subsys=dag`:

```
   69 gap          <- hypotheses
   13 no-ice-site  <- the band
    1 in-anchor    <- FACT
```

**One.** `dag.c` has a single ICE site, at `0x10b3219f`, source line 1429
(`c2_tus.tsv`). Its published band `0x10b3219f`–`0x10b3433f` is 47 functions of
gap extrapolation off that one anchor. **The scheduler band is not `[dag]`'s
weak half** — it is 13 of 61, and it is now the *only* part of the subsystem with
a positive TU test behind it. The other 47 are hypotheses of exactly the shape
the blind-spot box warns about, and they have been presented as the solid side.

---

## 2. What the band attribution is actually load-bearing FOR

Re-derived on this tree, not relayed.

| `[dag]` figure | source | moves if the band is re-attributed? |
|---|---|---|
| **`agreement` `[O]` 7 of 50 (14.0 %)** | `count_marks(P_DAG.md)` — literal `[R]`/`[O]`/`[I]` substrings after the page's first `---` line, `crates/c2-harness/src/subsys.rs:857-875` | **NO.** It reads a markdown file and nothing else. Re-derived: `R 43 O 7 I 0 total 50` |
| `read` sites **61** | `P_DAG.md`:9-10, `48 + 13` | **YES — and it is already wrong**: the 13 includes `FUN_10be663e`, which §1.2 refutes. The scheduler count is **12**, so the pair is `48 + 12 = 60` |
| `read` second denominator **83** | `ref/FUNCS.tsv` `subsys=dag` | **YES** — 13 of the 83 are the band |
| `read` numerator **32** (24 code + 8 data) | `P_DAG.md`'s own §2/§2.1 tables | **NO** — a hand count of the page's rows |
| `exercised` | RESIDUE | n/a |
| `byte-owned` | CITED `#3534` | n/a |

**So the brief's *"the load-bearing assumption under every `[dag]` number this
repo publishes"* is FALSE for the headline number and TRUE for two
denominators.** It is worth saying plainly because the reverse belief is what
made the row look epistemically hopeless: the number that reads 14.0 % is a
prose census, and the numbers the band *does* carry are the ones nobody quotes.

> ### ⛔ AND THE `agreement` STRENGTH IS WRITABLE BY PROSE — INCLUDING BY THIS LANE
>
> Because `agreement` counts marks in the page body, **any lane that edits a
> reference page moves its subsystem's published agreement percentage**, with no
> new evidence of any kind. This lane amends `ref/P_DAG.md` (§7), and here is
> the movement, measured before and after exactly as wave 20's L1 is required to
> do for the clause table:
>
> ```
>   BEFORE   [R] 43  [O] 7  [I] 0   total 50   ->  [O] 14.0 %
>   AFTER    [R] 46  [O] 7  [I] 0   total 53   ->  [O] 13.2 %
> ```
>
> **The subsystem's headline agreement went DOWN by 0.8 points on a lane that
> settled its band attribution, refuted its extent, corrected its denominator
> and read a previously-unread discriminator.** Not one `[O]` was lost and not
> one fact was withdrawn; three `[R]` sentences were added and the denominator
> grew. **The metric moves in the wrong direction for prose-adding work and in
> no direction at all for the band**, which is the whole of what `agreement`
> measures on this row. Board **#3841**.
>
> Recipe, so the pair can be re-derived rather than trusted:
> ```
> python3 -c "p=open('docs/whitebox/ref/P_DAG.md').read().split(chr(10));\
> i=[k for k,l in enumerate(p) if l.rstrip()=='---'][0];b=chr(10).join(p[i+1:]);\
> print(b.count('[R]'),b.count('[O]'),b.count('[I]'))"
> ```

---

## 3. The read that was TAKEN in-lane, because it prices row 1 empirically

`ref/P_DAG.md` §5's amended box (`#3433`) names one live defect and stops one
step short of it:

> *"**ALU→ADDRESS = 5 and ALU→DATA = 2 ARE THE SAME CELL.** Cell `(1,8)` holds
> the tag `-2` and is **the only cell of all 121 that does**; it resolves to
> **2** when `edge+0x19` bit 1 is set and **5** otherwise … A model that picks
> one number for this cell is wrong on the other half."*

**Nothing in this repo says who sets `edge+0x19` bit 1.** It is now read. `[R]`,
from the flat export's decompilation.

### 3.1 Bit 1 is CLEARED at edge creation

`FUN_10b32113` @ **`0x10b32113`** (116 B — `P_DAG.md` §2's "edge create"), after
setting `+0x10 kind` and `+0x14 latency`:

```c
*(byte *)(edge + 0x19) = *(byte *)(edge + 0x19) & 0xfd;   /* clear bit 1 */
```

So **every edge is born with the `-2` cell resolving to 5**, and the `2` is an
override applied later. `P_DAG.md` §5's *"resolves to 2 when the bit is set and
5 otherwise"* is right, and this adds the polarity a port needs: **5 is the
default.**

### 3.2 The sole setter is `FUN_10c1bc78` @ `0x10c1bc78` (114 B)

An image-wide scan of the decompilation for `+ 0x19)` returns 23 occurrences in
6 functions. Exactly one writes bit 1 on a DAG edge:

```c
void __thiscall FUN_10c1bc78(void *edge, undefined4 op, int param_2)
{
  src_tuple = *(int *)(*(int *)(edge + 8)  + 0x1c);   /* edge+0x8 = src node */
  dst_tuple = *(int *)(*(int *)(edge + 0xc) + 0x1c);  /* edge+0xc = dst node */
  if (param_2 != 0 && (*(byte *)(param_2 + 0x14) & 0x20) == 0
      && 0x14c < *(uint *)(dst_tuple + 4) && *(uint *)(dst_tuple + 4) < 0x181
      && FUN_10c1b98a(src_tuple) == 1        /* producer class 1 = integer ALU */
      && FUN_10c1b98a(dst_tuple) == 8) {     /* consumer class 8 = int ld/st   */
      *(byte *)(edge + 0x19) |= 2;
  }
  if (src_tuple->cat != 0x15 && dst_tuple->cat != 0x15
      && src_tuple->op != 0x2ed && dst_tuple->op != 0x2ed)
      FUN_10c1c1d4(edge);                    /* the edge-latency computation */
}
```

`node+0x1c → tuple` is the join `WB_SCHEDCHK_FINDINGS.md` §7.3 independently
identifies (written `0x10b327de`, read `0x10c1c1ea`), and this body reads it
from **both** ends of the edge — a second, independent witness for `edge+0x8` /
`edge+0xc` being the src/dst node pointers.

**Four conditions, and three of them are exactly cell (1,8):** producer class 1,
consumer class 8, consumer opcode in `[0x14d, 0x180]` — the range
`WB_SCHEDCONF_FINDINGS.md` §2.4 reads out of the tag dispatch at
`0x10c1c294`–`0x10c1c2b3`. **The fourth is new and is the whole finding.**

### 3.3 The fourth condition — the latency-2 half is GATED on a record nothing in this repo names

`param_2 != 0 && (param_2[0x14] & 0x20) == 0`. `param_2` comes from the only
DAG-side caller, `FUN_10b322ba` @ `0x10b322ba` (`P_DAG.md` §2's true-dependence
edge builder), where it is that function's **fifth argument**:

```c
if (param_3 == 1)    { e = FUN_10b32187(prev, node, 1);    FUN_10c1bc78(e, param_1, param_5); }
else if (param_3 == 4)   { e = FUN_10b32187(prev, node, 4);    FUN_10c1bcea(param_1, e); }
else if (param_3 == 0x20){ e = FUN_10b32187(prev, node, 0x20); FUN_10c1c02c(e, param_5); }
else                     {     FUN_10b32187(prev, node, param_3); }
```

**Consequences, each separable:**

* **`P_DAG.md` §2's row for `0x10b322ba` is incomplete in a way that matters.**
  It reads *"true-dep edges from last writers (**register kind 4**, memory kind
  `0x80`)"*. The function dispatches on **kinds 1, 4 and `0x20`**, plus a
  default, and it is **kind 1** — not kind 4 — that reaches the `+0x19` setter
  and the latency computation. The edge-kind inventory the whole latency model
  hangs off is missing its most consequential member. Board **#3842**.
* **`0x10b322ba` has two more callers than `P_DAG.md`'s "3 callers" row
  suggests** for the setter: `0x10c1bc78` is reached from `0x10b322ba`,
  `0x10c1bcea` and `0x10c1c02c` — one DAG-side and two `mdmisc.c`-side.
* **`crates/` does not carry the wrong number, because it carries no number.**
  `P_DAG.md` §5's *"`crates/` carries the `2` and not the `5`"* and
  `WB_SCHEDCONF_FINDINGS.md` §8.4's restatement of it are **loose**:
  `DISCLOSURE.md` has no `[dag]` adoption row at all; the `2` they point at is
  `order.rs`'s `u = min(2, #unproduced)`, a **store-slot count** that was
  *fitted* over 250 cells, and its identification with the ALU→store-data
  latency is `P_DAG.md` §4's own interpretive step, not a read. Correct
  statement: **the port has no latency model, so it cannot carry a wrong cell —
  and the read above is a prerequisite for the first one it builds, not a fix
  to an existing one.** Board **#3843**.

**This read took ~15 minutes of the lane, start to finish, and is the empirical
price of RD1 in §5.**

---

## 4. Two more things the export handed over on the way, recorded so a later lane does not re-find them

* **The DAG has two installable client hooks.** `FUN_10b32113` ends with
  `if (DAT_10c435bc != 0) (*DAT_10c435bc)(edge, kind);` and `FUN_10b327cd`
  (node create) with `if (DAT_10c435c0 != 0) (*DAT_10c435c0)();`. A per-edge and
  a per-node callback, dispatched through globals. `P_DAG.md` §6's *"a second
  author of tuple order exists"* bullet points at merge **clients**; this is the
  **mechanism** by which a client attaches, and neither pointer is named
  anywhere in this repo. RD6 in §5.
* **`FUN_10b327cd` assigns `node+0x4e` bits 2 and 3 from two predicate calls**
  (`FUN_10b32516` → bit 2, `FUN_10b324f9` → bit 3) and then **clears bit 4**
  (`& 0xef`) unconditionally. `P_DAG.md` §3's R7 box has bits 2/3 as *"the
  operand chain at `tuple+0x28`/`tuple+0x2c` holds a record of kind byte 2 or
  6"* — the two predicates are the addresses that claim needs and it does not
  carry them.

---

## 5. The ranked read plan — nine rows, in `READ_PLAN_2026-08-21.md` §3's style

**Denominator, stated because a top-N with an unstated N is `#3505`'s shape:**
**15 candidates considered, 9 ranked, 6 rejected.** The six, with the reason
each was rejected — five of them because the read is already taken, which is the
finding that most changes what a `[dag]` wave should be funded to do:

| rejected candidate | why |
|---|---|
| the latency matrix's six negative tags | **all six decoded**, `WB_SCHEDCONF_FINDINGS.md` §2.4 lines 199–204 |
| the region rule's five unpinned clauses | **read**; the gap is the *corpus*, not the reading (`#3726`). Pricing it as a read repeats R7's own defect |
| the `0x50` region cap | **read** at `0x10be5d66`; confirmed only as the ray ≥ 13, again a corpus gap (`#3727`) |
| `0x10c3afd8`, the per-opcode attribute table | **read** — [`ref/P_OPATTR.md`](ref/P_OPATTR.md), `#3460`–`#3463` |
| the block merger `0x10b3baa8` → `0x10b3a790` | **mostly closed**: `w-s7` measured both it and the splicer behind `sym+0x20 & 0x1000`, clear on **2,946 of 2,946** functions at the workload's mode (`#3737`, `#3738`) |
| `factor.c`'s tail merging | a different TU and a different subsystem row; naming it here is scope creep |

> **RANK BASIS, and the artifact check that was registered against it.** Every
> row below is ranked by **a named blocked claim with its citation** — a line in
> this tree that today says a thing is unknown, fitted, `[R]`-only or ungraded.
> A row that could not name one was deleted, not demoted.
> `work/w-dagprice/rank_artifact.py` scores the registered artifact test:
> **Spearman ρ(rank, named-body bytes) = +0.000 over n = 8**, against a
> registered threshold of `|ρ| ≥ 0.700`. **NOT FIRED.** The ranking is not
> predicted by candidate size in either direction — which is the check `#3505`
> is six for six on, and MEMORY's *"ranking instruments measure themselves"*
> four for four.

| rank | read | entry points | the blocked claim it unblocks, cited | span | lands |
|---:|---|---|---|---:|---|
| **RD1** | **`edge+0x19` bit 1 — the only two-valued latency cell's discriminator** | setter `0x10c1bc78` (114 B); cleared at `0x10b32113`; consumed `0x10c1c1d4`; caller `0x10b322ba` | `P_DAG.md` §5 box `#3433`: *"a model that picks one number for this cell is wrong on the other half"*; `WB_SCHEDCONF` §8.4 | **0.25 h — EXECUTED IN THIS LANE, §3** | `[R]` |
| **RD2** | **the DAG build `FUN_10b328da` (2,231 B, 21 callees)** — the largest unread body in the subsystem | `0x10b328da`; the edge minters `0x10b32187`/`0x10b32113`/`0x10b322ba`/`0x10b3227c`; node create `0x10b327cd` | `P_DAG.md`:98 is **one line** for it; §3 finds its `0x10b322ba` row is missing edge **kind 1**, and every latency the scheduler uses is joined through a kind | 1.5–2.5 h | `[R]` |
| **RD3** | **the 31 `cover=none` bodies of the `dag.c` band** — `0x10b33a9f` (683), `0x10b338f5` (426), `0x10b32676` (264), `0x10b33423` (259), `0x10b33dd8` (222), … | each in `ref/FUNCS.tsv` | `P_DAG.md`:9 — *"24 code entries … against a denominator of 61"*. **37 of 61 unread**, and `read` is the only `[dag]` strength that is neither RESIDUE nor a prose census | 2.0–3.0 h | `[R]` |
| **RD4** | **the mid-level (mode 1) pass** — what differs from mode 0 | `0x10be6382`'s mode parameter and its uses; the three mode-1 sites `0x10b7dc9f`/`0x10b7dcde`/`0x10b7dd1d` | `P_DAG.md` §6 bullet 3, verbatim: *"the mid-level (pre-lowering) pass's differences from the machine-level one, beyond the `0x2b8` store special case"*. **Three of the four runs are mode 1** | 1.0–2.0 h | `[R]` |
| **RD5** | **`edge+0x19` bits 0/2/3 and the re-schedule iteration** | `0x10c1bdff` (483 B — sets and tests **bit 0**); the bit-1 reader `0x10c1c3f7` (2,716 B) | `P_DAG.md` §2's `0x10c1bdff` row: *"the schedule is ITERATED — recorded latency requests can rewrite edge slots and force a re-schedule of the same region `[R]`"* — **nobody has said when it fires**, and a port that schedules once is wrong wherever it does | 1.0–1.5 h | `[R]` |
| **RD6** | **the DAG's two client hooks `DAT_10c435bc` (per edge) / `DAT_10c435c0` (per node)** — who installs them at `/O1 /EHsc` | `0x10b32113` tail, `0x10b327cd` tail; writers of the two globals | `P_DAG.md` §6 bullet 2: *"a second author of tuple order exists … the scheduler is not the only thing that moves tuples"* — this is the **attachment mechanism**, named nowhere | 0.5–1.0 h | `[R]` |
| **RD7** | **`0x10be663e`'s cluster** — `0x10be663e` (1,197), `0x10be6aeb` (223), `0x10be6bca` (229), outside caller `0x10c1f1bb` | as listed | **this lane's §1.2**: the function is inside the published `[dag]` band and is not the scheduler. A **denominator fix**, not a scheduler read | 0.5–1.0 h | `[R]` |
| **RD8** | **the 29 gap neighbours above the band**, currently `except.c` | `0x10be663e`..`0x10be717f` | `SUBSYS.md`'s own rule. Measured here: they own **zero** exclusive `.data` and their union with `except.c` does not beat `except.c` alone — the attribution has **no support** in either direction | 1.0–2.0 h | `[R]` |
| **RD9** | ⛔ **NOT A READ — the discriminating corpus and the DAG observation channel** | `WB_SCHEDCONF` §8.1 (a population that reorders, ≈1 d); `WB_SCHEDCHK` §7.3 (a node walker at the **existing** region hook, off by one region, via `DAT_10c435e0`) | `#3435`: *"that confrontation is not available on this corpus at any price"*; `#3728`: the order channel is **8 tuple positions of 3,015** | see §6 | the only row that can move a tier |

> **RD9 is ranked last and is the most important row in the table, and that is
> not a paradox.** It is last because it is **not a read**, and read-before-probe
> is about preferring a read *when a read would answer the question*. Here the
> reads are largely already taken — five of the six rejected candidates are
> rejected *because they are read* — and no further read changes a tier.
> **A `[dag]` wave funded as reads adds rows to a page that already cannot be
> graded.** That is the shape wave 20's own brief diagnoses for the inliner
> clause table, one subsystem over.

---

## 6. The price, with its derivation printed

`ROADMAP.md` §11.8 / `#3603`: *a figure whose inputs cannot be corrected cannot
be re-priced — it can only be withdrawn or left standing with its derivation
printed.* So every number here prints its inputs.

**`STEP5_PRICING_2026-08-21.md` §2.1 is cited and not copied** (`#3370`'s
mitigation — that block is canonical and every other surface in this tree points
at it rather than duplicating it). Nothing below restates a figure from it.

### 6.1 The unit, and why the published one is the wrong unit

`#3605` measured three executed rows against their published day-prices and
found them **pessimistic by 30×–1,200×**, closing with: *"these are **agent-lane
wall clock, not human effort** — which is the finding, not a caveat on it."*
This lane extends that sample from three to nine, prereg-commit → merge-commit,
by `#3605`'s own method (`work/w-dagprice/span3.sh`, re-runnable):

```
  w-read-r4    2026-08-23 00:18:00 -> 01:20:41    62 min
  w-read-r5    2026-08-23 00:20:40 -> 01:01:53    41 min
  w-read-r7    2026-08-23 02:12:32 -> 03:11:52    59 min
  w-read-r8    2026-08-23 02:04:22 -> 03:28:54    84 min
  w-s7         2026-08-28 07:30:59 -> 09:05:42    94 min
  w-f0price    2026-08-27 20:12:54 -> 22:16:39   123 min
  w-regcells   2026-08-27 20:17:26 -> 20:55:58    38 min
  w-encarms    2026-08-28 09:50:42 -> 12:22:59   152 min
  w-globarms   2026-08-29 02:08:22 -> 02:24:04    15 min   (subject-prefix span,
                                                            span2.sh)
```

**n = 9, range 15–152 min, median 62 min, mean 74 min.** Four of the nine carry a
published `READ_PLAN` §3 day-price, so the ratio can be computed rather than
asserted:

| lane | published | realized | ratio at 24 h/d | ratio at 8 h/d |
|---|---|---:|---:|---:|
| R4 | 3–5 d | 1.03 h | 70×–116× | 23×–39× |
| R5 | 15–25 d | 0.68 h | 526×–877× | 176×–294× |
| R7 | 3–5 d | 0.98 h | 73×–122× | 24×–41× |
| R8 | 5–10 d | 1.40 h | 86×–171× | 29×–57× |

*(derivation: `published_days × hours_per_day ÷ realized_hours`; both
conventions printed because `#3605`'s headline 30×–1,200× uses calendar hours
and a reader assuming an 8-hour day would silently get a third of it.)*

**Median ratio ≈ 100× at 24 h/d, ≈ 35× at 8 h/d.** The `READ_PLAN` §3 day-prices
are not wrong arithmetic; they are **the wrong unit**, and §4 of that page says
so for reads already. RD1–RD8's `span` column above is in the unit that was
actually spent.

### 6.2 The whole `[dag]` read plan, priced

**Construction: 7.75–13.25 agent-lane hours** for RD1–RD8, of which RD1 is
**spent** (§3). Derivation: the sum of the span column, `0.25 + 2.0 + 2.5 + 1.5
+ 1.25 + 0.75 + 0.75 + 1.5` at midpoints = **10.5 h**, bracketed by the row
ranges. Each row's range is the observed 15–152 min lane distribution scaled by
whether the row is one function (RD1, RD6, RD7), one large function (RD2, RD4,
RD5), or an enumeration (RD3, RD8).

**Conversion: 0. Not "small" — zero, and by construction, three ways over:**

1. **There is no adoption path.** `DISCLOSURE.md` has **no `[dag]` row**. The
   port schedules nothing (`c2rs subsys`: *"the port schedules nothing — emission
   order is tuple-list order … No site-level numerator is defined"*), and
   `READ_PLAN` §2's scope bound `#3366` fences `codegen::{alloc, order,
   schedule}` to `leaf/store.rs`. **No `[dag]` read can change an emitted byte
   until a construct lane exists that would consume it, and decision 20 §2 and
   this brief both forbid building one.**
2. **The byte judge cannot see it.** A scheduler model is graded on tuple order,
   and `#3728` sized that channel at **8 tuple positions of 3,015** at the final
   schedule — a model returning its input scores ~99 % (`#3435`). Two models are
   separated only if they disagree on one of those 8.
3. **The tap cannot see it either.** `P_DAG.md` §3's second correction box:
   *"not one term of the priority function is observable by the stage tap, at any
   setting, at any of its eight sites"* — `stagetap.c` emits **tuple** fields;
   **no site dereferences the DAG at all**.

**So the pair is (10.5 h construction, 0 conversion)** — and that is precisely
`#3605`'s shape restated in a subsystem where it can be predicted in advance
rather than discovered afterwards: *"the estimates were pricing CONSTRUCTION,
which is measurably nearly free, and pricing NOTHING about conversion, which is
not."*

### 6.3 THE DECLINE, and the number it rests on

**This lane declines to recommend funding RD1–RD8 as a wave, and the reason is
the price, not the difficulty.** *(Recorded here and in the rung's prose;
`declined` in the outcome vocabulary is reserved for a lane that declined to
convert a fixture, so this lane reports `built`.)*

Ten and a half hours of reading buys **eight more `[R]` rows on a page that
already has 43**, on a subsystem where the standing measurement is that **no
setting of any instrument this repo owns can promote one of them**. The
agreement strength would move — from 14.0 % toward whatever the new prose
census gives — **without one new fact being confirmed**, which §2's box shows is
a property of the metric and not of the work.

**What the same budget buys instead, with its own citations:**

| | price, in the same unit | what it changes |
|---|---|---|
| **RD9a — a population that reorders** | ≈ 1 d, `WB_SCHEDCONF` §8.1 (**published, not re-derived here**) | it is the **precondition**, not an optimisation — `WB_SCHEDCHK` §6.2. With 8 discriminating positions, a model with *k* free binary decisions is at best `8/k`-determined |
| **RD9b — the DAG node walker at the existing region hook** | **explicitly unpriced**, `WB_SCHEDCHK` §7.4, and this lane does not price it either | it is the only thing that makes any priority-function clause observable. §7.4 already refuted the one published estimate for it (§8.2's *"0.5 d, three fields"* — wrong record, wrong hook, wrong time) |

**Two directions, per `#3603`, and they point opposite ways.** Construction is
~30×–100× cheaper than the published prices say. Conversion is **not cheaper —
it is unavailable at any price on this corpus**, which is `#3435`'s exact
wording. A price quoting only the first is the error `#3603` names; a price
quoting only the second reads as "hopeless", which is also wrong, because RD9a
is priced at one day and would lift the ceiling for **every** row at once.

### 6.4 What would make THIS lane's price wrong

Registered, so the next lane can correct the inputs rather than withdraw the
figure:

1. **The span sample is nine agent lanes on one box in eight days.** If lane
   dispatch changes shape, the 15–152 min distribution is void. Recipe:
   `sh work/w-dagprice/span3.sh`.
2. **The four ratio rows use `READ_PLAN` §3's published brackets as the
   numerator.** If those brackets were never intended as wall clock — and §4 of
   that page half-says so — the ratios measure a units mismatch, not
   pessimism. That reading is *consistent with* this table, not excluded by it.
3. **"Conversion = 0" is conditional on decision 20 §2 and on `#3366`'s scope
   bound.** If a scheduler construct lane is ever funded, the 0 becomes
   unmeasured, not 0.
4. **RD3 and RD8 are enumerations and their spans are the least grounded**
   figures in the table — no executed lane in the sample was an enumeration of
   30 bodies.

---

## 7. What this lane changed in `ref/P_DAG.md`

Three amendment boxes, **amended beside and never rewritten**
(`ref/README.md` §2.1): the coverage banner's `61`, §2's `0x10b322ba` row, and
§5's *"`crates/` carries the 2"* clause. The page's mark census moves as a
consequence; both values are in the rung (§2's box).

## 8. What this lane refused

Every refusal registered in `work/w-dagprice/PREREG.md` §4 held:

1. **No TU claim from an absence, in either direction.** Claim C is carried by a
   positive test with a watched control, and the *name* is reported as
   unrecoverable rather than guessed.
2. **No claim that any read unblocks F0 or F5.** `P_REGALLOC.md` §7 as amended
   prices F0 at ≥ 10 raw sub-lanes plus two UNPRICED terms and says both
   published figures are floors. Nothing above touches that, and `P_REGALLOC.md`
   is outside this lane's seam.
3. **No scheduler-model grade from this corpus.** §6.2 point 2.
4. **No `STEP5_PRICING` §2.1 figures restated.** §6.
5. **No new `gate.sh` row** (`#3691`).
