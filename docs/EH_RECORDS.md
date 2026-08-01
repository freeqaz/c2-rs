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

**CORRECTED by §9.4.** That rule agrees with the transfer rule on all five rows
here and on all fourteen of §7.2, and it is still false: the predicate is not the
statement count but *whether an outbound control transfer occurs while a
destructible object is live*. `int P(int a){ SE s; return a+1; }` has "another
statement beside it" and gets **none** of §1–§5. The census axis built on the
statement count therefore over-counts `eh-plus-stmt` by an unmeasured amount.

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
| +0x1c | `pESTypeList` | 0 on all 21 |
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

*(The first two bullets are **superseded for the no-try unwind shape** by §9,
which derives both from source and holds `nIPMapEntries` exact on 27 EH
functions. They stand as written for try/catch, where §9.7 refutes the rule.)*

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

*(**CLOSED for the no-try unwind shape by §9**, in the favourable direction: the
map is derivable from source, and the label consequence named below is measured
in §9.8 — `eh-dtor` costs 18 label slots where `LABEL_COUNTER.md` §1.1 alone
predicts 5. It stands for try/catch, where §9.7 refutes the rule and the labels
land on inserted `or r8,r8,r8` marker nops.)*

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

---

## 9. The state model — the ip-to-state and unwind maps, from bytes (2026-07-31, GT-IP2STATE)

§8.6 lists *"the IP-to-state map's contents"* as the one field of the whole EH
`.rdata` it cannot derive from source, and §8.8 names it the riskiest unmeasured
thing in the document: `nIPMapEntries` took 1, 2, 3, 4 and 6 and *"tracks nothing
else"* (three catch clauses gave **one** entry; two destructible locals gave
**four**). §8.7 then concludes that this — the state model, item 3 — is the real
cost of the EH phase, not the funclet bodies.

This section measures it for the **no-try unwind shape**, which §8.7 identifies
as the smallest rung that admits anything and which is §7.3's largest bucket
(`eh-plus-stmt`, 160,944). **31 probe sources, 27 of them EH functions**, at the
workload's own flags (`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc`).
Reproduction in §9.9. Nothing here is implemented and nothing here should be
implemented from this document alone.

The result is that **for the no-try shape the whole EH `.rdata` is now
determined**, including the field §8 could not predict — and the same rule is
**REFUTED for try/catch** (§9.7), which is a boundary worth as much as the rule.

### 9.1 The rule

> **Walk the function's outbound control transfers in ascending `.text` order.
> Each has an EH state. Emit one `IpToStateEntry` every time that state differs
> from the previous entry's, with an implicit −1 before the first, and put the
> entry's `$M` label ON that instruction.**
>
> An *outbound control transfer* is `bl`, `bctrl`, or an unconditional `b`
> carrying a relocation — all three measured, §9.5.

The **state** at a transfer is an index into the list of *distinct sets of live
destructible objects* observed at such a transfer, numbered in order of first
occurrence; −1 is the empty set. An object is live from the instruction after
its constructor call returns until its own destructor call begins — the
destructor call itself sits at the state *below* the object it destroys.

Everything else about the section follows. Writing `S` for the number of states
and `E` for `nIPMapEntries`:

```
    maxState            = S
    nIPMapEntries       = E
    __unwindtable$F     = S x { i32 toState ; ADDR32 __unwind$NNNN }
    __ehfuncinfo$F      @ 8*S          (nTryBlocks = 0, pTryBlockMap = 0)
    $TNNNN              @ align8(8*S + 36)
    EH .rdata RawSize   = 8*S + 36 + pad + 8*E        <- 27/27, exact
    EH .rdata nrelocs   = S + 2 + E                   <- 27/27, exact
    S funclets, emitted after the body in ASCENDING state order
    S+1 .pdata records, emitted in DESCENDING .text offset (§8.2)
```

`toState` is the state of the live-set with this state's own object removed, and
`action` is the funclet that destroys it.

### 9.2 The count, on held-out cells

§8's only two no-try cells were `eh2` (one object, 2 entries) and `pR` (two
objects, 4 entries). A ladder of `int P(int a){ SE s; …; return gp(a)+s.m+…; }`
was **predicted at `2n` before capture** and n = 3, 4 are held out of the fit:

| probe | n | maxState | funclets | `nIPMapEntries` | the states, in order |
|---|---:|---:|---:|---:|---|
| `qN1` | 1 | 1 | 1 | **2** | 0, −1 |
| `qN2` | 2 | 2 | 2 | **4** | 0, 1, 0, −1 |
| `qN3` | 3 | 3 | 3 | **6** | 0, 1, 2, 1, 0, −1 |
| `qN4` | 4 | 4 | 4 | **8** | 0, 1, 2, 3, 2, 1, 0, −1 |

Every one of the 20 labels is on a `bl`: the ctors that raise the state, the
body call at the top state, and the dtors that lower it.

### 9.3 The state is per OBSERVED live-set, not per object

`qB3` is the cell that separates them, and it moves five fields at once:

```cpp
int P(int a){ SE s; SE t; return a+1; }      // two objects, no other call
```

**`maxState = 1`, one unwind funclet, two ipmap entries** — not 2/2/4. The four
transfers are ctor `s` (−1), ctor `t` (live `{s}`), dtor `t` (live `{s}`), dtor
`s` (−1). The set `{s,t}` is *never observed at a transfer*, so it gets no state,
no funclet and no unwind-map row. The funclet destroys `s` only.

`qSC2` — `{ SE s; a=gp(a)+s.m; } { SE t; a=gp(a)+t.m; }` — is the other half:
two **disjoint** scopes, both with one live object, and c2 allocates **two**
states with `toState = −1` on **both** and two distinct funclets that destroy
**the same stack slot**. So states are not reused across scopes and identical
funclets are not merged.

> **`toState = i − 1` is REFUTED.** It is right only for nested lifetimes.
> `toState` is the state of the live-set with the top object removed.

**Discriminating cells: `qB3` (1, decisive against per-object states) and
`qSC2` (1, decisive against `toState = i − 1`).** The six nested probes
(`qN2`–`qN4`, `qC2`, `qC3`, `qORD`) print `i − 1` and are the confirming half of
the same discrimination; on their own they are inert.

`qRE` (`if(a){SE s;…} a=gp(a); if(a){SE t;…}`) shows the map is a list of
address *ranges*, not of states: state −1 occurs twice, at entries 1 and 3.

### 9.4 What it retires — the cheap/EH boundary is NOT a statement count

§6 states the boundary as *"Exactly one sub-object statement and nothing else is
a bare branch. A second sub-object, or any other statement beside it, is the
whole of §1–§5"*, and §7.2/§7.3 build the census axis on that. **Measured, it is
false in the direction that inflates the EH side:**

| probe | source | statements | `__ehfuncinfo$` | symbol `Value` |
|---|---|---:|---|---:|
| `qNC` | `int P(int a){ SE s; return a+1; }` | 2 | **no** | 0 |
| `qB1` | `int P(int a){ SE s; int x=a*3; int y=x^7; return y+1; }` | 4 | **no** | 0 |
| `qB2` | `int P(int a){ SE s; return gp(a); }` | 2 | yes | 8 |
| `qB4` | `void P(){ SE s; }` | 1 | no | 0 |

`qNC` and `qB1` have "another statement beside" the object and are the whole of
the cheap side — no prefix, one `.pdata`, no `.rdata`, no funclet, no `r31`
discipline, locals addressed off `r1`. The predicate is not the statement count:

> **An EH record set exists iff `maxState >= 1`, i.e. iff at least one outbound
> control transfer occurs while a destructible object is live.** `S = 0` and the
> entire §1–§5 apparatus disappears with it.

That also explains every row of §7.2 without the statement rule: `~Two(){}` is EH
because member 1's *destructor call* is a transfer while member 2 is live;
`Ct2::Ct2(){Init();}` because `Init()` is a transfer while the base is live;
`Ct1::Ct1(){}` is bare because a constructor's normal path ends with nothing
live. **The statement count and the transfer rule agree on all fourteen of §7.2's
functions**, which is why the axis passed its own grading — `qNC` and `qB1` are
the shapes that were not in it.

**Consequence for §7.3, direction known, magnitude NOT MEASURED.** The
`eh-bare`/`eh-plus-stmt` split is keyed on statement counts, so it **over-counts
`eh-plus-stmt`** by however many bodies have a second statement that emits no
call. Re-scanning the workload is a harness change and was out of this lane's
scope; the 160,944 should be read as an upper bound on the no-try rung's stock
until it is re-graded on "a transfer while an object is live".

**Discriminating cells: 2** (`qNC`, `qB1`). `qB4` is §7.2's own shape and is
*inert* — both rules call it bare. `qC0` (Class C, no destructible object) is
inert for the same reason.

### 9.5 The tail-branch entry, which nothing would have predicted

A Class C function's epilogue ends `b __restgprlr_N` — a branch out of the
function. **c2 treats it as an outbound transfer and gives it an ip2state entry
with state 0**, after the last destructor has already returned the state to −1.
So `E = 2n + 1` for those functions, and the extra entry is always last, always
state 0, at every `n` measured.

| probe | epilogue | extra entry | `E` |
|---|---|---|---:|
| `qN1`–`qN4`, `qB3`, `qGAP`, `qIF`, `qREV`, `qRE`, `qSW`, `qG1`, `qG2` | `blr` | no | 2n |
| `qE1` | `bl __restfpr_28` then `blr` | **no** | 2 |
| `qDUP`, `qBB`, `qC1`–`qC3`, `qMID`, `qORD`, `qIND`, `qLOOP`, `qG3`, `qG4` | `b __restgprlr_N` | **yes, state 0** | 2n+1 |
| `qF1` | `bl __restfpr_28` then `b __restgprlr_27` | **yes, state 0** | 2n+1 |

**`qE1` and `qF1` are the matched pair that isolates it.** Both use an FPR helper
pair in the epilogue; `qE1`'s is a `bl` and gets no entry, `qF1`'s is followed by
a tail `b` and gets one. So the trigger is the **tail branch**, not "a helper in
the epilogue" and not the frame class. **Discriminating cells: 2** (that pair);
11 further positives and 12 negatives confirm and cannot refute.

Why the state is 0 rather than −1 is **NOT MODELLED**. It is 0 at `S = 1, 2, 3`,
so it is not `maxState − 1` and not "the last state".

`qIND` establishes that **`bctrl` is an outbound transfer**: a virtual call
carries entry 0. **1 discriminating cell** — every other probe's transfers are
`bl`/`b`, and is inert on it.

### 9.6 Placement, and the two-call dedup

`qGAP` puts four non-call instructions between the constructor's return and the
next call. The rival rule "the label marks the point where the state changes"
predicts `.text+0x2c`; **measured `.text+0x3c`, on the `bl`.** One cell,
decisive, and it is the only cell in the corpus with a gap wide enough to
separate them — `qN1`'s gap is one instruction.

Two or more transfers at the same state produce **one** entry: `qDUP` (2 calls),
`qC1` (3), `qE1` (4), `qF1` (8). **4 discriminating cells** against "one entry
per call", which would have printed 4, 5, 5 and 12 entries respectively.

### 9.7 REFUTED for try/catch — the rule is no-try only

Re-read with the same instrument, §8's try-carrying probes put **four of six**
labels on instructions that are not transfers at all:

```
  pA  [0] $M2579  state 1   .text+0x38  bl ?g            <- a call
      [1] $M2580  state 0   .text+0x3c  7d084378  or r8,r8,r8    <- a NOP
      [2] $M2581  state 1   .text+0x40  817f0050  lwz r11,80(r31)
      [3] $M2582  state -1  .text+0x4c  bl ??1S           <- a call
      [4] $M2583  state 0   .text+0x50  7d084378  or r8,r8,r8    <- a NOP
      [5] $M2584  state -1  .text+0x54  7fc3f378  mr  r3,r30
```

`pN` prints the same shape. **`or r8,r8,r8` (`0x7d084378`) occurs in 4 of 4
try-carrying probes (`eh1`, `pA`, `pI`, `pN`, 10 occurrences) and in 0 of the 27
no-try EH functions** — it is a pure marker instruction c2 emits to carry a
state boundary that does not fall on an instruction it can otherwise label. Not
every occurrence carries an ip2state label (`pI` has three of them and one
entry), so what it marks is **NOT MODELLED**.

Two further try-side facts fall out and are consistent with §9.1's state model
generalised: a `try` contributes states whose `action` is **0** (`eh1`, `pI`:
`maxState = 2`, both unwind rows null, `tryLow = tryHigh = 0`, `catchHigh = 1`),
and where a destructible object and a `try` coexist the object takes the lower
state (`pA`, `pN`: `maxState = 3`, state 0 = the object with a real funclet,
states 1–2 the try, `tryLow = tryHigh = 1`, `catchHigh = 2`). That explains
§8.6's *"2 per nesting level of `try` plus 1 per destructible object"* — but the
**ip placement** is a different mechanism and the try rung needs it measured
separately.

> **Range of §9.1: no-try only.** Every negative and every count law in §9 is
> scoped to `nTryBlocks = 0`, `/O1 /EHsc`, one EH function per TU (except `pF`),
> `S` 1–4, `E` 2–8, frame classes A/B/C/E/F, frames 96–288 B. **Not swept:
> `/Ox`, `/O2`, packed (no `/Gy`), a `try` of any kind, `_set_se_translator`,
> SEH, an object whose destructor is inlined away, arrays of destructible
> objects, temporaries, and any state region laid out non-contiguously in
> `.text` (§9.8).**

### 9.8 The label counter — the surcharge, decomposed

This is the consequence §8.8 flags: those `ADDR32` relocations point at `.text`
labels the planner does not allocate, so a wrong entry count is a wrong label
counter, and a wrong label number is six wrong bytes in an obj that still links.

Measured seed-free and in-TU by `scripts/gt_label_stride.py`, `/EHsc` mode,
**every row's in-TU control held (`base = 5` on all 19)**:

| probe | S | E | Σ(§1.1) | stride | model |
|---|---:|---:|---:|---:|---:|
| `eh-dtor` | 1 | 2 | 0 | **18** | 18 |
| `eh-dtor2` | 2 | 4 | 0 | **25** | 25 |
| `eh-dtor3` | 3 | 6 | 0 | **32** | 32 |
| `eh-dtor4` | 4 | 8 | 0 | **39** | 39 |
| `eh-dtor-fp` | 1 | 2 | 1 (`_fltused`) | **19** | 19 |
| `eh-dtor-const` | 1 | 2 | 3 (`_fltused` + 1 pooled) | **21** | 21 |
| `eh-dtor-dup` | 1 | 3 | 2 (gprlr) | **21** | 21 |
| `eh-dtor-cmpeq` | 1 | 3 | 2 (gprlr) | **21** | 21 |
| `eh-dtor-cmprr` | 1 | 3 | 4 (gprlr + signed cmp rr) | **23** | 23 |
| `eh-1state-2obj` | 1 | 2 | 0 | **19** | 18 ✗ |
| `eh-dtor-scope2` | 2 | 4 | 0 | **24** | 25 ✗ |
| `eh-dtor-loop` | 1 | 3 | 2 (gprlr) | **22** | 21 ✗ |

> **stride = 11 + 5·S + E + Σ(`LABEL_COUNTER.md` §1.1 surcharges)** — 9 of 12,
> **fitted, not established**, with the three misses named above and unexplained.
> The `11` carries the once-per-TU `__CxxFrameHandler` `+1` of §8.5d: with an EH
> function already led it is `10` (`eh-dtor-led` 17 = 10 + 5 + 2 ✓).

Three things generalize past the formula.

* **The ordinary §1.1 surcharge table survives EH unchanged, all four kinds
  measured exactly.** `_fltused` +1, a newly pooled FP constant +2, a helper
  pair +2, and a signed `<`/`>` over two call results +2 (`eh-dtor-cmprr` against
  its matched `eh-dtor-cmpeq` control, +2 exactly). EH terms *add* to §1.1; they
  do not replace it.
* **The allocation order inside one EH function is exactly determined** — read
  off the number spans of 27 objs, and this is what an emitter needs:

  ```
    1.  S x  __unwind$N          one per state, ASCENDING state
    2.  G    reserved, unused    G = 4 + Σ(the §1.1 surcharges that MINT a symbol)
    3.  E x  $M                  the ip2state labels, ASCENDING .text offset
    4.  1 x  $T                  the ip2state array, in the EH .rdata
    5.  1    reserved, unused
    6.  per region, ASCENDING .text (body, then funclet 0..S-1):
             $M end-of-prologue,  $M end-of-region,  $T its own .pdata
  ```

  `G` is the cell that keeps this from being closed. `G = 4 + Σmint` is exact on
  25 of 27 (`qN*` 4, `qC*`/`qDUP`/`qIND`/`qMID`/`qORD`/`qBB` 6 = 4+2, `qG2` 5 =
  4+1, `qG1` 7 = 4+1+2, `qE1` 7, `qF1` 9) and misses on `qB3` (5, expected 4)
  and `qLOOP` (8, expected 6) — the same two shapes that miss in the stride
  table. The base `4`, and those two cells, are **NOT MODELLED**.
* **`stride == minted` is refuted on every EH row here**, extending §8.5d's
  ninth family: 12 more rows, and the largest stride in this document is now
  **39** (`eh-dtor4`), above §8.5d's measured 6–34 band.

**What a planner that ignores all of this gets wrong, in numbers.** §1.1 alone
predicts `eh-dtor` at **5**; the truth is **18**. Every subsequent function in
that TU would be **13** label numbers low — and 34 low after an `eh-dtor4`.
Six wrong bytes per `$M`/`$T` reference, in an obj that links.

### 9.9 The estimate, scored

Written into `work/GTIP/ESTIMATE.md` before any capture beyond one baseline dump
of `eh2`.

| predicted | actual |
|---|---|
| the rule: walk transfers, dedup consecutive equal states, implicit −1, label ON the transfer | **right**, and on the held-out cells |
| `nIPMapEntries = 2n` for n straight-line locals | **right** at n = 3 and n = 4, held out of the fit |
| two calls at one state ⇒ one entry | **right**, 4 cells |
| a body with no call while the object is live ⇒ **1** entry (a floor) | **WRONG, and this one is the finding** — **zero**, and the whole EH record set disappears with it. I predicted a floor because §8 reports a minimum of 1 and `pIPtoStateMap` never null; both are true and neither implies a floor |
| the `$M` sits on the `bl`, not at the state change | **right** (`qGAP`, 4 instructions of separation) |
| `toState = i − 1` | **WRONG** — right for nested lifetimes, and `qSC2` prints −1 for both of two disjoint scopes. Bias: I fitted it on the straight-line ladder, which is exactly the shape that cannot separate the two readings |
| each ip2state entry costs +1 label slot; `eh-dtor`→`eh-dtor2` decomposes 2 of its +7 that way | **right on both**: +6 per state = funclet + its `.pdata` `$T` + 2 region `$M` + **2 ipmap `$M`**, and the residual +1 is the extra object |
| — (unpredicted) | the Class-C tail `b __restgprlr_N` gets its own entry at state 0 |
| — (unpredicted) | the rule does not extend to try/catch, and c2 emits `or r8,r8,r8` as a bare label anchor there |

**Named bias, stated in advance and it mattered.** The rule was imported from
general MSVC x64 EH knowledge — from memory, the trap §8.5e records twice. It
was written down with designed falsifiers (`qNC`, and n = 3, 4 held out) *before*
capture precisely so a confirmation could not be claimed from the two cells that
generated it. Two of the three things I got wrong were wrong in the same
direction: I assumed structure that the straight-line ladder could not
discriminate.

### 9.10 What stays NOT MODELLED

* **The try/catch ip placement** (§9.7) — the largest open piece, and it is the
  try rung's, not the no-try rung's.
* **Why the tail-branch entry's state is 0** (§9.5).
* **`G`'s base of 4, and the `qB3`/`qLOOP` cells** (§9.8).
* **A state region laid out non-contiguously in `.text`.** The map is a sorted
  range list; on all 27 EH functions here — including `goto` (`qREV`), a
  `switch` (`qSW`), a loop (`qLOOP`), an `if/else` (`qIF`) and a re-entered
  state (`qRE`) — c2 laid every state's range out contiguously and in an order
  the sorted map can express. **I could not construct a counterexample, which is
  not the same as there being none**, and a non-contiguous state would need
  entries this rule does not produce. This is the riskiest thing left for the
  no-try rung.
* **Arrays of destructible objects, temporaries with destructors, an inlined
  destructor, and a destructible object passed by value** — none probed, and
  each could add states without a visible constructor call.
* **`/Ox`, `/O2`, packed.** Every number in §9 is `/O1 /EHsc`.

### 9.11 Reproduction

```sh
export C2RS_WIBO=<the repo's resolved wibo>
scripts/gt_eh.py --write-probes work/GTIP/probe    # the §8 AND §9 corpora
scripts/gt_eh.py work/GTIP/probe/qN3.cpp           # the count law, held-out n
scripts/gt_eh.py work/GTIP/probe/qB3.cpp           # states are per LIVE-SET
scripts/gt_eh.py work/GTIP/probe/qSC2.cpp          # toState = -1 for BOTH
scripts/gt_eh.py work/GTIP/probe/qNC.cpp           # maxState 0 => NO EH records
scripts/gt_eh.py work/GTIP/probe/qGAP.cpp          # the label is on the `bl`
scripts/gt_eh.py work/GTIP/probe/qE1.cpp           # no tail `b` => no extra entry
scripts/gt_eh.py work/GTIP/probe/qF1.cpp           # ...and its matched pair
scripts/gt_eh.py work/GTIP/probe/pA.cpp            # REFUTED for try/catch
# every run prints `-- ip2state against the call sites`: one row per outbound
# transfer with the state the map assigns it, and `!!` for any map entry whose
# label is NOT on a transfer. That line is the whole discrimination.
scripts/gt_label_stride.py --mode '/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc' \
    eh-dtor eh-dtor2 eh-dtor3 eh-dtor4 eh-nostate eh-nostate3 eh-1state-2obj \
    eh-dtor-dup eh-dtor-scope2 eh-dtor-const eh-dtor-fp eh-dtor-cmprr \
    eh-dtor-cmpeq eh-dtor-loop
```

**Re-gate.** All 51 embedded probes were written out fresh and recompiled into a
second directory; the 36 with a prior capture in this session compare
**structurally identical, 0 mismatched** on section names, sizes,
characteristics and raw bytes and on every symbol's name, value, section and
storage class, with `$M`/`$T`/`__unwind$` numbers normalised to each group's own
base (six §9 sources carry an unused declaration in the working copy, which
moves the seed and nothing else — the seed-free method of `LABEL_COUNTER.md` §0
exists for exactly this). §8's own numbers are unmoved by the instrument edits:
**21 EH functions, 21 `0x19930522` magics, 21 nine-dword `FuncInfo`s, 61 `.pdata`
records over EH functions of which 9 are prefix-less.** `gt_label_stride.py`'s
shipped §1 table is unmoved (`plain` 5, `gpr3` 7, `fpr4-led` 7, `both-led` 9,
`const1-led` 7, `const2-led` 9, `leaf-int` 1, `leaf-float` 2) and so is §8.5d's
(`eh-cheap` 6, `eh-cheap-led` 6, `eh-dtor` 18, `eh-dtor-led` 17, `eh-dtor2` 25,
`eh-catch` 22, `eh-catch-led` 21, `eh-catch2` 31, `eh-both` 34, `eh-catchall`
22), controls failed 0.

**Re-gate.** All 20 embedded probes were recompiled into a second directory and
compared against the originals on everything but `.drectve` / `.debug$S` (which
carry the source path): **20 structurally identical, 0 mismatched** — section
names, sizes, characteristics and raw bytes, and every symbol's name, value,
section and storage class. And `gt_label_stride.py`'s shipped §1 table is
unmoved by this session's two edits to it: `plain` 5, `plain-3callees` 5,
`gpr3` 7, `fpr4-led` 7, `both-led` 9, `const1-led` 7, `const2-led` 9,
`leaf-int` 1, `leaf-float` 2, controls failed 0, `stride != minted` 0.

---

## 10. The axis re-derived on `maxState` — the split, measured (2026-07-31, EHMS)

§9.4 refuted the predicate §6/§7 were built on and left the magnitude open:
*"direction known, magnitude NOT MEASURED … re-scanning the workload is a
harness change and was out of this lane's scope."* This is that re-scan, plus
the probes that establish the IL-level predicate, plus the repair of a
cross-tab defect that had a **control group** printing as the largest blocker
row on the board.

Census delta **0** — decode-only, an axis, not a rung. Nothing acceptance reads
is touched.

### 10.1 The predicate, at IL level

The obj-level rule is §9.4's:

> An EH record set exists iff **`maxState >= 1`**, i.e. iff at least one outbound
> control transfer occurs while a destructible object is live.

Rendered into the statement-layer scanner, three lines:

* `5C` — an object goes live. **Raise** a running live count. (`5C` is the last
  token of its statement, so a constructor call in the same statement is before
  it and is correctly counted at the lower state, which is where c2 puts it.)
* `5D` / `5E` `<n> <state>` — `n` objects stop being live. **Lower** it by `n`,
  saturating at zero.
* `4C` — a call's argument list closes. If the live count is **non-zero**, that
  is a transfer at a non-empty live set: `maxState >= 1`, and the whole of §1–§5
  exists.

**The counting site is `4C`, not `BD`, and one probe settles it.** `BD` is the
call *descriptor* and it is emitted **before** the arguments are evaluated, so a
destructible temporary materialized by a nested call goes live *between* the
two. `int t1(int a){ return gp(mkSE().m) + a; }` emits

```
BD(gp) … BD(mkSE) 4C … 5C … 4C(gp)
```

Counting at `BD` puts gp's transfer at the empty live set and calls the body
cheap; its obj has an `__ehfuncinfo$?t1@@YAHH@Z`, `maxState = 1`, symbol
`Value = 8`. A **destructible temporary** is one of the four shapes §9.10 lists
as never probed, and it was the cell that broke the first reading.

### 10.2 Graded against the obj — 46 functions, 46 right

Three probe sources, `work/EHMS/probe/m1.cpp`, `m2.cpp`, `m3.cpp`, at the
workload's own flags (`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc`). Every
function's obj was read for an `__ehfuncinfo$` and its symbol `Value`; that
reading is the expected column, never a prediction.

**The maxState rule: 46/46. The statement rule: 35/46, wrong in BOTH
directions.** The eleven it misses:

| probe | source | obj | statement rule | maxState rule |
|---|---|---|---|---|
| `mA` | `int P(int a){ SE s; int x=a*3; int y=x^7; return y+1; }` | **no record** | `eh-plus-stmt` ✗ | `eh-state0` ✓ |
| `mI` | `int P(int a){ int x=gp(a); SE s; return x+1; }` | **no record** | `eh-plus-stmt` ✗ | `eh-state0` ✓ |
| `mJ` | `int P(int a){ SE s; for(…) a+=i; return a; }` | **no record** | `eh-plus-stmt` ✗ | `eh-state0` ✓ |
| `mF` | `int P(int a){ { SE s; } { SE t; } return a+1; }` | **no record** | `eh-multi` ✗ | `eh-state0` ✓ |
| `n2` | `void P(){ { SE s; } gv(); }` | **no record** | `eh-plus-stmt` ✗ | `eh-state0` ✓ |
| `n5` | `int P(int a){ {SE s;}{SE t;}{SE u;} return a+1; }` | **no record** | `eh-multi` ✗ | `eh-state0` ✓ |
| `C2` | `struct C2{ int k; Mem a; }; C2::C2(int x):k(gp(x)){}` | **no record** | `eh-plus-stmt` ✗ | `eh-state0` ✓ |
| `l2` | same shape as `mJ`, in a second TU (a repeat control) | **no record** | `eh-plus-stmt` ✗ | `eh-state0` ✓ |
| `mC` | `int P(int a){ SE s; return gp(a); }` | **RECORD** | `eh-bare` ✗ | `eh-state1` ✓ |
| `mK` | `int P(int a){ NC s; return gp(a); }` | **RECORD** | `eh-bare` ✗ | `eh-state1` ✓ |
| `t1` | `int P(int a){ return gp(mkSE().m) + a; }` | **RECORD** | `eh-bare` ✗ | `eh-state1` ✓ |

Read the two halves. **Eight false-EXPENSIVE**: a body with statements beside
the object, none of which calls anything, gets no record at all — §9.4's `qNC`
and `qB1` generalize, and the scope cases (`mF`, `n2`, `n5`) generalize further
because two objects whose lifetimes do not overlap never produce a state.
**Three false-CHEAP**: a `return` carries no `4B`
(`docs/IL_STMT_GRAMMAR.md` §9), so `SE s; return gp(a);` has *zero* other
statements and reads bare — while carrying the whole record set. **The cheap
side was never a lower bound.** It was a bound in neither direction.

Three cells are decisive and named as such:

* **`C1` against `C2`** — the same source, the same two members, the *declaration
  order swapped*. Members are constructed in declaration order, so `gp(x)` in the
  initializer list is a transfer at `{a}` in one and at `{}` in the other. `C1`
  gets `Value = 8` and `C2` gets `Value = 0`. Nothing but the transfer rule
  separates them; every statement-shaped predicate calls them identical.
* **`mF` / `n5`** — two and three destructible objects, no record. Kills "a second
  object is the EH record" (§6's boundary as written) outright.
* **`mH`** — `NC s; NC t;` where `NC` has a destructor and **no constructor**, so
  there is no constructor call to raise anything. `maxState = 1` anyway, and the
  ip2state entry sits on `bl ??1NC` — the *destructor* of `t`, at live `{s}`.
  The raising transfer need not be a constructor.

The five §7.2 cells are re-graded too and all five agree with both rules — which
is why §7.2 passed its own grading. **Both predicates are right on every cell
§7.2 contained.** The disagreement is entirely in shapes it did not have.

### 10.3 The split, measured

Scan `work/dc3-workload/scan-ehms.jsonl`, 878 TUs, corpus HEAD `f6074c8b`.
Census **691,744 / 2,462,571 = 28.09 %**, mismatch **0**, census/gate
disagreement **0** — every one identical to the baseline. The marker-carrying
population is **354,646** before and after, to the function: this is a
repartition of the same bodies, not a re-decode.

| EH class | ALL | BLOCKED |
|---|---:|---:|
| `eh-none` | 2,044,067 | 1,388,294 |
| **`eh-state1`** (`maxState >= 1`, the whole record set) | **237,180** | **237,180** |
| **`eh-state0`** (`maxState = 0`, the cheap side) | **117,463** | **81,492** |
| `eh-unknown` (stopped before any marker) | 63,858 | 63,858 |
| `eh-partial` (marker, no proof either way, then stopped) | **3** | 3 |

**Against the two prior bounds, both of which were loose:**

| | statement rule (§7.3, this scan) | measured | delta |
|---|---:|---:|---:|
| EH side (§6q's "EH stock") | 276,810 — **UPPER** bound | **237,180** | **−39,630, −14.3 %** |
| cheap side, all | 77,836 — called a *lower* bound | **117,463** | **+39,627, +50.9 %** |
| cheap side, blocked | 41,865 | **81,492** | **+39,627, +94.7 %** |
| undecided | 4,375 | **3** | −4,372 |

The migration cross, which is the whole reconciliation:

| | → `eh-state1` | → `eh-state0` | → `eh-partial` |
|---|---:|---:|---:|
| `eh-plus-stmt` 225,330 | 158,994 | **66,336** | — |
| `eh-multi` 47,105 | 47,090 | 15 | — |
| `eh-bare` 77,836 | **26,724** | 51,112 | — |
| `eh-partial` 4,375 | 4,372 | — | 3 |

**93,075 of the 354,646 marker-carrying bodies — 26.2 % — were on the wrong
side.** The two errors are large and they partly cancel, which is why the
headline moved only 14 %: 66,336 filed expensive that are cheap, 26,724 filed
cheap that are expensive. A predicate that is wrong 26 % of the time and looks
14 % wrong in aggregate is the worst kind, because the aggregate invites you to
call the correction a refinement.

**Two structural facts the numbers give for free:**

* **Every one of the 237,180 EH-side functions is blocked, and every one of the
  35,971 accepted marker-carrying functions is `eh-state0`.** The control group is
  not merely "the `empty-dtor-*` shapes read cheap" — the port has **never**
  accepted a function that needs an EH record, and the axis says so without being
  told. `eh-state1|INCLASS|*` is empty.
* **`eh-unknown` is unmoved to the function** (63,858 → 63,858). It must be: the
  walk never reached a marker, so no predicate keyed on markers can move it. That
  it came out exactly equal is the cheapest available check that the two axes ran
  over the same population.

### 10.4 What it costs the ranking — two rows INVERT

§7.4's cross is the table the board ranks from, and re-derived it does not merely
shift:

| census key | total | §7.4 `%EH` | **measured `%EH`** | §7.4 `eh-bare` | **measured cheap** |
|---|---:|---:|---:|---:|---:|
| `expr-op-0x27` | 411,967 | 8.2 % | **6.4 %** | 778 | 14,308 |
| `expr-intrinsic-this-adjust` | 135,938 | 25.7 % | **40.4 %** | 6,875 | 16,051 |
| `expr-intrinsic-base-member-addr` | 113,981 | **62.5 %** | **26.0 %** | 2 | **41,678** |
| `expr-bit-and` | 32,381 | 99.9 % | 99.9 % | 0 | 0 |
| `…-recv-object-then-branch-brtrue` | 23,633 | 99.9 % | 99.9 % | 0 | 0 |
| `body-0x9B` | 27,073 | **0.1 %** | **62.5 %** | **16,738** | **1** |
| `body-cflow-label` | 48,102 | 9.0 % | 9.1 % | 0 | 26 |
| `expr-intrinsic-base-upcast` | 19,468 | — | **42.5 %** | **8,277** | **0** |

* **`body-0x9B` inverts, and it is the largest single claim §7.4 made about the
  cheap side.** *"`body-0x9B` is 61.8 % `eh-bare` — 16,738 functions that need no
  EH model at all"*: measured, **exactly one** of them is cheap and 16,914 need
  the whole record. The 61.8 % was right about the fraction and wrong about which
  side.
* **`expr-intrinsic-base-upcast` inverts completely** — 8,277 "cheap" → **zero**
  cheap, 8,282 on the EH side.
* **`expr-intrinsic-base-member-addr` inverts the other way**: §7.4's *"62.5 %
  behind EH, the board's #3 row"* is measured at 26.0 %, and the row is now the
  **largest cheap population on the board** at 41,678.

Every one of those is a row that was ranked, or de-ranked, on the wrong number.

### 10.5 The cheap side is not a phase. It dissolves — RETIRED

§7.5 said EH *"is not the cheapest next thing — 40,881 functions sit on the cheap
side already measured"*, and §6o carried that forward. **That reading is
retired.** Not because the cheap side is small — it grew to 81,492 blocked — but
because **there is no cheap-side rung to schedule.** Its whole blocked stock:

| census key | n | share of the 81,492 |
|---|---:|---:|
| `expr-intrinsic-base-member-addr` | 41,678 | 51.1 % |
| `expr-intrinsic-this-adjust` | 16,051 | 19.7 % |
| `expr-op-0x27` | 14,308 | 17.6 % |
| `…-recv-intrinsic-this-adjust-then-plumbing-0x3A` | 2,058 | 2.5 % |
| `…-recv-field-off0-then-plumbing-0x3A` | 1,655 | 2.0 % |
| the tail (≈ 90 rows) | 5,742 | 7.0 % |

**88.4 % of it is three rows, and all three are general expression rows already
on the board** — the byte-offset add, the this-pointer adjust intrinsic, and the
base-member-address intrinsic. None is an EH construct. More to the point, in
each of them the EH-marked cheap bodies are a **minority slice**: 36.6 % of
`base-member-addr`, 11.8 % of `this-adjust`, **3.5 %** of `op-0x27`. The other
side of each row is ordinary non-EH code that the same widening retires anyway.

So the cheap side is not a gated phase waiting to be opened, and it is not even a
population that can be scheduled on its own: **every function on it is behind an
expression row that is already ranked, and widening that row retires its cheap-EH
slice for free.** The correct statement is that `/EHsc` costs those 81,492
functions *nothing* — which is what `maxState = 0` means — so they should never
have been counted as EH work in the first place.

The rows §7.4 nominated for that job (`body-0x9B`, `expr-intrinsic-base-upcast`)
are on the **other** side entirely.

### 10.6 The instrument defect this lane also fixed

The EH cross-tab built its rows as `format!("{}|{}", eh, verdict.key())`, and
`FnVerdict::key` spells **accepted shapes** and **blocker keys** into one
namespace. So the largest row of the entire EH cross was

```
eh-bare|empty-dtor-delegation    27,501
```

and `empty-dtor-delegation` is a shape the port **accepts**. It reads exactly
like a blocker, it sorted above every real one, and it was within one step of
being scheduled as a rung. The rows now name their population
(`|BLOCKED|` / `|INCLASS|`), there is a per-class `|BLOCKED` subtotal so a stock
can be sized without knowing the seventeen in-class label strings, and the
in-class rows print under a heading that says they are accepted functions.

The check §7 wanted from including them is kept and is now stronger, because it
is stated as a population rather than inferred from a label: **`eh-state1|INCLASS|*`
is empty and `eh-state0|INCLASS|*` holds all 35,971.**

### 10.7 What stays NOT MODELLED, and the one number at risk

* **`eh-unknown` = 63,858** — the walk stops before any marker, so the axis says
  nothing, in either direction. Unchanged by this work and unchangeable by it.
* **The 4,372 `eh-partial` bodies that became `eh-state1`.** A transfer already
  seen at a non-empty live set proves `maxState >= 1` whatever stops the walk
  later — but §6q's lesson stands: *landing off the tail is evidence something is
  wrong and no evidence about which token it was*, so a walk that later desynced
  may have mis-read the `5C`/`4C` that produced the proof. **Bound: if every one
  of the 4,372 were wrong, the EH side is 232,808, −1.8 %.** The headline does not
  depend on them.
* **Arrays of destructible objects, a destructible object passed by value, and an
  inlined destructor** — still unprobed (§9.10). The temporary is no longer on
  that list; it is probed, and it moved the counting site.
* **`/Ox`, `/O2`, packed.** Every number here is `/O1 /EHsc`, the workload's.
* **The obj-level grading is at probe scale** (§7.6's standing caveat, unchanged):
  46 functions were graded against their own objs; at workload scale the census
  population is the IL's function list and the obj emits only the subset needed.

### 10.8 The estimate, scored

Written in `work/EHMS/ESTIMATE.md` **before** the scan, with the bias named in
advance: *"I am primed to predict the cheap side grows… I expect to be too
small on it."*

| # | prediction | outcome | |
|---|---|---|---|
| P1 | cheap side, all = **110,000** (85k–150k), direction up | **117,463** | **HIT**, 6.4 % low |
| P2 | EH side, all = **240,000** (200k–265k), direction down | **237,180** | **HIT**, 1.2 % high |
| P3 | > 90 % of `eh-multi` stays EH | 99.97 % | **HIT** |
| P4 | the `eh-bare` false-cheap hole is real; 3,000–12,000 move | real; **26,724** | direction **HIT**, magnitude **MISS**, 2.2× above interval |
| P5 | `eh-unknown` delta exactly 0; 30–70 % of `eh-partial` decides | delta **0**; **99.93 %** decides | **HIT** / **MISS**, low |
| P6 | the cheap side's head is `body-0x9B` + `base-upcast` + `this-adjust`, ≥ 60 % | those three hold **19.7 %**; two of them are **0 %** cheap | **MISS** |
| P6′ | the cheap side dissolves into ordinary expression work | 88.4 % is three general expression rows | **HIT** |
| P7 | census delta 0, mismatch 0, disagreement 0 | 0 / 0 / 0 | **HIT** |

**Two headline numbers inside their intervals and within 7 %** — the first time
in this series after eight consecutive misses, and the intervals were widened
rather than recentred, which is what the named bias called for.

**Every miss is in the same direction, and it is the direction I named.** P4, P5
and P6 are all "I under-estimated how wrong the old axis was". The lesson is
sharper than the bias: I estimated the *correction* as a perturbation of the
published split, when the published split was produced by a predicate that is
wrong on a quarter of the population. **When a predicate is refuted, the prior
built on it carries no information about the magnitude of its own error** — P6 is
the proof, because it is the one prediction that copied the old table's row
identities forward, and it is the one that was wrong in kind rather than in size.

### 10.9 Reproduction

`work/` is gitignored, so the probe sources are reproduced here in full rather
than referenced. `m1.cpp` (11 functions) and `m3.cpp` (15) carry every
disagreeing cell; `m2.cpp` re-grades §7.2's own ctor/dtor shapes and is
confirmatory.

```cpp
// m1.cpp — the two predicates against each other
struct SE { int m; SE(); ~SE(); };
struct NC { int m; ~NC(); };            // destructor, no user constructor
int gp(int);  void gv();
int mA(int a){ SE s; int x=a*3; int y=x^7; return y+1; }   // cheap, stmt rule says EH
int mC(int a){ SE s; return gp(a); }                       // EH,    stmt rule says cheap
int mI(int a){ int x=gp(a); SE s; return x+1; }            // cheap, stmt rule says EH
int mF(int a){ { SE s; } { SE t; } return a+1; }           // cheap, stmt rule says EH
int mJ(int a){ SE s; for(int i=0;i<4;i++) a+=i; return a; }// cheap, stmt rule says EH
int mB(int a){ SE s; gv(); return a+1; }                   // EH,    both agree
int mD(int a){ SE s; return a+1; }                         // cheap, both agree
int mE(int a){ SE s; SE t; return a+1; }                   // EH,    both agree
int mG(int a){ NC s; return a+1; }                         // cheap, both agree
int mH(int a){ NC s; NC t; return a+1; }                   // EH — the raising
int mK(int a){ NC s; return gp(a); }                       // transfer is a DTOR call

// m2.cpp — §7.2's shapes, re-graded (all 20 agree, both rules)
struct Mem { int m; Mem(); ~Mem(); };  void Fini();  void Init();
struct One  { Mem a; ~One(){} };
struct Two  { Mem a; Mem b; ~Two(){} };
struct OneB { Mem a; ~OneB(){ Fini(); } };
struct B1   { Mem a; };
struct Ct1 : B1 { Ct1(){} };
struct Ct2 : B1 { Ct2(){ Init(); } };
struct Ct3 : B1 { Ct3(); };   Ct3::Ct3(){}
void u1(){ One x; }  void u2(){ Two x; }  void u3(){ OneB x; }
void u4(){ Ct1 x; }  void u5(){ Ct2 x; }  void u6(){ Ct3 x; }

// m3.cpp — the hardening cells
struct SE { int m; SE(); ~SE(); };
struct Mem { int m; Mem(); ~Mem(); };
int gp(int);  void gv();  SE mkSE();
void n1(){ { SE s; gv(); } }                          // EH
void n2(){ { SE s; } gv(); }                          // cheap, stmt rule says EH
int  n3(int a){ if(a){ SE s; } return gp(a); }        // cheap
int  n4(int a){ if(a){ SE s; gv(); } return a; }      // EH
int  n5(int a){ {SE s;}{SE t;}{SE u;} return a+1; }   // cheap, stmt rule says EH
struct C1 { Mem a; int k; C1(int); };  C1::C1(int x):k(gp(x)){}  // EH
struct C2 { int k; Mem a; C2(int); };  C2::C2(int x):k(gp(x)){}  // cheap — the
int  t1(int a){ return gp(mkSE().m) + a; }            // EH  — decisive pair, and
int  t2(int a){ mkSE(); return a+1; }                 // cheap  the temporary cell
int  l1(int a){ SE s; for(int i=0;i<4;i++) gv(); return a; }  // EH
int  l2(int a){ SE s; for(int i=0;i<4;i++) a+=i; return a; }  // cheap
void u1(){ C1 x(1); }  void u2(){ C2 x(1); }
```

```sh
export C2RS_WIBO=<the repo's resolved wibo>
cargo build --release                       # binary identity now prints in the
                                            # provenance block; do this FIRST
printf '/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc\n' > work/EHMS/flags.txt
for p in m1 m2 m3; do
  ./target/release/c2rs census work/EHMS/probe/$p.cpp --flags-file work/EHMS/flags.txt
  scripts/gt_eh.py work/EHMS/probe/$p.cpp | sed -n '/item 1/,/item 2/p'
done
# the census line prints `<maxState key> (<statement-count key>)` per function;
# `item 1` prints each symbol's Value — 0x8 iff the obj carries an EH record.

./target/release/c2rs gap --list work/dc3-workload/files.txt \
    --flags-file work/dc3-workload/flags.txt \
    --cwd <the workload tree> --jsonl work/dc3-workload/scan-ehms.jsonl
```

The four decisive cells are pinned as unit tests in
`crates/c2-il/src/func/body/shapes/control_flow.rs`
(`the_maxstate_axis_agrees_with_whether_the_obj_carries_an_eh_record`,
`the_boundary_is_one_transfer_wide`, `a_proven_state_survives_an_undecoded_tail`),
each with the obj reading in its doc comment, so the axis cannot regress silently
without the toolchain present.

### 10.10 A gate finding, beside the measurement

**The two `/EHsc` mode lanes are not standing lanes.** `scripts/mode_lane.sh`
takes extra flags, so `scripts/mode_lane.sh /O1 /EHsc` and `/Ox /EHsc` work and
were run for `docs/rungs/2026-07-31-ctor-base-delegation.md` — but nothing
enumerates the lanes. There is no lane registry, no gate script, and the four
recorded everywhere in `docs/` are `/Ox` · `/O1` · `/O2` · `/Ox /Gy`, none of
which compiles `/EH` at all. The `/EHsc` lanes exist only as an invocation a rung
author has to remember, on a workload that compiles `/EHsc` on **every** TU.

> **CLOSED, 2026-07-31 — and the fix is not the one this paragraph implies.**
> The finding above is correct and its obvious remedy ("remember to run the two
> `/EHsc` lanes") is not a remedy at all: it leaves the lane set as something a
> person recalls. What closed it is that the lane list became **data**
> (`scripts/lanes.txt`), with one command to run all of it
> (`scripts/gate.sh`) and a test that fails if the shipped registry stops
> carrying an `/EH` lane (`crates/c2-harness/tests/lane_registry.rs`). The
> standing set is now **12 lanes** — six code-shape configurations crossed with
> the exception-handling axis — so `/EHsc` is compiled at every configuration
> rather than at the two somebody thought to type. **A lane that exists but is
> not enumerated is a lane that does not run**, and adding a lane never fixes
> that; only enumerating them does (`docs/GAPS.md` §7,
> `docs/ARCHITECTURE_SEAMS.md` §2.4a). Run `scripts/gate.sh --jobs 4`, not the
> six invocations below.

Run here, all six are green and the two `/EHsc` lanes reproduce the figures that
rung recorded:

| lane | match | mismatch |
|---|---:|---:|
| `/O1` | 89 | **0** |
| `/Ox` | 91 | **0** |
| `/O2` | 89 | **0** |
| `/Ox /Gy` | 89 | **0** |
| **`/O1 /EHsc`** | **89** | **0** |
| **`/Ox /EHsc`** | **91** | **0** |

---

## 11. The records by NAME — c2's own `.cod`, across shapes (2026-08-01, W-EH, board #133)

§8.3 decoded the EH `.rdata` from **obj bytes**. This section re-derives the
same layout from c2's **assembly listing** (board #132's `c2rs listing`), where
every record is a named symbol and every field a separate `DD`. That makes this
a *second, name-carrying source* for a model already fitted — the #136
relationship (ROADMAP §9.9.3) — so **§8.3 is a control that could have gone
red**, which is the only reason a transcription is worth grading at all.

**Price, stated first: this moves the census by 0 by construction.** It is
Phase-5 groundwork. No rung is claimed.

Instrument: `scripts/gt_eh_cod.py`, **110 listings, 110 captured**:
**15 EH shapes × 4 flag sets** (`/O1 /Oi /EHsc`, `/O1 /Oi /EHa`, `/O2 /EHsc`,
`/Ox /EHsc`) = 60, plus **5 held-out `maxState` shapes**, **5 held-out gap
combinations** and **40 single-axis gap probes** at `/O1 /Oi /EHsc`. Of the 15 EH
shapes, **2 are the fitted set and 13 were held out**. The axes are **structural
counts**, per §9.13.1's consequence 2
— try blocks 0–4, nesting depth 0–4, catch clauses per try 1–4, destructible
objects 0–5, functions per TU 1–3, catch by value / by `&` / by `const&` / by
pointer / ellipsis — *not* the contents of one try. Every held-out probe's counts
were predicted from source and committed before capture.

### 11.1 The record set, and the order they are emitted in

Per EH function, in `.cod` emission order, all of it inside the function's own
COMDAT group:

```
.pdata   $T<a>   DD  <body or funclet symbol>      one per emitted body
                 DD  <unwind word>
.data    ??_R0<mangled>                            one per DISTINCT caught type per TU
.rdata   __unwindtable$F      maxState  x UnwindMapEntry    (8 B)
         __catchsym$F$k       nCatches  x HandlerType       (16 B), one array per try block
         __tryblocktable$F    nTryBlocks x TryBlockMapEntry (20 B)   -- absent iff nTryBlocks == 0
         __ehfuncinfo$F       FuncInfo, 9 dwords            (36 B)
         ORG $+4                                            -- alignment pad, PRINTED
         $T<b>                nIPMapEntries x IpToStateEntry (8 B)
```

Two things the listing states that §8.3 had to infer:

* **The 8-byte alignment of the ip-to-state array is printed outright**, as a
  literal `ORG $+4` directive between `__ehfuncinfo$` and its `$T`. §8.3 proved
  the 9-dword `FuncInfo` *by* that alignment, from two symbol offsets; here it is
  a directive. Both pad values occur — **pad 0 on 13 probes, pad 4 on 50** — so
  neither is a constant that could be mistaken for the other.
* **`__tryblocktable$F` is absent entirely when `nTryBlocks == 0`**, and
  `pTryBlockMap` is then 0 — §8.3 recorded the null pointer but not the missing
  record.

### 11.2 `FuncInfo` — §8.3's byte-derived table, confirmed field for field

The listing names all nine dwords, and **agrees with §8.3 on 9 of 9** (A3 HIT).
No field moved, none was added, and there is still no `dispUnwindHelp`.

| off | field | `.cod` operand | measured here |
|---|---|---|---|
| +0x00 | `magic` | `019930522H` | identical on all 105 |
| +0x04 | `maxState` | literal | 1..8 (§8.3 saw 1..6) |
| +0x08 | `pUnwindMap` | `__unwindtable$F` | never null |
| +0x0c | `nTryBlocks` | literal | 0..4 (§8.3 saw 0..3) |
| +0x10 | `pTryBlockMap` | `__tryblocktable$F` or `00H` | 0 iff `nTryBlocks == 0` |
| +0x14 | `nIPMapEntries` | literal | 1..10 |
| +0x18 | `pIPtoStateMap` | `$T<b>` | never null |
| +0x1c | `pESTypeList` | `00H` | **0 on all 105**, every mode |
| +0x20 | `EHFlags` | `01H` / `00H` | **see §11.4** |

`UnwindMapEntry = { i32 toState; ADDR32 action }`, `TryBlockMapEntry = { i32
tryLow, tryHigh, catchHigh, nCatches; ADDR32 pHandlerArray }`, `HandlerType =
{ u32 adjectives; ADDR32 pType; i32 dispCatchObj; ADDR32 addressOfHandler }`,
`IpToStateEntry = { ADDR32 $M; i32 state }` — all four confirmed by name.

**Try blocks are emitted INNERMOST FIRST.** `fit_nested2`'s table is
`(tryLow 3, tryHigh 3, catchHigh 4, nCatches 2)` then `(1, 4, 5, 2)`: the inner
block precedes the enclosing one, and the enclosing one's `tryLow..tryHigh`
spans it. Nothing in §8.3 fixed that order, and a table built in source order
would be wrong on every nested function.

### 11.3 `maxState` — a law, held out

Every one of A2's misses was `maxState`, all in one direction. Corrected:

> **`maxState` = (destructible objects in scope) + 2 × (lexical `try` blocks).**

A **try block is worth two states, not one.** Fitted on the 13 round-1 cells,
then registered and graded on **five shapes it was never fitted on**
(`z_nest2_dtor2` 6, `z_try4seq` 8, `z_try1catch4_dtor3` 5, `z_dtor5` 5,
`z_deep4` 8): **10 of 10 exact**, including a four-deep nest and a four-block
sequence, which are the two arrangements that separate "per try block" from
"per nesting level".

**`nIPMapEntries` is NOT MODELLED for try shapes and this lane declined to guess
it** — §9.7 already refuted the no-try rule there, and the observed values
(`h_try1` 1, `h_try2seq` 4, `h_try3seq` 7, `h_nest3` 3) are not a function of
any count in this table. Declining scored those nine cells **zero**; they are
not counted as passes.

### 11.4 `/EHa` — a second mode, and §8.3's `0x40` is `/EHsc`-only

§8.3 measured `EHFlags = 1` and ellipsis `adjectives = 0x40` "on all 21" — under
`/EHsc`, which was the only mode it ran. c2 **accepts `/EHa`**, and both claims
are mode-scoped:

| | `/EHsc` (O1, O2, Ox) | `/EHa` |
|---|---|---|
| `EHFlags` (+0x20) | **`01H`** | **`00H`** |
| `catch(...)` `adjectives` | **`040H`** | **`00H`** |

Everything else — every record, every field, every count — is byte-identical
between the two modes on all 21 probes. So `/EHa` moves exactly two dwords, and
a port that hard-codes either would be wrong on an `/EHa` TU.

`adjectives`, re-measured by name at `/EHsc` and consistent with §8.3:

| catch clause | `adjectives` | `pType` |
|---|---|---|
| `int e`, `V v` (by value, user copy ctor + dtor) | `00H` | `??_R0H@8`, `??_R0?AUV@@@8` |
| `const char* volatile p` | `01H` | `??_R0PAD@8` |
| `E1& e`, `E2& e` | `08H` | `??_R0?AUE1@@@8`, `??_R0?AUE2@@@8` |
| `const E1& e`, `const E2& e` | `09H` | as above |
| `...` | `040H` (`/EHsc`) · `00H` (`/EHa`) | **NULL** |

`0x01` const, `0x08` reference, `0x40` ellipsis-under-`/EHsc`. **`0x02`
(volatile) is still not isolated and stays `NOT MEASURED`** — the `volatile` in
the pointer row qualifies the pointer, not the pointee, and prints `01H`.

`TypeDescriptor` is unchanged from §8.4: `{ ADDR32 ??_7type_info@@6B@; u32 0;
char name[] }`, one COMDAT per distinct caught type per TU, and `catch(...)`
emits none.

### 11.5 Totality, and the check that has teeth

**A1 — every datum claimed by a named field: 598/598 fitted, 2,436/2,436 held
out, residue 0.**

That number on its own is worth nothing, and this document has said so before.
**Residue cannot see a SHORT read**: if the parser loses data the record simply
has fewer fields, every one is still claimed, and the run prints success. That is
not hypothetical — c2 **run-length-encodes** its data:

```
__ehfuncinfo$?f@@YAHH@Z DD 019930522H
        DD  01H
        DD  __unwindtable$?f@@YAHH@Z
        DD  2 DUP(00H)          <-- nTryBlocks AND pTryBlockMap, in one operand
```

The first version of this instrument read that record as **8 dwords, residue 0,
every field claimed** — with `pIPtoStateMap` decoded onto `nIPMapEntries`. So
totality is graded beside an **arity** check that predicts each record's length
from a count field in a *different* record (`__unwindtable$` from `maxState`,
each `__catchsym$` from its own try-block entry's `nCatches`, `$T` from
`nIPMapEntries`, `FuncInfo` from the constant 9): **332/332 consistent.**

Three falsifications, each red with a distinct signature:

| mutation | totality | arity |
|---|---|---|
| the `DUP` expansion removed (the bug that really happened) | **residue 0 — SILENT** | **16 red**, `FuncInfo got 8 want 9` |
| `FuncInfo` truncated to 8 named fields | residue 8 fitted / 60 held | — |
| `HandlerType` read as 5 dwords, x86's `copyFunction` | residue 36 / 240 | — |

**Read the first row.** The mutation that actually occurred in practice is
invisible to the totality metric and caught only by arity. A residue-only grade
would have shipped it.

### 11.6 The residue, named

**`__catchsym$F$k` — the `$k` suffix is NOT MODELLED**, and it is not cosmetic:
the array is anchored by a `STATIC` symbol whose *name* goes in the obj string
table, so a wrong `$k` is a wrong-bytes obj. Measured, `/O1 /Oi /EHsc`:

| probe | try blocks | `maxState` | `$k` |
|---|---:|---:|---|
| `h_try1` | 1 | 2 | `2` |
| `h_try2seq` | 2 | 4 | `4, 5` |
| `h_try3seq` | 3 | 6 | `6, 7, 8` |
| `z_try4seq` | 4 | 8 | `8, 9, 10, 11` |
| `h_catch4` | 1 | 2 | **`6`** |
| `h_dtor2_try2catch` | 1 | 4 | **`5`** |
| `z_try1catch4_dtor3` | 1 | 5 | **`9`** |
| `h_2fn` | 1 (each of 2 fns) | 2 | **`2`, `2`** |

On the sequential-try ladder the first `$k` equals `maxState` and the rest
ascend — and **`h_catch4` refutes that as a law** (`maxState` 2, `$k` 6). `h_2fn`
shows the counter is **per function**, not per TU: two functions both get `$2`.
It behaves like the per-function local-symbol ordinal that also numbers
`$LN`/`$LL` and the catch object's `e$NNNN`, and this lane did not model it.
**Anyone building the Phase-5 emitter needs this and does not have it.**

Also unmodelled, and smaller: the `.pdata` unwind word (`040000a04H` on every
funclet, `0c000XXXX` on bodies) is §2's business, not re-derived here.

### 11.7 What this does NOT give

* **Nothing about c2's *input*.** The listing is an output artifact. It says
  nothing about the `.gl`/`.ex` IL containers — see the `#121` verdict in
  ROADMAP §9.15.2.
* **No emitter.** A layout is a correspondence, and §9.9 already records that
  the oracle cannot grade a correspondence. Everything above is graded on
  totality-plus-arity and on predicting held-out shapes; the byte compare has
  graded none of it, because nothing was built.
* **`nIPMapEntries` for try shapes**, the `$k` suffix, and `0x02`
  (volatile) remain open, and they are named above rather than fitted.

### 11.8 Reproduction

```sh
scripts/gt_eh_cod.py gen
scripts/gt_eh_cod.py scan --jobs 6      # 105 listings, 21 probes x 4 modes
scripts/gt_eh_cod.py records h_catch4   # one probe, decoded field by field
scripts/gt_eh_cod.py totality           # A1  + the residue, printed
scripts/gt_eh_cod.py arity              # A1b — the check with teeth
scripts/gt_eh_cod.py predict            # A2  round 1, registered pre-capture
scripts/gt_eh_cod.py predict2           # A2  round 2, the maxState law held out
scripts/gt_eh_cod.py gaps               # board #138, ROADMAP §9.15.3
```
