# w-xtea3 — PREREG, frozen before the first `crates/` change

Lane `w-xtea3`, 2026-08-09. Branch `worktree-agent-a3d57ce1e0ac7115d`, base
`299f9a8c` (master tip, the `w-xtea2` merge). **Nothing under `crates/` is
modified at the moment this file is committed** — checked with `git status` and
recorded in the commit that adds it.

Commission: finish `src/system/utl/EncryptXTEA.cpp`, TU match **22 → 23**.
`w-xtea2` took it from four blocked bodies to three; the remaining three are
`?SetNonce` (32 B), `?Encipher` (116 B) and `?Encrypt` (96 B), plus a label
channel the port is **six slots** short on.

---

## 0. WORKLOAD STAMP (#2392 — dc3 is not pinned)

```text
c2-rs        299f9a8c2ef31e27bccce3b81d360ed1173b37e5
             worktree .claude/worktrees/agent-a3d57ce1e0ac7115d
             branch worktree-agent-a3d57ce1e0ac7115d
base binary  work/w-xtea3/c2rs-base   binary_sha 885099866f097b2537ec87c77877c11c
             built at the merge base and KEPT; every "base" column is its run
dc3-decomp   29802aa3fe00061337df11dfee95eaa201964821   2026-08-09T19:59:56Z
             878 TUs, workload_dirty false as the scan read it
             **dc3 has MOVED since `w-xtea2` (d7a3c1aa -> 29802aa3)** and the base
             scan reproduces w-xtea2's tip table digit for digit anyway
cl.exe       compilers/X360/16.00.11886.00/cl.exe
c2.dll       compilers/X360/16.00.11886.00/c2.dll
c1xx.dll     compilers/X360/16.00.11886.00/c1xx.dll
wibo         wibo 1.2.0-c2rs.1 (Linux x86_64), known-good 1.0.1-23
flags        /nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc + 8 /I roots
capture cache /home/…/c2-rs/work/capture-cache, context 18cf81cd95835a250ba5b88c4db5eb6a
```

Base scan, this lane's own run (`work/w-xtea3/base_metrics.txt`):

```text
match 22 · mismatch 0 · codegen-gap 0 · vocab-gap 849 · port-error 0
capture-fail 7 · frontier 5
fnbyte-exact 35799 · fnbyte-differs 1898 · fnbyte-refused 114633
```

Every digit agrees with `w-xtea2`'s tip table, so the base is that tree and
nothing is inherited unchecked.

### 0.1 A workload-generator defect found while stamping, and NOT a lane result

`scripts/gen_dc3_workload.sh` maps the original `e:/lazer_build_gmc1` include
roots onto the local tree with `path.startswith("e:/lazer_build_gmc1/…")` —
**forward slashes**. Today's `dc3-decomp` (`29802aa3`) emits those roots with
**backslashes**, so the mapping matches nothing, and the generated `flags.txt`
carries eight unmapped `e:\…` roots. The first scan taken with it read
**capture-fail 851 / match 15** — every TU failing `C1083`. The committed
`work/dc3-workload/flags.txt` in the main checkout is the mapped one and is what
this lane uses. Recorded here **before** any prediction, so that it cannot be
scored as a finding, and filed on the board.

---

## 1. DECLARED PRIORS — measured BEFORE this file froze, and therefore NOT scored

Read-only measurements taken during orientation. Declared so nothing below can
be scored as a prediction that was already known (`w-pool` #2571's defect).

**A. `CEILING.md` §11.4 item 8 — the GATE binds, off this lane's own scan row.**
`gate_cause "body-out-of-class"`, `gate_causes ["body-out-of-class"]` — no
`gl-stop-*`, no `bind-*` — with `emit-bound 5 == emit-gate-segments 5 ==
emit-record-offsets 5 == emit-records 5`. `fn_names` reads **4** against
`fn_total 5`, which is exactly the field #2621 warns is *not* the answer.

**B. Item 1 — the BYTE judge.** `fnbyte-denominator 5 · fnbyte-exact 2 ·
fnbyte-differs 0 · fnbyte-refused 3`, all three `fnbyte-decline|parse`.
`bytefrac-exact 28 / 272` (10.3 %). The remaining distance is 3 bodies / 244 B.

**C. Item 3 — the SYMBOL TABLE.** `work/w-xtea3/ref/xtea.dump`, re-captured at
the workload's own flags on today's dc3 and byte-identical to `w-xtea2`'s: 10
sections, 34 symbols, 1,942 B. `memcpy` at index 17 (callee region), the
`__restgprlr_26`/`__savegprlr_26` pair at 32/33 after the `.pdata` group, and
the label triple `$M2757` (idx 27, `.text+0x60`) · `$M2756` (idx 28,
`.text+0x0c`) · `$T2758` (idx 31). No `_fltused`, no `__real@` pool.

**D. Item 4 — is the refusal LIST MEMBERSHIP?** The three keys are
`expr-load-type-8882` ×1 (a hex type tag — **NC-3-shaped**) and `expr-op-0x27`
×2. `fn_prod` reports `tail-recv-not-a-plain-b9-load/b9-not-a-ptr4` and
`…/then-off-add`.

**E. Item 6 — factor A.** Inside `A∧B∧C`; a frontier member, so codegen alone
can convert it.

**F. Item 7 — the board, grepped before sizing.** #2344 (no 64-bit rotate/mask
encoder anywhere in `c2-core`), #2567 (`stdu`/`stdx` missing, `encode_addic`
present, no record form), #2661/#2662 (the label charge, measured), #2663 (the
minted external's placement is the user's frame class), #1980/#1981 (the
counted-loop class excludes a memory reference BY NAME), #746/#747
(`label_slots`' `None` for every loop class and the fixture that grades it),
#1638 (mode gates in the parser), #232 (do not widen a shipped byte-graded
class).

**G. The label arithmetic, inherited from `w-xtea2` §4.3 and NOT re-measured.**
`2721 + 9 + 3·5 = 2745`, plus strides `1 + 2 + 1 + 3 + 4 = 11`, equals
`$M2756` with zero residual. `plan_labels` charges `1+2+1+1+0 = 5`. The six
missing slots are `+2` on `?Encipher`'s leaf loop and `+4` before `?Encrypt`'s
own triple.

**H. The five encoders, enumerated off the reference obj's own words.**
`?SetNonce` needs `rldicl` (`clrldi 11,5,32`); `?Encipher` needs `rldicl`
(`clrldi 3,10,32`) and `rldimi` (`7923000e`); `?Encrypt` needs `addic.`
(`37bdffff`), `stdx` (`7d7af92a`) and `stdu` (`f97e0009`). Everything else in
all three bodies — `ld`, `std`, `add`, `xor`, `rlwinm`, `lwzx`, `mtctr`,
`bdnz`, `addis`, `addi`, `mr`, `subf`, `stwu`, `bc` — has an encoder today.

**I. The frame arithmetic for `?Encrypt`, from `FrameLayout`'s own rule.**
`saved_gprs 6` (r26–r31), `out_slots` floored at 8, `locals 0` →
`align16(80 + 8·7) = 144`, which is the obj's `stwu 1,-144(1)`. `32 − 6 = 26`,
which is `__savegprlr_26`. No term is fitted.

---

## 2. PREDICTIONS — the scored list

**Every row that is a claim about work downstream of a conversion is declared
CONDITIONAL here**, which is `w-xtea2` §9.1's own scoring lesson: four of its
misses were one mistake, because it registered confident claims about rungs it
never reached. A conditional row is scored only if its antecedent occurred, and
is recorded UNGRADED otherwise rather than banked.

| # | prediction | p |
|---|---|---:|
| **P1** | **`EncryptXTEA.cpp` CONVERTS — TU match 22 → 23** | **0.30** |
| P1a | if it does not convert, the decline is priced at **N ≤ 6** named mechanisms (against `w-xtea2`'s `3 bodies / ≥ 3 encoders / one three-layer label change`) | 0.60 |
| **P2** | **`fnbyte-exact` delta = +3** — all three remaining bodies and no other function moves | 0.30 |
| P2a | `fnbyte-exact` delta ≥ +1 | 0.80 |
| P2b | `fnbyte-exact` delta ≥ 0 — nothing regresses. **UNLOSABLE if this lane ships nothing; flagged, not counted** | 0.95 |
| P3 | **`?SetNonce` converts** — 32 B, 8 words, byte-exact | 0.70 |
| P4 | **`?Encrypt` converts** — 96 B, 24 words, byte-exact | 0.40 |
| P5 | **`?Encipher` converts** — 116 B, 29 words, byte-exact | 0.40 |
| P6 | *CONDITIONAL on P3* — `?SetNonce` needs exactly **one** new encoder (`rldicl`) and no other | 0.80 |
| P7 | *CONDITIONAL on P4* — `?Encrypt` needs exactly **three** new encoders (`addic.`, `stdx`, `stdu`) and no other | 0.75 |
| P8 | *CONDITIONAL on P5* — `?Encipher` needs exactly **two** new encoders (`rldicl`, `rldimi`) and no other; its two `rlwinm` fusions are covered by the existing `encode_rlwinm` | 0.80 |
| P9 | *CONDITIONAL on P4 ∨ P5* — the six label slots are paid **WITHOUT touching `coff::plan_labels`**, because `plan_labels` already applies `cur += f.label_lead` before each function's triple; the change is `IlFunction::label_lead` + `label_slots` + the `PortC2` wiring that carries it onto `coff::Function` | 0.60 |
| P10 | *CONDITIONAL on P4 ∨ P5* — `IlBundle::functions`' gate `label_slots(false)? != label_lead() + 1` needs **no** edit, because a leaf whose lead is 2 already satisfies it | 0.65 |
| P11 | the new classes must **refuse at `/Ox`** — a mode gate in the PARSER (#1638) | 0.90 |
| P12 | **`mismatch` 0 on every gate row**, both modes, `expr_sweep` and `mode_cross` included. **UNLOSABLE if this lane ships nothing; flagged** | 0.90 |
| P13 | the 878-TU verdict set moves for **`src/system/utl/EncryptXTEA.cpp` and nothing else** — 0 other TUs move in either direction, keyed on the FULL PATH | 0.85 |
| P14 | test-count DELTA: **targets 41 → 41** (no new integration-test FILE), test count strictly up | 0.75 |
| P15 | `hatch-red` still **REFUSES** at this lane's tip, with the same five arms `w-xtea2` reproduced at `af81b869` (`R2 DIRTY+HATCH, R6 RESIDUE, A2 PAID-MISSING, F1 FORCE, C1 HATCH-ONLY`) — pre-existing, #1389 | 0.85 |
| P16 | `?SetNonce`'s `clrldi 11,5,32` is **CSE'd across both statements** — emitted once, and the second `add` reads r11 rather than recomputing — so a per-statement lowering is one word long | 0.85 |

### 2.1 The falsifiability note

**P2b and P12 are unlosable if this lane ships nothing**, and P13 is nearly so.
They are registered because they are the gate's own obligations and are flagged
here so hitting them is not counted as calibration. **The losable rows are P1,
P1a, P2, P2a, P3, P4, P5, P11, P13, P14, P15 and P16** — twelve — plus the four
conditionals P6–P10, which are losable only if their antecedents occur.

---

## 3. DECLINE CLAUSES, each with a size

| # | clause | size |
|---|---|---|
| **D1** | If `?Encipher`'s 29 words cannot be transcribed byte-exact, DECLINE the TU rather than approximate the schedule. | 116 B, 29 words |
| **D2** | If the label charge cannot be paid without giving `label_slots` a **mode parameter** or a **sub-shape parameter**, DECLINE. `LABEL_COUNTER.md` §7.6's box forbids both. A class may ship a measured constant obtained by steps 1–5 and nothing else. | 6 slots |
| **D3** | Do **not** widen `gl_defined_names` / `bind::defined_name_set`. #2622/#2623 measured that repair at 0 conversions and **−1 `fnbyte-exact`**. | −1 `fnbyte-exact` |
| **D4** | Do **not** relax `LabelMap`'s invariant 4. Every shipped loop class computes its back edge through `encode_bc`/`encode_bdnz` directly. | 0 lines in `labels.rs` |
| **D5** | If a fixture cell cannot compile at `/Ox /GS- /c`, declare a `// c2rs-profile:` marker with a reason (#2330–#2335) rather than dropping the cell. | per cell |
| **D6** | Do **not** widen `counted_accum_loop` to admit a memory reference. #1981 defines that class to contain none. A new class is a new class. | 0 lines in `counted_accum_loop` |
| **D7** | Do **not** widen `memcpy_tail`, `store_run_call` or any other shipped byte-graded class to admit a body it does not emit (#232). | 0 lines |
| **D8** | Two **unnamed refusals** are budgeted across three bodies. If a third appears, stop and price rather than pay it. | 2 |
| **D9** | Every `_neg` cell is **one refusing body per TU** (a multi-cell file can never go `mismatch` — a TU verdict is a conjunction), is **compiled first and claimed second**, and every fence is proved by a must-fail mutation. An over-fenced cell is repaired by MERGING clauses, not by adding cells (#2664–#2666). | per cell |

---

## 4. BOARD CHECK, PRE-ARMED

Rows **#2690–#2719** are this lane's. Before sizing any rung, `grep BOARD.md`
for the key. Any row this lane does not mint is declared UNMINTED in the rung.

---

## 5. WHAT THIS LANE WILL NOT CLAIM

* It will not treat a census gain as a goal gain (`CEILING.md` §10.2).
* It will not quote `mismatch 0` as evidence of correctness (trap 1).
* It will not re-derive `w-xtea2`'s label arithmetic as its own finding — prior
  G is declared, not predicted.
* It will not report a `/O1`-only fixture scan: the `/Ox` half is mandatory
  (`w-biquad` shipped a live wrong emit that the `/O1`-only workload scan, the
  `/O1` fixture lane and every workspace test missed).
* It will not key 878-TU neutrality on the basename: 878 TUs collapse to 841
  basenames and a collapsed comparison drops 37 rows while printing `0 MOVED`.
