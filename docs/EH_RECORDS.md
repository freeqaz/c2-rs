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
