# Multi-call framed bodies — the byte-level encyclopedia

Ground-truth characterization of what real `c2` emits for a non-leaf function
that makes **more than one call**, saves callee-saved registers, marshals more
than one argument, or needs stack locals. The port today models exactly one
framed shape (one `bl`, one `+k` consumer, a fixed 96-byte frame,
`CODEGEN_PPC_MVP.md` §W4b2); everything below is the territory that
`docs/GAPS.md` #35 step 2 has to cross.

**Everything here is transcribed from an obj produced by the real toolchain**
(`cl.exe` 16.00.11886.00 under wibo 1.0.1-23), captured with
`scripts/gt_capture.sh` and read with `scripts/gt_dump.py`. Probe sources lived
in a scratch dir and are quoted inline. Flags are `/O1 /GS- /c` unless a row
says otherwise — `/O1` is the workload's mode and it implies `/Gy`, so every
`.text` below is a COMDAT.

> **Mode note, measured not assumed.** For every probe in §1–§5 the
> `.text` bytes at `/O1`, `/Ox` and `/O2` are **identical**; only `/Od` differs.
> Byte evidence: `int f(int a){ int s=0; s+=g(a+1); s+=g(a+2); return s; }`
> gives the same 72-byte body at all three, and 92 bytes of completely
> different code at `/Od`. The whole-grid refutation sweep (§2.5) was run at
> both `/O1` and `/Ox` and produced *the same 39 refutations in the same
> cells*. Nothing in this document keys on the `/O` level.

---

## 1. The frame

### 1.1 Frame layout, measured

Offsets are from the **new** SP (after `stwu r1,-F(r1)`); `F` is the frame size.

```
  SP+0                       back chain (the stwu stores the old SP here)
  SP+8                       reserved (never written by any probe)
  SP+16 .. SP+16+8*nOutSlots outgoing parameter home area, 8 bytes per argument
  align16(16+8*nOutSlots)    addressed locals / compiler temporaries
  ...
  F-8-8*nSaved               saved FPRs (lowest-numbered first), then
  F-16                       saved GPRs (r31 at F-16, r30 at F-24, …)
  F-8                        LR save word (4 bytes, in an 8-byte slot)
```

* `nOutSlots` = **the argument count of the widest call the function makes**,
  with a floor of 8. Slot *k* (1-based) is at `SP + 8 + 8k`; a 4-byte `int`
  lands in the **low** word of its doubleword slot, at `SP + 12 + 8k`.
* The same numbering describes *incoming* arguments relative to the **caller's**
  SP, which is how a 9th incoming `int` is read.

Byte evidence for the slot formula, `int g(int*,int×8); int f(int a){ … g(b,a+1,…,a+8) … }`
(`nOutSlots` = 9, frame 96):

```
   0018  91610054  stw r11,84(r1)        arg9 -> SP+84 = slot(SP+80) + 4
```

and the reader side, `int f(int a1..a9){ return g(a1)+…+g(a9); }` (frame 160):

```
   0084  806100f4  lwz r3,244(r1)        244 - 160 = old SP + 84 = incoming arg9
```

`/Od` gives the identical incoming answer for arg1 (`stw r3,116(r1)` with
`F=96` → old SP + 20 = slot 16 + 4), which is the cross-check that the +8+8k
numbering is the ABI's and not an artifact of one optimizer.

### 1.2 The frame-size rule

> **F = align16( max( 16 + 8·max(nOutSlots, 8), localsBase + localsBytes )
> + 8·nSaved + 8 )**, where `localsBase = align16(16 + 8·max(nOutSlots, 8))`
> and `nSaved` is the total number of saved callee-saved GPRs **plus** FPRs.
> The trailing `+8` is the LR slot.

Every "constant" in the shipped one-call model is a special case of this: the
96-byte frame of `CODEGEN_PPC_MVP.md` is `align16(16 + 64 + 0 + 8) = 96`, and
it is fixed only because that class has ≤8 outgoing args, no locals and no saved
registers.

Two of the three terms are *not* the ones you would guess from the one-call
shape, so both were pinned by a probe designed to refute an alternative:

* **"the 64-byte outgoing area is a floor, not a frame floor."** `int g();`
  with `int b[20]` (80 bytes of locals) and no arguments at all gives
  `stwu r1,-176(r1)` and reads `b[3]` at `lwz r11,92(r1)` — locals at SP+80,
  i.e. *after* a reserved 8-slot parameter area even though the only callee
  takes none. A "frame ≥ 96" model predicts 112 and is refuted by 176.
* **"locals start 16-aligned, not 8-aligned."** With `nOutSlots = 9` the
  parameter area ends at SP+88, yet the local array is at SP+96
  (`addi r3,r1,96` for `&b[0]`, across `int b[1]` … `int b[8]`), and the frame
  steps 112 → 128 → 144 exactly at `4L + 96 + 8` crossing a multiple of 16.
  An 8-aligned model mispredicts 3 of 8 rows.

**"widest call" is measured, and order-independent.** The sweep in §1.3 gives
each body one call, so it cannot tell "widest" from "last" or "first". Two calls
of different arity in one body, in both orders:

```
int g1(int); int g12(int,int,…,int);            /* 12 params */
int f(int a){ return g1(a)  + g12(a,…,a); }     112 B, stwu r1,-144(r1)
int h(int a){ return g12(a,…,a) + g1(a);  }     112 B, stwu r1,-144(r1)
```

Both are `align16(16 + 8·12 + 8·2 + 8) = 144` with `nSaved = 2` — identical
frames and identical sizes, so neither call order nor "the last call" is the
input. It is the maximum.

### 1.3 The refutation sweep

`scripts/gt_frame_sweep.py` generates the cross product of
`nOutSlots ∈ {1,2,3,4,6,8,9,10,11,12,15,21}`,
`localsBytes ∈ {4,8,12,20,28,36,64,132}` and 5 register-pressure levels,
compiles each with the real toolchain, reads `F` out of the `stwu` and `nSaved`
out of the emitted prologue, and compares.

```
mode /O1 /GS- /c: checked 480 framed cases, 39 refutations
mode /Ox /GS- /c: checked 480 framed cases, 39 refutations
```

**Every one of the 39 has `nSaved ≥ 18`** — that is `|r14..r31|`, the whole
callee-saved GPR file. Past that point the allocator spills to stack slots the
sweep's `localsBytes` term does not know about, and the observed frame is 16 to
48 bytes larger. So the honest boundary is:

> The frame rule is exact **while the allocator does not spill** (`nSaved ≤ 17`
> in 441/441 cases at both modes). Once it spills, `F` grows by an unmodeled
> spill area and the rule under-predicts. A step-2 implementation may use the
> rule and must refuse (not guess) once it would need 18 callee-saved GPRs.

---

## 2. Prologues and epilogues — five classes, byte-exact

Which class a function is in is decided by `nGPRsaved` and `nFPRsaved` alone.
`F` is the frame, `V = 0x10000 - F` the `stwu` immediate.

### 2.1 Class A — nothing saved (`nGPRsaved = nFPRsaved = 0`)

3-word prologue, 4-word epilogue. This is the shipped MVP shape.

```
7d8802a6  mflr r12
9181fff8  stw  r12,-8(r1)
9421ffXX  stwu r1,-F(r1)
...
3821XXXX  addi r1,r1,F
8181fff8  lwz  r12,-8(r1)
7d8803a6  mtlr r12
4e800020  blr
```

### 2.2 Class B — 1 or 2 saved GPRs, inline

`std` (a **64-bit** store — this is a 64-bit core) at `-8-8k(r1)`, **before**
the `stwu`, emitted lowest register first. Restores after `mtlr`, same order.

`int g(int); int f(int a,int b){ return g(a)+g(b); }` — 68 B, `F = 112`:

```
7d8802a6  mflr r12
9181fff8  stw  r12,-8(r1)
fbc1ffe8  std  r30,-24(r1)
fbe1fff0  std  r31,-16(r1)
9421ff90  stwu r1,-112(r1)
...
38210070  addi r1,r1,112
8181fff8  lwz  r12,-8(r1)
7d8803a6  mtlr r12
ebc1ffe8  ld   r30,-24(r1)
ebe1fff0  ld   r31,-16(r1)
4e800020  blr
```

One saved GPR is the same with a single `std r31,-16(r1)` / `ld r31,-16(r1)`
(`int f(int a){ return g(a)+a; }`, 48 B, `F = 96`).

### 2.3 Class C — 3 or more saved GPRs: the `__savegprlr_N` helpers

**The threshold is 3.** At 3 the prologue collapses to three words and the
epilogue becomes a **tail branch — there is no `blr` at all**.

`int f(int a,int b,int c){ return g(a)+g(b)+g(c); }` — 60 B, `F = 112`, saves
r29–r31:

```
7d8802a6  mflr r12
4bfffffd  bl   __savegprlr_29        REL24 at .text+0x4
9421ff90  stwu r1,-112(r1)
...
38210070  addi r1,r1,112
4bffffc8  b    __restgprlr_29        REL24 at .text+0x38, LK=0
```

`N = 32 - nGPRsaved`. Observed: `_29` (3), `_28` (4), `_27`(3+FPR case), `_26`
(6), `_25` (7), `_24` (8). The helper also saves/restores LR — `mflr r12` still
runs first because the helper expects LR in r12.

### 2.4 Class D/E — saved FPRs

**The FPR threshold is 4, not 3** — it is a different number from the GPR
threshold and nothing in the obj explains why.

1–3 saved FPRs are inline `stfd`/`lfd`, in the same slots below the GPR area.
`double f(double a,double b,double c){ return g(a)+g(b)+g(c); }` — 92 B,
`F = 112`:

```
7d8802a6  mflr r12
9181fff8  stw  r12,-8(r1)
dba1ffe0  stfd f29,-32(r1)
dbc1ffe8  stfd f30,-24(r1)
dbe1fff0  stfd f31,-16(r1)
9421ff90  stwu r1,-112(r1)
...
38210070  addi r1,r1,112
8181fff8  lwz  r12,-8(r1)
7d8803a6  mtlr r12
cba1ffe0  lfd  f29,-32(r1)
cbc1ffe8  lfd  f30,-24(r1)
cbe1fff0  lfd  f31,-16(r1)
4e800020  blr
```

4+ saved FPRs use `__savefpr_M` / `__restfpr_M` (`M = 32 - nFPRsaved`), which
take their **base pointer in r12**:

```
7d8802a6  mflr r12
9181fff8  stw  r12,-8(r1)
3981fff8  addi r12,r1,-8            base = -(8 + 8*nGPRsaved)
4bfffff5  bl   __savefpr_28
9421ff80  stwu r1,-128(r1)
...
38210080  addi r1,r1,128
3981fff8  addi r12,r1,-8
4bffffad  bl   __restfpr_28          a CALL, not a tail branch
8181fff8  lwz  r12,-8(r1)
7d8803a6  mtlr r12
4e800020  blr
```

The `addi r12,r1,-(8 + 8·nGPRsaved)` is the seam between the two save areas —
verified at 0 GPRs (`-8`), 2 GPRs (`-24`) and 3 GPRs (`-32`).

### 2.5 Class F — both helpers

`double f(int a,int b,int c,int d,double w,double x,double y,double z){…}` —
3 saved GPRs + 5 saved FPRs, `F = 160`. The GPR helper runs first (it consumes
r12's LR), then r12 is reloaded as the FPR base:

```
7d8802a6  mflr r12
4bfffffd  bl   __savegprlr_29
3981ffe0  addi r12,r1,-32
4bfffff5  bl   __savefpr_27
9421ff60  stwu r1,-160(r1)
...
382100a0  addi r1,r1,160
3981ffe0  addi r12,r1,-32
4bffff31  bl   __restfpr_27
4bffff2c  b    __restgprlr_29
```

### 2.6 `.pdata` and `$M` agree with the prologue length, in every class

The unwind word stays `0x40000000 | (len_words << 8) | prolog_words`, and
`$M(n)` (the low label) is the prologue end in bytes:

| class | probe | prologue words | `$M(n)` | `.pdata` |
|---|---|---|---|---|
| A | `g(a)+1` | 3 | `0x0c` | `40000903` |
| B (2 GPR) | `g(a)+g(b)` | 5 | `0x14` | `40001105` |
| C (3 GPR) | `g(a)+g(b)+g(c)` | 3 | `0x0c` | `40000f03` |
| C (7 GPR) | 7 live params | 3 | `0x0c` | `40001f03` |
| D (3 FPR) | 3 live doubles | 6 | `0x18` | `40001706` |
| D (1 FPR) | `g(a)*2.5f+a` | 4 | `0x10` | `40000e04` |
| E (4 FPR) | 4 live doubles | 5 | — | `40001905` |
| F (2 GPR + FPR helper) | `mx2` | 7 | — | `40003007` |
| F (GPR + FPR helpers) | `mx3` | 5 | — | `40003605` |

`prolog_words` counts the `stwu` and everything before it, including both
helper `bl`s and the `addi r12`.

---

## 3. Register discipline across calls

### 3.1 Callee-saved GPRs are allocated **descending from r31**

`int f(int a1..a8){ return g(a1)+…+g(a8); }`:

```
   000c  7c9f2378  mr r31,r4        a2
   0010  7cbe2b78  mr r30,r5        a3
   0014  7cdd3378  mr r29,r6        a4
   0018  7cfc3b78  mr r28,r7        a5
   001c  7d1b4378  mr r27,r8        a6
   0020  7d3a4b78  mr r26,r9        a7
   0024  7d595378  mr r25,r10       a8
   0028  4bffffd9  bl ?g
   002c  7c781b78  mr r24,r3        first call RESULT -> the next free reg
```

Parameters that must survive a call are copied first, in **parameter order**,
into r31, r30, r29, …; the first argument is left in r3 because the first call
consumes it immediately. Call **results** take the next descending register
after the parameters. The allocator reuses a register the moment its value dies:
in the 3-parameter case the accumulator moves into r31 (`add r31,r29,r3`) as
soon as a2's last use has passed.

Registers are the *only* home until r14 is reached; the 9th live value in
`f(int a1..a9)` is not spilled but re-read from its incoming stack slot
(`lwz r3,244(r1)`).

### 3.2 Argument marshalling: descending destination order, r11 breaks cycles

Non-conflicting moves are emitted **highest destination register first**:

```
int g4(int,int,int,int); int f(int a){ return g4(a,a,a,a)+1; }
   000c  7c661b78  mr r6,r3
   0010  7c651b78  mr r5,r3
   0014  7c641b78  mr r4,r3
```

A permutation is broken with **r11** as the scratch, saving the value destined
for r3 first and then walking the chain:

```
int g3(int,int,int); int f(int a,int b,int c){ return g3(c,a,b)+1; }
   000c  7cab2b78  mr r11,r5      c
   0010  7c852378  mr r5,r4       b
   0014  7c641b78  mr r4,r3       a
   0018  7d635b78  mr r3,r11      c
```

`g2(b,a)` and `g3(b,c,a)` produce the same three-step shape. r11 is the same
scratch the leaf selector uses (`CODEGEN_PPC_MVP.md` "COLOR scratch order").

### 3.3 The multi-call accumulator shape

`int f(int a){ int s=0; s+=g(a+1); … s+=g(a+n); return s; }` is exactly regular
for n = 2…6 — the body grows by 12 bytes per call and nothing else changes:

```
   0014  7c7f1b78  mr   r31,r3          a, live across every call
   0018  38630001  addi r3,r3,1
   001c  4bffffe5  bl   ?g
   0020  7c7e1b78  mr   r30,r3          first result
   0024  387f0002  addi r3,r31,2
   0028  4bffffd9  bl   ?g
   002c  7fc3f214  add  r30,r3,r30      accumulate in r30
   ...
   00XX  7c63f214  add  r3,r3,r30       last accumulate targets r3
```

Note the **operand order flips on the last accumulation** (`add r3,r3,r30`
against `add r30,r3,r30` for the intermediate ones) — the destination changes,
not the operands, and `add` is commutative so this is not a hazard here. It
would be for `subf`.

---

## 4. Symbol-table order — three rules the current template does not have

`OBJ_GY_SHAPES.md` §3.3 gives the per-function group as
`[.text sym+aux] [fn] [$M(n+1)] [callee external] [$M(n)] [.pdata sym+aux] [$T]`.
Three refinements, each refutation-tested:

### 4.1 Multiple new callees are emitted in **reverse first-reference order**

```
int f(int a){ return g1(a)+g2(a)+g3(a); }   ->  [15] ?g3  [16] ?g2  [17] ?g1
int f(int a){ return g3(a)+g2(a)+g1(a); }   ->  [15] ?g1  [16] ?g2  [17] ?g3
```

The second TU is the refutation of "alphabetical" and of "declaration order":
only reverse *reference* order fits both. This is the same LIFO the `.rdata`
constant pool uses within one function (`OBJ_GY_SHAPES.md` §2.3), and it has the
same failure mode — a naive append emits every index swapped, and every
relocation still *resolves*.

A callee already introduced by an earlier function is not re-emitted:
`f{g1,g2}` then `h{g2,g1}` gives f the pair `[15] ?g2 [16] ?g1` and h nothing.

### 4.2 A function's `.rdata` pairs precede its callee externals

`float f(float a){ return g(a)*2.5f + g(a)*4.5f; }`:

```
  [13] ?f                     [14] $M2554 (end)
  [15] .rdata+aux  [17] __real@40900000      (4.5f — referenced second)
  [18] .rdata+aux  [20] __real@40200000      (2.5f — referenced first)
  [21] ?g                                     the callee, AFTER both pools
  [22] $M2553 (prologue)  [23] .pdata+aux  [25] $T2555  [26] _fltused
```

### 4.3 `__savegprlr_N` / `__restgprlr_N` are emitted **after the whole group**

They are not inside the callee-external region. They land after the `$T`, and
**`rest` precedes `save`** even though `save` is referenced first:

```
  [17] .pdata+aux   [19] $T2563
  [20] __restgprlr_29
  [21] __savegprlr_29
  [22] .text (the NEXT function's section symbol)
```

Two framed functions needing different helper widths each get their own pair
after their own group (`__rest/__savegprlr_29` after f1, `__rest/__savegprlr_28`
after f2). `_fltused` occupies the same position for the first FP function
(`OBJ_GY_SHAPES.md` §1.2), which is consistent: both are TU-level externals
attached to the tail of the group that introduced them.

### 4.4 The `/Gy` label stride for a helper-using framed function is **7, not 5**

This **refutes** `OBJ_GY_SHAPES.md` §3.5's `framed -> cur += 5 if /Gy`. Using
that document's own seed rule (`B = u32(.gl[7..11]) + 9`, then `+3` per
function):

| TU | seed B | `+3·nfunc` | predicted first `$M` | observed |
|---|---|---|---|---|
| 1 framed, 2 saved GPRs (inline) | 2547 | 2550 | 2550 | **2550** ✓ |
| 1 framed, 3 callees, inline | 2549 | 2552 | 2552 | **2552** ✓ |
| 2 framed, both inline | 2550 | 2556 | 2556 / 2561 | **2556 / 2561** ✓ |
| 1 framed, `__savegprlr_29` | 2547 | 2550 | 2550 | **2552** ✗ (+2) |
| 1 framed, `__savegprlr_25` | 2551 | 2554 | 2554 | **2556** ✗ (+2) |
| framed(inline) then framed(helper) | 2550 | 2556 | 2556 / 2561 | **2556 / 2563** ✗ |
| framed(helper) ×2 | 2553 | 2559 | 2559 / 2564 | **2561 / 2568** ✗ |

> **A framed function that uses the `__savegprlr_N`/`__restgprlr_N` pair
> consumes two extra label slots, allocated *before* its own `$M` pair** — its
> first label is `cur + 2` and its stride is 7 under `/Gy`.

Fits all seven rows. It is *not* "one per introduced external" in general: the
three-callee TU introduces three externals and consumes the plain 5. The
natural reading is one slot per *helper* external, which predicts +4 for a
function using both the GPR and the FPR helper pair — **not captured, and
therefore not claimed.** A TU pairing an FPR-helper function with a following
function would separate them in one capture; that is the next probe if step 2
touches this.

The existing `plan_labels` gate refuses these bodies for other reasons today, so
this is a latent rather than a live wrong byte. It becomes live the moment a
framed function with ≥3 saved GPRs is admitted.

---

## 5. `.rdata` beside `.pdata` in one function — RESOLVED

The open question in `docs/GAPS.md` (no captured TU had a pooled FP constant and
a framed function *in the same function*, so the section order was unknown and
the port refused). `float g(float); float f(float a){ return g(a)*2.5f + a; }`:

| # | name | RawSize | Chars | aux |
|---|---|---|---|---|
| 5 | `.text` | 56 | `0x60401020` | len=56 nrel=5 cksum=0 Sel=1 |
| 6 | `.rdata` | 4 | `0x40301040` | len=4 cksum=0 Sel=2 |
| 7 | `.pdata` | 8 | `0x40401040` | len=8 cksum=`0x02fab1aa` **Number=5** Sel=5 |

> **Order within one function's group is `.text`, then every `.rdata` it
> introduces, then its `.pdata`.** The `.pdata` aux `Number` is still the
> 1-based section index of *its own* `.text`, counted through the intervening
> `.rdata` sections — verified at Number=5 with one `.rdata` between, Number=7
> in a TU where a leading float leaf pushed the framed function to section 7
> (sections `.text(k) .rdata .text(f) .rdata .pdata`, Number=7), and Number=5
> with **two** `.rdata` sections between (§4.2's probe).

The body, showing that the constant load lands *after* the call and that the
frame is Class D with one saved FPR (`F = align16(80 + 8 + 8) = 96`):

```
7d8802a6  mflr r12
9181fff8  stw  r12,-8(r1)
dbe1fff0  stfd f31,-16(r1)
9421ffa0  stwu r1,-96(r1)
ffe00890  fmr  f31,f1
4bffffed  bl   ?g                                 REL24  .text+0x14
3d600000  lis  r11,0        REFHI __real@40200000 + PAIR
c00b0000  lfs  f0,0(r11)    REFLO __real@40200000 + PAIR
ec21f83a  fmadds f1,f1,f0,f31
38210060  addi r1,r1,96
8181fff8  lwz  r12,-8(r1)
7d8803a6  mtlr r12
cbe1fff0  lfd  f31,-16(r1)
4e800020  blr
```

Symbol order is §4.2's: `.text`+aux, `?f`, `$M(end)`, `.rdata`+aux,
`__real@40200000`, `?g`, `$M(prologue)`, `.pdata`+aux, `$T`, `_fltused`.

---

## 6. What refused to yield a rule

* **Which values become callee-saved, and in what order.** §3.1 describes the
  discipline (descending from r31, parameters in order then results, reuse on
  death) but that is a *description of an allocator*, not a closed-form rule,
  and `nSaved` is an input to the frame formula. Every claim in §1 is
  conditional on `nSaved` being known; the sweep reads it out of the emitted
  code rather than predicting it. **A step-2 implementation needs a liveness
  pass before it needs the frame formula, and the frame formula is the easy
  half.**
* **The spill regime.** Past 18 saved GPRs the frame grows by an amount this
  measurement does not model (39/480 cases, §1.3). Not chased — the workload's
  calls-0 population is nowhere near that pressure.
* **Why the GPR helper threshold is 3 and the FPR helper threshold is 4.**
  Both are stable across every probe; neither is a code-size break-even (the
  helper is already shorter at 3 saved FPRs and c2 still emits inline `stfd`).
  Treated as two constants.
* **What the extra label slots are for.** §4.4 measures +2 for the GPR helper
  pair; the underlying counter is invisible in the obj, exactly like the 4th
  packed / 5th `/Gy` slot `OBJ_GY_SHAPES.md` §3.6 already records as unexplained.
* **The FPR-helper label stride.** Predicted +4 by the "one slot per helper
  external" reading, not captured. Explicitly not claimed.
* **The reserved 8 bytes at SP+8.** No probe ever wrote or read them. Every
  frame reserves them and the parameter area starts at +16.

---

## 6b. No live mis-emit found

Every probe TU built for this document — 87 of them, covering all five
prologue classes, the argument-marshalling cycles, the struct and FP edges, and
the EH shapes — was run through the port:

```
GAP REPORT (87 TUs)
  match 1   mismatch 0   codegen-gap 1   vocab-gap 85   port-error 0
```

**Zero mismatches.** The port refuses this whole territory rather than guessing
at it, including every case where a rule in this document contradicts a shipped
one. The §4.4 stride refutation is therefore latent, not live.

## 7. Suggested rung order for #35 step 2

Sized by what each rung needs that the previous one did not:

1. **Class A, many calls, no saved registers** — `void`/discarded-result call
   sequences. Needs nothing new: the 96-byte frame, the shipped prologue, one
   REL24 per call, and the `.pdata` word already computed from the text length.
   The symbol-order rules of §4.1 are needed the moment a body calls two
   different functions, and they are cheap.
2. **Argument marshalling** (§3.2). Self-contained, no frame change while the
   widest call has ≤8 arguments, and the cycle-breaking rule is three lines.
   Do this before anything that saves registers.
3. **`nOutSlots > 8` and addressed locals** (§1.2). Frame arithmetic only; the
   prologue is unchanged. This is where the `align16` terms start to matter and
   where a wrong model is a wrong `stwu` immediate — one byte, silent.
4. **Class B (1–2 saved GPRs)** (§2.2). Needs the liveness answer, which is the
   real cost of the whole step. Stop here for a first cut: it covers "a call
   result live across another call" with a fixed, two-instruction prologue
   delta.
5. **Class C (≥3 saved GPRs)** (§2.3) — needs the helper externals, the tail-branch
   epilogue *and* the +2 label stride of §4.4 at the same time. It is the first
   rung where getting the symbol table right is as much work as getting the code
   right.
6. Classes D/E/F (FPRs) last: they add a second save area, a second threshold,
   and an unmeasured label stride.
