# Argument and return conventions at the edges

What the X360 MSVC ABI actually does for the types `CODEGEN_PPC_MVP.md`'s
"Args: `r3..r10`, return `r3`" line does not cover: 64-bit integers, floating
point, and structs by value. Same evidence standard as
`docs/CODEGEN_FRAMED_CALLS.md` — every row is bytes out of a real obj
(`cl.exe` 16.00.11886.00 under wibo 1.0.1-23, `/O1 /GS- /c`), captured with
`scripts/gt_capture.sh` and read with `scripts/gt_dump.py`.

The slot numbering used throughout: **argument *k* (1-based) owns the doubleword
at `SP + 8 + 8k`** of the caller's frame, whether or not it is passed in a
register (`CODEGEN_FRAMED_CALLS.md` §1.1).

---

## 1. Integers

* **`long long` is one 64-bit GPR, not an r3:r4 pair.** This is a 64-bit core.
  `long long g(long long); long long f(long long a){ return g(a)+1; }` emits the
  Class-A frame and a plain `addi r3,r3,1` — byte-identical to the `int`
  version except for the mangled names (`?f@@YA_J_J@Z`).

  ```
  int g(int,long long,int,long long,int);
  int f(int a,long long b){ return g(a,b,a,b,a)+1; }
     000c  7c671b78  mr r7,r3        arg5 = a
     0010  7c862378  mr r6,r4        arg4 = b   (one register, not two)
     0014  7c651b78  mr r5,r3        arg3 = a
  ```

  A `long long` consumes exactly one slot. Slot 4 is r6 with a `long long` in
  slot 2, so nothing is skipped.

* **`char` / `short` / `bool` are returned in r3 and re-normalized by the
  consumer**, not by the producer's caller-side. `return (char)(gc(a)+1)`:

  ```
     0010  7c6b1b78  mr    r11,r3
     0014  396b0001  addi  r11,r11,1
     0018  7d630774  extsb r3,r11        extsh for short
  ```

  `bool` uses the `clrlwi/cntlzw/rlwinm` materialization for `!x`
  (`546b063e 7d6b0034 5563dffe`).

* **Beyond r10 the slot goes on the stack, in the low word.** With a 9-argument
  callee: `stw r11,84(r1)` for a 4-byte int in slot 9 (`SP+80`), and the frame
  does **not** grow (`F` stays 96) because the 8-slot floor already covers
  through SP+80. The 10th argument (`SP+88`) is the first that grows it.

---

## 2. Floating point

* **FPRs are assigned sequentially f1, f2, f3, … to the floating-point
  arguments in order**, independent of the argument's *position*. The
  discriminating capture:

  ```
  int g(double,int,float,int,double);
  int f(double a,int b,float c){ return g(a,b,c,b,a)+1; }
     000c  7c862378  mr  r6,r4        arg4 (int)    -> slot 4 = r6
     0010  fc600890  fmr f3,f1        arg5 (double) -> 3rd FP argument = f3
  ```

  f's own parameters are a→f1, b→r4, c→**f2** (2nd FP parameter, slot 3). The
  call needs a→f1 ✓, b→r4 ✓, c→f2 ✓ and a→f3, which is the single `fmr`. A
  positional model (`slot 3 → f3`) predicts a move for `c` and none for `a`, and
  is refuted by these two instructions.

* **A prototyped FP argument does NOT also fill its GPR slot.**

  ```
  int g(double,int); int f(int b,double a){ return g(a,b)+1; }
     000c  7c641b78  mr r4,r3        only the int moves; r3 is left alone
  ```

* **A variadic FP argument DOES**, via a memory round-trip through its own
  outgoing slot:

  ```
  extern "C" int printf(const char*,...);
  int f(double a,int b){ return printf("%f %d",a,b)+1; }
     000c  d8210018  stfd f1,24(r1)     slot 2 = SP+24
     0010  7c852378  mr   r5,r4         arg3 = b -> r5
     0014  e8810018  ld   r4,24(r1)     slot 2 also materialized in r4
  ```

  So the ellipsis rule is "register *and* slot", the prototyped rule is
  "register only". A port that models one and applies it to both emits code that
  links and misbehaves.

* **The two numberings, stated together**, since the pair is what a port needs:
  an FP argument takes `f<j>` counting FP arguments alone, and every other scalar
  takes `r<2 + k>` where `k` is its **slot** — which an FP argument advances even
  though it fills no register. `int t5(int a, float b){ return gfi(b,a); }` emits
  `mr r4,r3`: the callee's `int` is argument 2, so slot 2, so r4. That is the
  discriminator for the *second* rule, as the `t6` capture above is for the
  first, and a model with only one of them gets half the corpus wrong. Both are
  implemented as `c2_il::func::sy::ArgClass` and graded by
  `fixtures/cpp/w27_fp_reg.cpp` and `w28_fp_store.cpp`; the full capture set is
  `docs/CODEGEN_FP_ARGS.md` §1.

* **A `float` → `double` widening at a call boundary is FREE and the narrowing is
  not.** `double wid(float a){ return gd1(a); }` is a bare `b`;
  `float nar(double a){ return gf1(a); }` is `frsp f1,f1` then the branch. The IL
  spells both with the same `2C`, so the asymmetry is c2's and not the C
  standard's, and a port that treats the conversion as an annotation drops a real
  instruction (`CODEGEN_FP_ARGS.md` §2).

* **The FP file is width-agnostic**: a `double` takes one register, not two.
  `int t8(double a, float b, double c){ return gdfd(a,b,c); }` is a bare branch,
  which refutes the register-pair rule some other PowerPC ABIs use.

* **Returns**: `float` and `double` both in **f1**. `float` constants come from
  an `.rdata` COMDAT via REFHI/REFLO into r11 (`OBJ_GY_SHAPES.md` §2).

* An FP argument still **consumes its slot for frame-sizing purposes**:
  `g(int×8, double, double)` gives `nOutSlots = 10` and `F = 112`, matching
  `CODEGEN_FRAMED_CALLS.md` §1.2 with the FP arguments counted.

---

## 3. Structs by value

**Structs are passed by value in GPRs, doubleword by doubleword — there is no
by-reference threshold and no caller-side copy up to 64 bytes.**

| struct | size | passed as |
|---|---|---|
| `struct S1{char c;}` | 1 | r3 (pure passthrough — says nothing about justification) |
| `struct S3{char a,b,c;}` | 3 | r3, **left-justified** (see below) |
| `struct S2{int a,b;}` | 8 | r3 |
| `struct SF{float a,b;}` | 8 | **r3** — not FPRs |
| `struct SD{double a,b;}` | 16 | **r3,r4** — not FPRs |
| `struct SM{int a;double b;}` | 16 | r3,r4 |
| `struct S6{int v[6];}` | 24 | r3,r4,r5 |
| `struct S12{int v[12];}` | 48 | r3..r8 |
| `struct S20{int v[20];}` | 80 | r3..r10 + `memcpy` of the tail 16 B to SP+80 |

Byte evidence for the chunking, `int f(S12* p){ return g12(*p)+1; }`:

```
   000c  7c6b1b78  mr r11,r3
   0010  e8630000  ld r3,0(r3)
   0014  e88b0008  ld r4,8(r11)
   0018  e8ab0010  ld r5,16(r11)
   001c  e8cb0018  ld r6,24(r11)
   0020  e8eb0020  ld r7,32(r11)
   0024  e90b0028  ld r8,40(r11)
   0028  4bffffd9  bl ?g12
```

and for the overflow at 80 bytes (`S20`, `F = 112`, `nOutSlots = 10`):

```
   0014  38610050  addi r3,r1,80          &outgoing slot 9
   0018  389f0040  addi r4,r31,64         &src[64]
   001c  38a00010  li   r5,16
   0020  4bffffe1  bl   memcpy            the tail only
   0024  e87f0000  ld   r3,0(r31)         the first 8 slots stay in registers
   ...
   0040  e95f0038  ld   r10,56(r31)
```

Note `memcpy` is an emitted external — a struct-argument rung drags in a symbol
the IL never names.

**Small structs are left-justified in the register.** `S3` (3 bytes) is
reassembled with an explicit `sldi r3,r11,40` = shift left by `64 - 24`:

```
   000c  f8610070  std   r3,112(r1)     home the incoming doubleword
   0010  89410072  lbz   r10,114(r1)    byte 2
   0014  a1610070  lhz   r11,112(r1)    bytes 0..1
   0018  796b47e4  rldicr r11,r11,8,63
   001c  7d4b5b78  or    r11,r10,r11
   0020  796345c6  sldi  r3,r11,40      re-left-justify into r3
```

---

## 4. Struct returns

* **≤ 8 bytes: in r3.** `S2 f(int a){ S2 s=g2(a); s.a++; return s; }` never
  touches memory beyond a scratch slot and ends with `ld r3,80(r1)`.
* **> 8 bytes: a hidden first pointer in r3, which the function also returns
  in r3.** `S12 f(int a){ S12 s=g12(a); s.v[0]++; return s; }`, `F = 144`:

  ```
     0008  fbe1fff0  std  r31,-16(r1)
     0010  7c7f1b78  mr   r31,r3          save the incoming sret pointer
     0014  38610050  addi r3,r1,80        a local 48-byte buffer for g12's sret
     0018  4bffffe9  bl   ?g12
     001c  7c641b78  mr   r4,r3           g12 returned its sret pointer in r3
     0020  7fe3fb78  mr   r3,r31
     0024  38a00030  li   r5,48
     0028  4bffffd9  bl   memcpy
     ...
     0030  7fe3fb78  mr   r3,r31          return the sret pointer
  ```

  The hidden pointer occupies slot 1, so a 2-argument source call has
  `nOutSlots = 3`; the frame above is `align16(max(80, 80+48) + 8 + 8) = 144`,
  which is `CODEGEN_FRAMED_CALLS.md` §1.2 with the sret buffer counted as a
  local.

---

## 5. Not determined

* The exact size cut for "structs go via `memcpy`" — measured only at 80 bytes
  (over the 8-slot floor). 72 bytes was not captured, so whether the trigger is
  "> 64 bytes" or "> `8*nOutSlots` bytes for this call" is open.
* Whether a struct containing only `float`s ever reaches an FPR under some
  other flag combination. Under `/O1` it does not (`SF`, `SD`, `SM` above), and
  no `/fp:` variant was probed.
* `__vector`/VMX128 types: ~~not probed at all~~ — **probed 2026-07-31, enough
  to size it; see §5.1.** The stated *reason* for refusing was wrong.
* Sign/zero-extension duty for a `char`/`short` **argument** (returns are §1).
  Not isolated — every probe passed `int`.
* Bitfields, unions, and structs with a non-trivial copy constructor (which
  should force a caller-side temporary and a constructor call, not a `memcpy`).

---

## 5.1 VMX128 — sized, 2026-07-31

`__vector4` is not a compiler keyword; it is the XDK's
`typedef struct __declspec(intrin_type) __declspec(align(16)) { union { float
v[4]; unsigned u[4]; }; } __vector4;` (`src/xdk/LIBCMT/vectorintrinsics.h` in
the workload tree). Reproduced verbatim in a scratch header, every row below is
bytes from a real obj at `/O1 /GS- /c`.

**The one fact that changes the sizing, and it is the opposite of what §5 said.**

```c
  int gmix(int,int,int);
  int fmix(int k, __vector4 a, __vector4 b, int j, int m){ return gmix(m,j,k); }
     7cab2b78  mr r11,r5        m
     7c651b78  mr r5,r3         k
     7d635b78  mr r3,r11        m
     4bfffff4  b  ?gmix

  float gf(float,float);
  float ffm(__vector4 a, float x, __vector4 b, float y){ return gf(y,x); }
     fc001090  fmr f0,f2   ·  fc400890  fmr f2,f1  ·  fc200090  fmr f1,f0
```

`j` is in **r4** and `m` in **r5**, not r6/r7; `x` is in **f1** and `y` in
**f2**. So:

> **A `__vector4` formal takes a vector register and consumes NEITHER a GPR
> argument slot NOR an FPR one.** It is a genuinely independent third file.

§5's refusal was correct; its stated reason — *"a third register file would
renumber **both** of the numberings above for every argument after it"* — is
**refuted**. It renumbers *neither*. This matters because it is the cheap case:
`SyView::arg_classes` can classify a `.sy` class-`D` formal as "vector, one
register, no slot" without touching the GPR or FPR numbering at all. It is
strictly unlike the FP file, which does consume a GPR slot
(`docs/CODEGEN_FP_ARGS.md` §0) — the two look alike and behave differently.

Also measured:

* **Numbering and return.** `__vector4 f(__vector4 a){ return g1(a); }` is a
  bare `b ?g1` — the first vector formal is **v1** and the return value is
  **v1**. Vector formals number v1, v2, v3, … over the vector parameters alone,
  the same shape as the FP file.
* **Permutations use the same cycle machinery.** A 2-swap is 3 instructions and
  a 3-cycle is 4, matching the GPR file's r11 and the FP file's f0 shapes one
  for one. A mixed int+vector permutation emits the two files' moves
  **interleaved** (`v ; gpr ; v ; gpr ; gpr ; v` for
  `gm(j,b,k,a)`), which is `docs/CODEGEN_FP_ARGS.md` §1.1's open scheduling
  problem with a **third** file in it, not a new problem.
* **Saved vector registers are 16 bytes and inline at 2.**
  `__vector4 fs2(__vector4 a,__vector4 b){ __vector4 t=g1v(a); __vector4
  u=g1v(b); return g1v(t); }` frames at **128** with a 7-word prologue:
  `mflr r12 ; stw r12,-8(r1) ; li r12,-48 ; stvx v30 ; li r12,-32 ; stvx v31 ;
  stwu r1,-128(r1)`, and `.pdata` `40001607`. `align16(80 + 2·16 + 8) = 128`, so
  `CODEGEN_FRAMED_CALLS.md` §1.2's frame rule holds with the saved-vector term
  at **16** bytes per register rather than 8, and the vector save area starts at
  **-32** (the first 16-aligned slot below the LR word), not -24. The epilogue
  restores through **r0**, not r12 — r12 is holding LR.
* **The argument file is narrower than 14.** A call with 14 vector arguments
  frames at 256 and homes two of them to the stack through
  `addi r12,r1,480 / stvx` and `li r12,224 / stvx`; `.pdata` `40000c03`.

Not determined, and each of these is a rung's worth on its own:

* the exact width of the vector argument file (12 or 13) and the stack-homing
  rule past it — two spills at 14 arguments is consistent with both a 12-wide
  file and a 13-wide one plus a return-slot effect;
* whether a `__save…vmx` helper pair exists above some saved-register threshold,
  as the GPR (3) and FPR (4) files have. Two saved vector registers are inline;
  nothing above that was probed, and the label-slot consequence would follow
  `docs/LABEL_COUNTER.md`'s +2-per-helper-pair rule if one does;
* which callee-saved vector registers are allocated in what order (v31, v30 …
  descending is *consistent with* the one capture and is not established by it);
* every VMX128 *instruction* encoding. `llvm-mc -triple=powerpc` mis-decodes
  them — the 360's VMX128 splits the register number across non-standard bit
  fields — so `scripts/gt_dump.py`'s disassembly column is unreliable for any
  `.text` containing vector code and the raw words must be read instead. That
  is a tooling gap to close before any VMX rung, not a compiler fact.
