# W-MAIN2 — PREREG, frozen before the first `crates/` change

    Lane:   w-main2
    Branch: w-main2
    Base:   adfc2a78 (master tip at lane start)
    Rows:   #2970–#2999
    Date:   2026-08-10

**The freeze point, stated honestly.** Everything in §1 is read-only: the base
878-TU scan, the reference obj dump, the IL capture and a hand-parse of `.sy`,
`.gl` and `.db`. No `crates/` file has been opened for writing. Every prediction
below was written with §1 in hand and nothing more.

---

## 0. Stamps

| | |
|---|---|
| c2-rs base | `adfc2a78` |
| dc3 workload | `104e7df9c`, clean — the same tree the last five lanes stamped |
| workload list | `work/dc3-workload/files.txt` md5 `09189d4a41713c77e14dca9af5050b58`, 878 lines, **committed, never regenerated** |
| workload flags | `flags.txt` md5 `ef3b32e8ac8d3ab89a8be0a0a60e40c8` = `/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc …` |
| base binary | `work/w-main2/c2rs-base`, sha `ed7db9deadc8f73299be20baab5a239a`, **KEPT** |
| base scan | `work/w-main2/gap-base.log` / `base.jsonl`, `--jobs 24` |

---

## 1. Re-derivation at base — read-only, before the freeze

### 1.1 The scan

| metric | base |
|---|---:|
| `match` | **23** |
| `mismatch` | **0** |
| `frontier` | 4 |
| `fnbyte-exact` | **35810** |
| `fnbyte-differs` | 1898 |
| `factor-a/b/c/d/e` | 28 / 338 / 169 / 23 / 2 |

### 1.2 `docs/CEILING.md` §11.4, item by item, on `src/Main.cpp`

1. **The BYTE judge.** `fnbyte-refused 1`, `fnbyte-refused-parse 1`,
   `bytefrac-exact 0 / 124`, `bytefrac-accepted 0`. **T1 does NOT fire** — this
   is not `w-xtea3`'s shape. The one body is behind the reader, so codegen *is*
   part of the price.
2. Item 2 is therefore not reached: the blocker is codegen **and** emitter.
3. **The reference obj's SYMBOL TABLE, read** (`work/w-main2/ref/main.dump`,
   36 symbols, 8 sections, 1,814 B). Symbols with **no body**, i.e. obligations
   no per-function byte test can see: `__CxxFrameHandler` (undefined external),
   `__ehfuncinfo$main`, `__unwindtable$main`, `$T2592` (all STATIC in `.rdata`),
   `$T2596` / `$T2599` (STATIC, one per `.pdata`), `$M2590 $M2591 $M2594 $M2595
   $M2597 $M2598` (class 6 LABEL in `.text`), `__unwind$2585` (STATIC in
   `.text`, `Value=0x54` — **a second code region**). **Ten** such symbols.
   No `_fltused`, no `__real@`, no `__savegprlr_N`: the forecast is EH and
   nothing else.
4. **Is the refusal LIST MEMBERSHIP?** Yes, and #764's marker fires: the key
   `expr-call-in-expr-recv-object-then-op-0x5C` ends in a hex opcode tag. It is
   `mcall::eat_dtor_stmt_trailer`'s `eat_int_like` TYPE gate, already priced
   live by the **shipped** counterfactual `C2RS_SINK_MCALL_TRAILER=varint`
   (w-main §2.3), which moves the head one rung to `op-0x5E`.
5. **Confirm the key against the body.** The `.ex` segment at offset 2713
   carries `… 4C 5C A6 43 81 20 01 4B 4F 01 05 …` at segment offset 0x9a and
   `… 4F 01 06 5E 01 21 4B 54 02 29 …` at 0xc3. Both opcodes are present at the
   byte level; the key names the layer it is on. **And the workload population
   is grepped, not assumed**: w-main §2.3 counted the `5C` family at 1,213
   bodies / 198 emitted / 811 TUs — this is not a one-instance class.
6. **Factor A.** `src/Main.cpp` is in `A∧B∧C`, is in the printed **FRONTIER**
   (4 TUs), and the CFG screen reads `REACHABLE`. Item 6's warning does not
   fire: no reader or section work is needed before codegen can convert it.
   (#828 stands: the CFG screen is blind to funclets, so REACHABLE is not
   *cheap*.)
7. **The board.** #1865 (wb-eh's fifteen), #2263 (w-main's thirteen), #2629,
   #2760 (w-decouple's fourteen), #2621/#2622/#2623 (w-front5's binding
   counterfactual, now superseded by #2750/#2751), #828, #2265 (the label
   lead), #2266 (the EH axis). No row records this route as having measured
   zero; the two declines are declines on **price**, not on effect.
8. **`gate_cause`, and nothing else.** `gate_cause = body-out-of-class`;
   `gate_causes = [body-out-of-class, unclaimed-gl-symbol]`. **The binding is
   PAID** (`w-decouple` #2750/#2751) and is not re-derived here.
   `gl_body_starts` reads `1 of 1`; `selective_bind` reads `(1, 1, 3, 0)` and
   the three unclaimed mangled runs are `??0App@@QAA@HPAPAD@Z`,
   `??1App@@QAA@XZ`, `?Run@App@@QAAXXZ` — **undefined externals `main` calls**,
   which discharge with the body (#2760). Not re-paid.
9. **NC-5, both directions.** T1 does not fire, so the "in front of byte-exact
   bodies" instance cannot apply. The **behind-an-unwritten-body** instance
   (`w-decouple` #2756) is checked and does **not** fire either: all three
   callees are undefined externals, so `comdat::fenced_inlined_callee`,
   `elide`'s mechanism E and `splice`'s S7 have no locally-defined callee to
   fence. **This TU owes no fence exemption.**

### 1.3 The obj, read to the byte (composed, not re-derived)

`.text` 124 B, one COMDAT, 6 relocations, **two code regions**: `main` at
`0x08..0x54` (`Value = 0x8`) behind an 8-byte `{__CxxFrameHandler,
__ehfuncinfo$main}` ADDR32 prefix, and `__unwind$2585` at `0x54..0x7C`.
Two `.pdata` COMDATs (`Number=5 Sel=5`) **in reverse region order**: the
funclet's first (`begin addend 0x4c`, unwind `0x40000a04`), the body's second
(`begin addend 0`, unwind `0xc0001305`). A 64-byte `.rdata` (`Number=5 Sel=5`)
with five relocations: `__unwindtable$main` at +0x00, `__ehfuncinfo$main` at
+0x08, `$T2592` at +0x30.

The frame arithmetic, checked against `codegen::frame`: `sizeof(App) = 4`
(`.db` `LF_CLASS` size field), `saved_gprs = 1` (r31), `out_slots = 0` ⇒
`locals_base = 80`, `size = align16(80 + 4 + 8 + 8) = 112`. Both match the
reference's `addi 31,1,-112` / `stwu 1,-112(1)` and its `addi 3,31,80`.

`.gl` label counter = **2575** (`gl[7..11]` LE32).

---

## 2. PREREG

### 2.1 Route and mechanism (P1–P5)

| # | p | prediction |
|---|---|---|
| **P1** | 0.90 | The route is a **whole-TU recognizer** (factor **E**, the `dyninit_tu` precedent), not a widening of `IlBundle::functions()` / `codegen::select_function`. `Selected` is one-body-one-plan and a funclet is a second code region; `plan_labels` mints three labels for a framed function and this obj carries **ten**. |
| **P2** | 0.85 | **The frame is already PAID.** `FrameLayout { locals: 4, out_slots: 0, saved_gprs: 1, saved_fprs: 0 }` reproduces the reference's `stwu 1,-112(1)` and `addi 3,31,80` with **no new frame rule** — `wb-frame`'s `align16(80 + locals + 8 + 8·saved)` is exact here. |
| **P3** | 0.80 | The object's **size is in neither `.ex` nor `.sy`** and must come from `.db`'s CodeView type stream (`LF_CLASS` reached through the `.sy` local's type token `0x100a`) — a reader seam that **does not exist** in `c2-il` today. |
| **P4** | 0.75 | `__ehfuncinfo$` is **TEN** dwords (40 B), not the **nine** `WB_EH_FINDINGS` §3.1 and board **#1869** record. The obj's arithmetic forces it: 8 (`__unwindtable$`) + N + 16 (ip2state) = 64, and `$T2592` sits at +0x30 with `__ehfuncinfo$main` at +0x08. |
| **P5** | 0.70 | The EH-main label allocation is **affine in the `.gl` seed** and the other nine labels sit at **fixed offsets from `main`'s own `$M`**: with `n = $M(main)`, the obj gives `__unwind$` at `n−9`, the two ip2state `$M` at `n−4` / `n−3`, `$T`(ip2state) at `n−2`, `$T`(main's `.pdata`) at `n+2`, and the funclet's `$M`/`$M`/`$T` at `n+3` / `n+4` / `n+5`; and `n = seed + 9 + 3·funcs + 7`. A probe cell that moves **only** the seed moves all ten by the same delta. |

### 2.2 What ships (P6–P7, P11–P12)

| # | p | prediction |
|---|---|---|
| **P6** | 0.95 | `mismatch` **0** everywhere; `scripts/gate.sh` 18/18 PASS with `expr_sweep` and `mode_cross` **unsampled**; `c2rs selftest` green. |
| **P7** | **0.90** | **`fnbyte-exact` delta = 0** (35,810 → 35,810). **REGISTERED AS THE SCORED METRIC** per CEILING §10. A whole-TU recognizer is invisible to FUNCTION BYTE MATCH by construction — the byte-fraction control already prints the two `??__E` TUs at `0/24 bytes, factor E, EXPECTED`. **FALSIFIER, written down because this row looks unlosable:** if the route turns out to need `IlBundle::functions()` or `comdat::comdat_function_body` widened so that `main`'s body is graded per function, `fnbyte-exact` moves by ±1 and P7 is lost. That is exactly P1's negation, so P1 and P7 stand or fall together and are **not** independent evidence. |
| **P11** | 0.80 | Conditional on P8: `factor-d` 23 → 23 and `factor-e` **2 → 3**; `frontier` **4 → 3**; `factor-a/b/c` unchanged. Declared conditional. |
| **P12** | 0.80 | Workload **census** delta is exactly **0** functions — the per-function path still refuses `main`'s body, whichever route ships. |

### 2.3 The conversion call — mutually exclusive (P8–P10)

| # | p | call |
|---|---|---|
| **P8** | **0.40** | **`src/Main.cpp` CONVERTS: TU match 23 → 24.** |
| **P9** | **0.55** | **A priced decline**, script-counted, with **N ≤ 6** of the fourteen still unpaid — i.e. most of the chain is paid and the residue is named. |
| **P10** | 0.05 | Neither: the lane ships nothing and cannot re-derive the chain. |

### 2.4 Decline clauses, with sizes

| # | clause | size if it fires |
|---|---|---|
| **D1** | Any existing verdict moves: a TU's `fn_in_class` **falls**, `mismatch` > 0, or any of the 261 `gap-metric` keys moves other than the ones P11 names. **Commit first, then revert** (#1380). | the whole ship |
| **D2** | The label model cannot be separated from its seed by a probe cell (P5's counterfactual) → **the recognizer does not ship**, because a fitted-to-one-witness counter is a wrong-bytes obj waiting for the second TU. Reported as *not separated*, never folded into support. | the recognizer |
| **D3** | `.db`'s class size cannot be read fail-closed → the recognizer refuses on every TU and ships as a refusal + the named seam. | the size reader |
| **D4** | The `_neg` cells cannot be given **distinct probe-verified clause keys** → the fixture pair is not landed. A multi-cell `_neg` file can never go `mismatch`; an over-fenced cell grades none of its clauses and the repair is **merging**; a merged clause's must-fail mutation must delete the whole conjunction (#2698/#2699). Cells that grade nothing are **named, not counted**. | 2 fixtures |
| **D5** | The generated corpus (`expr_sweep`, `mode_cross`) samples rather than runs whole → the gate claim is not made. A clean fixture scan is **not sufficient**: 35 wrong objs read 0 mismatch at both `/O1` and `/Ox`. | the gate claim |

### 2.5 Pre-armed instrument — the unnamed refusal

The board's streak is on **fence order / clause reachability**. Pre-armed here
against a different one, because this lane adds a **whole-TU** arm *before*
`functions()` in `PortC2::build`: an arm placed there runs on **all 878 TUs**,
so its cheap early-out is a correctness gate as much as a performance one
(`dyninit_tu`'s own doc says so). The instrument is the **878-TU four-level
neutrality diff with directions**, plus a `_neg` cell whose key must differ
from the positive cell's. **One unnamed refusal is budgeted.**
