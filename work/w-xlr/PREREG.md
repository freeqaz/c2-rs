# w-xlr — PREREGISTRATION

**Committed BEFORE the first change to `crates/` and before the first cell this
lane authors.** Everything below was derived from (a) the reference obj for
`src/xdk/xlrc/xlrcimpl.cpp` captured at the workload's own flags with
`work/w-frame/refobj.sh`, (b) the captured IL bundle for the same TU, and (c)
reading the shipped tree. No `crates/` file has been touched and no fixture
authored at the time this is committed.

Lane `w-xlr`, worktree branch `wt-w-xlr` off master **`cc5c803f`**.

---

## 0. The base, measured on this tree with this binary

`work/w-xlr/scan_base.out` — 878 TUs, `--jobs 16`, the workload's own
flags/cwd.

| key | base |
|---|---:|
| `match` | **16** |
| `mismatch` | 0 |
| `codegen-gap` | 0 |
| `vocab-gap` | 855 |
| `capture-fail` | 7 |
| `port-error` | 0 |
| **`frontier`** | **11** |
| `frontier-if-a` | 133 |
| `factor-a` / `-b` / `-c` / `-d` / `-e` | 28 / 338 / 169 / 16 / 2 |
| `a-and-b-and-c` / `…-and-d` / `b-and-c` | 27 / 14 / 151 |
| `fnbyte-exact` / `-differs` / `-tus-full` | 36219 / 2111 / 12 |
| `writer-sections` | 10 |
| `progress-mass` | 0.20830 |
| function census / emitted census | 711492 / 39191 |
| `gap-metric` keys | **249** |

Workspace tests at base: measured into `work/w-xlr/tests_base.out` (the DELTA in
§5 is registered against whatever that run reports, per #1749).

---

## 1. The target, and what the survey said about it

`src/xdk/xlrc/xlrcimpl.cpp` — one emitted function,
`CXLrcImpl_CreateClientWithTransport`, `.text` **152 B / 38 words**, 4
relocations, `.pdata` 8 B, 23 symbols. `c2rs gap` on the TU alone reads
`vocab-gap`, 1 blocked emitted function, first blocker
**`assign-rhs-call-0x26`**, dispatch `disp-assign`, cflow class `cflow-if-n`,
frame class `calls-2plus`.

`w-osfinfo` §10's re-survey says this row's *"only remaining blocker is a single
named mechanism … it is a frame-emitter rung and not a body rung"*.

> ### §1.0 — THAT CLAIM IS REGISTERED HERE AS **WRONG**, BEFORE ANY PROBE.
>
> The `__savegprlr_26` frame is **one** of the refusals in §2 and it is not the
> one the census names. The reader refuses this TU at `assign-rhs-call-0x26`
> long before a `FrameLayout` is ever constructed, and `w-front3` §2.1 already
> classified that key as *"a real refusal with no lift — the production is
> unimplemented, not guarded"*, with a codegen column of **6 INFERRED**. This
> lane registers the survey's "single mechanism" reading as a **survey artifact
> of reading a published price rather than the tree**, and §2 prices it at
> **thirteen**. Scored in the rung either way.

### 1.1 The body, decoded from the IL (`4C 4F 11` … `4D`, 446 bytes)

One linear token stream, **forward-only label references, zero back edges**.
Seven labels (`L1` epilogue, `L5 L6 L8 L10 L12 L14`), six blocks. Reconstructed:

```c
long CXLrcImpl_CreateClientWithTransport(ID id, unsigned *outSize,
                                         Client **outClient, Transport **outT) {
    unsigned size = 4;              /* ADDRESS-TAKEN local */
    long result = 0;
    Client *client = CreateClient(&size);
    if (client == 0) {
        if (size < 4) result = 0x8007000E; else result = 0x800710DD;
    } else {
        Transport *t = CreateTransport(client, id, size);
        if (t == 0) result = 0x80004005;
        else { *outSize = size; *outClient = client; *outT = t; }
    }
    return result;
}
```

### 1.2 The 38 words, transcribed from the reference obj

```text
 0x00 7d8802a6 mflr r12
 0x04 4bfffffd bl  __savegprlr_26     REL24 [22]
 0x08 9421ff70 stwu r1,-144(r1)
 0x0c 39600004 li  r11,4              <- $M2589 = 0x0c, PrologLen 3
 0x10 7c7e1b78 mr  r30,r3
 0x14 91610050 stw r11,80(r1)
 0x18 38610050 addi r3,r1,80          &size
 0x1c 7c9d2378 mr  r29,r4
 0x20 7cbc2b78 mr  r28,r5
 0x24 7cdb3378 mr  r27,r6
 0x28 3b400000 li  r26,0
 0x2c 4bffffd5 bl  ?CreateClient@…    REL24 [16]
 0x30 7c7f1b79 mr. r31,r3             RECORD form, cr0
 0x34 40820024 bf  2,+36 -> 0x58
 0x38 81610050 lwz r11,80(r1)
 0x3c 3f408007 lis r26,0x8007
 0x40 2b0b0004 cmplwi cr6,r11,4
 0x44 4098000c bf  24,+12 -> 0x50
 0x48 635a000e ori r26,r26,0x000e
 0x4c 48000040 b   +64 -> 0x8c        intra-section b #1
 0x50 635a10dd ori r26,r26,0x10dd
 0x54 48000038 b   +56 -> 0x8c        intra-section b #2
 0x58 7fc4f378 mr  r4,r30
 0x5c 80a10050 lwz r5,80(r1)
 0x60 7fe3fb78 mr  r3,r31
 0x64 4bffff9d bl  CXLrcClient_CreateTransport  REL24 [15]
 0x68 28030000 cmplwi cr0,r3,0
 0x6c 40820010 bf  2,+16 -> 0x7c
 0x70 3f408000 lis r26,0x8000
 0x74 635a4005 ori r26,r26,0x4005
 0x78 48000014 b   +20 -> 0x8c        intra-section b #3
 0x7c 81610050 lwz r11,80(r1)
 0x80 917d0000 stw r11,0(r29)
 0x84 93fc0000 stw r31,0(r28)
 0x88 907b0000 stw r3,0(r27)
 0x8c 7f43d378 mr  r3,r26
 0x90 38210090 addi r1,r1,144
 0x94 4bffff6c b   __restgprlr_26     REL24 [21], LK=0, NO blr
```

`.pdata` `00000000 40002603` → FunctionLen 0x26 = 38 words, PrologLen 3.
Symbol group: `.text`+aux · fn · `$M2590`(0x98) · `CXLrcClient_CreateTransport`
· `?CreateClient@…` · `$M2589`(0x0c) · `.pdata`+aux · `$T2591` ·
`__restgprlr_26` · `__savegprlr_26`. The two ordinary callees sit **between the
two `$M`s** in reverse first-reference order; **the helper pair sits AFTER
`$T`**, itself reverse first-reference.

---

## 2. The refusal chain at THIS base — **THIRTEEN**, itemized

Each row is a fact the port does not have today, established by reading the
shipped tree (file and symbol named), not by lifting anything.

| # | crate | refusal | expectation |
|---|---|---|---|
| R1 | `c2-il` | **the recognizer for the body class.** `shapes::assign` refuses at `assign-rhs-call` whenever an assignment's RHS is a call and `env` is non-empty (`assign.rs:138`). This body's call-assign is the **third** statement | a new `shapes::xlrc_create_guard` module, ~1 forward cursor over 446 bytes |
| R2 | `c2-il` | **the address-taken local.** `.sy` admits *"plain `int`, never address-taken"*; `size` is `unsigned` **and** has its address passed to a callee | the class reads the local structurally (the `55` address-of arg) and pins ONE such local; no `.sy` widening |
| R3 | `c2-il` | **`IlFunction::callees()` needs an arm** — the streak-breaker of #1764. Two callees, first-reference order | paid before the first census run |
| R4 | `c2-il` | **`label_lead`** — a new term. See §3, where the number is *predicted* and the mechanism named | +2 |
| R5 | `c2-core` | **`FrameLayout::out_of_class_ctx` returns `frame-savegprlr-helper` at `saved_gprs >= 3`** (`frame.rs:207/223`). This body saves **six** | a helper prologue/epilogue variant; `size()` needs **no** change — it already computes 144 (checked by hand: `max(80, 80+4) + 8·7 = 140 → align16 = 144`) |
| R6 | `c2-core` | **the epilogue has no `blr`.** Every shipped epilogue ends in one (`frame.rs::epilogue`); this one ends in a REL24 `b` with LK=0 | a separate emitter path, not a flag on `epilogue()` |
| R7 | `c2-core` | **the helper externals are CALLS the IL never names.** `introduced_externals` is built from `self.calls`, and every existing `Call` comes from a callee the reader read out of the IL | two synthetic `Call`s whose names are derived from `saved_gprs` |
| R8 | `c2-core/coff` | **the helper pair's symbol placement is AFTER `$T`**, not in the callee slot between the `$M`s (`writer.rs:462…481`) | a separate `Function` field, excluded from `introduced_externals`, emitted after `$T`, with the relocation index map agreeing |
| R9 | `c2-core` | **`mr.`** — the record-form register move (`7c7f1b79`). `encode_mr` exists; the record bit does not | one encoder |
| R10 | `c2-core` | **`ori`** (`635a000e`) — the low half of a wide constant. `encode_addis` exists for `lis`; `ori` does not appear in `encode.rs` | one encoder |
| R11 | `c2-core` | **`cmplwi` on a NON-ZERO CR field** (`2b0b0004` is cr6, `28030000` is cr0). Both forms are needed and they are four words apart on different operands | one encoder taking `crf` |
| R12 | `c2-core` | **the six-block emitter** — 38 words, three intra-section `b`s and three `bf`s across two CR fields | a new `codegen::xlrc_create_guard`, ~1 block plan |
| R13 | `c2-core` | **the callee-saved register assignment.** r31←call result, r30..r27←the four formals, r26←the accumulator. Descending-from-r31 is the shipped rule and it does **not** predict this order (the result gets r31 though it is defined last) | **PINNED by transcription, refused if the shape varies** — #1706's rule |

**Against the published prices**: `w-front2` row 6 says `>= 1² + 6 = >= 7`;
`w-front3` row 2 says READER `>= 1` BOUND + CODEGEN 6 INFERRED. Thirteen at this
base, of which **four** are in the COFF/symbol layer that neither price has a
column for.

### 2.1 The unnamed-refusal budget — explicitly carried

Five conversion lanes running found a refusal no survey priced; `w-osfinfo`
broke the streak by **reading its predecessor's rung first and looking where the
last one was found**. This lane does the same. The three places checked ahead of
the first census run, and their status at this base:

1. `IlFunction::callees()` — **R3 above**, found by looking, priced.
2. `Bindings::unclaimed` / the unclaimed-`.gl`-data-symbol TU gate — this TU's
   `.gl` names two callees, one `__C1_11886` and the function; **no data
   symbol**. Expected not to fire.
3. Census filing under `callee-unresolved-tail-call` (#1704) — the `_neg`
   fixture's clauses will be read **per cell with an applied-and-reverted probe
   patch**, never off the fall-through blocker.

**Budgeted: ONE unnamed refusal, expected in `crates/c2-core/src/coff`** (the
symbol-index assignment feeding relocations, which R8 disturbs and which no
existing class exercises with a symbol *after* `$T`). Registered as a
prediction, scored in the rung.

---

## 3. The label lead — a PREDICTION with a mechanism, and a REFUTATION

`$M2589` / `$M2590` / `$T2591` in the reference obj; the `.gl` counter is
**2575**. `plan_labels` gives `2575 + LABEL_SEED_GAP(9) + 3·1 + lead = 2589`, so

> **the required lead is exactly 2.**

Two rules are in the tree and they **disagree on this body**:

| rule | source | predicts here |
|---|---|---:|
| **#1761** — *"the lead is the number of unconditional intra-section `b` words"* | `IlFunction::label_lead`'s own doc, a fit to four points | **3** |
| **`LABEL_COUNTER.md` §1.1** — *"`__savegprlr_N`/`__restgprlr_N`, each distinct N first introduced: **+2**"*, allocated **before** the function's own `$M` pair | 29 measured probes, `gt_label_stride.py`, both modes | **2** |

**Registered before any emit: #1761's rule is REFUTED by this body and §1.1's
surcharge table is right.** The three intra-section `b`s contribute **0**. The
lane will ship `+ 2 * u32::from(<this class>.is_some())` and write the
refutation into `label_lead`'s own doc, beside the rule it corrects — not only
into the rung.

**The honest size of that claim, stated in advance**: this is one witness, and
it does not by itself explain why `if_call_join`, `guard_chain_shared_tail` and
`osf_handle_guard` each charge 1 with one `b`. What it does establish is that
**`b`-count is not the mechanism**, because three `b`s here charge zero. The
rung will say that and will not invent a replacement rule for the other three.

**Counterfactual registered in advance**: the lane will build once with the term
forced to `0` and confirm the obj reads `mismatch`-shaped (bytes diverge in the
three symbol records and nowhere else), then restore it — the same control
`w-osfinfo` §3 ran.

---

## 4. Metric predictions (conditional on the conversion landing)

| metric | predicted |
|---|---:|
| `match` | **17** |
| `mismatch` | **0** |
| `codegen-gap` | 0 |
| `vocab-gap` | **854** |
| `capture-fail` | 7 |
| `port-error` | 0 |
| **`frontier`** | **10** |
| `frontier-if-a` | **132** |
| `factor-a` / `-b` / `-c` | 28 / 338 / 169 (unchanged) |
| `a-and-b-and-c` | 27 (unchanged) |
| `factor-d` | **17** |
| `a-and-b-and-c-and-d` | **15** |
| function census | **711493** (+1) |
| emitted census | **39192** (+1) |
| `fnbyte-exact` | **36220** (+1) |
| `fnbyte-differs` | 2111 (unchanged) |
| `fnbyte-tus-full` | **13** |
| `writer-sections` | 10 (unchanged) |
| **peer keys** | **0 vanished, 1 appeared** (`fnbyte-shape-*-exact` for the new class) |
| per-TU verdict set | **exactly one moves** (`xlrcimpl.cpp` `vocab-gap → match`), **zero move the other way**, compared as a SET by name at both ends |
| gate | 18/18 PASS, fixture-verdicts **5,382 + 18·(new fixtures)**, sweep 19,460/19,556, cross 90,424/90,812, hatch-red 14/14, ladder-red 5/5, **0 mismatch** |

---

## 5. Test-count DELTA, itemized (#1749's form: a delta, never a total)

| row | registered |
|---|---:|
| `codegen::xlrc_create_guard` unit tests (the 38 words, the three `b` displacements, the two CR fields, the frame, the refusals of R13's pinned shape) | **9** |
| `codegen::frame` — the helper prologue/epilogue pair | **3** |
| `codegen::encode` — one per new encoder (`mr.`, `ori`, `cmplwi` with `crf`) | **3** |
| `coff` — the helper external's placement after `$T` and its relocation index | **2** |
| `c2-il` `shapes::xlrc_create_guard` + `label_lead` | **4** |
| differential (the positive fixture) | **1** |
| **total DELTA** | **+22** |

§8.2.1 of `w-osfinfo` is explicit that the delta form removes the arithmetic
failure mode and **leaves the estimate exposed**; this is an estimate made
before the class is written, and it is registered as such.

---

## 6. Decline clauses — thresholds AND sizes

| clause | threshold | size if it fires |
|---|---|---|
| **D1 — the block plan** | decline if any of the 38 words needs a **chooser** (a decision c2 makes that this class has one witness of each side for). Expectation: **0 chosen words, 5 free immediate fields** (`k_init`=4, `k_lo`=0x8007000E, `k_hi`=0x800710DD, `k_fail`=0x80004005, the `cmplwi` bound 4) and everything else pinned | the whole conversion; report the chooser by name and the count of witnesses per side |
| **D2 — the reader** | decline if the recognizer needs a block IR, a value merge at a join, or a **back-reference**. Measured in advance: 0 back edges, 7 forward labels | the whole conversion; ~450 IL bytes unread |
| **D3 — the helper frame** | decline if the helper prologue/epilogue cannot be made **byte-neutral by construction** for every obj the port emits today. Expectation: neutral by construction because `needs_gpr_helper()` gates it and every emitted body has `saved_gprs <= 2` | the frame rung; ~R5+R6, 2 of 13 |
| **D4 — previously-emitted objs** | decline if ANY previously-emitted obj differs. Expectation **0**, measured as a per-TU SET comparison over 878 TUs at both ends **plus** the gate's per-lane `match` counts | the whole conversion; a widening that loses a TU is worse than a gap |
| **D5 — the symbol placement** | decline if the helper pair's post-`$T` placement cannot be expressed without changing the symbol index of any symbol in any currently-emitted obj | R8+R7, 2 of 13 |
| **D6 — the label lead** | decline if the measured lead is not the **2** §3 predicts. This is registered as a *prediction*, so a miss here is a scored miss and not a silent refit | 1 of 13, plus §3's refutation stands or falls with it |
| **D7 — the register assignment** | decline if R13's order (r31 ← the call result, r30..r27 ← the formals, r26 ← the accumulator) requires the emitter to **decide** anything. Pinned by transcription; the reader refuses any body whose formal count is not 4 or whose accumulator is not single | 1 of 13 |
| **D8 — a refusal becoming a wrong emit** | any `mismatch` anywhere, on either scan or any gate row. #232's direction | everything — a wrong emit is strictly worse than a gap |
| **D9 — the encoders** | every new encoder must be pinned to a byte real `c2` emitted, quoted from `work/w-xlr/probe/ref.obj` | 3 of 13 |
| **D10 — `ptr_walk_loop`'s unpaid #1638** | registered **NOT TAKEN**. Still open, still behind a matched TU | — |

---

## 7. The conversion call

| outcome | P |
|---|---:|
| `xlrcimpl` converts (`match` 16 → 17, FRONTIER 11 → 10) | **0.50** |
| nothing converts, priced decline | 0.50 |

**Lower than the streak's 0.55, and the reasons are registered rather than
hedged:**

* the price is **13** against `w-osfinfo`'s 9, and **four of the thirteen are in
  a seam no conversion lane has touched** — the COFF symbol layer. Every one of
  the last five lanes paid entirely inside `c2-il` + `codegen/`;
* this is the **first framed class the port has ever emitted with a prologue
  that is not `mflr`/`stw`/`stwu`**, and the epilogue has no `blr` — two
  invariants that are load-bearing in `frame.rs`, `pdata.rs` and the writer;
* against that: **every value in the 38 words is readable from the obj**, there
  are **zero back edges** in the IL, `FrameLayout::size()` already computes 144
  without modification, and the label lead is **already measured at 2** rather
  than guessed. The `.text` byte fraction is 0.0 %, so there is no partial
  credit — a term `w-osfinfo` §8.1 says has now run four lanes without biting
  and **is not registered as a risk here**.

---

## 8. What this lane will NOT do

* It will **not** widen `.sy` to address-taken locals. R2 is paid structurally.
* It will **not** claim `cflow-if-n` as a class or touch `PORT_CFG_CLASSES`.
* It will **not** take `ptr_walk_loop`'s unpaid #1638 clause (D10).
* It will **not** generalize the helper to FPRs, to `_RtlCheckStack12`, or to
  any `N` other than the one its reader pins.
* It will **not** invent a replacement for #1761's refuted `b`-count rule for
  the three classes it does not witness (§3).
