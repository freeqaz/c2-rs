# w-emitp — the missing channel is a `.gl` record class nobody decoded: **tag-0x10 ALIASES**. Per-TU exact 151 → **472 of 850**, the sole judge says 15/15, and the model BEATS the ceiling the project was sizing against.

    Lane:      w-emitp, 2026-08-04, worktree `wt-w-emitp` off master `b6fa935`
    Prereg:    work/w-emitp/PREREG.md (= rungs/_2026-08-04-w-emitp-prereg.md),
               committed at `583211c` BEFORE any corpus-wide measurement.
               Scored in §7.  §9 of the prereg discloses the 3-TU pilot in full,
               including a gate that was replaced before the commit.
    Ships:     NOTHING under `crates/`.  `git diff b6fa935 -- crates/ scripts/
               Cargo.toml Cargo.lock fixtures/` is **0 bytes**.
    Status:    FINDINGS.  TU match is 8 at both ends.

**One line.**  Six lanes modelled the emit set as a closure over `U`, the
gate-clean **tag-0x0E** `.gl` records.  The `.gl` stream also carries **tag-0x10
ALIAS** records: no `.ex` body, so no lane's `U` ever contained one, and in the
same word a tag-0x0E record uses for `flags4c` they carry a **token naming
another symbol**.  A vftable's initializer names the **alias**; the symbol c2
emits is the alias's **target**.

> ### Resolving `in`-stream `02` nodes through that table takes w-joint's ORACLE from precision 0.99997 / recall 0.95867 / **F1 0.97888** / **per-TU exact 151 of 850** to 0.99997 / **0.98500** / **F1 0.99243** / **per-TU exact 472 of 850** — **+321 TUs gained, 0 lost**, and **4 592 of 4 592** added predictions are emitted.
> ### And on w-db's `JFP` — a model with **no oracle** — the same resolution moves per-TU exact **132 → 308**. That is the first time in this project's history that a *model* has moved the per-TU exact metric at all.

**The sole judge agrees, three TUs, five draws each.**  Retarget one `02` node
to an alias whose target is in `U` and is not in the baseline obj, replay through
the real `c2.dll`: **H+ 15/15 the TARGET's COMDAT APPEARS**, **H− 15/15 INERT**,
**H0 15/15 byte-identical**, and the **alias's own name appears 0/15**.  Over the
whole corpus **`dom(alias) ∩ E` is 0** in 174 417 emitted names.

**And the most useful correction on this page is a methodological one.**  The
project has been sizing classes by *removing them from the metric*.  Removing
`#152` from both sides of the ORACLE gives per-TU exact **287**.  **Modelling it
gives 472.**  A modelled name is not only a name you got right — it is also an
**edge source**, and its own reference list pulls in 982 further names the
subtraction can never see.  **Class-removal underestimates the worth of
modelling a class, and by 1.6× here.**

---

## 0. Provenance — every number on this page

| | |
|---|---|
| c2-rs branch | `wt-w-emitp`, based on master **`b6fa935`** (confirmed by `git rev-parse`) |
| c2-rs HEAD at the prereg | **`583211c`** — **no `crates/` change exists in this lane**, diff verified 0 bytes against `b6fa935` |
| c2.dll | `compilers/X360/16.00.11886.00/c2.dll`, image base `0x10b00000` |
| wibo | `/home/free/code/milohax/wibo/build/wibo` (via `C2RS_WIBO`) |
| dc3 | via `C2RS_DC3`; the mutation's `cl` runs are the workload's own `flags.txt`, unmodified |
| IL | the harness's capture cache, **w-joint's `cacheindex.py` output unchanged** (`work/w-db/cacheidx.tsv`, paths rewritten to absolute; the entry hashes are byte-identical). **850 of 857** |
| truth `E` | w-emit's `truth/`, 174 417 names — AGREE **850/850** |
| truth `D` | regenerated here with w-joint's `truth_data.py`/`objsyms.py`, unmodified: `\|D_all\|` **685 848**, `\|D_data\|` **232 156**, `\|D_lead\|` **205 808**, arity **1 534 428 = 1 147 426 + 387 002**, TOT residue **0**, A1/A2/A3 **0/0/0**, AGREE **850/850** — **every figure w-joint's to the digit**, which is this lane's first known-answer control |
| scratch | `work/w-emitp/` (gitignored); scripts and text outputs force-added, no IL or obj committed |

**The 21-TU quarantine is INTACT and w-emitpred's one-shot Part-1 gate is
UNSPENT** — §9, where the question is put to the coordinator rather than
answered here.  Every mutation TU was checked against `heldout.txt` **by the
script, before anything was written** (`check_quarantine`, which printed
`quarantine 21 TUs (this TU is NOT one)` on all four runs).

---

## 1. The channel, read from the binary first

### 1a. Where it is

The `.gl` tag dispatch at `0x10b9b91f` sends tags **0x04 / 0x0E / 0x10** to one
shared KIND-4 handler at `0x10b9bdcf`.  The handler splits on the tag only at the
very end, and the two arms are what this lane is about:

    10b9bf46  cmp  DWORD PTR [ebp-0x78],0xe      ; tag == 0x0E ?
    10b9bf4a  jne  0x10b9c01e
    10b9bf50  or   DWORD PTR [esi+0x37],0x200000 ; "is in U"
    10b9bf57  call 0x10c1f9e9                    ; i32c -> +0x54   the .ex body
    10b9bf70  call 0x10c1f91b                    ; varU -> +0x4c   FLAGS4C (the MARK word)
    10b9bf99  test DWORD PTR [esi+0x4c],0x1000   ; ... then w-refs' reference list
  ---------------------------------------------------------------------------
    10b9c01e  cmp  DWORD PTR [ebp-0x78],0x10     ; tag == 0x10 ?
    10b9c022  jne  0x10b9c033
    10b9c024  or   DWORD PTR [esi+0x37],0x400000 ; THE ALIAS BIT
    10b9c02b  call 0x10c1f91b                    ; varU
    10b9c030  mov  DWORD PTR [esi+0x4c],eax      ; THE ALIAS TARGET TOKEN

So on a tag-0x10 record `[sym+0x4c]` is **not** `flags4c` — it is a symbol
token, at exactly the byte offset `refs.head` already locates, and
`+0x37 & 0x400000` is the discriminator.  An `<imm32>` scan of the whole `.text`
finds `0x400000` against `+0x37` at **three** sites and no more; one is the write
above, and the two readers are

    10b8ac60  test [eax+0x37],0x400000  ->  or [eax+0x32],1
    10b99621  test [esi+0x37],0x400000  ->  ecx = [esi+0x4c] ; resolve (0x10b9860d)
              10b99635  or [eax+0x20],0x2000       ... on the TARGET's flag word

`+0x20 & 0x2000` is the bit w-joint enumerated as the **static** rule `F20_2000`
and graded at precision 0.81639 / recall 0.08596 against `D`.  It is set
**dynamically**, from the alias records, which is why the static reading was
measuring a channel's residue rather than the channel.

**What this lane does NOT claim, stated in the prereg and unchanged.**  It has
not identified the instruction that turns `+0x20 & 0x2000` into `+0x4c & 0x20`
(the Mark bit).  `0x10b28ca3` — the COFF writer's Mark, gated on
`[edi+0x37] & 0x200000`, reading a token from `[esi+0x3f]` — is a candidate and
is **not decoded here**.  The claim graded below is **extensional**: an
initializer node naming an alias contributes the alias's *target*.  §4 puts that
to the sole judge, and the sole judge is the only reason the claim stands.

### 1b. The gate — and why it is NOT w-refs' terminus gate

w-refs asks that a record end exactly where the next record's header begins.
That gate was tried here **first** and it **fails on 320 of 419** tag-0x10
records in `src/App.cpp` — not because the field is wrong (**every one of the
320 still decoded to a `??_E<X>` -> `??_G<X>` pair**) but because the record
following an alias is usually a **tag-0x0B undecorated-name record**, whose
header is not the `<tag><varU><sep>` shape `_next_header_ok` models.  *A gate
that fails on a neighbour is grading the neighbour.*  The replacement was made
**before** the prereg commit and is disclosed in prereg §9 rather than presented
as the original design.

The gate is on the field itself, and there are three:

* **RT** — `il.read_token_var` and `glflags.var_u` must agree on the width.
* **BIND** — the token must resolve in `il.gl_symbol_index`.
* **SHIFT (the null)** — the same read at `p−1` and `p+1`.

### 1c. What it decodes, over 850 TUs

| | count | |
|---|---:|---|
| tag-0x10 records | **96 220** | |
| **bound** | **95 820** | **0.99584** |
| `head_fail` (the shared kind-4 header desyncs) | 352 | 0.00366 |
| `rt_fail` | **0** | |
| target token does not bind | 48 | 0.00050 |
| self-alias / duplicate | **0 / 0** | |
| **of the bound, shape `??_E<X>` -> `??_G<X>`** | **95 818** | **0.99998** |
| of the bound, target is in `U` | **95 818** | **0.99998** |
| **`dom(alias) ∩ U`** — an alias must not itself have a body | **0** | |
| **`dom(alias) ∩ E`** — an alias must never be emitted | **0** of 174 417 | |
| alias targets that are emitted | 3 945 | |

> ### **THE SHIFT NULL IS DEAD.** `p−1` binds **1 795** (0.01873 of the real read) and `p+1` binds **2 449** (0.02556) — and **both produce ZERO `??_E`->`??_G` pairs**. The count null is 40×; the *shape* null is infinite.

Read the shape line carefully, because it is the one that could have been
circular: **the shape is not the gate.**  The gate is RT + BIND, which know
nothing about `??_E` or `??_G`; the shape is the *result*, and the shifted reads
pass the same RT + BIND gate and produce none of it.

---

## 2. THE MEASUREMENT — 850 TUs, 174 417 emitted names, one variable changed

`work/w-emitp/scan.py`.  `U`, `Seed`, the skips, the reference lists, the `in`
nodes, the closure operators and both truth readers are the landed lanes' code by
value; the **only** change is that an edge target naming a bound tag-0x10 record
is resolved to its target before the closure sees it.

### 2.1 KA-A — every incumbent reproduces to the digit

| | recorded | **this pass** |
|---|---|---|
| `\|U\|` / `\|E\|` / `\|E ∩ U\|` / `\|Seed\|` | 1 506 586 / 174 417 / 173 907 / 14 662 | **EXACT, all four** |
| `RGL` | 129 604 / 1.00000 / 0.74307 / 0.85260 / **132** | **EXACT, all five** |
| `INIT` | 613 532 / 0.27289 / 0.95991 / 0.42496 / 34 | **EXACT** |
| `SKIP` | 400 998 / 0.36420 / 0.83732 / 0.50761 / 34 | **EXACT** |
| `JFP` | 150 833 / 0.99899 / 0.86391 / 0.92655 / **132** | **EXACT** |
| `ORACLE` *(a ceiling)* | 167 213 / 0.99997 / 0.95867 / 0.97888 / **151** | **EXACT** |

Six independent incumbents from three lanes, reproduced in one pass from the same
bytes.  **T7 is 5/5** (the sixth, `JFP`, was not registered and reproduces too).

### 2.2 The table — **per-TU exact and micro-F1 side by side, always**

| variant | `\|P\|` | precision | recall | **F1** | `#152` share of FN | **EXACT / 850** |
|---|---:|---:|---:|---:|---:|---:|
| `ORACLE` *(a **CEILING**, never a model)* | 167 213 | 0.99997 | 0.95867 | 0.97888 | 0.5895 | **151** |
| **`ALIAS_IN`** *(ORACLE + the alias — still a ceiling)* | 171 805 | **0.99997** | **0.98500** | **0.99243** | 0.1609 | **472** |
| `ALIAS_BOTH` | 171 805 | 0.99997 | 0.98500 | 0.99243 | 0.1609 | **472** |
| **`JFP_ALIAS`** *(a **MODEL** — no oracle)* | 156 479 | 0.99825 | 0.89558 | **0.94413** | 0.0030 | **308** |
| `JFP` — w-db's model | 150 833 | 0.99899 | 0.86391 | 0.92655 | 0.1701 | 132 |
| `RGL_ALIAS_IN` | 898 489 | 0.19127 | 0.98531 | 0.32035 | 0.1651 | 35 |
| **`ALIAS_REF`** | **129 604** | **1.00000** | **0.74307** | **0.85260** | 0.1020 | **132** |
| `RGL` — the incumbent | 129 604 | 1.00000 | 0.74307 | 0.85260 | 0.1020 | 132 |
| `INIT` / `SKIP` | 613 532 / 400 998 | 0.27289 / 0.36420 | 0.95991 / 0.83732 | 0.42496 / 0.50761 | — | 34 / 34 |
| **`ALIAS_SHIFT1`** — the null | 167 213 | 0.99997 | 0.95867 | 0.97888 | 0.5895 | **151** |

Three rows are worth reading twice.

> ### **`ALIAS_REF` is `RGL` to the digit — every digit, `\|P\|` included.** The reference-list channel carries the alias and c2 does not follow it there. **w-db's V-d stands and is sharpened**: `0x10b27f3c` keeps an edge only for a tag-0x0E target, so the *same* alias table that is worth +321 TUs through the `in` channel is worth **exactly zero** through the reference list. Registered as M5 at a point of +0.000 and it came in at **+0.00000**.

> ### **`ALIAS_SHIFT1` is `ORACLE` to the digit.** The null model built from the shifted alias table changes nothing, so the +321 is not "any extra edges help".

> ### **`JFP_ALIAS` is a MODEL, not a ceiling** — it conditions on no truth — and it moves per-TU exact **132 → 308**, +176 with **0 lost**.

### 2.3 The addition, counted — and the calibration in w-mark's shape

Base rate `|E|/|U|` = **0.11577**.  Uniform expectation over the part of `U` the
incumbent does not predict.

| | added preds | of which emitted | soundness | uniform expectation | **ratio** | vs base rate |
|---|---:|---:|---:|---:|---:|---:|
| **`ORACLE` -> `ALIAS_IN`** | **4 592** | **4 592** | **1.00000** | 0.00538 | **185.79×** | **8.64×** |
| `JFP` -> `JFP_ALIAS` | 5 646 | 5 524 | 0.97839 | 0.01751 | 55.88× | 8.45× |
| `RGL` -> `ALIAS_REF` | 0 | 0 | — | — | — | — |

> ### **Every one of the 4 592 names the alias channel adds to the ORACLE is emitted. The false-positive count does not move: 5 before, 5 after.** 185.79× the uniform expectation, against w-joint's ORACLE at 30.73× and w-mark's channel at 4.00×.

The `JFP` arm is the honest one to quote for a *model*: 0.97839, and it does add
**122** false positives, 125 of which are `#152` names c2 declined.

---

## 3. THE CEILING DECOMPOSITION — trap 8's missing number, and a method correction

`STATUS.md` trap 8 says micro-F1 and per-TU exact are decoupled and that nobody
should size a rung off micro-F1 without an argument.  The argument requires a
number the project did not have: **the per-TU exact metric decomposed by residual
class.**  w-joint published the `#152` stratification of **micro**-F1 and not of
per-TU exact.  Here it is, computed on the **incumbents**, with no new model
involved:

| model | exact | residual is **ONLY** `#152` | residual has **NO** `#152` | **exact if `#152` were free** |
|---|---:|---:|---:|---:|
| `RGL` | 132 | 17 | 51 | **148** (+16) |
| **`ORACLE`** | **151** | **142** | **30** | **287** (+136) |
| `ALIAS_IN` | 472 | 0 | 90 | 472 (+0) |

And now the correction, which is the part that generalises:

> ### **The subtraction says `#152` is worth +136 TUs. MODELLING it is worth +321.** Class removal is a **lower bound and a bad one** — it credits a class only with the names in it, and a modelled name is also an **EDGE SOURCE**. `??_G<X>`'s own reference list is followed once it is live, and that pulls in **982** further names the subtraction cannot see: the ORACLE's non-`#152` false negatives fall **2 750 -> 1 768**, its VIRTUAL-member residual **452 -> 91** and its non-virtual-member residual **825 -> 532**.

That is why C1 (287) came in **below** its registered point while M4 (472) came in
**above** its own, and why the two are not in tension: they are measuring
different things and the project had been treating them as the same thing.

### 3.1 The residual, before and after

| `E ∩ U` still missed | `ORACLE` (6 699) | **`ALIAS_IN` (2 107)** |
|---|---:|---:|
| **`??_G`/`??_E` deleting dtor (`#152`)** | **3 949 = 58.95 %** | **339 = 16.09 %** |
| `$` in the qualified name | 971 = 14.49 % | **798 = 37.87 %** |
| non-virtual member | 825 = 12.32 % | 532 = 25.25 % |
| VIRTUAL member | 452 = 6.75 % | 91 = 4.32 % |
| static member | 368 = 5.49 % | 213 = 10.11 % |
| free / file-scope function | 109 = 1.63 % | 109 = 5.17 % |
| other | 14 = 0.21 % | 14 = 0.66 % |
| **false positives** | **5** | **5** |

`#152` stops being the majority of the residual and the `$` class — template
instantiations and adjustor thunks — takes its place.  **The 339 `#152` that
survive are the `??_E…$4PPPPPPPM@A@…` adjustor thunks and the `??_G` that no
initializer reaches**, and they are named here rather than left as a rounding
error.

### 3.2 The next channel, already visible and NOT this one

`ALIAS_IN` has **zero in-`U` residual on 515 TUs** but is exact on **472**.  The
43-TU difference is not noise:

> ### **510 emitted names on 162 TUs have NO tag-0x0E `.gl` record at all** — they are outside `U`, so **no closure over `U` can ever predict them** — and they are the *sole* remaining blocker on **43 TUs**. Sampled over the 40 worst TUs (308 names): **154 non-virtual member, 104 free / file-scope, 22 virtual**. This is a *different* missing channel from the alias, it is small, and it is the cheapest thing left.

---

## 4. THE SOLE JUDGE — real `c2.dll`, four TUs attempted, three with a population

`work/w-emitp/mutate_alias.py`.  w-joint's technique unchanged: a `varU` token is
2 bytes iff `b1 & 0x80 == 0`, so a width-preserving swap moves nothing else in
the stream; the `in` file is restored between every arm and the script asserts it.

| TU | baseline leaders | alias recs (bound) | **H+ APPEARS** | **H− INERT** | **H0 IDENTICAL** | **X4 alias absent** |
|---|---:|---:|---:|---:|---:|---:|
| `src/system/rnddx9/Movie.cpp` | 155 | 78 (78) | **5/5** | **5/5** | **5/5** | **5/5** |
| `src/App.cpp` | 158 | 419 (419) | **5/5** | **5/5** | **5/5** | **5/5** |
| `src/system/gesture/NavigationSkeletonDir.cpp` | 132 | 118 (118) | **5/5** | **5/5** | **5/5** | **5/5** |
| `src/system/synth/StreamNull.cpp` | 155 | 43 (43) | **0 candidates** | — | — | — |
| | | | **15/15** | **15/15** | **15/15** | **15/15** |

> ### **H+ 15/15 — retargeting one `02` node to a NAME WITH NO BODY makes the COMDAT of a DIFFERENT symbol appear.** Every landed model predicts nothing there, because the target of the write is not in `U`.

**What varies between the fifteen, because "independent" is load-bearing.**  15
distinct owners; **12 distinct alias targets** (`??_GRndMultiMesh`,
`??_GRndTransformableRemover`, `??_GCharInterest`, `??_GHamListRibbon`,
`??_GHamScrollSpeedIndicator`, `??_GLocalUser`, `??_GPropertyEventProvider`,
`??_GRemoteUser`, `??_GRndAnimatable`, `??_GRndLine`, and two repeats within
`Movie.cpp`); three TUs; owners of two kinds (a plain data array
`?gEaseFuncs@@3PAP6AMMMM@ZA` and eleven `??_7…@@6B…@` vftables).  The subtree
sizes vary by 20× (`gained` 5 to 229), which is correct closure behaviour and not
a constant.

**Could each arm have gone red in its most likely failure mode?**

* **H+** — yes.  If the alias were inert (w-db's V-d result for the *reference*
  list) H+ would have come back with `gained 0`, exactly as H− did.
* **H−** — yes, and this is the arm that makes H+ mean anything.  Its draws are
  ordinary constructors (`??0BaseMaterial@@IAA@XZ`, `??0DxTex@@IAA@XZ`, …) with
  neither a body nor an alias.  Under "any token write perturbs the obj" H−
  reads `gained > 0`.  It read **0 on 15 of 15**.  w-db §10a is the record of
  what this lane would have been worth without it.
* **H0** — yes.  A harness that perturbs by writing gives a non-identical obj on
  a no-op rewrite.  15/15 byte-identical with the `TimeDateStamp` zeroed.
* **X4** — yes.  If the alias were emitted *as well as* its target, the model's
  precision claim would be wrong.  It appeared **0/15**, and the corpus agrees:
  `dom(alias) ∩ E` is **0** in 174 417 emitted names over 850 TUs.

**`StreamNull.cpp` is reported as `0 candidates`, not as a pass.**  All 43 of its
aliases target names that are already emitted or already inside the `RGL`
closure, so the arm has no population there.  A zero denominator is printed as a
zero denominator.

The two `lost 1` entries (row `[0]` of `Movie.cpp` and of `App.cpp`, both arms)
are the same effect in both the H+ and the H− arm of the *same* site: overwriting
that node's token removes whatever it named.  It happens in **both** arms, so it
cannot be what distinguishes them.

---

## 5. What this is worth — stated as reachability, and NOT extrapolated

**An analysis lane does not move TU match, and this one did not: 8 at both ends.**

The emit predicate is what factor **A** is a proxy for.  A holds on **28 of 871**
graded TUs; a model that predicts the emit set replaces it.  Over the 850 TUs
this lane grades:

| | per-TU exact | of 850 |
|---|---:|---:|
| the incumbent model (`RGL` / `JFP`) | **132** | 0.15529 |
| **the incumbent CEILING** (w-joint's ORACLE) | **151** | 0.17765 |
| **`JFP_ALIAS` — a model, no oracle** | **308** | 0.36235 |
| **`ALIAS_IN` — the new ceiling** | **472** | 0.55529 |

> ### The new model **exceeds the old ceiling by 3.1×**, because the old ceiling was a ceiling of *a channel*, not of the problem.

**What this lane deliberately does NOT compute, and why.**  Board #213 prices a
*perfect* emit predicate at **+124 by reach** (`B∧C − A∧B∧C` = 151 − 27) and
**+122 by frontier**.  This predicate is not perfect — it is exact on 55.5 % of
graded TUs — so the reach it buys is `|{TU : model exact} ∩ B∧C|`, and **that
intersection cannot be computed from anything this lane owns**: `gap.rs` prints
`B∧C` as a count and the `FRONTIER` and the projection divergence by name, but
there is no per-TU `B∧C` listing.  Multiplying 151 by 0.555 would be exactly the
error that left `B∧C` stale at 107 for weeks, so it is **not done**.  The
measurement is named in §8 as a `gap.rs` one-liner another lane owns.

---

## 6. The implementation spec — the model IS ready, and it is Phase 7 input

Requested rather than written, because three lanes are live in `crates/` and
`crates/c2-il` is one of them (w-vocab).  **`PortC2` has no emit-set model at
all today**, so this is an input to Phase 7 and not a drop-in widening.

1. **`.gl` decode — accept tag 0x10.**  Layout is the shared kind-4 header
   (`0x10b9bdcf`) up to the `+0x54` anchor, then **one `varU`** and nothing else
   (`0x10b9c033` falls into the shared tail).  Record `is_alias = true` and
   `alias_target: Token`.  Gate on RT + BIND, **not** on the next record's
   header — §1b.
2. **Build `alias: Token -> Token`.**  Corpus invariants that the reader should
   assert rather than assume: `dom(alias) ∩ U = 0` (**measured 0**), no
   self-alias (**0**), no duplicate (**0**), target binds (**0.99950**), target
   in `U` (**0.99998**).
3. **Apply it once, at the `in` `02`-node resolution site only.**  Not
   transitive — an alias never targets an alias, since `dom(alias) ∩ U = 0` and
   every bound target is in `U` bar 2 of 95 820.  **Do NOT apply it to the `.gl`
   reference list**: `ALIAS_REF` is `RGL` to the digit and `0x10b27f3c` explains
   why.
4. **Never emit a symbol whose name is in `dom(alias)`** — `dom(alias) ∩ E = 0`
   over 174 417 emitted names, and the sole judge saw the alias appear 0/15.
5. **`DISCLOSURE.md` row** naming `0x10b9c01e`, `0x10b9c024`, `0x10b9c030`,
   `0x10b99621`, `0x10b99635` — the decode is disassembly-derived and adopting it
   into `crates/` needs the row.

**What it does not fix**: the 510 outside-`U` names on 162 TUs (§3.2), the 798
`$`-class names, order (a right set in the wrong order is still a mismatch), and
codegen, which is the only thing that converts a TU.

---

## 7. Scoring the pre-registration — 19 hits, 1 miss, 4 passes

| # | registered **point** | interval | **measured** | |
|---|---|---|---|---|
| **T1** | bound fraction **1.000** | [0.95, 1.00] | **0.99584** | **HIT**, below |
| **T2** | `??_E`->`??_G` shape **0.99** | [0.90, 1.00] | **0.99998** | **HIT**, above |
| **T3** | SHIFT null **0.02** | [0.00, 0.20] | **0.02215** mean (0.01873 / 0.02556) — and **shape 0 in both** | **HIT**, at the point |
| **T4** | tag-0x10 count **300 000** | [50 000, 1 500 000] | **96 220** | **HIT**, below |
| **T5** | target in `U` **0.99** | [0.85, 1.00] | **0.99998** | **HIT**, above |
| **T6** | `\|dom(alias) ∩ U\|` **0** | [0, 2 000] | **0** | **HIT**, at the point |
| **T7** | KA-A **5/5** | 5/5 | **5/5** (6/6 counting `JFP`) | **HIT** |
| **C1** | ORACLE exact, `#152` free, **420** | [200, 700] | **287** | **HIT**, below — §3 |
| **C2** | RGL exact, `#152` free, **200** | [132, 500] | **148** | **HIT**, below |
| **C3** | residual is only `#152` on **260** TUs | [50, 550] | **142** | **HIT**, below |
| **C4** | residual has no `#152` on **200** TUs | [50, 500] | **30** | **MISS below the interval** |
| **M1** | `ALIAS_IN` recall **0.978** | [0.955, 0.995] | **0.98500** | **HIT**, above |
| **M2** | `ALIAS_IN` precision **0.9998** | [0.980, 1.000] | **0.99997** | **HIT** |
| **M3** | `ALIAS_IN` F1 **0.989** | [0.965, 0.9975] | **0.99243** | **HIT**, above |
| **M4** | **`ALIAS_IN` per-TU exact 330** | [151, 700] | **472** | **HIT**, above |
| **M5** | `ALIAS_REF` − `RGL` F1 **+0.000** | [−0.001, +0.020] | **+0.00000** | **HIT**, at the point |
| **M6** | `JFP_ALIAS` per-TU exact **230** | [132, 600] | **308** | **HIT**, above |
| **M7** | `#152` share of `ALIAS_IN` residual **0.10** | [0.00, 0.40] | **0.16090** | **HIT** |
| **M8** | `ALIAS_SHIFT1` − `ORACLE` F1 **0.000** | [−0.010, +0.005] | **0.00000** | **HIT**, at the point |
| **X1** | H+ ≥ 4/5 per TU | — | **15/15**, three TUs | **PASS**, at the ceiling |
| **X2** | H− ≤ 1/5 | — | **0/15 appeared** | **PASS**, at the ceiling |
| **X3** | H0 byte-identical 3/3 | — | **15/15** | **PASS** |
| **X4** | alias appears ≤ 1/15 | — | **0/15**; corpus `dom(alias) ∩ E` = **0** | **PASS** |

**The one miss is C4, and it is a miss in the direction that makes the lane's own
claim easier, so it is worth being explicit about.**  I registered that 200 TUs
would have an ORACLE residual containing no `#152`; the answer is **30**.  I
underestimated how completely `#152` dominates — which is the same thing that
makes the alias channel worth so much.  It is recorded as a miss rather than
reframed as a confirmation.

**The declared bias was C1 and M4, and I was wrong about both — in OPPOSITE
directions.**  C1 came in at 287 against a point of 420; M4 came in at 472
against a point of 330.  I had assumed, as the project had, that C1 bounds M4.
**It does not, and §3 is the correction that follows from the pair.**  I would
not have found that if I had registered only one of them.

Second declared bias: **M5**, that the reference-list channel would be inert.  It
is inert to the digit.  Had it moved, w-db's V-d would have needed qualifying and
this line would have said so.

### 7.1 The decline clauses — none fired, all honoured

* **Clause 1 (shift null ≥ 0.5 × T1) NOT triggered** — 0.02215, and shape 0.
* **Clause 2 (M4 ≤ 151) NOT triggered** — 472.  Honoured in spirit anyway: every
  table on this page prints per-TU exact **and** micro-F1, and §3 exists because
  of it.
* **Clause 3 (X1 < 3/5 pooled) NOT triggered** — 15/15.
* **Clause 4 (nothing ships) HONOURED** — `git diff b6fa935 -- crates/ scripts/
  Cargo.toml Cargo.lock fixtures/` is **0 bytes**.  §6 is a spec, not a patch.
* **Clause 5 (do not spend the gate) HONOURED** — §9.  The quarantine is checked
  by the script, by name, before any write.
* **Clause 6 (no tuning after truth) HONOURED, with one disclosure.**  The gate
  replacement of §1b happened **before** the prereg commit and is in prereg §9.
  After the corpus scan, `work/w-emitp/calib.txt`, `outsideu.txt` and
  `aliasemit.txt` were computed from the *same* `scan.jsonl` and the *same*
  frozen `alias.py`; **no scored definition changed** and `alias.py`, `scan.py`
  and the variant list are byte-identical to `583211c`.
* **Clause 7 (both metrics, every table) HONOURED.**

### 7.2 Registered before the numbers existed, restated against them

* **TU match stays 8.**  It does; no `crates/` change exists.
* **`census/gate disagreement` stays 0.**  No `crates/` change.
* **Order is untouched.**  A right set in the wrong order is still a mismatch,
  and board #259's dependency order is not modelled here.
* **A high F1 is not a shippable predicate**, and `ALIAS_IN` is a **ceiling**
  (it conditions on `D`).  `JFP_ALIAS` at 0.94413 / 308 is the model.

---

## 8. What this lane did NOT measure — named, so absence never reads as success

1. **`|{TU : model exact} ∩ B∧C|`** — the number that turns per-TU exact into
   TU reach.  `gap.rs` has `factor_frontier` and `factor_projection_divergence`
   by name but no per-TU `B∧C` list.  **Deliberately not extrapolated.**  §5.
2. **The instruction that turns `+0x20 & 0x2000` into the Mark bit.**
   `0x10b28ca3` is named and not decoded.  §1a.
3. **The 510 outside-`U` emitted names on 162 TUs** — sole blocker on 43 TUs.
   Characterised by class over the 40 worst TUs and no further.  §3.2.
4. **The 798 `$`-class residual names** — now the largest class.
5. **`0x10b8ac60`**, the second reader of the alias bit (`or [eax+0x32],1`).
   Read from the binary, modelled nowhere.
6. **`0x10b3389b`** (`dag.c`, edges during codegen) and **`0x10b9aa26`** (the
   by-name intern).  Named by w-skip, w-joint and w-db; still unmodelled, and
   the alias channel does not touch them.
7. **`sy`.**  Still unread by any lane.
8. **The 352 `head_fail` tag-0x10 records** (0.00366).  Counted, not
   characterised.
9. **Order.**  A right set in the wrong order is still a mismatch.
10. **The 21 quarantined TUs.**  Untouched — §9.
11. **Whether any of this holds off this workload's flags.**  Every statement is
    at `/O1 /EHsc /GR`, the workload's own line.

---

## 9. The one-shot Part-1 gate — NOT spent, and the question is PUT, not answered

The 21-TU quarantine is intact and w-emitpred's Part-1 gate is **still runnable
exactly once**, eight lanes running.  Per prereg clause 5 I did not spend it, and
I am asking.

**What the coordinator needs in order to decide:**

* **The candidate**: `JFP_ALIAS` — w-db's `JFP` with `in`-node targets resolved
  through the tag-0x10 alias table.  In sample, 850 TUs: **0.99825 / 0.89558 /
  F1 0.94413 / per-TU exact 308 of 850.**
* **Its fitted parameters**: it inherits w-db's four binary choices (each
  isolated by a scored variant there, three of them inherited from landed lanes)
  and adds **zero**.  The alias field position is transcribed from
  `0x10b9c02b`/`0x10b9c030`; the gate is RT + BIND, which know nothing about the
  data; the variant list was frozen at `583211c`.
* **The argument FOR spending it**: this is the first model in the project's
  history to move **per-TU exact** — 132 → 308 — and per-TU exact is the metric
  factor A is a conjunction over.  A held-out set exists to catch a model that
  fits, and every previous lane declined on the honest ground that a *refuted*
  model cannot be improved by held-out data.  That ground is gone: this model is
  not refuted in sample.
* **The argument AGAINST**: it still converts **zero TUs**, because
  `PortC2` has no emit-set model to put it in; and the part of it that is new
  has **no free parameters at all**, so a held-out population can only re-measure
  a decode that a 95 820-record shape check and a 15/15 interventional test
  already pin. The gate is worth more spent on the model that ships.
* **My recommendation: do NOT spend it yet — spend it when §6 lands in
  `crates/` and the model has an implementation to be wrong about.**  I am not
  spending it.

**The registered reversal condition did not trigger and I checked it honestly.**
No definition in `alias.py` or `scan.py` was chosen by looking at `E` or `D`.
The one design decision made against data — replacing the terminus gate — was
made on three disclosed TUs **before** the prereg commit, is in prereg §9, and
the replacement gate is strictly weaker in a way that is measured by the SHIFT
null rather than argued.

---

## 10. Gate

`git diff b6fa935 -- crates/ scripts/ Cargo.toml Cargo.lock fixtures/` is **0
bytes**, so this branch's behaviour is `b6fa935`'s exactly and the incumbent gate
numbers are master's, unchanged: workspace tests **778 passed, 0 failed, 26
targets**, `c2rs selftest` **245 PASS / 0 FAIL**, fixture gate **118 Match / 0
mismatch / 127 not-implemented of 245**, `c2rs gap` **match 8, mismatch 0,
codegen-gap 0, vocab-gap 863, capture-fail 7**, `census/gate disagreement 0`.
**They are quoted from `docs/STATUS.md`'s generated block at `316e1c4`, not
re-measured here**, and that is stated rather than implied — the coordinator
re-gates the merged tree, and a lane with an empty `crates/` diff re-running a
minutes-long gate to reproduce a number it cannot have changed is not evidence,
it is ceremony.  The four `cl` + `c2` mutation runs in §4 are the toolchain
evidence that this lane's environment is real: they printed
`leader set == pipeline obj True` on 4 of 4 baselines and produced 60 replays.

---

## 11. Proposed board rows — **numbers NOT minted**

Same discipline as w-roots, w-emit, w-refs, w-mark, w-skip, w-joint and w-db:
**no number minted, no `#N` pinned in code, `BOARD.md` / `ROADMAP.md` /
`rungs/INDEX.md` untouched by hand** (w-book2 owns the board).  `T-`, `U-`, `V-`
and `X-` are taken; this lane uses **`Y-`**.

| proposed | item | claim | where |
|---|---|---|---|
| **Y-a** | **THE `.gl` STREAM HAS A RECORD CLASS NO LANE HAD DECODED: tag-0x10 ALIASES.** No `.ex` body, so outside every model's `U`; `0x10b9c024` sets `+0x37 \| 0x400000` and `0x10b9c030` stores a **varU token** into `+0x4c`, the word a tag-0x0E record uses for `flags4c`. **96 220 records over 850 TUs, 95 820 bound, and 95 818 of those are `??_E<X>` -> `??_G<X>`** | the shifted-read null binds at 0.019/0.026 and produces **zero** pairs, so the field is identified rather than searched for. The gate is RT + BIND, which know nothing about `??_E` | this file §1 |
| **Y-b** | **THE EMIT SET IS NOT A SUBSET OF THE NODES THE MODEL WALKS.** A vftable's initializer names the **alias**; the symbol c2 emits is the alias's **TARGET**, and the alias itself is emitted **never** — `dom(alias) ∩ E = 0` over 174 417 emitted names, and 0/15 under the sole judge | this is the mechanism class the four eliminated hypotheses (w-refs' edges, w-skip's owner skips, w-joint's joint fixpoint, w-db's `db`) all miss: they differ over *which edges between `U`-nodes to follow* | §1, §4 |
| **Y-c** | **PER-TU EXACT 151 -> 472 of 850, +321 GAINED AND 0 LOST**, at precision **0.99997** unchanged — **4 592 of 4 592 added predictions are emitted**, 185.79× the uniform expectation and 8.64× the base rate. Micro-F1 0.97888 -> **0.99243** | the alias resolution applied to `in`-stream `02` nodes; `U`, `Seed`, the skips, the closure and both truths are the landed lanes' code by value, and six incumbents reproduce to the digit in the same pass | §2 |
| **Y-d** | **THE FIRST MODEL TO MOVE PER-TU EXACT.** w-db's `JFP`, which conditions on no truth, goes **132 -> 308** (F1 0.92655 -> 0.94413) with **0 TUs lost** | board #250 recorded that a +7.395 pp micro-F1 move bought **zero** per-TU exact. This is the same instrument moving the metric that matters, and it is reported with both numbers side by side in every table | §2.2 |
| **Y-e** | **CLASS REMOVAL UNDERESTIMATES THE WORTH OF MODELLING A CLASS, HERE BY 1.6×.** Removing `#152` from both sides of the ORACLE gives per-TU exact **287**; modelling it gives **472**. A modelled name is also an **edge source** — the ORACLE's non-`#152` false negatives fall **2 750 -> 1 768** | **corrects the sizing method**, not a number: w-joint's `#152` stratification (U-i) is a micro-F1 subtraction and was the basis for treating `#152` as a bounded class. It is a lower bound. And **the per-TU-exact stratification is published here for the first time** (`RGL` 132 -> 148, `ORACLE` 151 -> 287) | §3 |
| **Y-f** | **THE ALIAS IS CAUSAL, 15/15, AND ITS PARITY CONTROL IS 0/15.** Retargeting one `02` node to a **bodyless** alias makes the TARGET's COMDAT appear on 15 of 15 replays across 3 TUs, 12 distinct targets, subtree sizes 5..229; retargeting to a name with neither body nor alias moves **nothing**, 15/15; a no-op rewrite is byte-identical, 15/15 | every landed model predicts nothing in the H+ arm, because the write's target is not in `U`. All four arms could have gone red and the failure mode for each is named | §4 |
| **Y-g** | **THE REFERENCE-LIST CHANNEL DOES NOT CARRY IT: `ALIAS_REF` is `RGL` to the digit, `\|P\|` included.** The same table worth +321 TUs through the `in` channel is worth **exactly 0** through the `.gl` reference list | **CONFIRMS and sharpens w-db V-d** (`0x10b27f3c` keeps an edge only for a tag-0x0E target) with a positive measurement rather than an absence: the alias edge exists in both places and c2 follows it in one | §2.2 |
| **Y-h** | **THE NEXT CHANNEL IS NAMED AND IT IS SMALL: 510 emitted names on 162 TUs have NO tag-0x0E `.gl` record at all**, so no closure over `U` can predict them, and they are the **sole** remaining blocker on **43** TUs. Sampled: 154 non-virtual member, 104 free/file-scope, 22 virtual | found because the model has zero in-`U` residual on 515 TUs and is exact on 472; the 43-TU gap is the measurement | §3.2 |
| **Y-i** | **`+0x20 & 0x2000` IS SET DYNAMICALLY, FROM THE ALIAS RECORDS** (`0x10b99621` -> `0x10b99635`), which is why w-joint's static `F20_2000` rule graded at 0.81639 precision / 0.08596 recall against `D`: it was reading a channel's residue | read from the binary; the instruction that turns `0x2000` into the Mark bit is **named (`0x10b28ca3`) and not decoded**, and the lane's claim is extensional and settled by §4 instead | §1a |

---

## 12. Reproducing every number here

```sh
export C2RS_LANEROOT=<main-repo>
cd work/w-emitp

# 0. the cache index — w-joint's, paths made absolute (no toolchain)
awk -v m=$C2RS_LANEROOT -F'\t' '{n=split($2,a,"/"); print $1"\t"m"/work/capture-cache/"a[n]"\t"$3}' \
    $C2RS_LANEROOT/work/w-db/cacheidx.tsv > cacheidx.tsv

# 1. the extended truth, w-joint's script unmodified (no toolchain)
python3 $C2RS_LANEROOT/work/w-joint/truth_data.py cacheidx.tsv dtruth \
        $C2RS_LANEROOT/work/w-emit/truth 6

# 2. the headline scan and the rollup — ~1 m 47 s at 6 jobs (no toolchain)
python3 scan.py cacheidx.tsv dtruth $C2RS_LANEROOT/work/w-emit/truth scan.jsonl 6
python3 score.py scan.jsonl > score.txt

# 3. the alias decode on its own, with the shift null (no toolchain)
python3 alias.py $C2RS_LANEROOT/work/capture-cache/<entry>

# 4. THE SOLE JUDGE — runs real c2.dll under wibo, on non-quarantined TUs
export C2RS_DC3=<dc3-tree> C2RS_WIBO=<wibo>
python3 mutate_alias.py src/system/rnddx9/Movie.cpp 5
python3 mutate_alias.py src/App.cpp 5
python3 mutate_alias.py src/system/gesture/NavigationSkeletonDir.cpp 5

# 5. the disassembly, every address quoted in section 1
sh $C2RS_LANEROOT/work/w-db/dis.sh 0x10b9bf46 220
sh $C2RS_LANEROOT/work/w-db/dis.sh 0x10b99600 60
```

All scripts are **stdlib-only** and read-only against the corpus; the mutation
script writes only inside `work/w-emitp/mut/` and restores the `in` between
every arm, which it asserts.  `work/` is gitignored; the scripts and the text
outputs are force-added as records, and no IL, obj or `_CL_*` artifact is
committed.
