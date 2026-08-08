# w-cfg2 — PREREGISTRATION

Lane `w-cfg2`, worktree branch `wt-w-cfg2` off master **`5d101061`**.
Committed **before the first line under `crates/`** and before the first probe
that is part of the *build*. §0 declares, rather than hides, what the
target-selection survey already measured.

Baseline at `5d101061`, re-derived by this lane's own scan
(`work/w-cfg2/scan_base.out`): **match 12** · mismatch 0 · codegen-gap 0 ·
vocab-gap 859 · capture-fail 7 · **FRONTIER 15** · `factor-c` 169 ·
`a-and-b-and-c` 27 · `frontier-if-a` 137.

---

## §0 What was already measured, before this file existed — declared, not hidden

The brief instructs the lane to re-survey the frontier at its own base, so the
**target-selection survey ran first** and is not covered by the predictions
below. It consisted of exactly:

1. one full 878-TU `c2rs gap` scan at `5d101061` (`work/w-cfg2/scan_base.out`);
2. `c2rs compile` + `scripts/gt_dump.py` on the eight smallest frontier TUs by
   `.text` (`Main`, `Primes`, `undname`, `osfinfo`, `xlrcimpl`, `vswprnc`,
   `Biquad`, `jsonwriter`) — `work/w-cfg2/ref/*/dis.txt`, committed;
3. `c2rs census` on all fifteen frontier TUs, for the per-function blocker key,
   the CFG class and **the IL body size in bytes**;
4. `c2rs capture --keep-il` on `src/system/math/Primes.cpp` (the IL itself is
   never committed);
5. reading `crates/c2-core/src/codegen/frame.rs`, `coff/writer.rs`,
   `coff/data.rs`, `lib.rs::data_refs_of` to establish which of the survey's
   markers the port **already builds**.

Nothing in §2–§4 is scored against a number that came out of that survey.

### §0.1 The survey, by what a lowering must BUILD

Not by `.text` size — that is the brief's instruction and w-cfgclass's method.
`.text` and the IL body size are printed anyway because they bound the
transcription; the deciding column is the last one.

| TU | `.text` | fns blocked | IL body (B) | what a lowering must BUILD that the port has not built |
|---|---:|---:|---:|---|
| `Main.cpp` | 124 | 1 | **222** | **the whole EH record**: `eh-state1`, two ADDR32 words *inside* `.text` (`__CxxFrameHandler`, `__ehfuncinfo$main`), an unwind funclet at `0x54`, **two** `.pdata`, a 64-byte `.rdata` with 5 relocations (`__unwindtable$main`, `$T2592`, `$M2590`, `$M2591`, `__unwind$2585`) |
| **`Primes.cpp`** | **64** | **1** | **294** | **`.data` composed with `.text`**; a **1:2** REFHI/REFLO fan-out against a **STATIC** symbol. *Nothing else*: no frame, no `.pdata`, no `$M`/`$T`, no callee-saved GPR, no `_fltused`, and **all sixteen words are already reproduced from this crate's own encoders and asserted against real `c2`** by `codegen::frontier_bytes` (w-loop, `cfg(test)`) |
| `undname.cpp` | 140 | 1 | 532 | **two** data symbols in one body (`IlFunction::data_sym` is one `Option`), each with its `lis` **not** the body's first word (`data_refs_of` requires it); `stb` |
| `osfinfo.cpp` | 152 | 1 | 445 | two data symbols, `lis` not first; `srawi`, `mulli`, `lbz`, `add`, record-form `clrlwi.`, `cmplw`; a 6-block plan |
| `vswprnc.cpp` | 156 | 1 | 508 | a REFHI/REFLO against a **code** symbol (`_woutput_s_l`, Type 0x0020) used as an address, `lis` not first; a 5-deep argument rotate interleaved with it; `sth`; cr0-form `cmpwi` |
| `Biquad.cpp` | 176 | **2** | 838 + 162 | **two** plans (out of the brief's scope); FP indirect load/store through pointer formals, FP control flow, a pooled-constant `lis` **mid-body** (`coff::function` hoists it to the top) |
| `xlrcimpl.cpp` | 152 | 1 | 519 | `__savegprlr_26`/`__restgprlr_26` — `FrameLayout::needs_gpr_helper` refuses at ≥ 3 saved GPRs; an address-taken local; `lis`/`ori` 32-bit constants |
| `jsonwriter.cpp` | 304 | 1 | 1349 | `cflow-loop`, `rlwimi`, `sthu`, 76 words |
| `wordwrap`, `IPP_basicmath_xbox`, `EncryptXTEA`, `Pool`, `mmio`, `keygen_xbox` | — | 3 / 4 / 4 / 3 / 3 / 18 | — | **several plans each** — out of the brief's "ONE block plan" scope by construction |

**Two frontier bodies are smaller than `Primes`' 294 B** — `wordwrap`[0] at 97 B
and `IPP_basicmath_xbox`[1] at 230 B — and both sit in TUs with three and four
blocked functions. **`Main.cpp`'s 222 B is smaller and `Main.cpp` is the most
expensive row in the table**, which is the whole argument for the "must BUILD"
column: the two size axes disagree with the cost at the head of the ranking.

## §1 The target

**`src/system/math/Primes.cpp`** — `?NextHashPrime@@YAHH@Z`, a FRONTIER TU:
1 emitted function, 1 blocked, `expr-jump`, `cflow-loop`, `eh-none`, 294 B of IL,
64 B of `.text` (16 words), 248 B of `.data`, 6 relocation records,
**label-free**, **no `.pdata`**, **no frame**.

The brief requires this row to be **re-measured before re-attempting**, because
`w-loop` declined it on 2026-08-08. Re-measured at `5d101061`, all of it:

* the obj is byte-for-byte the shape w-loop recorded — 6 sections, `.text` 64 B /
  6 relocations, `.data` 248 B / 0 relocations, symbol
  `?primes@?1??NextHashPrime@@YAHH@Z@4PAHA` STATIC in section 6, no `.pdata`, no
  `$M`/`$T` (`work/w-cfg2/ref/Primes/dis.txt`);
* the census still reads `0/1 functions in class`, blocking feature `expr-jump`,
  `cflow-loop`, body **294 B**;
* the scan still reads `vocab-gap`, `0/64 bytes`, `1 blocked | label-free |
  needs a CFG class the port lacks: cflow-loop`.

**So w-loop's measurement holds, and it is not a decline under this brief's cost
model.** w-loop priced a *general* loop lowering and named five refusals; three
of the five (**R1** the reader, **R2** `.data` beside `.text`, **R3** the 1:2
relocation fan-out) are mechanisms and two (**R4** the rotated five-block CFG,
**R5** the allocation across the back edge including the exit block's
rematerialisation) are **descriptions of one block plan**. This lane's claim is
w-cfgclass's, one CFG axis over: a transcription owes R4 and R5 **once**, as
sixteen words.

Board **#1398** is the open row this re-attempts. No board row excludes
`expr-jump` (checked: #150 excludes `expr-op-0x27`, #407/#1363 exclude
`assign-store-type-8643`, #1593/#1600 are sink measurements over the key and not
exclusions). `keygen_xbox.cpp` and `mmio.cpp` are excluded by the brief and are
not the target.

## §2 What the port must grow — the registered inventory

Ground truth: `work/w-cfg2/ref/Primes/dis.txt`, real `c2.dll` under wibo at the
workload's own flags, plus `c2`'s own `/FAsc` listing for the block names.

```text
  0000  3d400000  lis   r10,0        REFHI -> ?primes@…   (+PAIR)
  0004  39600000  li    r11,0        i2 = 0
  0008  392a0000  addi  r9,r10,0     REFLO -> ?primes@…   (+PAIR)   r9 = &primes
  000c  814a0000  lwz   r10,0(r10)   REFLO -> ?primes@…   (+PAIR)   r10 = primes[0]
  0010  48000018  b     .+24         THE ROTATION: over the top, into the bottom test
  0014  7f0a1800  cmpw  cr6,r10,r3     <- LOOP TOP
  0018  4098001c  bf    24,.+28      cr6.LT false -> the value-return block
  001c  396b0001  addi  r11,r11,1    i2++
  0020  556a103a  slwi  r10,r11,2
  0024  7d4a482e  lwzx  r10,r10,r9
  0028  2f0a0000  cmpwi cr6,r10,0      <- the `b` at 0x10 lands HERE
  002c  409affe8  bf    26,.-24      THE BACK EDGE
  0030  4e800020  blr                return i — already in r3
  0034  556b103a  slwi  r11,r11,2      <- REMATERIALISED over a value live in r10
  0038  7c6b482e  lwzx  r3,r11,r9
  003c  4e800020  blr
```

## §3 Predictions — scored in the rung, misses in their registered direction

**P1.** Of the fifteen frontier TUs, **`Primes.cpp` is the only single-blocked-
function row whose entire remaining build list is two mechanisms**, and the two
are both in `crates/c2-core/src/coff/` rather than in `codegen/`. Every other
single-function row needs ≥ 3, counted the same way (§0.1).

**P2.** The **first** thing that refuses is in `crates/c2-il`, not
`crates/c2-core`: `expr-jump`, and **no IL body reaches `select_function`**. (This
re-confirms w-loop's R1 rather than discovering it; it is registered because
every clause below depends on it.)

**P3.** w-loop's **R4** and **R5** cost this lane **zero new mechanisms** — they
are paid by transcribing sixteen words, and `codegen::frontier_bytes` has already
built all sixteen from this crate's encoders. **No new instruction encoder is
needed.** If any new encoder turns out to be needed, P3 is wrong.

**P4.** w-loop's **R2** and **R3** are both real and are both paid in
`crates/c2-core/src/coff/`: the `.data` group (a section this writer places only
on the functionless path today) and a REFHI with **two** REFLOs against a
**STATIC** symbol (every REFHI/REFLO site in the writer is strictly 1:1 against
an undefined external).

**P5.** The `.data` group goes **after the code groups**, not between the two
`.XBLD$W` watermarks and not before `.text` — board **#1179**'s third slot, whose
trigger is *"the static's first referrer is a function body"*, which is exactly
this TU.

**P6.** The relocation table is **6 records**: REFHI+PAIR at `0x0000`, REFLO+PAIR
at `0x0008`, REFLO+PAIR at `0x000c` — one REFHI, **two** REFLOs, one symbol — and
the symbol's StorageClass is **STATIC (3)**, not EXTERNAL (2), with `SectionNumber`
naming the `.data` section.

**P7 — THE CONVERSION CALL.** I predict TU match **12 → 13**: this lane
**converts** `Primes.cpp`. Registered against board #770's streak, which is
ten-to-one *optimistic*, so this is the direction that streak says is usually
wrong. The registered reason is P3 + the fact that the emitter half is already
written and oracle-asserted.

**P8 — REGISTERED BIAS.** If P7 is wrong it will be wrong on the **coff half**,
not on the reader half — the inverse of w-cfgclass's P9, which got the sign
backwards on itself. I expect the reader production to be a linear token pattern
(w-cfgclass's D2 measured that shape on a *larger* body) and the `.data` +
relocation composition to be where the surprise is. Four of this project's five
recorded live wrong emits (#259, #276, #1148, #1152) are COFF **shape** defects
and none is a codegen defect.

**P9.** mismatch stays **0**, codegen-gap stays **0**, `scripts/gate.sh
--require-graded` passes 18/18 with 0 mismatch, and
`cargo test --workspace --release` is 0 failed.

**P10.** **Zero** `DISCLOSURE.md` rows. Everything below is derived from the
reference obj, the `/FAsc` listing seam and the IL — all black-box.

**P11.** The class returns `label_slots() == None` (it is label-free and charges
c2's compiler-label counter by an amount this lane does not measure), so a TU
pairing it with a framed function **refuses** — w-hash's #746/#747 mechanism —
and a fixture cell grades that refusal.

## §4 Decline clauses, with frozen thresholds

Evaluated in this order; the first that fires ends the build half and the lane
publishes the measured distance instead.

* **D1 — the initializer.** If the shipped `.gl` data-record cursor
  (`gl_data_objects_ordered`) and the shipped `.in` reader
  (`in_scalar_initializers`) cannot between them produce, for this bundle,
  **both** the name `?primes@?1??NextHashPrime@@YAHH@Z@4PAHA` **and** 248 bytes
  byte-identical to the reference obj's `.data`, decline. That would make the
  `.data` half a *reader* rung and not this one, and the honest deliverable is
  the measurement.
* **D2 — the block IR.** If the reader production is not expressible as a single
  pattern-matched token sequence — i.e. if it requires a general basic-block IR
  with a value merge at a join — decline and publish. #139 forbids splitting a
  lowering across the two crates and a half-built block IR is exactly the
  wrong-emit risk board **#232** cost 241 commits. (w-cfgclass's D2, unchanged.)
* **D3 — the section slot.** If the `.data` group's placement cannot be decided
  by a rule graded on **≥ 3** cells, refuse the composition **fail-closed**
  rather than emit a guessed section order. Board **#1148** is the standing
  precedent and it was closed at zero match cost by an honest refusal.
* **D4 — the fence.** If any fixture cell comes back `Port=Match` on a body whose
  bytes I have not compared word-for-word against real `c2`, stop: that is a
  fence accepting something it was not graded on.
* **D5 — the alarm.** If `gate.sh --require-graded` reports a single `mismatch`,
  revert to the last green commit and publish the revert with its reasoning. A
  refusal becoming a wrong emit is strictly worse than a gap (#232).

## §5 What this lane will NOT do

* Not adopt anything disassembly-derived (P10). `docs/whitebox/` is navigation.
* Not relax `codegen::labels` invariant 4. w-loop measured that relaxing it
  **converts nothing today**; this class computes its own displacements through
  `encode_bc` exactly as `ptr_walk_loop` does and never reaches `LabelMap`.
* Not build a CTR encoder — w-loop declined it with the measurement (`mtctr`/
  `bdnz` appear nowhere in these 64 bytes) and nothing here changes that.
* Not reopen `expr-jump` as a *family*. It is touched here only as the first gate
  in front of one named function class. The other 302 emitted bodies on that key
  (#1535) are untouched, and this lane predicts the emitted census moves by
  **+1**, not by a family's worth.
* Not widen `emit_data_obj`'s functionless scope in a way that changes any obj it
  emits today. Its `MAX_OBJECTS_PER_SECTION`, its `.bss` linkage rule and its
  refusals stay exactly as they are.

## §6 Board numbers

This lane's range is **#1680–#1699**. Numbers not minted are left explicitly
unminted in the rung.
