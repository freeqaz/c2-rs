# w-wordwrap — PREREG, frozen before the first `crates/` change

Lane `w-wordwrap`, 2026-08-09. Branch `worktree-agent-a69a2026929c37c17`, base
`a179e8be` (master tip, the `w-xtea3` merge + its STATUS regeneration).
**Nothing under `crates/` is modified at the moment this file is committed** —
checked with `git status` and recorded in the commit that adds it.

Commission: convert `src/system/rndobj/wordwrap.cpp`, TU match **23 → 24**.
It is one of the four remaining frontier members and one of only **two** that
codegen can reach at all (`w-front5` #2621/#2405: `src/Main.cpp` and
`mmio.cpp` both fail `Bindings::per_record` before a body is read). Of the two
it is the smaller: **3 bodies, 816 B**, against `keygen_xbox.cpp`'s 19 / 1,432.

**No published price has ever existed for this TU beyond a body list**
(#2625 gives bytes, classes and keys and explicitly stops there). Deriving one
is deliverable 1 and it is derived in §2 below, off this lane's own capture,
its own reference obj and its own probe — not off any inventory.

---

## 0. WORKLOAD STAMP (#2392 — dc3 is not pinned)

```text
c2-rs        a179e8bee8ce548dceafd64fc364dd72bf01efeb
             worktree .claude/worktrees/agent-a69a2026929c37c17
             branch   worktree-agent-a69a2026929c37c17
             merge-base with master == HEAD (nothing to rebase at freeze)
base binary  work/w-wordwrap/c2rs-base   md5 de3ccaff61773a875adbed30f9effd6c
             built at the merge base and KEPT; every "base" column is its run
dc3-decomp   76ff76519a8c4ea16dbbfcccf305a95d9f8d4f08   2026-08-09T21:28:44Z
             878 TUs.  **dc3 has MOVED again since `w-xtea3` (29802aa3 ->
             76ff7651)** and the base scan still reproduces `w-xtea3`'s tip
             table digit for digit — see below.
cl.exe       compilers/X360/16.00.11886.00/cl.exe
c2.dll       compilers/X360/16.00.11886.00/c2.dll
c1xx.dll     compilers/X360/16.00.11886.00/c1xx.dll
wibo         from PATH / ../wibo/build/release/wibo
flags        /nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc + 8 /I roots
             the COMMITTED work/dc3-workload/flags.txt, NOT a regenerated one
             (#2700: gen_dc3_workload.sh's include mapping is broken against
             today's dc3 and a regenerated flags.txt reads capture-fail 851)
capture cache  <main checkout>/work/capture-cache (shared by every worktree)
```

Base scan, this lane's own run (`work/w-wordwrap/base_metrics.txt`):

```text
match 23 · mismatch 0 · codegen-gap 0 · vocab-gap 848 · port-error 0
capture-fail 7 · frontier 4
fnbyte-exact 35802 · fnbyte-differs 1898 · fnbyte-refused 114630
```

Every digit agrees with `w-xtea3`'s tip table, so the base is that tree and
nothing is inherited unchecked.

---

## 1. DECLARED PRIORS — measured BEFORE this file froze, and therefore NOT scored

These are §2's price and §3's checklist pass. They are measurements, not
predictions, and none of them is graded in §5.

---

## 2. THE PRICE, DERIVED — 21 named mechanisms across 3 bodies, 12 of them in one body

`CEILING.md` §11.4 item 8 first: the gate **binds**. From this lane's own scan
row, `emit-bound 3 == emit-gate-segments 3 == emit-record-offsets 3 ==
emit-records 3`; `gate_cause "body-out-of-class"`, `gate_causes
["body-out-of-class", "unclaimed-gl-symbol"]` — no `gl-stop-*`, no `bind-*`.
(`fn_names` reads **5** against `fn_total` **3**, which is exactly the field
#2621 warns is *not* the answer: the two extra names are the TU's two `.bss`
data symbols.)

Item 1, the byte judge: `fnbyte-denominator 3 · exact 0 · differs 0 ·
refused 3`, all three `fnbyte-decline|parse`; `bytefrac-exact 0 / 816`.
**So T1 does not fire and item 9 does not apply — there is no all-exact body
here, and this TU's distance really is codegen.** That is the opposite of
`w-xtea3`'s finding and it is checked rather than assumed.

Every row below is read off `work/w-wordwrap/ref/wordwrap.dump` (the reference
obj at the workload's own flags) and `work/w-wordwrap/il/` (this lane's own
capture). "shipped" means a production already in `crates/` that the row can
compose; "NEW" means nothing in the tree emits it.

### 2.1 `?WordWrap_SetOption@@YAXI@Z` — 12 B, 3 words, **3 mechanisms**

`lis 11,0 ; stw 3,0(11) ; blr`, four relocations (REFHI/PAIR, REFLO/PAIR).
Census window: `… 32 86 42 75 4b >3a< f9 09 …` — the parse reaches the end of
the store statement and stops on the **exit-label goto**, so #2625's published
key `expr-jump` names none of the refusal (#1416, and `w-nc` #2387's
hand-found instance is this exact function).

| # | mechanism | state |
|---|---|---|
| **M1** | a store production whose destination is a **file-scope global** — `26 <tok>` naming a `.gl` DATA symbol, not a `.sy` automatic. `leaf_store::parse_ref_bind_stmt` reads `26 <tok>` and requires `sy.ptr_locals.contains(tok)`; nothing in the tree accepts the other side | **NEW** |
| **M2** | `lis rH,hi(sym)` + `stw rV,lo(sym)(rH)` against an undefined `.bss` EXTERNAL, with the REFHI/PAIR + REFLO/PAIR quad | REFHI/REFLO quads shipped (`comdat.rs` WR1, `data_syms`); this *store* form NEW |
| **M3** | the global must reach `IlFunction::data_syms` or the TU refuses with `unclaimed-gl-symbol` — the second `gate_cause` on this TU's row, already visible at base | shipped mechanism, NEW arm |

### 2.2 `?IsEastAsianChar@@YA_N_W@Z` — 164 B, 41 words, **6 mechanisms**

Census window: `… 33 86 41 74 04 0b >38< fd 09 …` — the parse reaches the
`g_uOption & 4` guard's AND and stops on the **branch-false**.

| # | mechanism | state |
|---|---|---|
| **M4** | a `wchar_t` formal zero-extended **once** into a scratch (`clrlwi 11,3,16`) and reused by all twelve tests | NEW |
| **M5** | `g_uOption & 4` fused into a **record-form rotate** — `lis ; lwz ; rlwinm. 11,11,0,29,29 ; bt 2` — a mask-and-test that emits no `andi.` and no literal | NEW |
| **M6** | the unsigned **range test** `a <= x <= b` as a two-compare pair on a **shared cr6**: `cmplwi 6,x,a ; bt 24 ; cmplwi 6,x,b ; bf 25` — the CR-bit numbering (24 = lt, 25 = gt) and the polarity of each | NEW |
| **M7** | short-circuit `\|\|`/`&&` (IL `1B`/`1C`) over **twelve** tests with two distinct exits and c2's own choice of which side falls through — a branch-polarity plan, not one instruction | NEW |
| **M8** | the `bool` value merge `li 11,1 ; b +8 ; li 11,0 ; clrlwi 3,11,24 ; blr` | NEW |
| **M9** | the block linearization: the `&4` guard's taken side jumps **forward past** the first disjunction into a second copy of the same three range tests (`.text+0x4c`), i.e. c2 duplicates the shared prefix rather than sharing it | NEW |

### 2.3 `?WordWrap_CanBreakLineAt@@YA_NPB_W0@Z` — 640 B, 160 words, **12 mechanisms**

Census window: `… b9 03 0a 86 43 81 20 >1f< 38 07 0a …` — the parse stops on
the pointer **compare-eq** of the very first statement (`cur == start`).
**160 words is 5.5× the largest body ever transcribed** (`w-xtea3`'s
`?Encipher`, 29 words) and 6.7× the largest framed one (`?Encrypt`, 24).

| # | mechanism | state |
|---|---|---|
| **M10** | the frame: `mflr 12 ; bl __savegprlr_29 ; stwu 1,-112(1)` / `addi 1,1,112 ; b __restgprlr_29` at **N = 29** | shipped (`wb-frame`, byte-exact via `w-xlr`) |
| **M11** | the label channel: `$M2666` at `.text+0x0c`, `$M2667` at `+0x280`, `$T2668` in `.pdata` — a framed triple in a TU whose **two earlier functions are leaves**, which is a lead arithmetic no shipped class has (`w-xtea3` #2694 shipped the first non-zero *leaf* lead; this is the first TU where the leaves come first and the framed one last) | shipped mechanism, NEW measurement |
| **M12** | two same-TU `bl ?IsEastAsianChar` REL24 sites — and the inline fence must permit them. Pre-checked (item 9): the callee is **164** emitted bytes against `INLINE_DECLINE_BYTES` 128, so `comdat::fenced_inlined_callee` does **not** refuse; NC-5 is not in this TU's way | shipped |
| **M13** | two globals hoisted into **callee-saved** registers for the whole body: `lis 10 ; lwz 29,0(10)` (the option word) and `lis 11 ; addi 30,11,0` (the table base, a REFLO **`addi`** address materialization rather than a load) | NEW |
| **M14** | the inlined **binary search**: `sub ; srawi 10,10,1 ; addze` (the signed divide-by-two idiom), `add ; slwi 10,9,2 ; lhzx 8,10,30` (scaled index + indexed halfword load) | NEW |
| **M15** | its back edge: a `do { } while (lo <= hi)` with **two** update blocks (`addi 7,9,-1` / `addi 11,9,1`) joining at `cmpw 6,11,7 ; bf 25,.-52`. This is **not** the `w-bdnz` counted class — no `mtctr`, no `bdnz`, step is data-dependent — and #1981 excludes a memory reference from `counted_accum_loop` **by name**, so that class cannot be widened to it either | NEW |
| **M16** | **three** copies of M14/M15, each reading a different field of the found row (`lhzx` +0 for `.ch`, `lbzx` +2 for `cantBreakBefore`, `lbzx` +3 for `cantBreakAfter`) and each with its own exit | NEW |
| **M17** | **out-of-line block placement**: the `+2`/`+3` field loads sit at `.text+0x1e0`, `+0x240` and `+0x274`, and **`+0x274` is placed AFTER the epilogue's `b __restgprlr_29`**. The port's `plan_text_order` has never had to place a block after the return | NEW |
| **M18** | the 4-way whitespace test `ch == 9 \|\| 13 \|\| 32 \|\| 0x3000`, appearing **five** times with five different joins | NEW |
| **M19** | `sub 11,5,4 ; rlwinm 11,11,0,0,30 ; cmpwi 6,11,2` — a pointer difference in bytes, masked even, compared signed | NEW |
| **M20** | negative displacements off a formal (`lhz 11,-4(5)`, `lhz 11,-2(5)`) | NEW |
| **M21** | the register plan itself over ~40 basic blocks and 3 callee-saved registers — `r31` = `ch`, `r30` = table base, `r29` = option, `r5` = a copy of `cur` made in the prologue (`mr 5,3`) so `r3` is free for the two calls | NEW |

### 2.4 The price, stated

> **`src/system/rndobj/wordwrap.cpp` costs ≥ 21 named mechanisms across three
> bodies — 3 + 6 + 12 — of which 18 are NEW and 12 are in one 160-word body
> whose block plan places a basic block after the epilogue. All three must
> convert together; the TU also owes a 588-byte `.bss` under Rule S1 (NC-2)
> and the `unclaimed-gl-symbol` accounting arm for both data symbols.**

For scale: the demonstrated rate is *one* one-function transcription per lane
at reach 1 (`CEILING.md` §10.30), and the largest single body ever transcribed
is 29 words. **M10–M21 is a lane of its own at least twice over.**

---

## 3. `CEILING.md` §11.4, RUN OFF THIS LANE'S OWN CAPTURE

* **Item 1 — the BYTE judge.** `fnbyte-denominator 3 · exact 0 · differs 0 ·
  refused 3`, all `fnbyte-decline|parse`. `bytefrac-exact 0/816`.
* **Item 2 — T1?** No. `fnbyte-exact` is **0**, not the denominator. The
  blocker IS codegen here.
* **Item 3 — the SYMBOL TABLE.** `work/w-wordwrap/ref/wordwrap.dump`: 9
  sections, **31 symbols**, 2,494 B. Two `.XBLD$W` COMDATs (`__C2_11886`,
  `__C1_11886`), a 588-byte `.bss` carrying `?g_uOption@@3IA` at **val 0x248**
  and `?g_LineBreakTable@@3PAULineBreakEntry@@A` at **val 0x0** — so the option
  word is placed *after* the 584-byte table, which Rule S1's slot ordering has
  to reproduce — three separate `.text` COMDATs, one `.pdata`, `$M2666`/
  `$M2667`/`$T2668`, and `__savegprlr_29`/`__restgprlr_29` after the `.pdata`
  group. **No `_fltused`, no `__real@` pool.**
* **Item 4 — LIST MEMBERSHIP?** No hex type tag in any of the three keys
  (`expr-jump`, `expr-brfalse`, `expr-cmp-eq`). Not NC-3.
* **Item 5 — do not trust the key's LAYER.** All three are fall-throughs.
  `expr-jump` is reported on a body with **no jump** (#2387, hand-checked on
  this exact function); `expr-brfalse` is reported at the `&`-then-branch of a
  guard; `expr-cmp-eq` is reported on the **first statement** of a 160-word
  body, which tells a lane nothing about the other 159 words. The keys were
  confirmed against the census windows above rather than quoted.
* **Item 6 — factor A.** Inside `A∧B∧C`; a frontier member. Item confirmed
  rather than assumed: `match 23 + frontier 4 = A∧B∧C 27`.
* **Item 7 — the board, grepped before sizing.** #2625, #2626, #2620, #2387,
  #1685, #2005, #1465, #1315/#1316, #807, #1981, #1638, #232, #1416, #2700.
  **#807 is the one that matters and no forward doc repeats it**: a lane
  reading this TU as *"just needs `cflow-if-n`"* is wrong — `cfg_reach` returns
  `NeedsClass` **before** it checks `classified < blocked_total`, and this TU
  has both, so teaching it `cflow-if-n` leaves it `Unclassified`.
* **Item 8 — the GATE's number.** §2's opening paragraph. It binds; `fn_names`
  is not the field and is 5 against `fn_total` 3.
* **Item 9 — the port's FENCES, before its obligations.** Checked even though
  T1 does not fire, because M12 depends on it: `comdat::fenced_inlined_callee`
  tests a same-TU callee against `INLINE_DECLINE_BYTES` = 128 and
  `?IsEastAsianChar` is **164** emitted bytes, so the fence permits the two
  `bl`s. `elide`'s mechanism E and `splice`'s S7 do not apply to a TU with no
  accepted body. **NC-5 is not in this TU's way**, and that is a measurement
  rather than an assumption.

---

## 4. WHAT THIS LANE WILL ATTEMPT, IN ORDER

1. `?WordWrap_SetOption` — M1/M2/M3. The smallest unconverted body on the whole
   frontier, worth **+1 `fnbyte-exact`** and **0 conversions** (#2625).
2. `?IsEastAsianChar` — M4…M9, if 1 lands with time left.
3. `?WordWrap_CanBreakLineAt` — M10…M21. Priced, and expected to be declined.

**Deliberate non-goals, declared now so they are not scored as omissions:**
widening any shipped byte-graded class (D7/#232); widening
`counted_accum_loop` (#1981 excludes a memory reference by name);
`gl_defined_names` (#2622/#2623: 0 conversions, −1 `fnbyte-exact`);
`INLINE_DECLINE_BYTES`; `scripts/gen_dc3_workload.sh` (#2700).

---

## 5. PREDICTIONS

Probabilities are for THIS lane at THIS base. Rows downstream of a conversion
this lane may never reach are marked **cond.** and are void if the antecedent
does not occur — `w-xtea2` §9.1's scoring lesson, which `w-xtea3` applied.

| # | prediction | p |
|---|---|---:|
| **P1** | `wordwrap.cpp` CONVERTS, TU match 23 → 24 | **0.07** |
| **P1a** | *cond. ¬P1* — the decline is published with N named mechanisms and N ≥ 15 | 0.85 |
| **P2** | `fnbyte-exact` delta ≥ **+1** | 0.75 |
| **P3** | `fnbyte-exact` delta is **exactly +1** (body 1 only) | 0.50 |
| **P4** | `?WordWrap_SetOption` converts (M1–M3 land) | 0.75 |
| **P5** | `?IsEastAsianChar` converts (M4–M9 land) | 0.20 |
| **P6** | `?WordWrap_CanBreakLineAt` converts (M10–M21 land) | 0.03 |
| **P7** | nothing regresses: `fnbyte-differs` unchanged, no key moves the wrong way | 0.95 (**unlosable** — flagged) |
| **P8** | `mismatch` **0** on every gate row, both modes, both binaries | 0.90 (**unlosable** — flagged) |
| **P9** | the 878 verdict set moves for `wordwrap.cpp` **and nothing else**, keyed on the FULL path | 0.85 |
| **P10** | *cond. P4* — M1 is a NEW production and no shipped byte-graded class is widened to reach it | 0.85 |
| **P11** | *cond. P4* — the `.bss` data symbol needs a `data_syms` arm, and without it the TU reports `unclaimed-gl-symbol` even with the body accepted | 0.60 |
| **P12** | *cond. P4* — `?WordWrap_SetOption` is byte-identical at `/O1` and `/Ox`, so the new class needs **no** mode gate | 0.45 |
| **P13** | `hatch-red` still REFUSES, pre-existing, same arms, reproduced at this lane's exact base | 0.85 |
| **P14** | the label channel is NOT touched: no `label_lead` / `label_slots` / `plan_labels` edit ships, because the only body that owns a `$M` triple is body 3 | 0.80 |
| **P15** | *cond. P4* — no new integration-test FILE; new tests are `#[cfg(test)]` modules in the new source file, so `cargo test` target count is unchanged | 0.75 |

### 5.1 THE ROW THAT COULD ACTUALLY DECIDE THE OUTCOME

`w-xtea3` went 16 for 16 and called it a calibration failure because the row
that decided its lane — *which fence would block last* — was never registered.
The equivalent row here is **P11**, and it is registered as a prediction rather
than a note: this TU's `gate_causes` already carries a **second** clause,
`unclaimed-gl-symbol`, and every mechanism in §2 is about `.text`. If body 1
converts and the TU's two `.bss` names are not claimed, the lane will have
bought a byte and moved no verdict — the same shape as `w-blockir`'s
`_fltused`, one field over. **The failure mode this lane is most likely to
walk into is buying `fnbyte-exact` and calling it progress**, which
`CEILING.md` §10.2 measured at +444 emitted and +0 goal.

**P1 is priced at 0.07 and that is not modesty.** §2.3 is twelve NEW
mechanisms in one 160-word body, against a demonstrated rate of one
29-word transcription per lane. A lane that priced this at 0.3 would be
pricing `w-xtea3`'s execution-of-a-completed-measurement, and no
measurement of body 3 exists — this file is the first one.
