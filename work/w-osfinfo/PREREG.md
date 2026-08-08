# w-osfinfo — PRE-REGISTRATION

    Lane:   w-osfinfo, worktree branch `wt-w-osfinfo` off master **`b96a3f19`**.
    Target: `src/xdk/LIBCMT/osfinfo.cpp` — the LAST TU of the undefined-external
            seam (w-extdata `vswprnc` · w-undname `undname` · this).
    Frozen: BEFORE the first change to `crates/` and before the first cell this
            lane authors. Everything below is a survey of the tree as it stands,
            plus predictions.

Base metrics, master `b96a3f19`: match **15** · mismatch 0 · codegen-gap 0 ·
vocab-gap **856** · capture-fail 7 · **FRONTIER 12** · tests **1,275 / 36
targets** · gate 5,346 fixture-verdicts.

---

## 0. What was surveyed before this file was written

Read-only, plus two artifacts that touch nothing in `crates/`:

* `work/w-osfinfo/ref/osfinfo/dis.txt` — the **real** obj at the workload's own
  flags, via `work/w-frame/refobj.sh` + `scripts/gt_dump.py`. Committed.
* `work/w-osfinfo/OSFINFO_BODY.md` — the IL body decoded token by token, from a
  `c2rs capture --keep-il`. The IL itself is never committed; the reading is.

---

## 1. The refusal chain, RE-DERIVED AT THIS BASE

The brief says not to trust a prior listing. Both inherited prices are
**re-derived here**, and both are wrong in at least one row.

### 1.1 `decode_causes` at `b96a3f19`

`c2rs census src/xdk/LIBCMT/osfinfo.cpp --flags-file work/dc3-workload/flags.txt`:

```text
  src/xdk/LIBCMT/osfinfo.cpp -> 0/1 functions in class
  [  0] GAP expr-cmp-ge   cflow-if-n   eh-none   445 B  (unnamed)
```

**w-extdata §6.2's `gl-stop-name-not-mangled` is GONE**, as w-undname §5 already
reported: the `.gl` name widening (#1721) reaches `_free_osfhnd` (twelve bytes,
string table). The census's fall-through blocker is `expr-cmp-ge` (`0x23`), the
first byte of the first guard — i.e. the generic expression layer, which is what
a whole-body recognizer replaces rather than repairs.

`c2rs gap` on the TU alone: **A 1 · B 1 · C 1 · D 0 · E 0**. Sections, binding
and emit set are all already satisfied; the whole distance is D.

### 1.2 The reference obj — 38 words, 10 relocations, 6 sections

152 B `.text`, one emitted function, prologue 3 words / epilogue 4 words, so
**31 body words**. Symbol table indices 15–18 are
`__doserrno` · `_errno` · `__pioinfo` · `_nhandle`, whose first references are
`+0x74` · `+0x68` · `+0x28` · `+0x14` — **strictly descending index against
ascending offset, kind ignored**. That is board #1720's merged rule, which
w-undname shipped; this TU is its **second** graded cell and its first with
**two callees**. No new writer rule is predicted.

### 1.3 The block plan (from `dis.txt`, offsets absolute)

```text
  0x0c  cmpwi cr6,r3,0          fh >= 0          SIGNED  ─┐
  0x10  bt  LT  -> Lerr                                   │ 4 guards, all
  0x14  lis  r11,0     REFHI _nhandle                     │ falling to ONE
  0x18  lwz  r11,0(r11) REFLO _nhandle   <-- (R3)         │ sunk error block
  0x1c  cmplw cr6,r3,r11        fh < _nhandle  UNSIGNED   │
  0x20  bf  LT  -> Lerr                                   │
  0x24  srawi r11,r3,5                                    │
  0x28  lis  r10,0     REFHI __pioinfo   <-- (R4)         │
  0x2c  slwi  r9,r11,2                                    │
  0x30  addi  r10,r10,0 REFLO __pioinfo                   │
  0x34  clrlwi r11,r3,27                                  │
  0x38  mulli r11,r11,72                                  │
  0x3c  lwzx  r10,r9,r10                                  │
  0x40  add   r11,r10,r11        pio                      │
  0x44  lbz   r10,4(r11)         pio->osfile              │
  0x48  clrlwi. r10,r10,31       & 1, RECORD form <-(R6)  │
  0x4c  bt  cr0.EQ -> Lerr                                │
  0x50  lwz   r10,0(r11)         pio->osfhnd              │
  0x54  cmpwi cr6,r10,-1                                  │
  0x58  bt  cr6.EQ -> Lerr                               ─┘
  0x5c  li    r10,-1
  0x60  li    r3,0
  0x64  b     Ljoin              the ONE intra-section `b`
  0x68 Lerr:  bl _errno          REL24
  0x6c  li    r11,9
  0x70  stw   r11,0(r3)
  0x74  bl    __doserrno         REL24
  0x78  mr    r11,r3
  0x7c  li    r10,0
  0x80  li    r3,-1
  0x84 Ljoin: stw r10,0(r11)     THE TAIL MERGE
```

**The single most surprising word is `Ljoin`.** c2 tail-merged
`pio->osfhnd = -1` and `*__doserrno() = 0` into ONE `stw r10,0(r11)`, allocating
the address to r11 and the value to r10 on **both** paths — two structurally
unrelated stores sharing one word. It is only legal because `off_hnd == 0`;
a non-zero `osfhnd` offset would need two displacements and is **not** this
class (fence clause F7 below).

### 1.4 The price at THIS base — nine named refusals

| # | refusal | where | size | inherited price said |
|---|---|---|---:|---|
| R1 | the recognizer for the body class | `c2-il` shapes | 381 IL bytes, 3 labels, 4 guards, 2 calls | w-undname row 1 |
| R2 | the emitter — 31 body words | `c2-core/codegen` | 31 words | w-undname row 2 |
| R3 | a REFLO carried in a **`lwz` displacement** (`lwz r11,0(r11)` at 0x18) | `data_refs_of` | 1 site form | w-undname row 3 ✓ |
| R4 | a high half in **r10**, not the scratch — the walk's KEY, not a clause on it | `data_refs_of` | 1 rewrite | w-undname row 4 ✓ |
| R5 | **`encode_cmplw` does NOT exist** | `codegen/encode.rs` | 1 encoder | **w-undname row 6 says it does — WRONG** |
| R6 | record-form `rlwinm.` (`clrlwi. r10,r10,31`) | `codegen/encode.rs` | 1 encoder | w-undname row 6 ✓ |
| R7 | `IlFunction::callees()` needs an arm — **two** callees | `c2-il` | 1 arm | not priced (w-undname §4.3's precedent) |
| R8 | `data_syms` producer must yield **two** names for this shape | `c2-il` | 1 site | not priced |
| R9 | the label lead | `IlFunction::label_lead` | 1 term | not priced |

**Two corrections to the inherited price, in opposite directions.**

* **w-undname row 5 — "the two pairs then interleave across registers" — is
  REFUTED.** They do not interleave. Pair 1 is `(0x14, 0x18)` and pair 2 is
  `(0x28, 0x30)`; pair 1 is closed before pair 2 opens. The forward walk's single
  `open` slot is **sufficient**, and no rewrite of §4.1's walk is needed beyond
  R3 and R4. That row does not fire.
* **w-undname row 6 — "`encode_cmplw` already exists — w-extdata's *2 encoders*
  was over by one" — is itself WRONG.** `crates/c2-core/src/codegen/encode.rs`
  has `encode_cmpw` (XO 0, signed) and `encode_cmplwi` (opcode 10, immediate),
  and **no `cmplw`**. The reference word is `7f035840` = opcode 31 / XO **32**,
  which neither produces. **w-extdata's original "2 encoders" was right.** The
  correction is re-corrected, and the lesson is the brief's own: re-derive at
  your base.

So the re-price is **9 named**, against w-undname's "≥ 6".

### 1.5 The lead — a MECHANISM, not an analogy (the brief's warning taken)

The brief warns that the analogy that predicted +1 was refuted on the last
class. So this lane does not argue by analogy. It argues by a **rule fitted to
all three prior measurements**, which is falsifiable in one run:

| class | witness | intra-section `b` words in the body | measured lead |
|---|---|---:|---:|
| `if_call_join` | `negate_test.cpp` | 1 (`b $LN8`) | **1** |
| `guard_chain_shared_tail` | `vswprnc.cpp` | 1 (range arm → shared tail) | **1** |
| `alloc_init_or_fail` | `undname.cpp` | **0** (no `48……` word at all) | **0** |

**Three for three: lead == the number of unconditional intra-section `b` words.**
`osfinfo` has exactly one (`b .+32` at 0x64), so this lane registers **lead = 1**.

This is registered as a *hypothesis with a mechanism* — c2 mints a named block
label for a `b` target and charges the counter for it before the function's own
`$M` triple — and it is the FIRST time this project has had a rule rather than a
per-class constant. If it is wrong, the outcome is three symbol records off by
one and nothing else, exactly as in the two prior misses, and the rule is
refuted in writing.

---

## 2. The unnamed-refusal budget (explicit, per the brief)

**Five conversion lanes in a row found a reader refusal no survey had priced.**
w-heap 2, w-data 1, w-cfg2 1, w-extdata 1 (#1721), w-undname 1 (#1743). This
lane registers, before any probe:

> **EXPECT ≥ 1 refusal not in §1.4, and expect it in `crates/c2-il`'s TU-level
> accounting rather than in `codegen/`.** The base rate is 5/5, and four of the
> six historical instances were in the IL crate.

Named candidates this lane can already see as *possible* and is NOT counting in
the price of 9, so that finding one is not scored as a hit:

* the `.sy` automatic-locals view (`SyLocals`) admitting `fa09`/`fb09` — one is
  an `int`, one is a **pointer** local; board #764 is the precedent for `.sy`
  refusing a non-`int` induction variable.
* the type-tag family `0x82 …` (the `osfile` byte field's type) — every type in
  every prior class of this seam is `0x86`-tagged.
* `Bindings::unclaimed` / the TU-level accounting gate, which is where #1743 was.

---

## 3. Decline clauses — thresholds AND sizes

| clause | threshold | size at which it fires |
|---|---|---|
| **D1 — the block plan.** Decline if any of the 31 body words needs a scheduler or a register allocator. | expectation: **0 words chosen**, and **11 immediate fields** (`k_shift` 5, `k_mask` 31, `k_scale` 4, `k_elem` 72, `off_file` 4, `off_hnd` 0, `k_bit` 1, `k_invalid` -1, `k_errno` 9, `k_doserrno` 0, `k_ok`/`k_fail` 0/-1 as one signed pair) | any word not a pure function of the parse |
| **D2 — the reader.** Decline if the recognizer needs a block IR, a value merge at a join, or a back-reference. | expectation: one forward cursor, 0 back-references (measured on the decode in `OSFINFO_BODY.md`, written before this file) | ≥ 1 back-reference |
| **D3 — `data_refs_of`'s key.** The walk is re-keyed off `SCRATCH_REG`. Decline if the re-key cannot be made **byte-neutral by construction** on the shipped population. | expectation: neutral, because every shipped body's high half IS the scratch and its low half IS an `addi rD,r11,0`, so the generalized walk visits the identical words | any previously-emitted obj differs |
| **D4 — previously-emitted objs.** Decline (and revert) if ANY obj the port already emits differs. | expectation **0**, proved by the 878-TU scan diffed **per TU BY NAME** at base and tip, plus the gate's per-lane `match` counts | ≥ 1 TU lost |
| **D5 — the `lwz` low-half form.** Decline if admitting `lwz rD,0(rT)` as a REFLO carrier cannot be fenced against an ordinary zero-displacement load. | **This body contains the counter-example already**: `lwz r10,0(r11)` at 0x50 is NOT a relocation. The fence registered in advance: the `lwz` form is recognized **only while a pair on that exact register is open**; the `addi rD,rT,0` form keeps its "closer with no open ⇒ refuse" canary because an `addi rD,rT,0` is never an ordinary instruction and a zero-displacement load always is. | if the fence needs a heuristic |
| **D6 — `ptr_walk_loop`'s unpaid #1638.** | registered **NOT TAKEN** | — |
| **D7 — the tail merge.** Decline if the shared `stw r10,0(r11)` needs the emitter to *decide* it is shareable. | it does not: the reader **requires `off_hnd == 0`** (F7) and refuses otherwise, so the emitter has one word and no choice | if `off_hnd != 0` must be supported |
| **D8 — a refusal becoming a wrong emit.** Any `mismatch` anywhere. | expectation 0 everywhere, on the scan and on every gate row | ≥ 1 |
| **D9 — the encoders.** Both new encoders must be pinned to a byte real `c2` emitted. | `cmplw cr6,r3,r11` = `7f035840` and `clrlwi. r10,r10,31` = `554a07ff`, both from `work/w-osfinfo/ref/osfinfo/dis.txt` | an encoder asserted from a manual only |

---

## 4. The conversion call

| outcome | P registered |
|---|---:|
| **`osfinfo` converts — match 15 → 16, FRONTIER 12 → 11** | **0.55** |
| nothing converts, priced decline | 0.45 |

Calibration note: the last three conversion lanes registered 0.6, 0.55, 0.55 and
all three were right. This lane holds **0.55** rather than raising it, for three
registered reasons:

1. **The unnamed-refusal base rate is 5/5** (§2) and this lane's price of 9 is a
   count of things it can name.
2. **This lane must re-key a function every emitting body runs through.**
   `data_refs_of` is not new code beside the old code — it is the same walk with
   a different key, and §3's D3 argues neutrality by construction, but
   w-undname's equivalent risk (deleting `check_external_order`) was the one it
   named and did not have materialize. Naming it twice does not make it safe.
3. **The `.text` byte fraction is 0.0 %, so there is no partial credit.** This
   term has now been registered three lanes running and has bitten **zero**
   times; it is recorded here as a term that is *probably not predictive*, so
   that a fourth non-bite is on the record as evidence against it rather than as
   a fourth free pass.

Reason NOT to hold it lower, also registered: **the emit set, the binding and
the section shape are all already satisfied** (§1.1, A 1 · B 1 · C 1) and the
frame is free at `saved_gprs 0` / `out_slots 0` (96 B, verified by w-extdata and
re-checked here against `FrameLayout::size`).

---

## 5. Metric predictions

| metric | predicted |
|---|---:|
| TU match | **16** |
| mismatch | 0 |
| codegen-gap | 0 |
| vocab-gap | **855** |
| port-error | 0 |
| capture-fail | 7 |
| **FRONTIER** | **11** |
| frontier-if-A | 133 |
| factor A / B / C | unchanged — 28 / 338 / 169 |
| `A∧B∧C` | 27 |
| factor D | **16** |
| `A∧B∧C∧D` | **14** |
| `A∧B∧C∧(D∨E)` | **16** |
| function census | 711,492 (**+1**) |
| emitted census | 39,191 (**+1**) |
| `fnbyte-exact` | 36,219 (**+1**) |
| `fnbyte-tus-full` | **12** |
| `fnbyte-differs` | 2,111 (unchanged) |
| **peer keys** | **0 vanished, 1 appeared** — `fnbyte-shape-<tag>-exact` for the new `Selected` variant, named in advance per w-undname §7.2 |

## 5.1 The workspace test count — registered as a **DELTA**, per #1749

w-undname's miss was a hand-summed total. This lane registers the **delta only**
and lets `cargo test` do the addition.

> **PREDICTED DELTA: +13 tests.**
>
> Itemized: **7** unit tests in `codegen::osf_handle_guard` (the word-for-word
> reference body; the four immediate-field guards; the two-CR/compare-form
> guard; the tail-merge guard; the `/O1`-only refusal), **2** in
> `codegen::encode` (one per new encoder, each pinned to a real `c2` word),
> **3** in `crate::tests` for `data_refs_of` (the re-keyed walk; the `lwz`
> low-half form; **the negative — `lwz rD,0(rT)` with no open pair is NOT a
> low half**, whose witness is this body's own 0x50), **1** differential test
> for the positive fixture.
>
> Base is whatever `cargo test --workspace --release` reads at `b96a3f19` in
> this worktree; the claim is `tip − base == 13`, checked by subtraction of two
> measured numbers and by naming all thirteen.

## 5.2 Fixtures

Two, following the class convention: `wosf_handle_guard.cpp` (positive, `/O1`)
and `wosf_handle_guard_neg.cpp` (negative, one cell per decline clause).
Expected gate delta: **+36 fixture-verdicts** (2 fixtures × 18 lanes), so
5,346 → **5,382**. A smaller delta means a lane silently skipped them.

---

## 6. What this lane registers it will NOT do

* It will **not** widen `data_refs_of` to interleaved pairs. §1.4 shows this body
  does not need it; a widening with no cell is `docs/STATUS.md` trap 0.
* It will **not** take `ptr_walk_loop`'s unpaid #1638 clause.
* It will **not** claim `cflow-if-n` as a class. `PORT_CFG_CLASSES` stays as it
  is; this is one more transcribed function class, `/O1` only.
* It will **not** generalize the `slwi`-vs-`mulli` choice. `k_scale` is pinned to
  4 (a `slwi` of 2) and `k_elem` is a `mulli` field that **refuses a power of
  two**, because with one witness of each form the chooser is exactly the word a
  scheduler would pick and D1 forbids it.
