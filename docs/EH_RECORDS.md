# EH obj structure — enough to size the rung, not to build it

The dc3 workload compiles `/EHsc`. This is a read-only characterization of what
that adds to an obj, captured from the real toolchain (`cl.exe`
16.00.11886.00 under wibo 1.0.1-23, `/O1 /GS- /c /EHsc`) with
`scripts/gt_capture.sh` + `scripts/gt_dump.py`. **No part of this is
implemented and nothing here should be implemented from this document alone** —
it exists so the EH rung can be ordered honestly against the others.

Two probes:

```cpp
// eh1 — a catch funclet
int g(int);
int f(int a){ try { return g(a); } catch(int e) { return e+1; } }

// eh2 — an unwind (destructor) funclet
struct S { S(); ~S(); int m; };
int g(int);
int f(int a){ S s; return g(a)+s.m; }
```

---

## 1. The function symbol is no longer at offset 0 of its `.text`

`?f@@YAHH@Z` has **`Value = 0x8`** in both probes. The COMDAT opens with an
8-byte, two-word, **non-code** prefix, each word carrying an `ADDR32`:

```
  .text+0x0   ADDR32 -> __CxxFrameHandler
  .text+0x4   ADDR32 -> __ehfuncinfo$?f@@YAHH@Z
  .text+0x8   7d8802a6  mflr r12          <- the function entry
```

Every consumer of "the function starts at 0" in `c2-core::coff` is wrong for an
EH function. This is the single largest structural difference and it is visible
before any funclet logic.

---

## 2. Two `.pdata` COMDATs per function, and `BeginAddress` has an addend

Both are `Selection = 5` (ASSOCIATIVE) with `Number = 5` (the function's own
`.text`), and both relocate `BeginAddress` with an `ADDR32` against the
**function symbol** — whose `Value` is 8. The stored u32 is the addend.

eh1 (`.text` = 116 B):

| sec | raw | addend | unwind word | covers |
|---|---|---|---|---|
| 6 | `00 00 00 48  c0 00 09 02` | 0x48 | `0xc0000902` | `0x50 .. 0x74`, 9 words |
| 7 | `00 00 00 00  c0 00 10 07` | 0x00 | `0xc0001007` | `0x08 .. 0x48`, 16 words |

eh2 (`.text` = 136 B):

| sec | raw | addend | unwind word | covers |
|---|---|---|---|---|
| 6 | `00 00 00 58  40 00 0a 04` | 0x58 | `0x40000a04` | `0x60 .. 0x88`, 10 words |
| 7 | `00 00 00 00  c0 00 16 06` | 0x00 | `0xc0001606` | `0x08 .. 0x60`, 22 words |

Reading:

* `BeginAddress = &f + addend`, and `&f = .text + 8`. eh1's funclet symbol
  `__catch$2554` is at `.text+0x50 = 8 + 0x48` ✓; eh2's `__unwind$2561` is at
  `.text+0x60 = 8 + 0x58` ✓.
* The unwind word keeps the shape
  `flags | (len_words << 8) | prolog_words` with `len_words` covering only the
  code, never the 8-byte prefix.
* **The funclet's `.pdata` COMDAT is emitted BEFORE the main body's** (section 6
  is the funclet in both probes), and its `$T` label number is the *higher* of
  the two — the same reverse-emission order as the `.rdata` pool and the callee
  externals (`CODEGEN_FRAMED_CALLS.md` §4.1).

### Bit 31

`0x80000000` is set on eh1's main record, eh1's **catch** funclet, and eh2's
main record — and **clear** on eh2's **unwind** funclet. The one thing that
tracks it exactly across the four records is the 8-byte handler prefix:

> **Bit 31 is set iff the covered region is preceded by the
> `{__CxxFrameHandler, __ehfuncinfo$…}` prefix.** eh1's catch funclet has its
> own prefix (`ADDR32` relocations at `.text+0x48` and `+0x4c`, entry at
> `+0x50`); eh2's unwind funclet does not (`__unwind$2561` is `.text+0x60` and
> the word there is real code, `3becff90 addi r31,r12,-112`).

Four records is a thin basis for that rule and it is offered as the reading that
fits, not as established. A nested try, or a function with a catch *and* a
destructor, would test it in one capture.

---

## 3. The EH data sections

eh1 gains three sections beyond the two `.pdata`:

| # | name | size | Chars | aux |
|---|---|---|---|---|
| 8 | `.rdata` | 96 | `0x40401040` | cksum `0x5ca81f8a`, **Number=5, Sel=5** |
| 9 | `.data` | 11 | `0xc0301040` | cksum `0x0a73aee7`, Number=0, **Sel=2** |

The EH `.rdata` is **associative to the function's `.text`** (Sel=5), unlike the
FP-constant `.rdata` which is Sel=2 — so "`.rdata` is always Selection 2" is
false once EH is in scope. It carries four static symbols at fixed offsets and
seven relocations:

```
  __unwindtable$?f@@YAHH@Z      +0x00
  __catchsym$?f@@YAHH@Z$2       +0x10
  __tryblocktable$?f@@YAHH@Z    +0x20
  __ehfuncinfo$?f@@YAHH@Z       +0x34
  reloc +0x14 ADDR32 ??_R0H@8                      (the type_info for `int`)
  reloc +0x1c ADDR32 __catch$2554                  (the funclet entry)
  reloc +0x30 ADDR32 __catchsym$…$2
  reloc +0x3c ADDR32 __unwindtable$…
  reloc +0x44 ADDR32 __tryblocktable$…
  reloc +0x4c ADDR32 $T2563                        (a label in this same .rdata)
  reloc +0x58 ADDR32 $M2562                        (a label in .text)
```

with the magic `19 93 05 22` at `__ehfuncinfo+0x0` (`0x19930522`, the MSVC
`FuncInfo` magic). The `.data` COMDAT is the `??_R0H@8` type-descriptor,
Selection 2, holding an `ADDR32` to the external `??_7type_info@@6B@`.

eh2 has no `.data` (no type descriptor is needed for a destructor) and a
64-byte `.rdata` with `__unwindtable$` + `__ehfuncinfo$` and an `ADDR32` to
`__unwind$2561`.

---

## 4. The body itself changes shape

An EH function uses **r31 as a frame pointer**, established *before* the `stwu`,
and addresses its locals off it:

```
   0008  7d8802a6  mflr r12
   000c  9181fff8  stw  r12,-8(r1)
   0010  fbc1ffe8  std  r30,-24(r1)
   0014  fbe1fff0  std  r31,-16(r1)
   0018  3be1ff90  addi r31,r1,-112        <- frame pointer = the FUTURE SP
   001c  9421ff90  stwu r1,-112(r1)
   ...
   002c  817f0050  lwz  r11,80(r31)        locals via r31, not r1
   ...
   0040  383f0070  addi r1,r31,112         epilogue restores SP from r31
```

and the funclet re-derives it from **r12** on entry (`addi r31,r12,-112`), which
is how the personality routine hands the establisher frame to a funclet.

eh1 also carries a state variable: `li r0,0 ; stw r0,4(r1)` in the prologue —
the EH state index at `SP+4`, i.e. inside the 8 bytes at `SP+0..8` that a
non-EH frame never touches (`CODEGEN_FRAMED_CALLS.md` §1.1 records them as
reserved-and-unwritten). That is what they are reserved *for*.

---

## 5. Sizing

An EH rung is not "one more shape". It needs, at minimum:

1. the 8-byte handler prefix and a function symbol at `Value = 8` — which
   breaks an invariant every existing emitter path assumes;
2. `.pdata` with a non-zero `BeginAddress` addend, and **N records per
   function** emitted funclet-first;
3. a new `Selection = 5` `.rdata` layout with four internal symbols, a magic
   word, and cross-references to `.text` labels (`$M`, `$LN`) that the label
   planner does not currently allocate;
4. a `.data` type-descriptor COMDAT per caught type, plus the `??_R0*` /
   `??_7type_info@@6B@` externals;
5. funclet code generation, with the r12→r31 establisher-frame convention;
6. a frame-pointer discipline (r31) that differs from every non-EH body.

Items 1–4 are obj structure and are mechanical once measured. Item 5 is a
codegen problem the size of the whole framed-call rung. **The honest sizing is
that EH is a later rung than everything in `CODEGEN_FRAMED_CALLS.md` §7, and
that its obj-structure half could be measured to completion cheaply if a
downstream rung ever needs it before the codegen half.**

Note that `/EHsc` on the command line does *not* by itself change anything: the
per-function optimization word is unmoved (`docs/OPT_MODE.md` §5), and a
function with no try block and no object with a destructor gets none of the
above. The workload's `/EHsc` is only a cost where the function actually needs
unwind data.

---

## 6. The sub-object boundary — where a generated ctor/dtor crosses into EH (2026-07-31, WRD)

§5 ends *"a function with no try block and no object with a destructor gets none
of the above"*. True, and it hides the boundary that matters for ranking: a
**compiler-generated constructor or destructor** has objects with destructors by
definition, and it is on the cheap side of the boundary only under a condition
nothing had stated. Measured, `work/WRD/probe/p6.cpp`, the workload's own flags
(`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc`):

| body | `.text` | EH funclet |
|---|---|---|
| `~One(){}` — ONE destructible member, nothing else | **4 B** (`b ??1MemA`) | no |
| `Ct1::Ct1(){}` — ctor, one base, nothing else | 48 B | no |
| `~Two(){}` — **two** destructible members | 120 B | **YES** |
| `~OneB(){ Fini(); }` — one member **plus one body statement** | 116 B | **YES** |
| `Ct2::Ct2(){ Init(); }` — ctor, one base **plus one body statement** | 120 B | **YES** |

> **Exactly one sub-object statement and nothing else is a bare branch. A second
> sub-object, or any other statement beside it, is the whole of §1–§5.**

That is not a frame-class step. Crossing it mints, per function: the two-word
`__CxxFrameHandler` / `__ehfuncinfo$` prefix (§1), a **second** `.pdata` COMDAT
(§2), a 64-byte `Selection = 5` `.rdata` with five relocations (§3), an unwind
funclet emitted after the body in the same COMDAT with the r12→r31 establisher
convention (§4), and an r31 frame-pointer discipline. `??1OneB@@QAA@XZ` in full,
relocations aligned:

```text
  <ADDR32 __CxxFrameHandler> <ADDR32 __ehfuncinfo$??1OneB@@QAA@XZ>
  mflr r12 ; stw r12,-8(r1) ; std r30,-24(r1) ; std r31,-16(r1)
  addi r31,r1,-112 ; stwu r1,-112(r1)
  mr   r30,r3 ; stw r3,132(r31)      ; `this` in BOTH a GPR and the EH frame
  bl   ?Fini@OneB                    ; the BODY statement — emitted FIRST
  mr   r3,r30
  bl   ??1MemA                       ; the sub-object dtor — emitted LAST
  addi r1,r31,112 ; lwz r12,-8(r1) ; mtlr r12 ; ld r30,-24(r1) ; ld r31,-16(r1) ; blr
  ---- unwind funclet, same COMDAT ----
  addi r31,r12,-112
  mflr r12 ; stw r12,-8(r1) ; stwu r1,-96(r1)
  lwz  r3,132(r31)                   ; `this` from the PARENT frame via r12
  bl   ??1MemA
  addi r1,r1,96 ; lwz r12,-8(r1) ; mtlr r12 ; blr
```

Two further facts, both free and neither recorded anywhere:

* **The emission order is the reverse of the IL statement order.** `.ex` puts the
  sub-object statement — the one carrying the `5C <int> <f>` trailer — *first*
  and the body statement second; the obj emits the body first and the sub-object
  destructor last. One bit, and it separates.
* **`this` is stack-homed at `132(r31)` as well as held in r30**, because the
  funclet reaches it only through the establisher frame. No non-EH body in the
  port does this.

### 6.1 What it retires

`shapes/ctor_dtor.rs`'s doc comment measures `~Two(){}` at the **fixture** profile
(`/Ox`, no `/EH`) as *"a frame, a callee-saved register and a call order this rung
does not model"* — `or r31,r3,r3 ; addi r3,r3,4 ; bl ; or r3,r31,r31 ; bl`. That
is correct at that profile and confirmed here (`p6.cpp` at `/Ox /GS- /c` has no
`__ehfuncinfo` anywhere). **It is not the workload's answer**, and every census
row is counted on the workload. Do not size a ctor/dtor widening from the
fixture-profile capture.

### 6.2 What it costs the board, measured

`cf-expr-0x5C` — the statement-layer decoder stopping on the `5C` sub-object
trailer — is **309,804 functions, 17.4 % of everything blocked** on the 878-TU
workload. The three rows WCH and WCL both ranked second and described as *"this
rung's body with the `B9 <formal>` swapped for a designator"* are inside it and
are none of that:

| census key | n | what it actually is |
|---|---|---|
| `expr-call-in-expr-recv-field-off0-then-chain-bind-whole` | 2,666 | dtor, member sub-object at **offset 0**, + one body statement |
| `expr-call-in-expr-recv-intrinsic-this-adjust-then-chain-bind-whole` | 1,686 | dtor, **base** sub-object (intrinsic 2113, adjust 0), + one body statement |
| `expr-call-in-expr-recv-field-then-chain-bind-whole` | 836 | dtor, member sub-object at **nonzero offset**, + one body statement |

There is no chain in any of them. `Blocker::ChainBind` is *"a `99` at depth 0 in a
value position"*, and here that `99` is the bind of the **body statement's own
member call on `this`** — `~T(){ Fini(); }`. Reproduced exactly, all three keys,
from `work/WRD/probe/p5.cpp` under the workload flags, and 33 of 33 census
representatives across 26 witness TUs carry both the `5C` statement trailer and
the `5E 01` one-sub-object trailer.

**Realizable without the EH model: 0 of 5,188.**

---

## 7. The EH axis, and the split of `cf-expr-0x5C` (2026-07-31, WEH)

§6 measured the boundary and left the population across it unmeasured — and
**nothing in the census key says which side a body falls on**. This section adds
the axis that does, and reports the split. It is a **decode-only** measurement on
the model of `docs/rungs/2026-07-31-cflow-decode.md`: it moves the census by
**0** and is not a rung.

### 7.1 What `5C` actually is — the row was mis-described

`ctor_dtor.rs` calls `5C <int> <f>` an *"opaque statement trailer"* of the
generated destructor. It is not specific to generated destructors, and the row
named after it is not a ctor/dtor row. **MEASURED**, `work/WEH/probe/p1.cpp` at
the workload's own flags (`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc`):

```cpp
int userfn(int a) { MemA s; g(a); return a + 1; }     // MemA has a destructor
```

carries `5C A6 43 8C 20 01` — and gets an `__ehfuncinfo$?userfn@@YAHH@Z`. There
is no sub-object anywhere in it. **`5C` marks the end of a statement in which an
object with a destructor became live**; in a compiler-generated constructor or
destructor that is the sub-object statement, and in an ordinary function it is
the local or the temporary. `5D` and `5E` are the constructor-side and
destructor-side **count** trailers, `<varint n> <varint state>`, whose `n` is how
many such objects the body's EH state tracks.

Widths, all measured on those probes and none inferred:

| token | payload | witnesses |
|---|---|---|
| `5C` | `<TYPE> <varint>` | TYPE is `86 41 74` (3 B) and `A6 43 8C 20` (4 B); state `01`, `03`, and the escape `80 01 01 00 00` |
| `5D` | `<varint n> <varint>` | `5D 01 21`, `5D 01 80 A1 00 00 00` |
| `5E` | `<varint n> <varint>` | `5E 01 21`, `5E 02 21`, `5E 01 23`, `5E 01 01` |

Neither a fixed width nor a plain-byte read survives the corpus. The falsification
is the scanner's standing one — land exactly on the seven-byte function tail with
every `54 <k>` depth agreeing — and **265,683 bodies that previously stopped on a
marker now walk through it to the tail**. A wrong width here desynchronizes and
the depth invariant catches it within a statement or two, so that number is the
width's evidence, not the probes alone. The remaining 44,688 stop later, on a
different opcode.

### 7.2 The axis, graded against the obj

Per body: count the `5C` statements, read the `5D`/`5E` count, and count the
**other** statements (`4B` ends, less the `5C` ones, less a trailer standing in
statement position rather than operand position — both spellings occur in one
probe). The key is then `eh-bare` / `eh-plus-stmt` / `eh-multi`, plus
`eh-partial` for a body that carries a marker and then stops decoding and
`eh-unknown` for one that stops before any marker.

Fourteen hand-written functions, `work/WEH/probe/p1.cpp` and `p2.cpp`, both sides,
at the workload's flags. The last column is the **obj**, inspected for an
`__ehfuncinfo$` — not a prediction:

| source | census key | EH key | `__ehfuncinfo$` |
|---|---|---|---|
| `~One(){}` — one member | `empty-dtor-member` (in class) | `eh-bare` | no |
| `Ct1::Ct1(){}` — one base | `expr-intrinsic-this-adjust` | `eh-bare` | no |
| `~Pad(){}` — one member at offset 4 | `empty-dtor-member-adjusted` (in class) | `eh-bare` | no |
| `~D1(){}` — one base, intrinsic 2113 | `empty-dtor-delegation` (in class) | `eh-bare` | no |
| `void onlylocal(){ MemA s; }` | `expr-call-in-expr-recv-object-then-op-0x5C` | `eh-bare` | no |
| `void onlytemp(){ mk(); }` | `expr-op-0x9B` | *stops first* | no |
| `~Two(){}` — two members | `…-then-call-recv-field-whole` | `eh-multi` | **yes** |
| `void twolocals(){ MemA s; MemB t; }` | `…-recv-object-then-op-0x5C` | `eh-multi` | **yes** |
| `~OneB(){ Fini(); }` | `…-then-chain-bind-whole` | `eh-plus-stmt` | **yes** |
| `Ct2::Ct2(){ Init(); }` | `expr-intrinsic-this-adjust` | `eh-plus-stmt` | **yes** |
| `int userfn(int a){ MemA s; g(a); return a+1; }` | `…-recv-object-then-op-0x5C` | `eh-plus-stmt` | **yes** |
| `int trycatch(int a){ try{…}catch(int){…} }` | `body-0x60` | *stops first* | **yes** |
| `int plain(int a){ g(a); return a+1; }` | `call-ref-0xB9` | `eh-none` | no |
| `int plain2(int a){ return a+1; }` | `straight-line` | `eh-none` | no |

**Twelve classified, zero wrong, two honestly undecidable.** Two facts fall out
of the table that are worth as much as the split:

* **`Ct1::Ct1(){}` and `Ct2::Ct2(){ Init(); }` are the SAME census key** —
  `expr-intrinsic-this-adjust`, the board's #2 blocker at 141,800 — and they are
  on opposite sides of the boundary. One is a 48-byte body the port could reach;
  the other needs the whole of §1–§5. That is the axis's reason to exist, stated
  in two functions.
* **The cheap side is not a property of being a generated destructor.**
  `void onlylocal(){ MemA s; }` is an ordinary function with no EH record. It is
  a property of the **count**: one object live, one statement, nothing else.

### 7.3 The split, on the 878-TU workload

Scan on `6d7dbc7`, `work/WEH/scan-weh.jsonl`. Census **685,165 / 2,462,571 =
27.82 %**, unchanged; every blocking-feature count, every frame-class count and
the gate-disagreement count are **byte-identical to the baseline** (deltas all
zero, disagreement 0 → 0).

The three control-flow rows that stopped on a marker are now decoded, and the
accounting is exact:

| | before | after |
|---|---|---|
| `cf-expr-0x5C` | 309,804 | 0 |
| `cf-expr-0x5D` | 566 | 0 |
| `cf-expr-0x5E` | 1 | 0 |
| **total moved** | **310,371** | redistributed, **+310,371** |
| bodies decoded end to end | 1,864,128 (75.7 %) | **2,129,811 (86.5 %)** |

and the set of bodies carrying an EH marker is **exactly** those 310,371 — it
cannot be otherwise, since a walk that stopped earlier never saw one.

| EH class | n | share of all 2,462,571 |
|---|---|---|
| `eh-none` | 1,864,128 | 75.7 % |
| `eh-unknown` | 288,072 | 11.7 % |
| `eh-plus-stmt` | 160,944 | 6.5 % |
| `eh-bare` | 76,845 | 3.1 % |
| `eh-partial` | 44,688 | 1.8 % |
| `eh-multi` | 27,894 | 1.1 % |

**THE SPLIT.** Of the 310,371 bodies that carry an EH marker:

* **cheap side — `eh-bare`, 76,845 = 24.8 %.** No handler prefix, no second
  `.pdata`, no funclet. Of these **35,964 are already in class** (all three
  `empty-dtor-*` buckets, and *only* those — the control group is exact), leaving
  **40,881 reachable without any EH model at all**.
* **EH side — 233,526 = 75.2 %.** `eh-plus-stmt` 160,944 + `eh-partial` 44,688 +
  `eh-multi` 27,894. Every one of these needs the whole of §1–§5.

Of the **blocked** residue (310,371 − 35,964 = 274,407): **85.1 % is behind the EH
model and 14.9 % is not.**

`eh-partial` counts as the EH side on a structural argument, not an assumption:
the bare shape decodes end to end by construction, so a walk that stops *after* a
marker has met content the bare shape does not have. The exception is the ≤566
bodies whose first marker is a `5D`/`5E` rather than a `5C`; even placing all of
`eh-partial` on the cheap side, the EH side is still 188,838 = 60.8 %.

### 7.4 What it costs the board

The axis crossed with the blocking feature. This is the correction the ranking
needs, and it is large:

| census key | total | behind EH | % EH | `eh-bare` | `eh-unknown` |
|---|---|---|---|---|---|
| `expr-op-0x27` | 411,967 | 33,663 | 8.2 % | 778 | 50,109 |
| `expr-intrinsic-this-adjust` | 141,800 | 36,475 | **25.7 %** | 6,875 | 30,579 |
| `expr-intrinsic-base-member-addr` | 113,981 | 71,219 | **62.5 %** | 2 | 6,081 |
| `expr-bit-and` | 32,381 | 32,364 | **99.9 %** | 0 | 2 |
| `expr-call-in-expr-recv-object-then-branch-brtrue` | 23,633 | 23,614 | **99.9 %** | 0 | 12 |
| `body-0x9B` | 27,073 | 14 | 0.1 % | **16,738** | 298 |
| `body-cflow-label` | 48,102 | 4,317 | 9.0 % | 0 | 19,363 |
| `expr-call-in-expr-recv-load-then-bit-and-and-branch-more` | 102,374 | 4 | 0.0 % | 0 | 0 |

Read it in both directions. **`expr-bit-and` (32,381) and
`expr-call-in-expr-recv-object-then-branch-brtrue` (23,633) are 99.9 % behind the
EH model** — 56,014 functions that no expression rung can reach, and neither row
says so. **62.5 % of `expr-intrinsic-base-member-addr`**, the board's #3 row, is
behind it too. The other direction is as useful: the #1 row `expr-op-0x27` is
only 8.2 % behind EH, and `body-0x9B` is 61.8 % **`eh-bare`** — 16,738 functions
that need no EH model at all.

The cheap side's own widening order, all `eh-bare` and all blocked (40,881 total):

| census key | n |
|---|---|
| `body-0x9B` | 16,738 |
| `expr-intrinsic-base-upcast` | 8,277 |
| `expr-intrinsic-this-adjust` | 6,875 |
| `…-recv-intrinsic-this-adjust-then-plumbing-0x3A` | 2,058 |
| `…-recv-field-off0-then-plumbing-0x3A` | 1,655 |
| 9 further rows ≥ 300 | 4,753 |
| the tail | 525 |

### 7.5 The conclusion, and what it does not say

**EH is the next phase, not a distant one** — but the reason is the *stock*, not
the ranking of any one row. 233,526 functions — **13.1 % of everything blocked** —
are behind a model that does not exist, and they are spread across rows that each
look like ordinary expression work. No expression rung can retire them and no first-blocker
histogram can see them. §5 above sizes the work and roadmap task
#53 (the function symbol moves to `Value = 8`) is its first step.

It does **not** say EH is the *cheapest* next thing. 40,881 functions sit on the
cheap side already measured, and §5 item 5 (funclet codegen with the r12→r31
establisher convention) is still the size of the whole framed-call rung. The
honest reading is that the obj-structure half (§5 items 1–4) is now known to
unblock a population worth measuring against, which is exactly the condition §5
named: *"its obj-structure half could be measured to completion cheaply if a
downstream rung ever needs it."* A downstream rung does.

### 7.6 What is still unmeasured, and it is the biggest risk

**The obj-level grading is at probe scale only.** The fourteen functions of §7.2
were each checked against their own obj. At workload scale that check is not
available: the census population is the IL's function list (2,462,571) and the
obj emits only the subset actually needed, so `src/App.cpp` censuses 9,033
functions and emits 158 `.text` COMDATs. The ratios are corroborative and no more
— 5.7 %, 8.3 % and 13.4 % of the emitted functions in three workload TUs carry an
`__ehfuncinfo$`, against 11.6–12.5 % of their IL functions on the EH side, and
`GameMode.cpp` emits 124 `.pdata` COMDATs for 112 `.text` ones, which is the
second-`.pdata`-per-EH-function of §2 showing through. None of that is a
per-function grade.

**`eh-unknown` is 288,072 bodies — larger than the entire measured EH side is
over the cheap one.** These stop decoding before reaching any marker, so the axis
says nothing about them, in either direction. Two of the eight rows in §7.4 have
more than 19,000 functions in that column (`expr-op-0x27` 50,109;
`expr-intrinsic-this-adjust` 30,579; `expr-call-in-expr-op-0x9B` 39,348;
`expr-op-0x64` 24,402), and a body there could be on either side. The split in
§7.3 is therefore a split of the 310,371 that are *legible*, not of everything
that touches EH — and the population that would move it most is the one the
statement-layer decoder still cannot read. Establishing `0x64` (145,237 bodies
after this change) and `0x67` (45,631) — now the two largest rows on the
control-flow axis — is what would shrink it.

---

## 8. The obj structure from bytes — items 1–4, at the workload profile (2026-07-31, GT-EH)

§5 sizes the EH rung from two probes and says items 1–4 "are obj structure and
are mechanical once measured". This section measures them, on **20 probe objs carrying 21 EH functions, at
the workload's own flags** (`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc`), and
one of §5's own claims does not survive: **item 3 is not mechanical.** Its
contents are a per-function state assignment over the control-flow graph, and
that is the same problem as item 5.

Everything below is `scripts/gt_eh.py` output over `work/GTEH/probe/*.cpp`;
reproduction in §8.9. Nothing here is implemented and nothing here should be
implemented from this document alone.

### 8.0 First, the instrument lied — by shifting every mnemonic in the section

`scripts/gt_dump.py` disassembled a whole `.text` in one `llvm-mc` call and, on
a length mismatch, padded the shortfall with `?` **at the end**. llvm-mc emits
*nothing* for a word it cannot decode (the diagnostic goes to stderr), and an EH
function's `.text` opens with **two relocated zero words** — the
`__CxxFrameHandler` / `__ehfuncinfo$` prefix of §1. So every EH dump this
project has ever produced had its whole mnemonic column shifted up by two rows,
against correct byte and relocation columns. The first eh1 dump read
`0008 7d8802a6 li 0,0` — the bytes of `mflr r12` with the mnemonic of an
instruction two words later.

`disasm()` now establishes alignment per word (memoised, so the fallback costs
one process per *distinct* word) instead of inferring it from a count. §4's
listing of eh2 is unaffected — that body has no undecodable word after the
prefix, so the shift was constant and the excerpt happened to start below it —
but nothing else in this document was safe until the fix landed.

### 8.1 Item 1 — the prefix, and `Value = 8` (roadmap task #53)

**Reproduced at the workload profile**, and the profile changes nothing: eh1 and
eh2 recompiled at `/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc` are
**byte-identical** to §1–§3's `/O1 /GS- /c /EHsc` in section table, `.pdata`
words, `.rdata` contents and every symbol `Value`. §6.1's warning applies to
`shapes/ctor_dtor.rs`'s `/Ox`-with-no-`/EH` capture, not to `/GS-` vs `/GR /Oi`.

> **Every region the personality routine can be entered at gets its own 8-byte
> prefix, and the function symbol is `Value = 8`.** The prefix is two words,
> each `ADDR32`, `{__CxxFrameHandler, __ehfuncinfo$<mangled>}` — always the
> *same* two targets, including inside a funclet. 20 objs, 21 EH functions,
> `Value = 0x8` on every one.

A **catch** funclet carries its own prefix (`__catch$NNNN` is 8 bytes past two
more relocated zero words); an **unwind** funclet does not (`__unwind$NNNN` is
real code). That asymmetry is the whole of §8.2a.

### 8.2 Item 2 — the `.pdata` set

> **One `.pdata` COMDAT per covered region: the body plus one per funclet, all
> `Selection = 5` associative to the function's own `.text`, emitted in
> DESCENDING `.text` offset — funclets first, body last.**

§2 saw this as "funclet-first" with one funclet. With four it is an ordering,
and it is exact on **61 records over 21 EH functions** (max 5 in one function —
`pJ`, four catch funclets and the body; and `pR`, whose two *same-kind* unwind
funclets are also in descending order, which one funclet could not have shown).

`BeginAddress` always relocates `ADDR32` against the **function symbol**
(`Value = 8`), with the stored u32 as the addend, so

```
    addend       = <region's .text offset> - 8
    unwind word  = bit31 | bit30 | (len_words << 8) | prolog_words
    len_words    = the region's code only; the 8-byte prefix is NEVER counted
```

The `$T` label of a funclet's `.pdata` is higher than the body's — §2's reading,
still exact at four funclets, and it follows from the descending emission order
rather than being a separate fact.

Bit 30 (`ThirtyTwoBit`) is **1 on all 61 records** and never discriminated
anything here.

#### 8.2a Bit 31 — §2's reading, tested where it can fail

§2 offered *"bit 31 is set iff the covered region is preceded by the prefix"* as
"the reading that fits, not as established", on four records, and named the
capture that would test it: **a function with a catch *and* a destructor**. That
is probe `pA`, and it prints all three in one obj:

| record | `.text` | prefix | bit31 |
|---|---|---|---|
| `__unwind$2570` | 0xac | **no** | **0** |
| `__catch$2569` | 0x88 | yes | 1 |
| body | 0x08 | yes | 1 |

Both funclets, one function, opposite bits. The rival hypothesis "bit 31 is
clear on a funclet" is dead in one cell.

**Discriminating cells: 9** — the nine prefix-less unwind-funclet records
(`eh2`, `pA`, `pD`, `pG`, `pH`, `pL`'s `c3`, `pN`, and both of `pR`). The other
**52** records all have the prefix and bit 31 set; they confirm and cannot
refute. Range: 1–3
nested try blocks, 0–4 catch clauses, 0–2 destructible locals, frames 96–288 B,
`/O1 /EHsc` only. **Not swept: `/Ox`, `/O2`, packed (no `/Gy`), a function whose
frame needs `__savegprlr_N`, `_set_se_translator`/SEH.**

### 8.3 Item 3 — the EH `.rdata`, decoded

The section is `Selection = 5`, associative to the function's `.text` (so
"`.rdata` is always Selection 2" stays false, §3). Its layout is fixed and its
sub-objects are anchored by c2's own `STATIC` symbols, never by a guessed
offset:

```
    __unwindtable$F     maxState x UnwindMapEntry (8 B)
    __catchsym$F$n      one array per try block, nCatches x HandlerType (16 B)
    __tryblocktable$F   nTryBlocks x TryBlockMapEntry (20 B)
    __ehfuncinfo$F      FuncInfo, 36 B
    $TNNNN              nIPMapEntries x IpToStateEntry (8 B), 8-BYTE ALIGNED
```

**`FuncInfo` is NINE dwords, 36 bytes** — measured, and the x86 layout imported
from memory was **wrong**: there is no `dispUnwindHelp` field.

| off | field | measured range |
|---|---|---|
| +0x00 | `magic` | `0x19930522` on all 21 |
| +0x04 | `maxState` | 1..6 |
| +0x08 | `pUnwindMap` | `ADDR32 __unwindtable$F`, never null |
| +0x0c | `nTryBlocks` | 0..3 |
| +0x10 | `pTryBlockMap` | `ADDR32`, or **0** when `nTryBlocks == 0` |
| +0x14 | `nIPMapEntries` | 1..6 |
| +0x18 | `pIPtoStateMap` | `ADDR32 $TNNNN`, never null |
| +0x1c | `pESTypeList` | 0 on all 15 |
| +0x20 | `EHFlags` | **1** on all 21 (`/EHsc`) |

The ip-to-state map is 8-byte aligned after `FuncInfo`: pad 0 when the array
lands at an odd multiple of 4 and pad 4 otherwise, both observed (`eh1` pad 0 at
0x58, `eh2` pad 4 at 0x30). That is how the 9-dword size was *proved* rather
than fitted — a 10-dword `FuncInfo` predicts `$T` at 0x2c in `eh2`, and the
symbol says 0x30 while `eh1`'s says 0x58, which no single constant size can
satisfy without the alignment.

`UnwindMapEntry = { i32 toState; ADDR32 action }`; `action` is the
`__unwind$NNNN` funclet or **0** for a state whose exit runs no destructor.
`TryBlockMapEntry = { i32 tryLow, tryHigh, catchHigh, nCatches; ADDR32
pHandlerArray }`. `IpToStateEntry = { ADDR32 $MNNNN; i32 state }`, `state` from
−1 up.

**`HandlerType` is 16 bytes — and that survived its designed falsifier.**
`pQ` catches a class **by value** whose type has a user copy constructor and a
destructor; on several MSVC targets that adds a `copyFunction` field. It does
not here: the array is exactly `4 x 16` and `adjectives` is **0**, so the copy
is the funclet's job. Had the record been 20 bytes the four `pType`
relocations would not have landed on the four 16-byte boundaries they do.

`adjectives`, measured (`pE`, `pQ`), 6 cells:

| catch clause | adjectives | `pType` | `.data` |
|---|---|---|---|
| `int e`, `char c`, `short s`, `long l` | `0x00` | `??_R0H@8` etc | yes |
| `E e2` — class by value, user copy ctor + dtor | `0x00` | `??_R0?AUE@@@8` | yes |
| `const F2& e` | `0x09` | `??_R0?AUF2@@@8` | yes |
| `int& r` | `0x08` | `??_R0H@8` | yes |
| `const char* volatile p` | `0x01` | **`??_R0PAD@8`** | yes |
| `...` | `0x40` | **NULL, no relocation** | **no** |

Reading: `0x01` const, `0x08` reference, `0x40` ellipsis. **`0x02` (volatile) was
not isolated and is `NOT MEASURED`.** The `const char*` row is worth its own
line: the descriptor is `??_R0PAD@8`, *not* `??_R0PBD@8` — the pointee's
`const` moved out of the mangled type and into `adjectives`.

`dispCatchObj` is the **signed displacement of the catch object from the entry
SP** — see §8.5c, where it is the discriminating measurement.

### 8.4 Item 4 — the type-descriptor `.data`

```
    TypeDescriptor = { ADDR32 ??_7type_info@@6B@ ; u32 0 ; char name[] }
    RawSize = 8 + strlen(name) + 1,  NOT padded
```
Exact on 6 distinct descriptors (`.H` 11, `.D` 11, `.F` 11, `.J` 11, `.PAD` 13,
`.?AUE@@` 16, `.?AUF2@@` 17). `Selection = 2` (`ANY`), `Number = 0`, one
`ADDR32` to the single external `??_7type_info@@6B@`.

> **One `.data` COMDAT per distinct caught type per TU, not per function.** `pF`
> is two EH functions each catching `int`: one `??_R0H@8`, emitted inside the
> *first* function's group, and the second function's group has no `.data` at
> all. `catch(...)` emits none.

### 8.5 The compositions

#### 8.5a Section order — `CODEGEN_FRAMED_CALLS.md` §5's rule does not extend

§5 there establishes *"`.text`, then every `.rdata` it introduces, then its
`.pdata`"*. Naively extended to EH that is **false**, and the correction needs
both kinds of `.rdata` in one function to see. `pD` and `pP` are that probe —
an EH function that also pools an FP constant:

| # | `pP` | Sel | Number |
|---|---|---|---|
| 5 | `.text` | 1 | — |
| 6 | `.rdata` (4 B, the pooled `2.5f`) | **2** | 0 |
| 7 | `.pdata` (catch funclet) | 5 | 5 |
| 8 | `.pdata` (body) | 5 | 5 |
| 9 | `.rdata` (96 B, EH) | **5** | 5 |
| 10 | `.data` (`??_R0H@8`) | **2** | 0 |

> **Order inside one EH function's group: `.text`, then every pooled-constant
> `.rdata` (Sel 2), then every `.pdata` (descending `.text` offset), then the EH
> `.rdata` (Sel 5), then each newly introduced type-descriptor `.data`
> (Sel 2).** The `.pdata` and EH `.rdata` aux `Number` still name the
> function's own `.text`, counted through everything between — `Number = 5` here
> with a `.rdata` in the way, and `Number = 10` for the second function of `pF`.

**Discriminating cells: 2** (`pD`, `pP`). Every other probe has only one kind of
`.rdata` and is *inert* on this question — it prints a consistent order and
proves nothing. **Range: `/O1 /EHsc`, one pooled constant, 1–2 functions per TU.
Not swept: two or more pooled constants, `__real@` reuse across an EH and a
non-EH function, `/Ox`, packed.**

#### 8.5b The frame formula is UNCHANGED

`align16(80 + locals + 8 + 8*nSaved)` (roadmap §6g) holds on every EH body
measured, with `r31` counted as a saved register like any other:

| probe | nSaved | locals | predicted | actual |
|---|---|---|---|---|
| `pG` | 2 | 0 | 112 | 112 |
| `pH` | 2 | 160 | 272 | 272 |
| `eh1` | 1 | 4 | 112 | 112 |
| `pK` | 2 | 176 | 288 | 288 |

and **every funclet in every probe has `F = 96`** — the same formula at
`nSaved = 0, locals = 0`, `align16(88) = 96`, including `eh1`'s catch funclet,
which makes no call and saves nothing yet still allocates 96. What EH changes is
not the formula but its inputs: `r31` is always saved, and the funclet-visible
objects are always in `locals`.

#### 8.5c Every EH displacement is measured from the ENTRY SP

`r31 = entry_SP - F`, established *before* the `stwu`, and the funclet
re-derives it as `addi r31,r12,-F`. So **`r12` on funclet entry is the parent's
entry SP** — that is the establisher frame the personality routine passes, and
the funclet has to know the parent's `F` to use it.

Three quantities are pinned to that base, each by a probe that varied `F` and
held everything else:

| quantity | `F = 112` | `F = 272/288` | fixed at |
|---|---|---|---|
| the stack home of a funclet-visible value (`pG`→`pH`) | `stw r3,132(r31)` | `stw r3,292(r31)` | **entry SP + 20** |
| `dispCatchObj` (`eh1`→`pK`) | −28, obj at `84(r31)` | −204, obj at `84(r31)` | **entry SP − k** |
| the unwind-help word (`eh1`→`pK`) | `stw r0,4(r1)` | `stw r0,4(r1)` | **entry SP + 4** |

**Discriminating cells: 2 matched pairs.** Both are decisive: an `r31`-relative
model predicts 132 unchanged in the first row and −28 unchanged in the second,
and both moved by exactly the frame delta.

The unwind-help word (`li r0,0 ; stw r0,4(r1)`, in the prologue *before* the
register saves) is emitted **iff `nTryBlocks >= 1`**: present in `eh1`, `pK`,
`pA`, `pB`, `pC`, `pE`; absent in `eh2`, `pD`, `pG`, `pH`. This corrects §4,
which reads it as the function's own `SP+4`: it is at the **entry** SP + 4,
i.e. in the *caller's* reserved 8 bytes, which is what
`CODEGEN_FRAMED_CALLS.md` §1.1's "reserved and unwritten" pair is reserved for —
reserved by the caller, written by the callee.

#### 8.5d The label counter — the surcharge table, `LABEL_COUNTER.md` §1.1

`scripts/gt_label_stride.py` now carries EH probes. Every row's in-TU control
held (`base = 5` on all 13). **The instrument had to be fixed first**: a funclet
entry `__catch$NNNN` / `__unwind$NNNN` is a *defined STATIC symbol of function
type inside its parent's `.text`*, so the group walker opened a new group on it
and silently truncated the parent's label set — `extra` and `minted` were wrong
for every EH row while `stride` still looked sane.

| probe | what P is | extra | **stride** | minted |
|---|---|---:|---:|---:|
| `plain` / `gpr2` | non-EH controls | 0 | **5** | 5 |
| `eh-void-ctl` | void, framed, two callees, no destructible object | 0 | **5** | 5 |
| `eh-cheap` | `void P(){ SE s; }` — **eh-bare, NO EH records** | 1 | **6** | 5 |
| `eh-cheap-led` | same, with an eh-bare function already led | 1 | **6** | 5 |
| `eh-dtor` | one destructible local, no try | 4 | **18** | 17 |
| `eh-dtor-led` | same, an EH function already led | 4 | **17** | 16 |
| `eh-dtor2` | **two** destructible locals | 5 | **25** | 24 |
| `eh-catch` | one `catch(int)` | 8 | **22** | 22 |
| `eh-catch-led` | same, led | 8 | **21** | 18 |
| `eh-catch2` | **two** catch clauses | 13 | **31** | 30 |
| `eh-catchall` | `catch(...)` | 8 | **22** | 19 |
| `eh-both` | catch **and** destructor | 11 | **34** | 32 |

Four things are established out of that table, and the decomposition is not one
of them:

* **`__CxxFrameHandler` is a once-per-TU `+1`, exactly like `_fltused`** —
  `eh-dtor` 18 → `eh-dtor-led` 17 and `eh-catch` 22 → `eh-catch-led` 21.
* **The type-descriptor `.data` group is worth ZERO slots.** `eh-catch` (a
  `.data`, `??_R0H@8` and `??_7type_info@@6B@` in its group) and `eh-catchall`
  (none of them) are both **22**. One discriminating cell, and it separates the
  `.data` from the `+1` that the `-led` rows would otherwise have conflated with
  it.
* **The cheap side is not free: an `eh-bare` body costs `+1`.** Two
  discriminating cells — `eh-void-ctl` is the matched non-destructible control
  at the same frame class and prints 5, and `eh-cheap-led` shows the charge is
  **per function, not per TU**. §7.3's 40,881 "reachable without any EH model at
  all" functions each pay one label slot that a plain framed emitter would miss.
* **`stride == minted` is refuted on 9 of 12 EH rows** — `LABEL_COUNTER.md` §3's
  standing refutation, now with a ninth and largest family. `eh-catch` agrees at
  22 = 22 and that agreement is a coincidence: its neighbour `eh-catchall`
  agrees on stride and disagrees on minted.

The decomposition into per-feature surcharges is **NOT MODELLED**. It is not
additive: `eh-catch` 22 + `eh-dtor` 18 − 5 = 35, and `eh-both` is **34**.

#### 8.5e The estimate, scored

Written before each capture. Bias named where it was operating.

| predicted | actual |
|---|---|
| workload profile changes items 1–4 | **no change**, byte-identical to §1–§3 |
| catch + dtor ⇒ bit31 1 / 0 / 1 | exactly that |
| `catch(...)` ⇒ `pType` NULL, no `.data` | yes, **and** `adjectives = 0x40`, unpredicted |
| two catch clauses ⇒ **two** `__catchsym$` arrays | **WRONG** — one array of two entries. I assumed one array per handler |
| a class caught by value with a copy ctor ⇒ a **5th** `HandlerType` field | **WRONG, and this one is a win** — 16 bytes, `adjectives = 0` |
| `FuncInfo` = 10 dwords (x86's 9 + `dispUnwindHelp`) | **WRONG** — 9 dwords, no `dispUnwindHelp`. Bias: importing an x86 layout from memory, the exact trap §5 of `CLAUDE.md` names, and it failed |
| EH `.rdata` after the `.pdata` | right, **but not independent** — read off §3's own section numbering. The genuinely new half (the pooled-constant `.rdata` keeps its slot *before* the `.pdata`) was a coin-flip and landed |
| label surcharge ≈ **+4** for an `eh2`-shaped function | **+13**. Off by 3.25x. Bias: I counted minted symbols, which §3 of `LABEL_COUNTER.md` already records as refuted, and did it anyway |
| `eh-bare` costs **0** slots | **+1** |

Two of nine predictions were right for the right reason; three were wrong and
two of those three were wrong because a layout was imported instead of measured.

### 8.6 What stays NOT MODELLED

* **The IP-to-state map's contents.** `nIPMapEntries` took the values 1, 2, 3,
  4 and 6 over the probes with no rule I can state, and it does not track
  anything else in the record — `pI` has three catch clauses and **one** entry,
  `pR` has two destructible locals and **four**. Each entry relocates against a
  `$M` label at a statement boundary and carries a state that only a CFG-wide
  assignment produces. This is the single field of the whole EH `.rdata` I
  cannot predict from source.
* **`tryLow` / `tryHigh` / `catchHigh` / `maxState`.** Observed: `maxState` is
  **2 per nesting level of `try`** (1 try 2, two nested 4, three nested 6) plus
  **1 per destructible object** (`pR` two objects, no try, `maxState = 2`;
  `pN` one try plus one object, 3), with `tryLow`/`tryHigh` descending with
  depth. That is a fit to seven shapes, not a model, and the number of catch
  clauses does not enter it at all.
* **The `__catchsym$F$n` index.** A **name**, so a wrong one is wrong bytes.
  Fitted on **10 cells** by
  `n(array j) = maxState + sum_i(nCatches_i) - nTryBlocks + j`
  — equivalently, the catch clauses take numbers `maxState .. maxState+total-1`
  and the arrays take the **top `nTryBlocks`** of them, in order. Three designed
  falsifiers were run against it and all three passed (`pJ` two trys x two
  catches, which killed two earlier fits; `pM` three nested trys; `pO` two trys
  with **unequal** catch counts, 1 and 3). **Fitted, not established.**
* **`adjectives` bit `0x02`** (volatile), and every `adjectives` combination
  involving a class hierarchy.
* **The label-counter surcharge decomposition** (§8.5d), and every EH stride
  outside the twelve shapes in that table.
* **`/Ox`, `/O2` and packed mode for all of the above.** Every negative in §8 is
  scoped to `/O1 /EHsc`.
* **`pESTypeList`** — 0 on all 21 EH functions, so a `throw()` specification is
  entirely unmeasured.

### 8.7 Sizing item 5, and a correction to §5

§5 says items 1–4 are "mechanical once measured" and item 5 is "a codegen
problem the size of the whole framed-call rung". After measuring 1–4:

**Item 5 is not one rung, and the expensive half is not the funclet body.**

1. **Items 1–4 do not compose into a shippable rung on their own.** There is no
   EH obj without a funclet — the `.rdata` relocates against `__unwind$`/
   `__catch$` and the `.pdata` set counts them. Like W-UNW-1, this is groundwork
   with a census delta of exactly 0.
2. **The smallest rung that admits anything is the no-try unwind shape** —
   `eh2`/`pG`/`pH`: prefix + `Value = 8`, two `.pdata`, a 64-byte `.rdata` with
   `nTryBlocks = 0` and a two-entry ipmap, the `r31` discipline, entry-SP-relative
   homing, the `+13` label surcharge, and one funclet body. It is the whole of
   §7.3's largest bucket (`eh-plus-stmt`, 160,944) and it needs **no** try-block
   table, **no** handler array, **no** type descriptor and **no** `.data`.
3. **Try/catch is a separate, larger rung**: the try-block table, handler arrays
   with their `adjectives`, the type-descriptor `.data` and its two externals,
   the state numbering, the unwind-help word, the `$LN` continuation label the
   catch funclet returns in `r3`, and the `__catchsym$` name.
4. **The state model is the real cost, and it is item 3, not item 5.** The
   unwind map, the ip-to-state map and `tryLow`/`tryHigh`/`catchHigh` are one
   assignment of EH states to program points. That is whole-function dataflow;
   the port has no such pass, and no shape matcher produces it. The funclet
   *body* — `addi r31,r12,-F`, a 96-byte frame, calls, `blr` — is small and
   regular by comparison; every funclet measured here is 5 to 11 instructions.

So the honest ordering is: **groundwork (1–4 + one funclet emitter) → the no-try
unwind rung → the state model → try/catch.** §5's "obj-structure half could be
measured to completion cheaply" was right about the cost of *measuring* it and
wrong about what the measurement would contain.

### 8.8 The riskiest thing still unmeasured

**The IP-to-state map.** It is the only part of the EH `.rdata` whose value I
cannot derive from the source; it puts `ADDR32` relocations onto `.text` labels
the label planner does not currently allocate — so getting it wrong moves the
label counter as well as the `.rdata` — and no instrument in this project can
see it. It is also the field most likely to differ on the `eh-unknown` bodies
(§7.6, 288,072 of them), which is where the population that would move §7.3's
split lives.

### 8.9 Reproduction

`work/` is gitignored and objs are never committed, so the twenty probe sources
are **embedded in the script**:

```sh
export C2RS_WIBO=<the repo's resolved wibo>
scripts/gt_eh.py --write-probes work/GTEH/probe    # the whole §8 corpus
# every probe, decoded: prefix, .pdata set, FuncInfo, maps, section order.
# The DEFAULT mode is the WORKLOAD profile, not the fixture profile.
scripts/gt_eh.py work/GTEH/probe/pA.cpp            # bit 31: both funclets, one obj
scripts/gt_eh.py work/GTEH/probe/pP.cpp            # section order: both .rdata kinds
scripts/gt_eh.py work/GTEH/probe/pQ.cpp            # adjectives + the HandlerType width
scripts/gt_eh.py work/GTEH/probe/pH.cpp --text     # the entry-SP base, F = 272
scripts/gt_eh.py work/GTEH/probe/pK.cpp --text     # dispCatchObj at F = 288
scripts/gt_eh.py work/GTEH/probe/pR.cpp            # TWO unwind funclets, one function
scripts/gt_eh.py work/GTEH/probe/pL.cpp            # the eh-bare / eh-plus-stmt boundary
scripts/gt_eh.py --obj <any.obj>                   # decode without recompiling
# the label surcharge -- the /EHsc mode is NOT optional: without it every EH
# row collapses onto its non-EH control, which is a VACUOUS run, not a zero.
scripts/gt_label_stride.py --mode '/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc' \
    plain gpr2 eh-void-ctl eh-cheap eh-cheap-led eh-dtor eh-dtor-led eh-dtor2 \
    eh-catch eh-catch-led eh-catch2 eh-both eh-catchall
```

`gt_eh.py` reads every sub-object boundary from c2's own `STATIC` symbols and
every array length from the count field that points at it — the handler-array
walk originally ran off the end of one array into the next (`pC`), which is why
lengths are no longer inferred from where the relocations stop.

**Re-gate.** All 20 embedded probes were recompiled into a second directory and
compared against the originals on everything but `.drectve` / `.debug$S` (which
carry the source path): **20 structurally identical, 0 mismatched** — section
names, sizes, characteristics and raw bytes, and every symbol's name, value,
section and storage class. And `gt_label_stride.py`'s shipped §1 table is
unmoved by this session's two edits to it: `plain` 5, `plain-3callees` 5,
`gpr3` 7, `fpr4-led` 7, `both-led` 9, `const1-led` 7, `const2-led` 9,
`leaf-int` 1, `leaf-float` 2, controls failed 0, `stride != minted` 0.
