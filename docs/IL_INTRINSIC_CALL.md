# `.ex` opcode `0x40` — the INTRINSIC CALL

**Status: characterization + a fail-closed decode (P2e).** The only Rust change is
diagnostic: `crates/c2-il/src/func.rs` now decodes the selector so the census
reports *which* intrinsic instead of one opaque byte. **Nothing was lowered** —
every `0x40` still returns `NotImplemented`, and the measured in-class count is
unchanged to the function (§7).

Every claim below cites bytes from a live `16.00.11886.00` capture. Claims from a
**controlled fixture** (one construct varied, everything else held fixed) are
marked **[CF]** and name the tracked fixture; claims from real dc3 translation
units are marked **[DC3]**. Unknowns are written UNKNOWN.

Tracked fixtures added by this work — all five are **negatives**, and each one
separates a hypothesis a previous reading of this opcode got wrong:

| fixture | what it separates |
|---|---|
| `il_intrinsic_nullary.cpp` | `40 <TYPE>` vs `40 <TYPE> <varint>` — the trailing field |
| `il_intrinsic_bits.cpp` | id per name *family* vs per signature; and which names are not intrinsics at all |
| `il_intrinsic_layout.cpp` | 2113 vs 2114 (the null guard); 2115's sign; `0x66`'s `02` as a count |
| `il_intrinsic_fold.cpp` | integer intrinsics fold over constants, floating ones do **not**; nested `0x40` |
| `il_intrinsic_byval.cpp` | ids 222/223 — trigger and literal pinned, semantics left UNKNOWN |

Reproduce:

```
cargo build --release -p c2-harness
./target/release/c2rs census  fixtures/cpp/il_intrinsic_layout.cpp --keep-il work/intr/il_lay
./target/release/c2rs compile fixtures/cpp/il_intrinsic_layout.cpp --keep-obj work/intr/lay.obj
./target/release/c2rs diff    fixtures/cpp/il_intrinsic_layout.cpp
# real workload
./target/release/c2rs census src/system/world/Dir.cpp \
    --flags-file work/dc3-workload/flags.txt --cwd ../dc3-decomp --keep-il work/intr/il_Dir
```

---

## 0. Headline

`0x40` is a **second CALL token** — the intrinsic call — occupying exactly the
slot `BD` occupies in an ordinary call (`docs/IL_CALL_GRAMMAR.md` §2). This
confirms and extends `docs/IL_CAST_CONVERT.md` §1, which established it is not a
cast. What is new here:

1. **The production is settled, including the field that is absent.** Two
   controlled *nullary* witnesses prove `40 <TYPE>` has no trailing field and that
   the argument list may be empty (§1).
2. **The id space of the real workload is closed.** Exactly **20 distinct
   selectors** occur across `Dir.cpp`, `App.cpp` and `Game.cpp`; **18 of the 20 are
   now named by controlled fixture** (§3, §4), including three that
   `docs/IL_CAST_CONVERT.md` left UNKNOWN (815 `_abs64`, 1948 `__mftb`, 2119
   `dynamic_cast`) and one it flagged as "implied, not proven" (337 `throw`).
3. **The bucket's true footprint is 16.1 % of blocked functions, not 9 %.** With
   the selector decoded, `expr-intrinsic-call` (213,411) resolves with **zero
   residue**, and 95.4 % of `call-token-0x33` (176,123) turns out to be the same
   production reached through a `26 <sym>` push. Total **381,488** blocked
   functions (§7).
4. **86 % of that is one family** — the class-layout adjustments 2113…2119, at
   329,205 functions (13.9 % of blocked functions). It is the single largest
   *semantic* gap in the workload.
5. **None of it is lowerable yet**, and §5 says exactly why for each subfamily.
   Three independent reasons, each pinned by a capture: the emission depends on
   *literal argument values* rather than the id; the destination register is chosen
   by the consumer, not the intrinsic; and the constant-folding rule differs
   between the integer and floating halves of the table.

Three corrections to `docs/IL_CAST_CONVERT.md` fall out (§4.1, §4.2, §5.3).

---

## 1. The production

```
INTRINSIC-CALL := 33 86 41 74 <varint id>     the selector (always int-typed)
                  40 <TYPE result>             the call token — NO trailing field
                  ( <expr> 55 <TYPE> )*        arguments, may be empty
                  4C                           apply
```

Compare `CALL := BD <TYPE ret> <flags:1> <varint fn-type-id> <args> 4C`. The
intrinsic token is strictly shorter: no calling-convention byte, no function-type
id. **The callee identity is not in the token at all** — it is the preceding int
literal.

### 1.1 The absent trailing field — **[CF] `il_intrinsic_nullary.cpp`**

A one-argument intrinsic cannot decide between `40 <TYPE>` and
`40 <TYPE> <varint>`: the varint would eat the first byte of the argument and the
parse would fail somewhere downstream, which is exactly how the field width of
`0x2C` was mis-modelled before. A **zero-argument** one decides it, because then
the `4C` apply sits immediately after the result type:

```
void n_break()    { __debugbreak(); }
  4c 4f 11 53 | 33 86 41 74 80 1f 02 00 00 | 40 82 07 03 | 4c 4b | 3a ff 09 …
                  selector 543                token, void   apply+discard
  -> 0fe00016   twi 31,r0,22

void *n_retaddr() { return _ReturnAddress(); }
  4c 4f 11 53 | 33 86 41 74 80 e5 00 00 00 | 40 86 43 83 08 | 4c | 41 86 43 83 08 | 3a 01 0a …
                  selector 229                token, void*     apply  result type
  -> 7c6802a6   mflr r3

unsigned __int64 n_mftb() { return __mftb(); }
  4c 4f 11 53 | 33 86 41 74 80 9c 07 00 00 | 40 88 82 23 | 4c | 41 88 82 23 | 3a 4f 0a …
  -> 7c6c42e6   mftb r3    (mfspr r3, SPR 268)
```

`n_retaddr` is the load-bearing one: its `4C` is bracketed on both sides by fixed
markers (the 4-byte `void *` result type before, the `41` result annotation
after), so it cannot be a varint payload. Unit test:
`intrinsic_call_token_has_no_trailing_field`.

**[DC3]** `Dir.cpp` fn683 shows the same nullary shape
(`33 86 41 74 80 9c 07 00 00 40 88 82 23 4c`), 15 sites across the three TUs.

### 1.2 The selector sits in the callee slot, not the argument list

Structural, and independent of the histogram in
`docs/IL_CAST_CONVERT.md` §1.2: every real argument is terminated by
`55 <TYPE>`, and the selector literal is **not**. In `t_abs`,
`33 86 41 74 0f` is followed directly by `40`. So the literal is being consumed
*by* the token, the way `26 <callee>` is consumed by `BD` — it is not an argument
that happens to come first.

Measured at scale by the decoder itself: over 878 dc3 TUs, **213,411 of 213,411**
`0x40` sites in the operand stream were preceded by a well-formed
`33 86 41 74 <varint>`. Zero residue (§7). The decode declines on anything else
(`selector_must_be_exactly_int_typed_or_the_decode_declines`), so the residual
`expr-intrinsic-call` bucket is what measures this claim going forward.

### 1.3 Arguments, and their order

Argument sub-expressions are pushed in **reverse** of the notional argument list,
the same rule ordinary calls follow (`docs/IL_CALL_GRAMMAR.md` §5). Reading
`t_memcpy` (`memcpy(d, s, n)`) **[CF] `il_intrinsic_call.cpp`**:

```
33 86 41 74 80 ac 00 00 00  40 86 43 83 08          selector 172, returns void*
  33 86 41 74 01  55 86 41 74                       an alignment hint, int 1
  33 86 41 74 01  55 86 41 74                       a second alignment hint
  b9 <n> 86 42 75 55 86 42 75                       n
  b9 <s> 86 43 81 20 55 86 43 81 20                 s
  b9 <d> 86 43 83 08 55 86 43 83 08                 d
4C
```

so the notional list is `(d, s, n, align, align)`. The two alignment hints have no
counterpart in the source. `?t_memset` pushes one; **[DC3]** `Dir.cpp` fn931
pushes `04` instead of `01` when the operands are 4-byte aligned, and the
expansion changes with it.

### 1.4 The argument region can contain another `0x40` — **[CF] `il_intrinsic_fold.cpp`**

`docs/IL_CAST_CONVERT.md` §6 left this open ("not observed at an aligned site, but
not excluded"). `abs(abs(a))`:

```
33 86 41 74 0f  40 86 41 74            outer abs
  33 86 41 74 0f  40 86 41 74          inner abs
    b9 <a> 86 41 74  55 86 41 74
  4C                                   inner apply
  55 86 41 74                          inner result pushed as the outer's argument
4C
```

Nesting is outer-token-first, exactly as `26 <callee> BD` nests. A decoder that
tracks a single "current call" desynchronises; the production is properly
recursive.

---

## 2. Where the parser meets it — the two census sites

The production reaches `parse_segment_detail` at two distinct points, which is why
its footprint was split across two census buckets:

| site | opening bytes | old bucket | new bucket |
|---|---|---|---|
| operand stream | `LO 53 [4F 01 v]* 33 86 41 74 <id> 40 …` | `expr-intrinsic-call` | `expr-intrinsic-<sel>` |
| after a symbol push | `LO 53 … 26 <tok> 33 86 41 74 <id> 40 …` | `call-token-0x33` | `call-intrinsic-<sel>` |

The second is `parse_call_shape` reaching the slot where it expects `BD`. **[DC3]**
it is dominated by member calls whose `this` is an adjusted base pointer
(`26 <method> 33 … 2113 40 …`, 137,496 functions) — a shape
`il_intrinsic_layout.cpp::l_this2` reproduces exactly.

`0x66` never appears as a blocking bucket, because it only ever occurs *inside* a
`0x40` argument region and so is always shadowed. Same for selector 222, which
only occurs nested inside 223 (§5.3).

---

## 3. The id table — scalar and CRT intrinsics

Selector value, the exact selector bytes, and the `.text` c2 emits. All rows are
**[CF]** from the fixture named in the last column, with the mangled name read
from `.gl` and the instructions read off that fixture's reference obj.

| id | hex | intrinsic | selector bytes | c2 emits | fixture |
|---:|---|---|---|---|---|
| 15 | `0x00f` | `abs`, `labs` | `33 86 41 74 0f` | `srawi r11,r3,31 ; xor r10,r3,r11 ; subf r3,r11,r10` | bits |
| 17 | `0x011` | `fabs` | `33 86 41 74 11` | `fabs f1,f1` | call, bits |
| 159 | `0x09f` | `_rotl` | `… 80 9f 00 00 00` | `rlwnm r3,r3,r4,0,31` | bits |
| 160 | `0x0a0` | `_rotr` | `… 80 a0 00 00 00` | `subfic r11,r4,32 ; rlwnm r3,r3,r11,0,31` | bits |
| 164 | `0x0a4` | `strcpy` | `… 80 a4 00 00 00` | inline byte loop (8) | — (probe) |
| 165 | `0x0a5` | `strcmp` | `… 80 a5 00 00 00` | inline loop (11) | call |
| 166 | `0x0a6` | `strcat` | `… 80 a6 00 00 00` | inline loop | — (probe) |
| 167 | `0x0a7` | `strlen` | `… 80 a7 00 00 00` | inline loop (9) | call |
| 170 | `0x0aa` | `memcmp` | `… 80 aa 00 00 00` | inline loop (15) | call |
| 172 | `0x0ac` | `memcpy` | `… 80 ac 00 00 00` | `b <memcpy>` (REL24) | call |
| 173 | `0x0ad` | `memset` | `… 80 ad 00 00 00` | `b <memset>` (REL24) | call |
| 226 | `0x0e2` | `_InterlockedIncrement` | `… 80 e2 00 00 00` | 8-instruction `lwarx`/`stwcx.` loop | bits |
| 229 | `0x0e5` | `_ReturnAddress` | `… 80 e5 00 00 00` | `mflr r3` | nullary |
| 236 | `0x0ec` | `__emul` | `… 80 ec 00 00 00` | `extsw ; extsw ; mulld` | bits |
| 237 | `0x0ed` | `__emulu` | `… 80 ed 00 00 00` | `rldicl ; rldicl ; mulld` | bits |
| 318 | `0x13e` | `_InterlockedExchangeAdd` | `… 80 3e 01 00 00` | `lwarx`/`stwcx.` loop | — (probe) |
| 337 | `0x151` | C++ `throw` | `… 80 51 01 00 00` | `_CxxThrowException(&tmp, &throwinfo)` | — (probe) |
| 543 | `0x21f` | `__debugbreak` | `… 80 1f 02 00 00` | `0fe00016` `twi 31,r0,22` | nullary |
| 813 | `0x32d` | `_rotl64` | `… 80 2d 03 00 00` | `rldcl r3,r3,r4,0` | bits |
| 814 | `0x32e` | `_rotr64` | `… 80 2e 03 00 00` | `subfic r11,r4,64 ; rldcl` | bits |
| 815 | `0x32f` | `_abs64` | `… 80 2f 03 00 00` | `sradi r11,r3,63 ; xor ; subf` | bits |
| 839 | `0x347` | `_byteswap_ushort` | `… 80 47 03 00 00` | `rlwinm ; rlwimi ; or` | bits |
| 840 | `0x348` | `_byteswap_ulong` | `… 80 48 03 00 00` | `rlwinm ; rlwimi ×3 ; or` | bits |
| 841 | `0x349` | `_byteswap_uint64` | `… 80 49 03 00 00` | 14 instructions through `16(r1)` | bits |
| 850 | `0x352` | `_CountLeadingZeros` | `… 80 52 03 00 00` | `cntlzw r3,r3` | bits |
| 921 | `0x399` | `_CountLeadingZeros64` | `… 80 99 03 00 00` | `cntlzd r3,r3` | bits |
| 1935 | `0x78f` | `__frsqrte` | `… 80 8f 07 00 00` | `frsqrte f1,f1` | bits |
| 1937 | `0x791` | `__fsel` | `… 80 91 07 00 00` | `fsel f1,f1,f2,f3` | bits |
| 1948 | `0x79c` | `__mftb` | `… 80 9c 07 00 00` | `mftb r3` | nullary |
| 1973 | `0x7b5` | `sqrt` | `… 80 b5 07 00 00` | `fsqrt f1,f1` | call, bits |

337 (`throw`) was "implied, not proven" in `docs/IL_CAST_CONVERT.md` §1.3 because
the probe's `.gl` names did not resolve. Compiling `int thr(){ throw 3; }` with
`/GR /EHsc` resolves them: `?thr@@YAHXZ` is the body carrying selector 337, whose
arguments are a `26 <sym>` (the `throwinfo`) and the address of the stored literal.

### 3.1 The id is per name FAMILY, not per signature — **[CF] `il_intrinsic_bits.cpp`**

`abs(int)` and `labs(long)` both emit selector **15**, differing only in the TYPE
fields (result `86 41 74` vs `86 41 12`) and producing byte-identical `.text`. So
an allow-list keyed on the id alone cannot know the operand width; the TYPEs carry
it. (`_abs64` is a *separate* id, 815 — the family is per C name, and the 64-bit
names are their own family.)

### 3.2 A name that looks like an intrinsic usually is not — **[CF] `il_intrinsic_bits.cpp`**

Declared identically to their neighbours and compiled with the same flags,
`fabsf`, `sqrtf`, `_rotl16`, `_MulHigh`, `_MulUnsignedHigh`,
`_AddressOfReturnAddress`, `__lwsync`, `__sync`, `__isync`, `__eieio`, `__nop`,
`__fre`, `wcslen`, `memmove`, `memchr`, and every `<math.h>` function probed
(`sin`, `cos`, `atan`, `pow`, `log`, `exp`, `floor`, `ceil`, `fmod`) all compile
to **ordinary `26 <tok> BD … 4C` calls** with a REL24 relocation. No `0x40`
anywhere.

That is the argument for allow-listing from captures rather than from the CRT
header set: the table is internal to c1xx and its membership is not derivable from
the declaration. `il_intrinsic_nullary.cpp` and `il_intrinsic_bits.cpp` each keep
a handful of these near-misses precisely so a future "any extern with this shape"
rule has a case that refutes it.

---

## 4. The class-layout family — ids 2113…2119

**86 % of the whole bucket** (329,205 of 381,488 blocked functions, §7). All
**[CF] `il_intrinsic_layout.cpp`**.

```
33 86 41 74 <id>  40 <TYPE result>
  66 <n> <n × token>   55 86 41 74     the class descriptor, pushed as `int`
  ( 33 86 41 74 <off>  55 86 41 74 )*  k byte offsets — k is fixed per id
  <object expr>        55 <TYPE ptr>   the pointer being adjusted
4C
```

| id | construct | `66` arity | offsets | c2 emits |
|---:|---|---:|---:|---|
| 2113 | base adjust for a member call's `this` | 2 | 1 | `addi r3,r3,<off>` — **no null guard** |
| 2114 | derived → base pointer conversion | 2 | 1 | `cmplwi r3,0 ; addi r3,r3,<off> ; bclr 4,26 ; li r3,0 ; blr` |
| 2115 | base → derived conversion | 2 | 1 | as 2114 with `addi r3,r3,-<off>` |
| 2116 | virtual-base pointer conversion | 3 | 4 | `lwz r11,0(r3) ; lwz r11,4(r11) ; add r3,r11,r3`, plus an explicit null branch |
| 2117 | `&`member inherited from a non-virtual base | 2 | 2 | one `lwz` with both offsets folded into the displacement |
| 2118 | `&`member of a virtual base | 2 or 3 | 5 | vbtable indirection + `lwz r3,<off>(r10)` |
| 2119 | `dynamic_cast` | 2 | 0 | `b <__RTDynamicCast>`; its arguments are two `26 <sym>` RTTI pushes |

### 4.1 2113 vs 2114 is the NULL GUARD

`docs/IL_CAST_CONVERT.md` §1.4 recorded "2113: base adjustment feeding a member
call's `this`; as 2114" and listed "the exact distinction between 2113 and 2114
(identical argument shapes, different ids)" as an open unknown. It is the guard,
and `il_intrinsic_layout.cpp` holds both halves with **the same class-pair
descriptor and the same offset literal**:

```
l_up2    (A2 *)m    33 86 41 74 80 42 08 00 00  40 86 43 b1 20
                    66 02 92 20 93 20 55 86 41 74
                    33 86 41 74 >08< 55 86 41 74  b9 <m> … 4c
  -> 2b030000 cmplwi r3,0 ; 38630008 addi r3,r3,8 ; 4c9a0020 bclr 4,26
     38600000 li r3,0     ; 4e800020 blr

l_this2  m->mb()    33 86 41 74 80 41 08 00 00  40 a6 43 96 20
                    66 02 92 20 93 20 55 86 41 74
                    33 86 41 74 >08< 55 86 41 74  b9 <m> … 4c  99 … bd …
  -> 38630008 addi r3,r3,8 ; 4bffffc4 b <A2::mb>
```

One instruction against five plus a control-flow split, from one selector byte.
The language guarantees a member call's object is non-null; a pointer *conversion*
must map null to null. `l_up1` (offset `00`) shows the guard is elided when the
offset is zero: a bare `blr`.

Unit test:
`same_descriptor_and_offset_different_selector_is_a_different_emission`.

### 4.2 2115's offset is NOT pre-negated

`docs/IL_CAST_CONVERT.md` §1.4 said "mirror of 2114 with a negated offset". The
bytes refute it: `l_down2`'s offset literal is `08`, positive, byte-identical to
`l_up2`'s, and the **id** is what makes the emission `addi r3,r3,-8`. A decoder
that read the sign out of the literal would emit the adjustment backwards.

### 4.3 `0x66`'s second byte is an ARITY, not the constant `02`

`docs/IL_CALL_GRAMMAR.md` §7 ranked `0x66` as the #1 unidentified opcode (1148
Dir.cpp bodies) and `docs/IL_CAST_CONVERT.md` §6 recorded "the `02` is fixed in
every observation but its meaning is unknown". It is the number of class tokens
that follow:

```
l_up2  (A1/A2 pair)          66 >02< 92 20 93 20
l_upv  (DD/D1/V1 triple)     66 >03< b2 20 b4 20 b5 20
l_fldv (DD/D1/V1 triple)     66 >03< b2 20 b4 20 b5 20
```

So `0x66` is `66 <n> <n × token>` — a class-descriptor tuple, not a call. Its
arity moves with the *inheritance shape*, independently of the offset count (which
moves with the id).

### 4.4 The offset count is fixed per id

Captured across `il_intrinsic_call.cpp`, `il_intrinsic_layout.cpp` and two
untracked multiple/virtual-inheritance probes: 2113/2114/2115 → 1 offset, 2117 →
2, 2116 → 4, 2118 → 5, 2119 → 0 (two `26 <sym>` pushes instead). The production is
already self-delimiting through `55 <TYPE>` and `4C`, so this is a free
consistency check rather than something a decoder needs.

### 4.5 Where the null check lives depends on the base's virtualness

For 2114 the guard is synthesized by c2 and invisible in the IL. For 2116 c1xx
writes it into the IL itself — `l_upv` opens `b9 <p> <DD*> 33 86 41 74 00 1f`
(a `== 0` compare) and closes with `43 42 00 00`, the ternary select. So the same
source-level null check sits on different sides of the c1xx/c2 boundary depending
on whether the base is virtual, which is one more reason the family cannot be
lowered from the id. (It also means `l_upv` blocks the parser one token *before*
its selector, on the `DD *` operand type — bucket `expr-load-type-8643B2`.)

### 4.6 Two ids are close cousins that do **not** use `0x40`

Pointer-to-member formation (`(PMF)&M::vb`, `&M::d`) uses no `0x40` at all: it is a
`9b` / `27` / `5c` / `44` composition, and `&M::d` for a data member folds to a
bare `33 86 41 74 10` literal (`li r3,16`). Recorded here because
`docs/IL_CAST_CONVERT.md` §1.5 guessed pointer-to-member for ids 222/223 (§5.3).

---

## 5. What can be lowered — nothing yet, and three separate reasons

### 5.1 The emission depends on the literal argument values, not the id

§4.1 is the sharp case: same id, same descriptor, one literal byte apart, four
instructions and a branch apart. Accepting a `0x40` on the strength of its
selector would silently drop a null-guarded pointer adjustment on the largest
subfamily in the workload.

### 5.1.1 For `memcpy` / `memset` / block-assign, the dependence is now a RULE — measured at 624 of 624 cells (lane `w-memfit`, 2026-08-09)

§5.1's sentence stands and is **sharpened, not weakened**, for the one subfamily
where it has been gridded. The emission does depend on the literal argument
values, and for the block-move family the dependence is a two-line function:

```text
  align = the front end's ALIGNMENT HINT for the pointee type
  n     = size / align            integer division, TRUNCATING

  size is not a compile-time constant   ->  CALL
  size == 0                             ->  NOTHING EMITTED
  the destination is a non-escaping local never read afterwards
                                        ->  NOTHING EMITTED
  n <= T                                ->  INLINE (loads/stores)
  otherwise                             ->  CALL

  T = 5   at /O1 and /O2 /Os        (favor SIZE)
  T = 10  at /O2, /Ox and /O1 /Ot   (favor SPEED)
```

`memset` obeys it identically on every cell it was crossed with.

**Where each part is measured.** `align` is the byte at `.ex` offsets 2733 and
2742 of a one-`memcpy` TU (two hints, both written), and it equals
`alignof(pointee)` for all eight pointee types tested —
`char`/`int`/`double`/`struct{double;double;}` = 1/4/8/8 and
`void`/`long long`/`struct{int}`/`struct{double[4]}` = 1/8/4/8
(`work/w-memfit/hint.py`). Note the fourth: a 16-byte struct of two doubles
hints **8**, its *alignment*, not its size.

**The grading, on three independently frozen grids, 624 cells, no exceptions:**

| grid | cells | score | the best rival that grid had frozen |
|---|---:|---:|---|
| `w-memcpy` GRID-M | 232 | **232 / 232** | `M-THRESH-32` at 182; `M-ALWAYSCALL` at 114 |
| `w-memcpy` GRID-M2 | 176 | **176 / 176** | `F-48` at 114; `F-ALL` at 114 |
| `wb-memcpy` GRID-W | 216 | **216 / 216** | `W-LEVEL` at 144; `W-T5` at 126 |

Both constants are recoverable **from obj cells alone**, held out in both
directions: fitted on GRID-W's 72 `/O1` cells the rule scores 232/232 and
176/176 on grids it was never fitted to; fitted on GRID-M + GRID-M2's 408 it
scores 72/72 on GRID-W's `/O1` and refuses `/O2`, `/Ox` and `/O1 /Ot` at 18/36
each, which is the favor-speed split (`work/w-memfit/holdout.py`).

**What is NOT licensed by this, stated so absence does not read as coverage.**

* **The rule decides `call` vs `inline`; it does not emit either.** For the
  `call` arm the callee has **no `.gl` token at all** — re-derived black box: a
  TU whose only call is `memcpy` carries the names `?f@@…`, `.XBLD$W`,
  `__C1_11886` and the `/include:` directive in its `.gl`, and **no `memcpy`**,
  while the obj carries `[14] memcpy sc=EXTERNAL sec=0 type=0x0020`. So c2 mints
  the symbol and `bundle::resolve` can never produce it.
* **The `inline` arm's body layout is bracketed, not measured.** `wb-memcpy`
  §5.1b classifies 114 inline cells by `unit = min(align,8)` halved until it
  divides the size, `count = size/unit`, `count <= 4` straight line and
  `count >= 5` a counted loop — 114 of 114 — but its `n` axis produces only
  `count = 4` on the unrolled side, so the unrolled body's **length as a
  function of `count`** is unmeasured.
* **The confident core, for any port predicate**, is the intersection of the
  three grids' axes: a **compile-time-constant, non-zero** size, a **live**
  destination, an alignment hint in {1, 4, 8}, and a **known** favor-speed
  setting. On that core the rule is **348 of 348** over `w-memcpy`'s two grids
  and **204 of 204** over GRID-W. Everything outside it must refuse — the
  arms outside the core are each right on their own cells (8 non-constant, 8
  zero-size, 56 dead-destination) but each rests on a mechanism that is not the
  expansion, and two of them are *upstream* of it.

Lane rung: [`rungs/2026-08-09-w-memfit.md`](rungs/2026-08-09-w-memfit.md).
Provenance, and why the search space is disclosed:
[`whitebox/DISCLOSURE.md`](whitebox/DISCLOSURE.md) row **W-MEMCPY-1**.

### 5.2 The destination register is chosen by the consumer — **[CF] `il_intrinsic_fold.cpp`**

Even the one-instruction intrinsics are not context-free:

```
double f(double a)          { return fabs(a);      }  -> fabs f1,f1
double f(double a,double b) { return fabs(a) + b;  }  -> fabs f0,f1 ; fadd f1,f0,f2
double f(double a)          { return sqrt(fabs(a));}  -> fabs f0,f1 ; fsqrt f1,f0

int f(int a) { return abs(a);      }  -> srawi r11,r3,31 ; xor r10,r3,r11 ; subf >r3< ,r11,r10
int f(int a) { return abs(a) + 1;  }  -> srawi ; xor ; subf >r11<,r11,r10 ; addi r3,r11,1
int f(int a) { return abs(abs(a)); }  -> srawi ; xor ; subf >r9< ,r11,r10 ; srawi r8,r9,31 ; xor r7,r9,r8 ; subf r3,r8,r7
```

Same selector, same expansion, three different destinations. Lowering any of them
needs the W5 scratch ladder (`docs/CODEGEN_W5_SCRATCH.md`) extended to intrinsic
results, which no capture covers. This is the same class of hazard the generated
expression sweep found ~20 live instances of in the plain arithmetic class.

### 5.3 The folding rule differs between the integer and floating halves

**c1xx does not fold intrinsics** — `abs(-5)` reaches c2 intact as
`33 86 41 74 0f  40 86 41 74  33 86 41 74 fb  55 86 41 74  4C`. What c2 then does
splits by intrinsic, not by shape — **[CF] `il_intrinsic_fold.cpp`**:

| source | `.text` | folded? |
|---|---|:--:|
| `abs(-5)` | `38600005` `li r3,5` | **yes** |
| `_rotl(1u, 4)` | `38600010` `li r3,16` | **yes** |
| `fabs(-1.5)` | `lis r11 ; lfd f0,0(r11) ; fabs f1,f0`, pooling `__real@bff8000000000000` | **no** |
| `sqrt(4.0)` | `lis ; lfd ; fsqrt f1,f0`, pooling `__real@4010000000000000` | **no** |

c2 emits the *unfolded* floating constant and applies the operation at run time.
So "an intrinsic over literals is a constant" is right for the integer half and
wrong for the floating half — the same per-(operator, value) structure
`fixtures/cpp/w13b_ffold.cpp` records for plain FP arithmetic, and only a fixture
holding both halves separates it.

### 5.4 What the fail-closed rule therefore is

Reject every `0x40`. The decode reports the selector and returns `Err`
(`intrinsic_call_decode_does_not_accept`). The narrowest thing that could ever be
admitted is id 2114 with offset literal `00`, which is provably nothing — and even
that requires modelling the `66 <n>` descriptor, the whole argument region, and the
pointer TYPEs, for a census gain that the histogram does not show as material.

---

## 6. Selector ids 222 / 223 — the remaining UNKNOWN

`0xDE` and `0xDF`, 1758 sites each **[DC3]**, 5th/6th most common. They always
occur as a nested pair, 222 inside 223's argument region, which is why only
`0xDF` ever surfaces as a blocking bucket (2491 functions).

```
33 86 41 74 80 df 00 00 00 | 40 86 46 80 20         223 -> a class type (kind 0x46)
  33 86 41 74 >04<  55 86 41 74                       sizeof(class)
  33 86 41 74 80 de 00 00 00 | 40 86 46 80 20       222 -> the same type
    9b <TYPE ptr> <tok>  55 <TYPE ptr>                a slot reference
    33 86 41 74 >04<  55 86 41 74                     sizeof(class) again
  4C
  55 86 46 80 20                                      222's result
  26 <copy-ctor> … BD … 4C                            the copy construction
  33 86 41 74 01  55 86 41 74
4C
```

Two facts pinned **[CF] `il_intrinsic_byval.cpp`**:

1. **The literal is `sizeof(class)`.** A 4-byte class emits `04` in both slots; a
   12-byte one emits `0c`, with nothing else about the two bodies differing.
2. **The trigger is a non-trivial COPY CONSTRUCTOR** — not the destructor, and not
   `/EHsc`. `CtorOnly` (copy ctor, no dtor) produces the pair; `DtorOnly` (dtor, no
   copy ctor) and a POD of the same size do not. Captured under this repo's default
   fixture flags, which do not include `/EHsc`.

**[DC3]** agrees: every 222/223 site in `Dir.cpp` wraps a 4-byte class (`Symbol`)
handed to a by-value parameter — e.g.
`?SystemConfig@@YAPAVDataArray@@VSymbol@@00@Z`, i.e.
`DataArray *SystemConfig(Symbol, Symbol, Symbol)` — with `??0Symbol@@QAA@PBD@Z`
called inside 223's argument region.

**UNKNOWN: which of the two does what.** Both return the class type; 222 takes
(slot, size) and 223 takes (size, 222's result, the copy construction, 1).
"222 addresses the caller-owned slot and 223 commits the copy into it" fits every
byte, and so does the reverse; the emission is a whole calling sequence, not two
separable pieces. **The fixture that would separate them** is one where the two
must nest asymmetrically — a by-value argument forwarded twice in one call,
`void f(C4 a, C4 b); void g(C4 x) { f(x, x); }`, where a slot-addressing operation
must appear twice and a commit operation once, or vice versa.

`docs/IL_CAST_CONVERT.md` §1.5 read 222/223 as "plausibly pointer-to-member
formation/adjustment". That is refuted: pointer-to-member formation uses no `0x40`
(§4.6).

Two further ids appear in the workload below the noise floor and are UNKNOWN:
**221 / `0xDD`** (6 functions) and **2120 / `0x848`** (1 function). Both stay hex.

---

## 7. Measured census effect

`c2rs gap --list work/dc3-workload/files.txt --flags-file work/dc3-workload/flags.txt
--cwd ../dc3-decomp --jobs 12`, 878 TUs, before and after, both at commit
`2b6cbe5` with only this change applied:

```
before   match 6 / mismatch 0 / vocab-gap 865 / capture-fail 7
         FUNCTION CENSUS (P2b): 87423/2462571 functions in class (3.55%)
after    match 6 / mismatch 0 / vocab-gap 865 / capture-fail 7
         FUNCTION CENSUS (P2b): 87423/2462571 functions in class (3.55%)
```

Both runs used a checkout of `2b6cbe5` with *only* this change applied, because
concurrent sessions were moving `crates/c2-il` at the time; a scan of the shared
tree at `6b57b0c` reports 110,277/2,462,571 (4.48 %), and all of that +22,854 is
the concurrent indirect-load work, none of it this change. The bucket split below
is identical in both.

**The in-class count moves by zero — to the function.** That is by construction and
not a disappointment: the decode replaces one `Err` with a differently-labelled
`Err`, so the gate is byte-for-byte identical and the census cannot drift from it.
Nothing here was lowered, for the reasons in §5.

What moved is the resolution of the histogram. The two buckets the production was
hiding in:

```
expr-intrinsic-call   213411  ->  0 residue      (100.0% decoded)
call-token-0x33       176123  ->  8046 residue   ( 95.4% decoded)
                      ------                     ---------------
                      389534      381488 intrinsic-call functions = 16.1% of blocked
```

so the production's real footprint is **16.1 % of blocked functions**, against the
9 % the `expr-intrinsic-call` bucket showed and the "on the order of 12 %"
`docs/IL_CAST_CONVERT.md` §0 estimated. Split:

| sub-family | functions | share of blocked |
|---|---:|---:|
| **class layout (2113–2119)** | **329,205** | **13.9 %** |
|  · 2117 `base-member-addr` | 148,915 | 6.3 % |
|  · 2113 `this-adjust` | 137,511 | 5.8 % |
|  · 2115 `base-downcast` | 19,978 | 0.8 % |
|  · 2114 `base-upcast` | 19,440 | 0.8 % |
|  · 2119 `dynamic-cast` | 1,651 | 0.1 % |
|  · 2116 `vbase-upcast` | 1,360 | 0.1 % |
|  · 2118 `vbase-member-addr` | 350 | — |
| CRT string / memory (164–173) | 39,311 | 1.7 % |
|  · `memset` | 30,901 | 1.3 % |
|  · `memcpy` | 3,366 | 0.1 % |
|  · `strlen`, `strcmp`, `memcmp`, `strcpy` | 5,044 | 0.2 % |
| floating point (`fabs`, `sqrt`, `__frsqrte`) | 8,806 | 0.4 % |
| by-value class copy (223, §6) | 2,491 | 0.1 % |
| integer (`abs`, `_abs64`) | 1,662 | 0.1 % |
| `__mftb`, `0xDD`, `0x848` | 13 | — |

(Sums to 381,488 exactly. The `expr-` and `call-` halves of each selector are
folded together here; run `c2rs gap … --jsonl <path>` and aggregate
`fn_blockers` to reproduce the unfolded per-bucket counts.)

Top of the widening order after the split:

```
   427706 ( 18.0%)  call-token-0xB9                 (member / indirect calls)
   170401 (  7.2%)  body-0x53                       (leading control flow)
   164544 (  6.9%)  expr-call-in-expr
   137496 (  5.8%)  call-intrinsic-this-adjust      <- 2113
   135754 (  5.7%)  expr-intrinsic-base-member-addr <- 2117
   125226 (  5.3%)  call-token-0x26
    85939 (  3.6%)  expr-load-type-864540           (float operand)
    79542 (  3.3%)  expr-load-type-888541           (double operand)
    49189 (  2.1%)  expr-load-type-864383           (void* operand)
    30901 (  1.3%)  expr-intrinsic-memset
```

The practical reading: **the class-layout family is the single largest semantic gap
in the workload after member calls**, and it is not one gap — 2117 (a folded `lwz`
displacement) and 2113 (an unguarded `addi`) are each individually larger than
every remaining operand-type bucket, and both are *small* lowerings blocked only by
the operand-stack type tracking that `docs/IL_CAST_CONVERT.md` §5 already
identifies as the prerequisite for `0x2C`, floats and pointers. That is a change to
the widening order: the same three prerequisites unlock ~14 % of blocked functions
through this family, not the ~9 % the old bucket suggested.

---

## 8. Open unknowns, restated

* 222/223's individual semantics (§6) — separating fixture given.
* Ids 221 / `0xDD` and 2120 / `0x848` (§6).
* Whether the id space contains anything outside the 20 selectors this workload
  uses. It cannot be enumerated from the IL; only more corpus breadth answers it.
* The type `kind` nibble `6`: `86 46 <id>` (a 4-byte class) and `86 06 20 ec 20`
  (a 32-byte one) both occur, and the second shows `read_type`'s
  `<tag> <kind> <LEB>` rule is **wrong for aggregates** — that triple would read as
  3 bytes and leave `ec 20` stranded, while the surrounding `41` result marker
  pins it at 5. Aggregates appear to carry an extra LEB (`20` = 32 = the size),
  and the tag is not size-derived either (a 4-byte class is `86`, a 12-byte one
  `c6`). Out of scope here, but it bounds any widening that reads a class type:
  `il_intrinsic_byval.cpp` is the fixture that already exhibits it.
* Why c1xx emits only 2 `4C 4F 11` body markers for the 5 functions of
  `il_intrinsic_byval.cpp` (against 5 function tails), so `split_function_bodies`
  under-counts that shape. Immaterial on the real workload — `Dir.cpp` has 5239 LO
  against 5243 tails, 0.08 % — but it means that fixture's census line reports 2
  functions, not 5.
