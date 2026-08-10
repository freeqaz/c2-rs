# w-decouple — the two name sets **DO** separate, the separation is **free** (`fnbyte-exact` **+0** where the naive widening cost −1), both gate-blocked frontier TUs **bind**, and the clause that pays for the widening closed a **live wrong emit that was already on master**

    Tag:       w-decouple
    Slug:      w-decouple
    Date:      2026-08-09
    Fixtures:  wdec_ecshort_leaf.cpp  wdec_ecshort_eight.cpp  wdec_ecshort_mix.cpp  wdec_ec_varargs_neg.cpp  wdec_ec_varargs_long_neg.cpp  wdec_ec_localcall_neg.cpp
               — six new, 363 → 369. Plus one PRE-EXISTING cell whose *claim*
               the split retired and which this lane did not author:
               `il_extern_c_name.cpp`, amended rather than deleted (§9 P21).
    Census:    per-function **714,541 → 714,541**, emitted **39,241 → 39,241**.
               TU match **23 → 23**, mismatch **0 → 0**, codegen-gap **0 → 0**,
               vocab-gap **848 → 848**, port-error **0 → 0**, capture-fail
               **7 → 7**, FRONTIER **4 → 4**. **`fnbyte-exact` 35,810 → 35,810
               (+0)**, `fnbyte-differs` **1,898 → 1,898**, `fnbyte-partial`
               **10 → 10**, `fnbyte-refused` **114,622 → 114,622**. **261
               `gap-metric` keys at both ends: 0 vanished, 0 appeared, 0
               changed.** Factors A/B/C/D/E **28 / 338 / 169 / 23 / 2**, unmoved.
    Record:    this file; PREREG `work/w-decouple/PREREG.md`, committed at
               **`e0205d97`** — before the first `crates/` change of any kind.
               Scored in §9.
    Lane:      `w-decouple`, worktree branch `worktree-agent-ad5efaf54aa302999`,
               off master **`1326c86f`** (the `w-wordwrap` merge). Merge-base
               re-checked before reporting; master did not advance during the
               lane. Workload stamp **dc3
               `a8cb9ca639df2e938553ae24200307fa7a31abce`**, tracked tree clean
               (0 dirty lines at `--untracked-files=no`) — **it has moved since
               `w-front5`'s `d7a3c1aa`**, and every inherited figure below was
               re-derived rather than quoted. **878** lines in
               `work/dc3-workload/files.txt`, `wc -l`-checked, used AS COMMITTED
               and never regenerated (#2700). Toolchain
               `compilers/X360/16.00.11886.00`; wibo from
               `../wibo/build/release/wibo`. **369** fixtures, list regenerated
               after the last fixture and `wc -l`-checked against
               `ls fixtures/cpp/*.cpp`. Base binary
               `work/w-decouple/c2rs-base`, md5
               **`2b866cf73559305d3ccde85bb3692fb4`**, built at the merge-base
               and **KEPT** (#2409, #2512).
    Ships:     `c2_il::func::gl::NameFit`, `gl_bound_names`,
               `GlBindStop::VariadicRecord`, `FN_FLAG_VARARGS` +
               `record_is_varargs`; the `fit`-gated clause in
               `gl_defined_names_framed`; `Bindings::per_record` and
               `diag::decode_causes` moved onto the binding policy;
               `cause::GL_VARARGS_RECORD`. Six fixtures. Two amended docs and
               one amended test. Board rows **#2750**–**#2762**;
               **#2763**–**#2779** left explicitly unminted.
    Adopts:    **nothing.** No `docs/whitebox/DISCLOSURE.md` row, no constant
               from a table. `FN_FLAG_VARARGS` is this lane's own GRID-V
               measurement over its own captures.
    Confirms:  `w-front5` #2621/#2624 (both stop records) as **outcomes**,
               re-derived at a dc3 that has moved. `w-fence2` GRID-K's flags
               byte, from the other side (`0x20` is not `0x40`). `w-xtea3`
               NC-5, on a second TU.
    Retires:   **#2623's "two call sites" as a count** — there are **three**,
               and the third is the one that decides whether the repair is
               free. **w-front5 §9 item 3** (*"`unclaimed-gl-symbol` on
               `Main.cpp` and `mmio.cpp` … Whoever takes either TU pays it"*) —
               it is discharged by the same body, on both TUs. **The `Main.cpp`
               and `mmio.cpp` binding layer as a price** — it is paid.

---

## 1. THE RESULT

> ### **THE TWO SETS SEPARATE, AND THE SEPARATION IS FREE.** `fnbyte-exact` **35,810 → 35,810**, delta **exactly 0**, on the row `w-front5` lost at **−1**. **0 of 878** TU verdicts move at L1, **0 of 878** move on any `emit` byte triple, **0** of 261 `gap-metric` keys vanish, appear **or change value**, and `mismatch` is **0** on 878 TUs and on 369 fixtures at **both** `/O1` and `/Ox`. Board **#2750**.

> ### **AND BOTH GATE-BLOCKED FRONTIER TUs BIND.** `src/Main.cpp` and `src/xdk/nuispeech/mmio.cpp` go from `gate_cause = gl-stop-name-not-mangled` to `gate_cause = body-out-of-class`. They are ordinary priced rows now, which is what the frontier needed. Board **#2751**.

> ### **THE CLAUSE THAT PAYS FOR THE WIDENING CLOSED A WRONG EMIT THAT WAS ALREADY ON MASTER.** `fixtures/cpp/wdec_ec_varargs_long_neg.cpp` — `extern "C" int v_long_name_here(int a, ...)` — grades **`Port=Mismatch`, bytes diverge**, under the **base** binary, at `/O1` *and* `/Ox`. An undecorated variadic name longer than eight bytes has BOUND since `w-extdata` replaced `looks_mangled` with a length test (#1721), and `bind::mangled_is_varargs` is `ends_with("ZZ")` and can never see it. The emit is **28 bytes and a whole `.pdata` section** short. Board **#2752**.

> ### **GRID-V — THE `.gl` DEFINED RECORD'S FLAGS BYTE CARRIES A VARIADIC BIT AT `0x40`.** Ten defined records in one TU: **3 of 3** variadic carry it, **7 of 7** others do not, across both linkages, both name lengths and both mangling states, cross-checked on `?cppv@@YAHHZZ` where the byte and the name test can both speak and **agree**. The reason it had to be read: `cva.cpp` and `cnv.cpp` have **byte-identical `.ex`** — 2,751 bytes, `cmp`-checked — and objs of **36 B / 6 sections** against **8 B / 5**. Board **#2753**.

> ### **`gl-stop-name-not-mangled` IS RETIRED FROM THE WORKLOAD.** First cause on **15** TUs and present in **16** cause sets at base; **0** and **0** at tip. Of the sixteen: **2 bind**, **8** fall through to `gl-stop-26-introduced`, **6** to `bind-record-count-ne-segments`. **#2627's denominator was 15 and the set is 16** — `src/system/synth_xbox/PitchCorrectedVoice.cpp` carries it behind `drectve-not-boilerplate`. Board **#2751**.

> ### **THE BRIEF'S OWN ITEM-8 INSTRUMENT IS WRONG ON BOTH TARGETS — the FOURTH instrument to be.** `emit-bound == emit-gate-segments` reads **1 == 1** on `src/Main.cpp` and **11 == 11** on `mmio.cpp`, and **neither binds**. `emit-*` is `EmitBinding`, a THIRD binding (#918); `fn_names` is the census's loose scan (#2621); the field that answers `CEILING.md` §11.4 item 8 is `gate_cause` / `gate_causes`, exactly as item 8 says. Board **#2754**.

| | base `1326c86f` | tip |
|---|---:|---:|
| **TU match** | 23 | **23** |
| mismatch · codegen-gap · port-error | 0 · 0 · 0 | **0 · 0 · 0** |
| vocab-gap · capture-fail | 848 · 7 | **848 · 7** |
| **FRONTIER** | 4 | **4** |
| factor A / B / C / D / E | 28 / 338 / 169 / 23 / 2 | **28 / 338 / 169 / 23 / 2** |
| **`fnbyte-exact`** | **35,810** | **35,810 (+0)** |
| `fnbyte-differs` · `-partial` · `-refused` | 1,898 · 10 · 114,622 | **1,898 · 10 · 114,622** |
| per-function · emitted census | 714,541 · 39,241 | 714,541 · 39,241 |
| **`gap-metric` keys** | 261 | **261 — 0 vanished, 0 appeared, 0 CHANGED** |
| **per-TU `class` SET (878, BY NAME)** | — | **0 moved** |
| **per-TU `emit` byte TRIPLE (878, BY NAME)** | — | **0 · 0 · 0** |
| per-TU `gate_cause` (878, by name) | — | **15 moved, every one directional** |
| fixtures at `/O1` (369) | 170 · **1** · 14 · 184 | **174 · 0 · 14 · 181** |
| fixtures at `/Ox` (369) | 146 · **1** · 19 · 203 | **150 · 0 · 19 · 200** |
| `crates/c2-il` unit tests | 598 | **600** |

**The one mismatch at base is this lane's own `_neg` fixture** and it is a real
one: see §5.

---

## 2. `CEILING.md` §11.4, RUN FIRST AND OFF THIS LANE'S OWN CAPTURE

Both TUs were re-captured at the workload's own flags
(`work/w-decouple/capture.sh`) and both reference objs at the same
(`work/w-decouple/refobj.sh`), on a **dc3 that has moved twice since the prices
this lane inherited** (`d7a3c1aa` → `a8cb9ca6`).

**Item 1 — ask the BYTE judge.** `frontier-codegen`, this scan: 4 TUs, 35
emitted functions, `exact 12 · wrong 0 · cg-ref 0 · reader 23 · ungraded 0`,
`partition-broken 0`. The measurable codegen price of this frontier is still
**zero**; 65 % of it is behind a reader refusal.

**Item 8 — quote the GATE's number.** The item that decided the lane, and the
brief's suggested field is wrong (§1, board #2754). `gate_causes` at base:

| TU | `gate_causes` at base | `emit-bound` / `emit-gate-segments` | binds? |
|---|---|---|---|
| `src/Main.cpp` | `[gl-stop-name-not-mangled, body-out-of-class]` | 1 / 1 | **NO** |
| `src/xdk/nuispeech/mmio.cpp` | `[gl-stop-name-not-mangled, body-out-of-class]` | 11 / 11 | **NO** |

`work/w-decouple/glwalk2.py` — a whole transcription of
`gl_defined_names_framed`, **all five stop clauses**, runnable under either
policy, because `w-front5`'s `glwalk.py` transcribes the first two only and that
is enough to name a stop and not enough to say what a widened walk does next —
re-derives both stop records at this base: `main`, 4 B, one record whose offset
2713 **is** the one `.ex` segment's start; `mmioSeek`, 8 B, after four bound.
Both hold.

**Item 3 — read the reference obj's SYMBOL TABLE.** §6 and §7. `w-main`'s P12 is
confirmed byte for byte a second time, and mmio's table produced this lane's
sharpest finding (§7.2).

**Item 4 — is the refusal LIST MEMBERSHIP?** Yes, and that is the whole lane:
`runs[k].2.len() <= INLINE_NAME_MAX`.

**Item 5 — do not trust the reported key's LAYER; and GREP THE WORKLOAD.**
Grepped: `gl-stop-name-not-mangled` is the first cause on 15 TUs and appears in
**16** cause sets, so the class has a population and it is not one. It converts
**2 of 16** into ordinary rows and **0** into matches, which is the honest
version of what a "population" buys here.

**Item 6 — check factor A.** Both targets are inside `A∧B∧C`; `match 23` +
`frontier 4` = `A∧B∧C 27`, which closes.

**Item 7 — check the board.** `grep`ed before sizing. #2622/#2623 price the
naive widening at −1 and **no row prices a decoupling**; #2374 measures the
`ecshort` refusal and never the accept.

**Item 9 — if T1 fires, read the port's own FENCES.** T1 does not fire on
either TU (`Main.cpp` 0 of 1 exact, mmio 10 of 11), but item 9 earned its place
anyway: §7.2 is an NC-5 that is *behind* a body rather than in front of it.

---

## 3. THE DESIGN — three call sites, not two

`gl_defined_names` is read at **three** places, and #2623 names two. The third
is the one that decides whether the repair is free.

| site | what it asks | policy | changed? |
|---|---|---|---|
| `bind::Bindings::per_record` | *"what name does each defined body carry, so the writer can emit its symbol?"* | **`NameFit::InlineOrStringTable`** | **yes** |
| `diag::decode_causes` | transcribes the above, so `causes.is_empty() == decodes` holds | **`NameFit::InlineOrStringTable`** | **yes** |
| `bind::defined_name_set` | the **census**'s inline-fence ground set | `NameFit::StringTableOnly` | **no** |
| `gl::plain_external_defined_names` | the **gate**'s W-FENCE2 fence EXEMPTION | `NameFit::StringTableOnly` | **no** |

### 3.1 Why the separation does not weaken either side

**The widening is MONOTONE, and that is an algebraic property rather than a
measurement.** `gl_defined_names_framed` refuses on a **whole-TU** basis — every
stop clause `return`s `Err` and `gl_defined_names` maps every variant to the
same empty pair. So the wide walk can differ from the narrow one **only on a TU
where the narrow one returned nothing**, and on exactly those TUs both fence
sets are already `∅`. Keeping them narrow therefore leaves the fence not merely
*safe* but **bit-identical on every TU in the workload**.

* `defined_name_set`'s stated property is *"a subset of the truth and never a
  superset"*. A smaller subset is still a subset.
* The GATE's own inline fence reads **`Bindings::names()`**, which IS the
  binding — so on a newly-binding TU it gets **stronger**, not weaker:
  `defined` becomes the full name list where the TU used to be refused outright.

`crates/c2-il/src/func/gl.rs`'s
`the_fence_walk_never_grows_when_the_binding_walk_widens` asserts the invariant
directly — `narrow == wide ∨ narrow == ∅` over every cell the module builds —
so a future re-coupling turns a unit test red without a scan. That is the
must-fail for #2622's mechanism, and it costs nothing to run.

### 3.2 The residue, named and then SIZED as a build

`plain_external_defined_names` is an **exemption**, and widening an exemption is
the *licensing* direction. It keeps the narrow walk, which means: on a
newly-binding TU with an intra-TU call edge, `exempt` is `∅` and the gate
refuses wholesale at `locally-defined-callee` rather than handing the TU to
`c2_core::comdat::fenced_inlined_callee`.

**That is not an argument, it is one row**, and mutation **M3** measures it
(§8): giving the exemption the wide walk moves `wdec_ec_localcall_neg.cpp` from
`vocab-gap` / `[locally-defined-callee]` to **`codegen-gap` / `[]`** — the gate
passes and the composition seam refuses instead. Neither is a match and neither
is a mismatch. **Not shipped**: the licensing direction is the one this project
does not take on an unmeasured population.

**The argument FOR widening it, recorded because it is the strongest one and
this lane did not take it**: GRID-K's own witnesses are `k_ext_a`, `k_cfi_a` and
`k_exp_a` at **seven** bytes and `k_stat_a` at **eight** — so W-FENCE2's
three-byte linkage reader was fitted on a population its own ground set cannot
reach.

### 3.3 The widening pays for itself — GRID-V

`bind::mangled_is_varargs` is `name.ends_with("ZZ")`, and its own doc names the
coupling it rests on:

> *"An `extern "C"` variadic function has an undecorated name and is invisible
> here. That is covered, for a different reason that must not be quietly relied
> upon: `gl_defined_names` accepts only `?…@@…` forms… **If that ever loosens,
> this gate stops covering C variadics** — measured today (`extern "C" int
> cva(int, ...)` is `Port=NotImplemented`), and stated here so the coupling is
> visible."*

`NameFit::InlineOrStringTable` is that loosening. The replacement is the
record's own flags byte, `work/w-decouple/probe/vgrid.cpp` read by
`work/w-decouple/gridv.py`:

| record | bytes at `name_nul+1..+6` | what it is |
|---|---|---|
| `v_s` | `86 01 05 04 **40**` | `extern "C"` VARIADIC, 3-byte name |
| `n_s` | `86 01 05 04 00` | its twin |
| `v_long_name_here` | `86 01 05 04 **40**` | `extern "C"` VARIADIC, 16-byte name |
| `n_long_name_here` | `86 01 05 04 00` | its twin |
| `fi_s` | `86 01 05 04 20` | `__forceinline` — GRID-K's F4 bit, **not** this one |
| `st_s` | `86 01 **03** 04 00` | `static` |
| `?cppv@@YAHHZZ` | `86 01 05 04 **40**` | a C++ VARIADIC — **the cross-check** |
| `?cppn@@YAHH@Z` | `86 01 05 04 00` | its twin |

**3 of 3 against 7 of 7**, and on `?cppv@@YAHHZZ` the byte and the incumbent
name test agree. The clause is guarded on `!looks_mangled` so it covers
**exactly** the population the name test cannot, and gated on the wide policy so
the narrow walk stays byte-for-byte the incumbent. It is **fail-closed** on an
unreadable record: a return size in its escaped form puts the flags byte
elsewhere, and a varargs question that cannot be answered must not answer *no*.

**And it is not a hypothetical.** §5.

---

## 4. WHAT MOVED, BY NAME — four levels

`work/w-decouple/NEUTRALITY.txt`, 878 TUs keyed by **`src`**, never by basename
(#2667: 878 collapse to 841 and a basename compare drops 37 rows while printing
"0 MOVED").

| level | population | moved | direction |
|---|---|---|---|
| **L1 `class`** | 878 workload TUs by name | **0** | 0 only-in-base, 0 only-in-tip |
| **L2 `gate_cause`** | 878 by name | **15** | every one out of `gl-stop-name-not-mangled`; **2 → `body-out-of-class`**, 8 → `gl-stop-26-introduced`, 5 → `bind-record-count-ne-segments` |
| **L2b `gate_causes` SET** | 878 by name | **16** | the 15 above plus `PitchCorrectedVoice.cpp`, which carried the clause behind `drectve-not-boilerplate` |
| **L3 `emit[fnbyte-exact]`** | 878 by name | **0** | — |
| **L3 `emit[fnbyte-differs]`** | 878 by name | **0** | — |
| **L3 `emit[fnbyte-refused]`** | 878 by name | **0** | — |
| **`gap-metric`, as a MAP** | 261 keys | **0 / 0 / 0** | vanished / appeared / **changed** |
| fixtures `/O1` | 369 by name | **5 at L1** | 3 new cells `vocab-gap → match`; `il_extern_c_name.cpp` `vocab-gap → match`; `wdec_ec_varargs_long_neg.cpp` **`mismatch → vocab-gap`** |
| fixtures `/Ox` | 369 by name | **5 at L1** | **the identical five, by name** |

**The per-TU byte triples for the two targets**, which is the reading the last
two lanes were asked for and the one that shows the lane cost nothing where it
changed the most:

| TU | `fnbyte` exact / differs / refused | `bytefrac` exact / denominator | base → tip |
|---|---|---|---|
| `src/Main.cpp` | 0 / 0 / 1 | 0 / 124 | **unchanged** |
| `src/xdk/nuispeech/mmio.cpp` | 10 / 0 / 1 | 256 / 380 | **unchanged** |
| `src/system/synth_xbox/FFT.cpp` | 1 / 0 / 115 | 8 / 6,624 | **unchanged** — the TU that carried **100 %** of #2622's −1 |

**`FFT.cpp` is the control and it holds.** Its `gate_cause` moves
(`gl-stop-name-not-mangled → bind-record-count-ne-segments`) and not one byte
of its emit does.

**The `gap-metric` block is bit-identical**, which is stronger than predicted
(P18 allowed the cause histogram to move as values). The first-cause histogram
is printed in the scan's prose and is not a `gap-metric` key, so the metric
block is untouched:

```text
base:  811 gl-stop-26-introduced · 15 gl-stop-name-not-mangled · 13 drectve · 7 bind-count · 2 body-out-of-class
tip:   819 gl-stop-26-introduced ·  0                          · 13 drectve · 12 bind-count · 4 body-out-of-class
```

---

## 5. THE WRONG EMIT THAT WAS ALREADY THERE

`wdec_ec_varargs_long_neg.cpp` is one line:

```cpp
extern "C" int v_long_name_here(int a, ...) { return a + 1; }
```

Under `work/w-decouple/c2rs-base` — the merge-base binary, kept — it grades
**`mismatch  (bytes diverge)`** at `/O1` and at `/Ox`. It is the only
mismatching row in either base fixture scan.

**Why it was reachable and invisible.** Sixteen bytes, so `INLINE_NAME_MAX`
never refused it; undecorated, so `looks_mangled` is false and
`mangled_is_varargs` is false; and the `.ex` of a variadic body is **identical**
to its non-variadic twin's, so no reader below the record can see it either.
`w-extdata` opened this in #1721 and its own rung says the clause's refusal
keeps *"exactly the uncharacterized half: a bound record name that FITS
INLINE"* — the half it let through was this one.

**What the obj says**, `work/w-decouple/probe/`:

```text
cnv.obj   .text  8 B   addi 3,3,1 ; blr                              5 sections
cva.obj   .text 36 B   7 register-home `std`s, then the same two     6 sections (+ .pdata)
```

28 bytes and a whole section. `Port=Mismatch @ offset 2` is `NumberOfSections`.

**This lane did not go looking for it.** It fell out of asking what the widening
would cost, which is the shape `CEILING.md` §11's NC-list is built out of: the
question *"what did the clause I am removing hold up?"* found something the
clause was not holding up on purpose.

---

## 6. `src/Main.cpp`, RE-PRICED AT THIS TIP

**Layer 0, the binding — PAID.** `gate_causes` is now
`[body-out-of-class, unclaimed-gl-symbol]`.

**`unclaimed-gl-symbol` is NOT a fifteenth mechanism, and w-front5 §9 item 3 is
retired.** The unclaimed set is exactly three runs —
`??0App@@QAA@HPAPAD@Z`, `??1App@@QAA@XZ`, `?Run@App@@QAAXXZ` — and all three
are **undefined externals in the reference obj that `main`'s own body calls**:
REL24 at `.text+0x28`, `+0x30`, `+0x38` and a second `??1App` at `+0x68` in the
funclet. `diag`'s accounting walks `funcs`, and `funcs` is empty because the one
body is out of class; its own comment says so — *"a gate that cannot fire on a
partial function list is not evidence that it would not fire on the whole
one."* **It discharges with the body it is behind.**

**Layer 1, the body and the obj**, off `work/w-decouple/ref/main.dump`. 8
sections, 36 symbols, 1,786 B. `.text` is **124 B holding two code regions** —
`main` at 0x08..0x54 and `__unwind$2585` at 0x54..0x7C — **two** `.pdata`, and a
**64 B `.rdata` with five relocations** carrying `__ehfuncinfo$main`,
`__unwindtable$main` and `$T2592`. Named obligations: `__CxxFrameHandler`
(ADDR32 at `.text+0`), `__ehfuncinfo$main` (ADDR32 at `.text+4`), the funclet as
a second region, the second `.pdata`, the `.rdata` EH set, and **six `$M`
labels** (`$M2590`, `$M2591`, `$M2594`, `$M2595`, `$M2597`, `$M2598`) with
`$T2596` and `$T2599`. `w-main`'s P12 confirmed byte for byte, a second time, on
a dc3 that has moved.

The one blocked body's key is `expr-call-in-expr-recv-object-then-op-0x5C`,
`cflow-straight`, 124 B, `fnbyte-refused`.

> **Price: `w-main`'s thirteen EH mechanisms + one reader clause. The binding is
> 0 and `unclaimed-gl-symbol` is 0.** w-front5's *"≥ 14, plus a fifteenth"*
> becomes **14** with both of its non-body terms struck off. Board **#2760**.

---

## 7. `src/xdk/nuispeech/mmio.cpp`, RE-PRICED AT THIS TIP — and it gained a mechanism nothing had named

**Layer 0, the binding — PAID.** 11 of 11 records bind, offsets exact.
`gate_causes` is `[body-out-of-class, unclaimed-gl-symbol]`. 10 of 11 bodies are
`fnbyte-exact` (**256 of 380 bytes**).

### 7.1 What is struck off

**`unclaimed-gl-symbol`**: one run, `?FreeHandle@@YAXPAX@Z`, an **undefined
external `mmioClose` calls** (`.text #14`, REL24 at `+0x60`). Same shape as
Main.cpp's — discharged by the same body. **Not a mechanism.**

**The obj shape is not one either**, and it is worth saying because the writer's
own comments talk about a *"packed single-`.text` writer"*: mmio's reference obj
has **18 sections with eleven separate `.text` COMDATs** (`chars 0x60401020`,
`sel=1`) and three `.pdata`. `EncryptXTEA.obj` — a TU that **matches** — has the
same shape at five. The port already emits it.

### 7.2 What it gained: `mmioClose → mmioFlush`, a KEPT call to an EIGHT-BYTE callee

`.text #14` (`mmioClose`, 124 B) at `+0x30`:

```text
   0030  4bffffd1  bl .-48       ; REL24 -> [33] mmioFlush
```

and `.text #10` is `mmioFlush` in its entirety:

```text
   0000  38600000  li 3, 0
   0004  4e800020  blr
```

**Eight bytes, and c2 kept the call.** All eleven of mmio's records read
`86 01 05 04 00` — plain external, the exact class W-FENCE2's exemption is
about — so the moment `mmioClose` parses, the gate asks
`callee_defined_here_unmodelled`, and this lane's answer is `exempt = ∅`
(§3.2), so mmio refuses at `locally-defined-callee`. Widen the exemption and
`comdat::fenced_inlined_callee` refuses instead, because `INLINE_DECLINE_BYTES`
is **128** and its rule is *"the port cannot prove c2 KEPT this"*.

**So mmio needs BOTH halves, and neither is in any published price for it.**
This is `CEILING.md` §11's **NC-5, second instance** — `w-xtea3`'s
`EncryptXTEA` was the first — with one difference that matters: there the fence
was the *last* blocker in front of five byte-exact bodies; here it is **behind**
a body that has not been written, which is why no instrument has seen it.

> **And that is also why GRID-W could not have seen it.** `w-fence2`'s band
> table reads **0–63 B: kept 0, inlined 5,881** over 7,552 sites, and its
> instrument is *"for every **IL call edge** to a callee its own TU defines"* —
> an IL call edge needs `IlFunction::callees()`, which needs the caller's body
> to parse. `mmioClose` is `Decline::Parse`. **A kept 8-byte edge exists in the
> workload and is outside the sample that says there are none**, because the
> sample's population is bounded by the reader — `CEILING.md` §11.4 item 8's
> shape, one instrument over. Board **#2756**. This lane did **not** re-run
> GRID-W and does not restate its integers; what it states is that this edge is
> not among them.

> **Price: `w-ifn`'s six codegen mechanisms for `mmioClose`'s 124 B
> `cflow-if-n` body (key `expr-cmp-eq`), + the fence exemption reaching this TU,
> + an NC-5 licence for an 8-byte kept callee. ≥ 8, with the binding and
> `unclaimed-gl-symbol` struck off.** Board **#2761**.

`mmioClose`'s body also carries an **indirect call through a loaded function
pointer** (`lwz 11,8(31)` → `mtctr 11` → `bctrl`), which is worth checking
against `w-ifn`'s six before anyone treats that six as complete.

---

## 8. THE MUTATION GRID — three cells, three graded, none vacuous

`work/w-decouple/mutate.sh`, committed **before** it was first run (#2668,
#2699), and it refuses outright on an uncommitted `crates/` or `fixtures/` edit.

| cell | clause deleted | graded |
|---|---|---|
| **M1** | `record_is_varargs` — the whole varargs conjunction | `wdec_ec_varargs_neg` and `wdec_ec_varargs_long_neg` both **`Port=Mismatch @ offset 2`**, `class mismatch`, at **both** modes. The three positive cells unmoved at `match`. #2698/#2699's rule met: the mutation deletes the **whole** conjunction and only the cells it fences go red |
| **M3** | the fence exemption's narrow walk | `wdec_ec_localcall_neg` **`vocab-gap` / `[locally-defined-callee]` → `codegen-gap` / `[]`** at `/O1`; a **no-op at `/Ox`** |
| **M4** | the binding's wide walk (re-couple) | all five cells → `vocab-gap`. The must-fail for the whole lane |

### 8.1 Two things the grid got wrong first, both recorded

**M3 read as vacuous because it was graded at the wrong MODE.** W-FENCE2's
exemption is gated on every segment being `/O1` (mode gate in the parser,
#1638), and `c2rs diff` defaults to `/Ox /GS- /c` — so the cell is a no-op at
the CLI's default mode **by construction**. That is the `/Ox`-half rule with its
sign flipped, and it cost one wrong conclusion before the grid was taught to run
both modes. It also read as vacuous because only `Port=` was being compared:
this clause never turns a refusal into a match, it moves the refusal from one
gate to a later one, so **the `class` and the `gate_causes` are the grading and
`Port=` is not**.

**The grid aborted before its own restore, once.** `gap` exits nonzero when a TU
mismatches — which is exactly what an M1 cell is for — and under `set -e` the
script died mid-run and left the tree mutated. The guard at the top of the
script caught it on the **next** invocation rather than the one that caused it.
#2699's restore trap, in a new place, and the fix is `|| true` on the one command
whose failure is the deliverable.

### 8.2 The cell that graded nothing, NAMED

`wdec_ec_localcall_neg.cpp` **as first written graded nothing at all**, at
either mode. Two-byte names (`cb` / `cf`) put it on `shape-token-unresolved`,
three gates before the exemption. The cause is a **third** name-length rule,
independent of `INLINE_NAME_MAX` (8) and of `looks_mangled`:
`gl::is_indexable_name` requires **`b.len() >= 3`**, so a two-byte callee never
enters `gl_symbol_index` and `Bindings::resolve` returns `None` for its call
token. The repair is **merging** (#2665/#2698), here in its source form: rename
to three bytes and the cell lands on `locally-defined-callee`.

Recorded rather than quietly fixed, because its sibling
`wdec_ecshort_eight.cpp` keeps a **one-byte** defined name and **matches**:
`Bindings::per_record` reads symbol RUNS and never the index, so the two readers
have different floors and only the one asked about a **callee** has this one.
Board **#2757**.

**After the merge, no cell in this grid grades nothing.**

---

## 9. THE PREREG, SCORED

Frozen at **`e0205d97`**, before the first `crates/` change. PREREG §0 is a
declared prior list and §0b is a findings-at-freeze list; both are **unscored on
purpose** (`w-pool` scored 30/31 and correctly called that a calibration
failure).

| # | p | outcome |
|---|---|---|
| **P1** | 0.88 | **HIT** — `defined_name_set` and `plain_external_defined_names` are bit-identical, and it is asserted as an invariant in `the_fence_walk_never_grows_when_the_binding_walk_widens` rather than inferred |
| **P2** | **0.90** | **HIT — the deciding row.** `fnbyte-exact` **35,810 → 35,810**, delta exactly 0, and its antecedent P5 holds, so it is scored and not VOID |
| P3 | 0.85 | **HIT** — `FFT.cpp` bit-identical in every `emit` key |
| P4 | 0.92 | **HIT** — mismatch 0 on 878 and on 369 fixtures at both modes |
| **P5** | **0.80** | **HIT** — both TUs bind; `gl-stop-name-not-mangled` leaves both sets |
| **P6** | **0.60** | **HIT on the number, MISS on the denominator** — exactly **2** bind, and the population is **16** cause sets, not #2627's 15. The extra is `PitchCorrectedVoice.cpp` |
| P7 | 0.75 | **HIT, both halves** — `unclaimed-gl-symbol` appears on both, and every unclaimed run is an undefined external a blocked body calls |
| **P8** | **0.70** | **MISS** — mmio does **not** gain `locally-defined-callee`. `decode_causes` asks that gate over the functions that DID build, and `mmioClose` is not one. The prediction was right about the EDGE (§7.2) and wrong about which instrument can see it — which is the same error the brief's item-8 field makes |
| P9 | 0.75 | **HIT** — three accepted cells, `vocab-gap` at base and `match` at tip |
| P10 | 0.60 | **HIT** — identical at `/Ox`, by name |
| **P11** | **0.65** | **HIT for M1, and the `_neg` cell it was written about was the WRONG ONE.** M1's mutation deletes the whole conjunction and both varargs cells go `mismatch`. The residue cell needed §8.2's merge first |
| P12 | 0.05 | did not fire (correct) — mmio does not convert |
| P13 | 0.01 | did not fire (correct) |
| P14 | 0.90 | **HIT** — match 23, FRONTIER 4 |
| P15 | 0.85 | **HIT** — L1 0 moved |
| **P16** | **0.80** | **MISS** — **15** rows move at L2, not 2. The prediction assumed the walk would only change where it BINDS; it changes wherever it runs FURTHER, and 13 TUs fall through to a later stop |
| P17 | 0.85 | **HIT** — L3 byte triples 0 / 0 / 0 |
| **P18** | **0.70** | **HIT, and stronger than written** — 261 keys, 0 vanished, 0 appeared, **0 changed**. The cause histogram is prose, not a `gap-metric` key |
| P19 | 0.75 | **HIT** — 28 / 338 / 169 / 23 / 2 |
| P20 | 0.90 | **HIT** — see §10 |
| **P21** | **0.55** | **HIT on a technicality that is the more interesting outcome.** No test FAILED. `an_undecorated_record_name_is_seen_then_refused` compiled and passed across the split **without a line changing**, because it calls the walk that is now the FENCE — so the thing that had to be re-pointed was a *claim*, not a compile. Amended, not deleted |
| **P22** | **0.55** | **HIT** — build M3 holds mismatch at 0 and differs from the ship on **exactly one** row. It differs in `class`/`gate_causes` rather than in `Port=`, which the prediction did not say |
| P23 | 0.35 | **HIT** — M3 converts nothing |

**19 of 23 hit; three misses and one split.** All three misses (**P6**, **P8**,
**P16**) are about **which population an instrument can see**, which is the
lane's own subject, and P8 is the one worth keeping: it predicted a mechanism
correctly and predicted the wrong instrument to find it in.

**The deciding row was P2 and its antecedent was P5**, both registered, both
scored, and the antecedent checked before the row was claimed — `w-wordwrap`'s
lesson, applied.

---

## 10. GATE

| lane | result |
|---|---|
| `scripts/gate.sh` | **GATE: PASS (HATCH-RED REFUSED)** — **18/18 lanes**, 0 FAIL, 0 SKIP, 0 NO-RESULT; **6,642 fixture-verdicts**; sweep **19,460 of 19,556** graded; cross **90,424 of 90,812** cells; **0 mismatch anywhere**; `ladder-red` PASS |
| `cargo test --workspace --release --no-fail-fast` (#2262) | **1,484 passed, 0 failed, 41 targets** (`c2-il` 598 → 600; `rung_registry` 2/2) |
| `c2rs selftest` | **369 PASS, 0 FAIL, 0 ERROR** |
| 878-TU workload scan | match **23** / mismatch **0** / `fnbyte-exact` **+0** / **0** L1 verdicts moved |
| fixtures at `/O1` and `/Ox` | 369 each, **0 mismatch at tip at either**, and the base's **1** is closed |

`work/w-decouple/gate_summary.txt` (machine paths scrubbed). The gate's
per-mode fixture rows read `369/369` on all fourteen modes.

### 10.1 The two test failures this lane caused, and what they were

`cargo test --workspace` came back **2 failed** on the first run, both in
`rung_registry` and both this lane's: the rung header's `Fixtures:` field held
prose instead of filenames, and `docs/rungs/INDEX.md` is **GENERATED** and was
stale. Fixed by naming the six fixtures and running
`scripts/gen_rung_index.sh`. Recorded rather than silently corrected, because
"the workspace suite was green" is only interesting if the run that was not is
also on the record.

### 10.2 Pre-existing, REPRODUCED AT THIS LANE'S EXACT BASE before attribution

`hatch-red` reports **`REFUSED HATCH-STALE`** — `hatch.py apply` cannot hatch
this tree, board **#1389**'s open shape, which exits 0 by design and forfeits
the unqualified headline (#1406).

**Reproduced, not argued**, and at this lane's own base: a detached worktree at
**`1326c86f`** with `work/w-hatch/hatch_red.py` copied in and run from inside it
gives

```text
   FAILED: R2 DIRTY+HATCH, R6 RESIDUE, A2 PAID-MISSING, F1 FORCE, C1 HATCH-ONLY
```

— **the same five arms, in the same order**, as `w-wordwrap` recorded at
`a179e8be`, `w-xtea3` at `299f9a8c` and `w-xtea2` at `af81b869`. Output kept at
`work/w-decouple/HATCH_RED_AT_BASE.txt`; the worktree was removed afterwards.
Every `crates/` change was committed before any gate row or mutation ran.

---

## 11. FOUND AND NOT TAKEN

1. **The fence exemption's wide walk.** Sized as build M3 (§3.2, §8): moves one
   row, converts nothing, and is the licensing direction. GRID-K's own witnesses
   argue for it. Not taken.
2. **`mmioClose`'s six + the NC-5 licence.** §7.2. The TU is now an ordinary
   priced row and this is its remaining chain; it is a lane.
3. **`main`'s thirteen EH mechanisms.** §6. Unchanged in content, and now with
   nothing in front of them.
4. **A `gate_binds` boolean and the stop RECORD as scan fields.** `w-front5`
   left this (its item 4) because it adds `gap-metric` keys to a zero-key-change
   lane. This lane is also a zero-key-change lane and leaves it for the same
   reason — but it now has a second argument: **four different fields have been
   used to answer item 8 and three of them were wrong** (`fn_names` #2621,
   `emit-bound`/`emit-gate-segments` #2754, and the census's own binding).
5. **The 13 TUs that fall through.** 8 to `gl-stop-26-introduced` (board #232's
   COMDAT-`.text` shape) and 5 to `bind-record-count-ne-segments`. Both are
   whole classes and neither is a name question.
6. **`is_indexable_name`'s `len() >= 3`.** §8.2. A third name-length rule, now
   reachable by a defined callee for the first time. Unmeasured against the
   workload; no TU is known to need it.
