# w-db — `db` is DEBUG and c2 never reads it. The model that predicts `D` works — F1 0.97075 against the best static rule's 0.11019 — and its own mutation refutes its mechanism, so it ships as a CORRELATION.

    Lane:      w-db, 2026-08-04, worktree `wt-w-db` off master `669ee6c`
    Prereg:    work/w-db/PREREG.md (= rungs/_2026-08-04-w-db-prereg.md),
               committed at `be53663` BEFORE any corpus-wide measurement.
               Scored in §7.  §9 of the prereg discloses the 3-TU pilot in full,
               including the one parameter chosen against data.
    Ships:     NOTHING under `crates/`.  No fixture, no codegen, no widening,
               no DISCLOSURE.md row.
    Status:    FINDINGS.  TU match is 8 at both ends.

**Decline clause 4 fires, so its result goes first.** ***The lane's registered
§0 — that a reference from an emitted function to a data symbol is the missing
`code -> data` edge — is REFUTED by this lane's own mutation, and I withdraw
it.*** Retargeting a `.gl` reference-list token whose target is a DATA symbol,
in a function c2 **does** emit, through the real `c2.dll`, changes the obj by
**zero bytes**: `gained=0 lost=0` on **10 of 10** replays across two TUs, the old
data symbol is not lost and the new one does not appear. **w-skip T-e and
w-joint's "there is NO code->data edge" are CONFIRMED as claims about the
mechanism** — `0x10b27f3c` drops non-function targets and nothing downstream
reads them. The instrument control is green and it is the lane's own first
design: writing at the *same* byte class with a *function* target moves **56 to
120 defined symbols per replay**, so the write reaches c2 and c2 acts on it.

**And the model still works, which is why the page is not a pure refutation.**
Over 850 TUs, 174 417 emitted names and a **702 263-name owner population**:

> ### CODE, against `E`: precision **0.99899**, recall **0.86391**, **F1 0.92655** — **+7.395 pp over w-refs' 0.85260**, on the far side of the registered 0.87260 wash bar.
> ### DATA, graded **DIRECTLY against `D`** for the first time: precision **0.98167**, recall **0.96007**, **F1 0.97075** — against the best of w-joint's twelve static rules at **0.11019**.

**But the payoff metric does not move, and that is the most important sentence
on this page.** `JFP`'s per-TU exact set is **w-refs' 132, name for name — 0
gained, 0 lost.** Micro-F1 moved 7.4 pp and per-TU exact moved by **zero**.

**`db` is answered and it is a null with a mechanism.** It is the **debug**
sub-stream, ordinal 4, and the workload has no `/Zi`, so `[module+0xcd8] &
0x2000` is clear and **c2 never reads it**. Removing the file entirely leaves the
obj **byte-identical** on 3/3 TUs; **0 of 685 848** defined symbols and **0 of
174 417** emitted functions occur in it as a string. Three lanes pointed at `db`;
it is not the instrument.

---

## 0. Provenance — every number on this page

| | |
|---|---|
| c2-rs branch | `wt-w-db`, based on master **`669ee6c`** (the merge of `wt-w-joint`) |
| c2-rs HEAD at the prereg | **`be53663`**, clean — **no `crates/` change exists in this lane** |
| **dc3-decomp HEAD BEFORE** | **`940d07dcb0960964ad61aa5f025658f993eb46b2`** |
| **dc3-decomp HEAD AFTER** | **`940d07dcb096…`** — **it did not move** (`work/w-db/prov_{before,after}.txt`) |
| wibo | **`1.0.1-23-g4a9dd6f`**, checked at lane start (`work/w-db/wibo.txt`) |
| c2.dll | `compilers/X360/16.00.11886.00/c2.dll`, sha256 `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`, image base `0x10b00000` |
| IL + obj | the harness's capture cache, w-joint's `cacheindex.py` unchanged, `tree 940d07dc…+clean` + workload argv signature. **850 of 857**, the 7 misses w-emit's 7 by name. 55 226 rejected for argv, 36 787 for tree, 8 710 for dirty |
| truth `E` | w-emit's `truth/`, 174 417 names — reproduced here on **850/850** (AGREE) |
| truth `D` | w-joint's `truth_data.py`/`objsyms.py` unchanged: `\|D_all\|` **685 848**, `\|D_data\|` **232 156**, arity **1 534 428 = 1 147 426 + 387 002** — every figure w-joint's to the digit |
| flags | the unmodified workload line, `work/dc3-workload/flags.txt` — **and it contains no `/Zi`**, which is §1's whole story |
| scratch | `work/w-db/` (gitignored); scripts and text outputs force-added, no IL or obj committed |

**The 21-TU quarantine is intact and w-emitpred's one-shot Part-1 gate is
UNSPENT** — §9. Every mutation TU was checked against `heldout.txt` **by the
script, before anything was written** (`check_quarantine`).

---

## 1. THE NAMED INSTRUMENT — `db`, opened, and it is DEBUG

w-joint, w-skip and w-mark all named `db` and all three declined it. It is
opened here, by address first and by the sole judge second.

### 1a. Where it is, from the binary

`work/w-db/dis.sh` reproduces every line. The five sub-stream names are
**contiguous** at `0x10b13358`, each with a UTF-16 twin:

    0x10b13358 "pch"   0x10b1335c "db"   0x10b13368 "sy"
    0x10b13374 "ex"    0x10b13380 "in"   0x10b1338c "gl"

A full `<imm32>` scan of `.text` finds **exactly two** references to `"db"`:

| site | what |
|---|---|
| `0x10b73bd3` | the **container writer**, gated at `0x10b73bb7` on `ds:0x10c40ef8 & 0x2000` **or** `ds:0x10c40ecc != 0` |
| **`0x10be7f41`** | the **reader**: `mov edx,0x10b1335c ; mov ecx,eax ; call 0x10b7e276`, inside the per-module loop at `0x10be7ef5` |

and the reader's gate is the line above it:

    10be7efe   test DWORD PTR [eax+0xcd8],0x2000     <<< the DEBUG-INFO bit
    10be7f08   je   0x10be7fd9                       <<< skip the module entirely
    10be7f1b   push 0x4                              <<< SUB-STREAM ORDINAL 4
    10be7f23   lea  ecx,[eax+0x280]                  <<< its slot
    ...
    10be7fc4   call 0x10be997b  ;  10be7fcf  call 0x10be9892

against `sy` = 1 (`[eax+0x25c]`), `ex` = 2 (`[eax+0x268]`), `in` = 3
(`[eax+0x274]`), all read by the p2 driver `0x10b7f022`. **`db` is read by no
part of the p2 driver at all** — it is read by `0x10be7ef5`, which is a debug
emitter, and only when the module carries `0x2000`.

### 1b. What it holds

924 MB over 850 TUs, median **651 564** bytes. It is **not** a plain CodeView
`<len:u16><leaf:u16>` stream — the registered T5 walk consumed **0 of 850**
files exactly and recognised **0** known leaves, which is a **MISS** and is
reported as one: `db` uses the container's own `varU`/`i32c` codec, not the
CV record framing. What it *carries* is unambiguous from its 5 533 922 strings:

    stlpmtx_std::map<Symbol,AccomplishmentCategory *,...>::value_compare
    mMotionParentDelta   mBaseFileName   ContentMgr::Callback
    kEaseCircOutIn   NUI_SPEECH_LANGUAGE_IT_IT   GetFlawlessMoveCount

— **source-level type, member and enumerator names**, undecorated. And the
measurement that settles the lane's question:

| | |
|---|---:|
| `D_all` names occurring as a string in `db` | **4 / 685 848 = 0.00001** |
| `D_data` names | **4 / 232 156 = 0.00002** |
| **`E` names** | **0 / 174 417 = 0.00000** |

> ### **`db` names not one of the 174 417 functions c2 emits and 4 of the 685 848 symbols it defines. It is the debug TYPE stream and it cannot determine `D`.**

### 1c. MUT-DB — the null, graded the way w-skip graded its null

An inert result is what a mislocated write looks like, so the arms are built to
separate the two (`work/w-db/mutate_db.py`, 3 TUs, none quarantined):

| arm | | `HttpReq` | `EventTrigger` | `StreamNull` |
|---|---|---|---|---|
| **P0** | rewrite `db` byte-for-byte | **obj byte-identical** | identical | identical |
| **P1** | `db` truncated to its 2-byte header | **byte-identical** | identical | identical |
| **P2** | `db` replaced with **another TU's** `db` | **byte-identical** | identical | identical |
| **P3** | **`db` REMOVED from the bundle** | **byte-identical** | identical | identical |
| **P4** | **positive control** — the same substitution on **`in`** | **c2 refuses / SIGSEGV** | refuses | refuses |

> ### **Deleting the whole `db` stream changes ZERO bytes of the obj, 3/3.** The same edit applied to `in` makes c2 **refuse**, so the substitution reaches c2. **M19 is a red, and the red is the answer**: on this workload c2 does not read `db` at all.

Two things this run got wrong first and had to fix, both recorded rather than
quietly repaired:

* the first run compared objs written to **per-arm output paths**, and `.debug$S`
  carries `S_OBJNAME` (w-joint **U-j**), so **P0 went red for a reason that had
  nothing to do with `db`**. One fixed output path for every arm.
* the second run compared raw bytes and the **COFF `TimeDateStamp`** differs
  between replays a second apart. The project's own correctness rule — zero
  offset 4..8 — is now applied before every compare. **Neither fix touches a
  scored definition** and both are disclosed under clause 5.

### 1d. And it agrees with the flags

`work/dc3-workload/flags.txt` is
`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc /I …` — **there is no `/Zi`**. So
`[module+0xcd8] & 0x2000` is clear, `0x10be7f08` skips the module, and `db` is
never opened. The disassembly and the sole judge say the same thing for the same
reason. **`.debug$S` is in all 850 objs anyway** — it carries `S_OBJNAME` /
`S_COMPILE`, which do not come from `db`.

---

## 2. THE MODEL — `JFP`, and the two variants that isolate every part of it

`work/w-db/scan.py`. Nodes are `.gl` names; the *only* change against w-joint is
that the reference-list edges are taken **unrestricted**, so a function may
reach a data symbol, and the data half is **derived** rather than supplied.

    ROOTS  Seed = { f in U : flags4c & 0x20, not & 0x02 }        w-roots
    EDGES  f -> every name its reference list names, refcount!=0  w-refs, NO ∩U
           d -> every name an `02` node of d's `in` record names  w-mark, NO ∩U
    GATE   a node outside U enters only if it owns an `in` record
    OUT    P = live ∩ U   vs E        Dpred = live ∩ W   vs D

### 2.1 KA-A — every incumbent reproduces to the digit

| | recorded | **this pass** |
|---|---|---|
| `\|U\|` / `\|E\|` / `\|E ∩ U\|` / `\|Seed\|` | 1 506 586 / 174 417 / 173 907 / 14 662 | **EXACT, all four** |
| `RGL` | 129 604 / 1.00000 / 0.74307 / 0.85260 / 132 | **EXACT, all five** |
| `INIT` | 613 532 / 0.27289 / 0.95991 / 0.42496 / 34 | **EXACT** |
| `SKIP` | 400 998 / 0.36420 / 0.83732 / 0.50761 / 34 | **EXACT** |
| `ORACLE` *(a ceiling)* | 167 213 / 0.99997 / 0.95867 / 0.97888 / 151 | **EXACT** |

### 2.2 The CODE half, against `E`

| variant | `\|P\|` | precision | recall | **F1** | **exact** |
|---|---:|---:|---:|---:|---:|
| `ORACLE` *(a **CEILING**, never a model)* | 167 213 | 0.99997 | 0.95867 | **0.97888** | **151** |
| **`JFP`** | 150 833 | **0.99899** | **0.86391** | **0.92655** | **132** |
| `JFP_UNGATED` | 150 833 | 0.99899 | 0.86391 | 0.92655 | 132 |
| `JFP_C1` | 150 833 | 0.99899 | 0.86391 | 0.92655 | 132 |
| `JFP_KEEPZERO` | 155 134 | 0.97354 | 0.86591 | 0.91657 | 129 |
| `JFP_URESTRICT` | 129 604 | 1.00000 | 0.74307 | **0.85260** | 132 |
| `JFP_CODEONLY` | 129 604 | 1.00000 | 0.74307 | **0.85260** | 132 |
| `RGL` — the incumbent | 129 604 | 1.00000 | 0.74307 | 0.85260 | 132 |

**`JFP` makes 152 false positives in 150 833 predictions** and closes 47.4 % of
the gap between the incumbent and the oracle ceiling.

**The two isolating variants are the interesting rows.** `JFP_URESTRICT` (code
edges restricted to `U`, i.e. w-refs') and `JFP_CODEONLY` (no `in` channel) are
each **exactly `RGL`, to the digit**. So:

> ### **Neither channel does anything on its own.** Restricted code edges + `in` edges = the incumbent. Unrestricted code edges without `in` edges = the incumbent. Only both together move anything, because a data symbol reached from code is what makes its initializer contribute function roots — which is **w-skip's owner-emitted filter, arrived at from the other side**.

### 2.3 The DATA half, graded DIRECTLY against `D` — the axis no lane has measured

Population: the **702 263** `in`-owner names. Positives: `D ∩ W` = **40 947**,
a base rate of **0.05831**.

| variant | `\|Dpred\|` | precision | recall | **F1** | **exact** |
|---|---:|---:|---:|---:|---:|
| **`JFP_C1`** | 40 046 | **0.98167** | **0.96007** | **0.97075** | **42 / 850** |
| `JFP` | 39 196 | 0.98127 | 0.93931 | 0.95983 | 0 / 850 |
| `JFP_UNGATED` | 39 196 | 0.98127 | 0.93931 | 0.95983 | 0 |
| `JFP_KEEPZERO` | 41 098 | 0.93586 | 0.93931 | 0.93758 | 0 |
| `JFP_CODEONLY` | 9 567 | 0.98693 | 0.23059 | 0.37384 | 0 |
| `JFP_URESTRICT` | **0** | — | 0.00000 | 0.00000 | 0 |

**`JFP_C1` differs from `JFP` by one root: `__C1_<build>`**, the leading `in`
record w-mark named. It is defined in every obj, `JFP` misses it in every obj,
and that single name is why the data-exact count is **0 in every TU** until it is
added and **42** after. A per-TU exact metric with a universal miss in it
measures nothing, which is the honest reading of that 0.

**And w-joint's twelve static `Rd` rules, graded against `D` in the same pass.**
This is the right comparison and not a strawman: w-joint's `Rd` *is* its guess at
`D ∩ owners` — its `ORACLE` variant is literally `rd_oracle(own, D)` — and
w-joint's own U-a shows the dd-closure of an `Rd` adds nothing (`|live| == |Rd|`
on 850/850 at three sizes), so grading `Rd ∩ W` is grading the rule and not a
truncation of it.

| rule | `\|Rd ∩ W\|` | precision | recall | F1 |
|---|---:|---:|---:|---:|
| **`ALL`** — the best | 702 262 | 0.05831 | 1.00000 | **0.11019** |
| `F20_60_20` | 702 262 | 0.05831 | 1.00000 | 0.11019 |
| `F20_4000` | 684 343 | 0.05773 | 0.96483 | 0.10894 |
| `TAG_01` | 587 199 | 0.05742 | 0.82341 | 0.10735 |
| `F20_1000` | 553 300 | 0.05731 | 0.77446 | 0.10673 |
| `F20_400` / `F20_480` | 432 137 | 0.05554 | 0.58610 | 0.10146 |
| `TAG_02` | 115 063 | 0.06284 | 0.17659 | 0.09270 |
| `F20_2000` | 84 311 | 0.04175 | 0.08596 | 0.05620 |
| `NONE` / `F20_80` / `SC_STATIC` | 0 | — | 0.00000 | 0.00000 |

> ### **The best static `.gl` rule against `D` is `ALL` at 0.11019 — i.e. "predict everything", which is the base rate. Not one of the twelve carries any information about `D` at all.** `JFP_C1` reaches **0.97075**. w-joint's U-c said no static rule predicts the filter; measured against `D` itself the statement is far stronger than its downstream form suggested.

---

## 3. THE CORRECTION I OWED AND WITHDREW — MUT-CD, the SOLE JUDGE

The prereg's §0 argued from `0x10b28a9b` (the COFF writer's own recursion,
guarded by `[sym+0x32] & 1`, re-entered from `0x10b28cb9`/`0x10b29057`) that data
symbols are emitted by a *second* closure over the **unpruned** reference
relation, and that w-skip T-e's prune therefore says nothing about them. **The
mutation says otherwise.**

### 3.1 The instrument control, which is this lane's own failed first design

`mutate_cd.py` retargets one `varU` token of a function's `.gl` reference list,
byte-length preserving by construction, and replays through real `c2.dll`. The
first design replaced the referrer's **first** list token, whatever it pointed
at:

| | `HttpReq.cpp`, 10 replays |
|---|---|
| **H+** referrer emitted, payload an undefined DATA symbol | **0/5 APPEARS** — but `gained=56..118`, `lost=57..120` |
| **H−** referrer not emitted | 0/5, `gained=0 lost=0` |

**0/5 with 56 to 120 symbols of collateral churn is not a null, it is a
confound** — the retarget destroyed the referrer's own closure. It is kept here
because it is exactly the control the clean arm needs: **a write at this byte
class reaches c2 and moves the emit set by up to 120 symbols.**

### 3.2 The clean arm, and the refutation

`--datatok`: the token replaced must **already** name a **defined DATA** symbol,
so the pruned Mark channel sees no change and the data target is the only
variable. Payload preference is `??_7`/`??_R`, the class the model gets right.

| TU | referrer (emitted) | old target | new target | gained | lost | APPEARS |
|---|---|---|---|---:|---:|---|
| `HttpReq` | `??0HttpReq@@QAA@W4ReqType@@IGPBD@Z` | `??_7HttpReq@@6B@` | `??_R4underflow_error@…` | **0** | **0** | **False** |
| `HttpReq` | `??1HttpReq@@UAA@XZ` | `??_7HttpReq@@6B@` | `??_R4range_error@…` | 0 | 0 | False |
| `PoolAlloc` | `??0FixedSizeAlloc@@QAA@HH@Z` | `??_7FixedSizeAlloc@@6B@` | `??_R4underflow_error@…` | 0 | 0 | False |
| `PoolAlloc` | `?PoolAlloc@@YAPAXHHPBDH0@Z` | `?gChunkAlloc@@3PAV…` | `??_R4overflow_error@…` | 0 | 0 | False |
| | | | **10 of 10** | **0** | **0** | **0/10** |

> ### **Every one of the ten replays produced an obj that is byte-identical to the baseline. The old data symbol is NOT lost and the new one does NOT appear. The `.gl` reference list's DATA entries are causally INERT.**

**M21 = 0/10 and the registered pass was ≥ 4/5. Clause 4 fires and §0 is
withdrawn.** **M22 is UNDECIDABLE**: on both TUs the H− population is **0** —
every referrer carrying a data token in its list is itself emitted — so the
control could not be run, and that is reported as an undecidable rather than as
a pass.

### 3.3 What that leaves standing, stated precisely

* **The mechanism claim is dead.** `0x10b27f3c` keeps an edge only for a
  tag-`0x0E` target, and nothing downstream reads the dropped ones for emission.
  **w-skip T-d/T-e and w-joint's `joint.py` are right about the mechanism**, and
  this lane's prereg §0 is wrong. Recorded as a refutation, not softened.
* **The predictive claim survives, as a correlation.** The `.gl` reference list
  is a faithful *record* of the references a function body makes; c2 gets the
  same information from the body's own relocations in `.ex`, which this lane did
  not touch. So `JFP` is an excellent **proxy** and not a model of a channel.
  **F1 0.92655 / 0.97075 are predictions, not mechanisms**, and every table on
  this page says so.
* **The next lane's experiment is named and not run**: the same retarget on a
  **`.ex` body relocation** rather than on the `.gl` list. That is the channel
  the correlation is a proxy for, and it is §8 item 1.

---

## 4. Calibration, stratification, and the counts behind the edge

### 4.1 The coincidence calibration decline clause 2 demands

Uniform expectation over the part of `U` the incumbent does not predict:
`(174 417 − 129 604) / (1 506 586 − 129 604)` = **0.03254**. Base rate
`|E|/|U|` = **0.11577**.

| | new marks over `P_RGL` | of which emitted | measured | **ratio** | vs base |
|---|---:|---:|---:|---:|---:|
| **`JFP`** | 21 229 | **21 077** | **0.99284** | **30.51×** | **8.58×** |
| w-joint `ORACLE` (a ceiling) | 31 457 | 31 456 | 0.99997 | 30.73× | 8.64× |
| w-mark `ALL` | 242 056 | 31 456 | 0.12995 | 3.99× | 1.22× |
| w-skip's filtered roots | — | — | 0.08739 | 2.69× | **0.82× — below chance** |
| w-emit's disqualified loose scan | — | — | 0.0277 | 1.07× | — |

> ### **21 077 of the 21 229 names `JFP` adds are emitted — 30.51× the uniform expectation, indistinguishable from the ORACLE's 30.73× and 7.6× further from chance than w-mark's channel.**

### 4.2 Stratified, so `#152` cannot dominate either direction

`??_G`/`??_E` deleting destructors are **5 344 = 3.064 %** of `E`.

| model | `\|P\|` | precision | recall | **F1** |
|---|---:|---:|---:|---:|
| `RGL` | 128 781 | 1.00000 | 0.76169 | 0.86473 |
| **`JFP`** | 149 439 | 0.99898 | 0.88297 | **0.93740** |

**Every conclusion is unchanged with `#152` excluded** — ordering, sign and
magnitude all survive.

**And `#152` does NOT dominate this residual, which is a registered MISS.**

| `E ∩ U` that `JFP` misses (23 226 names) | n | % |
|---|---:|---:|
| **free / file-scope function** | **14 766** | **63.58 %** |
| `??_G`/`??_E` deleting dtor (`#152`) | 3 950 | 17.01 % |
| `$` in the qualified name | 1 855 | 7.99 % |
| non-virtual member | 856 | 3.69 % |
| undecorated (`extern "C"` / CRT) | 851 | 3.66 % |
| VIRTUAL member | 552 | 2.38 % |
| static member | 372 | 1.60 % |

I registered `#152` at 0.55 with a floor of 0.20 and it is **0.1701** — a miss
below the interval. Under w-joint's *oracle* `#152` is 58.95 % of what is left;
under this *model* the free-function class comes back and dominates, because the
model's recall (0.86391) is well below the oracle's (0.95867) and the names it
misses are the address-taken free functions w-mark's channel closed at 99.60 %.
**The two residuals are not comparable, and reading 17.01 % as progress against
`#152` would be wrong.**

The **DATA** residual (2 485 names) is a different population again: 34.29 %
undecorated `extern "C"`/CRT, 32.47 % `$`-bearing, 14.69 % vftable/RTTI.

**And the false positives on the two axes are the SAME classes**
(`work/w-db/fpchar.py`, a reporting addition written after the scan and
disclosed under clause 5 — `scan.py` is untouched):

| | n | composition |
|---|---:|---|
| **CODE** false positives | **152** | 125 `$`-bearing (template/adjustor), **24 VIRTUAL members**, 3 `??_G` |
| **DATA** false positives | **734** | **734 vftable / RTTI — 100 %** |

and they line up class by class: `??_7DancerSkeleton@@6B@`,
`??_R0?AVDancerSkeleton@@@8`, `??_R1A@?0A@EA@DancerSkeleton@@8` on the data side
against `?ElapsedMs@DancerSkeleton@@UBAHXZ`, `?IsTracked@DancerSkeleton@@UBA_NXZ`
on the code side. **The model's whole error budget is a small number of classes
per TU whose vtable+RTTI it pulls in when c2 does not, and then their virtual
members follow.** 734 wrong data names out of 40 046 and 152 wrong code names out
of 150 833 is what that costs.

### 4.3 The edge, counted

| | |
|---|---:|
| reference-list targets with refcount != 0 | 2 573 569 |
| **…of which NOT in `U`** (the unrestricted half) | **950 824 = 0.36946** |
| …of those, an `in` owner | 192 919 |
| `in` nodes / unbound node tokens | 1 885 284 / **454 = 0.00024** |
| `in` owners / owners this decoder cannot name | 702 262 / 177 114 |
| **owners ∩ `E`** — the circularity check | **0** |
| owners ∩ `D` | 40 947 = 0.05831 |

**More than a third of every function's reference-list targets are outside
`U`**, and w-refs threw all of it away. `outside U` is **not** the same as
`data` — it also contains tag-`0x0E` records that `refs.scan`'s extern-class
gate skipped — so the tighter number is the **192 919** that are `in` owners,
and that is the population the model's data half ranges over. That is the fact
the model exploits and — per §3 — **not** the fact c2 acts on.

### 4.4 The one parameter I fitted against data turns out to be INERT

The prereg disclosed that the owner gate (`∈ W`) was chosen after seeing three
TUs' ungated false positives. Corpus-wide, **`JFP_UNGATED` is `JFP` to the
digit, on both axes.** A `.gl` data name with no `in` record has no outgoing
edge, so it is a leaf and cannot relay anything; gating its *entry* changes
nothing once the data axis is scored on the owner population. **The gate looked
load-bearing on 3 TUs and is measurably a no-op on 850.** It is left in the
frozen definition rather than removed, and reported as inert.

---

## 5. Invariants — the instrument is graded on its own, not on the oracle

| invariant | registered | **measured** | |
|---|---|---|---|
| **AGREE** — my code-COMDAT leaders == w-emit's independent truth | 850/850, ≥845 | **850 / 850**, 0 disagree | **GREEN** |
| **TOT** — every entity in one bucket, residue named | 0, ≤500 | **0** entities, **0** unclaimed COMDAT sections | **GREEN** |
| **AR A1/A2/A3** | 0 TUs each | **0 / 0 / 0**; 1 534 428 records = 1 147 426 entities + 387 002 aux; 22 035 337 long-name bytes | **GREEN** |
| **INJ** | 338, [0,600] | **338 conflicting definitions over 116 TUs**, all `$LN<n>` local labels, **0 of them an `in` owner** | **RED, characterised** |
| **KA-POS** — the run GRADED something | > 0 | **21 229** discriminating names (`P_JFP △ P_RGL`) | **GREEN** |
| `in` terminus gate | 850/850 | **850 / 850** | **GREEN** |
| MUT baseline reproduces the pipeline obj's leader set | 3/3 | **5 / 5** bundles | **GREEN** |

**Could AGREE have gone red?** Yes, in the most likely failure mode: the capture
obj (`-il -typedil`, a different `-Fo`) need not be the obj a plain `cl` makes,
and AGREE compares against 850 objs from a separate plain `cl` run in another
lane at another wibo. It did not.

---

## 6. What the port would have to do with this — and why it is not yet a rung

**Nothing here converts a TU.** `JFP`'s exact set is `RGL`'s 132, and TU match is
**8** at both ends. The bound that matters is unchanged: `A∧B∧C` = 25, 8 matched,
**FRONTIER 17**.

The reason is worth stating, because a 7.4 pp F1 gain that converts nothing is
the exact shape of `STATUS.md` trap 3. **`JFP` is right about 21 077 more names
per corpus and still wrong about at least one name on every TU that was already
wrong** — recall 0.86391 means ~13.6 % of each TU's emitted names are missing,
and a TU needs *all* of them. Per-TU exact is a conjunction over ~205 names on
average; it does not move until recall is very close to 1.

---

## 7. Scoring the pre-registration — 24 hits, 4 misses, 2 passes, 1 fail, 1 undecidable

| # | registered **point** | interval | **measured** | |
|---|---|---|---|---|
| **T1** | `db` present **850/850** | [845,850] | **850 / 850** | **HIT**, at the point |
| **T2** | `db` median **300 000** B | [10k, 5M] | **651 564** (924 MB total) | **HIT**, above |
| **T3** | `D_all` in `db` **0.02** | [0.00,0.60] | **0.00001** (4 / 685 848) | **HIT**, below |
| **T4** | `E` in `db` **0.02** | [0.00,0.60] | **0.00000** (0 / 174 417) | **HIT**, at the floor |
| **T5** | CV leaves recognised on **850/850** | ≥845 | **0 / 850** | **MISS at the floor** — §1b |
| **T6** | AGREE **850/850** | [845,850] | **850 / 850** | **HIT**, at the point |
| **T7** | TOT residue **0** | [0,500] | **0** | **HIT**, at the point |
| **T8** | arity **0/0/0** | [0,5] each | **0 / 0 / 0** | **HIT** |
| **T9** | INJ **338** | [0,600] | **338**, 116 TUs, all `$LN` | **HIT**, at the point |
| **M1** | `JFP` precision **0.995** | [0.900,1.000] | **0.99899** | **HIT**, above |
| **M2** | recall **0.930** | [0.800,0.980] | **0.86391** | **HIT**, below |
| **M3** | **F1 0.960** | [0.860,0.990] | **0.92655** | **HIT**, below the point — **+7.395 pp over the incumbent, past the 0.87260 bar, so clause 1 does NOT fire** |
| **M4** | per-TU exact **0.171** | [0.100,0.400] | **0.15529** (132/850) | **HIT**, below — and **+0 against the incumbent** |
| **M5** | `JFP_URESTRICT` F1 **0.870** | [0.800,0.960] | **0.85260** | **HIT**, below |
| **M6** | `JFP_CODEONLY` F1 **0.900** | [0.700,0.980] | **0.85260** | **HIT**, below |
| **M7** | data precision **0.990** | [0.850,1.000] | **0.98167** | **HIT** |
| **M8** | **data recall 0.920** | [0.700,0.990] | **0.96007** | **HIT, above** — *the outcome I declared I most expected to be wrong about, and I was right about it* |
| **M9** | **data F1 0.950** | [0.780,0.995] | **0.97075** | **HIT**, above |
| **M10** | data per-TU exact **0.250** | [0.050,0.600] | **0.04941** (`JFP_C1`), **0.0** (`JFP`) | **MISS below the interval** — §2.3 |
| **M11** | best static `Rd` vs `D` **0.45** | [0.10,0.85] | **0.11019** (`ALL`) | **HIT**, far below |
| **M12** | base rate **0.058** | [0.03,0.12] | **0.05831** | **HIT**, at the point |
| **M13** | coincidence **25×** | [3×,31×] | **30.51×** | **HIT**, above |
| **M14** | vs base rate **7×** | [1×,9×] | **8.58×** | **HIT** |
| **M15** | stratified F1 **0.975** | [0.880,0.998] | **0.93740** | **HIT**, below |
| **M16** | `#152` share **0.55** | [0.20,0.90] | **0.17010** | **MISS below the interval** — §4.2 |
| **M17** | KA-A, every incumbent exact | exact | **all five exact** | **HIT** |
| **M18** | KA-POS **> 20 000** | > 0 | **21 229** | **HIT** |
| **M19** | `db` mutation changes the obj **3/3** | ≥2/3 | **0 / 3** | **MISS at the floor — and the miss IS the finding**, §1c |
| **M20** | non-debug identical + defined set unchanged **3/3** | ≥2/3 | **3 / 3** (the *whole* obj is identical) | **PASS** |
| **M21** | **H+ 5/5** — code→data | ≥4/5 | **0 / 10** | **FAIL — clause 4 fires**, §3 |
| **M22** | H− ≤ 1/5 | — | **0 candidates on both TUs** | **UNDECIDABLE**, §3.2 |
| **M23** | baseline reproduces the pipeline leaders **3/3** | 3/3 | **5 / 5** | **PASS** |

**The declared bias was M8, data recall — the number I said a corpus would
punish. It did not: 0.96007 against a registered 0.920.** I was wrong in the
direction that costs me: **the thing I did not flag, M21, is the one that failed,
and it is the lane's causal claim rather than any of its correlations.** That is
the honest summary — I calibrated the observational numbers well and the
mechanistic one not at all, which is the same shape w-skip recorded.

Second declared: **M4, per-TU exact.** I registered it at 0.171 against an
incumbent 0.15529 and it came in **exactly at the incumbent**. I was right that
micro-F1 could move without per-TU exact moving, and I still registered a point
above the incumbent.

**Four misses. Three of them (T5, M16, M19) are the model looking *worse* or the
instrument being *less* than I said; M10 is a universal single-name miss that
`JFP_C1` mostly repairs. None is in the direction of the model working better
than registered.**

### 7.1 The decline clauses — one fired, all honoured

* **Clause 4 (M21/M22 fail to discriminate) TRIGGERED and honoured literally.**
  §0's correction is withdrawn in the **first paragraph**; the model is labelled
  a **correlation with no mechanism** in the headline, in §2 and in §3.3.
* **Clause 1 (F1 < 0.87260) NOT fired** — 0.92655. **And I did not go looking
  for a further channel after the numbers arrived**: `sy`, `0x10b3389b`,
  `0x10b9aa26`, node kind `0x14`, the `.ex` body relocation and the 177 114
  unnameable owner tokens are **named in §8 and left undecoded**.
* **Clause 2 (data precision < 0.85) NOT triggered** — 0.98167. The calibration
  is published anyway, in w-mark's exact shape (§4.1).
* **Clause 3 (`db` reaches the non-debug obj) NOT triggered** — 3/3 identical.
* **Clause 5 (no instrument tuning after truth) HONOURED for every scored
  number, with four disclosures.** (i) the `db` compare gained a fixed output
  path after `S_OBJNAME` made P0 red; (ii) it then gained `TimeDateStamp`
  zeroing after two replays a second apart differed; (iii) the **P4 positive
  control** on `in` was added *after* P1/P2 came back inert — it widens the
  evidence *against* a claim of instrument failure, which is the direction that
  costs me; (iv) the `--datatok` arm and the `??_7`/`??_R` payload preference
  were added *after* the first MUT-CD came back confounded, and the clean arm is
  **more** hostile to this lane's hypothesis, not less. **`scan.py`, `score.py`,
  the six variants, the edge definitions, the gate and the truth reader are
  byte-identical to the prereg commit**, and KA-A proves the incumbents to the
  digit.
* **Clause 6 (nothing ships) HONOURED.** `git diff 669ee6c -- crates/ scripts/`
  is empty; `PortC2` still returns `NotImplemented` outside its class.
* **Clause 7 (`Rfloor` is not a key) HONOURED** — it is not computed here at
  all, and no number on this page keys on it.
* **Clause 8 (`ORACLE` is never a model) HONOURED** — labelled a ceiling in the
  headline, §2.1, §2.2 and §4.1.
* **Clause 9 (`db`'s null graded like w-skip's) HONOURED** — P0 reproduces the
  obj byte for byte and P4 shows the same substitution makes c2 refuse on a
  stream it does read.

### 7.2 Registered before the numbers existed, restated against them

* **TU match stays 8.** It did — 8 at both ends (§10).
* **`census/gate disagreement` stays 0.** It did.
* **A high F1 is not a shippable predicate**, and this page is the cleanest
  demonstration the project has: **+7.395 pp of F1 and +0 TUs.**
* **Order is untouched.** A right set in the wrong order is still a mismatch.

---

## 8. What this lane did NOT measure — named, so absence never reads as success

1. **The `.ex` body relocation channel.** §3 shows the `.gl` reference list is a
   *proxy*; the thing c2 actually reads for data emission is untouched, and the
   retarget-a-body-relocation experiment is **named and not run**. **This is the
   single highest-value next measurement.**
2. **`0x10b28a9b`'s dispatch by storage class.** Kind-1 symbols with
   `(+0x37 >> 0x15) & 7 ∈ {1,3}` are **never written** (`0x10b28bb1`/`0x10b28bbd`)
   and the four storage-class arms (`0x10b28d1d`, `0x10b28c7d`, `0x10b28c6e`,
   `0x10b28c00`) are decoded nowhere. Read from the binary, modelled **nowhere**.
3. **`sy`.** Still unread by any lane.
4. **`0x10b3389b`** (`dag.c`, edges during codegen) and **`0x10b9aa26`** (the
   by-name intern). Named by w-skip and w-joint; still unmodelled.
5. **Node kind `0x14`.** Only the `0x02` byte kind is decoded.
6. **The 177 114 unnameable `in` owner tokens** (0.20141) and the **454
   unnameable `02` targets** (0.00024). Not characterised.
7. **`#152`.** 17.01 % of this model's residual, 58.95 % of the oracle's, and
   unreachable by any initializer or reference model.
8. **`db`'s internal grammar.** It is shown to be the debug stream, to name no
   emitted symbol, and to be unread by c2 on this workload. Its **record
   framing is NOT decoded** — T5 is a red, and `db` is not a plain CodeView
   record stream.
9. **Whether `db` matters under `/Zi`.** Every statement here is about the
   workload's flags. A `/Zi` corpus would set `[module+0xcd8] & 0x2000` and
   `0x10be7f41` would run. **Untested.**
10. **Order.** A right set in the wrong order is still a mismatch.
11. **The 21 quarantined TUs.** Untouched (§9).

---

## 9. The one-shot Part-1 gate — NOT spent, and the question is put to the coordinator

The 21-TU quarantine is intact and w-emitpred's Part-1 gate is **still runnable
exactly once**, seven lanes running.

w-joint said the gate is owed by whoever first ships a model that predicts `D`.
**This lane has that model and it does have fitted parameters**, so the gate is
genuinely earned here. Per prereg §8 I **did not spend it unilaterally**. What
the coordinator needs in order to decide:

* **The model**: `JFP` / `JFP_C1` — least fixpoint over unrestricted `.gl`
  reference-list edges plus `in` initializer edges, roots `Seed`, data entry
  gated on owning an `in` record.
* **In sample, 850 TUs**: code **0.99899 / 0.86391 / 0.92655 / 132-of-850**;
  data, against `D` directly, **0.98167 / 0.96007 / 0.97075 / 42-of-850**.
* **The fitted parameters and what varies across them**: (1) code-edge target
  restriction — varied by `JFP_URESTRICT`, which is exactly `RGL`; (2) the data
  entry gate — varied by `JFP_UNGATED`, **measured inert**; (3) refcount-0 edges
  — varied by `JFP_KEEPZERO`, −0.998 pp of code F1; (4) the root set — varied by
  `JFP_C1`, +1.09 pp of data F1 and +42 data-exact TUs. **That is four binary
  choices, three of them inherited from landed lanes, and every one is isolated
  by a scored variant.** There is very little surface to overfit.
* **It is NOT refuted in sample** — 0.92655 against a 0.87260 bar — so clause 1
  did not fire and w-skip's "a held-out set cannot improve a refutation" does
  **not** apply here.
* **The argument for spending it**: §3 removed the mechanism, so `JFP` is now a
  *purely empirical* predictor, and an empirical predictor is exactly the object
  a held-out population exists to test.
* **The argument against**: the model converts **zero TUs**, so nothing depends
  on it yet, and the gate is worth more spent on a model that is about to ship.

**I recommend not spending it yet, and I am not spending it.** The registered
reversal condition did not trigger and I checked it honestly: no definition in
`scan.py` was chosen by looking at `E` or `D` after the prereg commit; the one
parameter chosen against data (the owner gate) was chosen on **three disclosed
TUs before the prereg** and is measured **inert** on 850.

---

## 10. Gate — every incumbent reproduced, on a tree with no `crates/` change

`git diff 669ee6c -- crates/ scripts/` is **EMPTY**, so the incumbent column and
this column are one tree measured once. Every number was **re-measured here**,
none transcribed. Logs: `work/w-db/gate.log`, `gate2.log`, `selftest.log`,
`gap.log`.

| | master `669ee6c` — re-measured | **this tree (`d07d86b`)** |
|---|---|---|
| `cargo test --workspace --release` | — | **706 passed, 0 FAILED, 1 ignored, 25 targets** |
| `cargo build --release` | — | **0 warnings** |
| `c2rs selftest` | — | **225 PASS, 0 FAIL** |
| `scripts/gate.sh --jobs 6` | — | **12/12 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT, 2 700 fixture-verdicts, 0 mismatch** |
| **`scripts/expr_sweep.sh`** | — | **47 fragments, checked = 14 484, mismatches = 0** |
| TU match / mismatch / codegen-gap / vocab-gap / capture-fail | — | **8 / 0 / 0 / 863 / 7** |
| A / B / C / D / E | — | **28 / 338 / 114 / 8 / 2** |
| `A∧B∧C` / `A∧B∧C∧(D∨E)` / `B∧C` | — | **25 / 8 / 107** |
| **FRONTIER** | — | **17** |
| **`census/gate disagreement`** | — | **0** |
| capture cache | — | **871 hit, 7 miss, 0 POISONED** |

*Compared on the **FAILED** count and the **target** count, never the passed
count — a failing target aborts the run, so a lower passed count reads as green.*

**Two counts in the predecessor rung docs are stale and this tree's are the
current ones:** `c2rs selftest` is **225**, not w-skip's/w-joint's **222**; the
workspace test count is **706**, not w-joint's **698**. Master grew fixtures
under both. `FAILED` is 0 and `targets` is 25 in every reading.

**`scripts/gate.sh` does not run `scripts/expr_sweep.sh`** — the merge gate is
blind to that whole class, so the sweep is run separately above and its
**14 484 / 0** is part of this lane's green, not an extra.

**dc3-decomp HEAD after the run: `940d07dcb0960964ad61aa5f025658f993eb46b2` — it
did not move** (`work/w-db/prov_{before,after}.txt`). wibo `1.0.1-23-g4a9dd6f`
throughout.

---

## 11. Proposed board rows — **numbers NOT minted**

Same discipline as w-roots, w-emit, w-refs, w-mark, w-skip and w-joint: **no
number minted, no `#N` pinned in code, `BOARD.md` / `ROADMAP.md` /
`rungs/INDEX.md` untouched by hand** (w-book2 owns the board). `T-`, `U-` and
`X-` are taken; this lane uses **`V-`**.

| proposed | item | claim | where |
|---|---|---|---|
| **V-a** | **`db` is the DEBUG sub-stream and c2 NEVER READS IT on this workload.** Ordinal 4, read only at `0x10be7f41` under `[module+0xcd8] & 0x2000`, and the workload has no `/Zi`. **Deleting the whole stream leaves the obj byte-identical, 3/3**, while the same substitution on `in` makes c2 refuse. **0 of 174 417** emitted functions and **4 of 685 848** defined symbols occur in it as a string | three lanes named `db` as the next instrument; it is not one. The null is graded like w-skip's: P0 reproduces the obj byte for byte, P4 shows the write path works | this file §1 |
| **V-b** | **THE MODEL THAT PREDICTS `D`: a least fixpoint over UNRESTRICTED `.gl` reference edges plus `in` initializer edges reaches, against `D` DIRECTLY over the 702 263-owner population, precision 0.98167 / recall 0.96007 / F1 0.97075** — against the best of w-joint's twelve static rules at **0.11019**, which is the base rate | the data half graded directly against `D` for the first time; w-joint's `Rd` rules re-graded on the same axis in the same pass | §2.3 |
| **V-c** | **On the code axis the same fixpoint reaches F1 0.92655 (0.99899 / 0.86391), +7.395 pp over w-refs' 0.85260, closing 47.4 % of the gap to w-joint's 0.97888 ceiling — AND CONVERTS ZERO TUs**: its per-TU exact set is w-refs' **132, name for name, 0 gained and 0 lost** | the cleanest instance the project has of `STATUS.md` trap 3 — a large F1 move on the payoff metric's leading indicator that moves the payoff metric by nothing | §2.2, §6 |
| **V-d** | **REFUTED BY ITS OWN MUTATION: the `.gl` reference list's DATA entries are causally INERT.** Retargeting a data token in an emitted function's list changes the obj by **zero bytes, 10/10**, while the same script writing a *function* target at the same byte class moves **56–120** defined symbols | **CONFIRMS w-skip T-e and w-joint's "no code->data edge" as mechanism claims** and withdraws this lane's own prereg §0. The model is published as a **correlation** — the list is a faithful *record* of the body's references, and the body's relocations are the untouched channel | §3 |
| **V-e** | **NEITHER CHANNEL DOES ANYTHING ALONE.** `JFP_URESTRICT` (code edges restricted to `U`) and `JFP_CODEONLY` (no `in` edges) are **each exactly `RGL`, to the digit**. Only both together move anything | the quantitative form of w-skip's owner-emitted filter reached from the other side: a data symbol reached from code is what lets its initializer contribute function roots | §2.2 |
| **V-f** | **The model's roots are 30.51× the uniform expectation and 8.58× the base rate — 21 077 of 21 229 added names are emitted** — indistinguishable from w-joint's ORACLE (30.73×/8.64×) and 7.6× further from chance than w-mark's channel | published in w-mark's exact calibration shape so all four lanes are comparable | §4.1 |
| **V-g** | **`#152` is 17.01 % of THIS model's code residual, not 58.95 %** — the free/file-scope class returns and is **63.58 %** of it. The two residuals are not comparable, because the model's recall (0.86391) is 9.5 pp below the oracle's | registered at 0.55 with a floor of 0.20 and **missed below the interval**; reported as a miss rather than reframed | §4.2 |
| **V-h** | **The one parameter fitted against data is MEASURABLY INERT.** The owner-entry gate was chosen on three disclosed TUs before the prereg; corpus-wide `JFP_UNGATED` equals `JFP` to the digit on both axes, because a `.gl` data name with no `in` record is a leaf and cannot relay | left in the frozen definition rather than removed, and reported as inert — the disclosure and the measurement in one row | §4.4 |
| **V-i** | **More than a third of every function's reference-list targets are outside `U`: 950 824 of 2 573 569 = 0.36946**, of which **192 919** are `in` owners. w-refs' `∩ U` discards all of it. (`outside U` also contains gate-skipped tag-`0x0E` records, so 192 919 and not 950 824 is the data population) | the fact the model exploits — and, per V-d, **not** a fact c2 acts on | §4.3 |
| **V-j** | **`0x10b28a9b`, the COFF symbol writer, refuses kind-1 symbols with `(+0x37 >> 0x15) & 7 ∈ {1,3}`** (`0x10b28bb1`/`0x10b28bbd`) and then dispatches by the storage-class nibble into four undecoded arms | read from the binary, modelled **nowhere**; named so the absence cannot read as success | §8 item 2 |

---

## 12. Reproducing every number here

```sh
# 0. index the capture cache -> 850 TUs at the pinned dc3 rev  (no toolchain)
python3 work/w-db/cacheindex.py <main-repo>/work/capture-cache \
        work/emitpred/magnitude/truthlist.txt work/w-db/cacheidx.tsv \
        940d07dcb0960964ad61aa5f025658f993eb46b2

# 1. the extended truth + its invariants                        (no toolchain)
python3 work/w-db/truth_data.py work/w-db/cacheidx.tsv work/w-db/dtruth \
        <main-repo>/work/w-emit/truth 16
python3 work/w-db/dupcheck.py --inj work/w-db/dtruth work/w-db/cacheidx.tsv

# 2. THE `db` READ -- T1..T5                                    (no toolchain)
work/w-db/dis.sh 0x10be7ef0 260     # the reader, its 0x2000 gate, ordinal 4
work/w-db/dis.sh 0x10b73ae0 340     # the container writer and its own gate
work/w-db/dis.sh 0x10b28a9b 400     # the COFF symbol writer's kind-1 arms
python3 work/w-db/dbscan.py work/w-db/cacheidx.tsv work/w-db/dtruth 16

# 3. the headline scan and the scores                           (no toolchain)
python3 work/w-db/scan.py work/w-db/cacheidx.tsv work/w-db/dtruth \
        <main-repo>/work/w-emit/truth work/w-db/scan.jsonl 16
python3 work/w-db/score.py work/w-db/scan.jsonl        # -> score.txt

# 4. MUT-DB and MUT-CD -- RUN real c2.dll under wibo, non-quarantined TUs
export C2RS_DC3=<dc3-tree> C2RS_WIBO=<wibo>
python3 work/w-db/mutate_db.py src/system/net/HttpReq.cpp src/system/utl/PoolAlloc.cpp
python3 work/w-db/mutate_db.py src/system/rndobj/EventTrigger.cpp src/system/utl/PoolAlloc.cpp
python3 work/w-db/mutate_db.py src/system/synth/StreamNull.cpp src/system/utl/PoolAlloc.cpp
python3 work/w-db/mutate_cd.py src/system/net/HttpReq.cpp 5              # the CONTROL
python3 work/w-db/mutate_cd.py src/system/net/HttpReq.cpp 5 --datatok    # the CLEAN arm
python3 work/w-db/mutate_cd.py src/system/utl/PoolAlloc.cpp 5 --datatok
```

All scripts are **stdlib-only** and read-only against the corpus; the mutation
scripts write only inside `work/w-db/mut*/` and restore every stream between
runs. `work/` is gitignored; the scripts and the text outputs are force-added as
records, and no IL, obj or `_CL_*` artifact is committed.
