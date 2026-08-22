# WB-J `wb-label` — the label counter, settled: one global, one increment instruction, and a shared id space with the front end

> **PROVENANCE — DISASSEMBLY-DERIVED.** §1 is read from a static analysis of
> Microsoft's `c2.dll`, image sha256
> `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`, verified
> before the first VA was quoted. §2–§6 are **obj bytes** from real `cl.exe`
> 16.00.11886.00 under wibo and stand without §1. Pre-drafted disclosure rows
> are in §8.

Lane `wb-label` (WB-J), 2026-08-09. Board rows **#2430–#2459**. PREREGs:
`WB_LABEL_PREREG.md` (before the first export grep),
`WB_LABEL_PREREG_R2.md` (before the first `cl.exe`),
`WB_LABEL_PREREG_R3.md` (before the held-out cells). Raw output:
`work/wb-label/RESULTS.md`.

---

## 0. The headline, in four sentences

1. **There is one counter, `DAT_10c2edd0`, and exactly one instruction that
   increments it** — `inc DWORD PTR ds:0x10c2edd0` at **`0x10b97de5`**.
2. **c1xx and c2 share the id space.** The front end numbers the labels *it*
   creates and ships them inside the IL with their ids already assigned; c2's
   IL reader takes those ids **from the stream without touching the counter**,
   and c2 allocates fresh ids only for labels **it invents**. The `.gl` counter
   is the next free id, which is why the seed is a function of the source text.
3. **`docs/LABEL_COUNTER.md`'s table is not wrong.** Every row this lane
   re-measured reproduces it to the digit. What was wrong is the *instrument*
   four lanes used: `w-json`'s counterfactual form measures **Δseed + Δcharge**,
   and Δseed moves with declarations and locals. **Eight unused declarations
   that emit not one instruction move it by +16 while the true charge stays 1.**
4. **`w-bdnz`'s `+7` is reproduced to the digit and its true charge is `+2`** —
   which is `LABEL_COUNTER.md` §4.2.1's `for` row, the row that lane called six
   low.

---

## 1. The mechanism, with VAs

### 1.1 The counter and its one increment site

```text
0x10b97dd0  FUN_10b97dd0   (28 bytes)  — "take a number"
   10b97dd0: 83 3d d0 ed c2 10 00   cmp    DWORD PTR ds:0x10c2edd0,0x0
   10b97dd7: 75 07                  jne    0x10b97de0
   10b97dd9: 6a 37                  push   0x37
   10b97ddb: e8 d6 70 08 00         call   0x10c1eeb6        ; internal error 0x37
   10b97de0: a1 d0 ed c2 10         mov    eax,ds:0x10c2edd0
   10b97de5: ff 05 d0 ed c2 10      inc    DWORD PTR ds:0x10c2edd0   <== THE COUNTER
   10b97deb: c3                     ret
```

| thing | VA / value |
|---|---|
| **the counter** | **`DAT_10c2edd0`**, one 32-bit TU-global |
| **the only increment** | **`0x10b97de5`**, one instruction |
| the allocator wrapping it | **`FUN_10b97dd0` @ `0x10b97dd0`**, **31 direct call sites** |
| the generic **label constructor** | **`FUN_10b9a455` @ `0x10b9a455`** — one of those 31 callers, itself called **132 times from 86 distinct functions** |
| the **downward** end of the same id space | `DAT_10c2ed40`, decremented at the name-hash insert (`0x10b8…`, decomp line 94835), with a **crossing check**: `if (DAT_10c2ed40 <= DAT_10c2edd0) fatal` |
| the id field on a symbol | **`sym[+0x28]`** — `wb-eh`'s field, confirmed: `FUN_10b9a455` writes `puVar1[10] = FUN_10b97dd0()` |
| the **name formatter** | **`FUN_10b99dfe`** (682 bytes) — it **reads** `sym[+0x28]` and never increments |
| the decimal writer it calls | **`FUN_10c1e739`** (radix 10), from the call site `0x10b9a08e` that `wb-eh` located |

**The guard is the proof that the counter is seeded rather than zeroed**: c2
faults with internal error `0x37` if the counter is still 0 when a number is
asked for.

### 1.2 What the formatter prints, and from which field

`FUN_10b99dfe` switches on `sym[+0x30]` (the object kind) and `sym[+0x31]` (a
sub-kind character):

| `sym[+0x31]` | prints | number from |
|---|---|---|
| `'W'` | `$M` | `sym[+0x28]` |
| `'T'` | `__unwind$` | `sym[+0x28]` |
| `'V'` | `__catch$` | `sym[+0x28]` |
| `'Z'` | `__annotation$` | `sym[+0x28]` |
| kind 1 | `$S` / `$SG` / `$T` | `sym[+0x28]` |
| kind 4 | `$E` | `sym[+0x28]` |
| a **named** symbol, `sym[0x4d] ≥ 3` | `<name>$` + number | `sym[+0x28]` |
| **anonymous, kind 3** | **`$LC` / `$LL` / `$LN`** (bit `0x10` / bit `0x4` / neither of `sym[+0x43]`) | **`sym[+0x3f]`** |

> **`$LN<k>` numbers come from a DIFFERENT counter.** `sym[+0x3f]` is filled from
> **`DAT_10c2e918`**, which is reset to **1 per function** in **`FUN_10b7e113` @
> `0x10b7e113`**. `CFG_SHAPE.md` §7's *"listing-local"* is right. What is new is
> that `$LC`/`$LL`/`$LN` is a **flag** distinction (`$LL` = a loop label), not a
> spelling one, and that the two counters are **not in lockstep** — §3.3.

### 1.3 The two constructors, and why only one charges

**`FUN_10b9a455` @ `0x10b9a455` — c2 invents a label** (charges the counter):

```c
puVar1 = FUN_10b984c3(3,0,0);            /* kind 3 = label */
*(char*)(puVar1+0x31) = 0x20;            /* anonymous; caller overwrites */
puVar1[10]  = FUN_10b97dd0();            /* <== +0x28, TAKES A NUMBER */
puVar1 = FUN_10b9a0e0(puVar1,1);
*(int*)(puVar1+0x3f) = DAT_10c2e918++;   /* the $L ordinal */
```

**The IL reader, case 3 — c2 receives a label the front end already numbered**
(charges **nothing**):

```c
puVar6 = FUN_10b984c3(local_a9c,iVar12,1);
*(char*)(puVar6+0xc) = 3;                /* kind 3 = label */
puVar6[10] = FUN_10c1f91b();             /* <== +0x28 READ FROM THE IL STREAM */
...
if (name is empty) { *(int*)(puVar6+0x3f) = DAT_10c2e918++; }   /* $L ordinal only */
else { intern the name }                                        /* e.g. "top" */
```

And the seed install, two sites, **with no constant added**:

```c
/* IL directive 0x16 */   _DAT_10c2eaa0 = FUN_10c1f91b();  DAT_10c2edd0 = _DAT_10c2eaa0;
/* per-TU setup     */    _DAT_10c2eaa0 = FUN_10c1fb8b();
                          if (_DAT_10c2eaa0 < DAT_10c2edd0) _DAT_10c2eaa0 = DAT_10c2edd0;
                          DAT_10c2edd0 = _DAT_10c2eaa0;      /* = max(IL value, current) */
```

> **THE MECHANISM, STATED ONCE.** `c1xx` allocates ids out of this space for the
> symbols and labels it creates, writes them into the IL, and hands c2 the next
> free value. c2 sets its counter to that value and allocates upward for the
> labels **it** invents; named symbols are interned downward from `DAT_10c2ed40`
> and the two ends must not cross. **The measured "surcharge" of a construct is
> the number of labels c2 invents for it over and above the ones the IL already
> named.**

Three published puzzles fall out of that sentence with no further assumption:

* **Why the charge is not a function of the emitted object** (§4.1, §4.2.2). It
  is a difference between two label *sets*, one of which is c1xx's; the folds
  that produce the final bytes happen after both.
* **Why a 12-arm `switch` costs +1 and not +13.** The arm labels arrive
  pre-numbered in the IL. Measured: `p-switch` lead **1** at `/O1` (§3.1).
* **Why the source-labelled `goto` spelling costs the same as `do/while`.**
  `p_goto`'s loop-top label is printed **`$top$2561`** — a *source* name with a
  **front-end** id, 2561, which is **below this TU's first c2 label (2613)**.
  It cost c2 nothing.

### 1.4 `LABEL_SEED_GAP = 9` — the shipped constant, accounted for

`crates/c2-core/src/coff/label.rs` has carried `LABEL_SEED_GAP = 9` since the MVP
as a fitted offset. **The seed-install path adds no constant** (§1.3), so the 9
is **nine allocations c2 makes between installing the seed and the first
function's first label**. `FUN_10b9a4a7` @ `0x10b9a4a7` is one of the kinds that
does it: it constructs a **named kind-1 section object** (it sets alignment and
characteristics — `puVar2[8] = 0x180`, `puVar2[7] = 8`) and takes an upward id.
This lane did **not** enumerate the nine; it establishes only that they are
allocations and not an offset (**P8.4 refuted, P8.1 supported, count open**).

---

## 2. Why the published table is wrong — it is not, and this is the answer to deliverable 2

The commission asked whether `LABEL_COUNTER.md`'s table is *a wrong model, a
right model measured wrongly, or a model of a different quantity*.

> **It is a RIGHT MODEL MEASURED WRONGLY, and the wrong measurements are of a
> DIFFERENT QUANTITY.** The table describes the in-TU **stride**. Four lanes
> reported the whole-TU **displacement** `Δseed + Δcharge`. Both numbers are
> real; only the first is the one `coff::plan_labels` consumes.

Everything this lane re-measured on §4's own rows reproduces:

| `LABEL_COUNTER.md` row | published | this lane |
|---|---:|---:|
| `leaf-dowhile` | 2 | **2** |
| `leaf-forever` | 4 | **4** |
| `leaf-goto-back` | 2 | **2** |
| `leaf-for-k` (the branch-free `mulli` body) | 3 | **3** |
| framed base `/Gy` / packed | 5 / 4 | **5 / 4** |
| leaf base, both modes | 1 / 1 | **1 / 1** |
| `for` **net of the helper pair** (`ctl-for`: stride 9, `minted` 7) | +2 | **+2** |

**Three things in the document are genuinely wrong or missing, and they are
named:**

1. **There is no EH row** (§4's table). `w-main` added one to a measured-not-
   modelled table; this lane measures a **two-handler `try`** at **stride 28
   (`/O1`) and 25 (`/Ox`)**, `minted` **28 / 24**. The EH charge is almost
   entirely a **minting** charge, which is why it is the one shape this lane
   predicted correctly from a formula before compiling (§4).
2. **§4.1's *"a per-function `.ex` field is the only unexamined channel"* is
   wrong about where to look.** The channel is not a field: it is that the IL
   already carries the front end's labels **with their ids**, and c2's charge is
   its increment over them.
3. **`CFG_SHAPE.md` §7's *"nothing should be derived from the `$LN` numbers"*
   should say more.** The numbers are a real, dense, per-function allocation
   ordinal from `DAT_10c2e918` — they are just **not** the global counter, and
   §3.3 shows they cannot be used as a proxy for it.

---

## 3. The five lanes, reconciled

### 3.1 The measurement that settles it

`work/wb-label/seedgrid.py` compiles each body **twice**: once in `w-json`'s
counterfactual form `[subject, z]`, once in the in-the-middle form
`[a0, subject, a1, a2]`.

| cell | body | counterfactual lead | **true in-TU stride** | true charge |
|---|---|---:|---:|---:|
| `s_ctl` | `return a+1;` | — | 1 | 0 |
| **`s_decl8`** | **the same body**, + 8 unused declarations | **+16** | **1** | **0** |
| `s_loc2` | 2 unused locals, straight line | +2 | 1 | 0 |
| **`s_loc8`** | 8 unused locals, straight line | **+8** | **1** | **0** |
| **`s_loop`** | `w-bdnz`'s `for` cell | **+7** | **3** | **+2** |
| `s_dowhile` | the `do/while` spelling | +5 | 2 | +1 |

> **Eight declarations that emit not one instruction move the counterfactual
> reading by SIXTEEN and the true charge by ZERO.** That is the whole channel,
> in one row. `s_decl8`'s obj and `s_ctl`'s obj differ in no `.text` byte.

`s_loop` reproduces **`w-bdnz`'s published `+7` to the digit** and its true
charge is **+2** — `LABEL_COUNTER.md` §4.2.1's `leaf-for` row exactly.

### 3.2 Lane by lane

| lane | number | right? | what it actually measured |
|---|---|---|---|
| **`w-json`** (#1800–#1812) | lead **4** where the table predicts 2 | **the number is right, the interpretation is wrong** | `Δseed + Δcharge`. The 2 the table predicts is the charge; the balance is the seed (and, per §4's own warning box, any `minted` surcharge the cell's body obliges). Its counterfactuals at 0/2/3 going red and 4 landing is consistent: with a counterfactual instrument you are fitting the displacement, and only one value of it reproduces the obj **for that TU** |
| **`w-osfinfo`** (#1760–#1771) | *"the lead is the count of unconditional intra-section `b` words"* | **a fit, refuted, and now explained** | It is a proxy for *"how many join labels did c2 have to invent"*. It works while c2's invented labels and the surviving `b` words correspond and decouples the moment a fold removes one — which is exactly what `w-xlr` found one lane later (predicted 3, forced 2). The **shipped `label_lead` of 1 for that class is still correct**; the *rule* is not |
| **`w-bdnz`** (#1980–#1988) | **+7** `/O1`, **+8** `/Ox` | **the numbers are right; the conclusion "the table is six low" is WRONG** | Reproduced here at **+7** with a true charge of **+2**. Its own `lab_forever` row — *"two `int` locals cost +2 with no loop"* — is the seed showing through, and this lane's `s_loc2` reproduces it at **+2 with a true charge of 0**. **Its verdict (`None`) was right for a reason it did not have** |
| **`w-blockir`** (#2300–#2311) | **+10/+13**, **+11/+15** | **numbers right, "sub-shape dependence" is the SEED** | Its `lead_c` differs from `lead_a` by **one more array parameter**, and reads **+1** higher at `/O1` and **+2** at `/Ox`. That is a front-end symbol count, not a codegen sub-shape. Its verdict (`None`) was right |
| **`w-ifn`** (#2350–#2362) | stride **5/4**, four ways | **right, and its own banner is right** | It used the in-TU stride and got the charge. What it could not see was a **once-per-TU minting slot taken before the FIRST function's triple**, because it put the subject first. Its banner's rule — *put the subject in the MIDDLE* — is this lane's procedure, and this lane's grid uses it on every row |

### 3.3 What this lane could NOT explain, named

**`w-json`'s 4 is reconciled in kind but not in arithmetic.** This lane did not
recompile `w-json`'s cells, so the split of its 4 into seed and charge is
inferred from the mechanism, not measured. Registered as unexplained per
**P4.5**, which predicted exactly one such case.

**And the `/FAsc` listing did NOT become the instrument** — the second thing the
commission hoped for, and it fails cleanly:

```text
  p_dowhile   stride 2   prints exactly one label:  $LL3@p_dowhile
  p_forever   stride 4   prints exactly one label:  $LL3@p_forever
  p_goto      stride 2   prints exactly one label:  $top$2561
  p_mulli     stride 3   prints NO label
```

Two bodies with **identical 24 `.text` bytes** and **different charges (2 and
4)** print **the same label name at the same index**. A third with the same bytes
and the same charge as the first prints a *different kind* of label. **The
listing discriminates where the charge does not and agrees where the charge
differs.** `WB_LABEL_PREREG_R2.md` P7.4 is a **MISS** and P7.2
(`stride ≥ max($LN)`) is **violated on the first row measured** — because
`sym[+0x3f]` counts label *objects*, including the ones that arrived
pre-numbered from the IL and cost nothing.

> **`cl /FAsc` is closed as a route to the label charge**, by measurement, at the
> byte level, on the same triple that closed *"derive it from the blocks"*.

---

## 4. The frozen obj-check, cell by cell

Frozen in `WB_LABEL_PREREG_R2.md` §3 before the first `cl.exe`. Construction:
subject in the middle, `base` measured in the same obj, `minted` read on every
row. **Misses are retractions.**

| cell | predicted `/O1` | actual | predicted `/Ox` | actual | verdict |
|---|---:|---:|---:|---:|---|
| **X1** jump-tabled `switch`, 12 sparse arms | 20 | **6** | 19 | **7** | **MISS ×2** |
| **X2** `for` with an `if` inside | 13 | **7** | 12 | **7** | **MISS ×2** |
| **X3** `while` with an early `return` | 13 | **8** | 12 | **7** | **MISS ×2** |
| **X4** `try` with two `catch` handlers | **28** | **28** | **25** | **25** | **HIT ×2** |
| **X5** `switch` inside a `for` | 26 | **8** | 24 | **10** | **MISS ×2** |
| **X6** loop unrolled at `/Ox` only | 12 | **7** | ≥13 | **11** | **MISS** on magnitude |

**1 of 6 cells, 2 of 12 numbers.** The registered meta-predictions:

| # | registered | outcome |
|---|---|---|
| **P9.1** | ≥2 of six will miss | **HIT** — five did, and it was registered as the *expected* result, not a hedge |
| **P9.2** | the **minting** component is right on all six; every miss is **internal** | **HIT.** `minted` is **5** (`/O1`) and **3** (`/Ox`) — the plain framed value — on X1, X2, X3, X5 and X6, so their entire surcharge mints nothing. X4 mints **28**, equal to its stride | 
| **P9.3** | X5 is **not** additive | **MISS, and this is the lane's most useful miss.** `X5 = X1 + X2` **exactly, at both modes**: 3 = 1 + 2 and 6 = 3 + 3 |
| **P9.4** | X6's mode gap is strictly positive | **HIT** — lead 2 at `/O1`, 7 at `/Ox` |

**Why X4 is the one that hit.** EH charge is **minting** charge — every slot
buys a symbol record (`stride 28 == minted 28`). The minting population is
computable from a formula (`EH_RECORDS.md` §9.8's `11 + 5·S + E`); the internal
population is not. That is `P9.2` landing in the sharpest possible form, and it
is the single most useful line in this section for a conversion lane.

**Why X1/X2/X3/X5 came in at a fifth of the prediction.** The prediction assumed
c2 mints a label per case arm and per join. It does not: those labels arrive
**pre-numbered in the IL** (§1.3). Retracted in §7.

### 4.1 The held-out grade of the additivity rule (R3)

X5's additivity was a fit to one cell, so `WB_LABEL_PREREG_R3.md` froze a
primitive table and six **held-out** compositions before compiling them.

Primitive leads, `/O1`: `if` **0**, `if/else` **0**, `switch` **1**,
`do/while` **1**, `for` **2**, `while` **2**.

| cell | `Σ lead` `/O1` | actual | `Σ lead` `/Ox` | actual |
|---|---:|---:|---:|---:|
| H1 `if` in a `while` | 2 | **2** ✓ | 11 | **3** ✗ |
| H2 `if/else` in a `for` | 2 | **2** ✓ | 10 | **3** ✗ |
| H3 two sequential `if`s | 0 | **0** ✓ | 2 | **1** ✗ |
| H4 `switch` in a `while` | 3 | **3** ✓ | 13 | **6** ✗ |
| H5 `for` in a `for` | 4 | **4** ✓ | 20 | **10** ✗ |
| H6 `do/while` in an `if` | 1 | **1** ✓ | 2 | **2** ✓ |

**`/O1`: 6 of 6.** **`/Ox`: 1 of 6.** P11.1 (≥5 at `/O1`) **HIT**; P11.2 (≤2 at
`/Ox`) **HIT**; P11.4's named risk cell H5 **did not** miss. **P11.5 MISS** — no
`/Ox` cell took the GPR helper pair, where ≥3 were predicted to.

> **CONSTRUCT-ADDITIVITY HOLDS AT `/O1` AND FAILS AT `/Ox`**, 6/6 against 1/6, on
> cells frozen before they were compiled. The `/Ox` failure has a visible cause:
> `p-for`'s `/Ox` lead is **10** while the same loop with an `if` in its body
> (`X2`) is **3** — at `/Ox` a loop's charge is a property of what the unroller
> did, and the keyword predicts nothing.

---

## 5. The deliverable a conversion lane needs

### 5.1 The procedure — sound, and it costs one build

> **DO NOT USE THE COUNTERFACTUAL FORM.** `[subject, control]` against
> `[leaf, control]` measures `Δseed + Δcharge`, and Δseed is a function of the
> **source text**: eight unused declarations move it by **+16** (§3.1). Four
> lanes used it and four lanes got a number that is not the charge.

1. **Put the subject in the MIDDLE**, one TU per subject:
   `a0 · P · a1 · a2` with `a0/a1/a2` a plain framed call.
   `scripts/gt_label_stride.py` already implements exactly this, and
   `work/wb-label/labgrid.py` is a 150-line copy if you need new probes.
2. **Measure `base` in the same obj** as `first(a2) − first(a1)`. It must be
   **5** under `/Gy` and **4** packed. If it is not, the row is void — do not
   report it.
3. **`stride(P) = first(a1) − first(a0) − base`.** This is what
   `coff::plan_labels` consumes; nothing else is.
4. **Read the `minted` column and subtract minting surcharges** before you call
   anything a control-flow lead. A loop body that spills callee-saved registers
   pays **+2** for the `__savegprlr_N`/`__restgprlr_N` pair *and* a control-flow
   charge; `ctl-for` is stride 9 / `minted` 7 and its control-flow part is **2**.
   `LABEL_COUNTER.md` §4's warning box says this and it is the column a
   re-derivation drops.
5. **A once-per-TU slot is invisible to a stride if the subject is first**
   (`w-ifn`) **and a wrong charge on the LAST function moves nothing**
   (`w-blockir` #2305). Step 1 fixes both by construction — that is why the
   subject goes in the middle, not for tidiness.
6. **At `/O1` you may PREDICT before you build**, with the primitive table in
   §4.1 (6/6 held out), and then confirm with **one** compile. **At `/Ox` you may
   not** (1/6) — measure.

### 5.2 Is there a closed-form rule?

**At `/O1`, over source constructs, yes and it is additive** — and it is
**still not a rule the port may use**, because the port reads IL, not source, and
the quantity the table is additive over is *how many labels c2 invents*, which no
IL field carries. It is a **measurement aid**: it turns four counterfactual
builds into one prediction plus one confirming build.

**At `/Ox`, no.** Not a mode correction to the `/O1` table either — the same
composition rule scores 1/6.

### 5.3 `label_slots`' parameters — the answer

> **NEITHER a mode parameter NOR a sub-shape parameter. `w-bdnz`'s argument
> (#1983) SURVIVES, and its reason is replaced by a stronger one.**

`w-bdnz` argued `None` because the charge is mode-dependent and `label_slots`
has no mode parameter. That is true but it is the weaker half. The stronger
reason:

* the charge is **the number of labels c2 invents beyond the ones the IL already
  numbered**, and **the IL does not carry that number**;
* adding a mode parameter would let a class be right at `/O1` and wrong at
  `/Ox` — but adding a *sub-shape* parameter is worse, because `w-blockir`'s
  "sub-shape dependence" was **the seed**, so a sub-shape arm would be fitted to
  an artifact (**the fifth instrument-measures-itself instance**, memory index
  `ranking-instruments-measure-themselves`);
* and #1761 is the standing precedent: a **fit** written into `label_lead` was
  refuted by the next obj, one lane later.

**What a class may ship is a measured constant for that class, obtained by §5.1,
and nothing else.** `None` remains correct for every unmeasured class. This lane
changes **no line of `crates/`**.

### 5.4 What would change the answer, stated in advance

If a future lane finds a per-function IL field that carries c2's invented-label
count, §5.3 is void and `label_slots` should read it. This lane looked in the
place `§4.1` named (a `.ex` field) only indirectly and did **not** rule it out;
what it rules out is *the listing* and *the emitted bytes*.

---

## 6. What this lane leaves NOT MODELLED

| # | open |
|---|---|
| 1 | ~~**The nine.**~~ **ANSWERED 2026-08-22 by read R3** ([`ref/P_LABEL.md`](ref/P_LABEL.md) §4, [`WB_LABELCHARGE_FINDINGS.md`](WB_LABELCHARGE_FINDINGS.md) §5, board #3388): the gap is **`7 + 2·[/Og] + 1·[/GF ∧ a string literal pooled in the data phase]`**, measured over 22 cells — **7** at `/Od`/`/Os`/`/Ot`/`/Oy`/`/Ob2`, **9** at `/Og`/`/Ox`, **10** at `/O1`/`/O2`/`/Ox /GF` with a pooled string. So *"nine allocations"* is refuted as a fixed count. **It does NOT move with section needs**, which is what this row asked: a `.data`, `.bss` or `.rdata` global moves it by **zero**, because the default segment's standard sections take ids from a **reserved low region** and charge nothing. **Which units make the 7/9/10 is still not enumerated** — bounded to five named once-per-TU sites; per-unit attribution needs a live tap on `0x10b97de5`. |
| 2 | **The `/Gy` "+3 per function"** is re-confirmed on 22 of 22 rows here (`first(a0)` at `/O1` minus at `/Ox` is `3 × nfuncs` on every one, EH included) — but **what** the three are is not read out of the binary. **STILL OPEN after R3** — re-confirmed a third time as an exact `3 × nfuncs` in every `/Gy` cell of `gt_label_seedgap.py`'s grid, and the three are still unread (`WB_LABELCHARGE_FINDINGS.md` P3.5, a registered MISS). |
| 3 | **`w-json`'s 4** is reconciled in kind, not in arithmetic (§3.3). |
| 4 | **The `/Ox` loop charge.** `p-for` 10, `X2` 3, `X6` 7, `H5` 10 — four magnitudes, no rule, and this lane does not propose one. |
| 5 | **A third once-per-TU minting slot** (P8.3) was predicted and not looked for. `_fltused` and `memcpy` remain the only two known, and the mechanism (§1.3) says the list is open, not closed. |
| 6 | **The downward pool.** `DAT_10c2ed40` and the crossing check are read but the interaction between the two ends is unmeasured — no probe here came near it. |

---

## 7. Retractions

| # | retracted |
|---|---|
| **R1** | **`WB_LABEL_PREREG_R2.md` §3's X1, X2, X3, X5, X6 predictions** — five cells, ten numbers, all missed, off by up to a factor of three. The error is named: the model assumed c2 mints a label per case arm and per join, and those labels arrive pre-numbered in the IL. |
| **R2** | **Round 1 `P2.1`** — *"every row of §1.1's surcharge table equals the number of COFF symbol records that surcharge causes c2 to mint"*. Refuted by the base rows: a framed function at `/Ox` strides **4** and mints **3**; a leaf strides **1** and mints **0**. Minting *causes* charge; charge is not *equal to* minting. |
| **R3** | **Round 1 `P2.2`** — the `/Gy`−packed difference as the `.pdata` COMDAT section symbol. Registered as the weakest link and it is refuted by the leaf rows. |
| **R4** | **Round 1 `P2.5`** in its strong form — *"there is no closed-form function from source construct to charge"*. At `/O1` there is one, and it survived a 6/6 holdout. The claim is narrowed to `/Ox` and to *"not a function of anything the port reads"*. |
| **R5** | **Round 2 `P7.2` and `P7.4`** — the `/FAsc` listing as an instrument for the internal population. Refuted on the first row measured. |
| **R6** | **Round 3 `P11.5`** — `minted` +2 on ≥3 of the six `/Ox` compositions. It was **0** on all six. |

**Not retracted, and worth saying so:** `P9.1` (≥2 misses of six) and `P11.2`
(≤2 hits at `/Ox`) were registered pessimistic *before* the builds and both
landed. A lane that predicts its own miss rate correctly is reporting a model,
not a hope.

---

## 8. DISCLOSURE pre-drafts

Nothing in this lane is adopted into `crates/`, so **no disclosure row is owed
today**. These are pre-drafted for the rung that adopts one.

| tier | finding | what adoption would look like | debt |
|---|---|---|---|
| **TIER 2** | `LABEL_SEED_GAP = 9` is nine **allocations**, not an offset | Only if the nine are enumerated and the constant becomes computed rather than fitted. Today the constant is **black-box fitted** (`OBJ_GY_SHAPES.md` §3.4/§3.5, 25 TUs) and the white-box reading merely **explains** it. **Explaining a black-box constant incurs no debt; replacing it with a disassembly-derived one does.** | none today |
| **TIER 2** | the counter is `DAT_10c2edd0`, incremented only at `0x10b97de5` | Nothing in the port reads a c2 address. Cited in docs only. | none |
| **TIER 2** | c1xx and c2 share the id space; IL labels arrive pre-numbered (`+0x28 = FUN_10c1f91b()`, no bump) | **This is the one to watch.** If a lane ever teaches the port to read label ids out of the IL stream to compute a charge, that decoder's *shape* is disassembly-derived and owes a row naming `FUN_10c1f91b`'s call site and the `+0x28` field. | owed **on adoption** |
| **TIER 1** | `$LC`/`$LL`/`$LN` are one formatter selected by `sym[+0x43]` bits `0x10`/`0x4` | Observable from any `/FAsc` listing — the listing is named in ROADMAP §9.8 as a black-box observable. **Tier 1, no debt.** | none |
| **TIER 2** | `sym[+0x28]` is the id for `$M`/`$T`/`$S`/`$E`/`__catch$`/`__unwind$`; `sym[+0x3f]` for `$L*` | Confirms `wb-eh`'s reading and extends it. Docs only. | none |
