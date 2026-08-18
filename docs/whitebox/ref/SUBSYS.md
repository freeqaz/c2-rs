# `SUBSYS` — subsystem → translation unit → page

Which page do you want? Start here. Front door: [`README.md`](README.md) ·
index: [`ADDR.tsv`](ADDR.tsv).

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
| **EH state synthesis** | `ehexcept.c`, `except.c` (+ the `.pdata` drivers) | `0x10be04e7`–`0x10be3800` | [`P_EH.md`](P_EH.md) | 19 / 47 |

---

## 2. Not covered by a page — where to go instead

Nothing below has a reference page. Where a findings document already covers
it, that document is the answer; where nothing does, the cell says so.

| translation unit | anchor | what it is | where the record is |
|---|---|---|---|
| `main.c` | `0x10b7e339` | the driver, the work queue, **the emit walk** | `C2_MAP.md` §3E — but see [`README.md`](README.md) §6.2, the walk is in a Ghidra-missed function at `0x10b7f022` |
| `reader.c` | `0x10bbc9ab` | the `.ex` reader | `WB_READER_FINDINGS.md` — 29 operand classes, the TYPE word, the 48 frontier refusals |
| `lower.c` / `lowersmd.c` / `cgintrin.c` | `0x10c053e7` / `0x10c23539` / `0x10bf080f` | lowering and the pattern library | `WB_SELECT_RECONCILED.md` (the join of three independent readings), `WB_TABLES_FINDINGS.md`, `WB_MEMCPY_FINDINGS.md` |
| `code.c` | `0x10bf9f15` | prologue/epilogue, the per-function register environment | `WB_FRAME_FINDINGS.md` — **but §2.4's "frame-establish pseudo-register" is wrong**, `0x53` is `lr` ([`P_REGALLOC.md`](P_REGALLOC.md) §6) |
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
