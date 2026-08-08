# `.ex` CALL token and body-statement grammar

**Status: characterization (P2c). No Rust source was changed by this document.**
Every structural claim below cites bytes. Where a claim rests on a *controlled*
fixture (one source construct varied, everything else held fixed) it is marked
**[CF]**; where it rests on the real `system/world/Dir.cpp` capture it is marked
**[DIR]**. Unknowns are called out as unknowns — the port must fail closed on
them, never guess.

Motivation: the CALL/body grammar is the #1 measured blocker of the real
workload. On Dir.cpp (5239 function bodies) the current parser accepts 8. The
blocking buckets `call-anchor-*` (521), `call-token-*` (1283), `body-0x3A`
(245), `body-0x53` (178) are all explained below.

Fixtures used for the controlled experiments live in `work/exp/*.cpp`
(untracked scratch; regenerate with `c2rs census <cpp> --keep-il <dir>`). They
are freestanding and include-free.

---

## 0. Summary of corrections to the current model

| Current model (`crates/c2-il/src/func.rs`) | Reality |
|---|---|
| `CALL := BD <3-byte ret type> 00 80 01 10 00 00` (fixed 10 bytes) | `CALL := BD <type> <cc-flags:1> <varint fn-type-id>`; **variable width**, 8–13 bytes observed |
| `CALL_CALLEE_ANCHOR = 00 80 01 10 00 00` | Not an anchor and not the callee. `00` is a calling-convention flags byte; `80 01 10 00 00` is `read_varint(0x1001)` — the *function-type id*, which is `0x1001` only because it is the first type a single-function fixture TU creates |
| `INT_TYPE : [u8;3] = 86 41 74` — types are 3 bytes | `TYPE := <tag> <kind> <LEB128 id>`; **3, 4 or 5 bytes**. Dir.cpp call sites: 4157 × 3-byte, 3123 × 4-byte, 1358 × 5-byte |
| `26 <tok>` is a call-result reference that must be followed by `BD` | `26 <tok>` is a **symbol/lvalue push**. It is the callee for a direct call, and equally the *destination* of an assignment statement. Two in a row (`26 dest 26 callee BD …`) is normal |
| `read_token_var` (2 bytes, or 4 when byte 1 has bit 7 set) | **Correct — confirmed by controlled fixture.** See §3.2 |
| body is one statement | body is a statement *list*, with nested scopes and control flow |

---

## 1. The three encodings

Three distinct variable-width encodings coexist in `.ex`. Conflating them is
what produced the bogus census buckets.

### 1.1 Operand token — `read_token_var` (2 or 4 bytes) — **CONFIRMED CORRECT**

Used for every *symbol* operand: `26 <tok>`, `B9 <tok>`, `3A <tok>`,
`2D <tok>`, `29 <tok>`, `38/39 <tok>`, and the `.gl` symbol records.

```
tok := b0 b1                      if b1 & 0x80 == 0   -> value = b0 | (b1 << 8)
     | b0 b1 b2 b3                if b1 & 0x80 != 0   -> value = b0
                                                       | ((b1 & 0x7f) << 8)
                                                       | (b2 << 15)
                                                       | (b3 << 23)
```

**[CF] `work/exp/e20.cpp`** — 32000 `extern int vNNNNN;` declarations pushed the
token counter past `0x8000`, forcing the wide form in a body whose surrounding
markers are fixed:

```
b9 e7 86 01 00 | 86 43 83 08 | 55 86 43 83 08 | 4c | 41 86 43 83 08 | 3a e9 86 01 00 | 54 02 29 e9 86 01 00
   ^4-byte tok    ^type          ^arg push       ^apply  ^result type    ^4-byte tok      ^4-byte tok
```

The `41` result-type marker and the `54 02 29` return are fixed byte patterns
and land exactly where a 4-byte token predicts; a 2-byte or 3-byte read
misaligns them. The *values* corroborate it exactly: symbol tokens are allocated
sequentially in declaration order from `0x09E3` in every TU, so `v31999` =
`0x09E3 + 31999` = `0x86E2`, and the load of `v31999` in the second function is
literally `b9 e2 86 01 00` — decoding `0xE2 | (0x06 << 8) | (1 << 15)` = `34530`
= `0x86E2`. Every other token in that TU checks out the same way (`pf`'s
parameter 34531, `pf` 34532, `lastfn`'s parameter 34533, `lastfn` 34534, `f`'s
`q` 34535, `f` 34536, `f`'s return temp 34537, `g`'s `a` 34538, `g` 34539,
`g`'s return temp 34540 — all as observed).

So `read_token_var` stands. Do not change it.

### 1.2 Statement/literal varint — `read_varint` (1 or 5 bytes) — also correct

```
varint := b            if b < 0x80    -> value = b
        | 80 b0 b1 b2 b3              -> value = i32::from_le_bytes([b0,b1,b2,b3])
```

**[CF] `e11.cpp`** (400 statements): `4f 01 80 91 01 00 00` = statement marker,
index `0x191` = 401. **[CF] `e19.cpp`**: char literal `'x'` is `33 82 11 70 78`
(payload `0x78`, the short form).

This is the encoding the CALL token's last field uses (§2).

### 1.3 Type — `<tag> <kind> <LEB128 id>` (3+ bytes) — **NEW**

```
TYPE := tag kind leb128            tag has bit 7 set; leb128 is 7-bit groups,
                                   little-endian, continuation in bit 7
```

Total length is therefore 3 bytes for an id < 0x80, 4 for id < 0x4000, 5 for id
< 0x200000. This is **not** the operand-token rule (§1.1) and **not** simply
"+1 when bit 7 of the second byte is set" — the latter happens to agree for all
ids below 0x4000, which covers most of a small TU, but breaks on big ones.

Evidence for the exact boundary comes from the fact that a type is always
bracketed by fixed one-byte markers:

* **[CF] `e12.cpp`** `void *f(void *p) { return p; }`:
  `4c 4f 11 53 | b9 e3 09 | 86 43 83 08 | 41 | 86 43 83 08 | 3a e5 09 | 54 02 29 e5 09`.
  The `41` result-type marker and the `3A` assign pin `void*` at **exactly 4
  bytes**. A 5-byte (`tag` + 4-byte-token) read would swallow the `41`.
* **[CF] `e8.cpp`** `int f(int a){return g(a)+1;}`:
  `… 02 | 41 86 41 74 | 3a e7 09` pins `int` at **exactly 3 bytes**.
* **[CF] `e23.cpp`** (6000 struct types, all used — pushes ids past 0x4000):
  `33 86 43 9b b9 02 00 | 55 86 43 9b b9 02 | 4c 4b`. The `55` arg-push and the
  `4c 4b` call-end pin this type at **exactly 5 bytes**; its LEB payload
  `9b b9 02` decodes to 40091.
* **[DIR]** Over all 8628 well-formed `BD` sites in Dir.cpp the return-type
  width distribution is 3 → 4157, 4 → 3123, 5 → 1358. Five-byte types are
  ~16 % of real call sites, so a "3 or 4 bytes" rule mis-parses one call in six.

Decoding the LEB payload as a *type-table id* is independently corroborated by
the interleaving with the CALL token's fn-type id — see §2.3.

Observed `(tag, kind)` pairs **[CF]**, with the size implied by the tag
(`tag = 0x80 | (size_in_bytes << 1)`):

| type | bytes | tag | kind | leb id |
|---|---|---|---|---|
| `void` | `82 07 03` | 0x82 | 0x07 | 3 |
| `char` | `82 11 70` | 0x82 | 0x11 | 112 |
| `bool` | `82 12 30` | 0x82 | 0x12 | 48 |
| `unsigned char` | `82 12 20` | 0x82 | 0x12 | 32 |
| `short` | `84 21 11` | 0x84 | 0x21 | 17 |
| `wchar_t` | `84 22 71` | 0x84 | 0x22 | 113 |
| `int` | `86 41 74` | 0x86 | 0x41 | 116 |
| `unsigned` | `86 42 75` | 0x86 | 0x42 | 117 |
| `void *` | `86 43 83 08` | 0x86 | 0x43 | 1027 |
| `char *` | `86 43 f0 08` | 0x86 | 0x43 | 1136 |
| `int *` | `86 43 f4 08` | 0x86 | 0x43 | 1140 |
| `int **` | `86 43 82 20` | 0x86 | 0x43 | 4098 |
| `float` | `86 45 40` | 0x86 | 0x45 | 64 |
| `int (*)(int)` | `86 44 94 20` | 0x86 | 0x44 | 4116 |
| `long long` | `88 81 13` | 0x88 | 0x81 | 19 |
| `double` | `88 85 41` | 0x88 | 0x85 | 65 |

Low nibble of `kind` is a class (1 signed int, 2 unsigned int, 3 pointer,
4 function-pointer, 5 floating). Tag `0xA6` also occurs **[DIR]** and behaves
identically (`a6 43 …`, `a6 42 …`) — it is presumably a cv-qualified variant of
`0x86`; tags `0x96`, `0xC6` also occur. **UNKNOWN:** the exact meaning of the
tag's high bits. A decoder does not need it: it only needs the *width* rule,
which is tag-independent.

**UNKNOWN:** whether the `kind` byte is genuinely a fixed byte or the low half
of a wider field. Every observed `kind` except `0x85`/`0x81` (the 8-byte types)
has bit 7 clear, so a "two consecutive LEBs" reading is also consistent for all
but `88 85 41` / `88 81 13`, which forces the fixed-byte reading. Treat as
`<2 fixed bytes> <LEB>`, which fits 100 % of observations.

---

## 2. The CALL token

### 2.1 Grammar

```
CALL := BD  <TYPE ret>  <flags:1 byte>  <varint fn-type-id>
```

Nothing in it is fixed except the `BD` opcode. A decoder finds the end by
decoding each field — no anchor is needed or possible.

| field | width | meaning |
|---|---|---|
| `BD` | 1 | opcode |
| `<TYPE ret>` | 3/4/5 (§1.3) | the call expression's result type |
| `<flags>` | 1 | calling convention / varargs (§2.2) |
| `<varint fn-type-id>` | 1 or 5 (§1.2) | id of the callee's *function type* (§2.3) |

Observed total width: 8 (`bd 82 07 03 00 05`, hypothetical short varint) through
13 (`bd 86 43 9b b9 02 00 80 9f 9c 00 00`). In practice **[DIR]** 8628 of 8638
call tokens use the 5-byte varint form (function-type ids are ≥ 0x1000 for any
type created in the TU), so 10–12 bytes is the normal range.

The current code's `CALL_CALLEE_ANCHOR = [00 80 01 10 00 00]` is the `flags=0` +
`varint(0x1001)` of a TU whose first-created type happens to be the callee's
function type. That is true of every MVP fixture and of essentially nothing
else, which is exactly the `call-anchor-*` census bucket:

```
[DIR] 361 x call-anchor-0x00
   26 2f 15  bd 82 07 03 >00< 80 80 10 00 00  4c 4b        (fn-type id 0x1080)
[DIR]  93 x call-anchor-0x08
   26 74 00  bd 86 43 83 >08<  00 80 ae 10 00 00  b9 …     (4-byte void* ret type)
[DIR]  67 x call-anchor-0x20
   26 88 66  bd 86 43 9d >20<  00 80 e8 1d 00 00  b9 …     (4-byte ret type)
```

`call-anchor-0x08`/`-0x20` are the parser skipping a fixed 3 bytes of a 4-byte
type and landing on the type's own tail byte. `call-anchor-0x00` is a correct
3-byte type followed by an fn-type id that is not `0x1001`.

### 2.2 The flags byte — **[CF] `e19.cpp`**

One source file, three externals differing only in calling convention, same
`int` return type:

```
extern int va(const char *, ...);   ->  bd 86 41 74 40 80 03 10 00 00
extern int __stdcall sc(int);       ->  bd 86 41 74 00 80 06 10 00 00
extern int __fastcall fc(int);      ->  bd 86 41 74 04 80 07 10 00 00
```

The return type is byte-identical across all three and only this byte moves:
`0x00` = `__cdecl` (and `__stdcall`, which is a no-op on PPC), `0x04` =
`__fastcall`, `0x40` = varargs. Member calls **[CF] `e13.cpp`** also use `0x00`.
**[DIR]** 8628 of 8638 sites are `0x00`; the 10 exceptions are false `BD` hits
inside data.

**UNKNOWN:** whether this field is a plain byte or a `read_varint`. Both readings
agree for every observed value (all < 0x80). A decoder should accept only the
values it has evidence for and fail closed otherwise.

### 2.3 The last field is the *function type*, not the callee — **[CF]**

This is the single most important correction, and it was tested directly.

* **[CF] `e2.cpp`** — three *different* callees, identical signature:
  ```
  extern void g1(); extern void g2(); extern void g3();
  void f() { g1(); g2(); g3(); }

  26 e3 09  bd 82 07 03 00 80 01 10 00 00  4c 4b
  26 e4 09  bd 82 07 03 00 80 01 10 00 00  4c 4b
  26 e5 09  bd 82 07 03 00 80 01 10 00 00  4c 4b
  ```
  The CALL tokens are **byte-identical**; only the `26 <tok>` changes. If the
  field were the callee it would have to differ. **[CF] `e11.cpp`** repeats this
  with 400 distinct callees — all 400 CALL tokens are `bd 82 07 03 00 80 01 10 00 00`.

* **[CF] `e4.cpp`** — one call each to six externals differing *only* in return
  type: the field runs `01, 02, 03, 04, 05, 06`.
* **[CF] `e3.cpp`** — four void externals differing only in parameter count
  (0,1,2,3): the field runs `01, 03, 05, 07` — each new parameter *list* costs
  one extra id, each new function type one more.
* **[CF] `e10.cpp`** — eleven distinct signatures; the CALL fields are
  `01,03,04,09,0a,0b,0c,0d,0e,11,15`, and the *gaps* are exactly filled by the
  pointer types the same calls carry inline: `int**` = LEB id 4098, `S*` = 4102,
  `const int*` = 4112, `int(*)(int)` = 4116, against CALL ids
  `0x1001`=4097, `0x1003`=4099, `0x1009`=4105, `0x1011`=4113, `0x1015`=4117.
  Inline types and CALL fn-type ids interleave in **one** id space, which both
  identifies the field and independently confirms the LEB decoding of §1.3.
* **[CF] `e23.cpp`** at the top of that space: pointer ids 40079, 40085, 40091
  (inline LEB `8f b9 02`, `95 b9 02`, `9b b9 02`) interleaved with CALL fn-type
  ids 40083, 40089, 40095 (varint `80 93 9c 00 00`, `80 99 9c 00 00`,
  `80 9f 9c 00 00`).
* **[CF] `e7.cpp` `.gl`** — the callee's own `.gl` record carries the *same*
  varint: `04 | e3 09 | 00 | "?a@@YAXXZ" 00 | 82 07 04 | 00 00 00 00 | 80 01 10 00 00`.

**Consequence for codegen: the CALL token contains no callee identity at all.**
It must be decoded (to know where the token ends) and then discarded.

---

## 3. Resolving the callee

### 3.1 `26 <tok>` → `.gl`, not `.sy`

`.gl` is the symbol table. Records have the shape

```
<kind byte> <operand token> 00 <NUL-terminated name> 00 <TYPE> … [<varint fn-type-id>]
```

**[CF] `e7.cpp`** (`extern void a(); extern void b(); extern void c();`,
called in the order `c(); b(); a();`):

```
.gl:  04 e3 09 00 "?a@@YAXXZ" 00 82 07 04 00 00 00 00 80 01 10 00 00
      04 e4 09 00 "?b@@YAXXZ" 00 …
      02 04 e5 09 00 "?c@@YAXXZ" 00 …
      c2 0e e6 09 00 "?f@@YAXXZ" 00 …
.ex:  26 e5 09 bd …   26 e4 09 bd …   26 e3 09 bd …
```

Tokens are assigned in **declaration** order (`a`=0x09E3, `b`=0x09E4,
`c`=0x09E5, `f`=0x09E6) and used in **call** order — which is what proves
`26 <tok>` names the callee rather than a per-call temp. The complementary test
**[CF] `e5.cpp`** `g1(); g2(); g1(); g1();` yields tokens `e3, e4, e3, e3`: the
token *repeats* for a repeated callee.

Note that a function's formal parameters are declared before the function
itself, so a callee's token is not `first_token + index`; the table must be read.

**[DIR] verification.** Building a token→name index from `.gl` by that record
shape (5947 named records) and applying it to every direct `26 <tok> BD` site in
Dir.cpp resolves **2323 of 2323 (100 %)**, to plausible names —
`ldiv`, `lldiv`, `memmove`, `memchr`, `??$min@I@stlpmtx_std@@YAABIABI0@Z`,
`?MemPushTemp@@YAXXZ`, `?MemPopHeap@@YAXXZ`, …

`.sy` is **not** the symbol table for this purpose. Dir.cpp's `.sy` (384 KB)
contains local/parameter names (`this`, `s`) and 186 static-member mangled
names; `MemPushTemp` is absent from `.sy` and present in `.gl`. `e7.cpp`'s `.sy`
is 13 bytes total. Nothing in the call path needs `.sy`.

### 3.2 The callee *expression* is not always `26 <tok>`

`BD` is a postfix operator applied to whatever the preceding operand stream
pushed. Three shapes are observed:

```
direct    26 <callee>                                     BD …     [CF] e1,e2,e5,e7
indirect  b9 <tok> <TYPE>                                  BD …     [CF] e14
member    26 <method> <obj-expr> 99 <TYPE> <varint>        BD …     [CF] e13
```

* **[CF] `e14.cpp`** `int f(FN p, int a){ return p(a); }` →
  `53 | b9 e5 09 86 44 82 20 | bd 86 41 74 00 80 02 10 00 00 | b9 e6 09 86 41 74 55 86 41 74 | 4c`.
  **No `26` at all.** An indirect call has no callee name anywhere.
* **[CF] `e13.cpp`** `s->set(3); return s->get();` →
  `26 e7 09 | b9 ee 09 86 43 81 20 | 99 86 43 86 20 00 | bd 82 07 03 00 80 06 10 00 00 | 33 86 41 74 03 55 86 41 74 | 4c 4b`.
  `99 <TYPE> <varint>` binds the object to the member function.

This is the `call-token-0xB9` bucket — **809 functions on Dir.cpp, the single
largest**:

```
[DIR] 53 26 71 0a  b9 8b 0a 86 43 9d 20  99 86 43 90 20 00  bd 82 12 30 00 80 10 10 00 00
```

which now reads exactly: statement start, method symbol `0x0A71`, load `this`
(`0x0A8B`, type `86 43 9d 20`), bind-member (`99`, type `86 43 90 20`, offset 0),
CALL returning `bool` (`82 12 30`), cdecl, fn-type id `0x1010`.

---

## 4. `26 <tok>` and the statement grammar

`26 <tok>` is a **symbol / lvalue push**. It appears in three roles, and only
context distinguishes them:

1. the callee of a direct call (consumed by `BD`);
2. the destination of an assignment (consumed by `32 <TYPE>`);
3. the function's own symbol in the pre-`LO` header (`53 53 26 <fn> 46`).

**[CF] `e15.cpp`** `extern int gv, gw; extern int h(int); void f(int a){ gv=a; gw=7; gv=h(a); }`:

```
4c 4f 11 53
  26 e3 09  b9 e7 09 86 41 74            32 86 41 74  4b     gv = a;
  26 e4 09  33 86 41 74 07               32 86 41 74  4b     gw = 7;
  26 e3 09  26 e6 09 bd 86 41 74 00 80 01 10 00 00
            b9 e7 09 86 41 74 55 86 41 74 4c
                                          32 86 41 74  4b     gv = h(a);
3a e9 09  54 02 29 e9 09  4f 12 47 54 01 54 00
```

`32 <TYPE>` is the store; `4B` ends the expression statement. Two `26` in a row
(`26 dest 26 callee BD …`) is the ordinary "assign a call result" statement —
this is the `call-token-0x26` bucket (115 **[DIR]**). A void call statement has
no destination and no `32`, so it is `26 <callee> BD … 4C 4B` **[CF] e1**.

**[CF] `e16.cpp`** `int x=h(a); int y=h(x); return x+y;` shows the same shape for
locals, three statements in one body, with `4B` between them and *no* `53`
between them.

### 4.1 Statement-list layout

```
body      := 'LO'(4C 4F 11) 53 stmt* return-plumbing 4F 12 47 54 01 54 00 …
stmt      := stmt-marker? ( expr-stmt | scope | control )
stmt-marker := 4F 01 <varint>            (a source-line/sequence index)
expr-stmt := <operand stream> 4B
scope     := 53 stmt* 54 03
```

* `53` opens the body **and** each nested scope/control statement. **[CF]
  `e18.cpp`** and **[CF] `e26.cpp`**.
* `4F 01 <varint>` line markers appear between statements in multi-statement
  TUs. **[CF] `e11.cpp`**: `4f 01 80 91 01 00 00` = 401. Note the current
  `eat_opt_stmt_marker` assumes a **fixed 1-byte** index and so mis-parses any
  index ≥ 0x80 — this is the `body-0xAD` / `body-0xB3` / `body-0xD6` / `body-0xF3`
  census family **[DIR]** (`4c 4f 11 53 4f 01 80 >ad< 02 00 00 …`: 0x80-prefixed
  5-byte varints being read as one byte).

### 4.2 `body-0x3A` = an **empty** function body

**[CF] `e17.cpp`** `void f() {}` →

```
4c 4f 11 53 | 3a e4 09 | 54 02 29 e4 09 | 4f 12 47 54 01 54 00
```

There are no statements at all: the body opens and goes straight to the return
plumbing, so the first byte after the opening `53` is the `3A` assign.

**[DIR]** confirms this by name — the 245 `body-0x3A` functions are empty inline
stubs:

```
fn51 ??3@YAXPAX0@Z                                   53 3a 34 16 54 02 29 34 16 4f 12 47 54 01 54 00
fn55 ?_M_initialize@_STLP_mutex_base@stlpmtx_std@@…  53 3a 5d 16 54 02 29 5d 16 4f 12 …
fn56 ?_M_destroy@_STLP_mutex_base@stlpmtx_std@@…     53 3a 5f 16 54 02 29 5f 16 4f 12 …
fn57 ?_M_acquire_lock@…                              53 3a 61 16 54 02 29 61 16 4f 12 …
fn68 ?_Destroy_Range@stlpmtx_std@@YAXPAD0@Z          53 3a 2c 17 54 02 29 2c 17 4f 12 …
```

**UNKNOWN (documented):** a minority variant carries a trailing expression
*after* the `54 02 29 <tok>` and before `4F 12`:

```
[DIR] fn48  53 3a 19 16 54 02 29 19 16 | b9 18 16 a6 43 d4 21 41 a6 43 d4 21 | 4f 12 47 54 01 54 00
```

i.e. a value-returning function with an empty statement list. I cannot account
for the operand order here (`3A`/`54 02 29` before the value expression, the
reverse of every other body). A decoder must reject it rather than guess.

### 4.3 `body-0x53` = first statement is a control/compound statement

**[CF] `e26.cpp`** `int f(int a){ if (a>0) { return g(a); } return 0; }`:

```
4c 4f 11 53
 53                                             <- the `if` statement opens a scope
   b9 e5 09 86 41 74  33 86 41 74 00  24        <- a > 0     (24 = cmp-gt)
   38 e8 09                                     <- branch-if-false -> label 0x09E8
   53 53                                        <- then-clause + its compound block
     26 e4 09 bd 86 41 74 00 80 01 10 00 00
     b9 e5 09 86 41 74 55 86 41 74 4c
     41 86 41 74 3a e7 09                       <- store the return value
     54 05                                      <- goto epilogue
   54 04 29 e8 09                               <- define label 0x09E8
 54 03                                          <- close
 33 86 41 74 00 41 86 41 74 3a e7 09
 54 02 29 e7 09  4f 12 47 54 01 54 00
```

**[DIR]** the census witness is the same shape:
`4c 4f 11 53 4f 01 1f >53< b9 70 0a 86 43 83 20 38 8f 0a 53 …` — a leading `if`.

**[CF] `e18.cpp`** additionally shows a bare `{ … }` nested block producing the
same `53 … 54 03` bracket.

Control vocabulary observed (all **[CF] e18/e26**): `38 <tok>` branch-if-false,
`39 <tok>` branch (variant), `29 <tok>` label/temp reference, `54 03` scope end,
`54 04 29 <tok>` label definition, `54 05` goto-epilogue, `54 02 29 <tok>`
function return. **UNKNOWN:** the full `54 NN` table; treating `54 NN` as a
2-byte control op plus a separate `29 <tok>` operand is what makes real bodies
parse (§6), but I have direct source evidence only for `02/03/04/05`.

---

## 5. Multi-argument calls

```
CALL-EXPR := <callee-expr> CALL <arg>* 4C
arg       := <operand stream producing the value> 55 <TYPE>
```

* Arguments are emitted in **reverse declaration order**.
* Each argument is terminated by `55 <TYPE>`, where `<TYPE>` is the *parameter's*
  type (the conversion target), not the argument expression's type.
* `4C` ends the argument list and applies the call. A zero-argument call is
  `CALL 4C` with nothing between.
* `4B` (statement end / discard) follows only when the call is a statement, i.e.
  its value is unused. It is **not** part of the call.

**`4C`'s WIDTH is one byte, and it is measured on the ARGUMENT-BEARING
population** (`lane w-4c`, board **#1383**). This is worth stating here because
the obvious evidence is the *wrong* evidence: a zero-argument call's `4C` is
trivial to anchor — it is the byte the `BD` token ends on — and board **#1318**
measured 26,701 of them and **declined to pin the width**, because the `4C` that
closes a call *with* arguments is 2.46 M of the 3.5 M `BD` tokens and was not in
that population at all.

Measured over **1,978,436** argument-bearing sites (`work/w-4c/argwalk.py`),
anchored by walking the argument region and stopping AT the first `4C` — never
stepping over one, so the site's position is fixed by the *other* tokens' widths:
payload-free desyncs **0** once the residue is disposed of (§below); `4C <one
byte>` fails at 1,460,194; `4C <TYPE>` at 214,003, and at **87.7 %** of sites the
next byte's bit 7 is clear so there is nowhere for a TYPE to be; `4C <token>` at
1,371,969. Confirmed a second way by a fresh capture whose calls take 0, 1, 2 and
3 arguments, graded `ReferenceReplay=ByteExact` with the three argument-bearing
functions `Port=Match` (`work/w-4c/probe/ce_args.cpp`).

Two structural facts fell out and belong with the grammar above:

* **The closing `4C` follows the last argument's `55 <TYPE>` at 1,956,648 of
  1,978,436 sites, and every one of the 21,788 exceptions is a `0x64`** — the
  by-value-return materialize, which sits between the last argument and the `4C`
  (`… 55 <TYPE> · 9B <agg> <tok> · 2C <A*> 00 · 64 <A*> · 4C`).
* **160,539 anchored calls have a non-empty argument region with NO `55` at
  all** — the same by-value-return family without arguments. `<arg>*` above is
  the argument grammar, not the whole of what can appear between `CALL` and
  `4C`.

**And `0x59`/`0x08` are OPCODES that appear immediately after a float-returning
call's `4C`, not payload.** They are unpinned in every width table here; they
occur at token-start positions **6,031** and **3,819** times and **never** after
a `4C` (`work/w-4c/unwit.py`), which is what separates the two readings.

**[CF] `e3.cpp`** `void f(int a,int b,int c){ h0(); h1(a); h2(a,b); h3(a,b,c); }`,
formals `a`=0x09ED, `b`=0x09EE, `c`=0x09EF:

```
26 e3 09 bd … 4c 4b                                                  h0()
26 e5 09 bd … b9 ed 09 86 41 74  55 86 41 74                4c 4b    h1(a)
26 e8 09 bd … b9 ee 09 86 41 74  55 86 41 74
                b9 ed 09 86 41 74  55 86 41 74               4c 4b    h2(a,b)   <- b then a
26 ec 09 bd … b9 ef 09 …55…  b9 ee 09 …55…  b9 ed 09 …55…   4c 4b    h3(a,b,c) <- c,b,a
```

**[CF] `e9.cpp`** `void p3(char,float,int); … p3('x', 2.0f, a);` shows the
per-parameter type:

```
b9 ea 09 86 41 74                      55 86 41 74        <- a   , int
33 86 4a 40 <8 IEEE bytes> 04 00       55 86 45 40        <- 2.0f, float
33 82 11 70 78                         55 82 11 70        <- 'x' , char
4c 4b
```

Two further facts fall out: a floating literal is `33 <TYPE> <8 raw IEEE-754
double bytes> <2-byte size field>` (`04 00` for `float`, `08 00` for `double`) —
*not* a varint; and the literal's own type token differs from the parameter's
(`86 4a 40` vs `86 45 40`; `88 8a 41` vs `88 85 41`).

Varargs **[CF] `e19.cpp`**: the ellipsis arguments are pushed with the type
`86 00 74` (`kind = 0x00`) rather than a declared parameter type.

An alternative arg-push opcode `64 <TYPE>` also occurs **[DIR]**
(`… 2c 86 43 9e 20 00  64 86 43 9e 20  …`). **UNKNOWN:** how it differs from
`55`.

The existing code's `55 <INT_TYPE> 4C` is exactly the one-argument, `int`
special case of the above.

### 5.1 The token that most often follows the `4C` — `5C`, the EH LIVE-STATE marker

Not part of the call, and recorded here because `w-4c` pinned `4C`'s width and
the very next floor the instrument hit was this byte (board **#1390**, then
**#1423**). Grammar, **measured**:

```
EH-LIVE := 5C <TYPE> <varint state>
```

Emitted at the end of a statement in which an object with a destructor became
live — `docs/EH_RECORDS.md` §7.1, which measured the width on 2026-07-31 and
whose finding is that it is **not** a ctor/dtor token: `int userfn(int a){ MemA
s; g(a); return a+1; }` carries one too.

**It was never unwitnessed.** `control_flow.rs`'s `operand()` has read exactly
this width since WEH, and four `ctor_dtor.rs` recognizers eat `5C <TYPE>
<state>` inside shapes the differential grades byte-exact. What `expr.rs`'s
`chain_skip_form` was missing was the ROW — and unlike `0xBD`, `SkipForm` could
already spell the width, so this was an omission and not an expressiveness
problem.

Measured over **335,716** anchored workload sites (`work/w-5c/scwalk.py`),
anchored by walking from the tree's own `LO` body marker with the whole
`5C`/`5D`/`5E` family removed from the stepper and stopping **AT** the token, so
the site's position is fixed by the other tokens' widths:

| reading | desyncs / 335,716 | |
|---|---:|---|
| **`5C <TYPE> <varint>`** | **0** | **0.000 %** |
| payload-free | 335,716 | 100.000 % — and the byte after a `5C` has bit 7 set at **every** site, so there is nowhere for a payload *not* to be |
| `5C <TYPE>` | 210,570 | 62.723 % |
| `5C <varint>` | 130,991 | 39.018 % |
| `5C <TYPE> <token>` | 59,181 | 17.628 % |

A second, walk-free anchor — `55 <TYPE> 4C 5C`, i.e. §5's own argument-closing
call-end with the marker after it — sees **37,742** sites at **0** desyncs and
lands inside a token the first anchor stepped **0** times.

**`5C` is a statement-terminal trailer and NOT a bracket, which is why it is
worth a different sentence from `4C`'s.** `4C` closes every call; the bodies
that carry a `5C` carry a **median of 1** and a mean of **1.245**. Under the
pinned reading the marker stands immediately before the `4B` statement end at
**275,112 of 335,716 (81.95 %)**; the other 18.05 % stand before a `9B`, `55`,
`99`, `26` or `30`, which is the **operand-position** spelling `EH_RECORDS.md`
§7.2 records beside the statement one.

**The state field is a `read_varint` and not a raw byte, and the corpus decides
it** — unlike §2.2's call-flags byte, which it does not. The two readings agree
at every state below `0x80`, and the anchored walk reaches **zero** escaped
sites. An over-inclusive raw scan (bias stated, base rate printed) finds
**9,744** sites whose state byte is `80`: the varint reading lands on the `4B`
at **9,645 (98.98 %)** against a **60.66 %** base rate, and the one-raw-byte
reading at **0 (0.00 %)**. Every one of those 9,645 is the same byte sequence,
`5C 86 41 74 80 01 01 00 00 4B` in 812 TUs — which is `EH_RECORDS.md` §7.1's
published escape, reproduced at this master.

**`5D`/`5E`, the EH COUNT trailers, are `<varint n> <varint state>` and stay
UNPINNED in `chain_skip_form`** — no `SkipForm` variant can spell `<varint>
<varint>`, which is `0xBD`'s expressiveness problem and an enum change rather
than a table row. They are where 61 % of `5C`'s traffic goes next: with `5C`
stepped, `expr-op-0x5D` reads **+14,699** and `expr-op-0x5E` **+7,670** over the
878-TU workload.

---

## 6. What `parse_segment` / `parse_call_shape` must become

### 6.1 Required changes

1. **Replace the CALL token match** with a decode:
   `BD`, then `read_type` (§1.3), then a flags byte restricted to
   `{0x00}` (accepting `0x04`/`0x40` would require a fastcall/varargs codegen
   the port does not have), then `read_varint` for the fn-type id, which is then
   **discarded**. Delete `CALL_CALLEE_ANCHOR`.
2. **Add `read_type`**: `<tag with bit 7 set> <kind> <LEB128>`. `INT_TYPE` stays
   as the *acceptance* test for the MVP class, but the *width* must come from
   `read_type` so that a non-int type is skipped correctly when it is legal to
   skip one, and reported as a whole type in the census either way.
3. **Callee binding must come from `.gl` by token**, not from
   "`names[n_defined..]`". Build a token→name index from `.gl` records
   (`<kind> <token> 00 <name> 00 …`) and resolve the `26 <tok>` that precedes
   the `BD`. This also removes the single-function/single-external restriction
   that positional pairing forces.
4. **`26 <tok>` must be re-modelled as a symbol push**, with the call shape
   recognized as `<callee-expr> BD <args> 4C`, not `26 <tok> BD`.
5. **Argument region**: loop `expr` + `55 <TYPE>` until `4C`, instead of a single
   `expr` + `55 INT 4C`.
6. **`eat_opt_stmt_marker` must use `read_varint`**, not a fixed 1-byte index.
7. **Statement list**: the body is `53 stmt*`, statements separated by `4B`,
   not a single statement.

### 6.2 Shapes that must still fail closed

None of these may be accepted by a widened parser; each is a real construct the
port has no codegen for, and each is distinguishable *before* any emission:

* **Indirect calls** — callee expression is `b9 <tok> <TYPE>` (or any expression)
  with no `26`. There is no callee name to relocate against. **[CF] e14**
* **Member calls** — `26 <method> <obj-expr> 99 <TYPE> <varint>` before `BD`.
  Needs a `this` argument and possibly a vtable dispatch. **[CF] e13**,
  **[DIR] 809 functions**
* **Non-cdecl / varargs** — flags byte `0x04` or `0x40`. **[CF] e19**
* **A callee token that does not resolve in `.gl`** — must reject, never
  fabricate or fall back to positional pairing.
* **Any type that is not `86 41 74`** in an operand or result position, even
  though its *width* is now known. Knowing how to skip a `double` is not knowing
  how to lower it.
* **Control flow** — a second `53`, any `38`/`39` branch, any `54 NN` other than
  the `54 02 29 <tok>` return. **[CF] e18/e26**
* **The `0x66` call-family opcode** (§7) and every other unidentified opcode.
* **The `body-0x3A` trailing-expression variant** (§4.2).
* **Multi-statement bodies** beyond whatever statement class is deliberately
  added — the whole-body positive parse must still reach the segment end.

### 6.3 Validation of the model

To falsify the grammar rather than admire it, I implemented it as a throwaway
Python whole-body parser (`work/exp/tools/gram2.py`, untracked) that consumes
each of the 5239 Dir.cpp bodies field-by-field and requires the parse to land
**exactly** on the fixed 7-byte function tail `4F 12 47 54 01 54 00`. Any width
error anywhere in a body misaligns that tail and fails.

```
bodies: 5239   landed exactly on the function tail: 2729 (52.1%)
```

All 24 controlled fixtures parse at 100 %. The residue is dominated by
*unidentified opcodes*, not width errors — the top blocker is `0x66` at 1148
bodies (§7). This is the strongest available evidence that §1–§5 are right: a
wrong token width, type width, or CALL-token layout would not let half of a
1.5 MB real translation unit land on a fixed 7-byte pattern.

---

## 7. Ranked unknowns (next characterization targets)

Counts are bodies blocked in the §6.3 validation parser, so they rank what is
left *after* the CALL grammar is implemented.

| op | bodies | observed shape | note |
|---|---|---|---|
| `0x66` | 1148 | `66 02 <tok> <tok>` then an argument region ending in `4C` | A second CALL-family opcode. **[DIR] fn41**: `33 86 41 74 80 41 08 00 00  40 86 43 a5 21  66 02 b1 21 8a 21  55 86 41 74 … 4c` — a size literal, a cast, then `66`. Likely operator-new / temporary construction. The first token varies with the enclosing function, the second (`8a 21`) is stable across fn41/42/45/46 |
| `0x67` | 125 | `67 <byte> <tok>` immediately after `LO 53 [marker]` | A function-prologue op (`67 04 75 2d`, `67 34 9f 2e`, `67 38 a1 2e`) |
| `0x19` | 161 | `19 <TYPE>` | unary, result type follows |
| `0x43` | 91 | always `43 42 00 00` in observation | the census calls `0x43` "ternary" |
| `0x5c` | 75 | `5c <TYPE> <varint>` directly after a call's `4C` | e.g. `5c 86 41 74 01`, `5c a6 43 ea 28 01` |
| `0x44` | 31 | `44 <TYPE>` | unary |
| `0x0f`/`0x35`/`0x36`/`0x1a`/`0x1c` | ~100 | mostly `<op> <TYPE>` | |

**SUPERSEDED for `0x67`, and for `0x64` (which this table does not list at all) —
see `docs/IL_DECODE_REACH.md`.** `0x67` is **virtual dispatch**, not a prologue
op: `67 <varint vtable-BYTE-offset> <method token>`, followed by two indirect
loads, a `9A <TYPE>` slot bind and the call. The row's *"`67 <byte>`"* is the
reading this document could reach — `04`, `34`, `38` are all below `0x80`, where
a plain byte and a signed varint are the same bytes. A class with forty virtual
functions separates them (`67 80 80 00 00 00 <tok>` at slot 32) and the plain-byte
reading costs 926 bodies on the 878-TU workload. `0x64` is the **by-value
return's materialize**, `64 <TYPE>`, in the slot a `BD` occupies and closed by the
same `4C`.

Identified along the way and folded into the validation parser (all consistent,
none contradicted): `0x30 <TYPE>` indirect load, `0x27 <TYPE>` pointer add,
`0x40 <TYPE>` cast/convert, `0x32 <TYPE>` store, `0x41 <TYPE>` result-type
annotation, `0x2C <TYPE> <varint>` address-of/decay,
`0x99`/`0x9B <TYPE> <varint>` member bind, `0x1F` (no operand, produces `bool`).

Member access is a composition rather than an opcode — **[CF] `e21.cpp`**
`struct S{int a,b; double c; int d;}; use(s->d) + s->b`:

```
b9 f0 09 86 43 83 20        load s
33 86 41 74 10              literal 16      <- offsetof(S,d)
27 86 43 f4 08              pointer add     -> int*
30 86 41 74                 indirect load   -> int
…
33 86 41 74 04              literal 4       <- offsetof(S,b)
```

---

## 8. Reproduction

```
# controlled fixtures (untracked scratch)
./target/release/c2rs census work/exp/e2.cpp  --keep-il work/exp/il_e2
python3 work/exp/tools/dump.py work/exp/il_e2

# real workload
./target/release/c2rs census src/system/world/Dir.cpp \
    --flags-file work/dc3-workload/flags.txt --cwd ../dc3-decomp
python3 work/exp/tools/gram2.py work/il-scratch
```

Fixture index: `e1` void tail call · `e2` 3 distinct void callees · `e3` arg
counts 0–3 · `e4` 6 return types · `e5` repeated callees · `e7` declaration vs
call order · `e8` framed int call · `e9` mixed parameter types · `e10` 11 exotic
return types · `e11` 400 callees · `e12` `void*` passthrough · `e13` member
calls · `e14` indirect call · `e15` global assignment · `e16` multi-statement ·
`e17` empty body · `e18` nested scope + `if` · `e19` calling conventions ·
`e20` 32000 symbols (wide tokens) · `e21` struct member offsets ·
`e23` 6000 types (wide type ids) · `e24`–`e28` return-plumbing probes.
