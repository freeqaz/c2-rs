# The FRONTIER, ranked by distinct unmodeled constructs

Lane **w-frame**, 2026-08-04. Deliverable 1. Reproduce with
`work/w-frame/featmap.py rank` and `work/w-frame/analyse.py`.

`c2rs gap` prints the 17 FRONTIER TUs ranked by **blocked-function count**.
Three lanes took a target off the head of that list and each converted zero:
`w-front` (declined on seam), `w-pair` (six store-scheduling rules killed,
`xboxheap.cpp` scheduler-blocked), `w-cfgimpl` (shipped `cond-tail-pair`,
byte-exact, **0 TUs**). All three independently reported the same cause, and
`BOARD.md` carries it as **#198** — *"the frontier is ranked by
blocked-function count, and that key is wrong"*, itself the fourth instance of
**#150**'s shape.

This is the key those lanes said was missing.

---

## 1. Method

For each of the 17 FRONTIER TUs (read from a real `c2rs gap` run, never
transcribed):

1. **Compile with the real toolchain at the WORKLOAD's own flags.** `cl.exe`
   16.00.11886.00 / `c2.dll` under wibo `1.0.1-23-g4a9dd6f`, flags read from
   `work/dc3-workload/flags.txt`, cwd `../dc3-decomp`. **Not `c2rs compile`** —
   board #195: it hardcodes `/Ox /GS- /c` and drops `--flag`, and w-cfg measured
   that the profile reaches the bytes (per-function optimization word
   `0x00a00005` → `0x00200005`).
2. **Read the obj** — section table, symbol table, relocations, and a big-endian
   PowerPC disassembly of every **code** section (`.text`, `.text$yc`,
   `.text$yd`), via `scripts/gt_dump.py`'s `llvm-mc` seam.
3. **Classify** each emitted function into a feature set over four axes:
   * `insn:<mnemonic>` — instruction family;
   * `frame:<class>` — `leaf` / `stwu` / `mflr` / `savegprlr`;
   * `reloc:<kind>` — `REL24`, `REFHI`/`REFLO`/`PAIR`, `ADDR32`, …;
   * `sym:<family>` — EH records (`__ehfuncinfo$`, `__unwindtable$`,
     `__unwind$`, `__CxxFrameHandler`), frame helpers (`__savegprlr_`), RTTI,
     vftables, string literals, dyninit thunks, and the runtime helpers c2
     *chooses* to call (`memcpy`, `_alldiv`, …);
   plus TU-level `sect:<name>` for every section beyond the four every obj has.
4. **`union(TU)`** = the union over **every** emitted function. A TU matches only
   when the whole obj is byte-exact, so the already-in-class functions count too.
5. **`port_vocab`** = the same classifier over the objs **the port already
   reproduces byte-exact** — the 102 `Port=Match` fixtures plus the 8 matching
   workload TUs. **Measured, never asserted**: if the port emits a construct
   today it is in the vocabulary by construction and cannot inflate a gap.
6. **`gap(TU) = |union(TU) \ port_vocab|`** — the ranking key. A second key,
   **`wit`**, is the per-function distance to the *nearest single witnessed
   function* (`min over witnesses of |f \ w|`), summed over the TU: a flat token
   set is blind to **combination**, and `wit` is not.

### 1.1 What the key is not — registered before the numbers existed

Both of these are in the prereg (`docs/rungs/_2026-08-04-w-frame-prereg.md` §2)
and both **fired on real cells**:

* **It under-counts derivation.** One bucket can be several independent facts.
  Calibration cell: `xboxmem.cpp` scores **gap 1**; w-cfgimpl's 10-cell probe
  grid measured its two remaining functions at **seven** independent facts.
* **It is blind to schedule.** Calibration cell: `xboxheap.cpp` scores **gap 0**
  — every construct already in vocabulary — and w-pair measured it diverging
  at **instruction 0** on instruction *order*. **`gap == 0` is not a conversion
  claim.**

Both errors point the same way: they make TUs look **cheaper** than they are.
Every number below is a **ceiling on cheapness**.

## 2. Controls

| control | result |
|---|---|
| **Leave-one-out on the matched TUs.** Each of the 8 matching workload TUs scored against a vocabulary that **excludes it**. Must be 0 — the port demonstrably emits them byte-exact. | **8/8 clean.** `lex = []` for all 8; `wit = 0` for the 3 that contain code (`TomCryptLicense`, `ZlibLicense`, `Spew`). The other **5 matching TUs emit no function at all** and are reported as `wit = -1`, not silently as 0. |
| **The mode control.** `port_vocab` came from `/Ox` fixture objs; the frontier is `/O1`. Recompiled **all 102** matched fixtures at the workload profile. | **0 tokens gained.** 68 → 67 (`insn:undecodable` is `/Ox`-only). The asymmetry inflates no gap. |
| **Known-unemittable control.** `xboxheap.cpp` must score *low* and is *known* unemittable. | **gap 0** — the blind spot fires on the exact TU that motivated writing it down. |
| **Provenance.** dc3 HEAD bracketed before/after; wibo version recorded. | `940d07dcb096` both ends, clean; `wibo 1.0.1-23-g4a9dd6f`. |

## 3. The ranking

`code B` is the total size of all code sections. `frame × branch` is the count of
functions that are **both** framed and multi-block — see §4.

| # | TU | blocked/emitted | code B | gap | wit | frame × branch | missing constructs |
|---|---|---:|---:|---:|---:|:---:|---|
| 1 | `src/xdk/nuispeech/xboxheap.cpp` | 1/1 | 80 | **0** | 0 | — | — |
| 2 | `src/xdk/nuispeech/xboxmem.cpp` | 2/4 | 132 | **1** | 4 | — | `rlwimi` |
| 3 | `src/system/synth_xbox/Biquad.cpp` | 2/2 | 176 | **1** | 5 | — | `fdivs` |
| 4 | `src/xdk/LIBCMT/undname.cpp` | 1/1 | 140 | **1** | 7 | 1/1 | `bt` |
| 5 | `src/system/negate_test.cpp` | 2/2 | 160 | **2** | 4 | 2/2 | `bt`, `cmpwi` |
| 6 | `src/xdk/LIBCMT/vsnprnc.cpp` | 2/2 | 164 | **2** | 8 | 1/2 | `bt`, `cmpwi` |
| 7 | `src/xdk/LIBCMT/vswprnc.cpp` | 1/1 | 156 | **2** | 8 | 1/1 | `bt`, `cmpwi` |
| 8 | `src/xdk/xlrc/xlrcimpl.cpp` | 1/1 | 152 | **4** | 8 | 1/1 | `frame:savegprlr`, `mr.`, `helper-savegprlr`, `helper-restgprlr` |
| 9 | `src/xdk/nuispeech/mmio.cpp` | 3/11 | 380 | **4** | 13 | 3/11 | `bctrl`, `cmplw`, `mtctr`, `helper-memcpy` |
| 10 | `src/Main.cpp` | 1/1 | 124 | **5** | 3 | — | `reloc:ADDR32`, `eh-funcinfo`, `eh-funclet`, `eh-personality`, `eh-unwindtable` |
| 11 | `src/system/synth_xbox/IPP_basicmath_xbox.cpp` | 4/4 | 184 | **6** | 28 | — | `bclr`, `bdnz`, `lfsx`, `mtctr`, `stfsu`, `stfsx` |
| 12 | `src/system/math/Sort.cpp` | 1/1 | 80 | **7** | 11 | — | `bt`, `divw`, `lbzu`, `mr.`, `mulli`, `rotlwi`, `twi` |
| 13 | `src/system/utl/Pool.cpp` | 3/3 | 132 | **7** | 13 | — | `bclr`, `bdnz`, `cmpwi`, `divw`, `mtctr`, `rotlwi`, `twi` |
| 14 | `src/xdk/LIBCMT/osfinfo.cpp` | 1/1 | 152 | **7** | 13 | 1/1 | `bt`, `clrlwi.`, `cmplw`, `cmpwi`, `lwzx`, `mulli`, `slwi` |
| 15 | `src/xdk/xjson/jsonwriter.cpp` | 1/1 | 304 | **8** | 14 | 1/1 | `frame:savegprlr`, `bt`, `cmplw`, `lhzx`, `rlwimi`, `sthu`, `helper-savegprlr`, `helper-restgprlr` |
| 16 | `src/system/rndobj/wordwrap.cpp` | 3/3 | 816 | **12** | 28 | 1/3 | `frame:savegprlr`, `bt`, `clrlwi.`, `cmplw`, `cmpw`, `cmpwi`, `lbzx`, `lhzx`, `rlwinm.`, `slwi`, `helper-savegprlr`, `helper-restgprlr` |
| 17 | `src/system/utl/EncryptXTEA.cpp` | 4/5 | 272 | **15** | 23 | 1/5 | `frame:savegprlr`, `addic.`, `bdnz`, `clrldi`, `lwzx`, `mtctr`, `rldicl`, `rldimi`, `slwi`, `stdu`, `stdx`, `xor`, `helper-memcpy`, `helper-savegprlr`, `helper-restgprlr` |

### 3.1 The published key and this one disagree — measured

Spearman ρ over the 17:

| pair | ρ |
|---|---:|
| **blocked-function count vs `gap`** | **+0.295** |
| blocked-function count vs `wit` | +0.511 |
| `gap` vs `wit` | +0.813 |

`EncryptXTEA.cpp` and `IPP_basicmath_xbox.cpp` are joint-**last** by both of this
lane's keys and joint-**first** on the published list (4 blocked each).
`negate_test.cpp` is **9th** of 17 on the published list and **5th** here.
`mmio.cpp` — named by w-pair as *"the natural first target for the CFG step"* on
the strength of 8 of its 11 functions already being in class — is **9th**.

### 3.2 The constructs, by how many TUs want them

Ranking rungs by construct rather than by TU:

| construct | frontier TUs wanting it |
|---|---:|
| `insn:bt` (branch-if-true — the `bc` the port does not emit) | **8** |
| `insn:cmpwi` (signed literal compare) | **6** |
| `frame:savegprlr` + its two helpers | **4** |
| `insn:cmplw` (register-vs-register unsigned compare) | **4** |
| `insn:mtctr` (CTR loops and indirect calls) | **4** |
| `insn:bdnz` | 3 |
| `insn:slwi` | 3 |
| everything else | ≤ 2 |
| **22 tokens appear in exactly ONE TU** | 1 each |

Note what the top of that table is: **`bt` and `cmpwi` are one production**.
w-cfgimpl already emits `bf` and `cmplwi` with `BO_TRUE`/`BO_FALSE` and the CR
bits as *named constants* (its prereg A4), so widening to `bt`/`cmpwi` is a
parameter change, not a rewrite. **It converts nothing on its own** — see §4.

---

## 4. The finding this ranking exists to produce

> **The port has emitted 105 functions byte-exact. 28 of them are framed. 2 of
> them branch. ZERO are both.**

Measured over every function in every obj the port reproduces byte-exact — the
102 `Port=Match` fixtures and the 8 matching workload TUs. `frame` is
`stwu | mflr | savegprlr`; `branch` is a real block boundary (`bf`, `bt`,
`bclr`, `bdnz`) and deliberately **excludes `insn:b`**, because a tail call is a
`b` and counting it would make the product look witnessed when it is not.

The port's **entire branching capability is two functions**, both leaves, both
from w-cfgimpl's `cond-tail-pair` rung:
`w8_cond_tail.cpp:?f@@YAXPAX0K@Z` and `w8_cond_tail_value.cpp:?fs@@YAKPAX0K@Z`.

**The frontier needs the product in 10 of 17 TUs, across 13 functions.**

That is the precise form of w-cfgimpl's *"the frontier's wall is the general
framed-function class"*, and it is **sharper than that statement**: the wall is
not frames — the port emits 28 framed functions. It is not branches either. It
is **frames × blocks**, a cell of the cross-product with no witness anywhere in
the port's history. Everything that makes a framed body multi-block is
un-witnessed at once: block layout inside a frame, branch-displacement fixup
against a materialized epilogue, the intra-section `b` encoding (board **#191**),
and callee-saved register allocation across the blocks.

### 4.1 …and it is not the whole wall

**Seven** frontier TUs need no framed-and-branching function at all, and every
one of them is blocked by something else the ranking makes explicit:

| TU | why it still does not convert |
|---|---|
| `xboxheap.cpp` | schedule — w-pair, diverges at instruction 0 |
| `xboxmem.cpp` | w-cfgimpl's four folds + an in-arm assignment, both declined with a probe grid |
| `Biquad.cpp` | leaf-branching + `fdivs`; a `lis`/`lfs` pair **straddling** the compare (schedule again) |
| `Main.cpp` | the EH critical path — `.pdata` ×2, the EH `.rdata` group, the two-word in-`.text` prefix, a funclet |
| `Sort.cpp`, `Pool.cpp`, `IPP_basicmath_xbox.cpp` | leaf CTR loops (`mtctr`/`bdnz`/`bclr`), `divw`, `twi` — 6–7 constructs each |

So the honest statement of the frontier's shape is **two walls, not one**:
frames × blocks (10 TUs) and, for the other 7, a per-TU set of expression and
schedule facts with almost no overlap between them (22 of the missing tokens
occur in exactly one TU).

---

## 5. Where the key stops, and what has to replace it

The mechanized key **excludes** correctly: everything at `gap ≥ 6` is genuinely
6+ constructs away, and no lane should look there. It **does not rank the head**:
its top three (`xboxheap` 0, `xboxmem` 1, `Biquad` 1) are all TUs whose real
cost is independently known to be far higher, because their remaining distance
is *derivation* and *schedule*, which no obj-side classifier can see.

**The final pick still requires reading the disassembly.** §6 of the rung doc
does that read for the plausible candidates and hand-counts the *independent*
refusals — which is the quantity the project's own estimate rule is written in
terms of, and which is 5 at its lowest.
