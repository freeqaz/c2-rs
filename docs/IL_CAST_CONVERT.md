# `.ex` casts and conversions — `0x2C`, and what `0x40` actually is

**Status: characterization (P2d). No Rust source was changed by this document.**
Every claim below cites bytes from a live `16.00.11886.00` capture made this
session. Claims resting on a *controlled fixture* (one construct varied,
everything else held fixed) are marked **[CF]**; claims resting on real dc3
translation units are marked **[DC3]**. Unknowns are written as UNKNOWN — the
port must fail closed on them, never guess.

Tracked fixtures added by this work:

* `fixtures/cpp/il_convert_scalar.cpp` — 19 scalar conversions, one per function
* `fixtures/cpp/il_intrinsic_call.cpp` — 12 `0x40` sites (9 CRT intrinsics +
  3 class-layout adjustments)

Reproduce:

```
./target/release/c2rs census   fixtures/cpp/il_convert_scalar.cpp --keep-il work/cast/il_conv
./target/release/c2rs compile  fixtures/cpp/il_convert_scalar.cpp --keep-obj work/cast/conv.obj
# real workload
./target/release/c2rs census src/system/world/Dir.cpp \
    --flags-file work/dc3-workload/flags.txt --cwd ../dc3-decomp --keep-il work/cast/il_dir
```

---

## 0. Headline: the `expr-cast` census bucket is misnamed

`c2_il::func::Block::feature` maps operand-stream byte `0x40` to the name
`cast` (→ bucket `expr-cast`, 6.8 % of blocked functions, 167,205 functions in
the P2 scan), on the conjecture that it is `40 <target-type>`, a cast applied to
the preceding value.

**That conjecture is refuted.** `0x40` is not a cast at all. It is a second
**CALL token** — the *intrinsic* call — and it occupies exactly the syntactic
slot that `BD` occupies in an ordinary call. The construct that actually
performs a cast/conversion is `0x2C`, whose census bucket is `expr-convert`.

Two independent lines of evidence:

1. **[CF]** Every scalar cast that can be written in C++ — int↔unsigned,
   int→char/short, char/short→int, int↔long long, int↔float, int↔double,
   float↔double, pointer↔pointer, pointer↔integer, bool→int — emits `0x2C` and
   never `0x40` (§2). `fixtures/cpp/il_convert_scalar.cpp` contains 19 such
   conversions and contains **zero** `0x40` bytes in its operand stream.
2. **[CF]** Writing a call to a compiler intrinsic produces `0x40` immediately
   (§1). `?t_strlen@@YAIPBD@Z` in `fixtures/cpp/il_intrinsic_call.cpp` is

   ```
   4c 4f 11 53
     33 86 41 74 80 a7 00 00 00     LITERAL int 167          <- the intrinsic id
     40 86 42 75                    CALL-INTRINSIC -> unsigned
     b9 0d 0a 86 43 83 20           LOAD s (const char *)
     55 86 43 83 20                 arg push (const char *)
     4c                             apply
   41 86 42 75  3a 0f 0a  54 02 29 0f 0a  4f 12 47 54 01 54 00
   ```

   which is `strlen(s)` — and c2 expands it to an inline byte loop, not a call.

A third bucket is the same production: `call-token-0x33` (5.9 %, 144,276
functions) is an intrinsic call whose *result is assigned*. **[DC3] Dir.cpp**,
359 functions:

```
… 46 | 4c 4f 11 53 | 26 9e 15 | >33< 86 41 74 80 41 08 00 00 | 40 86 43 a5 21 | 66 02 b1 21 8a 21 55 86 41 74 …
                     ^dest sym    ^intrinsic id 2113            ^CALL-INTRINSIC
```

The parser consumed `26 <dest>` expecting `BD` and found `33`. So the true
footprint of the `0x40` production is `expr-cast` **plus** a large share of
`call-token-0x33` — on the order of 12 % of blocked functions, not 6.8 %.

**Recommended (diagnostic-only) census corrections, not applied here:**
`0x40` → `intrinsic-call`, `0x2C` → `convert` (already correct), and the tables
in `docs/GAPS.md` and `docs/ROADMAP.md` that gloss `expr-cast` as
"`40 <target-type>`" should be reworded.

---

## 1. The `0x40` production — INTRINSIC CALL

### 1.1 Grammar

```
INTRINSIC-CALL := 33 <int-TYPE> <literal id>     the intrinsic selector
                  40 <TYPE result>                the call token
                  ( <expr> 55 <TYPE> )*           arguments, reverse source order
                  4C                              apply
```

Compare the ordinary call (`docs/IL_CALL_GRAMMAR.md` §2):

```
CALL := <callee-expr:26 tok> BD <TYPE ret> <flags:1> <varint fn-type-id> <args> 4C
```

The `0x40` token is **strictly shorter**: `40 <TYPE>` and nothing else — no
calling-convention flags byte, no function-type id. The callee identity lives
entirely in the *preceding* int literal.

### 1.2 The selector literal is always `int`-typed — measured

Instrumenting the whole-body validation parser (`work/exp/tools/gram2.py`, the
one that lands 52 % of Dir.cpp exactly on the 7-byte function tail) to record
the token immediately preceding every `0x40` it reaches:

| TU | bodies | `0x40` sites reached | preceded by `33 86 41 74 <lit>` |
|---|---:|---:|---:|
| `src/system/world/Dir.cpp` | 5239 | 1463 | **1463 (100 %)** |
| `src/App.cpp` | 9033 | 2584 | **2584 (100 %)** |
| `src/lazer/game/Game.cpp` | 9639 | 2791 | 2790 (99.96 %) |

The single Game.cpp exception is a parse-misalignment artifact, not a second
form: at that site the parser had just consumed `9b a2 11 9e 30 7e` as
`9b <TYPE> <varint>` and landed mid-token; the byte after the putative `40` is
`2c`, whose bit 7 is clear, so `read_type` rejects it and the body fails
immediately. No site anywhere shows `0x40` applied to a LOAD, to an arithmetic
result, or to anything other than an `int` literal.

**This alone refutes "cast applied to the preceding value":** a cast opcode
would overwhelmingly follow LOADs and sub-expressions. `0x40` follows a bare
constant, 100 % of the time.

### 1.3 Intrinsic ids pinned by controlled fixture

All from `fixtures/cpp/il_intrinsic_call.cpp` except `strcpy` (164), `_rotr`
(160) and `throw` (337), which are from the untracked probes `work/cast/k4.cpp`
and `work/cast/k5.cpp`. The id is the literal's value; the emitted code is read
off the reference obj.

| id (dec / hex) | intrinsic | IL selector bytes | c2 emits |
|---:|---|---|---|
| 15 / `0x0f` | `abs`, `labs` | `33 86 41 74 0f` | `srawi r11,r3,31 ; xor r10,r3,r11 ; subf r3,r11,r10` |
| 17 / `0x11` | `fabs` | `33 86 41 74 11` | `fabs f1,f1` |
| 159 / `0x9f` | `_rotl` | `33 86 41 74 80 9f 00 00 00` | `rlwnm r3,r3,r4,0,31` |
| 160 / `0xa0` | `_rotr` | `33 86 41 74 80 a0 00 00 00` | `subfic r11,r4,32 ; rlwnm r3,r3,r11,0,31` |
| 164 / `0xa4` | `strcpy` | `33 86 41 74 80 a4 00 00 00` | inline byte loop (8 instrs) |
| 165 / `0xa5` | `strcmp` | `33 86 41 74 80 a5 00 00 00` | inline loop (11 instrs) |
| 167 / `0xa7` | `strlen` | `33 86 41 74 80 a7 00 00 00` | inline loop (9 instrs) |
| 170 / `0xaa` | `memcmp` | `33 86 41 74 80 aa 00 00 00` | inline loop (15 instrs) |
| 172 / `0xac` | `memcpy` | `33 86 41 74 80 ac 00 00 00` | `b <memcpy>` (REL24 tail call) |
| 173 / `0xad` | `memset` | `33 86 41 74 80 ad 00 00 00` | `b <memset>` (REL24 tail call) |
| 1973 / `0x7b5` | `sqrt` | `33 86 41 74 80 b5 07 00 00` | `fsqrt f1,f1` |
| 337 / `0x151` | C++ `throw` — **implied, not proven** | `33 86 41 74 80 51 01 00 00` | see note |

The `337` attribution is weaker than the rest and is flagged as such. In the
`work/cast/k5.cpp` probe (which contains `int thr() { throw 3; }`) exactly one
body is

```
4c 4f 11 53 | 9b 86 41 74 49 0a | 33 86 41 74 03 | 32 86 41 74      store 3 into a temp
  33 86 41 74 80 51 01 00 00 | 40 82 07 03                          CALL-INTRINSIC 337 -> void
  26 44 0a  55 86 43 a9 20                                          a symbol, pushed as a pointer
  9b 86 41 74 49 0a  2c 86 43 83 08 00  55 86 43 83 08              &temp, decayed to void*
4c | 44 4b | 54 02 29 43 0a | 4f 12 47 54 01 54 00
```

i.e. store the literal 3, then call a void helper with (a symbol, the address of
that temp) — the shape of `_CxxThrowException(&tmp, &throwinfo)`. The `.gl`
name→segment pairing for that TU is unreliable (the harness reports every
function as `(unnamed)`), so this is inference from the body shape, not a
name-matched observation. Treat as UNKNOWN for gating purposes.

`memcpy`/`memset`/`memcmp` carry **extra trailing arguments the source does not
have**: literal `int` alignment hints. `?t_memcpy` pushes `33 86 41 74 01
55 86 41 74` twice (dest and source alignment = 1) before the three real
arguments; `?t_memset` pushes it once; **[DC3]** `Dir.cpp` fn931 pushes `04`
instead of `01` when the operands are 4-byte aligned. The hint changes the
expansion, so it cannot be ignored.

### 1.4 The class-layout family — ids 2113…2119 (`0x841`…`0x847`)

This family is the bulk of the bucket. Aligned-parse counts:

| id | Dir.cpp | App.cpp | Game.cpp |
|---:|---:|---:|---:|
| 2113 `0x841` | 529 | 869 | 939 |
| 2117 `0x845` | 421 | 835 | 870 |
| 2114 `0x842` | 67 | 106 | 121 |
| 2115 `0x843` | 67 | 98 | 112 |
| 2116 `0x844` | 59 | 100 | 120 |
| 2119 `0x847` | 59 | 78 | 100 |
| 2118 `0x846` | 5 | (few) | 7 |

Their argument shape is fixed:

```
33 86 41 74 <id> 40 <TYPE result>
  66 02 <tok classA> <tok classB>   55 86 41 74      class-pair descriptor
  ( 33 86 41 74 <offset>            55 86 41 74 )*   k byte offsets
  <object expr>                     55 <TYPE ptr>    the pointer being adjusted
4C
```

(`0x66` is the opcode `docs/IL_CALL_GRAMMAR.md` §7 ranks as the #1 unidentified
blocker at 1148 Dir.cpp bodies. It is not a call — it is this descriptor.)

Pinned semantics **[CF]** (`fixtures/cpp/il_intrinsic_call.cpp`, plus
`work/cast/k6.cpp` and `work/cast/k7.cpp` for multiple/virtual inheritance):

| id | construct | k offsets | c2 emits |
|---:|---|---|---|
| 2114 | derived → base pointer upcast | 1 | see below |
| 2115 | base → derived downcast | 1 | mirror of 2114 with a negated offset |
| 2113 | base adjustment feeding a member call's `this` | 1 | as 2114 |
| 2117 | address of a member inherited from a non-virtual base | 2 (base off, member off) | folds into a `lwz` displacement |
| 2116 | virtual-base pointer upcast | 4 | vbtable indirection + branch |
| 2118 | address of a member of a **virtual** base | 5 | `lwz r11,0(r3) ; lwz r10,4(r11) ; lwzx r3,r10,r3` |
| 2119 | UNKNOWN — its arguments are `26 <sym>` pushes, not offset literals | — | UNKNOWN |

**The codegen depends on the literal argument value, not just on the id.**
`fixtures/cpp/il_intrinsic_call.cpp`, `struct M : A1, A2`:

```
?up_zero    33 86 41 74 80 42 08 00 00  40 86 43 85 20  66 02 88 20 84 20 55 86 41 74
            33 86 41 74 >00< 55 86 41 74  b9 39 0a 86 43 8e 20 55 86 43 8e 20  4c
  -> 4e800020                                     blr                        (NOTHING)

?up_nonzero 33 86 41 74 80 42 08 00 00  40 86 43 8f 20  66 02 88 20 89 20 55 86 41 74
            33 86 41 74 >04< 55 86 41 74  b9 3c 0a 86 43 8e 20 55 86 43 8e 20  4c
  -> 2b030000  cmplwi r3,0
     38630004  addi   r3,r3,4
     4c9a0020  bclr   4,26          (return if r3 was non-null)
     38600000  li     r3,0
     4e800020  blr
```

Same opcode, same intrinsic id, same argument count; **one literal byte apart**,
and the difference is four instructions plus a control-flow split. Accepting the
`0x40` family on the strength of the id would silently drop a null-guarded
pointer adjustment.

`?fld_base` (id 2117, offsets `00`,`04`) → `lwz r3,4(r3)`; the offsets are folded
into the load displacement (`work/cast/k7.cpp`: `(0,0)`→`lwz r3,0(r3)`,
`(0,4)`→`lwz r3,4(r3)`, `(4,4)`→`lwz r3,8(r3)`).

### 1.5 Other observed ids — UNKNOWN

| id | shape | note |
|---:|---|---|
| 222 / `0xde`, 223 / `0xdf` | result type kind `0x46`; operands include `9b <TYPE> <tok>` member binds; 223's first argument is a 222 call | Always occur as a pair, 46/81/82 times per TU. Plausibly pointer-to-member formation/adjustment. **UNKNOWN** |
| 815 / `0x32f` | one `long long` argument, `long long` result (`33 86 41 74 80 2f 03 00 00 40 88 81 13 b9 a7 0c 88 81 13 55 88 81 13 4c`, **[DC3]** Dir fn11) | plausibly `_abs64`. **UNKNOWN** |
| 1948 / `0x79c` | **zero arguments**: `33 86 41 74 80 9c 07 00 00 40 88 82 23 4c` (**[DC3]** Dir fn683), `unsigned long long` result | a nullary clock/counter read. **UNKNOWN**. Establishes that the argument list may be empty. |

The id space is a c1xx-internal table. **It is not enumerable from the IL**, so
the only sound policy is an allow-list of ids pinned by controlled fixture, and
even then only for ids whose argument values are also constrained (§1.4).

### 1.6 How to tell `0x40` from `0x2C` — the decision rule

They are never ambiguous, because they occupy different slots:

| | `0x2C` (convert) | `0x40` (intrinsic call) |
|---|---|---|
| token | `2C <TYPE> <varint>` | `40 <TYPE>` |
| trailing field | yes — a varint (§2.1) | **none** |
| preceding stack token | the value being converted (LOAD, literal, sub-expression, `26` symbol push, comparison result) | always `33 <int-TYPE> <id>` |
| following tokens | whatever the enclosing expression needs | an argument region terminated by `4C` |
| c1xx emits it for | a C++ conversion, a decay, an lvalue→rvalue type change | an intrinsic / compiler-helper call |

A decoder that reads `2C` as `<opcode> <TYPE> <varint>` and `40` as
`<opcode> <TYPE>` stays aligned on both; swapping them desynchronises within
one token.

---

## 2. `0x2C` — the actual conversion opcode

### 2.1 Grammar

```
CONVERT := 2C <TYPE target> <varint>
```

The trailing varint is `0` at **every** aligned site measured:

| TU | aligned `0x2C` sites | trailing varint |
|---|---:|---|
| Dir.cpp | 3313 | `0` × 3313 |
| App.cpp | 5244 | `0` × 5244 |
| Game.cpp | 5541 | `0` × 5541 |

**UNKNOWN:** what a non-zero value would mean. `docs/IL_CALL_GRAMMAR.md` §7
lists the field as a varint on the strength of `0x99`/`0x9B` sharing the shape;
14,098 observations say only that it is always zero here. A decoder should
accept `2C <TYPE> 00` and fail closed on anything else.

`0x2C` is *purely* a type annotation: it names the **target** type and says
nothing about the source. The source type lives on the operand stack, which is
exactly the information `parse_expr` does not have (§5).

### 2.2 The conversion table

Source is always a formal (nothing folds); the value is returned, so the
conversion is the whole function body and the emitted instructions are the whole
`.text` minus the `blr`. `[CF] fixtures/cpp/il_convert_scalar.cpp` unless noted;
the remainder are from `work/cast/k1.cpp`, `k2.cpp`, `k3.cpp` (same session,
same flags).

**The no-op column is the load-bearing one.**

| source | target | IL convert token | c2 `.text` | no-op? |
|---|---|---|---|:--:|
| `int` | `int` | *(none emitted)* | — | n/a |
| `int` | `unsigned` | `2c 86 42 75 00` | — | **yes** |
| `unsigned` | `int` | `2c 86 41 74 00` | — | **yes** |
| `int` | `char` | `2c 82 11 70 00` | `7c630774 extsb r3,r3` | no |
| `int` | `short` | `2c 84 21 11 00` | `7c630734 extsh r3,r3` | no |
| `int` | `unsigned char` | `2c 82 12 20 00` | `5463063e rlwinm r3,r3,0,24,31` | no |
| `int` | `unsigned short` | `2c 84 22 21 00` | `5463043e rlwinm r3,r3,0,16,31` | no |
| `char` | `int` | `2c 86 41 74 00` | `7c630774 extsb r3,r3` | no |
| `short` | `int` | `2c 86 41 74 00` | `7c630734 extsh r3,r3` | no |
| `unsigned char` | `int` | `2c 86 41 74 00` | `5463063e rlwinm r3,r3,0,24,31` | no |
| `unsigned short` | `int` | `2c 86 41 74 00` | `5463043e rlwinm r3,r3,0,16,31` | no |
| `char` | `unsigned` | `2c 86 42 75 00` | `7c630774 extsb r3,r3` | no |
| `bool` | `int` | `2c 86 41 74 00` | `5463063e rlwinm r3,r3,0,24,31` | no |
| `bool` | `unsigned` | `2c 86 42 75 00` | `5463063e rlwinm r3,r3,0,24,31` | no |
| `bool` | `char` | `2c 82 11 70 00` | — | **yes** |
| `int` | `long long` | `2c 88 81 13 00` | `7c6307b4 extsw r3,r3` | no |
| `unsigned` | `long long` | `2c 88 81 13 00` | `78630020 rldicl r3,r3,0,32` | no |
| `unsigned` | `unsigned long long` | `2c 88 82 23 00` | `78630020 rldicl r3,r3,0,32` | no |
| `long long` | `int` | `2c 86 41 74 00` | `7c6307b4 extsw r3,r3` | no |
| `unsigned long long` | `unsigned` | `2c 86 42 75 00` | `5463003e rlwinm r3,r3,0,0,31` | no |
| `int` | `float` | `2c 86 45 40 00` | `extsw r11,r3 ; std r11,-16(r1) ; lfd f0,-16(r1) ; fcfid f13,f0 ; frsp f1,f13` | no |
| `int` | `double` | `2c 88 85 41 00` | `extsw r11,r3 ; std ; lfd ; fcfid f1,f0` | no |
| `unsigned` | `float` | `2c 86 45 40 00` | `rldicl r11,r3,0,32 ; std ; lfd ; fcfid ; frsp` | no |
| `unsigned` | `double` | `2c 88 85 41 00` | `rldicl r11,r3,0,32 ; std ; lfd ; fcfid` | no |
| `char` | `float` | `2c 86 45 40 00` | `extsb r10,r3 ; std ; lfd ; fcfid ; frsp` | no |
| `long long` | `double` | `2c 88 85 41 00` | `std r3,16(r1) ; lfd f0 ; fcfid f1,f0` | no |
| `float` | `int` | `2c 86 41 74 00` | `fctiwz f0,f1 ; stfd f0,-16(r1) ; lwz r3,-12(r1)` | no |
| `double` | `int` | `2c 86 41 74 00` | `fctiwz f0,f1 ; stfd ; lwz r3,-12(r1)` | no |
| `float` | `unsigned` | `2c 86 42 75 00` | `fctidz f0,f1 ; stfd ; lwz r3,-12(r1)` | no |
| `float` | `short` | `2c 84 21 11 00` | `fctiwz ; stfd ; lhz r3,-10(r1)` | no |
| `double` | `long long` | `2c 88 81 13 00` | `fctidz f0,f1 ; stfd ; ld r3,-16(r1)` | no |
| `float` | `double` | `2c 88 85 41 00` | — | **yes** |
| `double` | `float` | `2c 86 45 40 00` | `fc200818 frsp f1,f1` | no |
| `int *` | `void *` | `2c 86 43 83 08 00` | — | **yes** |
| `void *` | `int *` | `2c 86 43 f4 08 00` | — | **yes** |
| `int *` | `char *` | `2c 86 43 f0 08 00` | — | **yes** |
| `void *` | `S *` (unrelated) | `2c 86 43 81 20 00` | — | **yes** |
| `void *` | `unsigned` | `2c 86 42 75 00` | — | **yes** |
| `unsigned` | `void *` | `2c 86 43 83 08 00` | — | **yes** |
| `long long` | `unsigned long long` | `2c 88 82 23 00` | — | **yes** |
| `unsigned long long` | `long long` | `2c 88 81 13 00` | — | **yes** |
| `const int` (`a6 41 80 20`) | `int` | `2c 86 41 74 00` | — | **yes** |
| array `26 <sym>` → pointer (decay) | | `2c 86 43 f4 08 00` | — | **yes** (the `lis/addi` comes from the `26`) |

Three consequences the port cannot design around:

**(a) The IL bytes do not determine the emission.** The single token
`2c 86 41 74 00` — "convert to `int`" — is *simultaneously* a no-op (source
`unsigned`, source `const int`), an `extsb` (source `char`), an `extsh` (source
`short`), a `clrlwi 24` (source `unsigned char` or `bool`), a `clrlwi 16`
(source `unsigned short`), an `extsw` (source `long long`), and a
three-instruction `fctiwz ; stfd ; lwz` sequence (source `float` or `double`).
The discriminator is the **source operand's type**, which is carried on the
operand stack and is nowhere in the `2C` token.

**(b) Conversions do not compose token-by-token.**
`work/cast/k10.cpp` `?dblcast@@YAHH@Z`, `(int)(unsigned char)(short)a`:

```
b9 0c 0a 86 41 74 | 2c 84 21 11 00 | 2c 82 12 20 00 | 2c 86 41 74 00 | 41 86 41 74
->  5463063e  rlwinm r3,r3,0,24,31
    4e800020  blr
```

Three `2C` tokens, one instruction. Lowering each convert independently
(`extsh ; clrlwi 24 ; <nothing>`) would emit two extra instructions and fail the
byte compare.

**(c) A cast *to* `bool` is not a `2C` at all.** c1xx synthesizes a `!= 0`
comparison first. `work/cast/k2.cpp` `?i2b@@YA_NH@Z`, `(bool)a`:

```
b9 0e 0a 86 41 74 | 33 86 41 74 00 | 20 | 2c 82 12 30 00 | 41 82 12 30
->  3163ffff  addic r11,r3,-1
    7c6b1910  subfe r3,r11,r3
```

i.e. the W6 branchless materialization spine. `(bool)p` for a pointer is
byte-identical.

### 2.3 Constant casts are folded by c1xx — no `2C` survives

**[CF] `work/cast/k10.cpp`**

```
int fold_c() { return (int)(char)300; }   ->  33 86 41 74 2c  41 86 41 74     (44)
int fold_f() { return (int)1.5;       }   ->  33 86 41 74 01  41 86 41 74     (1)
double fold_i() { return (double)3;   }   ->  33 88 8a 41 <8 IEEE bytes> 08 00
```

`li r3,44`, `li r3,1`, an `.rdata` double load. So the port never has to lower a
cast of a literal: if a `2C` is present, at least one operand is a runtime value.

### 2.4 `0x59` — an unmodeled marker between an fp op and a convert

**[CF] `work/cast/k11.cpp` / `k12.cpp`.** A floating-point arithmetic op followed
immediately by a `2C` has a `59` byte wedged between them; the same op with no
following convert does not:

```
double dsub  (double a,double b) { return a-b;          }  -> 03      41 88 85 41
float  fsub2 (float  a,float  b) { return a-b;          }  -> 03      41 86 45 40
float  dsub_f(double a,double b) { return (float)(a-b); }  -> 03 >59< 2c 86 45 40 00
double dsub_d(float  a,float  b) { return (double)(a-b);}  -> 03 >59< 2c 88 85 41 00
double dd_id (double a,double b) { return (double)(a-b);}  -> 03 >59< 2c 88 85 41 00
float  fmul_f(double a,double b) { return (float)(a*b); }  -> 04 >59< 2c 86 45 40 00
float  fneg_f(double a)          { return (float)(-a);  }  -> 08 >59< 2c 86 45 40 00
```

Integer ops do not do this (`(char)(a+b)` is `02 2c 82 11 70 00`), and a convert
over a bare LOAD does not either (`(float)a` is `b9 … 88 85 41 2c 86 45 40 00`).
Note `dd_id`: c1xx emits an *identity* `double`→`double` convert here, which it
never does for `int`→`int`.

**UNKNOWN:** the meaning of `0x59`. Plausibly "the pending fp result is
unrounded; the following convert is the rounding step" — but that is a guess and
nothing here tests it. The corresponding emissions are
`dsub_f` → `fsub f0,f1,f2 ; frsp f1,f0` and `dsub_d` → `fsubs f1,f1,f2` (single
instruction; the widening convert is free), so the marker is *not* itself an
instruction. It must be rejected.

---

## 3. Literal payload encodings — confirmed and corrected

The premise in the task ("the `double` literal payload is 8 raw IEEE bytes
rather than a varint") is **CONFIRMED**, and there are two further corrections
to the model in `crates/c2-il/src/func.rs::read_varint`.

### 3.1 Floating literals — 8 raw IEEE-754 bytes + a 2-byte size field

```
FLOAT-LITERAL := 33 <literal-TYPE> <8 bytes: IEEE-754 binary64, LITTLE-endian> <u16 LE size>
```

**[CF] `work/cast/k9.cpp`:**

| source | bytes |
|---|---|
| `return 1.0f;` | `33 86 4a 40  00 00 00 00 00 00 f0 3f  04 00` |
| `return 0.5f;` | `33 86 4a 40  00 00 00 00 00 00 e0 3f  04 00` |
| `return -2.5f;` | `33 86 4a 40  00 00 00 00 00 00 04 c0  04 00` |
| `return 1.0;` | `33 88 8a 41  00 00 00 00 00 00 f0 3f  08 00` |
| `return 0.0;` | `33 88 8a 41  00 00 00 00 00 00 00 00  08 00` |
| `return 3.14159265358979;` | `33 88 8a 41  11 2d 44 54 fb 21 09 40  08 00` |

`11 2d 44 54 fb 21 09 40` read little-endian is `0x400921FB54442D11` = the
binary64 nearest π. **A `float` literal is stored as a `double` too** — `1.0f`
and `1.0` have identical 8-byte payloads and differ only in the type triple and
the trailing size.

The 2-byte trailer is the **target size in bytes**, u16 LE: `04 00` for `float`,
`08 00` for `double`. It is not part of the value.

The literal's own type triple is **not** the value type triple:

| type | value triple | *literal* triple |
|---|---|---|
| `float` | `86 45 40` (kind `0x45`) | `86 4a 40` (kind `0x4a`) |
| `double` | `88 85 41` (kind `0x85`) | `88 8a 41` (kind `0x8a`) |

Same tag, same LEB id, kind + 5. `?d_addf@@YANM@Z` (`(double)(a + 1.5f)`) shows
both in one body: `33 86 4a 40 … 04 00` for the literal, `2c 88 85 41 00` for the
convert, `41 88 85 41` for the result type.

**[DC3]** consistent with `docs/IL_CALL_GRAMMAR.md` §5 (`e9.cpp`,
`p3('x', 2.0f, a)`), which found the same shape independently.

### 3.2 Integer literals — the payload is a *signed* byte, not a varint

`read_varint` currently models `b < 0x80 → value = b` (unsigned) and
`b == 0x80 → 4-byte LE i32`. Measured behaviour **[CF] `work/cast/k11.cpp`**:

```
LIT-PAYLOAD(T) := <b>                       if the value fits int8 and b != 0x80
                | 80 <N bytes, LE>          otherwise,  N = 4  for sizeof(T) <= 4
                                                        N = 8  for sizeof(T) == 8
```

| source | bytes | note |
|---|---|---|
| `return 127;` | `33 86 41 74 7f` | short form |
| `return 128;` | `33 86 41 74 80 80 00 00 00` | escape |
| `return -5;` | `33 86 41 74 fb` | **short form, `0xfb` = −5 signed** |
| `return -128;` | `33 86 41 74 80 80 ff ff ff` | escape — `0x80` is the marker, so −128 cannot use the short form |
| `return -129;` | `33 86 41 74 80 7f ff ff ff` | escape |
| `return (char)200;` | `33 82 11 70 c8` | short form; `li r3,-56` |
| `return (char)-128;` | `33 82 11 70 80 80 ff ff ff` | escape, **4** payload bytes for a 1-byte type |
| `return (short)-128;` | `33 84 21 11 80 80 ff ff ff` | escape, 4 bytes |
| `return (short)300;` | `33 84 21 11 80 2c 01 00 00` | escape, 4 bytes |
| `return 0xFFFFFFFFu;` | `33 86 42 75 80 ff ff ff ff` | escape, 4 bytes |
| `return -5LL;` | `33 88 81 13 fb` | short form |
| `return (long long)-128;` | `33 88 81 13 80 80 ff ff ff ff ff ff ff` | escape, **8** payload bytes |
| `return 0x1122334455667788LL;` | `33 88 81 13 80 88 77 66 55 44 33 22 11` | escape, 8 bytes |
| `return true;` | `33 82 12 30 01` | short form |

So: the escape width is 4 for any type whose tag is `0x82`/`0x84`/`0x86`, and 8
for tag `0x88`. The short form is a **signed** 8-bit value. The current
`read_varint` rejects `0x81..0xFF` (returns `None`), which is fail-closed and
therefore safe — but it means every negative small literal currently blocks a
function, which is a silent, self-inflicted share of the census.

---

## 4. The fail-closed negative list

Nothing below may be accepted by a widened parser. Each is distinguishable
*before* any emission, and the distinguishing byte is given.

### 4.1 `0x40` — reject the whole production, for now

| what | distinguishing bytes | why |
|---|---|---|
| Any `0x40` at all | the byte `40` at an operand-stream token boundary, always preceded by `33 86 41 74 <lit>` | It is a call, not a cast. The expansions range from one instruction (`fabs`) to a 15-instruction loop (`memcmp`) to an external tail call with a REL24 relocation (`memcpy`, `memset`). |
| `memcpy`/`memset`/`memcmp` (ids 172/173/170) | the extra leading `33 86 41 74 <align> 55 86 41 74` argument(s) | The alignment hint changes the expansion (`01` vs `04` observed). Dropping it, or lowering the call as if it had only its source arguments, mis-emits. |
| Class-layout family (2113–2119) | the `66 02 <tok> <tok>` first argument | The emission depends on the **offset literal values**, not on the id: `…<off=00>…` → nothing, `…<off=04>…` → `cmplwi ; addi ; bclr ; li ; blr`. See §1.4. |
| Ids not in §1.3's table | any other selector value | The id space is a c1xx-internal table that cannot be enumerated from the IL. An unrecognized id must reject. |
| `2119`, `222`, `223`, `815`, `1948`, `337` | their selector values | Semantics UNKNOWN (§1.5). |

The only `0x40` cases that could ever be *safely* accepted are ones where both
the id **and** the argument literals are pinned by a controlled fixture and the
expansion has been verified byte-for-byte — e.g. id 2114 with offset literal
`00`, which is provably nothing. Even that is not free: it requires modelling
the `66` descriptor and the whole argument region so the parse stays aligned.

### 4.2 `0x2C` — reject unless the *source* type is known and pinned

| what | distinguishing bytes | why |
|---|---|---|
| Any `2C` whose stack operand's type the parser did not track | — | The same `2c 86 41 74 00` is a no-op, an `extsb`, an `extsh`, a `clrlwi 24/16`, an `extsw`, or a 3-instruction `fctiwz` sequence, chosen entirely by the source type. |
| Narrowing to `char` / `short` | target triple `82 11 70` / `84 21 11` | `extsb` / `extsh` — real instructions. |
| Narrowing to `unsigned char` / `unsigned short` | `82 12 20` / `84 22 21` | `rlwinm …,0,24,31` / `…,0,16,31`. |
| Widening **from** `char`/`short`/`unsigned char`/`unsigned short`/`bool` | source LOAD's triple, *not* the `2C` | `extsb`/`extsh`/`clrlwi` — this is the case a "converts to `int` are free" rule gets wrong. |
| Anything touching `float`/`double`/`long long` | triples `86 45 40`, `88 85 41`, `88 81 13`, `88 82 23` | `fcfid`/`fctiwz`/`fctidz`/`frsp`/`extsw`/`rldicl`, plus stack traffic through `-16(r1)` that the MVP frame model does not have. |
| A **chain** of `2C` tokens | two or more consecutive `2C` | c2 collapses the chain (§2.2(b)). Per-token lowering over-emits. |
| A `2C` preceded by `0x59` | the `59` byte | fp rounding marker, meaning UNKNOWN (§2.4). |
| `2C` with a non-zero trailing varint | the byte after the type | Never observed; meaning UNKNOWN. |
| Derived↔base pointer conversions | these are **`0x40`**, not `2C` | `(Base *)derived` never uses `2C`. Do not let a "pointer casts are free" rule leak into the `0x40` family. |

The conversions that are **provably free** and could be admitted once the
operand stack carries types (§5):

* `int`↔`unsigned` and `long long`↔`unsigned long long` (either direction)
* cv-qualification strips at the same width — the source triple carries tag
  `0xa6` and a TU-local id (`a6 41 80 20` = `const int`), the target is the plain
  `86 41 74`
* any pointer → any pointer, and array-to-pointer decay over a `26 <sym>` push
* pointer ↔ 32-bit integer
* `float` → `double` (widening only; `double`→`float` is `frsp`)
* `bool` → `char` (but **not** `bool` → `int`/`unsigned`, which is `clrlwi 24`)

---

## 5. What `parse_expr` would need

`crates/c2-il/src/func.rs::parse_expr` currently pushes `IlOp::Load(tok)` /
`IlOp::Lit(i32)` onto a `Vec<IlOp>` and gates every operand on a literal byte
match against `INT_TYPE` (`86 41 74`). The operand stack therefore carries **no
type information at all** — which is exactly why `0x2C` cannot be admitted
today, and why admitting it by matching on the `2C` token's own bytes would be
a mis-emit rather than a widening.

The minimum required change, in dependency order:

1. **A type value, not a byte match.** `read_type` already returns
   `(tag, kind, id, width)` but is used only for skipping. It needs a small
   `IlType` with (a) the size implied by the tag (`0x82`→1, `0x84`→2, `0x86`→4,
   `0x88`→8 — verified across every triple in §2.2), (b) a signedness/class from
   the `kind` low nibble (1 signed int, 2 unsigned int, 3 pointer, 4 function
   pointer, 5 floating; `6` observed but UNKNOWN), and (c) the raw triple for
   census reporting. `INT_TYPE` stays as an *acceptance* predicate; it stops
   being the width rule.

2. **A typed operand stack.** `parse_expr` must return, or maintain alongside,
   a `Vec<IlType>` shadow stack: LOAD pushes the operand's inline type, LIT
   pushes the literal's type, a binary op pops two and pushes the result type.
   This is the piece that does not exist in any form today.

3. **A `Convert { from: IlType, to: IlType }` op**, lowered from the *pair*, not
   from the `2C` token. The allow-list is §4.2's "provably free" set; every
   other pair returns `NotImplemented`.

4. **Peephole the chain.** Consecutive `Convert`s must be folded to a single
   source→target pair before lowering, or `(int)(unsigned char)(short)a` emits
   three instructions where c2 emits one (§2.2(b)).

5. **`read_varint` must take the operand type**, to get the 8-byte escape for
   `long long` and the signed short form (§3.2), and a separate
   `read_float_literal` for the `<8 IEEE bytes> <u16 size>` payload (§3.1).
   Both are needed just to stay *aligned*, before any question of lowering.

6. **`0x40` needs its own token type and stays rejecting.** Decoding it —
   `40 <TYPE>` then an argument loop `(<expr> 55 <TYPE>)* 4C`, plus the `66 02
   <tok> <tok>` descriptor — is required for the *census* to report the
   intrinsic id rather than a single opaque byte, and for the parse to reach the
   segment end in bodies where an intrinsic call is not the blocking feature.
   Decoding is not accepting: the shape parses, `parse_segment` still returns
   `NotImplemented`.

Note that steps 1–2 are also what `expr-load-type-864540` (float, 3.4 %),
`-888541` (double, 3.1 %) and `-864383` (`void*`, 1.9 %) need, and step 6 is
what `expr-cast` (6.8 %) and much of `call-token-0x33` (5.9 %) need. None of
them is a cast-specific change.

---

## 6. Open unknowns, restated

* The `0x40` intrinsic id space beyond §1.3 — in particular `2119`, `222`,
  `223`, `815`, `1948`, and the exact distinction between `2113` and `2114`
  (identical argument shapes, different ids).
* `0x66`'s exact field layout beyond `66 02 <tok> <tok>`; the `02` is fixed in
  every observation but its meaning is unknown.
* `0x59` (§2.4).
* The `0x2C` trailing varint's meaning when non-zero (never observed).
* The type `kind` nibble `6` (seen as `86 46 <id>` in the 222/223 results).
* Whether the `0x40` argument list can contain a nested `0x40` (not observed at
  an aligned site, but not excluded).
