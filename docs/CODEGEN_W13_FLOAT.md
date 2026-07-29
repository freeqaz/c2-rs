# CODEGEN W13 — float / double codegen (characterization + the ported boundary)

Roadmap step **W13**. Measured demand: the operand-type buckets
`expr-load-type-864540` (float, 3.4 % of blocked functions) and
`expr-load-type-888541` (double, 3.1 %) — 6.5 % together, comparable to the
largest single schedulable bucket. `crates/c2-il/src/func.rs::parse_expr`
accepts only `INT_TYPE = 86 41 74` on a `B9` LOAD, so every FP body used to
block at its first operand.

Every register, opcode, section and relocation claim below is backed by bytes
read out of an obj produced by the real toolchain (`cl.exe` 16.00.11886.00 under
wibo, the harness' standard `/Ox /GS- /c`), and every instruction word cited in
§3 is re-derived from its bit fields and checked against the observed word
(30/30 exact). Nothing here comes from `paint/`.

**Two rungs have since landed against this document, and it now serves both
roles — characterization *and* the specification of what the port accepts.**

- **W13a** (commit 9c7ba7d) — float/double **leaves over parameters**. §1–§4.
- **W13b** (commit cebfb88) — a **single pooled floating-point constant** per
  body: one `.rdata` COMDAT, an `addis`/`lfs`-`lfd` pair, a REFHI/REFLO
  relocation quad. §5, and the fail-closed list of §6.

Fixture verdicts, re-measured at cebfb88 (`c2rs diff`; source only — the IL and
objs are regenerated, never committed):

```
fixtures/cpp/mvp_fmul3.cpp     -> ReferenceReplay=ByteExact (ref= 861B replay= 861B)  Port=Match           (W13a)
fixtures/cpp/w13b_fconst.cpp   -> ReferenceReplay=ByteExact (ref=1017B replay=1017B)  Port=Match           (W13b)
fixtures/cpp/w13b_fdedup.cpp   -> ReferenceReplay=ByteExact (ref=1512B replay=1512B)  Port=Match           (W13b)
fixtures/cpp/w13b_ffold.cpp    -> ReferenceReplay=ByteExact (ref=1328B replay=1328B)  Port=NotImplemented  (negative)
fixtures/cpp/w13b_fpool.cpp    -> ReferenceReplay=ByteExact (ref=1225B replay=1225B)  Port=NotImplemented  (negative)
fixtures/cpp/w13_fabi.cpp      -> ReferenceReplay=ByteExact (ref=1268B replay=1268B)  Port=NotImplemented
fixtures/cpp/w13_fops.cpp      -> ReferenceReplay=ByteExact (ref=1363B replay=1363B)  Port=NotImplemented
fixtures/cpp/w13_fscratch.cpp  -> ReferenceReplay=ByteExact (ref=1498B replay=1498B)  Port=NotImplemented
fixtures/cpp/w13_fneg.cpp      -> ReferenceReplay=ByteExact (ref=3144B replay=3144B)  Port=NotImplemented
```

The four `w13_*` characterization TUs and the two `w13b_*` negatives still
report `Port=NotImplemented` **as a whole TU**, which is what pins the boundary:
decode is all-or-nothing per TU, so one out-of-class function refuses the file.
Inside `w13_fneg.cpp` two functions have nonetheless been *promoted* out of
negative status — see §5.10.

Byte offsets quoted below are `.text` offsets inside those fixtures' reference
objs unless stated otherwise.

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
`+5` (`0x45`→`0x4a`, `0x85`→`0x8a`). For `int` the two coincide, which is why the
old single-`INT_TYPE` model never had to distinguish them. Both pairs are now
distinct constants in the parser (`FLOAT_TYPE`/`DOUBLE_TYPE` vs
`FLOAT_LIT_TYPE`/`DOUBLE_LIT_TYPE`), and a literal whose width disagrees with the
operand width is rejected outright — a mixed-width literal implies a conversion.

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

**The ONE-constant case is now resolved; the multi-constant case is not.**
Superseding what this section said before W13b landed: the missing rule for a
single constant was not a cursor variant at all but an *ordering* fact — the
constant claims its FP register **before** any interior temporary does, in IL
order, off the same cursor. That is §5.8, and it is what made W13b byte-exact.

Beyond one constant the interaction really is unresolved. `p_const2::k4`
(`(a+1.0f)*(b+2.0f)*(c+3.0f)`) allocates
`f0(c1), f13(c2), f12(t1), f11(t2), f0(c3 — reused), f10(t3), f9(t4), f1` —
the third constant load takes a *dead* register out of cursor order. No model
tried here reproduces both `k4` and `k_two`/`k5`, and the two-constant captures
of §5.6 show why: with two surviving constants c2 also *reschedules*, so
allocation order and emission order come apart. Two or more literals are
therefore still a hard reject (§6, B1).

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

The "no constants" qualifier is doing real work: add one literal and the shell
gains a `.rdata` COMDAT, two symbols, four relocations, and — because `.text` is
no longer last — a different relocation *offset* (§5.7). That is W13b, §5.

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

## 5. W13b — pooled floating-point constants (LANDED, gated at one per body)

A float has no immediate form on PPC, so a literal is *always* materialized out
of memory. Every FP constant reference costs, in the obj:

> **What the port accepts:** exactly **one** surviving FP literal per function
> body, in an otherwise-W13a leaf. `w13b_fconst.cpp` and `w13b_fdedup.cpp` are
> `Port=Match`. `w13b_fpool.cpp` (two literals in the IL) and `w13b_ffold.cpp`
> (the identity folds) must keep refusing, and the gates that refuse them live in
> the **IL parser** (`crates/c2-il/src/func.rs::try_parse_float_leaf`) rather
> than in codegen, so the census and the emission gate cannot disagree about what
> is in class.
>
> The reason for the ceiling is §5.9: **c2, not c1xx, is the floating-point
> constant evaluator.** The IL still carries every literal the source wrote, and
> the backend folds, reassociates and strength-reduces them. So "how many
> constants does the source have" is not the question; "how many survive c2" is,
> and only a capture answers it.

### 5.1 One `.rdata` COMDAT section per distinct constant

| | float | double |
|---|---|---|
| `SizeOfRawData` | **4** | **8** |
| `Characteristics` | **`0x40301040`** | **`0x40401040`** |
| decoded | `CNT_INITIALIZED_DATA(0x40)` \| `LNK_COMDAT(0x1000)` \| `ALIGN_4BYTES(0x00300000)` \| `MEM_READ(0x40000000)` | same but `ALIGN_8BYTES(0x00400000)` |
| contents | the IEEE-754 bits, **big-endian** | ditto |

The pools are **appended after `.text`**, one section each, in
**first-reference order** — so `.text` is no longer the last section in the obj
once any constant is pooled, which is the layout fact of §5.7. Each is a COMDAT
with `Selection = 2` (`IMAGE_COMDAT_SELECT_ANY`, "pick any") and an aux
**checksum of 0**; the section carries **no relocations** of its own.

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
* **Dedup is TU-wide and keyed on `(bit pattern, width)`.** In `w13_fneg`,
  `n_k_add` (`a+1.0f`), `n_k_two`'s first constant and `n_k_ret`… all resolve to
  whichever section already holds that pattern — `__real@40000000` is created for
  `n_k_two` and then **reused** by `n_self_add` (`a+a`) three functions later,
  with no second section. Sections and symbol pairs are appended in
  **first-reference order** (sec 6, 7, 8, …).
* A float and a double of the same numeric *value* are **two** constants — two
  sections and two symbols. `w13b_fdedup.cpp` is the dedicated witness: `ka` and
  `kc` both use `1.0f` and share one 4-byte COMDAT and one `__real@3f800000`,
  while `kd` (`double a + 1.0`) gets its own 8-byte COMDAT and its own
  `__real@3ff0000000000000`. **The key is the pattern *and* the width, not the
  value** — a pool keyed on value alone matches every single-width fixture and
  then silently merges those two.

**Where the two symbols go — immediately after the symbol of the function that
FIRST references the constant**, not grouped at the end of the symbol table and
not next to the section. Both symbols carry `Value = 0` (each constant owns its
whole section, so its displacement into it is 0). `_fltused` still lands after
the **whole group** belonging to the first float function — i.e. after that
function's symbol, its callee's if any, and all the `.rdata`/`__real@` pairs it
introduced. `w13b_fdedup.cpp` is what pins this: with four functions and three
distinct constants, "group at the end" and "next to the first referencing
function" produce different symbol indices, and the relocation records name
those indices, so getting it wrong corrupts the relocations too.

### 5.3 The reference site: `addis` + `lfs`/`lfd`, four relocations

```
?n_k_add@@YAMM@Z         float f(float a){ return a + 1.0f; }
 0070 3d600000   addis r11,r0,0        ; REFHI  site
 0074 c00b0000   lfs   f0,0(r11)       ; REFLO  site
 0078 ec21002a   fadds f1,f1,f0
 007c 4e800020   blr
```

Both immediates are emitted as **0** — the linker patches them from the
relocations. `lfs` is primary opcode **48**, `lfd` is **50** (§3.4). The address
GPR comes off the **integer** scratch cursor, descending from `r11` exactly as
the integer selector's temporaries do: so a float constant consumes a **GPR** as
well as an FPR, and the two allocators interact. That is a structural change to
`select_text`, not just an extra encoder.

Relocation records (10 bytes each, `<VA:u32> <SymbolTableIndex:u32> <Type:u16>`,
little-endian) — **four per constant reference**, in this order:

| VA | type | meaning | symbol / payload |
|---|---|---|---|
| `addis` offset | `0x0010` | `IMAGE_REL_PPC_REFHI` | `__real@…` |
| `addis` offset | `0x0012` | `IMAGE_REL_PPC_PAIR` | index field = **0** |
| `lfs`/`lfd` offset | `0x0011` | `IMAGE_REL_PPC_REFLO` | `__real@…` |
| `lfs`/`lfd` offset | `0x0012` | `IMAGE_REL_PPC_PAIR` | index field = **0** |

Raw bytes of the first pair from a two-function probe obj:
`08 00 00 00 | 11 00 00 00 | 10 00` then `08 00 00 00 | 00 00 00 00 | 12 00`.
Note the reloc target is the **`__real@…` external symbol**, not the `.rdata`
section symbol.

**Ordering: the records are sorted ascending by `VirtualAddress`, and each PAIR
immediately follows its partner.** For one constant that is the same as emission
order; with several reference sites it is not, which is why the emitter sorts
rather than appends.

**Why the PAIR's symbol-index field is always 0 — this is now explained, not
merely observed.** The field is not a symbol index in a PAIR record; it is the
**displacement into the target section**. Every constant owns its *whole*
COMDAT — `SizeOfRawData` is exactly the constant's 4 or 8 bytes — so the
displacement is necessarily 0. Earlier revisions of this document recorded the 0
as an unexplained "addend never seen non-zero"; the mechanism is that there is
nowhere else in the section for it to point.

### 5.4 Constants c2 *synthesizes* — why 13b could not be deferred cleanly

Three rewrites turn constant-free source into a constant reference, which is why
even the W13a "no constants" class had to know about them: the source having no
literal is no guarantee the obj has no constant.

| source | emitted | constant |
|---|---|---|
| `a + a` | `addis r11 ; lfs f0 ; ec210032 fmuls f1,f1,f0` | `__real@40000000` (2.0f) |
| `a / 3.0f` | `addis r11 ; lfs f0 ; fmuls f1,f1,f0` | `__real@3eaaaaab` = `(float)(1/3)` |
| `a / 3.0` | `addis r11 ; lfd f0 ; fc210032 fmul f1,f1,f0` | `__real@3fd5555555555555` |

Division by *any* literal becomes a multiply by the reciprocal **rounded to the
expression's precision** — `a/2.0f` → `__real@3f000000`, `a/10.0f` →
`__real@3dcccccd`. This is not reciprocal-exact and is applied unconditionally.
Division by a *variable* stays a real `fdivs`/`fdiv` (§3.1).

Identity folds go the other way and emit **nothing**: `a + 0.0f` and `a * 1.0f`
are both a bare `4e800020 blr` (`w13_fneg` 0xf0 and 0xf8). The full fold table —
including `a - 0.0f`, which also vanishes, and `a * 0.0f`, which pointedly does
**not** — is §5.9. Read the two together: this section is the direction where a
constant-free source *gains* a constant, §5.9 the direction where a source
literal *loses* one, and the port refuses in both directions.

### 5.5 The FP literal in the IL

```
FP_LIT := 33 <lit-TYPE> <8 bytes: IEEE-754 binary64, LITTLE-endian> <width:u16 LE>
```

**Correction to an earlier revision of this section.** It read the trailer as
two separate fields, `<size:1> <00:1>`, and filed the trailing `00` as UNKNOWN.
It is one **little-endian `u16`** carrying the width in bytes — `04 00` or
`08 00` — so there is no unexplained byte and nothing left open here. The
observed bytes are identical; only the reading changed, and the parser reads it
as a `u16` (`func.rs::try_parse_float_leaf`).

`lit-TYPE` is `86 4a 40` for float and `88 8a 41` for double. **These are not
the operand type tags** `86 45 40` / `88 85 41` — the `kind` byte is `+5`
(§0's table). For `int` the literal and operand types coincide, which is why the
old single-`INT_TYPE` model never had to distinguish them.

Verified capture of `float k_add(float a){return a + 1.0f;}` — the whole body
segment, with the literal bracketed by its neighbours so the field boundaries are
unambiguous:

```
4c 4f 11 53 b9 e3 09 86 45 40  33 86 4a 40 00 00 00 00 00 00 f0 3f 04 00  02 41 86 45 …
            \__ LOAD a: float _/  \__ FP_LIT 1.0f, width 4 ________________/  \_ ADD, result type
```

and the earlier probe set, unchanged and re-read under the `u16` rule:

```
float  a+1.0f :  33 86 4a 40  00 00 00 00 00 00 f0 3f  04 00
double a+1.0  :  33 88 8a 41  00 00 00 00 00 00 f0 3f  08 00
float  a+0.5f :  33 86 4a 40  00 00 00 00 00 00 e0 3f  04 00
double a+2.5  :  33 88 8a 41  00 00 00 00 00 00 04 40  08 00
float  a+0.1f :  33 86 4a 40  00 00 00 a0 99 99 b9 3f  04 00
double a+0.1  :  33 88 8a 41  9a 99 99 99 99 99 b9 3f  08 00
```

* The payload is **always** a binary64 pattern, for both literal types, and the
  width comes from the trailer (and must agree with `lit-TYPE`).
* For a `float` literal the payload is **a binary64 pattern already rounded to
  binary32**: `0.1f` carries `0x3FB99999A0000000` = 0.10000000149011612, whose
  `f32` round-trip is exactly `3dcccccd` — the `.rdata` bytes and the `__real@`
  name. So `.rdata` content = `(f32)payload` big-endian, losslessly, with no
  double-rounding question. The port keeps the raw bits and re-checks that
  narrowing is exact before accepting, rather than trusting the invariant.

### 5.6 Two surviving constants: the schedule changes (characterized, NOT implementable)

With **one** constant the address setup and the load sit adjacently, immediately
before the use, so the REFLO site is exactly `hi_off + 4`. That is the assumption
the whole one-constant path is built on, and with two constants it fails: c2
hoists **every** `addis` into a prologue group in IL order (`r11` then `r10`),
then schedules each `lfs` at its **first use**, and recycles the FP register once
a constant dies.

Two captures, and they are all there is:

```
p1 = float p1(float a,float b){return (a + 1.0f) - (b + 2.0f);}
 3d600000 addis r11,r0,0   REFHI __real@3f800000
 3d400000 addis r10,r0,0   REFHI __real@40000000
 c00b0000 lfs   f0,0(r11)  REFLO __real@3f800000
 c1aa0000 lfs   f13,0(r10) REFLO __real@40000000
 ec01002a fadds f0,f1,f0
 eda2682a fadds f13,f2,f13
 ec206828 fsubs f1,f0,f13
 4e800020 blr
```

```
p5 = float p5(float a,float b,float c){return a + 1.0f - b - 2.0f + c;}
 3d600000 addis r11,r0,0   REFHI __real@3f800000
 3d400000 addis r10,r0,0   REFHI __real@40000000
 c00b0000 lfs   f0,0(r11)  REFLO __real@3f800000
 eda1002a fadds f13,f1,f0
 c00a0000 lfs   f0,0(r10)  REFLO __real@40000000   <- f0 REUSED
 ed8d1028 fsubs f12,f13,f2
 ed6c0028 fsubs f11,f12,f0
 ec2b182a fadds f1,f11,f3
 4e800020 blr
```

The two captures already **disagree about where the second `lfs` goes** — `p1`
keeps both loads in the prologue group, `p5` defers the second to its first use
and reloads into `f0`. *[INFERENCE, not measurement:* the discriminating variable
looks like liveness of the previous constant — in `p5` the first constant is dead
before the second is needed, so `f0` is free again; in `p1` both are live across
the first arithmetic. Two data points cannot establish that.*]*

**Two captures characterizing a scheduler is not enough to implement from**, and
this is stated as a limit rather than a to-do: `w13_fneg::n_k_two` (§5.3's old
example) and `p_const2::k4`/`k5` show at least a third and fourth arrangement.
Two or more surviving literals are therefore rejected in the parser
(`w13b_fpool.cpp`), and the ceiling stands until a capture set large enough to
separate candidate scheduling rules exists.

### 5.7 A layout correction that applies to the WHOLE emitter

> **A section's relocation records sit immediately after *that section's own* raw
> data — not after every section's raw data.**

This is not a W13b fact; it is a COFF-layout fact the emitter had wrong and could
not previously observe. While `.text` was always the **last** section, "after
this section's data" and "after all sections' data" name the same offset, so both
readings produced byte-identical objs. Pooling a constant puts a `.rdata` behind
`.text` and separates them: c2 places the four `.text` REFHI/REFLO records
**between** `.text` and the first `.rdata`, and the port was appending them after
both.

`w13b_fdedup.cpp` is the fixture that caught it. Worth recording as a
coverage lesson of exactly the §1-of-`GAPS.md` kind: the defect was latent from
the first relocation the emitter ever wrote, and no amount of green on the
existing corpus was evidence against it, because the corpus contained no obj in
which the two rules differ.

### 5.8 Constants claim their FP register FIRST

> **A constant claims its FP register *before* any interior temporary does, in IL
> order, off the same rotating `FP_POOL` cursor (`[f0, f13..f1]`, §2).**

Witness — `ke` from `w13b_fpool.cpp`,
`float ke(float a,float b){return a*2.0f*b*3.0f;}`, which c2 reassociates and
folds to `(a*b)*6.0f`:

```
eda100b2 fmuls f13,f1,f2      ; the interior temp -> f13, NOT f0
3d600000 addis r11,r0,0       REFHI __real@40c00000   (6.0f)
c00b0000 lfs   f0,0(r11)      REFLO __real@40c00000
ec2d0032 fmuls f1,f13,f0
4e800020 blr
```

(The two `fmuls` words are **re-derived** from §3's A-form field layout rather
than quoted from an obj dump; the capture is quoted at mnemonic level. The
register assignment — which is the claim — is what the capture shows.)

The temp is **f13** and the constant is **f0**, so the constant took pool slot 0
even though the multiply is emitted first. **The plausible-but-wrong rule is
"allocate in emission order"**, and it is wrong in a way nothing else in the
corpus can see: emission order would put the multiply in `f0` and the constant in
`f13`, which still matches *every* single-operator body (`a + 1.0f`,
`a * 2.0f`, …), because there is no interior temporary in those to collide with.
`ke` had to be written specifically to separate the two rules — a body with a
constant **and** an interior temp — which is why it is called out here rather than
left implicit in the allocator.

### 5.9 Why the gate is ONE constant per body: c2 is the constant evaluator

The IL contains every literal the source wrote; `c1xx` folds none of them. So the
count of literals in the IL is *not* the count of constants in the obj, and every
transform below is c2's:

**Identity folds, per `(operator, value)` pair** — `w13b_ffold.cpp`:

| source | emitted | constant pooled |
|---|---|---|
| `a + 0.0f` | `4e800020` bare `blr` | none |
| `a * 1.0f` | `4e800020` bare `blr` | none |
| `a - 0.0f` | `4e800020` bare `blr` | none |
| `a * 0.0f` | `addis`/`lfs` + `fmuls` | **`__real@00000000`** |

`a * 0.0f` is the load-bearing row. **The gate cannot be "refuse the value 0.0
or 1.0"** — that would refuse `a * 0.0f`, which c2 really does lower as a load
and a multiply (signed zero and NaN make folding it to a constant zero unsafe).
Nor can it be "anything times zero is zero", which would emit a wrong bare
`blr`. It is per **pair**, and only a fixture holding both halves separates the
two candidate rules. W13b briefly mis-emitted all four of the folds, which is
why this fixture exists.

**Constant divisors strength-reduce to a reciprocal multiply.** `a / 2.0f` emits
an `fmuls` against `__real@3f000000` — no `fdivs` at all — and `a/3.0f/7.0f`
collapses to a **single** `fmuls` against `__real@3d430c31` = 1/21. That value is
not exactly representable, so this is a genuine numeric transform, not a
rewrite: reproducing it means reproducing c2's rounding, and a Div with a literal
operand is refused rather than approximated.

**Reassociation.** `a*2.0f*b*3.0f` becomes `(a*b)*6.0f`, pooling
`__real@40c00000` — two IL literals, one obj constant, and the surviving value
appears nowhere in the source.

Together: modeling more than one literal means modeling c2's constant evaluator
*and* (§5.6) its scheduler. Hence the ceiling, enforced in the parser.

### 5.10 Two former negatives promoted

`w13_fneg.cpp` N3 was written as four constant-related negatives. Two are now
**byte-exact** and no longer negatives — `n_k_add` (`float a + 1.0f`) and
`n_k_dadd` (`double a + 1.0`), whose shapes are the subjects of
`w13b_fconst.cpp` and `w13b_fdedup.cpp::kd`. They are kept in `w13_fneg.cpp` so
the class boundary stays visible next to the two that still refuse:

* `n_k_ret` (`return 1.5f;`) — no operand to load the constant *into*, and no
  expression at all, so the leaf grammar does not apply.
* `n_k_two` (`(a + 1.0f) * (a + 2.0f)`) — two surviving constants, §5.6, and a
  repeated leaf besides.

Because decode is all-or-nothing per TU, `w13_fneg.cpp` as a *file* still reports
`Port=NotImplemented`; the promotion is a statement about the two function
shapes, witnessed by the dedicated fixtures.

---

## 6. The precise fail-closed negative list

W13a = **FP leaf, parameters only, no constants**; W13b adds **at most one
surviving literal**. Accept only if *all* of A1–A10 and, when a literal is
present, *all* of B1–B4. Every one of these is enforced in
`crates/c2-il/src/func.rs::try_parse_float_leaf` — in the **parser**, not in
codegen, so `c2rs census` and the emitter cannot disagree about the class.

| # | condition | why (fixture) |
|---|---|---|
| **A1** | every LOAD operand type is `86 45 40` (float) or `88 85 41` (double), and the function's result type is the *same* one | a mixed-width expression evaluates in double and may need `frsp` (`w13_fops::d_mixed`, `f_narrow`) |
| **A2** | ~~no `33` literal anywhere in the body~~ — **superseded by W13b**: at most **one** literal, subject to B1–B4 | each surviving FP literal costs an `.rdata` COMDAT + 4 relocations + a GPR + an FPR (§5); the *second* one also changes the schedule (§5.6) |
| **A3** | no `2c` CONVERT node | int↔FP is a red-zone round trip (`w13_fneg::n_i2f`, `n_f2i`) |
| **A4** | the operator set is `{02 ADD, 03 SUB, 04 MUL, 05 DIV, 08 NEG}` | anything else is unmodeled |
| **A5** | **no `*` node is an operand of a `+`/`-` node** | contraction to `fmadds`/`fmsubs`/`fnmsubs` is mandatory and not modeled (§3.3, `n_fma`…`n_fma_tree`) |
| **A6** | no `+`/`-` node is an operand of a `+`/`-` node, and no `*` node is an operand of a `*` node, **unless** the shape is a flat chain with all-parameter leaves | the flat chain is canonicalized by sorting terms (§2.4); the parenthesized nested form is not, and the two are distinguished only by the unexplained `0x59` |
| **A7** | `a - a`-style algebraic simplification cannot fire — i.e. no leaf appears twice under a `+`/`-`, and no `x + x` | `a+a` becomes `x*2.0f`, a constant (§5.4); `a+0.0f`/`a*1.0f` vanish |
| **A8** | peak simultaneous live FP values (live FP parameters + live temps) ≤ **14** | `w13_fneg::n_spill` saves f31/f30/f29 and grows `.pdata` |
| **A9** | ≤ 13 FP parameters **and** ≤ 8 positional parameters total | the 14th FP parameter and the 9th positional parameter are stack-homed (`w13_fabi::dp_fourteen`, `ip_after_floats`) |
| **A10** | division is by a **register**, never by a literal | `x/k` becomes a reciprocal multiply against a synthesized constant (§5.4) |

And, when the body contains a literal (the W13b half):

| # | condition | why (fixture) |
|---|---|---|
| **B1** | **at most one** `33` FP literal in the body | with two, c2 hoists every `addis` into a prologue group and schedules the loads at first use, so the REFLO site stops being `hi_off + 4` (§5.6, `w13b_fpool.cpp`) |
| **B2** | no `05 DIV` anywhere in a body that has a literal | a constant divisor becomes a reciprocal multiply, inexactly (§5.9, `w13b_ffold::q3`, `w13b_fpool::kdiv`) |
| **B3** | the literal is not an **identity for an operator present in the body** — value `0.0` with any `+`/`-`, value `1.0` with any `*` | those fold to a bare `blr` with nothing pooled; `a * 0.0f` does **not** fold, so the gate is per `(operator, value)` pair and not per value (§5.9, `w13b_ffold.cpp`) |
| **B4** | for a float literal, the binary64 payload narrows to binary32 **exactly** | otherwise the four bytes the port would pool are not the four c2 pooled (§5.5) |

B3 is deliberately a slight **over**-refusal: it keys on "an operator in the body"
rather than on the operator the literal is actually an operand of, because
over-refusing costs a refusal while under-refusing emits three instructions where
c2 emits none.

The negatives that a naive tree selector gets *wrong* rather than *out of range*
— the ones that must be tested explicitly — are A5 (it would emit
`fmuls`+`fadds` where c2 emits one `fmadds`), A6's parenthesized form (it would
flatten, or fail to sort, the chain), A7 and A10 (it would emit an instruction
where c2 emits none, or none where c2 emits a constant load), and B3 (which W13b
got wrong in development, emitting a pooled `__real@00000000` and an `fadds`
where c2 emits a bare `blr`).

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
* An FP constant also allocates from the **integer** pool for its `addis` address
  register (§5.3), so the FP and integer allocators are no longer independent —
  and it takes its FP register **before** the interior temporaries (§5.8), so the
  allocator cannot simply walk the emitted instruction list in order.
* A section's relocations belong after **its own** raw data (§5.7). This is
  emitter-wide, not FP-specific.

---

## 7. Open / unexplained (do not guess)

1. **The `0x59` byte** (§0.1). Syntactic on the surface, but it is the only IL
   difference between a product tree that c2 flattens and one it does not.
2. **The cursor/constant interaction beyond one constant** (§2.6, §5.6) —
   `p_const2::k4`'s third constant load takes a dead register out of cursor
   order, and `p1`/`p5` disagree about where the second `lfs` goes. The
   *one*-constant case is closed (§5.8: the constant allocates first, in IL
   order). This is what caps W13b at one literal per body.
3. **The `_fltused` trigger** (§4) — a counterexample exists that executes
   `fctiwz` and does not reference it.
4. ~~**The trailing `00`** of an FP literal, and the REFHI/REFLO PAIR addend.~~
   **CLOSED.** The literal's trailer is a single little-endian `u16` width, not
   `<size> <00>`, so there is no trailing unknown (§5.5). The PAIR field is a
   displacement into the target section, and it is 0 because each constant owns
   its whole COMDAT (§5.3). Neither was a new fact — both were misreadings of
   bytes already in hand, which is the cheapest kind of open item to close and the
   easiest to leave open indefinitely.
5. **Instruction scheduling** with **two or more** constants: `w13_fneg::n_k_two`
   and `p1` emit every `addis` before every `lfs`, `p5` defers its second load to
   first use, and `p_const2::k5` interleaves
   (`fadds ; addis ; fadds ; lfs ; fmadds`). The FP cursor rule holds
   byte-for-byte in all of them, so allocation happens after scheduling — but the
   scheduler's ordering heuristic is unmodeled, exactly as in W5 §8.3. **Four
   arrangements from four captures is characterization, not a rule**, and no
   attempt should be made to implement the two-constant path from them.
6. Varargs / struct-by-value / `long double` FP argument passing (§1).
7. Whether a **single** constant referenced from **two sites in one body** keeps
   the `hi_off + 4` adjacency, or gets one `addis` shared between them. `ka`/`kc`
   in `w13b_fdedup.cpp` reference the same constant from *different* functions,
   which exercises relocation ordering across a `.text` (four sites, sixteen
   records) but says nothing about two sites inside one body. In class today the
   question cannot arise — A7 rejects a repeated leaf and B1 a second literal — so
   it is a boundary to probe before relaxing either.
