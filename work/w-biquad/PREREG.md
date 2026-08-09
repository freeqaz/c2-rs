# w-biquad — PREREG

**Frozen before the first change to `crates/` and before the first fixture this
lane authors.** Everything below is either (a) re-derived by a command in this
tree at the base commit, or (b) a prediction that §8 will score. Nothing is
quoted forward from a rung, a board row or the commission without being
re-measured here — nine inherited prices were wrong the week before this lane.

    Lane:   w-biquad, worktree branch `w-biquad`
    Base:   master `111b63576fb20e9f06dedc2e75922231e72d7d4d`
            ("docs: regenerate the STATUS block — TU MATCH 20")
    Board:  #2530–#2559
    Target: `src/system/synth_xbox/Biquad.cpp` — 2 functions, 176 `.text` bytes

---

## 0. Workload stamp — the numbers below are only as pinned as this

The dc3 tree is **not** pinned by this repo (#2392) and two lanes have quoted
figures that had already moved (#2360). So the stamp is stated in full and every
base number in §1 comes from `work/w-biquad/base.out`, produced by
`work/w-biquad/scan.sh base` with `BIN=work/w-biquad/c2rs.base` — a binary
**built at the merge-base and KEPT**, never `git checkout master -- crates/`
(#2409).

| | |
|---|---|
| c2-rs base commit | `111b6357` (master, merge-base) |
| dc3-decomp commit | `d7a3c1aa9d5d57a1176790c0e15a723edd2e03a0`, 2026-08-09T13:09:42Z |
| dc3-decomp worktree | 2 untracked paths (`-.cache`, `work/`), **0 tracked modifications** |
| workload list | `work/dc3-workload/files.txt`, **878** lines (`wc -l`) |
| workload flags | `/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc` + 8 `/I` roots |
| toolchain | `compilers/X360`, cl 16.00.11886.00, wibo from `../../../../wibo/build` |
| base binary | `work/w-biquad/c2rs.base`, sha in §8 |

`/O1` **implies `/Gy`**, so every claim below is on the function-level-linking
path (`PortC2::fn_level_linking == true`) unless it says otherwise.

---

## 1. The base, re-derived

`work/w-biquad/base.out` — an 878-TU scan run before the first `crates/` line.

| | base `111b6357` |
|---|---:|
| **TU match** | **20** |
| mismatch · codegen-gap · port-error | 0 · 0 · 0 |
| vocab-gap · capture-fail | 851 · 7 |
| **FRONTIER** | **7** |
| factor A / B / C / D / E | 28 / 338 / 169 / 20 / 2 |
| `B and C` jointly | 151 |
| `A and B and C` | 27 |
| **`fnbyte-exact`** | **35,793** |
| `fnbyte-differs` | 1,898 |
| `fnbyte-denominator` | 162,092 |
| `fnbyte-match` | 0.22083 |
| `fnbyte-refused` | 114,649 |
| `fnbyte-unbound` | 9,220 |
| `writer-sections` | 10 |
| `Biquad.cpp` accepted `.text` bytes | **0 / 176** |
| `Biquad.cpp` blocked / emitted functions | **2 / 2** |
| workspace tests | measured in §8 |

**`Biquad.cpp` is in the FRONTIER**, which is defined as `A and B and C and not
(D or E)` — so factors **A** (emit set reachable) and **B** (binding complete)
and **C** (section shape) all hold for it *at base*. See §2.

### 1.1 The refusal ladder, re-derived at THIS tip

Run with the measurement-only sinks, which push no `IlOp` and cannot move an obj
byte. `w-park` measured this on 2026-08-08 at a different master; it has **not**
moved.

| sinks | `Biquad.cpp`'s two bodies |
|---|---|
| none | `expr-cmp-eq` ×1 (`?SetCoefficients`) · `expr-call-in-expr-recv-load-then-plumbing-0x3A` ×1 (`??0Biquad`) |
| `C2RS_SINK_REL=expr` | `expr-brfalse` ×1 · unchanged ×1 |
| `C2RS_SINK_REL=expr C2RS_SINK_BRANCH=stmt` | **`expr-op-0x27`** ×1 · unchanged ×1 |

---

## 2. The binding predicate, checked FIRST (CEILING §11 item 8, #2400–#2414)

`mmio.cpp` had one `.gl` name against eleven `.ex` segments because
`looks_mangled` requires `@@`, so every codegen mechanism priced for it would
have converted nothing. The check is made **before** any codegen is priced.

**Measured, `work/w-biquad/biquad_base.jsonl` and `base.out`:**

* `Biquad.cpp` has **2 `.ex` segments** and **2 emitted `.text` COMDATs**;
* the single-TU scan prints *"emit-set MODEL ceiling: **1 of 1 TUs bind every
  emitted symbol today**; 0 carry an emitted symbol with NO `.gl` body record
  and are a wall for any segment-driven model"*, and *"unbound emitted symbols:
  0 have a body record (instrument defect), **0 have none (wall)**"*;
* both names carry `@@` (`?SetCoefficients@Biquad@DSP@@QAAXPAM@Z`,
  `??0Biquad@DSP@@QAA@PAM@Z`), so `looks_mangled` is satisfied;
* factor **B** holds for this TU in the 878-TU scan.

> **P0 (registered).** `Bindings::per_record` binds **both** of `Biquad.cpp`'s
> emitted names, and the mmio trap does **not** fire here. If this is wrong the
> lane is over before it starts and the whole commission is void.

---

## 3. The whole-obj obligations, enumerated BEFORE codegen (CEILING §11.4)

`w-blockir` had four byte-exact bodies and a `mismatch` obj, one `_fltused`
short. `Biquad.cpp` is a float TU with pooled constants **and** a framed
function, so the list is longer. Read off the reference obj
(`work/w-biquad/real.obj`, `scripts/gt_dump.py`) rather than assumed:

```
   1 .drectve   2 .debug$S   3 .XBLD$W   4 .XBLD$W
   5 .text  (140 B, 8 rel)  ?SetCoefficients@Biquad@DSP@@QAAXPAM@Z
   6 .rdata (4 B)           __real@3f800000     chars 0x40301040 sel=2
   7 .rdata (4 B)           __real@00000000     chars 0x40301040 sel=2
   8 .text  (36 B, 1 rel)   ??0Biquad@DSP@@QAA@PAM@Z
   9 .pdata (8 B, 1 rel)    COMDAT, SELECT_ASSOCIATIVE -> section 8, real CheckSum
```

| # | obligation | status at base |
|---|---|---|
| O1 | two `.rdata` pool COMDATs | **`emit_comdat_obj` has NO `.rdata` at all**; `codegen::select::function_gate` refuses `Selected::Float { consts }` under `/Gy`, and `PortC2::build`'s `/Gy` arm passes `fp_refs: Vec::new()` |
| O2 | `_fltused` undefined external, after the FIRST float function's **complete** symbol group (index 20, i.e. after BOTH `.rdata` groups) | the field exists (`Function::is_float`); its interaction with pool groups on the `/Gy` path is unwritten |
| O3 | `.pdata` COMDAT associative to the ctor's `.text` | **shipped** (W-UNW-1) |
| O4 | the `$M2574` / `$M2575` / `$T2576` triple | `plan_labels` shipped; the **charge** of the new leaf class is unmeasured |
| O5 | a REL24 against a **locally defined** callee, no undefined external minted | shipped by `w-fence2`, gated on `plain_external_defined_names` |
| O6 | section ORDER `.text .rdata .rdata .text .pdata` — pools interleaved at the first function that needs them | `emit_obj` (packed) states the interleave rule in a comment and **cannot express it**; `emit_comdat_obj` has never seen a pool |

> **P1 (registered).** The `.rdata` pool SECTION ORDER is **REVERSE
> first-reference order**, not first-reference order as `emit_obj`'s comment
> claims. Three witnesses, all compiled by this lane before this line was
> written: `work/w-biquad/probe/pool1.cpp` (uses 2.5f then 7.5f → sections
> 7.5, 2.5), `pool2.cpp` (the same two constants, use order reversed → sections
> 2.5, 7.5), and `Biquad.cpp` itself (uses 0.0f then 1.0f → sections 1.0, 0.0).
> pool1 was additionally compiled at `/Ox`, where the order is the same, so the
> packed writer's documented rule is **latently wrong at n ≥ 2** — a claim §8
> must either confirm on an obj or withdraw.

---

## 4. What this lane will BUILD, and what it will NOT

The commission's instruction is *"take the smallest thing that converts the TU
and decline the rest by name with sizes"*. The designator layer is **already
implemented** in the reader — `shapes::designator::walk_offset_adds` consumes
`33 <int-like> k · 27 <PTR>` and `33 <int-like> k · 28 00 00`, sums them and
reports the last re-type. `expr-op-0x27` is raised by `parse_expr`'s
fall-through arm, i.e. it says *"no shape recognizer claimed this body"*, not
*"the offset add cannot be read"*.

> **This is the single most important correction this PREREG makes, and it is
> registered before any code**: the head of `Biquad.cpp`'s ladder is **not** a
> reader rung and **not** a designator-layer hole. `w-readpx` (`WB_READER_FINDINGS`
> §3.3) already priced `expr-op-0x27`'s grammar cost at **NONE** and ranked it a
> **lowering** with an UNKNOWABLE `fnbyte-exact` delta and **0 TUs**;
> `w-dclass` §6.1 measured its head worth **six functions and zero TUs**;
> `w-band` found it reads `NoSignal`. **The size of this key is not its worth,
> and this lane's target is a TU, not a family.** The family's measured worth at
> this tip is reported in §7 in constructs and stems — as context, never as a
> promise.

Planned ships (each may be dropped; §6's decline clauses price the drop):

* **S1** a body shape for `?SetCoefficients`: a two-armed `if`/`else` over a
  null-guarded pointer formal whose then-arm and join are pooled-constant float
  member stores and whose else-arm is a CSE'd division run;
* **S2** whatever `??0Biquad` needs — a framed same-TU call with **no argument
  setup**, a dead `mr r10,r3` park and a `this` return;
* **S3** `.rdata` pool sections on the `/Gy` `emit_comdat_obj` path, in the
  order P1 registers, with the symbol interleave and `_fltused` placement O2
  requires;
* **S4** fixtures + tests, positive and negative, at `/O1` **and** `/Ox`.

Explicitly **NOT** built, and named here so the decline is not retro-fitted:

* a general dominator computation for B-RULE. The class transcribes the
  two-pool case its own obj shows and refuses a third pool;
* **B-RULE-2**, the compare/branch separation slot, is `medium` at exactly 3
  witnesses (`WB_CHOOSER_FINDINGS` §3.3). This lane **ships without depending on
  it**: the entry block's word order is transcribed from `Biquad.cpp`'s own obj,
  and no clause consults a separation-slot rule. If a cell ever needs one, the
  lane declines instead of widening a 3-witness rule (#260);
* the float-constant materialisation chooser (`WB_CHOOSER_FINDINGS` §4.2, `lis`
  into a GPR inside a loop) — unmeasured, one obj, not gridded;
* `expr-op-0x28`'s width disagreement (`WB_READER_FINDINGS` §3.4): all 28
  witnesses read the literal `28 00 00`, which is exactly what
  `walk_offset_adds` already requires, so this lane changes nothing about it and
  claims nothing about it.

---

## 5. The predictions

Scored in §8. Probabilities are the registered form; a census-only prediction is
unscored (CEILING §10).

| # | prediction | form |
|---|---|---|
| **P0** | `Bindings::per_record` binds both emitted names; the mmio `looks_mangled` trap does not fire | **0.97** |
| **P1** | the `.rdata` pool section order is REVERSE first-reference | **0.90** |
| **P2** | **`Biquad.cpp` CONVERTS** — TU match **20 → 21** | **0.55** |
| **P2a** | conditional on P2, `fnbyte-exact` moves **exactly +2** (the TU's two functions and nothing else) | **0.80** |
| **P2b** | unconditionally, `fnbyte-exact` delta ∈ {0, +1, +2} and **`fnbyte-differs` does not rise** | **0.90** |
| **P3** | `?SetCoefficients`'s `label_lead` is **0** — `LABEL_COUNTER` §7.4 puts an `if/else` at lead 0 at `/O1`, and §7.6 step 6 says predict then confirm with ONE compile | **0.75** |
| **P4** | the whole-obj obligation that costs the most iterations is **O1/O6** (the pool sections on the `/Gy` path), not a `.text` byte | **0.65** |
| **P5** | **test-count DELTA: +4.** Five consecutive lanes over-estimated this in the same direction, so the registered number is deliberately low; §8 scores the sign of the error as well as its size | **point estimate** |
| **P6** | `mismatch` stays **0** everywhere — the 878-TU scan, all 18 gate lanes, the sweep and the cross | **0.93** |
| **P7** | the per-TU verdict SET over all 878 by name moves by **at most one TU, in the converting direction**: 0 only-in-base, 0 only-in-tip, ≤1 changed | **0.88** |
| **P8** | the family `expr-op-0x27` is worth **0 additional TUs** at this lane's tip beyond `Biquad.cpp` itself | **0.92** |

**P2's 0.55 is the honest number and the reasons are stated before the outcome
is known.** In favour: every ingredient is already in the tree (pooled `.rdata`
constants with REFHI/REFLO, `encode_bc`/`encode_b_intra`, `encode_fdiv`, the
associative `.pdata`, the same-TU REL24 `w-fence2` shipped last), the IL is
regular and fully decodable by readers that exist, and both bodies are
transcribable word for word off an obj this lane has already produced. Against:
the work spans **four** layers at once (reader, emitter, writer, label channel),
`emit_comdat_obj` has never emitted a `.rdata` and the pool/`_fltused`/label
interleave is three unmeasured orderings stacked, and `w-park` priced this TU at
fifteen and declined.

---

## 6. Decline clauses, with sizes

A priced decline is an acceptable outcome. It must be re-derived, script-counted
and named refusal by refusal, and it must say which of `w-park`'s fifteen are
paid.

* **D1** — if **P0** is false (the binding predicate does not hold), the lane
  stops at §2 and reports that, because no codegen mechanism can convert a TU
  whose symbols do not bind.
* **D2** — if the pool section order cannot be pinned to a rule with **≥3
  witnesses**, the lane ships nothing that writes a `.rdata` on the `/Gy` path
  and declines O1/O6 by name. A guessed section order is a wrong section count
  at file offset 2.
* **D3** — if `?SetCoefficients`'s `label_lead` cannot be measured by
  `LABEL_COUNTER` §7.6's in-the-middle procedure with `base` reading **5** under
  `/Gy`, `label_slots` returns **`None`** for the class (w-bdnz #1983's
  precedent), which refuses the TU rather than fitting a constant. The lane then
  declines and says so.
* **D4** — if, at the point where the emitter work would begin, the unpaid count
  across the two bodies is **≥ 8 distinct unbuilt mechanisms**, the lane spends
  its remainder making the decline countable rather than making it smaller
  (`w-park`'s D7, re-used by name).
* **D5** — one **unnamed** refusal is budgeted. A second unnamed refusal is a
  decline.
* **D6** — nothing ships that depends on **B-RULE-2** (3 witnesses, `medium`).
  If a cell forces it, the lane declines that cell rather than widening the
  rule.

---

## 7. Instruments that must be run, and their DIRECTIONS

Verdict neutrality at three levels, each compared as a **key→value map or a set
BY NAME**, never as a `diff` — a count hides one TU lost and one gained:

1. **878 TUs by name** — `work/w-biquad/verdicts.py`: only-in-base,
   only-in-tip, changed, each listed. Direction required: any change is toward
   `match`.
2. **every `gap-metric` key** — `work/w-biquad/keydiff.py`: vanished, appeared,
   changed, each listed with both values. Direction required: no key vanishes.
3. **all 331+ fixtures at `/O1` AND `/Ox`** — the list regenerated after this
   lane's last fixture and `wc -l`-checked, so a fixture added and not scanned
   cannot read as an unchanged count. Direction required: no fixture moves from
   `match`.

Plus: the full gate (18/18, **0 mismatch anywhere**),
`cargo test --workspace --release --no-fail-fast` (#2262), `scripts/board_audit.sh`,
the rung registry, and `c2rs selftest` green (331 PASS / 0 ERROR).

`hatch-red` currently REFUSES on a **pre-existing HATCH-DRIFT** in
`body/shapes/calls.rs` (board #1406). It will be reproduced at master with
`crates/` reverted **before** anything is attributed to this lane.

`_neg` cells have been inert or confounded in six of the last nine lanes. Every
`_neg` cell this lane ships is ordered so that every fence is live, and each is
proved with a **must-fail mutation** that is run, not reasoned about
(`w-blockir` #2305: the order *was* the whole cell).

---

## 8. Scoring

Filled in by `docs/rungs/2026-08-09-w-biquad.md` §9. Nothing above may be edited
after the first `crates/` change; corrections go in the rung.
