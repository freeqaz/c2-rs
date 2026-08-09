# w-pool2 — PREREGISTRATION

    Lane:       w-pool2  ·  board rows #2590–#2619  ·  rung docs/rungs/2026-08-09-w-pool2.md
    Branch:     worktree-agent-a71a139075f540042, off master **5831a092**
                (`w-pool: scrub absolute machine paths out of the lane's committed transcripts`)
    Commission: convert `src/system/utl/Pool.cpp`, TU match **21 -> 22**.
    Frozen:     BEFORE the first `crates/` change and BEFORE the first fixture
                line. `git status --short` is empty of tracked modifications at
                this commit. Everything measured below was produced by the BASE
                binary `work/w-pool2/c2rs-base`, md5
                **`ac6c85985569b02382c53d2893eee002`**, copied out of
                `target/release/` before any edit (board #2409 — never
                `git checkout master -- crates/` as a counterfactual).

    Workload stamp: dc3 `d7a3c1aa9d5d57a1176790c0e15a723edd2e03a0`, tracked tree
                CLEAN except two untracked dirs (`-.cache`, `work/`); 878 lines in
                `work/dc3-workload/files.txt`, `wc -l`-checked. Toolchain
                `compilers/X360/16.00.11886.00` (symlinked into this worktree by
                `scripts/configure_existing_worktree.sh`), wibo the sibling
                `../wibo/build/release/wibo`. Flags = the workload's own
                `/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc /I …`.

    Base, re-derived by THIS lane's own scan (`work/w-pool2/base_scan.log`,
    `base.jsonl`), not quoted from `w-pool` or `STATUS.md`:

        match 21 · mismatch 0 · codegen-gap 0 · vocab-gap 850 · port-error 0
        capture-fail 7 · graded 871 · tu-total 878
        frontier 6 · factor-a 28 · factor-b 338 · factor-c 169 · factor-d 21
        factor-e 2 · b-and-c 151 · a-and-b-and-c 27 · a-and-b-and-c-and-d-or-e 21
        fnbyte-exact 35795 · fnbyte-differs 1898 · fnbyte-refused 114637
        fnbyte-partial 10 · fnbyte-denominator 162092 · fnbyte-unbound 9220
        gap-metric keys 260
        per-function census 714538/2463470 · emitted census 39238/162092
        fixtures 339 · `#[test]` 1448 · integration-test files 29

---

## 0. WHAT IS ALREADY MEASURED — findings, NOT predictions

`w-pool` scored 30/31 and filed that as a **calibration failure**: most of its
claims restated things already on disk. This section exists so that §2 cannot do
the same. Everything here was produced before the freeze and is therefore
**barred from being registered as a prediction below**.

### 0.1 THE BINDING PREDICATE (CEILING §11.4 item 8) — PASSES, from this lane's own row

`work/w-pool2/base.jsonl`, the `Pool.cpp` row:

```
class vocab-gap · ex_len 3589 · fn_names 2 · fn_total 3 · fn_in_class 0
gate_cause  body-out-of-class
gate_causes [body-out-of-class]          <- the ONLY cause
```

Neither `bind-record-count-ne-segments` nor `bind-offset-ne-segment-start`
fires, and no `gl-stop-*` fires, so `Bindings::per_record` returns `Some` and
binds all three records 1:1. **`fn_names 2 < fn_total 3` is NOT `mmio.cpp`'s
trap** — `mangled_names` drops the `??`-prefixed constructor — and, as `w-pool`
§1.1 established, only the absence of the two `bind-*` causes settles it. Stated
here rather than re-derived wrong.

### 0.2 THE WHOLE-OBJ OBLIGATION SET IS EMPTY — walked off THIS lane's own capture

`work/w-pool2/ref/Pool.obj`, captured at the workload flags,
`scripts/gt_dump.py`: **1,227 B · 7 sections · 4 distinct names · 20 symbols ·
0 relocations in the whole file.**

| NC-1 | owed |
|---|---|
| 1 `_fltused` | **0** — no FP anywhere |
| 2 `__real@…` pool | **0** |
| 3 undefined externals | **0** — zero relocations |
| 4 `__savegprlr_N`/`__restgprlr_N` | **0** — no frame, no `.pdata` |
| 5 the compiler-label counter | **0 slots** — zero `$M`/`$T` symbols; the scan row's own `emit-label-syms` reads **0** |
| 6 `@comp.id` + shell | shipped |
| 7 minted intrinsic external | **0** |

NC-2: 4 distinct section names, all in `PORT_WRITER_SECTIONS`; count 7, the
`/Gy` three-COMDAT shape; no `.data`/`.bss`/`.rdata`, so Rule S1 is vacuous; no
`.pdata`. **All eleven items zero.** `LABEL_COUNTER.md` §7.6's procedure is
therefore not needed — there is no label in the obj to charge.

### 0.3 THE THREE BODIES, off this lane's own obj and IL

132 `.text` bytes: `??0Pool` 80 B (20 words), `?Alloc` 28 B (7), `?Free` 24 B (6).
`bytefrac-exact 0 / denominator 132`. Blockers: ctor `expr-op-0x27`
(`cflow-loop`), `?Alloc` `expr-op-0x27` (`cflow-if-1`), `?Free` `expr-brtrue`
(`cflow-if-1`). The `.ex` splits at `4F 1F` into 431 / 234 / 230 B segments,
hexdumped in `work/w-pool2/exdump.py`'s output.

### 0.4 THE WRITER ALREADY OWNS EVERY INSTRUCTION

18 distinct opcodes across the 33 instructions; `w-pool` §2.2 measured **0
missing encoders** and this lane re-checked the four that matter by name:
`encode_bclr`, `encode_bdnz`, `encode_mtctr`, `encode_twi` all exist in
`codegen/encode.rs`, and `counted_accum_loop`/`float_walk_loop` already emit
`bclr` + `mtctr` + `bdnz` together.

### 0.5 THE `/Ox` BODY IS A DIFFERENT BODY — captured here, and it decides the class boundary

`work/w-pool2/ref/PoolOx.obj` (`/Ox /GS- /c`, 144 B packed, one `.text`):

* the ctor is **21 words, not 20** — an extra `mr r11,r5` — and a completely
  different register plan (`r9`/`r10`/`r8`/`r7`/`r11` for `/O1`'s
  `r10`/`r11`/`r9`/`r5`);
* **`?Alloc` does not fold to `bclr` at all** at `/Ox`: it is
  `cmplwi ; bf 26,+8 ; blr ; lwz ; stw ; blr` — fold **band 3**, where `/O1` is
  band 2. `?Free` stays `bclr` at both.

So the class this lane can ship is `/O1` only, and the mode gate goes in the
**parser** (#1638/#1710), not only in the emitter.

---

## 1. THE FIVE MECHANISMS, RE-DERIVED AT THIS BASE — which this lane intends to pay

| # | `w-pool`'s mechanism | this lane's re-derivation | intent |
|---|---|---|---|
| **P1** | the store-run VALUE clause (`expr-op-0x27`) | confirmed: `?Free`'s two stores are a run whose first value is a member load | **PAY, but NOT by widening `store-run`.** A whole-body production per `w-biquad` #2531 — the designator layer already exists and needs no grammar |
| **P2** | fold band 2 (#187, a DECLINED c2 cost model) | **DECLINE THE COST MODEL AND SIDESTEP IT.** Band 1's own stated precondition (§3.5: "both arms are constants … cheap to build from a mask") is **false by construction** on every body in this class, because the guarded arm computes no value at all and the fall-out arm is a store run. `?a_store`, `?f_eqvoid`, `?Pool::Alloc`, `?Pool::Free` are four rows of §3.5's own table in that sub-family and all four are `bclr`. The class is drawn so band 1 is unreachable — **#187 is not settled, tested, or fitted by this lane** | **SIDESTEP** |
| **P3** | the ctor's signed-divide guard, INTERLEAVED | confirmed: the five `div_mod_leaf` words sit at r10/r9/r11 split across four unrelated instructions | **PAY as a TRANSCRIPTION**, on `div_mod_leaf`'s own stated precedent ("eight constant bodies … no free fields"). No scheduler, no allocator |
| **P4** | a `bdnz` loop whose trip count is the `divw` | **RE-PRICED DOWNWARD, and this is the one place the brief's map moves.** `WB_LOOP_FINDINGS` §9 item 4's unread non-unit trip-count arithmetic is **not owed here**: the source is `do{…}while(--n)`, step **−1**, which is inside §9 item 4's own honest class (`step ∈ {+1,−1}`), and `mtctr` takes `n` directly after one `addi −1`. **And rule 1 (rotation-plus-guard) is not owed either** — the guard `cmpwi cr6,r10,1 ; bf 25,+28` is the SOURCE's own `if (count > 1)`, present in the IL as `24`/`38 <label>`, not a synthesized zero-trip guard | **PAY**, and it is smaller than priced |
| **P5** | chain depth `D = 8` | not re-run. `w-pool` §5.1 publishes it as a chain depth and #1102 measured `D` over-counting exactly this shape (a whole-body production `parse_expr` is not on the path to). **This lane's three productions are whole-body, so `D` prices none of them** | **NOT A PRICE** |

---

## 2. THE REGISTERED CLAIMS

Probabilities are the honest ones. Claims whose evidence is already in §0 are
**not** registered — that is `w-pool` §9.2's lesson taken.

| # | claim | p |
|---|---|--:|
| **C1** | **`Pool.cpp` CONVERTS: TU match 21 → 22**, mismatch 0 | **0.55** |
| **C1a** | if C1 fails, the decline names ≥ 3 mechanisms **with the byte each owes**, and names WHICH of the three bodies reached byte-exact | 0.95 |
| **C2** | **`fnbyte-exact` delta is exactly +3** — the three `Pool.cpp` bodies and nothing else. **Positive, not zero**: unlike `w-fence2` (0) these three bodies are `fnbyte-refused` at base with `fnbyte-denominator 3`, so a conversion moves the numerator by exactly the body count; unlike `w-biquad` (+2) the body count is 3 | 0.50 |
| **C2a** | delta ∈ [0, +3] and `fnbyte-differs` does **not** rise above 1898 | 0.92 |
| **C2b** | delta is **0 if and only if C1 fails** — i.e. no partial credit: the three classes are drawn at this TU and cannot admit a workload body elsewhere | 0.80 |
| **C3** | **the emitted census (39,238) moves by ≤ +3**, and the per-function census (714,538) by ≤ +3 — no breadth, per `w-fltret`'s +444/+0 warning | 0.85 |
| **C4** | **`?Free` is byte-exact FIRST**, before either other body — it is the 6-word one and `w-pool`'s shipped positive cell is already 3 of its 6 words | 0.80 |
| **C5** | **the ctor is the LAST body to go byte-exact** and costs more lane time than the other two together | 0.85 |
| **C6** | **the guarded arm's `return 0` in `?Alloc` costs ZERO instructions** — c2 emits no `li r3,0`, because the scrutinee is already in r3 and is 0 on that edge. I will grade this on a probe cell that changes ONLY the returned literal (`return (void*)1`) and expect that cell to emit an extra word | **0.60** |
| **C7** | **a `_neg` cell that changes only `if (count > 1)` to `if (count > 0)` changes the ctor's OBJ** (the guard is not dead) | 0.75 |
| **C8** | **the class must be `/O1`-only and the parser must carry the gate**; shipping the clause in the emitter alone would make `census_gate` disagree (#1638/#1710). I expect `census/gate disagreement … PARSER-EXPRESSIBLE` to stay **0** | 0.88 |
| **C9** | **#187 is neither settled nor needed.** No line this lane ships reads a band-1↔band-2 discriminator | 0.92 |
| **C10** | **`expr-op-0x27`'s family size is unmoved to the unit** across this lane (22,409 emitted / 844 TUs / 403,879 blocked at `w-biquad`'s reading) — a conversion out of this population moves the workload's #1 key by zero | 0.85 |
| **C11** | **`#[test]` DELTA +14**, band ±6 the whole claim. `w-biquad` under-shot at +4/actual +12 and `w-pool` hit +4 on the nose with one target; this lane ships **three** shapes | 0.45 |
| **C12** | **cargo targets 39 → 40** — ONE new integration-test file, and **a new test file is a new target** (the last two lanes split-missed on exactly this) | 0.80 |
| **C13** | **at least one `_neg` cell's first draft is confounded** and the probe, not the reading, catches it (`w-biquad` #2535: seven of eleven) | 0.60 |
| **C14** | **878 TUs BY NAME: ≤ 1 changed, and if 1 then it is `Pool.cpp` toward `match`.** Regressions (was `match`, is not): **0** | 0.90 |
| **C15** | **all 339 + N fixtures at `/O1` AND `/Ox`, both binaries, list regenerated after the last fixture and `wc -l`-checked**; `/Ox` changed **0**, `/O1` changed ≤ 1 | 0.85 |
| **C16** | **`mismatch` 0 at all three levels and everywhere in the gate at the TIP.** Registered knowing `w-biquad` created one mid-lane and only the both-modes fixture scan saw it | 0.90 |
| **C17** | **gate 18/18 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT** | 0.85 |
| **C18** | **`c2rs selftest` (339+N) PASS / 0 ERROR**; `board_audit.sh` five zeros; `rung_registry` 2 passed | 0.90 |
| **C19** | **`hatch-red` REFUSES on a PRE-EXISTING failure**, reproduced at this lane's exact base tree before attribution (#1406, #1322) | 0.85 |
| **C20** | **the positive fixture is a `match` at `/O1` and NOT at `/Ox`** — §0.5 says the `/Ox` body is different, so the `/Ox` verdict is a clean `codegen-gap`, never a `mismatch` | 0.80 |
| **C21** | **no cell needs a `// c2rs-profile:` marker** — all compile at the default `/Ox /GS- /c` (#2330–#2335) | 0.85 |

### 2.1 DECLINE CLAUSES, with sizes — things this lane will NOT do

| # | declined | size |
|---|---|---|
| **D1** | **widen `leaf_store`'s `store-run` value clause** (P1 as a grammar change). 403,879 blocked bodies carry `expr-op-0x27`; `w-fltret`'s +444/+0 is the standing warning, and `leaf_store`'s own doc fixes the run boundary with a **reorder** neighbour. A widening fitted to `?Free` alone is n = 1 (`w-blockir` #2306) | 3,303 lines of `leaf_store.rs`, 403,879 rows |
| **D2** | **settle #187's band-1↔band-2 cost model.** It needs a ~30-cell probe varying only the constant pair over a fixed relation. This lane draws its class where band 1 is unreachable instead | ~30 cells |
| **D3** | **generalize `div_mod_leaf` to a scheduled/allocated form.** That is a register allocator | 8 constant bodies today |
| **D4** | **any `/Ox` or `/O2` arm for these three classes.** §0.5 measured three separate differences and this lane graded none of them | 21 words vs 20, band 3 vs band 2 |
| **D5** | **`EncryptXTEA.cpp`.** `w-xtea` #2339 prices it at ≥ 27 and #2340 says the label counter, not codegen, is its binding term | ≥ 27 |
| **D6** | **adding a name to `PORT_WRITER_SECTIONS`** — factor C is already true on this TU (4 of 10) and a name with no caller inflates C and converts nothing (#278, #301) | 0 |
| **D7** | **the `w-biquad` pool-surcharge check.** `Pool.obj` has zero `$M`/`$T` symbols and no FP constant, so the newly-pooled-FP-constant surcharge cannot apply. Checked, not assumed | 0 slots |

### 2.2 THE UNNAMED-REFUSAL BUDGET — 1, and the PRE-ARMED places

`w-pool` overran at 2 and **neither was pre-armed**, the worse half being the
board check itself. So the board check is pre-armed here, first:

1. **THE BOARD CHECK (CEILING §11.4 item 7).** `grep BOARD.md` before pricing
   any of the three bodies, and before minting any row. `w-pool` #2566 is the
   instance that rewrote a price mid-lane.
2. **A `_neg` cell confounded by source formatting** (`w-biquad` #2535).
3. **The `/Ox` half of the fixture scan finding a live wrong emit** the `/O1`
   lane and every workspace test missed (`w-biquad` #2533).
4. **The mode gate shipping in the emitter only** (#1638/#1710).
5. **A unit error** — a chain depth, a mechanism count and a word count are all
   small integers attached to `Pool.cpp` (#2571).
6. **`fn_names 2 < fn_total 3` re-derived as a binding failure** (§0.1 exists
   precisely so this cannot be spent).

**Budget: 1 unnamed refusal.** Anything beyond that is reported as an overrun
with its cause named, as `w-pool` §9.1 did.
