# The floating-point argument register file — W27 + W28, landed

Two numberings run over one parameter list, neither of them is the formal's
index, and they disagree in **opposite directions**. That single fact is the
whole content of this rung, it produced two of the ten live wrong-bytes emits
this project has found (`docs/GAPS.md` §6 (6) and (7)), and it produced an
eleventh while the rung was being built (§4 below).

Every row here is bytes out of a real obj (`cl.exe` 16.00.11886.00 under wibo
1.0.1-23, `/O1 /GS- /c`), captured with `scripts/gt_capture.sh` and read with
`scripts/gt_dump.py`. Nothing in this document is inferred from the ABI
documentation; where a rule is not captured it says so.

---

## 0. The rule, stated exactly

* A **floating-point** parameter takes `f<j>`, where `j` counts the FP
  parameters **alone**, 1-based, and does **not** fill its GPR.
* Every other scalar takes `r<2 + k>`, where `k` is its **argument slot**,
  1-based — and an FP parameter still *consumes* a slot, so this numbering
  counts it.
* `this`, when present, takes r3 and slot 1, and displaces **nothing** in the FP
  file.
* The FP numbering is **width-agnostic**: a `double` takes one FP register, not
  two.

So for `int t6(int a, float b, float c)`: `a` → r3, `b` → f1, `c` → f2. The
index rule says f2 and f3; a packed-GPR rule would say `a` → r3 and the next
integer → r4. Both are refuted below by one instruction each.

## 1. The FP argument file — MEASURED

`work/fpgt/p1.cpp`, `p2.cpp`, `p3.cpp` (gitignored scratch; every source line is
reproduced in the fixture headers). Callee prototypes are declared, never
defined, so each body is a tail call and the only bytes are the argument setup.

```text
  float  t1(float a)          { return g1f(a); }    48000000  b g1f
  float  t2(float a, float b) { return g1f(b); }    fc201090  fmr f1,f2
                                                    4bfffffc  b g1f
  int    t4(int a, float b)   { return gif(a,b); }  48000000  b gif
  int    t5(int a, float b)   { return gfi(b,a); }  7c641b78  mr r4,r3
  int    t6(int a,float b,float c){return gffi(b,c,a);}
                                                    7c651b78  mr r5,r3
  int    t8(double a,float b,double c){return gdfd(a,b,c);}
                                                    48000000  b gdfd
```

Read them as a set — no one of them carries the rule:

* **`t6` is the discriminator for the FP file.** `b` and `c` are the callee's
  first two `float` arguments and they are already in f1 and f2, so the only
  instruction is the `int`. Under the index rule `b` is f2 and `c` is f3, and the
  body needs two `fmr`s that c2 does not emit.
* **`t5` is the discriminator for the GPR file.** `gfi(float,int)` puts its
  `int` in **r4**, not r3: the `float` took slot 1 and left the register empty.
  A model that packed the GPRs emits `b` with no move at all.
* **`t4` is the control** — the same two types in the other order, and *nothing*
  moves. Together `t4`/`t5` say the slot numbering is real and not an artifact of
  either function's own shape.
* **`t8`** puts `double, float, double` in f1, f2, f3. Under a "double takes two
  FPRs" rule (true of some other PowerPC ABIs) the third argument is f4 and this
  body needs three moves.

The same for the leaf class, where the value is returned rather than passed
(`fixtures/cpp/w27_fp_reg.cpp` is one function per row):

```text
  float f1_2(float a, float b)              { return b; }   fc201090  fmr f1,f2
  float f1_3(float a, float b, float c)     { return c; }   fc201890  fmr f1,f3
  double d1_2(double a, double b)           { return b; }   fc201090  fmr f1,f2
  float m_1(int k, float a)                 { return a; }   (nothing)  blr
  float m_3(float a, int k, float b)        { return b; }   fc201090  fmr f1,f2
  float w_1(double a, float b)              { return b; }   fc201090  fmr f1,f2
  float mixfp(int a, float b, float c)      { return b*c; } ec2100b2  fmuls f1,f1,f2
  float unused(float a,float b,float c)     { return a+b; } ec21102a  fadds f1,f1,f2
  float S::m1(float a, float b)             { return b; }   fc201090  fmr f1,f2
```

`m_1` and `m_3` are the pair that matters: a non-FP formal **before** the FP one
does not move it, and a non-FP formal **between** two FP ones does not move the
second. `S::m1` is byte-identical to its free-function twin, which is how `this`
is established to be outside the FP file.

**`fmr` is primary 63 whatever the width.** `f1_2` and `d1_2` emit the identical
`fc201090`; there is no `fmrs`, though the A-form arithmetic really does switch
to primary 59 for single precision. A register move is a bit copy and the FPRs
hold double internally.

### 1.1 Permutations, and where the modelled class stops

```text
  float c2s(float a,float b) { return g2f(b,a); }
     fc001090 fmr f0,f2  ·  fc400890 fmr f2,f1  ·  fc200090 fmr f1,f0
  float c3(float a,float b,float c) { return g3f(b,c,a); }
     fc001090 · fc401890 · fc600890 · fc200090      (a 3-cycle through f0)
  int both(int a,int b,float c,float d) { return gif2(b,a,d,c); }
     fc001090 fmr f0,f2 · 7c8b2378 mr r11,r4 · 7c641b78 mr r4,r3
     fc400890 fmr f2,f1 · 7d635b78 mr r3,r11 · fc200090 fmr f1,f0
```

The FP file's cycle scratch is **f0** exactly as the GPR file's is r11, and the
shapes match one for one. But `both` shows the two files' move sequences
**interleaved** — save-FP, save-GPR, move-GPR, move-FP, restore-GPR, restore-FP —
which is a scheduling decision no per-file solver reproduces, and the 8-argument
case hoists all four saves into a group first. Nothing here models a permutation
in both files at once, and §5 ranks it.

## 2. Conversions at the boundary — the asymmetry is c2's, not the C standard's

```text
  double wid(float a) { return gd1(a); }   48000000  b gd1        (NOTHING)
  float  nar(double a){ return gf1(a); }   fc200818  frsp f1,f1
                                           4bfffffc  b gf1
```

`float` → `double` is **free** — an FPR already holds double — and `double` →
`float` is a real `frsp`. The two are spelled with the same `2C <TYPE> 00` in the
IL, so this is `GAPS.md` §6's recurring shape again: one field, two facts,
indistinguishable until an instruction separates them. Both are refused by W27
and W28; §5 ranks admitting the free half.

## 3. The FP store leaf — MEASURED

`void f(S* s, float v) { s->f = v; }` is one `stfs`/`stfd` and a `blr`, and it is
the **fourth** consumer of the sub-object designator the indirect-load leaf
(`lwz`), the address leaf (`addi`) and the integer store leaf (`stw`) already
share — so it needed no new address decode at all.

```text
  void s_f (S* s, float v)            { s->f = v; }      d0230004  stfs f1,4(r3)
  void s_d (S* s, double v)           { s->d = v; }      d8230008  stfd f1,8(r3)
  void s_e2(S* s, float v)            { s->arr[2] = v; } d0230018  stfs f1,24(r3)
  void s_pf(float* p, float v)        { *p = v; }        d0230000  stfs f1,0(r3)
  void s_arg2(int x,S* s,float v)     { s->f = v; }      d0240004  stfs f1,4(r4)
  void s_arg3(int x,int y,S* s,float v){ s->f = v; }     d0250004  stfs f1,4(r5)
  void s_two (S* s,float u,float v)   { s->f = v; }      d0430004  stfs f2,4(r3)
  void s_twou(S* s,float u,float v)   { s->f = u; }      d0230004  stfs f1,4(r3)
  void s_mix (S* s,float u,int k,float v){ s->f = v; }   d0430004  stfs f2,4(r3)
  void M::set2(int k, float v)        { m = v; }         d0230000  stfs f1,0(r3)
  void s_base(D* d, float v)          { d->bf = v; }     d0230000  stfs f1,0(r3)
```

`s_two`/`s_twou`/`s_mix` grade the FP numbering and `s_arg2`/`s_arg3` grade the
GPR one, in the same production — **this is the shape where both rules are
exercised by one instruction**, which is why the fixture is worth more than its
size. `stfs`/`stfd` are primary 52/54 and both plain D-form, so unlike the
integer `std` (DS-form) there is no displacement-alignment gate.

Refused, each because a capture shows it emits something else:

```text
  void s_narrow(S* s, double v) { s->f = v; }  fc000818 frsp f0,f1
                                               d0030004 stfs f0,4(r3)   <- from f0
  void s_widen (S* s, float v)  { s->d = v; }  d8230008 stfd f1,8(r3)   <- free
  void s_lit(S* s) { s->f = 1.5f; }            3d600000 lis r11,0    REFHI __real@3fc00000
                                               c00b0000 lfs f0,0(r11) REFLO
                                               d0030004 stfs f0,4(r3)
```

An **integer** literal in the same position is admitted (`s->a = 7` is
`li r11,7 ; stw r11`); an FP one is three instructions, an `.rdata` COMDAT and
four relocations. That is the one place the two store paths differ in what a
literal costs.

## 4. `_fltused` — the eleventh live wrong-bytes emit, found by this rung

A translation unit that touches floating point carries an undefined external
`_fltused`, placed immediately after the **first FP-touching function's complete
symbol group**. The port had that rule and applied it to
`coff::Function::is_float`, which was set from `float_leaf.is_some()`.

`is_float` was carrying two facts: *"this body does FP arithmetic, so its label
stride is 2"* and *"this translation unit needs the CRT's float-support hook"*.
Every function that had ever set it satisfied both, because the only FP class the
port had was the W13 arithmetic leaf. **An FP store satisfies only the second.**
The port emitted all fourteen positive FP-store objs one symbol short —
`Port=Mismatch @ offset 12`, the COFF header's `NumberOfSymbols` — on the first
run of the fixture that could see it.

The ordering was then captured rather than assumed, because "the first
FP-touching function" and "the first FP-arithmetic function" also agree on an
all-FP-store TU:

```text
  int a_int; void b_fps; int c_int; void d_fps            -> _fltused at symbol 17,
                                                             i.e. after b_fps
  void a_fps; int b_int; float c_leaf                     -> _fltused at symbol 14,
                                                             after a_fps, AHEAD of
                                                             the arithmetic leaf
```

`fixtures/cpp/w28_fltused_order.cpp` is the first case; the sweep permutes four
such functions. `IlFunction::touches_floating_point` is now the one producer and
`label_slots` the other, one reader each.

This is the tenth-and-eleventh instance of the same shape, and the tell was
available for free and in the usual place: **`is_float` had one producer and two
consumers that wanted different questions answered.**

## 4.1 The label-counter stride — the twelfth mis-emit, found by the merge

Splitting `is_float` fixed the `_fltused` consumer and left the other one.
`IlFunction::label_slots` still read `float_leaf`, so the FP **store** leaf got a
compiler-label stride of 1 where c2 gives it 2, and the framed function
downstream got the wrong `$M`/`$T` numbers. MEASURED as the three-way capture
that separates the two candidate rules — one leaf ahead of one framed function,
reading the framed function's labels:

```text
  void lead(S* s, int v)      { s->i = v; }     $M2558 $M2559 $T2560
  void lead(S* s, float v)    { s->f = v; }     $M2559 $M2560 $T2561
  float lead(float a, float b){ return a * b; } $M2559 $M2560 $T2561
```

**The stride goes with the register file, not with the body shape.** Unlike the
arithmetic leaf — whose stride is 2, 4 or 6 depending on how many constants it
pools, and which therefore reports *undetermined* — the FP store leaf refuses its
pooled-constant cases in the parser, so 2 is exact.

The pair is only reachable since Class A many-calls (#35 step 2): the counter has
an observable effect only when a framed function follows, and before that no
framed shape could share an in-class TU with an FP store. **Neither branch's
fixtures contained it, and both branches were green.** Found by compiling the
cross product of the two rungs as the first thing done to the merged tree; the
sweep now generates it. `GAPS.md` §6 (12), `fixtures/cpp/w28_fp_store_framed_neg.cpp`.

With the stride told truthfully the TU-level gate refuses the pair, because
`coff::plan_labels` advances by 1 for every non-framed function regardless of its
class. That is an honest refusal and costs **0 functions** on the workload; §5
ranks admitting it.

## 5. What is NOT established, and what it costs — ranked

| item | size | what stops it |
|---|---:|---|
| the FP **tail call**, multi-argument | **26,136** measured, strictly (was "85,231" for the whole family; the single-argument half landed as W31, +58,135) | the two files' *interleaved* move schedule (§1.1) — so **split it**: an all-FP-argument call has no GPR moves to interleave with and is a strictly smaller claim |
| `2C` float→double in any position | not separated | free at the call boundary (§2) and at a store; the *narrowing* twin is a real `frsp` through f0 and the IL spells both the same |
| a pooled FP constant under `/Gy` | 1 on the workload | `.rdata` COMDAT association under function-level linking; W27 holds the pooled-constant population at exactly what it was to keep the census/gate disagreement at 0 |
| a permutation in **both** register files | inside the 85,231 | the two files' moves interleave on a schedule (§1.1) that no per-file solver reproduces |
| more than one FP cycle | inside the 85,231 | c2 hoists every save into a group first (the 8-argument capture) |
| `__vector` / VMX128 formals | unmeasured | a **third** register file; `docs/ABI_EDGES.md` §5 has it unprobed, and `.sy` class `D` is it (`arg_classes` refuses under `param-kind-unknown`) |
| a `.sy` formal class outside the whitelist | 0 observed | refused rather than assumed to be a GPR |
| an FP function sharing a TU with a **framed** one | 0 on the workload | `coff::plan_labels` advances the compiler-label counter by 1 per non-framed function; admitting a stride-2 leaf beside a framed one needs the planner to take a per-function stride, which is the framed side's label model rather than an FP class (§4.1) |

Also not established: whether an FP formal past f13 is stack-homed the way the
GPR one is past r10 — the 14-parameter capture frames and spills through
`lfs`/`stfs`, so it is a frame, not a leaf, and both rungs refuse above f13.

## 6. The 167,021 claim, measured

`docs/IL_STORE_LEAF.md` §7.1 recorded that the `calls-1` mass behind
`expr-load-type-8645` and `-8885` — **167,021** functions, 99.9 % of those two
rows' non-`calls-0` content — is "a `2C`-converted FP value in a call-argument
region … a single-call body with an FP argument is a tail call … decoding the FP
value class is what makes that 167,021 reachable". That was inferred from a
counterfactual's residue, not measured.

**Measured, by whole-body counterfactual** (scratch, reverted; `eat_int_like`
widened to accept any TYPE of kind class 5 at width 4 or 8, so the FP type is
admitted at the LOAD, the LIT, the `2C` target, the `55` call-end *and* the `41`
result annotation at once — the maximally lax version, and therefore an upper
bound; `parse_segment_detail` sinks every body that fired it under its own census
key so nothing is claimed in class, and the numerator stays at 473,611 in every
build):

| functions | shape that parsed | frame class |
|---:|---|---|
| **59,095** | `IntTailCall` — one argument | `calls-1` |
| **26,136** | `MultiArgTailCall` — a permutation | `calls-1` |
| 1,004 | `StraightLine` — the `fmr`, W27's own rung | `calls-0` |
| 2 | other | `calls-0` |
| **86,237** | total whole-body complete | |

So of the 167,021: **85,231 become whole-body complete, 51.0 %.** The claim is
**confirmed in kind and halved in size** — the population really is FP tail
calls, and really is not a frame problem (0 of it is `calls-2plus`), but the
other 81,790 block on something else and no FP rung reaches them.

**What the probe can and cannot see.** It admits the FP type at statement *and*
expression positions simultaneously, so it is not the `expr-op-0x27` failure
`docs/ROADMAP.md` §6i records (a token admitted where only an expression could
finish). What it cannot see is any *codegen* fact the grammar does not
distinguish — §1.1's two-file interleaving is exactly that, and it is inside this
number. It is also lax about conversions: with one gate for every position, a
cross-width `2C` is admitted, and §2 shows one direction of that is a real
`frsp`. **85,231 is an upper bound, and the strict rung is smaller by an
unmeasured amount.**

> **Measured 2026-07-31, and the upper bound was nearly tight.** W31 shipped the
> single-argument half: 59,095 lax → **58,135 strict, 98.4 %**, so the whole
> laxness residue is **960 functions (1.6 %)**. Decomposed by counterfactual:
> same-width move 55,924, free `float`→`double` widening 2,211, `frsp` narrowing
> **0**. The multi-argument half re-measures strictly at **26,136**. So "smaller
> by an unmeasured amount" resolved to 1.6 %, not to the large discount the
> hedge implied — worth recording, because the *conservative* reading of a lax
> counterfactual was itself wrong by more than the laxness was. A separate run admitting FP *literal* types as well
(`expr-lit-type-864A`, 10,665 released) added **0** whole bodies, which is worth
recording as its own refutation: the FP constant machinery buys nothing here.

The rows themselves are unmoved by W27/W28 — `expr-load-type-8645` fell by
exactly the 1,004 of the `fmr`, and the FP store came out of `expr-op-0x27`
instead, because a store's parse blocks at the `27` offset-add long before it
reaches the value's type. That is `GAPS.md` §6's unstable-attribution rule
paying off in the predicted direction, and it is why the estimate for both rungs
was taken from a counterfactual rather than from a row size.

## 7. Estimates against outcomes

| rung | estimate | outcome | bias, and its cause |
|---|---:|---:|---|
| W27 the `fmr` | **+1,005** | **+1,004** | HIGH by 1, cause named in advance: the pooled-constant clause (§5) holds that population fixed to keep the census/gate disagreement at 0, and it cost exactly the 1 function that had exposed it |
| W28 the FP store | **+7,984** | **+7,927** | HIGH by 57 (0.7 %), from the FP-literal and conversion refusals the counterfactual did not gate |
| both | +8,989 | **+8,931** | |
| **W31 the FP tail call** | **+34,000** | **+58,135** | **LOW by 24,135 (1.71×) — the first UNDER-estimate of the series, and it was recorded in advance as biased HIGH.** Every cause named in advance (computed FP arguments, cross-file conversions, result conversions, `arg_classes`) is real, and together they are the entire 960-function residue. The error was one level up: the bucket was treated as a sample of FP call sites, when it is a sample of FP call sites **that already pass `arg_loads_are_formals`** — and that filter had already removed the population being subtracted for. What survives such a filter is forwarding shims. Generalization worth keeping: **when estimating off a bucket, ask what the bucket was already filtered by, or every deduction gets taken twice.** |

Census **473,611 → 482,542 (19.23 % → 19.60 %)**, mismatch 0, disagreement 0,
**570 keys unchanged**, and the sum of the blocker-key deltas is exactly −8,931
over exactly two keys — the eighth rung running where the bucket drop equals the
gain to the function. All 8,931 are `calls-0`.

## 8. Reproduction

```sh
# the ground truth (gitignored scratch; every source is in a fixture header)
scripts/gt_capture.sh /tmp/fpgt/p1.cpp /O1 /GS- /c && scripts/gt_dump.py /tmp/fpgt/p1.obj --text-only
#   p1  FP tail calls: identity, second-FP-formal, swap, int/FP mixes, both widths
#   p2  3-cycle, both-file permutation, the frsp pair, 8 and 14 FP arguments, a member
#   p3  the leaf class: every position, every mix, the two recorded mis-emits
#   p4  the FP store: both widths, every offset form, both files' numbering, the refusers
#   flt/flt2  `_fltused` placement in a MIXED translation unit

# the `.sy` side — six formal types in one function, which is what separates the
# type KIND from the per-TU tid
c2rs census /tmp/fpgt_sy.cpp --keep-il /tmp/fpil   # then hexdump the .sy

# the counterfactual (scratch, reverted; nothing is claimed in class — the
# numerator stays at 473,611 in every build):
#   in `readers::eat_int_like`, fall through to a widening that accepts any TYPE
#   with kind class 5 at width 4 or 8, set a thread-local when it fires, and in
#   `parse_segment_detail` convert any Ok(shape) whose flag is set into an Err
#   tagged by the BodyShape.        -> 86,237, decomposed in §6
#   the same with the FP LITERAL classes (kind class A) added   -> +0 bodies
#   the FP store: the same tripwire in `store_value_width`      -> 7,984
```
