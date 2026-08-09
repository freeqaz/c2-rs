# w-wordwrap — `wordwrap.cpp` DECLINES at a price nobody had ever published, and the body that did convert turned out to be a CONSTRUCT with a workload population

    Tag:       w-wordwrap
    Slug:      w-wordwrap
    Date:      2026-08-09
    Fixtures:  wwrap_gstore.cpp  wwrap_gstore_widths.cpp
               wwrap_gstore_conv_neg.cpp  wwrap_gstore_lit_neg.cpp
               wwrap_gstore_two_neg.cpp   wwrap_gstore_second_neg.cpp
               wwrap_gstore_float_neg.cpp wwrap_gstore_sub_neg.cpp
               wwrap_gstore_gg_neg.cpp
    Census:    per-function 714,545 → 714,555 · emitted 39,245 → 39,253
    Record:    work/w-wordwrap/{PREREG.md,MUTATIONS.md,NEUTRALITY.txt,board_rows.md}

---

## 1. THE RESULT

> ### **TU match 23 → 23. `src/system/rndobj/wordwrap.cpp` DECLINES, and the decline is priced at **21 named mechanisms** — 3 + 6 + 12 across its three bodies, of which 18 are NEW and 12 sit in one 160-word body whose block plan places a basic block AFTER the epilogue.** No price for this TU has ever existed: #2625 gives bytes, classes and keys and stops there. §3 derives one off this lane's own capture, its own reference obj and its own two probe grids. Board **#2720**.

> ### **`fnbyte-exact` 35,802 → 35,810 — `+8`, not the `+1` the target body is worth.** `void f(T x) { g = x; }` is not a transcription of one function; it is a **construct**, and the workload has eight of it. The PREREG predicted the delta as *"exactly +1"* at 0.50 and the miss is the lane's most useful finding: a lane that prices a frontier body as a one-off prices its own class at reach 1 by assumption. Board **#2721**.

> ### **THE CLASS IS BYTE-EXACT AND ITS OBJ IS HONESTLY REFUSED, ON PURPOSE.** The object stored to is a namespace-scope, non-COMDAT, **uninitialized** global — a `.bss` — and `coff::writer::emit_obj_multi` returns `None` on it **by name**: a non-COMDAT `.bss` goes in the shell between the two `.XBLD$W` watermarks, where every `data_defs` path in that writer places a COMDAT `.data` immediately after its owning function's own `.text`. Different section order, different symbol order, different layout walk, and no cell has graded it on a function-bearing TU. So the TU is `NotImplemented` and the FUNCTION is graded — `comdat::text_reloc_plan` compares relocation targets by **NAME**, never by storage class, so the REFHI/PAIR/REFLO/PAIR quad a `.bss` object needs is the identical plan. Board **#2722**.

> ### **GRID G's `G_narrow` is the cell that makes the class safe, and a length check could not have found it.** `g_us = (unsigned short)x` is **twelve bytes** — the same length as the accepted cell — and **two of its three words differ**: the address scratch moves from r11 to **r10** the moment the body needs a second register. Board **#2723**.

> ### **`/Od` IS THE ONLY MODE THAT DIFFERS, so the gate is `is_some()` and not `== O1`.** `/O1`, `/O1 /Oi`, `/O2`, `/Ox` and `/Ox /Gy` all emit the identical three words. Every `w-xtea3` class is `/O1`-only because its `/Ox` bytes differ; this one is the first shipped class whose mode gate is a *measured non-difference*. Board **#2724**.

| | base `a179e8be` | tip |
|---|---:|---:|
| **TU match** | 23 | **23** |
| mismatch · codegen-gap · port-error | 0 · 0 · 0 | **0 · 0 · 0** |
| vocab-gap · capture-fail | 848 · 7 | 848 · 7 |
| **FRONTIER** | 4 | 4 |
| **`fnbyte-exact`** | **35,802** | **35,810 (+8)** |
| `fnbyte-differs` · `fnbyte-refused` | 1,898 · 114,630 | **1,898** · 114,622 |
| `wordwrap.cpp` blocked bodies / bytes | 3 of 3 · 0/816 (0.0 %) | **2 of 3 · 12/816 (1.5 %)** |
| `frontier-codegen` exact / reader | 11 / 24 | **12 / 23** |
| `frontier-bytefrac-zero` | 2 | **1** |
| **`gap-metric` keys** | 261 | **261 — 0 vanished, 0 appeared, 27 changed** |
| **per-TU verdict SET (878, BY FULL PATH)** | — | **0 MOVED** |
| **per-TU BYTE TRIPLE (878, BY FULL PATH)** | — | **8 MOVED, every one `(n,d,r) → (n+1,d,r−1)`** |
| `c2rs selftest` | 354 | **363 PASS, 0 FAIL, 0 ERROR** |

---

## 2. WORKLOAD STAMP (#2392 — dc3 is not pinned)

```text
c2-rs        a179e8bee8ce548dceafd64fc364dd72bf01efeb → this branch
             worktree .claude/worktrees/agent-a69a2026929c37c17
             branch   worktree-agent-a69a2026929c37c17
             merge-base with master == the base, so nothing was rebased away
base binary  work/w-wordwrap/c2rs-base  md5 de3ccaff61773a875adbed30f9effd6c
             built at the merge base and KEPT; every "base" column is its run
tip binary   work/w-wordwrap/c2rs-tip   md5 2c0886cb18db5d5af9539c7b1ef8f542
dc3-decomp   76ff76519a8c4ea16dbbfcccf305a95d9f8d4f08   2026-08-09T21:28:44Z
             878 TUs.  **dc3 MOVED again under this lane** — `w-xtea3` read
             29802aa3 — and the base scan still reproduces its tip table digit
             for digit (match 23, frontier 4, fnbyte-exact 35,802).
cl.exe/c2.dll/c1xx.dll   compilers/X360/16.00.11886.00
wibo         ../wibo/build/release/wibo
flags        /nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc + 8 /I roots
             the COMMITTED work/dc3-workload/flags.txt, NOT a regenerated one
             (#2700: the generator's include mapping is broken against today's
             dc3 and a regenerated flags.txt reads capture-fail 851)
capture cache  <main checkout>/work/capture-cache, shared by every worktree
```

---

## 3. THE PRICE — 21 named mechanisms, and it is DERIVED

Frozen in `work/w-wordwrap/PREREG.md` §2 **before the first `crates/` change**,
off this lane's own reference obj (`work/w-wordwrap/ref/wordwrap.dump`, 9
sections / 31 symbols / 2,494 B) and its own capture.

| body | bytes | words | mechanisms | of which NEW |
|---|---:|---:|---:|---:|
| `?WordWrap_SetOption@@YAXI@Z` | 12 | 3 | **3** (M1–M3) | 2 — **paid by this lane** |
| `?IsEastAsianChar@@YA_N_W@Z` | 164 | 41 | **6** (M4–M9) | 6 |
| `?WordWrap_CanBreakLineAt@@YA_NPB_W0@Z` | 640 | 160 | **12** (M10–M21) | 10 |

The full table with each mechanism's state is PREREG §2. The four that make the
decline what it is:

* **M14/M15/M16 — three inlined binary searches**, each `sub · srawi ,1 · addze`
  (the signed divide-by-two idiom) · `add · slwi ,2 · lhzx`, with a
  `do { } while (lo <= hi)` back edge fed by **two** update blocks. This is
  **not** the `w-bdnz` counted class — no `mtctr`, no `bdnz`, the step is
  data-dependent — and #1981 excludes a memory reference from
  `counted_accum_loop` **by name**, so that class cannot be widened to it either.
* **M17 — out-of-line block placement, including a block AFTER the epilogue.**
  `.text+0x274` (`addi 11,30,3 · lbzx 11,10,11 · b .-32`) sits *below* the
  `b __restgprlr_29` at `+0x270`. `plan_text_order` has never had to place a
  block after the return.
* **M21 — the register plan over ~40 basic blocks** with r31 = `ch`, r30 = the
  table base, r29 = the option word, and `mr 5,3` in the prologue so r3 is free
  for the two calls.
* **M11 — the label channel**: `$M2666` at `.text+0x0c`, `$M2667` at `+0x280`,
  `$T2668` in `.pdata`. A framed triple in a TU whose **two earlier functions
  are leaves**, which is a lead arithmetic no shipped class has.

**For scale: 160 words is 5.5× the largest body ever transcribed** (`w-xtea3`'s
`?Encipher`, 29 words) and 6.7× the largest framed one (`?Encrypt`, 24). The
project's demonstrated rate is one such transcription per lane (`CEILING.md`
§10.30). **M10–M21 is a lane of its own at least twice over**, and this lane
says so rather than leaving the number implicit.

---

## 4. `CEILING.md` §11.4, RUN FIRST AND OFF THIS LANE'S OWN CAPTURE

Full pass in PREREG §3. The four that carried the lane:

**Item 8 — quote the GATE's number.** `emit-bound 3 == emit-gate-segments 3 ==
emit-record-offsets 3 == emit-records 3`; `gate_cause "body-out-of-class"`,
`gate_causes ["body-out-of-class", "unclaimed-gl-symbol"]` — no `gl-stop-*`, no
`bind-*`. The gate **binds**. `fn_names` reads **5** against `fn_total` **3**,
which is exactly the field #2621 warns is not the answer; here the two extra
names are the TU's two `.bss` data symbols.

**Item 2 — is this T1?** **No**, and that is checked rather than assumed:
`fnbyte-exact` is **0** of 3, not the denominator. Unlike `EncryptXTEA.cpp` this
TU's remaining distance really is codegen, so NC-1's six obligations and NC-2's
four are not what is missing.

**Item 9 — the port's FENCES, before its obligations.** Run **even though T1
does not fire**, because M12 depends on it: `comdat::fenced_inlined_callee`
tests a same-TU callee against `INLINE_DECLINE_BYTES` = 128 and
`?IsEastAsianChar` is **164** emitted bytes, so the fence permits the two `bl`
sites. **NC-5 is not in this TU's way**, and that is a measurement rather than
an assumption. `elide`'s mechanism E and `splice`'s S7 cannot apply to a TU with
no accepted body.

**Item 5 — do not trust the key's LAYER**, and all three of this TU's keys are
fall-throughs. `expr-jump` is reported on a body with **no jump** (#2387, hand
checked on this exact function; the census window this lane took stops on the
exit-label goto every accepted leaf also carries). `expr-cmp-eq` is reported on
the **first statement** of a 160-word body, which tells a lane nothing about the
other 159 words.

**Item 7 — the board, grepped before sizing**, and #807 is the row that matters
and that no forward doc repeats: a lane reading this TU as *"just needs
`cflow-if-n`"* is wrong, because `cfg_reach` returns `NeedsClass` **before** it
checks `classified < blocked_total` and this TU has both.

---

## 5. WHAT SHIPPED — `global_store_leaf`, and why it is a CONSTRUCT and not a transcription

```text
   0000  3d600000  lis      r11,0          REFHI <g> + PAIR
   0004  906b0000  st{b,h,w,d} r3,0(r11)   REFLO <g> + PAIR
   0008  4e800020  blr
```

Three words, **one free field** (the store width), and every other choice is a
compiled reading. Two grids, both real `c2.dll` under wibo:

* **GRID G** (`work/w-wordwrap/probe/gstore.cpp`, 17 cells) — the boundary.
  `G_i`, `G_static`, `G_ext` and `G_vol` are byte-identical to the target;
  `G_lit`, `G_widen`, `G_narrow`, `G_two`, `G_arr`, `G_arr2` each move the
  address scratch to **r10**, because the body needs a second register.
* **GRID T** (`work/w-wordwrap/probe/gtype.cpp`, 16 cells) — the store-opcode
  table, enumerated on `(tag, kind)` rather than computed from a width nibble
  (`read_type`'s own doc records `0x86`/`0xA6`/`0x96`/`0xC6` as unreliable
  there). `86 45 …` (float) and `88 85 …` (double) are **absent by name**: c2
  stores them with `stfs f1` / `stfd f1`, out of the register file `params` does
  not describe.

### 5.1 It has a workload POPULATION, which is the finding

`fnbyte-exact` moved **+8**, in **eight different TUs**, every one
`(n, d, r) → (n+1, d, r−1)`:

```text
   src/system/char/Character.cpp          src/system/meta/FixedSizeSaveable.cpp
   src/system/obj/DirLoader.cpp           src/system/rndobj/wordwrap.cpp
   src/system/synth/Sequence.cpp          src/system/utl/ChunkStream.cpp
   src/system/utl/Locale.cpp              src/system/utl/MemMgr.cpp
```

Seven of the eight are locatable in the dc3 source as one line each —
`SetActiveChunkObject`, `DirLoader::SetCacheMode`,
`Character::SetDebugDrawInterestObjects`, `RandomGroupSeq::ForceSerialSequences`,
`MemForceNewOperatorAlign`, `FixedSizeSaveable::EnablePrintouts`, and the target
`WordWrap_SetOption`. The eighth (`Locale.cpp`) is not found by a source grep and
is named by the byte instrument only.

**#2005 is corrected rather than confirmed.** That row ranked seven
*"call-free, class-sized"* `cflow-straight` bodies and named this construct as
one of them, at **0 convertible** — because it was sizing them against the
`w-bdnz` loop class's cell widths. The construct's real population is a
different set of eight and it is entirely convertible. That is
[[ranking-instruments-measure-themselves]] once more, from the other side: the
ranking that said *zero* measured the wrong class's widths.

The memory note [[codegen-is-the-lever]] says adoption is **construct rungs**,
and this is the first lane to hit that from the frontier direction: it went
looking for one body and found a construct.

### 5.2 The `.bss` object, carried and refused

`IlDataDef` gains `uninitialized`; `Bindings` gains `resolve_bss_def`, the exact
complement of `resolve_data_def`'s three gates (**not** COMDAT, **not**
initialized, not thread-local) with **no `.in` read at all**, because an
uninitialized object has no `.in` record for the totality gate to check.

`coff::writer::emit_obj_multi` refuses it **by name**, and the refusal is the
honest state rather than a gap:

* a non-COMDAT `.bss` goes in the **shell**, between the two `.XBLD$W`
  watermarks and before any `.text` (`work/w-wordwrap/probe/setopt.obj` is one
  obj showing it: `.drectve · .debug$S · .XBLD$W · .bss · .XBLD$W · .text`);
* every `data_defs` path in that writer places a **COMDAT `.data`** immediately
  after its owning function's own `.text` (GRID C's interleave, `w-data`);
* `wordwrap.obj`'s own `.bss` is **588 B holding TWO objects shared by THREE
  functions**, at `?g_LineBreakTable` **+0x0** and `?g_uOption` **+0x248** —
  the reverse of declaration order, which is Rule A1's permutation and a walk
  this writer has never run on a function-bearing TU.

So the obj is `NotImplemented` and the **function** is graded, which is a
different question and is answerable today. That is the state `w-front5`
measured on `mmio.cpp` — ten of eleven bodies `fnbyte-exact`, TU refused —
reached deliberately instead of by accident.

`data_defs_of` learns the four **store** spellings of a low half; the scan
loop's are `addi`/`lwz` and this class's is the store itself. All four widths
are carried, because a table with only `stw` would emit one relocation where a
`char` or a `long long` global needs two — a wrong `NumberOfRelocations`, not a
gap.

---

## 6. NEUTRALITY, AT FOUR LEVELS, WITH DIRECTIONS

`work/w-wordwrap/NEUTRALITY.txt`, produced by `work/w-wordwrap/neutrality.sh`,
whose two comparators assert the runs cover the same key set before comparing
anything.

### 6.1 878 workload TUs, BY FULL PATH — **0 MOVED**

```text
   workload 878   878 rows, 0 MOVED
       a = b = {'vocab-gap': 848, 'match': 23, 'capture-fail': 7}
```

**Not one verdict moved in either direction, including the target's.** The key
is the WHOLE path and not the basename: 878 workload TUs collapse to **841**
basenames — 37 collisions — and the collapsed comparison silently drops 37 rows
while still printing `0 MOVED` (#2667).

### 6.2 The per-TU BYTE TRIPLE, same 878 rows, same key — **8 MOVED, all up**

```text
   src/system/char/Character.cpp          (264, 12, 934) -> (265, 12, 933)
   src/system/meta/FixedSizeSaveable.cpp   (23,  5, 111) -> ( 24,  5, 110)
   src/system/obj/DirLoader.cpp            (74,  4, 312) -> ( 75,  4, 311)
   src/system/rndobj/wordwrap.cpp          (  0, 0,   3) -> (  1,  0,   2)
   src/system/synth/Sequence.cpp           ( 95, 7, 469) -> ( 96,  7, 468)
   src/system/utl/ChunkStream.cpp          ( 17, 0,  74) -> ( 18,  0,  73)
   src/system/utl/Locale.cpp               ( 27, 2,  82) -> ( 28,  2,  81)
   src/system/utl/MemMgr.cpp               ( 20, 2,  99) -> ( 21,  2,  98)
   total exact  35802 -> 35810   differs 1898 -> 1898   refused 114630 -> 114622
```

**The middle column does not move on a single row.** That is what the aggregate
key map cannot say: a `+1` in one TU and a `−1` in another sum to zero in the
totals and move no verdict at all.

### 6.3 Every `gap-metric` key, as a key → value MAP

261 keys at base, **261** at tip. **0 vanished, 0 appeared, 27 changed**, every
one in the direction eight converted bodies imply — `fnbyte-exact`,
`-exact-bytes`, `-exact-relocated`, `-calltarget-agree`, `-reloc-graded`,
`-shape-plain-exact`, `progress-emitted-in-class` all `+8`;
`fnbyte-refused`/`-decline-parse`/`-refused-parse` all `−8`;
`frontier-codegen-exact` 11 → **12**, `frontier-codegen-reader` 24 → **23**,
`frontier-bytefrac-zero` 2 → **1**.

`fnbyte-differs` does not appear in the changed list at all — it is 1,898 before
and after, checked **per key** rather than by subtracting totals (`w-empty`'s
rule).

**The per-function CENSUS moves +10 where the byte judge moves +8**
(714,545 → 714,555 against 35,802 → 35,810), and the two are not the same
population: the census is keyed on `Bindings::positional` and grades every body
it can parse, the byte judge on `FnCensus::emit_name` and only bodies with a
reference COMDAT to compare against. #2220's fail-open, visible at a scale
small enough to read. **Only the +8 maps to the goal** (§10.2).

One key deserves naming because it moved the *other* way by one:
**`fnbyte-name-disagree` 74,677 → 74,678**. That is #918's census/gate binding
disagreement counter, and a class whose TU the gate refuses while the byte
instrument grades it is exactly what adds to it. It is the instrument recording
this lane's own shape, not a regression.

### 6.4 Every fixture at `/O1` AND `/Ox`, both binaries

The list is regenerated inside `work/w-wordwrap/fixscan.sh` on every invocation
and its length printed. **A fixture added after a cached list was written is a
fixture nobody graded.**

The `/Ox` half is mandatory and is not a formality: `w-biquad` shipped a live
wrong emit at offset 760 that the `/O1`-only workload scan, the `/O1` fixture
lane and every workspace test missed.

```text
   fixtures /O1   363 rows, 2 MOVED — both this lane's, both vocab-gap -> codegen-gap
      base: {'vocab-gap': 181, 'match': 170, 'codegen-gap': 12}
      tip : {'vocab-gap': 179, 'match': 170, 'codegen-gap': 14}
   fixtures /O1   363 rows, 3 BYTE TRIPLES moved
      w25_store_leaf_neg.cpp   (4,0,11) -> (5,0,10)   <== a DIFFERENT class's `_neg`
      wwrap_gstore.cpp         (0,0, 1) -> (1,0, 0)
      wwrap_gstore_widths.cpp  (0,0, 3) -> (3,0, 0)
      total differs 7 -> 7
   fixtures /Ox   363 rows, 2 MOVED — the same two, the same direction
      base: {'vocab-gap': 200, 'match': 146, 'codegen-gap': 17}
      tip : {'vocab-gap': 198, 'match': 146, 'codegen-gap': 19}
   fixtures /Ox   363 rows, 0 BYTE TRIPLES moved
```

**`mismatch` 0 in all six runs.** `match` does not move at either mode, because
the class's obj is refused (§5.2) — so its cells go `vocab-gap → codegen-gap`,
which is T1 ALL-EXACT-NO-MATCH firing **on purpose**, on the NC-2 obligation the
writer refusal names.

At `/Ox` the byte triples do not move at all, and the reason is a property of
the mode rather than of the class: without `/Gy` there is no COMDAT `.text`
split, so `fnbyte-denominator` is **0** for every one of these fixtures and
there is nothing to compare. The `/Ox` half's evidence here is the twelve-profile
`mismatch 0`, not a byte count.

**`w25_store_leaf_neg.cpp` is a `_neg` cell of a DIFFERENT class that gained a
byte.** Its `n_global(int v){ g_i = v; }` — which that file lists as outside the
STORE-LEAF class, *"needs a `.data`/`.bss` symbol and an ADDR32 relocation
pair"* — is exactly this class's shape. Its verdict does not move and its own
assertion (*0 of these may be in the store-leaf class*) still holds. Recorded in
the fixture itself, so a reader who sees a `_neg` file gain an exact body checks
**which class** took it rather than reading it as a leak.

---

## 7. THE MUTATION GRID — four graded, one VOID, three that grade nothing

`work/w-wordwrap/MUTATIONS.md`. **The grading is `fnbyte-*` and not the TU
verdict**, and that is forced: the class's obj is refused by name, so no fixture
of this class can reach `match`. `fnbyte` is still the oracle — real c2's obj,
bytes **and** all four relocation records — so a mutation that admits a `_neg`
cell surfaces as **`fnbyte-differs`**, the same evidence a `mismatch` would be.

| # | conjunction deleted | cell | base → mutated |
|---|---|---|---|
| M1 | the value token must be `params[0]` | `gg_neg` | refused → **differs** |
| M2 | the `2C` clause **and** the store-type restatement | `conv_neg` | refused → **differs** |
| M3 | the arity fence **and** the value-token comparison | `second_neg` | refused → **differs** |
| M4 | GRID T's missing float row | `float_neg` | refused → **differs** |

**M3 reproduces #2665 live rather than quoting it.** Its first run deleted the
arity fence alone and came back **GREEN**, because `second_neg`'s value is
`params[1]` and `val_tok != params[0]` refused the body anyway. The two clauses
are one conjunction over that cell; the failed first run is recorded beside the
merged second.

**M5 — the mode gate — is VOID and says why.** `census_functions`' own
post-parse gate (b) raises `opt-mode-00800005` at `/Od` independently, so no
deletion of the parser clause can be graded (and at `/Od` without `/Gy` there is
no COMDAT `.text` split at all, so `fnbyte-denominator` is 0). The clause is
kept regardless — #1638 puts the gate in the PARSER so the census reports the
class's own key — and is recorded as VOID rather than counted.

**Three cells grade nothing and are named rather than counted** (#2698):
`lit_neg`, `two_neg` and `sub_neg` are refused **structurally**, so deleting the
clause that names them desynchronizes the cursor rather than admitting the body.
They stay as the compiled record of what c2 emits for each neighbour.

**The control is run, not asserted**: disabling the dispatcher arm returns both
accepted fixtures to `vocab-gap` with `fnbyte-exact 0`, so every accepted cell
is accepted by this production and by nothing else.

---

## 8. GATE

| lane | result |
|---|---|
| `scripts/gate.sh --jobs 4` | **GATE: PASS (HATCH-RED REFUSED)** — **18/18 lanes**, 0 FAIL, 0 SKIP, 0 NO-RESULT; **6,534 fixture-verdicts**; sweep **19,460 of 19,556** graded; cross **90,424 of 90,812** cells; **0 mismatch anywhere**; `ladder-red` PASS. 618 s wall |
| `cargo test --workspace --release --no-fail-fast` | see §8.2 |
| `c2rs selftest` | **363 PASS, 0 FAIL, 0 ERROR** (base 354) |
| 878-TU workload scan | match **23** / mismatch **0** / **0 verdicts moved**, 8 byte triples moved and every one up |
| fixtures at `/O1` and `/Ox` | **363** each, both binaries, **0 mismatch**, 2 moved at each mode and both are this lane's |
| must-fail mutations | **4 of 4** graded, 1 VOID, `work/w-wordwrap/MUTATIONS.md` |
| must-refuse control | **1 of 1** (§7) |

### 8.1 The 18 lanes — every one at 363/363, and the class is visible to ALL of them

```text
   O1              363/363   170 match   0 mismatch   /O1
   O1-EHsc         363/363   170          0           /O1 /EHsc
   O1-Oi           363/363   172          0           /O1 /Oi
   O1-Oi-EHsc      363/363   172          0           /O1 /Oi /EHsc
   O1-Oi-GR        363/363   172          0           /O1 /Oi /GR
   O1-Oi-EHsc-GR   363/363   172          0           /O1 /Oi /EHsc /GR
   Ox / Ox-EHsc    363/363   146          0
   Ox-Gy / -EHsc   363/363   144          0
   Ox-GR / -EHsc   363/363   146          0
   O2 / O2-EHsc    363/363   150          0
   Od / Od-EHsc    363/363    18          0
   Od-GR / -EHsc   363/363    18          0
   expr-sweep    19556/19556 19460        0           generated cases
   mode-cross    90812/90812 90424        0           case-lane cells
   ladder-red        5/5         3        n/a         arms (2 green controls)
```

**No lane's `match` column moves**, at any mode, which is the direct consequence
of §5.2: the class's obj is refused everywhere, so its nine fixtures are
`codegen-gap` at every profile and `match` at none. Twelve profiles graded them
and **none found a `mismatch`** — which is what the `/Ox` half is for, and it is
not a formality: `w-biquad` shipped a live wrong emit at offset 760 that the
`/O1`-only workload scan, the `/O1` fixture lane and every workspace test
missed.

### 8.2 `hatch-red` — REFUSED, pre-existing, REPRODUCED AT THIS LANE'S EXACT BASE

`hatch-red` reports **`REFUSED HATCH-STALE`** — `hatch.py apply` cannot hatch
this tree, board **#1389**'s open shape, which exits 0 by design and forfeits
the unqualified headline (#1406).

It is **reproduced rather than argued**, and at this lane's own base rather than
at an earlier lane's: a **detached worktree at `a179e8be`** with
`work/w-hatch/hatch_red.py` copied in and run *from inside it* gives

```text
   FAILED: R2 DIRTY+HATCH, R6 RESIDUE, A2 PAID-MISSING, F1 FORCE, C1 HATCH-ONLY
```

— **the same five arms, in the same order**, as `w-xtea3` recorded at `299f9a8c`
and `w-xtea2` at `af81b869`. The worktree was removed afterwards.

**And the gate row cost nothing**, because everything was committed before it
ran (#2668/#2699): `git status` is clean over `crates/` at the start and the
release build inside the gate is a no-op.

---

## 9. THE PREREG, SCORED

Frozen at `f06a6f4b`, before the first `crates/` change, with `git status`
recorded clean in that commit. §1 of that file is a declared-prior list and is
unscored on purpose. Rows downstream of a conversion the lane might never reach
are marked **cond.** — `w-xtea2` §9.1's scoring lesson, which `w-xtea3` applied
and which this lane applied again and *still* got wrong once (see P11).

| # | prediction | p | outcome |
|---|---|---:|---|
| **P1** | `wordwrap.cpp` CONVERTS, 23 → 24 | **0.07** | **MISS** — and the price is why |
| **P1a** | *cond. ¬P1* — the decline is published with N ≥ 15 named | 0.85 | **HIT** — N = **21** (§3) |
| **P2** | `fnbyte-exact` delta ≥ +1 | 0.75 | **HIT** |
| **P3** | the delta is **exactly +1** | 0.50 | **MISS** — it is **+8** |
| **P4** | `?WordWrap_SetOption` converts | 0.75 | **HIT** |
| **P5** | `?IsEastAsianChar` converts | 0.20 | **MISS** — not attempted |
| **P6** | `?WordWrap_CanBreakLineAt` converts | 0.03 | **MISS** — not attempted |
| **P7** | nothing regresses | 0.95 | **HIT** (unlosable — flagged) |
| **P8** | `mismatch` 0 on every gate row | 0.90 | **HIT** (unlosable — flagged) |
| **P9** | the 878 verdict set moves for `wordwrap.cpp` **and nothing else** | 0.85 | **MISS** — it moves for **nothing**, the target included |
| **P10** | *cond. P4* — M1 is a NEW production and no shipped byte-graded class is widened | 0.85 | **HIT on its letter, and the letter was too narrow** — see below |
| **P11** | *cond. P4* — the `.bss` symbol needs a `data_syms` arm, and without it the TU reports `unclaimed-gl-symbol` | 0.60 | **VOID — and this is the calibration finding** |
| **P12** | *cond. P4* — the body is identical at `/O1` and `/Ox`, so no mode gate is needed | 0.45 | **HIT**, at five profiles rather than two (#2724) |
| **P13** | `hatch-red` still REFUSES, pre-existing, reproduced at this lane's base | 0.85 | see §8.1 |
| **P14** | the label channel is not touched | 0.80 | **HIT** — no `label_lead` / `label_slots` / `plan_labels` edit ships |
| **P15** | *cond. P4* — no new integration-test FILE | 0.75 | **HIT** — the tests are `#[cfg(test)]` modules inside the two new source files |

**Thirteen graded, eight HIT, four MISS, one VOID.** The four misses are worth
more than the eight hits and they are not four mistakes.

### 9.1 P3 is the useful miss: a frontier body was priced as a one-off

`fnbyte-exact` moved **+8** where the PREREG said *exactly +1* at 0.50. The
error is not arithmetic; it is a category error the whole lane was framed
around. The commission, the frontier inventory, `#2625` and this lane's own §3
all describe `?WordWrap_SetOption` as **a body** — the smallest unconverted body
on the frontier, *worth +1 `fnbyte-exact` and 0 conversions*. It is not a body.
It is `void f(T x) { g = x; }`, a **construct**, and the workload has eight of
it.

A frontier-driven lane sees one instance of every class it builds, because the
frontier is a list of TUs. **Nothing in the frontier's framing can tell a lane
whether the class it is about to write has a population**, and the cheap check —
grep the workload for the construct before pricing the class — was never run
here or, as far as the board shows, anywhere.

### 9.2 P9 and P11 are ONE mistake, and it is the one `w-xtea3` warned about

P9 (*"the verdict set moves for `wordwrap.cpp` and nothing else"*) and P11
(*"the `.bss` symbol needs a `data_syms` arm, and without it the TU reports
`unclaimed-gl-symbol`"*) were both written as though the TU would reach the obj
writer. P11 was even flagged in PREREG §5.1 as **the row that could actually
decide the outcome** — and it is void, because its real antecedent is *the TU
assembles an obj*, which needs all three bodies, and the lane priced that at
**0.07**.

So the lane repeated `w-xtea2`'s error in a subtler form: not *"a confident
claim about a rung never reached"* but **a conditional whose stated antecedent
(P4) was not its real one**. `w-xtea3`'s rule — *declare downstream predictions
conditional* — is necessary and was followed; it is not sufficient. The missing
half is: **check that the antecedent you wrote is the one the claim actually
needs.**

### 9.3 The row that decided the outcome, and it was not registered either

Neither the conversion nor the byte delta was decided by any row above. What
decided both is a fact about the **writer**: a non-COMDAT `.bss` on a
function-bearing TU has no graded placement, so the class's obj is refused no
matter how many of its bodies are exact. That is why nine fixtures grade
`codegen-gap`, why the target TU's verdict cannot move even if all three bodies
land, and why the lane's whole yield is a byte number.

It is registered now, as board **#2727**, with two graded cells left behind for
it. A lane that had run `CEILING.md` §11.4 item 3 — *read the reference obj's
SYMBOL TABLE* — as a **forecast about what the writer would owe** rather than as
a checklist would have carried it in the PREREG. §3 of that file describes the
588-byte `.bss` and both its symbols' offsets correctly, and then prices only
`.text`.

### 9.4 P10's letter was too narrow, and the widening is named

P10 said *"no shipped byte-graded class is widened"*, and none was. But
`crate::data_defs_of` — a **shared relocation derivation** that
`static_scan_loop` also uses — did gain four store spellings for its low-half
detector. It is not a class, so the prediction's letter holds; it is exactly the
kind of shared, byte-graded code D7/#232 is about, so the letter should have
covered it. Empirically neutral: `fnbyte-differs` is 1,898 on every one of 878
TUs before and after, and no verdict moved.

---

## 10. WHAT THIS LANE DELIBERATELY DID NOT DO

* **`?IsEastAsianChar` and `?WordWrap_CanBreakLineAt` were not attempted.**
  Priced at M4–M21 (§3) and declined, with the 160-word body's twelve mechanisms
  named. This is the decline the commission asked for if the conversion did not
  land, and the number is 21.
* **`counted_accum_loop` was not widened** to the binary search's back edge.
  #1981 excludes a memory reference from that class BY NAME and every one of the
  three searches has an `lhzx` inside its loop.
* **`INLINE_DECLINE_BYTES` was not touched**, and nothing needed it: item 9 shows
  the fence already permits `wordwrap.cpp`'s two `bl` sites.
* **`gl_defined_names` was not widened.** #2622/#2623 measured that repair at 0
  conversions and **−1 `fnbyte-exact`**.
* **The label channel was not touched** — no `label_lead`, `label_slots` or
  `plan_labels` edit ships. The only body in this TU that owns a `$M` triple is
  the 640-byte one, which did not convert.
* **`data_syms` was not used for the destination.** GRID G's `G_ext` shows a
  global this TU does not define emits the identical `.text`, and admitting it
  through `data_syms` would have needed a new `LoForm` in `data_refs_of` — a
  widening of a shared, byte-graded derivation for a cell the frontier does not
  contain. Refused; the cost is that no fixture of this class reaches `match`.
* **The `.bss` writer placement was not derived.** It is M2/M3's second half and
  it is part of the price, not a gap this lane papered over.
* **`scripts/gen_dc3_workload.sh` was NOT repaired** (#2700), by name.

---

## 11. FOUND AND NOT TAKEN

1. **The `.bss` placement on a function-bearing TU is now the cheapest unpaid
   obj-level rung on the board**, and this lane leaves two graded cells behind
   for whoever takes it: `work/w-wordwrap/probe/setopt.obj` (one object, 4 B,
   one function) and `work/w-wordwrap/ref/wordwrap.obj` (two objects, 588 B,
   three functions, in Rule A1's reversed order). Paying it turns nine fixtures
   from `codegen-gap` into obj-level grading and is the only thing standing
   between this class and a TU verdict.
2. **`#2005`'s seven-body ranking measured the wrong class's cell widths** and
   published **0 convertible** for a construct with eight convertible members.
   Every other row on the board that ranks bodies by *size against an accepted
   class* inherits that defect.
3. **`fnbyte-name-disagree` is now a lane-shape detector.** It moved +1 here, and
   the reason is precisely that the gate refuses a TU the byte instrument
   grades. A sweep of its 74,678 rows would separate "the port cannot bind this"
   from "the port binds it and refuses the obj" for free.
4. **Seven of the eight new `fnbyte-exact` bodies are one line of C++ each**, and
   the eighth is not locatable by a source grep. An instrument that printed the
   emitted SYMBOL beside a moved byte-triple would have named all eight; the
   census truncates its per-function rows on a 1,400-function TU.
