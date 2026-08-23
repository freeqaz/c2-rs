# DISCLOSURE — disassembly-derived findings adopted into the port

> **PROVENANCE — DISASSEMBLY-DERIVED.** This directory is the output of a static
> analysis of Microsoft's `c2.dll`. See [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0
> for the exact bytes.

## What this file is for

`README.md` currently makes a **blanket** clean-room claim: the original binary
is treated as a black box and only its observable output informs the port.
`docs/ROADMAP.md` §9.8 states the consequence precisely:

> If a disassembly-derived constant is ever adopted, that blanket claim must
> weaken to per-finding disclosure, naming the site in the relevant `docs/` file.

§9.4 previously recommended taking on **no** white-box debt. The user has now
explicitly authorized this analysis, so that recommendation is superseded for the
`w-map` lane — but the disclosure discipline is not. This file is the ledger that
makes the consequence handleable instead of quietly broken.

## Two provenance tiers — they are not the same, and the difference is cheap to keep

The lane's central artifact, the translation-unit partition, is **not uniformly
white-box**. It has two components with genuinely different provenance, and
pooling them would concede more than the work actually costs.

| tier | what | provenance | debt |
|---|---|---|---|
| **TIER 1** | **the list of 53 file names** (`coff.c`, `coffemit.c`, … — [`C2_MAP.md`](C2_MAP.md) §3A) | c2's C1001 path prints `compiler file '%s', line %d`, so these are **plain `strings` output** — an observable of the black box, recoverable without a disassembler | **none** |
| **TIER 2** | **every address**: the ICE-site xrefs, the derived per-file ranges, and all function labels | reading the disassembly | white-box |

`docs/ROADMAP.md` §9.8 already blesses tier 1's class explicitly: **the
diagnostic strings are named there as an observable output of the black box**,
alongside the obj, the `/FAsc` listing and the error text. Nothing about
extracting them requires or implies disassembly — `strings c2.dll | grep vctools`
is sufficient and is the same category of observation as reading a `C1007`
message.

**Consequence: the file-name list on its own incurs no white-box debt at all.**
A reader who only wants to know that this compiler's back end is built from
`p2\`, `p2\ppc\`, `p2\smd\` and `common\`, that EH is split across `ehexcept.c`
and `except.c`, or that `coff.c` and `coffemit.c` are separate translation
units, can have all of that from tier 1.

What tier 2 buys on top is *where* — the ranges that turn the name list into a
map. That is real white-box debt and is not minimised here. But it is worth
noting which half of the lane's headline result rests on it: the **link-order
validation** in §3.2 (7 ascending runs against 26.5 expected,
P = 1.5 × 10⁻²⁵; every run directory-pure) is a joint fact about tier 1 and
tier 2 and needs both. The **file inventory** needs only tier 1.

Keep the tiers apart in anything derived from this directory. Blurring them
costs the project more than the analysis did.

## The rule

**Navigation is free; adoption is not.**

* Using this directory to decide *where to look* in the binary, or to decide
  *which black-box experiment to run next*, costs nothing and needs no entry
  here. A map is navigation.
* **Copying a value, a table, a bit layout, or an algorithm out of the
  disassembly and into `crates/` is adoption**, and requires a row below *in the
  same commit as the code change*, naming the address it came from.
* The grey zone — a white-box finding that told you what to look for, which you
  then re-derived and confirmed purely from black-box observation — should still
  be logged, marked `route:` in the Kind column. It cost the blanket claim
  nothing, but a future reader deserves to know the search was not blind.

## Adopted findings

| # | Kind | What was adopted | Address in `c2.dll` | Adopted into | Commit | Notes |
|---|---|---|---|---|---|---|
| **W-OBJPLAN-1** | **adoption** | **The emit-SEED bit: `0x20` at symbol offset `0x4c`.** c2's work-queue walk over the global function list (`.data 0x10c4630c`, next `+0x78`) loads the per-function flag word from `[eax+0x4c]` and selects on `test dl,0x20`; bit `0x02` is set by the loop itself, so the load-bearing bit is `0x20`. The port's `plan::predict` seeds its predicted emit set with exactly that bit, read out of the low byte of the `.gl` tag-`0x0e` record's `+0x4c` field — a **bit position**, so adoption and not navigation. | **`0x10b7f16b`** (`mov edx,[eax+0x4c]`), **`0x10b7f16e`** (`test dl,0x20`), `0x10b7f171`/`0x10b7f173`/`0x10b7f176` (the skip/dequeue arms), all inside `0x10b7f022` — **not** inside `FUN_10b7f1ff`, which is `C2_MAP.md` §3E's own corrected reading; the record decode that places the field is §3E's tag-`0x0e` walk at handler `0x10b9bdcf` | `crates/c2-core/src/plan/mod.rs` — `FN_FLAG_EMIT_SEED` and its doc | (this commit) | **It is a SEED and the port says so in the constant's own name.** §3E's cascade measurement is that clearing `0x20` on 17 of 20 functions of a bundle with a real call graph changed nothing — the emitted set is the seeded set CLOSED under "referenced by an already-emitted function", and §3E's practical warning is that a port using the seed alone *"will over-delete on real TUs"*. Nothing in `crates/` emits on the strength of this bit: it feeds **one instrument** (`gap-metric plan-emitset-*`), which is graded against real c2's own objs on every scan, and the seed's containment in the observed emit set is published as `plan-emitset-seed-subset` precisely so a wrong identification would show up as an over-claim rather than as a plausible number. **The grey-zone alternative does not exist here**: the byte the reader returns is already decoded (`gl_function_attrs`, whose consumer reads bit 6 of the same byte); what the disassembly supplies is *which bit means emit*, and no black-box experiment over `.gl` can name a bit position. **AMENDED TWICE THE SAME DAY — first by the instrument this row exists for, then by the REVIEW OF THAT INSTRUMENT. The reading is sound; the RULE built on it is refuted; and the CEILING first published beside the refutation was a fact about the reader's scan, not about `.gl`.** Over 870 TUs `gl_function_attrs` names **28,107** `.gl` function records; **28,104** carry bit 6 and **0** read `0x00`, which rules out the UNIFORM-ZERO mis-decode signature that reader's own doc records — **and rules out nothing else.** This bit is set on **331** of them — 1.18 % — and the seed is **EMPTY on 739 of the 854 TUs** where the reader answers. A closure over an empty seed is empty, so §3E's seed-plus-closure model **cannot be built out of this reader's output**; that refutation stands. **THE FIRST AMENDMENT ALSO SAID *"the map covers only 17.7 % of the emitted set, which ceilings anything keyed off it whichever bit is chosen"*, AND THAT IS WITHDRAWN.** It was `|names| / |emitted|` with no intersection taken, and it attributed to `.gl` what belongs to the walk: `gl_function_attrs` advances `p += 1` past any unframed offset with **no refusal and no counter**. Measured at stamp `6f3a818e9893`, twice, byte-identically: of the 28,107 records the reader names, **3,933** are functions c2 actually emitted — **2.4 %** of the **162,146** emitted — while `c2_il::mangled_names`, which walks `.gl` symbol runs in file order and **does not use the record framing at all**, names **70,114** of them, **43.2 %**. **Eighteen times the reach, over the same `.gl` bytes.** The attribute byte's full histogram (bit0 1 · bit1 0 · bit2 1,494 · bit3 28,107 · bit4 0 · bit5 331 · bit6 28,104 · bit7 26,961) shows three varying bits and four constant ones — structure, hence consistent with a decoded field on the records reached, offered as corroboration and not as proof. Where the seed IS non-empty it is very good — **exact** against c2's emitted set on **27** TUs, **9** of them TUs the port does not convert, and it over-claims exactly **once** in the whole workload (`src/system/utl/TempoMap.cpp`). §3E's own cascade was measured on a 20-function probe bundle; this is 870 real TUs. **The constant stays in `crates/` because the refutation is the deliverable and it re-prices the emit-set-closure lane before anyone builds it.** **Registered deciding probes, now TWO and in this order: (1) INSTRUMENT `gl_function_attrs`' SKIP PATH** — count the positions it steps past and publish the count beside `plan-glattr-names`, because a reader whose coverage is unknown must say so rather than let its hit rate be read as a container fact (#3237); **(2)** the probe this row registered first and which is unchanged — read `+0x4c` **whole** (not its low byte, and not through this reader's framing) on a large TU and find the field that IS set per emitted function. |
| **W-ALIAS-1** | **adoption** | **The `.gl` tag-0x10 ALIAS record's grammar and its discriminator bit.** The tag dispatch routes `0x04`/`0x0E`/`0x10` to one shared kind-4 handler that splits only at the end; the `0x10` arm sets `[sym+0x37] \|= 0x400000` and stores **one `varU`** into `[sym+0x4c]`, at the same anchor a tag-0x0E record puts its `.ex` body offset. So on a tag-0x10 record that word is a **symbol token**, not a flag word — which is the whole finding, and it is a *bit layout*, so it is adoption and not navigation. | `0x10b9b91f` (dispatch), `0x10b9bdcf` (shared kind-4 header), **`0x10b9c01e`** (the tag test), **`0x10b9c024`** (`\| 0x400000`), **`0x10b9c030`** (the store), `0x10b9c033` (the shared tail) | `crates/c2-il/src/func/glalias.rs` — module docs, `ALIAS_TAG`, `record_head` | `d2bdadc` | Independently confirmed against real `c2.dll` by lane `w-emitp` (15/15 interventional draws, 0/15 parity control) and reproduced by two implementations agreeing on 850 TUs. The **grey-zone alternative was tried first and is insufficient**: a black-box search for the field position binds at 0.019/0.026 one byte either side, so the position is identified by the disassembly and only *graded* by the corpus. |
| **W-ALIAS-2** | **route** | **`+0x37 & 0x400000` has exactly two readers, and the emit-relevant one resolves the token and sets `+0x20 \|= 0x2000` on the TARGET.** This is what licenses the extensional claim the port's model uses — an initializer node naming an alias contributes the alias's *target* — and it is the reason `dom(alias)` is never itself emitted. | **`0x10b99621`** (`test [esi+0x37],0x400000`), **`0x10b99635`** (`or [eax+0x20],0x2000`), `0x10b8ac60` (the second reader, `or [eax+0x32],1` — read, modelled nowhere) | `crates/c2-il/src/func/glalias.rs` — module docs only; **no value or layout is copied from these sites** | `d2bdadc` | Logged as `route:` per the grey-zone rule: the reading told this lane what the record *means*, and the meaning was then established by black-box experiment (`w-emitp` §4, real `c2.dll`) and by corpus measurement (`dom(alias) ∩ E` = 0 over 174 417 emitted names). The instruction that turns `+0x20 & 0x2000` into the COFF Mark bit is **named (`0x10b28ca3`) and NOT decoded**. |
| **W-MEMCPY-1** | **route** | **The block-move expansion decision.** `align` = the front end's alignment hint; `n = size / align` **truncating**; `inline` iff `n <= T`, `T = 5` with favor-size and `10` with favor-speed; a non-constant size is a call; a zero size and a dead non-escaping local destination emit nothing. Written into [`../IL_INTRINSIC_CALL.md`](../IL_INTRINSIC_CALL.md) §5.1.1 and pointed at from one comment in `crates/`. **No constant, address, bit position or layout is in `crates/`** — the code's behaviour is unchanged and every intrinsic is still refused. | `0x10bf65b8`, `0x10bf65d1`, **`0x10bf65e3`** (`cmp eax,5`), **`0x10bf65de`** (`cmp eax,0xa`), `0x10bf65e6`, `0x10bf657f` / `0x10bf6584`, `0x10bf658b`, `0x10bf669d`; memset's copy at `0x10bf5e30`–`0x10bf5e46`. Named here for re-checking and **not decoded into any file** | `docs/IL_INTRINSIC_CALL.md` §5.1.1; `crates/c2-il/src/func/body/expr.rs` — **one comment, which points here and states no constant** | `cc14d018` | **Logged `route:`, and `WB_MEMCPY_FINDINGS.md` §9 pre-drafted it as `adoption`. The downgrade is earned, not asserted.** `work/w-memfit/holdout.py` fits **both** constants from obj cells alone — an exhaustive search over four candidate quantities × every threshold 0..2048 — and holds them out in both directions: fitted on GRID-W's 72 `/O1` cells the rule scores **232/232 and 176/176** on `w-memcpy`'s two grids, which it was never fitted to; fitted on those 408 it scores **72/72** on GRID-W `/O1` and refuses `/O2`, `/Ox`, `/O1 /Ot` at 18/36 each. **624 of 624 across the three grids.** What the disassembly supplied is the **search space** — `size / align, truncating` is a quantity nobody enumerated before reading it, and `w-memcpy` froze six rivals over 408 cells without one of them being a quotient. That is navigation, and the grey-zone rule says log it. Reciprocally, GRID-W has **0** cells that can see the truncation (its `n` axis is exact multiples) and `w-memcpy`'s have **22**, truncating 22 / ceiling 0 — so the oracle decides a part of the reading the whitebox lane's own grid could not |

| **W-GLATTRS-1** | **adoption** | **The `.gl` function record `SIZE` field's `0x80` escape is a LENGTH escape with a TWO-byte little-endian payload** — three bytes total — and `0x81..=0xff` is a separate one-byte sign-extended form, not part of it. What is adopted is a **field width**, `GL_SIZE_ESCAPE_PAYLOAD = 2`, so that the reader can step over `SIZE` and land on `ATTR`. **No threshold, no value and no semantics of `SIZE` is adopted**: board #3275 refused a rule keyed on the field's value, and nothing in `crates/` reads it. Also documented, and likewise not used: `ATTR` at `0x10c1f91b` is a two-or-four-byte value with a continuation flag in bit 15, of which the port reads the low byte. | **`0x10c1f9a6`** (`il-read-varint16` — `cmp dl,0x80` at `0x10c1f9ba`, the two payload byte reads at `0x10c1f9d8`/`0x10c1f9e0`, `movsx ax,dl` at `0x10c1f9bf`); `0x10c1f9e9` (`il-read-varint32`, the same shape at four bytes — the contrast that explains why `SRCPOS` escapes to 5 and `SIZE` to 3); `0x10b9bf67`/`0x10b9bf6c` (the call site and the only 16-bit store to `[sym+0x50]`); `0x10c1f91b` (the `ATTR` varU, documented only) | `crates/c2-il/src/func/gl.rs` — `GL_SIZE_ESCAPE_PAYLOAD` and the `SIZE` arm of `gl_function_attrs` | `9aed8eab1`-successor | **The width is over-determined and the whitebox source is the least of the three.** (a) A black-box twin grid, 18 cells over two profiles: sources differing only by `__declspec(noinline) ` versus 21 spaces, so byte-length-identical from one path; the first `.gl` byte to differ past the source hash is at the offset this width predicts and differs by exactly `0x40`, **18/18**, and the `ATTR` offset steps by **two** across the `SIZE 127 -> 139` boundary. (b) The workload's 28,739 direct-form records establish a ten-byte `ATTR` vocabulary independently; on the 99 escaped records this width scores **99/99** inside it against **3 / 0 / 1** for widths 1 / 2 / 5, at a 5.9 % background rate. (c) The disassembly. **Endianness is black box too**: the probe ladder steps `SIZE` by 12 per statement and runs 103 -> 127 -> **139** -> 163 -> 211 -> 259 -> 379 straight through the escape, where big-endian would read 35,584. The refused `0x81..=0xff` arm is refused on a **count** — zero witnesses in 1,461,374 workload records — and not on a reading |

| **W-STAGETAP-1** | **adoption** | **Seven call-site addresses and the load-slide anchor**, copied verbatim into the stage tap's site table so it can install call-site detours at c2's own per-function phase boundaries. What is adopted is a **set of addresses and the `e8 rel32` shape at each**, which is as adoption as it gets. Note what is NOT adopted: no algorithm, no constant that reaches an emitted byte, and nothing that any refusal predicate or emit path reads. The tap is a development instrument and the obj byte compare remains the sole judge. | **`0x10b7dc9f`** (`e8 de 86 06 00` → `0x10be6382`, scheduler run 1), **`0x10b7dcb7`** (→ `0x10b57633`, globregs), **`0x10b7dcde`** (→ `0x10be6382`, run 2), **`0x10b7dcf6`** (→ `0x10b31c9a`, the register allocator), **`0x10b7dd1d`** (→ `0x10be6382`, run 3), **`0x10b7e00c`** (→ `0x10be6382`, run 4 / mode 0), **`0x10be643e`** (→ `0x10be5d4b`, the region finder — sole call site); **`0x10bebffd`** (`_InvokeCompilerPass@12`, the slide anchor); read-only, cited in comments and not copied: `0x10c2e2fc` (the optimizer-on flag the scheduler sites are gated on), `0x10b7dc83`/`0x10b7dcc2`/`0x10b7dd01` (the three `cmp` sites) and — **added in the fix round, because the row omitted them and the omission was read as "the optimizer flag is the whole gate"** — the SECOND per-function gate at `0x10b7dc8b`/`0x10b7dcca`/`0x10b7dd09` (`test BYTE PTR [esi+0x1c],bl`), `sched0`'s optimizer gate at `0x10b7dfd9`, and its three further tests at `0x10b7dfe3`/`0x10b7dff2`/`0x10b7dff9` | `c2host/stagetap.c` — `g_sites[]` and `C2_INVOKE_VA` (adopted at **`2bfc70caf`**); `crates/c2-reference/src/stage.rs` — `STAGE_SITES` / `OPT_GATED_SITES` **as names only, no address** | this lane | **Every row is fail-closed at run time, which is the part worth carrying.** `tap_arm` refuses to patch a site unless the byte there is still `0xE8` *and* the decoded target equals the recorded target **plus the measured slide**; a refusal prints and writes nothing. So a different `c2.dll`, or a relocated image handled wrongly, lands on the check rather than on a patched guess — and the check fired on this lane's first armed run, when the plan's `HMODULE`-as-base assumption produced `slide=ef500018`. The image is pinned: sha256 `c80981c0…a66258`, the one the whole whitebox record is written against |
| **W-STAGETAP-2** | **adoption** | **The tuple record's first eleven bytes**: `+0x0` next, `+0x4` opcode, `+0x8` category byte, `+0x9` flags, `+0xa` condition code (`& 0x1f`). A **bit/field layout**, therefore adoption. The first three are read from the region finder's OWN code on the path the tap sits on; `+0x9` and `+0xa` come from `WB_DAGORDER_FINDINGS.md` §2 and are **not confirmed by anything this tap reads**. | `0x10be5d5c` / `0x10be5d92` (`mov ecx,[ecx]` / `mov esi,[esi]` — next), **`0x10be5d55`** (`cmp DWORD PTR [ecx+0x4],ebx` with `ebx = 0x30f` — opcode), **`0x10be5d6b`** (`movzx edi,BYTE PTR [esi+0x8]` — category), `0x10be5d66` (`cmp edx,0x50` — the region bound); `+0x9`/`+0xa` via `WB_DAGORDER_FINDINGS.md` §2, address not re-derived here | `c2host/stagetap.c` — `tap_walk_tuples` (adopted at **`a09f33704`**) | this lane | **Partly obj-corroborated, and the corroboration is a different artifact each time.** The per-function site counts equal c2's own `/FAsc` `PROC` count and the obj's `.text` COMDAT count (3 paths, `il_call_perm.cpp`: 7/7/7). The opcode values the walk reads include `0x30f` at category `0x17`, which is exactly the terminator pair `0x10be5d8b` tests — the layout predicts a value the code independently branches on. **What is NOT corroborated is `+0x9`/`+0xa`**, and a measurement bounds their usefulness rather than their correctness: over 83 tuple pairs and a 128-byte raw window, **the register allocator writes nothing in the tuple record at all**, so no field in this window localizes COLOR |
| **W-STAGETAP-3** | **adoption** | **One more call-site address — the observation point AFTER the final schedule.** `0x10b7e701` in the per-function orchestrator `0x10b7e6af` is the first call after `0x10b7df57` (run 4, mode 0) returns, with `ecx` still holding the function record. Adopted for the same reason as W-STAGETAP-1's seven: a site address plus the `e8 rel32` shape at it. `docs/ARCH_REVIEW_2026-08-21.md` finding 1 is what makes it necessary — the region tap fires at region-finder ENTRY and run 4 has no successor run, so every `sched0` block is run 4's INPUT and the run that fixes emitted instruction order had its output observed nowhere. | **`0x10b7e701`** (`e8 2c f9 ff ff` → `0x10b7e032`); read-only context, cited and not copied: `0x10b7e6b0` (`mov esi,ecx` — the function record), `0x10b7e6fa` (`call 0x10b7df57`, run 4), `0x10b7e6ff` (`mov ecx,esi`), and the containment `0x10b7df57 + 219 = 0x10b7e032` that puts `sched0`'s site `0x10b7e00c` inside run 4 | `c2host/stagetap.c` — `g_sites[]`'s eighth row; `crates/c2-reference/src/stage.rs` — `STAGE_SITES` **as the name `after0` only, no address** | `w-restim` | **Fail-closed like the other seven, and obj-corroborated on its first run**: `after0` fires **7** times on `il_call_perm.cpp`, equal to `sched1` and to c2's own `/FAsc` `PROC` count and the obj's `.text` COMDAT count. The standing per-site equality test (`the_snapshot_is_nonempty_and_agrees_with_a_second_derivation`) covers it with no change |
| **W-STAGETAP-4** | **adoption** | **The operand record and the symbol/candidate record it points to** — `tuple+0x28`/`+0x2c` operand list heads; `op+0x0` next, `op+0x8` kind, `op+0xa` type word (class nibble `>> 12`), `op+0x1c` symbol; `sym+0x4` kind, `sym+0x1c` id, `sym+0x10 -> +0x1c` assigned register, `sym+0x8 -> +0x1c` physical register, in the `n = r+1` encoding. A field layout, therefore adoption. **Two guards are adopted with it and are load-bearing**: operands are walked only when the tuple's `+0x9` bit 0 (real-instruction) is set, and `op+0x1c` is followed only when `op+0x8 == 1` — both are c2's own guards, and running without them produced operand kind `0xf8` and symbol kind `0x88` out of label tuples, plus a c2 static data address (`0x10c2f268`) in what is supposed to be an address-free canonical stream | `0x10b2ceb7` (`piVar13[10]` / `piVar13[0xb]` list heads; `piVar18[7]` symbol; `cVar1 = *(char *)(iVar9 + 4)` symbol kind; kind-2 `*(uint *)(iVar9 + 0x1c)` fed to the candidate lookup `0x10b2c21d`; kind-1 `*(uint *)(*(int *)(iVar9 + 8) + 0x1c)`; the real-instruction guard `*(byte *)((int)piVar13 + 9) & 1`), `0x10bfebf7` (`puVar1[0xb]`, the `*(char *)(puVar3 + 2) == '\x01'` operand-kind guard, and the `0x0f..0x20` register bound that fixes the `r+1` encoding), `0x10b31ac9` (`*(uint *)(*(int *)(param_3 + 0x10) + 0x1c)` — the assigned-register descriptor), `0x10b022cc` (operand-nibble → register class, cited not copied) | `c2host/stagetap.c` — `tap_walk_operands`, `ap_regfield` | `w-restim` | **Adopted to answer `docs/rungs/2026-08-20-stageoracle.md` §6.1 q1, and the answer it produced REFUTES the lane's own prediction A1** (registered 0.85): with all of it read at both phases, the pre/post-COLOR pair is still IDENTICAL. So this layout's value here is a NEGATIVE result about where COLOR's output is not, which is what §6.1 asked for. `P_REGALLOC.md` §4.1's `+0x1c` is the candidate ID, not a register — the register is one further hop through `+0x10` |
| **W-STAGETAP-5** | **adoption** | **The function record's route to its tuples, and the block record**: `func+0x8` → header, `header+0x4` → first block, `block+0x4` → next block, `block+0x20` → the block's LAST tuple, `block+0x1c` → the walk's stop value, `tuple+0x10` → prev. Field layout, therefore adoption. This is `w-stageoracle`'s **P10** — the function-record → tuple-list-head offset it registered at 0.20 and reported NOT NEEDED — answered in the affirmative, and it is what makes an observation at `after0` possible at all (there is no region tap after run 4). | `0x10b2ceb7`: `iVar14 = *(int *)(*(int *)(param_1 + 8) + 4)` (function → header → first block), `piVar13 = *(int **)(iVar14 + 0x20)` (block → last tuple), `while (piVar13 != *(int **)(iVar14 + 0x1c))` (the stop value), `piVar13 = (int *)piVar17[4]` (prev, i.e. `tuple+0x10`), `iVar14 = *(int *)(iVar14 + 4)` (block → next block); the `+0x0` next / `+0x10` prev pairing is `0x10be626c`'s, via `P_DAG.md` §2. The allocator driver `0x10b31c9a` passes the function record straight into `0x10b2ceb7`, which is what identifies `param_1` | `c2host/stagetap.c` — `tap_walk_function` | `w-restim` | **CROSS-DERIVED against the region walk, and the first version of that cross-derivation was VACUOUS.** A multiset containment passed 14 of 14 and would have passed on a reversed or re-blocked walk; replaced by an ORDER-SENSITIVE check, the region walk's opening rows are the in-order TAIL of a funcwalk block on 14 of 14 (`stage snap`'s `FW-XDERIV` rows) — the within-block direction is confirmed. **And the same check measured the reading's limit**: the region walk continues past the block end into tuples `block+0x4` orders EARLIER, so `block+0x4` is a TRAVERSAL order and NOT the tuple list's order. Every comparison built on it is the same traversal at two phases, never a claim about emitted order |
| **W-MID-1** | **adoption** | **The machine mnemonic table `0x10b1b260`, stride 12, `{char *name, u32 form, u32 flags}`, indexed 0-based by the tuple opcode, with `_last` at index `0x295`** — so the machine opcode space is `0x001..0x294`. What is adopted is a **table address, its stride and its index origin**, plus the `_last` sentinel index. **No table entry is copied**: `crates/c2-reference/tests/middle_interfaces.rs` reads the strings out of the pinned `c2.dll` at run time and refuses if four spot cells disagree. Also documented and NOT adopted anywhere: the SECOND table at `0x10b1d180`, stride 16, `{name, real opcode, BO, BI}` — the extended-mnemonic table, which has its OWN index space starting at 1 | **`0x10b1b260`** (the table); **`0x10c00900`**–`0x10c00952` (the inline-asm name lookup that fixes stride 12 via `imul eax,eax,0xc` and terminates on the `_last` string `0x10b19ce4`); `0x10b1b264` (the `+4` form field, read at `0x10c029dd`/`0x10c032b8`); **`0x10b1d180`** and `0x10c0174b`–`0x10c01790` (the second table and its `shl eax,4` walker, documented in `WB_MIDDLE_INTERFACES.md` §2.2, copied nowhere) | `crates/c2-reference/tests/middle_interfaces.rs` — `MNEMONIC_TABLE`, `MNEMONIC_STRIDE`, `MNEMONIC_LAST`; `docs/whitebox/scripts/dump_opcode_tables.py` | this lane | **Test-only, and the reason it is a row anyway is that a table address is adoption whatever reads it.** Independently corroborated three ways: `P_DAG.md` §2.1 reached `addi`=11 / `lis`=625 / `blr`=645 from a different starting point and this table agrees 0-based on all of them; the alphabetical PPC ordering `add, add., addo, addo., addc, …, addi` falls out of indices 1..11 with no gaps; and every base word in W-MID-2 lands on the correct architectural encoding for the mnemonic at the same index, which a wrong origin could not do. **The second table is a TRAP and is named as one**: read as a continuation of the first, tuple opcode `0x30f` decodes as `tdlngi` in a function that is `return a+b+c` |
| **W-MID-2** | **adoption** | **The PPC base-encoding table `0x10c3a578`, stride 4, indexed by machine opcode** — one 32-bit instruction word per opcode with every operand field zero — **and the encode-form table `0x10c39b18`, stride 4**, whose value minus one is the index into the encoder's 111-arm jump table at `0x10bfae2d`. Table addresses and strides; **no entry is copied**, both are read out of the pinned image at run time. Two form VALUES are adopted as constants — `0x31` (three-register) and `0x37` (`ret`/`blr`) — so the test can assert which arm it is encoding for instead of silently taking another | **`0x10bf9f3c`** (`mov ebx,[edx*4+0x10c3a578]` — the SOLE reader of the base table in the image), **`0x10bf9f43`** (`mov edx,[edx*4+0x10c39b18]`), `0x10bf9f4d`/`0x10bf9f51`/**`0x10bf9f57`** (`dec` / `cmp edx,0x6e` / `jmp [edx*4+0x10bfae2d]`), `0x10bfae19` and `0x10bfae1b` (the two OR-onto-base tails); entry point `0x10bf9f15`, in `code.c` | `crates/c2-reference/tests/middle_interfaces.rs` — `BASE_WORD_TABLE`, `FORM_TABLE`, `FORM_XO_RT_RA_RB`, `FORM_RET` | this lane | **Obj-confirmed, 9 words, 32 bits of 32** (`the_final_tuple_order_reproduces_the_text_words`, on `mvp_add3` and `mvp_two`, both of which the port already emits byte-exact). Spot-checked against the architecture on 12 further opcodes with no cell wrong. **The grey-zone alternative does not exist**: a black-box experiment can show that `add` encodes to `0x7C000214`, which the PowerPC manual already says — what the disassembly supplies is that c2 keeps these words in ONE array indexed by the same number the tuple carries, which is the fact that makes the seam legible, and no obj can exhibit an array |
| **W-MID-3** | **adoption** | **The instruction encoder's two operand arms.** Form `0x31`: `RA` from `tuple+0x28`, `RT` from `tuple+0x2c`, `RB` from `[tuple+0x28]`, each via `operand+0x1c` then `+0x28`, composed `((RT << 5 \| RA) << 5 \| RB) << 11` — i.e. bits 21/16/11. Form `0x37`: **no operand is read at all**; the word is the base OR `0x02800000`, the `BO` field 20. A field/bit layout, therefore adoption. **The reconciliation with `W-STAGETAP-4` is adopted with it**: `sym+0x08 -> +0x1c` (the physical register, `n = r+1`) equals `operand+0x1c -> +0x28` plus one, which is what lets this lane's check run on `w-restim`'s tap without a second walk | **`0x10bfa456`**–`0x10bfa473` (the three-register arm: `mov edx,[esi+0x1c]` / `mov eax,[eax+0x1c]` / the two `+0x28` loads / `shl 5` / `shl 5` / `shl 0xb`), **`0x10bfa2a5`**–`0x10bfa2ab` (the `ret`/`blr` arm: `or ebx,0x2800000`), `0x10bf9f26`/`0x10bf9f2c`/`0x10bf9f33` (the three operand pointers) | `crates/c2-reference/tests/middle_interfaces.rs` — `encode` | this lane | **Obj-confirmed on the same 9 words.** The `n = r+1` reconciliation is confirmed by construction: `w-restim`'s field reads `0x0c` where the obj's `add` has `r11`, and `0x0c - 1 = 11`. **What is NOT adopted, stated so absence does not read as coverage: 2 of the 111 arms are read, and the relocation/label half of the emit seam is read NOT AT ALL** |
| **W-MID-4** | **route** | **The `.ex` `0x4F` sub-record is read off an 8-byte-stride descriptor table at `0x10b26268` and then a ~14-arm switch.** Named so a future lane knows where the record's widths live; **nothing is decoded and no width is taken from it.** The one width this lane uses — `4F 01 <byte>` is three bytes — is TRANSCRIBED from the corpus and labelled as such in the code, and every other `0x4F` sub-opcode refuses | `0x10b9761e` (`FUN_10b9761e`), **`0x10b97641`** (`mov eax,[eax*8+0x10b26268]`), `0x10b9763d` (`movsx eax,BYTE PTR [esi+0x24]` — the sub-opcode is the `i16c` the class-`0x0C` arm stored), `0x10b9766c` onward (the switch) | *(nothing — no value or width is copied)* | this lane | Logged `route:` per the grey-zone rule. The reading told this lane that the `0x4F` widths are table-driven and therefore out of a minimum-subset lane's reach; the width actually used is a corpus transcription, and the decoder REFUSES every sub-opcode it has not seen rather than guessing from the table it did not decode |

> ### **2026-08-23 — `W-MID-4`'s ADDRESSES ARE ALL CORRECT AND ITS CLAIM TEXT IS WRONG (read R9, board `#3442`).**
>
> The row says the table was *"named so a future lane knows **where the
> record's widths live**"*. **No width lives there, or anywhere else.**
> `0x10b26268` is a 64-entry, stride-8 table whose first dword is a
> `const char *` to a NUL-terminated string of **field-type codes**;
> `FUN_10b9761e` is a format-string interpreter. A record's width is a **sum
> over that string**, and three of the codes are data-dependent. The row's
> four addresses (`0x10b9761e`, `0x10b97641`, `0x10b9763d`, `0x10b9766c`) are
> each exactly right, and the `~14-arm` count is right — over **field types**,
> not sub-opcodes.
>
> **The row's disposition does not change and no new row is owed.** It stays
> `route:`, this lane adopted nothing, and the width the port uses is still a
> corpus transcription — **now known to be wrong above source line 127**
> (`#3443`, reported for a follow-up lane, not fixed under a docs-only fence).
> Spec: [`ref/P_SUB4F.md`](ref/P_SUB4F.md). Grade:
> [`WB_SUB4F_FINDINGS.md`](WB_SUB4F_FINDINGS.md).
>
> > **AMENDED BESIDE, 2026-08-23 — a row IS owed now, and it is `W-SUB4F-2`
> > below.** The paragraph above is correct as written and is left as written:
> > when `w-read-r9` wrote it, that lane had adopted nothing. The follow-up
> > lane it names (`w-4f01`) has since **adopted the width rule into
> > `crates/`**, so the debt this row declined transfers to `W-SUB4F-2`. This
> > row stays `route:` — its own four addresses are still adopted nowhere.
> > **The sentence *"the width the port uses is still a corpus transcription"*
> > is no longer true of the tree**, and that is the only clause superseded.

| **W-SEEDGAP-1** | **route** | **`LABEL_SEED_GAP`'s three coefficients — `7 + 2·[/Og] + 1·[/GF ∧ a string literal pooled in the data phase]` — and they are BLACK BOX, which is why this row exists to say so rather than to disclose anything.** The lane that adopted them (`w-seedgap`, board `#3402`–`#3405`) was dispatched with an instruction to file a disclosure row for "the formula's terms" as read facts. **That premise is wrong and the row corrects it**: every one of the three numbers comes from `scripts/gt_label_seedgap.py`'s 22-cell obj grid, which compiles real `cl.exe` under wibo, reads the seed straight out of the captured `.gl` as `u32_le(.gl[7..11])` and subtracts it from the obj's first `$M`/`$T`. No address, value, bit position, field width or table offset is copied into `crates/` by this lane. This is precisely the case the file's own closing rule §5 asks for — *"if the same fact can be established by a black-box experiment against the real toolchain, run it and adopt that instead"* — and it already had been, by lane `w-read-r3`, before any code changed. | *(none adopted)* — read-only, cited in comments and **not decoded**: **`0x10b97de5`** (the sole increment of the TU-global label counter `DAT_10c2edd0`, named in [`ref/P_LABEL.md`](ref/P_LABEL.md) §4 as the live tap that WOULD attribute each unit of the 7/9/10 and which no lane has built), and §3.1's reserved low-id region `{0x0d,0x0f,0x16,0x17,0x19,0x1a,0x1b}` at `0x10be78a8`/`0x10be794d`/`0x10be79fa`/`0x10c1252c`, which EXPLAINS why section needs charge nothing but is not what establishes it — the eleven zero-mover cells are | `crates/c2-core/src/coff/label.rs` — `SeedGapModel::READ`, `SeedGapInputs`, `global_optimizer_of_opt_word`; `crates/c2-il/src/func/bundle.rs` — `OPT_WORD_OD`, `OPT_WORD_PRAGMA_OFF` | `w-seedgap` | **The two `OPT_WORD_*` constants added alongside are IL captures, not disassembly**: `0x00800005` and `0x00800004` have been in [`../OPT_MODE.md`](../OPT_MODE.md)'s matrix and in `OPT_WORD_OX`'s own doc since the word table was read off `.ex` bytes, and naming them admits nothing — `opt_word_mode` is untouched and still refuses both. **What the whitebox record DID supply here is a bound, and it is a negative one**: `P_LABEL.md` §4 closes the *candidate site population* to five once-per-TU allocations and shows the gap is mode-dependent, which refutes "nine fixed allocations" — so `SeedGapModel` is knowingly a **fit to a read grid, one level short of the mechanism**, and its doc says exactly that rather than presenting the arithmetic as understood |
| **W-SUB4F-2** | **adoption** | **`4F 01`'s payload is a VI32 varint, so the source-line record is 3 bytes below line 128 and 7 at or above it.** One byte when the value is `< 0x80`; otherwise the escape byte `0x80` followed by four little-endian bytes. Sub-opcode `0x01`'s descriptor in the 64-entry, stride-8 table at `0x10b26268` points at the one-character format string `6c` (`'l'`), and `FUN_10b9761e`'s arm for that code reads the field through the VI32 reader under gate `DAT_10c2eb4c`. **What is adopted is the WIDTH RULE and the escape byte, nothing else** — no table offset, no descriptor layout, no gate address is copied into `crates/`; `read_line_record` is eleven lines of `if payload == 0x80`. **This row exists because `W-SUB4F-1` above says no row was owed, and that was true when it was written**: `w-read-r9` reported the defect under a docs-only fence and adopted nothing. This lane adopts, so the debt transfers here. **The black-box alternative was priced first and it does NOT replace the read** (§5's rule): the twin grid in `docs/whitebox/scripts/sub4f_probe.py --grid` establishes the *boundary* empirically at 10/10 cells — and it was re-run in this lane before a byte was edited, with the `#line 127` cell carrying both widths in one file — but a grid over ten `#line` values cannot establish that the escape byte is `0x80` rather than any other sentinel, nor that the escaped form is exactly four LE bytes rather than a longer form that happens to fit. The read supplies the *rule*; the grid supplies the *confirmation*, and both are cited. | **`0x10c1f9e9`** (the VI32 reader — the one address the rule rests on). Read-only context, cited in comments and **not decoded into `crates/`**: `0x10b26268` (the format-pointer table), `FUN_10b9761e` (the interpreter), `0x10b9780e` (code `0x6c`'s arm) and its gate `DAT_10c2eb4c`. | `crates/c2-il/src/codec.rs` — `read_line_record` / `encode_line_record`, and the `ExToken::Stmt` / `BlockStart` / `ModuleEnd` payloads; `crates/c2-reference/tests/middle_interfaces.rs` — `decode_body_to_tuples`'s `0x4F` arm | (this commit) | **LATENT, NOT LIVE, AND THAT IS THE POINT.** Every fixture in `fixtures/cpp` sat below source line 128, where a fixed-byte read and a VI32 read consume the same three bytes and yield the same value — so the wrong width was green on the entire corpus and **no gate had ever seen it**. **Two things the read bought that the grid did not.** (a) A *third* wrong site R9's defect table does not name, and a fourth outside every line range it cited: the `4F 01` nested inside the block-start and module-end markers. `4F 02 20 00` is exactly a 4-byte record (`P_SUB4F.md` §4: sub `0x02`, format `73`, VARU), so the `4F 01` after it is a separate record — the module-end decoder looked for its trailing `4D` at a fixed `p+7` and **refused the whole record** whenever the line was wide. (b) A semantic correction that came free: those payloads are the function's **first and last source lines**, not the "statement/block index" the doc comments called them, shown by a capture at `#line 258` putting 258 and 263 there. Every fixture is a one-line function below line 128, where the two readings are the same byte. **Registered narrowing:** true VI32 sign-extends a lead byte in `0x81..=0xFF` to a negative one-byte value; the adopted reader **refuses** that byte instead, matching the fail-closed idiom already in `func::readers::read_varint` and `func::ehscope::opt_line`. Unreachable for a line number, deliberate, and asserted by a test so it is not later mistaken for a bug. Spec: [`ref/P_SUB4F.md`](ref/P_SUB4F.md) §4. Grade: [`WB_SUB4F_FINDINGS.md`](WB_SUB4F_FINDINGS.md) §5. Board **#3443**, **#3452**–**#3455**; rung [`../rungs/2026-08-23-w-4f01.md`](../rungs/2026-08-23-w-4f01.md) |

> ### **2026-08-09 — `WB_MEMCPY_FINDINGS.md` §9's other three pre-drafted rows are NOT carried, and each has a reason.**
>
> * **W-MEMCPY-2** (`0x10c2e310` is bit 23 of the option word) — **not carried.**
>   The port needs the *behaviour* (the threshold follows favor-speed, not the
>   `/O<n>` level) and that is what GRID-W measures, at 180/180 across five flag
>   sets. Nothing anywhere reads an option-word layout. A row would disclose a
>   bit position the project does not use.
> * **W-MEMCPY-3** (the callee name is minted inside c2 from a string literal) —
>   **not carried, because it was re-derived black box in this lane and needs no
>   route.** A TU whose only call is `memcpy` has `?f@@…`, `.XBLD$W`,
>   `__C1_11886` and the `/include:` directive in its `.gl` and **no `memcpy`**,
>   while its obj carries `[14] memcpy sc=EXTERNAL sec=0 type=0x0020`. That is
>   two observations of the black box's own output, and `w-memcpy` §2 had already
>   made the first of them before any disassembly existed.
> * **W-MEMCPY-4** (the removal site) — **not carried, by that document's own
>   instruction.** The rule adopted is `E-DEADDST`, obj-established at 36/36 in
>   GRID-W and 44/44 in GRID-M2, and it needs no address at all. `0x10b482ba`
>   stays `unknown`.

> ### **2026-08-09 — the `W-SELECT-*` rows are PRE-DRAFTED IN TWO PLACES WITH DIFFERENT CONTENTS, and lane `wb-selfit` reconciled them. NOTHING IS CARRIED.**
>
> `WB_SELECT_FINDINGS.md` §10 and `WB_SELECT_FINDINGS_R2.md` §9 each pre-draft
> five rows under the same five names, from two independent readings of one
> image on one day. **Ten drafts, five names, no adopted row** — no lane in that
> family has changed `crates/`, so none of them belongs in the table above yet.
> [`WB_SELECT_RECONCILED.md`](WB_SELECT_RECONCILED.md) §14.2 merges them to six;
> the operative points for whoever carries them:
>
> * **`W-SELECT-2` (the operator × type tables) — use `WB_SELECT_FINDINGS_R2.md`'s
>   version.** The other lane's enumeration is missing the thirteenth table,
>   `convert` @ `0x10b1fd08` (board **#2200**). **The black-box alternative is
>   complete and should be preferred**: the two grids plus `diag.cpp` re-derive
>   every live entry, the signedness split, `srawi`+`addze`, the `lha` fusion and
>   the absence of a magic-number multiply **with no address**.
> * **`W-SELECT-3` (the cost model and the tie rule) is the row that genuinely
>   needs an address, and the case is now STRONGER than either lane made it.**
>   Both wrote that no obj separates *"`cntlzw` was cheaper"* from *"ties go to
>   `cntlzw`"*. Board **#2204** adds that no obj in this project ever reached the
>   comparison: `FUN_10c1b517` routes an against-zero relational to
>   `FUN_10c1a908` first, and **five of the two grids' 24 cells** are exactly
>   that. Use `WB_SELECT_FINDINGS.md`'s relation-code table — the other lane's
>   has two transposed pairs (**#2207**), so the canonical form is `UGT`.
> * **A SECOND row needs an address, and it is a COUNT.** 13 tables, 41 dispatch
>   arms, 18 expansion arms. `WB_SELECT_FINDINGS_R2.md`'s `W-SELECT-4` note said
>   so first and it is upheld: **no obj yields a count of arms**, and those three
>   numbers are what both judgment rows' prices rest on. A port that only
>   *implements* the rules needs none of them.
> * **`W-SELECT-5` — RELEASED, by `wb-tables`, and this note defers to it.**
>   `wb-selfit` reached the clause *"`&` with a contiguous mask is `rlwinm`,
>   never `andi.`"* is over-general and the deciding routine is
>   **`FUN_10c0a2e2`** not `FUN_10c1772b` (**#2210**, **#2203**), and stopped
>   there with the predicate open. **`wb-tables` closed it** —
>   `WB_TABLES_FINDINGS.md` §4.2, rules (S) and (B) obj-confirmed on 32 cells —
>   so the expansion is **black-box re-derivable from `grids/wb-tables/` and a
>   code lane shipping it needs no row** (**#2119**). Carry it only if
>   `FUN_10c0a170`'s word prices or `FUN_10c1772b`'s tie to the relaxed mask are
>   copied; neither is visible in any obj.
> * **One row neither WB-I lane proposed**: `FUN_10c1a908` @ `0x10c1a908`, the
>   against-zero relational, ~20 arms, **unread by all three lanes** and the
>   thing five already-graded cells actually exercised (**#2204**). Navigation,
>   held — and for an integer `lower_expr` it is a **larger** gap than
>   `FUN_10c194b8`, which is the floating-point path and not the `{0,1}` path
>   two documents call it (**#2205**).

> ### **2026-08-08 — lane `w-phase7` gave W-ALIAS-1 and W-ALIAS-2 their first CONSUMER, and adopted NO new address doing it.**
>
> The `Adopted into` column of **W-ALIAS-1** should now be read as
> `crates/c2-il/src/func/glalias.rs` **plus** `IlBundle::data_tu`'s alias
> fence and `IlBundle::in_alias_report`, and **W-ALIAS-2**'s as unchanged
> (module docs only). No constant, offset, bit position or layout beyond the
> two rows above entered `crates/` in that lane:
>
> * `ObjImage::weak_externals` and `ObjImage::relocs_named` are **PE/COFF
>   format** readers — `IMAGE_SYM_CLASS_WEAK_EXTERNAL`, the weak aux record's
>   `TagIndex`/`Characteristics`, the relocation table — all published format,
>   none of it derived from `c2.dll`. **No white-box debt.**
> * The realisation rule *"c2 writes `??_E<X> → ??_G<X>` iff `??_G<X>` is a
>   `.text` COMDAT leader of the same obj"* is **extensional**, derived from
>   878 objs and graded per record (4,013/4,013, 0 miss, 0 extra). It is a
>   statement about c2's **output**, which is the black box's own observable.
>
> **And W-ALIAS-2's `route:` claim is now confirmed harder than it was.** That
> row's stated meaning — *"an initializer node naming an alias contributes the
> alias's target"* — was licensed by `w-emitp`'s 15/15 interventional draws.
> The weak-external reading is a second, independent confirmation **from the
> obj alone**, needing no mutation and no disassembly: the pairing `??_E<X> →
> ??_G<X>` is written into the symbol table where anybody can read it. A
> `route:` row whose meaning is independently visible in the output is the
> cheapest kind of white-box debt there is.
>
> **What is still NOT adopted, and what the next lane would need.** A Rust
> emit-set model needs the `.gl` **reference-list** decode
> (`work/w-refs/refs.py`), which carries `0x10b9bf99` (the list, gated on
> `flags4c & 0x1000`), `0x10b276e4` (the Mark walk) and `0x10b9be44` (the
> storage-class-`0xa` skip). **None of those three is in this ledger and none
> is in `crates/`.** `w-phase7` declined the port rather than adopt them
> silently — see `rungs/2026-08-08-w-phase7.md` §7.2, whose first named step is
> a row here.

**These are the first two rows, and `README.md` changed in the same branch** —
its clean-room claim now reads per-finding and points here, exactly as step 4 of
the checklist below requires. Everything else the `w-map` lane produced remains
navigation, not adoption.

**What is NOT adopted, stated so absence does not read as coverage.** The four
`.gl` scalar encodings the record walk needs (`0x10c1f8fc`, `0x10c1f91b`,
`0x10c1f9a6`, `0x10c1f9e9`, `0x10c1fae7`, `0x10c1f90a`, `0x10c1fcef`) are named
in comments as *navigation*: the same encodings were already re-derived from
black-box IL in `crates/c2-il/src/func/readers.rs` before any disassembly was
read, and the copies in `glalias.rs` exist only because the walk needs them at
`.gl` positions. No row is claimed for them, and if a future reader disagrees
with that call the fix is to add a row, not to remove the comment.

## If you are about to add the first row

1. Add the row *before* or *with* the code change, never after.
2. Name the address, not just the function — a future reader must be able to
   re-check your reading.
3. Say in the code comment that the value is disclosed, and point at this file.
4. Tell the coordinator: `README.md`'s wording must change from a blanket claim
   to a per-finding one at the same time. That is a one-line edit and it must not
   lag the code.
5. Prefer the alternative first: if the same fact can be established by a
   black-box experiment against the real toolchain, run it and adopt *that*
   instead. The oracle is cheap; the clean-room claim is not.
