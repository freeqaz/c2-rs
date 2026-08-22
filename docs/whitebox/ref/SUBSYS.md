# `SUBSYS` — subsystem → translation unit → page

Which page do you want? Start here. Front door: [`README.md`](README.md) ·
index: [`ADDR.tsv`](ADDR.tsv) · **whole-image index:
[`FUNCS.tsv`](FUNCS.tsv)**.

> **If your subsystem is not in §1, go to `FUNCS.tsv` and not to a probe grid.**
> §1 is six pages over roughly a quarter of the record's addresses; `FUNCS.tsv`
> has a row for **every one of the image's 4,917 functions**, with the TU, the
> degree, the coverage state, and the string literals and imports each one
> references. It answers *"is anyone home at this address"* for the 89 % of
> functions no document has ever named. See §5.

The **translation-unit** column is c2's own bookkeeping: its C1001 path prints
`compiler file '%s', line %d`, so the binary carries its original source file
names as literals. **The names are Tier 1 — plain `strings` output, no
white-box debt.** The *addresses* are Tier 2.

> ### The instrument's blind spot, and it has bitten twice
>
> `c2_tus.tsv` is built from ICE sites, so **a translation unit with no ICE
> site is invisible to it** and its code is silently absorbed into the preceding
> file's gap. There is **no way to count how many invisible files exist** — the
> method cannot see its own misses, and 52 is a lower bound, never the count.
>
> The instruction scheduler is one of them, and *"there is no `sched.c`"* — a
> true statement about the **instrument** — stood as board `#1823` for months as
> a claim about the **image**. `ADDR.tsv` marks that band `tu_conf =
> no-ice-site`. **In-anchor attributions are facts; gap attributions are
> hypotheses.**

---

## 1. Covered by a page

| subsystem | translation unit(s) | anchor / band | page | entries / band |
|---|---|---|---|---|
| **obj writer** | `coff.c` (model), `coffemit.c` (every `fwrite`) | `0x10b281af`–`0x10b2b0dd` | [`P_COFF.md`](P_COFF.md) | 21 / 120 |
| **section & symbol model** | `p2symtab.c`, `emit.cpp` | `0x10b97dfb`–`0x10b9b8e9`, `0x10be71c9`–`0x10be7e81` | [`P_SECTION.md`](P_SECTION.md) | 24 / 137 |
| **register allocator** | `color.c` (+ `globregs.c`, `regasg.c`) | `0x10b2c21d`–`0x10b3219f` | [`P_REGALLOC.md`](P_REGALLOC.md) | 33 / 70 |
| **DAG build + scheduler** | `dag.c`, **and an unnamed TU with no ICE site** | `0x10b3219f`–`0x10b3433f`, `0x10be5cce`–`0x10be663f` | [`P_DAG.md`](P_DAG.md) | 32 / 61 |
| **inliner** | `inline.c` | `0x10b5b86d`–`0x10b62b00` | [`P_INLINE.md`](P_INLINE.md) | 16 / 93 |
| **instruction encoder** (tuple → one PPC word, plus the `.text` relocation requests) | `code.c` | `0x10bf96d0`–`0x10bfae2a` | [`P_ENCODE.md`](P_ENCODE.md) | 14 / 14 |
| **EH state synthesis** | `ehexcept.c`, `except.c` (+ the `.pdata` drivers) | `0x10be04e7`–`0x10be3800` | [`P_EH.md`](P_EH.md) | 19 / 47 |
| **symbol records: storage class, section number, WEAK EXTERNALS** | `coff.c` (`FUN_10b28a9b`) + `coffemit.c`'s three appenders | `0x10b28a9b`–`0x10b28d6f`, `0x10b2a757` / `0x10b2a8da` / `0x10b2af4f`, `0x10b2823b` | [`P_SYMBOL.md`](P_SYMBOL.md) | 27 / 5 |

---

## 2. Not covered by a page — where to go instead

Nothing below has a reference page. Where a findings document already covers
it, that document is the answer; where nothing does, the cell says so.

| translation unit | anchor | what it is | where the record is |
|---|---|---|---|
| `main.c` | `0x10b7e339` | the driver, the work queue, **the emit walk** | `C2_MAP.md` §3E — but see [`README.md`](README.md) §6.2, the walk is in a Ghidra-missed function at `0x10b7f022` |
| `reader.c` | `0x10bbc9ab` | the `.ex` reader | `WB_READER_FINDINGS.md` — 29 operand classes, the TYPE word, the 48 frontier refusals |
| `lower.c` / `lowersmd.c` / `cgintrin.c` | `0x10c053e7` / `0x10c23539` / `0x10bf080f` | lowering and the pattern library | `WB_SELECT_RECONCILED.md` (the join of three independent readings), `WB_TABLES_FINDINGS.md`, `WB_MEMCPY_FINDINGS.md` |
| `code.c` | `0x10bf9f15` | prologue/epilogue, the per-function register environment — **and the anchor address itself is THE INSTRUCTION ENCODER**: tuple → 32-bit PPC word, via base-word table `0x10c3a578`, encode-form table `0x10c39b18` and a 111-arm switch at `0x10bfae2d` | [`../WB_MIDDLE_INTERFACES.md`](../WB_MIDDLE_INTERFACES.md) §5 (obj-confirmed, 9 words, 32 bits of 32); `WB_FRAME_FINDINGS.md` — **but §2.4's "frame-establish pseudo-register" is wrong**, `0x53` is `lr` ([`P_REGALLOC.md`](P_REGALLOC.md) §6) |
| `lur.c` | `0x10b75e1e` | loop unrolling / rewriting, **15 115 lines** | `WB_LOOP_FINDINGS.md` — the `mtctr`/`bdnz` decision, the rotated pre-test guard |
| `factor.c` | `0x10b34a89` | tail merging (block-level reorder) | `WB_MERGER4_FINDINGS.md`, `WB_DAGCLIENTS_FINDINGS.md` — **tuple order has a second author** |
| `list.c` / `mdlist.c` | `0x10b709b8` / `0x10c11060` | the `/FAsc` listing writers | `C2_MAP.md` §6 P3; the listing seam is how c2 narrates its own output |
| `getflags.c` | `0x10c1f415` | the flag/argv parser | `C2_MAP.md` §6 P2 — the reconstructed 147-entry table replayed **156/156** against real `c2` |
| `hash.c` | `0x10b5a1fc` | **the CSE/value-number hash, `% 0x65`** — *not* the string hash | `C2_MAP.md` §6 P1. c2's actual string hash is `0x10b8a01b`, in a file with **no ICE site** |
| `p2pragma.c` | `0x10b97502` | `#pragma` handling | nothing |
| `globopt.c` / `globlopt.c` | `0x10b4c762` / `0x10b4565a` | the global optimizer, 13 k lines each | nothing |
| `globdf.c` / `globregs.c` | `0x10b40197` / `0x10b55eae` | dataflow, global register promotion | `WB_LIVE_FINDINGS.md` §3 for the liveness fixpoints |
| `dbg.cpp` / `dbgcpp.h` | `0x10be85fe` | the `.debug$S` writer, the only C++ file besides `emit.cpp`/`dll.cpp` | nothing; the gate is `0x10b28548` |
| `pogocg.c` `pogoinline.c` `pogoopt.c` | `0x10ba37cd` … | PGO, **104 imports from `pgodb100.dll`** | nothing, deliberately — dead on this workload; carving it out is pure profit |
| `ltcg.c` | `0x10b72fe7` | link-time codegen | nothing |
| `stack.c` / `mod.c` / `tuple.c` / `fg.c` | `0x10bd0c77` … | stack slots, the module, the tuple IR, the flow graph | scattered; `WB_DAGORDER_FINDINGS.md` §2 for the tuple category enum |
| `error.c` `get_err.c` `ioin.c` `vlines.c` | `0x10c1e4ec` … | `be\common\` | `C2_MAP.md` §1 — **the diagnostic text is not in `c2.dll`**, only numbers, and they are stored as `number − 1000` |
| `dll.cpp` | `0x10bec23c` | the four exports | `C2_MAP.md` §2 |
| `inlnasm.c` `mdmisc.c` `smdmisc.c` `code.c` `mdlist.c` | `0x10c01d50` … | `be\p2\ppc\` and `be\p2\smd\` | the scheduler's machine model lives in `mdmisc.c`'s gap — [`P_DAG.md`](P_DAG.md) §2.1 |

---

## 3. The link-order model, in one paragraph

The 52 files sort by anchor address into **seven maximal ascending runs, and
every run is directory-pure**: 33 + 2 + 1 + 3 files from `be\p2`, then all 6 of
`be\p2\ppc`, all 5 of `be\common`, all 2 of `be\p2\smd` — each non-`p2`
directory appearing as one complete, contiguous, fully alphabetical block.
Against a null of random ordering, `P(runs ≤ 7) = 1.5 × 10⁻²⁵` (exact,
Eulerian) and the 33-file run alone is `1/33! = 1.2 × 10⁻³⁷`. The file *names*
were never used to build the partition, so they are an independent test of it.

**But 72.8% of the anchored span is gap**, 22 of 52 files have a zero-width
anchor (a single function), and the call-graph cross-validation **did not
confirm** the partition — it lacks discriminating power, which is a different
statement and is recorded as such in `C2_MAP.md` §7.1.

> **Measured at function granularity, 2026-08-19 (`FUNCS.tsv`, lane
> `w-c2map3`):** of the image's 4,917 functions, **1,435 (29.2 %) sit inside an
> anchor** — the attributions that are *facts* — **3,469 (70.6 %) are in a
> gap**, i.e. hypotheses, and 104 sit below the first anchor with no attribution
> at all. That is the same 72.8 % seen through a different denominator, and it
> is the number to quote when a claim rests on "function *F* is in `globopt.c`".

---

## 4. `CEILING.md` §6.1's seven phases → what the whitebox record has

Measured at base `e82c9ede6` and re-measured at this tip. **This table is the
answer to "which phase would I be starting from zero on".** `ref` means a
reference page here; `findings` means a dated `WB_*_FINDINGS.md`.

| # | phase | `ref` page | findings | verdict |
|---:|---|---|---|---|
| 1 | **Emitter CFG classes** (`cflow-loop`, `cflow-if-n`, `cflow-if-2`) | — | the **label counter** only (`WB_LABEL_FINDINGS.md`) | **UNSERVED.** The label *numbering* is settled; **block order and branch selection are not read anywhere**. §4.1 names the next read |
| 2 | **An inliner** | [`P_INLINE.md`](P_INLINE.md) | `WB_INLINE_FINDINGS.md` | served |
| 3 | **`memset` / selector lowering** | — | `WB_SELECT_RECONCILED.md`, `WB_TABLES_FINDINGS.md`, `WB_MEMCPY_FINDINGS.md` | served by findings; §2's row is the entry |
| 4 | **Exception handling** | [`P_EH.md`](P_EH.md) | `WB_EH_FINDINGS.md` | served |
| 5 | **Weak externals at scale** (`alias-weak-needed-tus` **675/871**) | **[`P_SYMBOL.md`](P_SYMBOL.md) §2** | — | **served 2026-08-19.** Before that: searching `docs/whitebox/` for `weak.?extern` returned **one** hit, `DISCLOSURE.md`, mentioning the word in passing |
| 6 | **COMDAT synthesis** (§2.3's **450**) | **[`P_SYMBOL.md`](P_SYMBOL.md) §3** | — | **decision site named 2026-08-19** (`0x10b28be6`). Which arm the 450 take is **not** measured, and the page says so |
| 7 | **Regalloc + scheduling across a back edge** | [`P_REGALLOC.md`](P_REGALLOC.md), [`P_DAG.md`](P_DAG.md) | `WB_DAGORDER*`, `WB_DAGCLIENTS`, `WB_LIVE` | served, and re-priced upward twice |

**4 of 7 → 6 of 7.**

### 4.1 Phase 1 is the one still open, and here is the next read

Lane `w-c2map3` did **not** reach it and says so rather than thinning a claim
across it. What a lane opening phase 1 should read first, in this order:

1. `fg.c` at `0x10b36133` — the flow graph. `FUNCS.tsv` gives it **1 function
   with a string hook** in the whole TU (`0x10b3d0f6`, `"Precisions don't
   match"`), so the strings route is nearly dead here and the call graph is the
   handle.
2. `factor.c` at `0x10b34a89` — tail merging, **41 functions, 40 of them
   `cover = none`**. `WB_DAGCLIENTS_FINDINGS.md` already proves two of its
   routines (`0x10b3b167`, `0x10b3b41b`) **reorder tuples**, which makes it a
   block-order author as well as a merger.
3. `0x10b968b0` in `optimize.c` — the only unpaged function in the record
   holding the label format strings `"%s$%s$%d%s"` / `"$%s$%d%s"`.

**Do not start from a probe grid.** `#761`'s cost is on the board: what shipped
for `cflow-loop` was *"a twenty-word transcription of one function class at
`/O1`"*, and `gap-metric cfg-reach-shipped` has been **2 of 16** since.

---

## 5. `FUNCS.tsv` — the whole-image index, and what it is not

`ADDR.tsv` is **bounded by prose**: `build_ref.py` writes a row only for an
address already cited under `docs/` or already hand-labelled. That is the right
shape for *"what is known about this address"* and the wrong shape for *"I am
holding an address nobody has written about"* — which at this tip is **4,388 of
4,917 functions**.

[`FUNCS.tsv`](FUNCS.tsv) is the complement: one row per function in the image,
with TU + confidence, `paged`/`labelled`/`cited`/`none`, degree, hop distance to
the nearest covered function, and the string literals and imports it references.

**Read the honest limits before you rely on it:**

| column | how far it gets you |
|---|---|
| `tu` | a **fact** for 29.2 %, a **hypothesis** for 70.6 % (§3) |
| `strings` / `imports` | present on only **520 of 4,917 = 10.6 %**. It is a *triage* instrument and it triages one function in ten |
| `hop` | **nearly useless in the middle of its range, and this was measured**: 2,196 functions (44.7 %) are at hop 2 and 1,427 at hop 1, so *"two calls from the register allocator"* describes almost half the image. Only the extremes carry signal — `0` (covered), and `-`/`5`/`6+` (**134 functions**, genuinely isolated). Reported rather than dropped, because a future lane will otherwise re-derive the same ranking and believe it. Fifth entry in this repo's *"ranking instruments measure themselves"* pattern |
| `conf = mech` | **weaker than `[R]`.** `[R]` asserts the instructions were read correctly; `mech` asserts only that these tables join here |
