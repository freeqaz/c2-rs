# EH on the critical path — what `.rdata$r` actually is, and what the cheap form costs

    Lane:      w-eh5, 2026-08-04, worktree `wt-w-eh5` off `master` 39dcfb7
    Prereg:    rungs/_2026-08-02-w-eh5-prereg.md — committed at `0f8195d`,
               before the first obj byte was read; scored in §9.
    Evidence:  the 871 cached reference objs of the dc3 workload, read with this
               lane's own COFF reader, plus 27 probe objs compiled at the
               workload's own flags with one axis varied at a time.
    Status:    MEASUREMENT AND RE-PLANNING. No code. Nothing under `crates/`.
               `EH_RECORDS.md` is read-only for this lane and is not edited.

**One-line statement:** *`.rdata$r` is **RTTI**, not EH — it is a pure function
of `/GR` and survives removing `/EHsc` — so rung three of the section ladder is
not Phase 5; the EH record set lands in plain `.rdata`, a name the port's writer
**already has**, which means **EH's contribution to factor C is zero**; and the
cheap no-try form is not cheap, because the count law `EH_RECORDS.md` §9.1
established on 27 probes holds on **44.66 %** of the workload's 10,642 no-try EH
functions.*

---

## 0. The known-answer gate on this lane's reader

Registered as R0 before anything else, because every number below is one reader
away from being fiction.

| registered | measured | |
|---|---|---|
| 13 distinct section names over 871 objs | **13** | ✓ |
| `.rdata$r` in **676** objs, `.xdata$x` in **67** | **676 / 67** | ✓ |
| C = **84** for the port's 6-name writer set | **84** | ✓ |
| the greedy ladder 109 → 172 → 574 → 698 → 745 → 745 → 871 | **exact, all seven rows** | ✓ |

Three independent readers now agree on these counts (lane D's, the
coordinator's, this one's). **What none of them had read is what is *inside* the
sections** — which is the whole of §1.

---

## 1. `.rdata$r` is RTTI. It is not EH, in either direction.

### 1.1 By symbol, over all 871 objs

Every symbol defined in a `.rdata$r` COMDAT across the entire workload:

| bucket | n | share |
|---|---:|---:|
| the COMDAT's own section symbol (`.rdata$r`) | 24,163 | 50.00 % |
| **`??_R1` / `??_R2` / `??_R3` / `??_R4`** — base-class descriptor, base-class array, class-hierarchy descriptor, complete-object locator | **24,163** | **50.00 %** |
| `__ehfuncinfo$` / `__unwindtable$` / `__tryblocktable$` / `__catchsym$` | **0** | **0.00 %** |

**100 % of the content symbols are RTTI and none of them is EH.** 22 objs carry
`.rdata$r` and have no `__ehfuncinfo$` anywhere; 86 objs carry EH records and
have no `.rdata$r`. The two populations are not the same set and neither
contains the other.

### 1.2 By separated axes — the falsifier, run rather than asserted

Four probes at the workload's own flags, then **one axis moved at a time** —
the discipline `STATUS.md`'s trap list exists for. `a_rtti` is a polymorphic
class with **no `try`, no `catch`, and no destructible object**; `b_ehnotry` is
`EH_RECORDS.md` §9.4's `qB2`/§10.2's `mC` shape with no polymorphism.

| probe | `/GR /EHsc` | `/GR- /EHsc` | `/GR` *no* `/EHsc` |
|---|---|---|---|
| `a_rtti` | **`.rdata$r`** ✓, no EH symbols | **`.rdata$r` GONE** | **`.rdata$r` still there** |
| `b_ehnotry` | EH records in **`.rdata`**, no `.rdata$r` | unchanged | EH records **gone** |
| `c_plain` (neither) | no `.rdata$r`, no EH | no | no |
| `d_catch` (`catch(int)`) | EH in `.rdata`, `??_R0` in `.data`, **no `.rdata$r`** | unchanged | unchanged (C4530) |

> **`.rdata$r` is emitted iff `/GR` and a vtable is emitted. `/EHsc` is
> irrelevant to it, and the EH record set never enters it — not even for a
> `catch`, whose type descriptor goes to `.data`.**

**Discriminating cells: 2 each way** — `a_rtti` at `/GR-` (kills "`.rdata$r`
needs EH" is not even the claim; it kills any EH dependence, since EH was held
constant), and `a_rtti` with `/EHsc` removed (kills "EH is necessary").
`b_ehnotry` and `d_catch` are the reverse pair: the whole EH record set with
`.rdata$r` never appearing.

### 1.3 Where the claim came from, and where it did not

`EH_RECORDS.md` **never said `.rdata$r`.** §3 and §8.3 both name the section
**`.rdata`**, plain, `Selection = 5`, associative to the function's `.text`, and
§8.4 puts the type descriptor in `.data`, `Selection = 2`. That document is
right, at 21 EH functions, and this lane re-confirms it at **11,575**.

The `.rdata$r` = EH reading appears first in ROADMAP §10.19 and propagates
verbatim into `PHASE7_PLAN.md` §1, `STATUS.md`, and BOARD **#160**. §10.19
records a *coordinator's independent verification* — and that verification
re-derived **the count of the name** with a second reader, not the contents of
the section. Two implementations agreeing on 676 says nothing about what 676 is
a count of.

This is `EH_RECORDS.md` §11.4's failure mode exactly: *"the check was nearly
performed on the two fields where the answer was already known."* Here the check
*was* performed, twice, on the one field where the answer was never in doubt.

---

## 2. Where the EH records actually are, and what that does to factor C

### 2.1 The record set is in a section the writer already has

| section | EH symbols defined in it, all 871 objs |
|---|---|
| **`.rdata`** | `__ehfuncinfo$` **11,575** · `__unwindtable$` **11,575** · `$T` (ip-to-state array) 11,566 · `__tryblocktable$` 933 · `__catchsym$` 933 |
| `.data` | `??_R0` type descriptors **8,581** |
| `.rdata$r` | **0** |
| `.xdata$x` | `_TI*` / `_CTA*` / `_CT*` — **throw**-side data (ThrowInfo, CatchableTypeArray, CatchableType), 622 records over 67 objs, entirely STLport exception classes (`bad_alloc`, `length_error`, `logic_error`, `out_of_range`, …) |

`.rdata` is **already in the port's six-name writer vocabulary**. So:

> **Factor C — the section-*name* predicate — costs EH exactly zero. Teaching
> the writer nothing new at all admits every EH record set in the workload.**

That is not the same as saying the port can *write* them: the EH `.rdata` is
`Selection = 5` **associative** to the function's `.text`, a COMDAT capability
distinct from the name, and `EH_RECORDS.md` §3 already records that
*"`.rdata` is always Selection 2" is false once EH is in scope*. Factor C does
not see that, which is one more reason C is necessary and not sufficient.

### 2.2 The ladder, re-derived with the owner of each rung corrected

Same seven steps, same seven numbers, the **labels** replaced by what the
sections were measured to contain:

| teach | C | +| what actually lives there |
|---|---:|---:|---|
| *(today)* | 84 | | |
| `.data` | 109 | +25 | statics **and** `??_R0` type descriptors (727 of 754 `.data`-carrying objs hold ≥1 `??_R0`; **245** hold nothing else) |
| **`.rdata$r`** | 172 | **+63** | **RTTI `??_R1..R4`. `/GR`. NOT EH.** |
| `.bss` | 574 | +402 | zero-initialized statics |
| `.text$yd` | 698 | +124 | `??__F` atexit thunks |
| `.xdata$x` | 745 | **+47** | **throw-site data — the only EH-owned name in the whole vocabulary** |
| `.CRT$XCU` | 745 | +0 | dynamic-initializer table |
| `.text$yc` | 871 | +126 | `??__E` dyninit thunks |

**Rung three is an RTTI rung**, and it is a data-emission problem: four COMDATs
of fixed layout per polymorphic class, plus the `??_R1` name mangling. It has no
funclets, no state model, no label surcharge and no frame discipline. It is a
**different and far cheaper phase than Phase 5**, and it is worth +63 TUs of C
where Phase 5 is worth 0.

**EH's only C-relevant name is `.xdata$x`**, worth +47 at its ladder position
and 67 at the top — and **R6a measured 0**: there is not one TU in the workload
whose *only* beyond-reach section is `.xdata$x`. As a section-vocabulary rung,
Phase 5 is worth **nothing on its own, at any position in the ladder**.

### 2.3 What EH *does* block, which is a different factor entirely

| | TUs |
|---|---:|
| objs carrying ≥ 1 EH record (`__ehfuncinfo$`) | **740 of 871** |
| of those, in section reach **today** (inside C = 84) | **12** |
| the six matched TUs | **0** |
| the 14 first-conversion targets named in `PHASE7_PLAN.md` §3 | **0 — none of them has a single EH function** |

EH blocks by **factor D** (codegen class), not C. 740 of 871 objs cannot be
byte-exact until the port emits EH record sets correctly, which is *more* than
the 676 the front page attributed to it — and **86 objs carry EH records with no
`.rdata$r` at all**, so the incumbent number was not even an over-count of the
right thing.

**The plan's own target list survives.** All 14 R7 first-conversion targets have
zero EH functions, so the `no EH sections` filter that selected them picked the
right TUs for the wrong reason. Nothing in `PHASE7_PLAN.md` §3's R1 or R7 needs
re-choosing; only §1's third bullet and R2's ordering do.

---

## 3. The cheapest case — the minimum, byte for byte

Two minima, because §1 split the question in two.

### 3.1 The minimum for `.rdata$r` (RTTI) — two lines, no EH anywhere

A ladder of nine probes at the workload flags. `.rdata$r` is **absent** from
`struct A { virtual int f(); }; int A::f(){…}`, from an out-of-line override in
a derived class, and from any TU that only *calls* through a vtable it does not
emit. It appears as soon as the **vtable itself** is emitted — a definition of an
object, or a constructor or destructor body:

```cpp
struct A { virtual int f(); };
A g_a;                                   // <- the whole source
```

Four `Selection = 2` COMDATs, 72 bytes, 6 relocations, plus `??_R0?AUA@@@8` in
`.data` and `??_7A@@6B@` in `.rdata`:

```
.rdata$r  ??_R4A@@6B@          20 B   signature 0 | offset 0 | cdOffset 0
                                      | ADDR32 ??_R0?AUA@@@8 | ADDR32 ??_R3A@@8
.rdata$r  ??_R3A@@8            16 B   signature 0 | attributes 0 | numBaseClasses 1
                                      | ADDR32 ??_R2A@@8
.rdata$r  ??_R2A@@8             8 B   ADDR32 ??_R1A@?0A@EA@A@@8 | 0
.rdata$r  ??_R1A@?0A@EA@A@@8   28 B   ADDR32 ??_R0?AUA@@@8 | numContained 0
                                      | mdisp 0 | pdisp -1 | vdisp 0
                                      | attributes 0x40 | ADDR32 ??_R3A@@8
```

The one non-mechanical part is the `??_R1` **name**: `A@?0A@EA@A@@8` encodes
`(mdisp, pdisp, vdisp, attributes)` in the back-reference alphabet, so a wrong
hierarchy is a wrong string-table entry — the same class of problem as
`__catchsym$F$k` (#143), and it belongs to the RTTI rung, not to Phase 5.

### 3.2 The minimum for the EH record set — the no-try unwind shape

```cpp
struct SE { int m; SE(); ~SE(); };
int gp(int);
int P(int a){ SE s; return gp(a); }
```

One `.rdata` COMDAT, **64 bytes**, `Selection = 5`, `Number` = the function's own
`.text`, 5 relocations — exactly `8·S + 36 + pad(4) + 8·E` at `S = 1, E = 2`:

```
  +0x00  ffffffff                    <- __unwindtable$?P@@YAHH@Z   toState = -1
  +0x04  ADDR32 __unwind$2561                                      action
  +0x08  19930522                    <- __ehfuncinfo$?P@@YAHH@Z    magic
  +0x0c  00000001                                                  maxState = 1
  +0x10  ADDR32 __unwindtable$?P@@YAHH@Z                           pUnwindMap
  +0x14  00000000                                                  nTryBlocks = 0
  +0x18  00000000                                                  pTryBlockMap = 0
  +0x1c  00000002                                                  nIPMapEntries = 2
  +0x20  ADDR32 $T2568                                             pIPtoStateMap
  +0x24  00000000                                                  pESTypeList = 0
  +0x28  00000001                                                  EHFlags = 1  (/EHsc)
  +0x2c  00000000                                                  alignment pad
  +0x30  ADDR32 $M2566 | 00000000    <- $T2568                     state 0
  +0x38  ADDR32 $M2567 | ffffffff                                  state -1
```

and outside the record: the 8-byte `{__CxxFrameHandler, __ehfuncinfo$}` prefix
with the function symbol at `Value = 8`, **two** `.pdata` COMDATs, one
`__unwind$` funclet, the `r31` establisher-frame discipline, and a **+13** label
surcharge.

**But that is not the workload's commonest shape.** The most frequent EH record
in the dc3 workload is **56 bytes** — `S = 1`, `E = 1`, a single ip-to-state
entry at state 0 — and there are **3,513** of them, every one exactly 56 bytes.
`src/App.cpp`'s `??0FilePath@@QAA@PBD@Z` is a representative and dumps
identically to the above with `nIPMapEntries = 1` and one `$M` row.

**Why `E = 1`, measured with a matched triple** (probe `e1a`/`e1b` against the
control `e1c`, one axis moved):

| probe | source | S | E | size |
|---|---|---:|---:|---:|
| `e1a` | `struct T { SE a; T(); }; T::T(){ gv(); }` — a **constructor** | 1 | **1** | 56 |
| `e1b` | `SE mk(int a){ SE s; s.m = gp(a); return s; }` — a **value return** | 1 | **1** | 56 |
| `e1c` | `int P(int a){ SE s; return gp(a); }` — the §9.1 ladder shape | 1 | **2** | 64 |

> **An object whose lifetime does not end inside the function produces only the
> raising transfer.** A constructor never runs its members' destructors on the
> normal path, and an NRVO'd return object is handed to the caller — so there is
> no state-lowering transfer and no second map entry.

That shape is **not on `EH_RECORDS.md` §9.10's unprobed list** (which names
arrays, temporaries, inlined destructors and by-value parameters). It is a new
hole, and it is the single largest one in the workload.

---

## 4. The stratification — of **740**, not 676

The 676 is the RTTI population (§1). The population Phase 5 owns is the 740 objs
carrying `__ehfuncinfo$`. Read from `FuncInfo` (9 dwords, big-endian) for every
one of the **11,575** EH functions in the workload's emitted code.

### 4.1 The instrument, gated before it was believed

`EH_RECORDS.md` §9.1's size law, applied as an **arity check** in §11.5's sense
— predict each record's length from a count field and compare:

    EH .rdata RawSize  ==  8·S + 36 + pad + 8·E        exact on 10,608 / 10,642
                                                        no-try functions (99.68 %)

All **34** exceptions are `E = 0`, where the `$T` array is absent and so is its
alignment pad (44 rather than 48) — an explained boundary, not a residue. `magic
== 0x19930522` on **11,575 / 11,575**; `EHFlags == 1` on 11,575 / 11,575
(`/EHsc`, and §11.4's mode scope holds at workload scale); `pESTypeList == 0` on
11,575 / 11,575. Every one of `EH_RECORDS.md` §8.3's byte-derived fields, taken
from 21 functions, survives a 550× larger population unchanged.

*(A note on the reading, because it nearly cost this lane the section: the record
payload is **big-endian**. Read little-endian, `nTryBlocks` still splits 0 /
non-0 correctly and every derived count looks plausible — `magic` matching 0 of
11,575 is what caught it. Compare a count, never a status.)*

### 4.2 The split

| | objs (of 740) | | EH functions (of 11,575) | |
|---|---:|---:|---:|---:|
| every EH function `nTryBlocks == 0` — **the cheap no-try form** | **448** | 60.5 % | **10,642** | **91.94 %** |
| ≥ 1 function with `nTryBlocks ≥ 1` | **292** | 39.5 % | 933 | 8.06 % |

**Every one of the 933 try-carrying functions in the whole workload has exactly
one try block and exactly one catch clause.** Not one nested try, not one
sequential pair, in 871 real TUs. The four-deep nests and four-block sequences
`EH_RECORDS.md` §11 held out its `maxState` law on do not occur here at all.

`.xdata$x` — the throw side — is in 67 objs, **all 67 of which carry EH
records**, and 0 without. (Registered as "no relationship, at chance"; refuted,
see §9.)

### 4.3 Inside the no-try form, the cost driver is `S` and the open field is `E`

| S | no-try functions | |
|---:|---:|---:|
| **1** | **7,100** | 66.72 % |
| 2 | 1,295 | 12.17 % |
| 3 | 624 | 5.86 % |
| 4 | 411 | 3.86 % |
| ≥ 5 | 1,212 | 11.39 % |

`max S = 183`, `max E = 367` — the tail is not decorative.

**And this is the number that decides whether rung three is a small rung:**

> **`EH_RECORDS.md` §9.1's count law — `E = 2S`, or `2S+1` with a Class-C tail
> branch — holds on 4,753 of 10,642 no-try functions: 44.66 %.**

| name class | n | law holds | `E ≤ S` |
|---|---:|---:|---:|
| ordinary | 7,748 | 51.4 % | 33.6 % |
| ctor `??0` | 1,731 | **17.7 %** | **71.8 %** |
| dtor `??1` | 1,138 | 39.5 % | 43.3 % |
| `operator=` `??4` | 23 | 65.2 % | 30.4 % |

The law is not wrong — §9.2 held it out at n = 3 and n = 4 and it is exact on
the shape it describes. It is **scoped to a shape that is a minority of the real
workload**, exactly as §9.7 already scoped it away from try/catch. §3.2's
matched triple names the missing mechanism.

### 4.4 The answer to "is rung three a small rung"

Objs where **every** EH function falls inside a given modelled shape:

| shape | objs | of 740 |
|---|---:|---:|
| no-try (any `S`, any `E`) | 448 | 60.5 % |
| no-try **and** `S ≤ 2` | 125 | 16.9 % |
| no-try **and** `S = 1` | 65 | 8.8 % |
| **no-try and `E` inside §9.1's law — i.e. everything the project can actually predict today** | **26** | **3.5 %** |

**26 of 740.** That is the honest size of what is modelled, and it is the number
that answers question 2 of this lane's brief.

---

## 5. Which board items block the cheap form

### 5.1 The registered four

| item | blocks the cheap no-try form? | measured basis |
|---|---|---|
| **#53** — the 8-byte `{__CxxFrameHandler, __ehfuncinfo$}` prefix, function symbol at `Value = 8` | **YES, unconditionally** | **11,573 of 11,573** EH functions with a `.text` symbol have `Value == 8`, and **0** non-EH `.text` function symbols in the entire workload have `Value == 8`. A perfect biconditional over 11,573 cells, up from `EH_RECORDS.md` §8.1's 21. There is no EH record set without it |
| **#143** — `__catchsym$F$k`, the per-function symbol ordinal | **NO** | `__catchsym$` count is **933** = exactly the try-carrying functions. The no-try form emits none. **Bonus, since it was one query:** §8.6's fitted `$k(j) = maxState + Σ nCatches − nTryBlocks + j` is **exact on 931 of 933** workload try functions (929 of them trivially, `$k = 2`); the two misses are `?Apply@FlowMathOp@@QAAMM@Z` (`maxState 10`, `$k` 11 vs 10) and `?CallCheatScript@CheatsManager@@QA…` (`maxState 11`, `$k` 15 vs 11) — both a try **with destructible objects alongside**, the shape §9.7 says is unmeasured. #143 is 99.8 %-determined on the workload, with its residue named |
| **#144** — `nIPMapEntries` for try/catch shapes | **NO as registered — but its *sense* does** | The try half is 933 functions and the cheap form has none of them. **However**, `E` is equally unmodelled for **5,889 of the 10,642 no-try functions** (§4.3). #144 is registered against the wrong half of the population: the field is open on **55.3 % of the cheap form**, which is 6× the entire try population |
| **#146** — repair `G = 4 + Σmint` in `EH_RECORDS.md` §9.8 | **YES** | The label counter is TU-global. §9.8 already measures the consequence: §1.1 alone predicts `eh-dtor` at 5 against a truth of 18, so every later function in the TU is 13 label numbers low, and 34 low after an `eh-dtor4`. Six wrong bytes per `$M`/`$T` reference, in an obj that still links. The workload's `max S = 183` puts the error far past the band §9.8's twelve rows were fitted on |

### 5.2 The blockers of the cheap form that have no board row

Registered as R5b before looking. All four are measured or document-derived, none
is a guess:

1. **`nIPMapEntries` for the no-try majority** (§4.3) — 55.3 % of the cheap
   form. This is the sharpest one and it did not exist as an item.
2. **The state model** — `EH_RECORDS.md` §8.7 item 4: the unwind map, the
   ip-to-state map and the state assignment are one whole-function dataflow
   pass, and the port has no such pass. §8.7's ordering (*"the state model is the
   real cost, and it is item 3, not item 5"*) is confirmed by §4.3: the field
   that resists is precisely the dataflow one.
3. **The funclet emitter and the `r31` establisher-frame discipline** — `S`
   funclets per function, `addi r31,r12,-F`, entry-SP-relative homing. Every
   funclet in the probe corpus is 5–11 instructions, but no non-EH body in the
   port does any of it.
4. **`Selection = 5` associative `.rdata`** — a COFF-writer capability that
   factor C's name predicate cannot see (§2.1).

---

## 6. Phase 5, re-ranked against its new job — and it declines

Its old ranking was **function mass**: 233,526 blocked functions
(`EH_RECORDS.md` §7.5, on the statement-count axis; §10.3 re-measured it at
237,180 on `maxState`). Its briefed new ranking is **TUs brought into section
reach**. Measured:

| the question | the number |
|---|---:|
| TUs Phase 5 brings into section reach, on its own | **0** |
| TUs whose only beyond-reach section is EH-owned (`.xdata$x`) | **0** |
| `.xdata$x`'s marginal worth as a ladder rung | +47 at position 5, 67 at the top |
| the incumbent's claim — `.rdata$r`, +63 at rung 3 | **belongs to RTTI** |
| TUs Phase 5 unblocks *inside today's* C = 84 | **12** — all no-try, 5 of them entirely `S = 1` |
| TUs that cannot be byte-exact until Phase 5 exists, terminally | **740 of 871** |

**Phase 5 is not on the TU-assembly critical path in the sense §10.19 meant it.**
It moves factor C by zero. It is on the critical path in a different and larger
sense: it gates factor **D** over 740 of 871 objs, which is more than the 676 the
front page credited it with, and 86 of those objs have no `.rdata$r` at all.

**And it is still expensive** — the declined outcome R6d registered in advance:

* the cheap no-try form covers 448 of 740 objs (60.5 %), but only **26 objs
  (3.5 %)** fall entirely inside what is actually modelled today;
* the one field that resists is a whole-function dataflow pass the port does not
  have, and it is open on **55 %** of the cheap form, not on the try shapes the
  board item names;
* the label counter (#146) is TU-global, so a wrong `E` is wrong bytes in every
  *later* function of the TU as well as in the EH one;
* #53 is a hard prerequisite for every one of the 11,573.

**The re-ordering this implies.** RTTI — four fixed-layout COMDATs, a mangled
`??_R1` name, no funclets, no state model, no label surcharge, no frame
discipline — is worth **+63 TUs of C** and is what rung three of the ladder
actually is. It should be scheduled as rung three **instead of** Phase 5, which
belongs after it, is worth 0 C, and needs a dataflow pass first.

---

## 7. What stays NOT MODELLED after this lane

* **`nIPMapEntries` for 55.3 % of no-try functions.** §3.2 names the mechanism
  for the `E = 1` family (3,513 functions, lifetime outlives the function) and
  does **not** model the rest — `S = 1, E = 5` (194), `S = 2, E = 2` (271),
  `S = 3, E = 4` (97) and the long tail are each unexplained here.
* **Everything on the try side.** 933 functions, one try block and one catch
  clause each; §9.7's ip placement, and #143's two named misses, stand open.
* **`.xdata$x`'s contents as a shape.** Counted and identified (`_TI`/`_CTA`/
  `_CT`, 622 records, 67 objs) but not decoded field by field; every one comes
  from STLport headers, so a workload-only reading of it may not generalize.
* **The `??_R1` name mangling** and the RTTI rung's own layout beyond the
  4-COMDAT minimum of §3.1 — one probe, one class, no multiple or virtual
  inheritance. `??_R2` with more than one base is unprobed.
* **`/Ox`, `/O2`, packed.** Every number here is `/O1 /Oi /EHsc /GR`, the
  workload's, per `EH_RECORDS.md`'s standing scope.
* **Whether the 12 in-reach EH TUs are inside factors A, B or D.** This lane
  measured C and the EH shapes; the joint is not computed.

---

## 8. Reproduction

`work/` is gitignored. The scripts are small and self-contained.

```sh
# the reader and the five analyses (871 cached reference objs; no toolchain needed)
python3 work/w-eh5/r0.py     # R0 gate: 13 names, 676/67, C = 84, the ladder
python3 work/w-eh5/r1.py     # what is defined in .rdata$r / .xdata$x / .rdata / .data
python3 work/w-eh5/r3.py     # FuncInfo for all 11,575 EH functions (BIG-ENDIAN)
python3 work/w-eh5/r4.py     # the arity check + the (S,E) joint distribution
python3 work/w-eh5/r5.py     # name classes, the 9.1 law at scale, obj coverage
python3 work/w-eh5/r6.py     # section attribution, the ladder, R6a
python3 work/w-eh5/r7.py     # #53 at workload scale; the $k distribution
python3 work/w-eh5/r8.py     # #143's formula on 933 workload try functions
python3 work/w-eh5/dump.py <obj> '.rdata$r'    # byte dump with relocations by name

# the probes — the workload's own flags, ONE axis moved at a time
bash work/w-eh5/probe.sh     # a_rtti / b_ehnotry / c_plain / d_catch
                             # x {W, /GR-, no /EHsc, neither}
# the minimal-source ladders and the E=1 triple are r1..r5, s1..s4, e1a/e1b/e1c
# under work/w-eh5/probe/, all at
#   /nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc
```

The obj→TU map is `work/phase7plan-d/objmap.pkl` from lane `w-phase7plan`'s
worktree, read but not modified; the cache itself is
`work/capture-cache/<hash>/out.obj`.

---

## 9. Pre-registration, scored — 11 hits, 3 misses, 1 refuted

Registered at `0f8195d`, before the first obj byte was read.

| # | registered | outcome | score |
|---|---|---|---|
| R0a–c | 13 names / 676 / 67 / C = 84 | all exact, plus all 7 ladder rows | **GATE PASSED** |
| R1a | ≥ 90 % of `.rdata$r` symbols are `??_R1..R4`; point 97 % | **100 %** of content symbols (the other half are the COMDATs' own section symbols) | **HIT** |
| R1b | EH-record symbols < 1 % of `.rdata$r`; **abandon H1 at ≥ 10 %** | **0.00 %**, zero of 24,163 | **HIT** |
| R1c | ≥ 1 obj with `.rdata$r` and no `__ehfuncinfo$` | **22** (and 86 the other way) | **HIT** |
| R2 | where the records live: plain `.rdata` 0.55 / `.xdata$x` 0.35 / `.rdata$r` 0.10 | **plain `.rdata`, 11,575 / 11,575** | **HIT** on the modal branch |
| R2b | objs with ≥ 1 `__ehfuncinfo$` = **520**, interval [350, 700] | **740** | **MISS, above the interval** — and against my declared deflationary bias, which is the protocol working. EH is in *more* objs than the incumbent credited it with |
| R3a | objs no-try-only = 70 % [50, 85] | **60.5 %** | **HIT** |
| R3b | objs with ≥ 1 try = 30 % [15, 50] | 39.5 % | **HIT** |
| R3c | EH functions no-try = 85 % [70, 95] | **91.94 %** | **HIT** |
| R3d | `.xdata$x` ∩ EH records at chance — "no relationship" | **67 of 67**, 0 without | **REFUTED.** `.xdata$x` is throw-side EH data; it *is* EH, just not the record set. I registered the right hypothesis for `.rdata$r` and the wrong one for its neighbour, and for the same reason both times: I reasoned from the name |
| R3e | median `maxState` 1; ≥ 60 % of no-try at `S = 1` | median 1; **66.72 %** | **HIT** |
| R4a | `.rdata$r` survives removing `/EHsc`, dies at `/GR-` | exactly, 2 discriminating cells each way | **HIT** |
| R4b | the minimal EH record is `8·1 + 36 + pad + 8·2` | **64 bytes, exact**. But the workload's *commonest* is **56** (`E = 1`), a shape I did not register at all | **HIT on the probe, blind to the workload** |
| R5 | #53 YES · #143 NO · #144 NO · #146 YES | #53 ✓ (11,573/11,573) · #143 ✓ · #146 ✓ · **#144 NO as registered but its sense blocks 55 % of the cheap form** | **3 HIT, 1 MISS** — and the miss is the lane's most useful output |
| R5b | ≥ 1 blocker of the cheap form with no board row; candidates named as the state model and the funclet emitter | found **four**, and the sharpest (`nIPMapEntries` for no-try) is not one of the two I named | **HIT, better than registered** |
| R6a | TUs whose only beyond-reach section is EH-owned = **8** [0, 40] | **0** | inside the interval at its floor; **point badly wrong** — call it a MISS that the interval covered |
| R6b | EH's total C contribution ≤ 100 TUs [0, 150] | **0 today**, 67 at the top of the ladder | **HIT** |
| R6d | the decline branch, registered so it could not be claimed as a surprise | **it fires** | — |

**The declared bias was deflationary about EH, and it cost exactly where it was
declared.** R2b and R6a are both "I expected EH to matter less than it does", and
R3d is the same reflex applied to a section I had already decided was not EH. The
one place the bias did *not* operate is H1 itself, which was graded on 24,163
symbols and two separated-axis probe pairs rather than on the naming convention
that suggested it — and that is the only reason the result is usable, because
the naming convention is exactly what produced the incumbent claim.

---

## 10. Clean-room ledger

Black-box throughout: COFF bytes of our own captures and of the cached reference
objs, compile-and-observe probes with flags varied one axis at a time, and the
`/FAsc` listing **not used at all** (§10.16's third strike; every byte here comes
from an obj). **Disassembly-derived constants adopted: none.** The blanket
clean-room claim stands unweakened.
