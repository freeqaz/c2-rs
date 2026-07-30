# MVP codegen — IL → PPC big-endian `.text`

Instruction encoding and register-allocation facts for the MVP function
class (straight-line integer add + return). Verified against the live
toolchain with differential probe fixtures this session; COLOR behavior
cross-referenced with `../dc3-decomp/msvc-src/docs/COLOR_RE.md`.

Target for `int add3(int a,int b,int c){return a+b+c;}` — 12 bytes,
big-endian:

```
7d632214   add r11,r3,r4
7c6b2a14   add r3,r11,r5
4e800020   blr
```

## Instruction encoding

PPC instructions are fixed 32-bit words, stored **big-endian** in `.text`
(unlike the COFF struct fields, which are little-endian). IBM bit
convention: bit 0 = MSB.

### `add rD, rA, rB` (rD = rA + rB) — primary op 31, XO 266

```
word = (31<<26) | (RD<<21) | (RA<<16) | (RB<<11) | (OE<<10) | (266<<1) | Rc
```

OE=0, Rc=0 for everything c2 emits here (Rc=1 would be `add.`). Verified
bit-exact against both MVP words. `add` is operand-commutative (rA↔rB) —
the ONLY arithmetic op in this family where a swap is licensed (see the
hazard list).

### `blr` — primary op 19 (bclr), BO=20 (always), XO=16

```
(19<<26) | (20<<21) | (16<<1) = 0x4E800020
```

Fixed 4-byte constant; no operands.

## X360 integer ABI

- Args: `r3..r10` left-to-right → `add3(a,b,c)`: a=r3, b=r4, c=r5.
- Return: `r3`.
- Volatile: r0, r3–r12. Callee-saved: r14–r31. r1=SP, r13=reserved
  (small-data/thread), **r12 = reserved volatile scratch** (call glue).
- Leaf function touching only volatiles: **no prologue/epilogue, no stack
  frame** — body then `blr`.

## COLOR scratch order (observed, matches COLOR_RE.md linear-scan model)

Parameters and the return value are pinned to their ABI registers; only
genuine temporaries draw from the allocator. Phase-1 volatile priority is
`r12, r11, r10, …` with **r12 skipped (reserved)**, so:

- First free temp → **r11**, reused across a serial chain.
  Probe: `a+b+c+d` → `add r11,r3,r4; add r11,r11,r5; add r3,r11,r6; blr`.
- Second simultaneous temp → **r10** (descending).
  Probe: `(a+b)*(c+d)` → `add r11,r3,r4; add r10,r5,r6; mullw r3,r11,r10; blr`.

MVP rule: **serial integer temps use r11; the final operation writes r3.**

## Instruction selection (postfix IL → regs)

The `.ex` body is a postfix/stack expression stream (see
`IL_BUNDLE_MVP.md`): `LOAD a, LOAD b, ADD, LOAD c, ADD, RETURN` for
`(a+b)+c` — left-associativity is already encoded.

1. Pre-color params to incoming ABI regs by position; return temp → r3.
2. Walk the stream with an operand stack of physical registers; `LOAD`
   pushes the var's register.
3. `ADD`: pop rhs, pop lhs. Dest = r11 for every add except the last;
   dest = r3 for the final add (the one feeding RETURN). Emit
   `add dest, lhs, rhs`, push dest. Keeping lhs = under, rhs = top
   reproduces c2's rA/rB choice.
4. `RETURN` of a value already in r3 emits only `blr`.

## Integer sub / mul (W2, implemented + gated)

Two more binary ops are in the straight-line class, verified byte-exact:

- **`mul` → `mullw rD,rA,rB`** (op 31, XO 235): `rD = rA*rB`, **commutative**.
  `a*b*c` → `mullw r11,r3,r4 ; mullw r3,r11,r5 ; blr`
  (`7d6321d6 7c6b29d6 4e800020`). Same operand-order freedom as `add`.
- **`sub` → `subf rD,rA,rB`** (op 31, XO 40): `rD = rB − rA`
  (**first register operand is the subtrahend**), **NON-commutative**.
  `a-b-c` → `subf r11,r4,r3 ; subf r3,r5,r11 ; blr`
  (`7d641850 7c655850 4e800020`). To realize source `lhs − rhs` the selector
  emits `subf dest, rhs, lhs` (rA=rhs=subtrahend, rB=lhs=minuend). **Swapping
  rA/rB is a silent sign inversion** — a valid `subf`, just the wrong one,
  invisible to `fuzzy%`. This is exactly the CLAUDE.md correctness-boundary
  hazard, so `encode_subf` is a separate function with the mapping documented
  at its one call site (`select_text`'s `Sub` arm), per the opt-in-encoder
  rule. IL opcodes: `add`=`0x02`, `sub`=`0x03`, `mul`=`0x04` (postfix, each
  pops two operands).

## Integer literals / immediates (W3, implemented)

IL literals are `33 <int-type> <varint>`, where the varint is a **single byte
if `< 0x80`** (the value directly), else **`0x80` + a 4-byte LE i32**
(verified: 5→`05`, 42→`2a`, 200→`80 c8000000`, 70000→`80 70110100`). Codegen
folds a literal operand into an immediate instruction exactly as c2 does — the
selection stack carries `Reg | Imm`:

- **`reg + k`** (either order) → **`addi rD, reg, k`** (`a+5` → `addi r3,r3,5`
  = `38630005`).
- **`reg − k`** → **`addi rD, reg, −k`** — c2 folds the subtraction of a
  constant into an add-immediate with negated value (`a-5` → `addi r3,r3,-5`
  = `3863fffb`). (`k − reg` is `subfic`, not modeled — rejected.)
- **bare `return k`** → **`li rD, k` = `addi rD, r0, k`** (`return 42` →
  `addi r3,r0,42` = `3860002a`; `addi` special-cases rA=0 to the literal 0).

**Wide immediates (W3b, implemented)** — constants outside signed 16-bit:

- **`reg ± wide-K`** → **`addis`+`addi`**, splitting `K` into a
  sign-compensated high half and a sign-extended low half (`lo = (i16)K`,
  `hi = (K − lo) >> 16`, so the `addi`'s sign extension is absorbed). `a+70000`
  → `addis r3,r3,1 ; addi r3,r3,4464`; `a-70000` → `addis r3,r3,-1 ; addi
  r3,r3,-4464`.
- **bare wide `return K`** → the **`lis`+`ori` idiom** `addis rD,r0,hi ; ori
  rD,rD,lo` (unsigned halves, no sign compensation). `return 70000` → `addis
  r3,r0,1 ; ori r3,r3,4464`.

Still out-of-class (rejected, not mis-emitted): **multiply by a constant**
(strength-reduces to shift+add, e.g. `a*3` → `rlwinm r11,r3,1,0,30 ; add
r3,r3,r11`); `const − reg` (`subfic`); a negative wide bare constant.

## The frame model (roadmap #35 step 1) — MEASURED 2026-07-30

Everything here was read out of reference objs at `/Ox /GS- /c` (probe sources
`work/fm/**`, one-liners of the form `int g(…); T f(…){ … g(…) … }`). The model
is `c2_core::codegen::FrameLayout`; every row below is pinned by a unit test
against the captured words.

> **Read `docs/CODEGEN_FRAMED_CALLS.md` alongside this.** It was produced
> independently and in parallel from 480 *designed* compiles per mode, and the
> two derivations **agree** — which is the strongest thing that can be said
> about either, since neither knew the other's probes. It is the wider document
> (the `nOutSlots` term, argument marshalling, symbol order, EH); this section is
> the narrower and deeper one on two axes it does not cover — **stack probing and
> `_RtlCheckStack12`** (its `localsBytes` tops out at 132, this one's at 200,000)
> and the **mixed inline GPR+FPR epilogue order**. Where they overlap, prefer its
> numbers: it has 10x the witnesses.

### 1. Frame sizing — one formula, 44 witnesses, zero residual

```
  frame_size = align16( 80 + locals + 8 + 8 × (saved_gprs + saved_fprs) )
```

This is the `nOutSlots <= 8` case of the general rule
(`CODEGEN_FRAMED_CALLS.md` §1.2), which every probe of this rung satisfies:

```
  locals_base = align16(16 + 8·max(nOutSlots, 8))
  F = align16( max(16 + 8·max(nOutSlots, 8), locals_base + localsBytes)
               + 8·nSaved + 8 )
```

`FrameLayout` implements the general form, and its unit test carries four rows
from the other document's sweep as a cross-check — including the one that refutes
the natural wrong model: `int g();` with `int b[20]` and **no** outgoing
arguments is a 176-byte frame, not 112, because the 64-byte parameter area is
reserved whether or not anything is passed. The rule stops being exact past 17
saved registers, where the allocator spills; that is refused, not guessed.

* **80** is the fixed head: a 16-byte linkage area (back chain at `0(r1)`) plus a
  64-byte parameter save area for r3–r10. The first local is always at `80(r1)`
  (`stb r3,80(r1) ; addi r3,r1,80` for a `char buf[…]` passed by address).
* **8** is the LR slot at `F−8(r1)`, written *before* the `stwu` as
  `stw r12,-8(r1)` and read back *after* the `addi r1,r1,F`.
* **8 per saved register**, GPRs and FPRs sharing one descending slot array
  directly under the LR slot.

| probe | locals | saved | frame | witness |
|---|---:|---|---:|---|
| `return g(a)+1` | 0 | — | 96 | `9421ffa0` |
| `return g(a)+b` | 0 | r31 | 96 | `9421ffa0` |
| `…+b+c` | 0 | r30,r31 | 112 | `9421ff90` |
| `…+b+c+d` | 0 | r29–r31 | 112 | `9421ff90` |
| `…+b+c+d+e` | 0 | r28–r31 | 128 | `9421ff80` |
| `…7 live` | 0 | r25–r31 | 144 | `9421ff70` |
| `float g(a)*b` | 0 | f31 | 96 | `9421ffa0` |
| `…*b*c*d` | 0 | f28–f31 | 128 | `9421ff80` |
| `char buf[1]` | 1 | — | 96 | |
| `char buf[9]` | 9 | — | 112 | |
| `int buf[16]` | 64 | — | 160 | |
| `int buf[900]` | 3600 | — | 3696 | `9421f190` |
| `char buf[8096]` | 8096 | — | 8192 | `9421e000` |
| `char buf[200000]` | 200000 | — | 200096 | |

This replaces the roadmap's "96 B for one by-value temporary, 112 B for two":
the driver of that pair is the **callee-saved register count**, not a count of
temporaries. A by-value temporary moves the `locals` column instead — e.g. the
8-byte int→double conversion spill at `80(r1)` in a mixed int/float body.

### 2. Callee-saved registers

Always a contiguous run ending at the top of the file: GPRs `r(32−n)…r31`,
FPRs `f(32−n)…f31`. Slots descend from the LR slot, **GPRs first**:

```
  int b,c live across the call, one float too    F = 128
  F−8   LR      F−16  r31     F−24  r30     F−32  f31
  fbc1ffe8  std  r30,-24(r1)     ← prologue: LR store, then GPRs, then FPRs,
  fbe1fff0  std  r31,-16(r1)       each run ascending in slot address
  dbe1ffe0  stfd f31,-32(r1)
  …
  cbe1ffe0  lfd  f31,-32(r1)     ← epilogue: one list, ascending in address,
  ebc1ffe8  ld   r30,-24(r1)       so FPRs come FIRST — the two lists are not
  ebe1fff0  ld   r31,-16(r1)       mirror images
```

GPRs are saved with **`std`** (64-bit) and FPRs with `stfd`; the epilogue's GPR
and FPR restores come **after** the `mtlr r12`.

*Which* value gets which register is the COLOR allocator's business and is **not**
modeled: with two saved GPRs the first live value takes r30 and the second r31,
but with three or more the assignment is not monotone in source order
(`g(a)+b+c+d` gives b→r29, c→r31, d→r30). That is roadmap #35 step 2.

### 3. `__savegprlr_N` / `__restgprlr_N` — threshold **3**

```
  int f(int a,int b,int c,int d){ return g(a)+b+c+d; }   3 saved GPRs
  7d8802a6  mflr r12
  4bfffffd  bl   __savegprlr_29      REL24, external — saves r29..r31 AND the LR,
  9421ff90  stwu r1,-112(r1)         so the `stw r12,-8(r1)` disappears
  …
  38210070  addi r1,r1,112
  4bffffc4  b    __restgprlr_29      REL24 — a TAIL branch: the helper restores
                                     r29..r31 and the LR and returns, so there is
                                     no mtlr/blr at all
```

`N` is the **lowest** saved register. Two saved GPRs are open-coded `std`s
(`mix1`, 30112-byte frame, `std r30,-24 ; std r31,-16`); three are the helper
(`mix2`, same frame) — the threshold is pinned by that pair, not by one side.

### 4. `__savefpr_N` / `__restfpr_N` — threshold **4**, and it is a different one

```
  float f(…5 floats…)                                   4 saved FPRs
  7d8802a6  mflr r12
  9181fff8  stw  r12,-8(r1)          the FPR helper does NOT save the LR
  3981fff8  addi r12,r1,-8           r12 = the slot array base
  4bfffff5  bl   __savefpr_28        REL24, external
  9421ff80  stwu r1,-128(r1)
  …
  38210080  addi r1,r1,128
  3981fff8  addi r12,r1,-8
  4bffffc1  bl   __restfpr_28        a CALL, not a tail branch
  8181fff8  lwz  r12,-8(r1)
  7d8803a6  mtlr r12
  4e800020  blr
```

Three saved FPRs are open-coded `stfd`s and four are the helper — so the GPR
threshold (3) and the FPR threshold (4) are **not the same number**, which is why
`FrameLayout` has two predicates and not one. The naming is `__savefpr_N` /
`__restfpr_N`, established from the obj's symbol table rather than assumed from
the GPR pair (which is `gprlr`, not `gpr`, because it also carries the LR).

Not determined: the `addi r12,r1,-8` offset when GPRs are saved *too*. It must
become `-(8 + 8×gprs)` for the FPRs to land under them, but the combination
(≥3 GPRs and ≥4 FPRs) has no capture and is refused rather than guessed.

### 5. Stack probing, and `_RtlCheckStack12` — threshold **5 pages**

Below `0x5000` the frame is probed inline, one touch per page boundary crossed:

```
  n_probes = floor((frame_size − 1) / 4096)        ld r12,-4096k(r1), k = 1..n
```

`F = 4096` probes nothing and `F = 4112` probes once (`d04`/`d06`), so the
boundary is the number of boundaries *crossed*, not `F/4096`. From `0x5000` up it
is the runtime helper:

```
  char buf[32000]                    F = 32096
  398082a0  li   r12,-32096          the size, negated, in r12
  4bfffff5  bl   _RtlCheckStack12    REL24, external
  7c21616e  stwux r1,r1,r12          opcode 31 XO 183 — the variable-size stwu
  …
  38217d60  addi r1,r1,32096
```

* `F = 20464` is four inline probes and `F = 20480 = 5 × 4096` is the helper —
  the threshold is on the frame size, **not** on the probe count (both would be
  4). Pinned by that pair.
* `li r12,−F` while `F ≤ 32768`, else `lis r12,hi ; ori r12,r12,lo` (`F = 32768`
  → `li r12,-32768`; `F = 32784` → `3d80ffff 618c7ff0`).
* The epilogue frees with `addi r1,r1,F` while `+F` fits the immediate, else
  `lwz r1,0(r1)` through the back chain (`F = 32752` → `addi`; `F = 32768` →
  `lwz`).

This refutes the roadmap's framing of the item as "a call to `_RtlCheckStack12`
for frames past a page": past *one* page there is no call at all, only inline
`ld` touches, and the call arrives four pages later.

### 5a. The `/Gy` label stride of a helper-using frame is 7, not 5

`CODEGEN_FRAMED_CALLS.md` §4.4 refutes `OBJ_GY_SHAPES.md` §3.5's
`framed -> cur += 5 if /Gy` for exactly the frames this section refuses: a framed
function using the `__savegprlr_N`/`__restgprlr_N` pair consumes **two extra
label slots, allocated before its own `$M` pair**. Seven witnesses, differenced
against the `.gl+7+9` seed.

It is latent rather than live *because* `FrameLayout` refuses those frames — the
port emits only the no-helper class, whose stride is the 5 the emitter models. It
becomes six wrong bytes per label the moment a framed function with three or more
saved GPRs is admitted, so **the helper codegen and the stride correction have to
land in the same rung.** The FPR-helper stride is predicted +4 by the same
reading and is *not* captured; it is not claimed.

### 6. What the emitter builds, and what it refuses

`FrameLayout::prologue`/`epilogue` build any layout that needs **no external
helper and no stack check**; the three helper shapes refuse by name
(`frame-savegprlr-helper`, `frame-savefpr-helper`, `frame-rtlcheckstack12`)
because each puts a second REL24 site in the prologue that `coff::Function` does
not model. The thresholds are therefore load-bearing gates rather than
decoration. Only the all-zero layout is reachable from the accepted class today;
the rest are pinned by unit tests against the captured words so the next rung
inherits measurements instead of a guess.

## W4b2 non-leaf calls — IMPLEMENTED, byte-exact (single-function TU)

`return g(a) + k` (the call result is used, so f is non-leaf) is implemented and
byte-exact for a **single-function TU** (`fixtures/cpp/mvp_framed.cpp`,
`differential_mvp_framed_call_byte_exact`). It needs a `.pdata` unwind section
and three compiler label symbols on top of the tail-call layout. IL detection is
`c2_il::func::parse_segment` (the `FramedCall` shape); codegen is
`codegen::framed_call_text`; the COFF image is `coff::emit_framed_obj` (the
5-section `emit_obj` path is untouched).

**`.text` (0x24 bytes when the call's argument is the formal already in r3):**
```
7d8802a6  mflr r12
9181fff8  stw  r12,-8(r1)          prologue (3 words): save LR
9421ffa0  stwu r1,-96(r1)          allocate the 96-byte frame (§"frame model")
[7c832378 or r3,rN,rN]             ARGUMENT SETUP — only when the argument is
                                   NOT the formal in r3
4bfffff5  bl   g                   REL24 reloc (disp = −(its own offset), LK=1)
38630001  addi r3,r3,1             the post-call op (here +1); k varies
38210060  addi r1,r1,96            epilogue (4 words): free frame
8181fff8  lwz  r12,-8(r1)          restore saved LR
7d8803a6  mtlr r12
4e800020  blr
```
Prologue and epilogue are the all-zero [`FrameLayout`]; only the `addi r3,r3,k`
immediate, the callee, the `bl` displacement and the argument setup vary.

> **The argument setup was missing, and that was a live wrong-bytes emit
> (found and fixed 2026-07-30).** This body was emitted as one byte-constant
> 0x24-byte blob. The parser required the call's argument to be *a* formal and
> then dropped the formals list, so the emitter assumed it was the formal already
> in r3 — and c2 emits `or r3,rN,rN` first whenever it is not, making the body 10
> words with the `.pdata` `FuncLen`, both `$M` label values and the REL24 site all
> following it wrong. **37 of 47 probes around the accepted class mismatched**:
> every argument at a non-zero formal position, every member function (`this`
> occupies r3, so a one-parameter member's argument is in r4), and every free
> function with a leading `float`, `double`, `long long`, pointer or 8-byte
> aggregate parameter — each of which takes one GPR slot on this ABI.
>
> It hid because every framed fixture and all 363 generated framed cases were
> `int F(int a) { return g(a) + 1; }`: one parameter, necessarily in r3, so the
> argument's *index* and its *register* were the same number everywhere the class
> had ever been graded. That is `docs/GAPS.md` §6's recurring shape for the fifth
> time, and the corpus held only the safe half of the pair. Fixtures
> `wfr_argreg.cpp` (position), `wfr_argreg_types.cpp` (leading parameter type),
> `wfr_argreg_member.cpp` (`this`); sweep axis 5.
>
> **Past the eighth formal it is not a register move at all** —
> `int f(int a,…,int i){ return g(i)+1; }` is `lwz r3,180(r1)`, whose slot
> displacement is a function of the whole list's ABI footprint. Refused
> (`framed-arg-over-eight-formals`, `wfr_argreg_neg.cpp`), sized at **zero
> functions** on the 878-TU workload. `a*5` post-op strength-reduces (`rlwinm`+`add`,
size 0x28) — **out of the `+k` scope, rejected**: `parse_call_shape` accepts as
framed only a literal `33 86 41 74 <varint>` **immediately followed by ADD
(`0x02`)** whose `k` is non-zero and fits a signed-16-bit `addi` (so `*k` =
`0x04`, `-k` = `0x03`, and wide `k` are all rejected → `NotImplemented`, never
mis-emitted; `+0` folds to the integer tail call, below).

**W4b2-v — acceptance is a positive whole-body parse (honesty claim, scoped).**
The honesty guarantee is precisely this: **every accepted body is exactly one of
three recognized shapes, and every other body is rejected**, because
`c2_il::func::parse_segment` tokenizes the entire `.ex` operand stream (from the
`4C 4F 11` 'LO' marker to the segment end) and accepts only on a *complete*
positive match that *reaches the end* — it is not a search for a favorable
pattern near the first CALL. This is the doctrinal fix for two rounds of the
same over-acceptance: the earlier trio (`parse_body` / `is_tail_call` /
`parse_framed_call`) each matched on a *local* byte neighborhood, so a second
call, a trailing statement, or in-argument arithmetic sat *outside* the window
each gate inspected and was silently dropped. The full grammar (token classes +
the shape productions) is in `docs/IL_BUNDLE_MVP.md` ("`.ex` whole-body
grammar"). The four accepted shapes are: the straight-line int arithmetic leaf;
the bare terminal void tail call `void f(){ g(); }` (`26 tok` · CALL · `4C 4B` ·
return plumbing); the **integer tail-call family** `return g(<arg>)` (`26 tok` ·
CALL · an argument sub-expression · `55 86 41 74 4C` call-end · a **net-identity
post-op**; see below); and the framed `return g(a) + k` (k ≠ 0) (`26 tok` · CALL
· one passthrough LOAD · `55 86 41 74` call-end · `4C` · one literal `+ k` ·
ADD).

The load-bearing boundary inside the call shape is the `55 86 41 74` call-end
marker: a **post-op** is emitted *after* it, **argument setup** *before* it.
Captured evidence (`.ex` from the `LO` marker):
```
g(a)+1  … 55 86 41 74 | 4c 33 86 41 74 01 02 …   literal+ADD AFTER  the marker → framed +1
g(a+1)  … 33 86 41 74 01 02 | 55 86 41 74 4c 41 … literal+ADD BEFORE the marker → arg-setup
```
Both are now modeled (see the int tail-call family below): the post-op
classifies the shape, the argument region is a modeled sub-expression computed
into r3.

Regression fixtures, each asserted `NotImplemented` in
`differential_out_of_class_call_shapes_not_implemented`: `mvp_call_submod.cpp`
(`g(a)-1`), `mvp_call_mulmod.cpp` (`g(a)*5`), `mvp_call_widemod.cpp`
(`g(a)+70000`), and the W4b2-v probes `mvp_call_twice.cpp` (`g();g();`),
`mvp_call_then_stmt.cpp` (`g();return a+1;`), `mvp_call_argframed_plusk.cpp`
(`g(a+1)+1` — arg-setup AND a framed post-op, out of the modeled classes),
`mvp_call_two_framed.cpp` (`g(a)+g(a+1)`), `mvp_call_plus1plus2.cpp`
(`g(a)+1+2`). The two mis-emits the old gates produced — a bare `b g` that
dropped a second call/statement, and a framed obj that dropped in-argument work
— are now impossible by construction: the parse would not reach the segment end.

## W4b2-iv/-vi integer tail-call family — IMPLEMENTED, byte-exact

An **integer tail call** `return g(<arg>)` lowers to `<arg-setup> ; b <callee>`:
the single call argument computed into r3, then a `b` tail branch (REL24, LK=0,
no frame — a 5-section leaf, the integer analog of the void tail call). The
post-op after the `55 … 4C` call-end classifies the shape; a **net-identity
post-op** (absent, or `+0` folded) is a tail call, a genuine `+k` is framed.
Verified live against 16.00.11886.00 (each a 5-section, 15-symbol obj; the `bl`
callee is symbol 14; `.text` big-endian):

| source | `.text` | REL24 offset |
|---|---|---|
| `return g(a)` (passthrough) | `48000000` (`b g`) | `0x0` |
| `return g(a) + 0` (W4b2-vi fold) | `48000000` (`b g`) — **byte-identical to `g(a)`** | `0x0` |
| `return g(a + 1)` (W4b2-iv arg-setup) | `38630001 4bfffffc` (`addi r3,r3,1 ; b g`) | `0x4` |

Fixtures `mvp_tailret.cpp` / `mvp_plus0.cpp` / `mvp_argtail.cpp`, each asserted
`Port=Match` (`differential_mvp_{tailret,plus0,argtail}_*`).

**Modeling (positive parser + codegen).** `parse_call_shape`
(`c2_il::func`) parses the argument region as a modeled sub-expression up to the
`55` call-end (`parse_expr`), then reads the post-op: **no post-op or `+0`** →
`BodyShape::IntTailCall{params, arg_ops}`; **`+k` (k≠0) over a bare `[Load]`
arg** → `FramedCall`; a `+k` over a *computed* arg (`g(a+1)+1`) → reject. This
is a positive redirection, not a special case: *an integer call whose net
post-op is identity is a tail call* — `g(a)+0 == g(a)`, and the optimizer emits
the bare `b g` (so a `FramedCall{add_k:0}` would mis-emit a `.pdata` frame the
reference elides — the W4b2-vi leak, now closed). Codegen
`codegen::int_tail_call_text` reuses `select_text` to compute the argument into
r3 (params → registers, the exact leaf-arithmetic class), drops its trailing
`blr` (the value stays live in r3), and appends the tail branch — so the branch
offset is the arg-setup length (0x0 for passthrough, 0x4 after one `addi`), which
is also the REL24 site. **Scope:** a single argument computed into r3, in the
arithmetic class `select_text` already models (`a+k`/`a-k` → `addi`; `a*b` reg
× reg). Out of scope, fail-closed: multi-argument setup, `k-a` (`subfic`),
constant multiply (strength reduction), a computed arg with a framed post-op.
Non-commutative arg-setup is rejected inside `select_text` — the CLAUDE.md
correctness-boundary discipline holds (no silent operand-order corruption).

**`.pdata` unwind word — RESOLVED: it encodes function length.** The 8-byte
RUNTIME_FUNCTION is `BeginAddress(u32=0, reloc-patched)` + a packed unwind word,
both **big-endian** (like `.text`). Diffing the 0x24-byte `+k` body against the
0x28-byte `*5` body gave `40000903` vs `40000A03` (Δ = 0x100 for +1 word), so:

```
unwind = 0x40000000 | (function_length_words << 8) | prolog_length_words
```

with `function_length_words = text_len/4` and `prolog_length_words = 3` (the
`mflr;stw;stwu` prologue). For the `+k` class the body is always 9 words →
constant `0x40000903`. `build_pdata` computes it from the text length rather than
hardcoding the word. Section characteristics `0x40400040` (CNT_INIT_DATA |
ALIGN_8 | MEM_READ). One reloc: `va=0, symidx=?f, type 0x2 (ADDR32)`. The
`.pdata` aux section-def carries a **real reflected-CRC-32 CheckSum** of its raw
bytes (`0xd3dfb2ce` for the `+k` frame) — a non-COMDAT section that nonetheless
gets a checksum, unlike the leaf `.text`/`.drectve`/`.debug$S` (which store 0).

**Relocations + symbol layout** — see `OBJ_FORMAT_MVP.md` "6-section framed-call
variant". Key facts: 6 sections, 20 symbols; the `bl` REL24 targets the external
`?g` (not a `$M` label); the label symbols are `$M2545` (val=.text+0xC, the `bl`,
class 6), `$M2546` (val=.text end, class 6), `$T2547` (in `.pdata`, class 3); and
the file layout **interleaves** each reloc'd section's raw+reloc (`.text` raw,
`.text` reloc, `.pdata` raw, `.pdata` reloc) rather than packing all raw then all
relocs.

**Counter-determinism (W-UNW-1, CLOSED 2026-07-30):** a single non-leaf function
emits `$M2545 / $M2546 / $T2547`, identical across reruns, filenames and symbol
names — but that is a *fixture* constant, not a toolchain one. The base is the
u32 at `.gl` offset 7 plus 9, and it shifts with the TU's content (an unused
`typedef` moves it by 1) and with preceding functions consuming counter slots.
The framed path is no longer scoped to a single-function TU: the seed is read
and the stride applied per function (`OBJ_GY_SHAPES.md` §3.5/§3.6,
`c2_core::coff::plan_labels`). The labels are no longer hardcoded anywhere.

## Non-commutative hazard list — do NOT generalize the MVP encoder

These are load-bearing operand orders; a swap is a silent, fuzzy-invisible
corruption — exactly the failure class differential testing exists to catch.
Gate each behind explicit opt-in when implemented:

- **`subf` (op 31, XO 40): `subf rD,rA,rB` computes rB − rA — reversed.**
  Probe verified: `a-b-c` → `subf r11,r4,r3; subf r3,r5,r11; blr`
  (minuend→rB, subtrahend→rA). Immediate forms differ again.
- Shifts `slw`/`sraw`/`srw` (XO 24/792/536): fixed operand order;
  signedness selects arithmetic vs logical right shift; by-constant uses
  `rlwinm`.
- Compares `cmpw`/`cmplw`: signed vs unsigned differ; direction not
  swappable; results feed boolean-materialization sequences.
- `mullw` (XO 235) is commutative like `add`.

MVP generator stays restricted to commutative `add` + fixed `blr`.
