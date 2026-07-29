# CODEGEN W13 — float / double codegen (characterization)

Roadmap step **W13**. Measured demand: the operand-type buckets
`expr-load-type-864540` (float, 3.4 % of blocked functions) and
`expr-load-type-888541` (double, 3.1 %) — 6.5 % together, comparable to the
largest single schedulable bucket. `crates/c2-il/src/func.rs::parse_expr`
accepts only `INT_TYPE = 86 41 74` on a `B9` LOAD, so every FP body blocks at
its first operand.

**This document is characterization only — no Rust was changed.** Every
register, opcode and section claim below is backed by bytes read out of an obj
produced by the real toolchain (`cl.exe` 16.00.11886.00 under wibo, the harness'
standard `/Ox /GS- /c`), and every instruction word cited in §3 is re-derived
from its bit fields and checked against the observed word (30/30 exact).
Nothing here comes from `paint/`.

New fixtures (source only; the IL and objs are regenerated, never committed):

```
fixtures/cpp/w13_fabi.cpp     -> ReferenceReplay=ByteExact (ref=1268B replay=1268B)  Port=NotImplemented
fixtures/cpp/w13_fops.cpp     -> ReferenceReplay=ByteExact (ref=1363B replay=1363B)  Port=NotImplemented
fixtures/cpp/w13_fscratch.cpp -> ReferenceReplay=ByteExact (ref=1498B replay=1498B)  Port=NotImplemented
fixtures/cpp/w13_fneg.cpp     -> ReferenceReplay=ByteExact (ref=3144B replay=3144B)  Port=NotImplemented
fixtures/cpp/mvp_fmul3.cpp    -> ReferenceReplay=ByteExact (ref= 861B replay= 861B)  Port=NotImplemented
```

Byte offsets quoted below are `.text` offsets inside those four fixtures'
reference objs unless stated otherwise.

---

## 0. The IL side (what `parse_expr` has to learn)

The float/double leaf segment has **exactly the same shape** as the existing
straight-line integer segment — only the TYPE bytes change. `f_add` from
`w13_fops.cpp`:

```
4c 4f 11 53
b9 e3 09 86 45 40    LOAD a  : float
b9 e4 09 86 45 40    LOAD b  : float
02                   ADD
41 86 45 40          result type : float
3a e6 09             assign
54 02 29 e6 09       return
```

and `d_add`, byte-for-byte the same with `88 85 41` in place of `86 45 40`.

| element | float | double | int (existing) |
|---|---|---|---|
| operand TYPE (`B9`/`41`) | `86 45 40` | `88 85 41` | `86 41 74` |
| literal TYPE (`33`) | `86 4a 40` | `88 8a 41` | `86 41 74` |

Note the **literal type is not the operand type** for FP: the `kind` byte is
`+5` (`0x45`→`0x4a`, `0x85`→`0x8a`). For `int` the two coincide, which is why
the current single-`INT_TYPE` model never noticed.

Operator bytes (`w13_fops.cpp`, one function per opcode — the segments differ
only in this byte):

| byte | op | fixture |
|---|---|---|
| `02` | ADD | `f_add`, `d_add` |
| `03` | SUB | `f_sub`, `d_sub` |
| `04` | MUL | `f_mul`, `d_mul` |
| `05` | DIV | `f_div`, `d_div` — **new**, no integer precedent in the parser |
| `08` | unary NEG | `f_neg`, `d_neg` |
| `2c <TYPE> 00` | CONVERT | `n_i2f`: `b9 29 0a 86 41 74 2c 86 45 40 00` |

### 0.1 The `0x59` marker — FP-only, source-parenthesis-shaped

An extra byte `59` appears **after an FP binary operator whose result is a
parenthesized sub-expression operand of another operator**. It never appears in
integer IL: `w5_tree2.cpp`'s `(a+b)*(c+d)` `.ex` contains zero `0x59` bytes,
while `w13_fscratch.cpp::ft2` (the same source in float) contains two:

```
ft2 (float):  b9 09 0a 86 45 40  b9 0a 0a 86 45 40  02 >59<
              b9 0b 0a 86 45 40  b9 0c 0a 86 45 40  02 >59<  04
t2_mul_add (int): b9 e3 09 86 41 74 b9 e4 09 86 41 74 02
                  b9 e5 09 86 41 74 b9 e6 09 86 41 74 02  04
```

Controlled probes (`/tmp` scratch, five one-line functions) pin it to source
parentheses, not to tree shape:

| source | `.ex` operator stream |
|---|---|
| `(a+b)*c` | `02 59 … 04` |
| `a*(b+c)` | `… 02 59 04` |
| `a+b*c`   | `04 02` — **no** `59` |
| `(a*b)*c` | `04 59 … 04` |
| `a-(b-c)` | `03 59 03` |

`(a*b)*c` and `a*b*c` compile to **identical** `.text`
(`ec0100b2 ec2000f2 4e800020` in both `p_59::q4` and `w13_fscratch::fm3`), so a
`59` on a left-leaning chain is output-neutral. It is **not** output-neutral in
general — see §2.4 (it is the only observable difference between the shapes that
do and do not get product-flattened). **UNKNOWN:** the exact semantics. The
parser must consume it; the selector must not ignore it.

---

## 1. The FP calling convention (as observed)

> **Float and double use the same registers and the same rule.** Parameters go
> in **f1…f13**, numbered by *floating-point-parameter order* — not by
> positional slot. The result comes back in **f1** for both widths. A float
> parameter still consumes its positional **GPR home slot**.

All bytes from `fixtures/cpp/w13_fabi.cpp`:

```
?fp_pass1@@YAMM@Z          float f(float a){return a;}
 0000 4e800020   blr                       ; a is already in f1
 0004 00000000                             ; 8-byte function alignment pad
?fp_pass2@@YAMMM@Z         float f(float a,float b){return b;}
 0008 fc201090   fmr   f1,f2
?dp_pass2@@YANNN@Z         double, identical bytes
 0010 fc201090   fmr   f1,f2
?dp_pass3@@YANNNN@Z
 0018 fc201890   fmr   f1,f3
?fp_skip@@YAMMHM@Z         float f(float a,int b,float c){return c;}
 0020 fc201090   fmr   f1,f2               ; c is the 2nd FLOAT -> f2, not f3
?fp_nine@@YAMMMMMMMMMM@Z   9 floats, return the 9th
 0028 fc204890   fmr   f1,f9               ; FPRs go deeper than the 8 GPRs
?dp_thirteen@@YANN…@Z      13 doubles, return the 13th
 0030 fc206890   fmr   f1,f13              ; f13 is the last register slot
?dp_fourteen@@YANN…@Z      14 doubles, return the 14th
 0038 c8210078   lfd   f1,120(r1)          ; the 14th spills to the home area
?ip_after_floats@@YAHMMMMMMMMH@Z   8 floats then int z
 0040 80610054   lwz   r3,84(r1)           ; z is positional #9 -> stack
```

* `fp_skip` is the discriminator for "float order, not positional order":
  `c` is positional parameter 3 but float parameter 2, and it is in **f2**.
* `ip_after_floats` is the discriminator for "float still burns a GPR slot":
  `z` is positional parameter 9, so it is off the stack even though `r3…r10`
  are untouched by the eight float parameters.
* **Home-slot formula (verified for both classes):** positional parameter *n*
  homes at `r1 + 8 + 8n`. `dp_fourteen` reads `120(r1)` = 8+8·14 ✓;
  `ip_after_floats` reads `84(r1)`, i.e. the **low word** of the 8-byte slot at
  `80` = 8+8·9 ✓ (big-endian, a 4-byte value is right-aligned in its
  doubleword). A stack float is `lfs f1,140(r1)` = slot 8+8·16 = 136, +4 for the
  right-aligned float (`p_param::f_p16`, 16 float parameters).
* Function alignment in `.text` is **8 bytes**, padded with `00000000`. This is
  *not* float-specific — an integer TU pads identically (`p_align::q1`:
  `4e800020 00000000`).

**UNKNOWN:** varargs (whether a float argument is also duplicated into a GPR),
struct-by-value containing floats, and `long double`. Not probed.

---

## 2. The FP temporary allocator

> **The FP scratch order is NOT the integer rotating cursor of
> `docs/CODEGEN_W5_SCRATCH.md`.** It is a rotating **descending** cursor over
> the pool
> `[f0, f13, f12, f11, f10, f9, f8, f7, f6, f5, f4, f3, f2, f1]` —
> **f0 first**, then down from **f13**, wrapping back to f0 — advancing one slot
> per emitted value and **skipping any register that still holds a live value**.
> The function result always lands in **f1**.

Differences from the integer rule, stated explicitly because they must not be
assumed to match:

| | integer (W5) | floating point (W13) |
|---|---|---|
| pool | `[r11,r10,…,r4,r3]`, 9 regs; `r12`, `r2`, `r0` never allocated | `[f0,f13,f12,…,f2,f1]`, **14** regs; **f0 is allocatable and is the cursor's first slot** |
| top of pool | `r11` | `f0`, *then* f13 |
| result register | `r3` (also the last pool slot) | `f1` (also the last pool slot) |
| additive chain | collapses to **one** accumulator (`add r11,r11,r5`) | **does not collapse** — every intermediate is a distinct value |
| spill target | `r31` + `std`/`ld` red zone | `f31,f30,f29` + `stfd`/`lfd` red zone |

### 2.1 Evidence — the pure descent (`w13_fscratch::fm6`, 6 params)

```
?fm6@@YAMMMMMMM@Z          a*b*c*d*e*f
 0038 ec0100b2   fmuls f0,f1,f2      -> f0    (cursor slot 0)
 003c eda000f2   fmuls f13,f0,f3     -> f13
 0040 ed8d0132   fmuls f12,f13,f4    -> f12
 0044 ed6c0172   fmuls f11,f12,f5    -> f11
 0048 ec2b01b2   fmuls f1,f11,f6     -> f1    (result)
```

`fm3`/`fm4`/`fm5` are the same descent truncated. `fs4` (`a-b-c-d`) and `fa4`
(`a+b+c+d`) descend identically — `f0, f13, f1` — which is the second
non-obvious fact: **an FP `+` chain does not collapse to a single accumulator**,
unlike the integer `add` chain (`w5_chain::c4_add` → `add r11,r11,r5`).

```
?fa4@@YAMMMMM@Z            a+b+c+d
 0060 ec01102a   fadds f0,f1,f2
 0064 eda0182a   fadds f13,f0,f3     ; NOT `fadds f0,f0,f3`
 0068 ec2d202a   fadds f1,f13,f4
```

### 2.2 Evidence — liveness skip and the wrap (`w13_fscratch::fm13`)

13 parameters occupy f1…f13; only f0 is free at entry. Twelve consecutive
allocations, two wraps, ten skips:

| # | bytes | cursor scan | dest |
|---|---|---|---|
| 1 | `ec0100b2` `fmuls f0,f1,f2` | f0 free | **f0** |
| 2 | `ec6000f2` `fmuls f3,f0,f3` | f13…f4 all live; f3 = `a3`, **last read is this instruction** | **f3** |
| 3 | `ec430132` `fmuls f2,f3,f4` | f2 = `a2`, dead since #1 | **f2** |
| 4 | `ec220172` `fmuls f1,f2,f5` | f1 = `a1`, dead since #1 | **f1** |
| 5 | `ec0101b2` `fmuls f0,f1,f6` | cursor **wraps** past f1 to f0; t1 died at #2 | **f0** |
| 6 | `ece001f2` `fmuls f7,f0,f7` | f13…f8 live; f7 = `a7` dies here | **f7** |
| 7 | `ecc70232` `fmuls f6,f7,f8` | f6 = `a6`, dead since #5 | **f6** |
| 8–11 | `eca60272 ec8502b2 ec6402f2 ec430332` | plain descent | **f5, f4, f3, f2** |
| 12 | `ec220372` `fmuls f1,f2,f13` | result | f1 |

This single function pins everything the integer document pins for GPRs:
the pool's membership (f0 *and* f1/f2/f3 are ordinary temps when dead), the
descending order, the wrap point, the liveness gate, and the
"a source dying at *this* instruction is available as this instruction's
destination" rule (`fmuls f3,f0,f3`).

The single-skip case is `w13_fscratch::fskip` (`(a*b)*(c*d)*m`, 13 params):

```
 0100 ec0100b2   fmuls f0,f1,f2      -> f0
 0104 ed830132   fmuls f12,f3,f4     -> f12  (cursor f13 skipped: `m` is LIVE)
 0108 ed600332   fmuls f11,f0,f12    -> f11
 010c ec2b0372   fmuls f1,f11,f13    -> f1
```

### 2.3 The pool's bottom — spilling (`w13_fneg::n_spill`)

Beyond 14 simultaneously live FP values c2 saves **non-volatile** FPRs into the
red zone, with **no frame** (`r1` is not moved) and a `.pdata` entry:

```
 0128 dba1ffe8   stfd f29,-24(r1)
 012c dbc1fff0   stfd f30,-16(r1)
 0130 dbe1fff8   stfd f31,-8(r1)
 …    (f31, f30, f29 used as temps: `efe6382a fadds f31,f6,f7` …)
 0194 cba1ffe8   lfd  f29,-24(r1)
 0198 cbc1fff0   lfd  f30,-16(r1)
 019c cbe1fff8   lfd  f31,-8(r1)
 01a0 4e800020   blr
```

So the volatile FP pool is exactly **f0…f13 (14 registers)** and the spill order
is **f31, f30, f29** (descending, saved lowest-numbered-first at the most
negative offset). This is the FP mirror of W5's N4 and gives the same style of
gate: reject when peak simultaneous live FP values > 14.

### 2.4 Canonicalization: a flat FP chain is re-linearized in parameter order

`w13_fscratch::fm13` and `fm13r` differ only in that `fm13r` writes the thirteen
factors in **reverse**. Their `.ex` streams really are reversed
(`fm13`: `b9 15 0a … b9 16 0a … 04 …`; `fm13r`: `b9 30 0a … b9 2f 0a … 04 …`,
tokens descending), and their `.text` is **byte-identical**, starting
`ec0100b2 fmuls f0,f1,f2` = `a1*a2` in both. Four more probes agree:
`a*c*b`, `c*a*b`, `a+c+b`, `c+a+b`, `b*a*d*c` all produce the canonical
ascending-parameter chain. **c2 sorts the terms of a flat commutative FP chain
by parameter order.** A `-` chain is *not* reordered (`fs4`: `a-b-c-d` keeps
source order).

But a **parenthesized** product is *not* flattened, unlike the integer case
(W5 N1, where `(a*b)*(c*d)` became a chain). `fskip`'s `(a*b)*(c*d)*m` keeps its
tree (`fmuls f0,f1,f2 ; fmuls f12,f3,f4 ; fmuls f11,f0,f12 ; fmuls f1,f11,f13`).
The only IL difference between the flattened and non-flattened shapes is the
`0x59` of §0.1.

### 2.5 Operand order at a node

`w13_fscratch::ft2` (`(a+b)*(c+d)`, and `dt2` the double twin):

```
 0070 ec01102a   fadds f0,f1,f2       ; left  = a+b -> f0
 0074 eda3202a   fadds f13,f3,f4      ; right = c+d -> f13
 0078 ec200372   fmuls f1,f0,f13      ; rA = left, rC = right
```

Left child → the earlier (higher-priority) cursor slot, right child → the next.
`fsubs` is **already in source order** — `fsubs fD,fA,fB` computes `fA − fB`,
the opposite of the integer `subf` reversal:

```
?fs4  a-b-c-d
 0050 ec011028   fsubs f0,f1,f2       ; f1 - f2 = a-b     (rA = minuend)
 0054 eda01828   fsubs f13,f0,f3
 0058 ec2d2028   fsubs f1,f13,f4
```

Where the two children need different numbers of registers, the heavier is
ranked first, as in W5 §3.3 — `p_59::q2` (`a*(b+c)`) emits the sum first
(`ec02182a fadds f0,f2,f3`) and then `ec200072 fmuls f1,f0,f1`.

### 2.6 What is NOT explained

**The interaction between the cursor and FP constant loads is unresolved.**
The single-constant shapes are consistent (§5.3), but `p_const2::k4`
(`(a+1.0f)*(b+2.0f)*(c+3.0f)`) allocates
`f0(c1), f13(c2), f12(t1), f11(t2), f0(c3 — reused), f10(t3), f9(t4), f1` —
the third constant load takes a *dead* register out of cursor order. No model
tried here reproduces both `k4` and `k_two`/`k5`. Constants are therefore a hard
reject for W13a (§6, N3/N4).

---

## 3. Per-operation encodings

Every word below was observed in a fixture obj **and** re-derived from its bit
fields; the derivation script checks 30/30 exact. Field layout:

* **A-form** (arithmetic, primary opcode **59** = single / **63** = double):
  `op | D<<21 | A<<16 | B<<11 | C<<6 | XO<<1 | Rc`.
  `add`/`sub`/`div` use **A** and **B** (C = 0); `mul` uses **A** and **C**
  (B = 0); the fused forms use all three (`fD = fA*fC ± fB`).
* **X-form** (opcode 63): `63 | D<<21 | 0<<16 | B<<11 | XO<<1 | Rc`.
* **D-form** (loads/stores): `op | D<<21 | A<<16 | disp16`.

### 3.1 The four binary operations (`fixtures/cpp/w13_fops.cpp`)

| source | instruction | XO | fields (D,A,B,C) | word | offset |
|---|---|---|---|---|---|
| `float a+b` | `fadds f1,f1,f2` | 21 | 1,1,2,0 | `ec21102a` | 0x00 |
| `float a-b` | `fsubs f1,f1,f2` | 20 | 1,1,2,0 | `ec211028` | 0x08 |
| `float a*b` | `fmuls f1,f1,f2` | 25 | 1,1,**0**,**2** | `ec2100b2` | 0x10 |
| `float a/b` | `fdivs f1,f1,f2` | 18 | 1,1,2,0 | `ec211024` | 0x18 |
| `double a+b` | `fadd f1,f1,f2` | 21 | 1,1,2,0 | `fc21102a` | 0x20 |
| `double a-b` | `fsub f1,f1,f2` | 20 | 1,1,2,0 | `fc211028` | 0x28 |
| `double a*b` | `fmul f1,f1,f2` | 25 | 1,1,**0**,**2** | `fc2100b2` | 0x30 |
| `double a/b` | `fdiv f1,f1,f2` | 18 | 1,1,2,0 | `fc211024` | 0x38 |

**The single/double pair differs in exactly one bit of the primary opcode**
(59 = `0b111011` vs 63 = `0b111111`, i.e. `0xEC…` vs `0xFC…`); the XO and all
register fields are identical. `fmul`'s multiplier is in the **C** field, not
**B** — a `fmuls fD,fA,fB` encoder written by analogy with `fadds` emits a
multiply by f0 and is silently wrong.

Non-`f1` destinations, from `w13_fscratch`:
`ec0100b2 fmuls f0,f1,f2`, `eda000f2 fmuls f13,f0,f3`,
`eda3202a fadds f13,f3,f4`, `ec200372 fmuls f1,f0,f13` — all re-derived exact.

### 3.2 Unary / move / convert

| instruction | form | XO | fields | word | fixture site |
|---|---|---|---|---|---|
| `fneg f1,f1` | X-63 | 40 | D=1,B=1 | `fc200850` | `w13_fops` 0x40 (float) **and** 0x48 (double) — same bytes |
| `fmr f1,f2` | X-63 | 72 | D=1,B=2 | `fc201090` | `w13_fabi` 0x08 |
| `fmr f1,f13` | X-63 | 72 | D=1,B=13 | `fc206890` | `w13_fabi` 0x30 |
| `frsp f1,f1` | X-63 | 12 | D=1,B=1 | `fc200818` | `w13_fops` 0x50 |

* **`double → float` is `frsp`; `float → double` is nothing at all**
  (`w13_fops::d_widen` is a bare `4e800020 blr`).
* `fneg` is opcode-63 for both widths — there is no `fnegs`.
* A mixed-width expression is evaluated in **double** precision:
  `w13_fops::d_mixed` (`float a + double b`) is `fc21102a fadd`, and
  `(float)((double)a*(double)b)` (probe) is `fc0100b2 fmul` + `fc200018 frsp`,
  **not** `fmuls`. The `s` suffix is selected by the *expression* type.

### 3.3 The fused forms — mandatory, not optional

`fD = fA*fC + fB` and friends. Any `*` feeding a `+`/`-` is contracted; the port
cannot emit `fmuls` + `fadds` for these:

| source | instruction | XO | fields (D,A,B,C) | word | site |
|---|---|---|---|---|---|
| `a*b + c` | `fmadds f1,f1,f2,f3` | 29 | 1,1,3,2 | `ec2118ba` | `w13_fneg::n_fma` 0x00 |
| `c + a*b` | `fmadds f1,f1,f2,f3` | 29 | 1,1,3,2 | `ec2118ba` | `n_fma_comm` 0x08 — **identical bytes** |
| `a*b - c` | `fmsubs f1,f1,f2,f3` | 28 | 1,1,3,2 | `ec2118b8` | `n_fms` 0x10 |
| `c - a*b` | `fnmsubs f1,f1,f2,f3` | 30 | 1,1,3,2 | `ec2118bc` | `n_fnms` 0x18 |
| `double a*b+c*d` | `fmadd f1,f1,f2,f0` | 29 | 1,1,0,2 | `fc2100ba` | `n_dfma2` 0x34 |
| (deep) | `fmsubs f1,f12,f11,f10` | 28 | 1,12,10,11 | `ec2c52f8` | `n_rank` 0x64 |

`fnmsubs` computes `−(fA*fC − fB)` = `fB − fA*fC`, which is why `c − a*b` is one
instruction with no `fneg`.

### 3.4 Loads / stores

| instruction | opcode | fields | word | site |
|---|---|---|---|---|
| `lfs f0,0(r11)` | 48 | D=0,A=11,d=0 | `c00b0000` | `w13_fneg` 0x74 |
| `lfs f1,0(r11)` | 48 | D=1,A=11,d=0 | `c02b0000` | `w13_fneg` 0x94 |
| `lfd f0,0(r11)` | 50 | D=0,A=11,d=0 | `c80b0000` | `w13_fneg` 0x84 |
| `lfd f1,120(r1)` | 50 | D=1,A=1,d=120 | `c8210078` | `w13_fabi` 0x38 |
| `lfs f1,140(r1)` | 48 | D=1,A=1,d=140 | `c021008c` | probe `f_p16` |
| `stfd f31,-8(r1)` | 54 | D=31,A=1,d=−8 | `dbe1fff8` | `w13_fneg` 0x130 |
| `lfd f31,-8(r1)` | 50 | D=31,A=1,d=−8 | `cbe1fff8` | `w13_fneg` 0x19c |

---

## 4. Does a float expression change the obj SHELL?

> **For a pure W13a leaf (FP parameters and FP arithmetic, no constants, no
> spill): one extra symbol, and nothing else.** No extra section, no `.pdata`,
> no alignment change, no `.text` characteristics change.

`w13_fabi`, `w13_fops` and `w13_fscratch` — 33 float/double functions between
them, including 13-parameter chains — all produce exactly the five familiar
sections:

```
.drectve 132 0x00100A00 | .debug$S 152 0x42100040 | .XBLD$W 16 0xC0401040
.XBLD$W  16  0xC2301040 | .text     N  0x60400020
```

identical to the integer `p_align` control (`.text` = `0x60400020` =
`CNT_CODE | ALIGN_8BYTES | MEM_EXECUTE | MEM_READ` in both cases), and `.text`
carries **zero relocations**.

The one difference is an **undefined external symbol `_fltused`**:

```
[ 11] sec=5   val=0x00000000 sc=3  typ=0x0000 .text
[ 13] sec=5   val=0x00000000 sc=2  typ=0x0020 ?f_add@@YAMMM@Z
[ 14] sec=0   val=0x00000000 sc=2  typ=0x0020 _fltused        <-- sec 0 = undefined
[ 15] sec=5   val=0x00000008 sc=2  typ=0x0020 ?f_sub@@YAMMM@Z
```

Storage class 2 (EXTERNAL), section 0 (undefined), value 0, **type 0x0020**
(the DT_FUNCTION nibble, same as a function symbol), name in the string table.
Its position is **immediately after the first FP function's symbol group** — in
`p_mixorder` (an int function first, then a float one) it lands after `gf` and
after `gf`'s `.rdata`/`__real@` symbols, and the int function that follows comes
after it. There is exactly one `_fltused` per obj.

An integer-only TU has **no** `_fltused` (`p_align`, 3 functions, 16 symbols,
none of them `_fltused`).

**UNKNOWN — the exact trigger.** It is not "the TU executes an FP
instruction": `int z(int a, float b){return a+(int)b;}` emits
`fc00081e fctiwz f0,f1 ; d801fff0 stfd f0,-16(r1) ; 8161fff4 lwz r11,-12(r1) ;
7c6b1a14 add r3,r11,r3` and **no** `_fltused`. Observed positives:
FP-typed return (`float z(int a){return 1.0f;}`), FP store through a pointer,
FP comparison, an unused float parameter in a TU that also has an FP-returning
function. Every function inside the proposed W13a class is a positive, so the
port can emit `_fltused` unconditionally *for that class*; the general rule is
not pinned and must not be extrapolated.

`.pdata` appears in `w13_fneg` **only** because of `n_spill` (8 bytes,
`chars=0x40400040`, one ADDR32 (type 0x02) reloc at offset 0 targeting
`?n_spill@@YAMMMMMMMMM@Z`, payload `00000000 40001f03`). No non-spilling FP leaf
in any fixture produced a `.pdata` entry.

---

## 5. W13b — what a float constant costs

Every FP constant reference costs, in the obj:

### 5.1 One `.rdata` COMDAT section per distinct constant

| | float | double |
|---|---|---|
| `SizeOfRawData` | **4** | **8** |
| `Characteristics` | **`0x40301040`** | **`0x40401040`** |
| decoded | `CNT_INITIALIZED_DATA(0x40)` \| `LNK_COMDAT(0x1000)` \| `ALIGN_4BYTES(0x00300000)` \| `MEM_READ(0x40000000)` | same but `ALIGN_8BYTES(0x00400000)` |
| contents | the IEEE-754 bits, **big-endian** | ditto |

From `w13_fneg.obj`, in first-use order:

```
.rdata 4 0x40301040  3f800000              (1.0f)
.rdata 8 0x40401040  3ff0000000000000      (1.0)
.rdata 4 0x40301040  3fc00000              (1.5f)
.rdata 4 0x40301040  40000000              (2.0f)
.rdata 4 0x40301040  3eaaaaab              (1/3 rounded to float)
.rdata 8 0x40401040  3fd5555555555555      (1/3 rounded to double)
```

### 5.2 Two symbols per section, and the dedup rule

```
[ 23] sec=6  val=0 sc=3 typ=0x0000 .rdata
      aux: 04 00 00 00 | 00 00 | 00 00 | 00 00 00 00 | 00 00 | 02 | 00 00 00
           Length=4     nRel=0  nLine=0  CheckSum=0    Number=0  Selection=2
[ 25] sec=6  val=0 sc=2 typ=0x0000 __real@3f800000
```

* The COMDAT anchor is the `.rdata` **section symbol** (storage class 3 STATIC,
  one aux record, **`Selection = 2` = `IMAGE_COMDAT_SELECT_ANY`**, CheckSum
  **0**, Number 0), immediately followed by the constant's **external** symbol
  (storage class 2, type 0x0000 — *not* 0x0020, unlike function symbols).
* **Symbol name = `__real@` + the IEEE bits in lowercase hex**, 8 digits for
  float, 16 for double: `__real@3f800000`, `__real@3ff0000000000000`.
* **Dedup is by bit pattern, TU-wide.** In `w13_fneg`, `n_k_add` (`a+1.0f`),
  `n_k_two`'s first constant and `n_k_ret`… all resolve to whichever section
  already holds that pattern — `__real@40000000` is created for `n_k_two` and
  then **reused** by `n_self_add` (`a+a`) three functions later, with no second
  section. Sections and symbol pairs are appended in **first-use order**
  (sec 6, 7, 8, …).
* A float and a double of the same numeric value are **different** constants
  (`3f800000` and `3ff0000000000000` are separate sections).

### 5.3 The reference site: `addis` + `lfs`/`lfd`, four relocations

```
?n_k_add@@YAMM@Z         float f(float a){ return a + 1.0f; }
 0070 3d600000   addis r11,r0,0        ; REFHI  site
 0074 c00b0000   lfs   f0,0(r11)       ; REFLO  site
 0078 ec21002a   fadds f1,f1,f0
 007c 4e800020   blr
```

Relocation records (10 bytes each, `<VA:u32> <SymbolTableIndex:u32> <Type:u16>`,
little-endian) — **four per constant reference**, in this order:

| VA | type | meaning | symbol / payload |
|---|---|---|---|
| `addis` offset | `0x0010` | `IMAGE_REL_PPC_REFHI` | `__real@…` |
| `addis` offset | `0x0012` | `IMAGE_REL_PPC_PAIR` | index field = **0** (the addend) |
| `lfs`/`lfd` offset | `0x0011` | `IMAGE_REL_PPC_REFLO` | `__real@…` |
| `lfs`/`lfd` offset | `0x0012` | `IMAGE_REL_PPC_PAIR` | index field = **0** |

Raw bytes of the first pair from a two-function probe obj:
`08 00 00 00 | 11 00 00 00 | 10 00` then `08 00 00 00 | 00 00 00 00 | 12 00`.
Note the reloc target is the **`__real@…` external symbol**, not the `.rdata`
section symbol; and the PAIR record's symbol-index field is the low-half addend,
**0** in every observation (a non-zero addend was never produced — **UNKNOWN**).

`w13_fneg::n_k_two` shows two constants in one function: the two `addis` are
emitted **first**, then the two loads, and the address registers come from the
**integer** descending cursor `r11`, `r10`:

```
 00a0 3d600000   addis r11,r0,0        REFHI __real@3f800000
 00a4 3d400000   addis r10,r0,0        REFHI __real@40000000
 00a8 c00b0000   lfs   f0,0(r11)       REFLO __real@3f800000
 00ac c1aa0000   lfs   f13,0(r10)      REFLO __real@40000000
 00b0 ec01002a   fadds f0,f1,f0
 00b4 eda1682a   fadds f13,f1,f13
 00b8 ec200372   fmuls f1,f0,f13
```

So a float constant also consumes a **GPR** from the integer pool — the two
allocators interact, which is a structural change to `select_text`, not just an
extra encoder.

### 5.4 Constants c2 *synthesizes* — the reason 13b cannot be deferred cleanly

Three rewrites turn constant-free source into a constant reference:

| source | emitted | constant |
|---|---|---|
| `a + a` | `addis r11 ; lfs f0 ; ec210032 fmuls f1,f1,f0` | `__real@40000000` (2.0f) |
| `a / 3.0f` | `addis r11 ; lfs f0 ; fmuls f1,f1,f0` | `__real@3eaaaaab` = `(float)(1/3)` |
| `a / 3.0` | `addis r11 ; lfd f0 ; fc210032 fmul f1,f1,f0` | `__real@3fd5555555555555` |

Division by *any* literal becomes a multiply by the reciprocal **rounded to the
expression's precision** — `a/2.0f` → `__real@3f000000`, `a/10.0f` →
`__real@3dcccccd`. This is not reciprocal-exact and is applied unconditionally.
Division by a *variable* stays a real `fdivs`/`fdiv` (§3.1).

Two identity folds go the other way and emit **nothing**:
`a + 0.0f` and `a * 1.0f` are both a bare `4e800020 blr`
(`w13_fneg` 0xf0 and 0xf8).

### 5.5 The FP literal in the IL

```
FP_LIT := 33 <TYPE> <8 bytes: IEEE-754 DOUBLE, little-endian> <size:1> <00:1>
```

Confirmed — the hint in the task brief is right about the raw 8 bytes, and the
two trailing bytes complete it:

```
float  a+1.0f :  33 86 4a 40  00 00 00 00 00 00 f0 3f  04 00  02
double a+1.0  :  33 88 8a 41  00 00 00 00 00 00 f0 3f  08 00  02
float  a+0.5f :  33 86 4a 40  00 00 00 00 00 00 e0 3f  04 00  02
double a+2.5  :  33 88 8a 41  00 00 00 00 00 00 04 40  08 00  02
float  a+0.1f :  33 86 4a 40  00 00 00 a0 99 99 b9 3f  04 00  02
double a+0.1  :  33 88 8a 41  9a 99 99 99 99 99 b9 3f  08 00  02
```

* The payload is **always** a `double`, for both literal types.
* The byte after the payload is the **width in bytes** of the materialized
  constant: `04` for a float literal, `08` for a double literal.
* For a `float` literal the payload is **already rounded to float precision**:
  `0.1f` carries `0x3FB99999A0000000` = 0.10000000149011612, whose `f32`
  round-trip is exactly `3dcccccd` — the `.rdata` bytes and the `__real@` name.
  So `.rdata` content = `(f32)payload` big-endian, losslessly, with no
  double-rounding question.
* The final `00` byte was `00` in every observation. **UNKNOWN** meaning.

---

## 6. The precise fail-closed negative list for W13a

W13a = **FP leaf, parameters only, no constants**. Accept only if *all* of:

| # | condition | why (fixture) |
|---|---|---|
| **A1** | every LOAD operand type is `86 45 40` (float) or `88 85 41` (double), and the function's result type is the *same* one | a mixed-width expression evaluates in double and may need `frsp` (`w13_fops::d_mixed`, `f_narrow`) |
| **A2** | no `33` literal anywhere in the body | every FP literal costs an `.rdata` COMDAT + 4 relocations + a GPR (§5) |
| **A3** | no `2c` CONVERT node | int↔FP is a red-zone round trip (`w13_fneg::n_i2f`, `n_f2i`) |
| **A4** | the operator set is `{02 ADD, 03 SUB, 04 MUL, 05 DIV, 08 NEG}` | anything else is unmodeled |
| **A5** | **no `*` node is an operand of a `+`/`-` node** | contraction to `fmadds`/`fmsubs`/`fnmsubs` is mandatory and not modeled (§3.3, `n_fma`…`n_fma_tree`) |
| **A6** | no `+`/`-` node is an operand of a `+`/`-` node, and no `*` node is an operand of a `*` node, **unless** the shape is a flat chain with all-parameter leaves | the flat chain is canonicalized by sorting terms (§2.4); the parenthesized nested form is not, and the two are distinguished only by the unexplained `0x59` |
| **A7** | `a - a`-style algebraic simplification cannot fire — i.e. no leaf appears twice under a `+`/`-`, and no `x + x` | `a+a` becomes `x*2.0f`, a constant (§5.4); `a+0.0f`/`a*1.0f` vanish |
| **A8** | peak simultaneous live FP values (live FP parameters + live temps) ≤ **14** | `w13_fneg::n_spill` saves f31/f30/f29 and grows `.pdata` |
| **A9** | ≤ 13 FP parameters **and** ≤ 8 positional parameters total | the 14th FP parameter and the 9th positional parameter are stack-homed (`w13_fabi::dp_fourteen`, `ip_after_floats`) |
| **A10** | division is by a **register**, never by a literal | `x/k` becomes a reciprocal multiply against a synthesized constant (§5.4) |

The negatives that a naive tree selector gets *wrong* rather than *out of range*
— the ones that must be tested explicitly — are A5 (it would emit
`fmuls`+`fadds` where c2 emits one `fmadds`), A6's parenthesized form (it would
flatten, or fail to sort, the chain), A7 and A10 (it would emit an instruction
where c2 emits none, or none where c2 emits a constant load).

Additional gates that the *existing* integer code would fail if the FP path were
grafted onto it:

* The FP pool is `[f0, f13, …, f1]`, **not** the integer `[r11, …, r3]` shape —
  in particular `f0` is allocatable and is the *first* slot, and the result
  register `f1` is the *last*. `select_text`'s `if d < 9 { reject }` guard has no
  FP analogue and must not be copied.
* An FP `+` chain must **not** reuse a single accumulator. The integer
  `PlanOp::Bin { op: IlOp::Add, .. } => SCRATCH_REG` special case is exactly
  wrong for FP (`w13_fscratch::fa4`).
* `fsubs fD,fA,fB` = `fA − fB` — the **opposite** of `encode_subf`'s
  load-bearing reversal. Reusing the integer operand-order convention silently
  negates every FP subtraction.

---

## 7. Open / unexplained (do not guess)

1. **The `0x59` byte** (§0.1). Syntactic on the surface, but it is the only IL
   difference between a product tree that c2 flattens and one it does not.
2. **The cursor/constant interaction** (§2.6) — `p_const2::k4`'s third constant
   load takes a dead register out of cursor order. Blocks W13b.
3. **The `_fltused` trigger** (§4) — a counterexample exists that executes
   `fctiwz` and does not reference it.
4. **The trailing `00`** of an FP literal (§5.5), and whether a REFHI/REFLO
   PAIR's addend field is ever non-zero (§5.3).
5. **Instruction scheduling** with constants: `w13_fneg::n_k_two` emits both
   `addis` before both `lfs`, but `p_const2::k5` interleaves
   (`fadds ; addis ; fadds ; lfs ; fmadds`). The FP cursor rule holds
   byte-for-byte in both, so allocation happens after scheduling — but the
   scheduler's ordering heuristic is unmodeled, exactly as in W5 §8.3.
6. Varargs / struct-by-value / `long double` FP argument passing (§1).
