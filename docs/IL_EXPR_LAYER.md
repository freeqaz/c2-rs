# The `.ex` expression layer — designators, loads, and what still refuses

**Status: characterization (P2e), plus one implemented production.** Every
structural claim cites bytes from a capture made for this document. Claims resting
on a **controlled fixture** (one construct varied, the rest held fixed) are marked
**[CF]** and name the tracked file in `fixtures/cpp/`; claims resting on an
untracked scratch probe are marked **[P]** and name it (`work/expr/pN.cpp`,
regenerable — see §12). Unknowns are written as UNKNOWN and the port fails closed
on them.

Scope: the *operand stream*. The statement layer is `docs/IL_STMT_GRAMMAR.md`, the
call region `docs/IL_CALL_GRAMMAR.md`, conversions `docs/IL_CAST_CONVERT.md`, the
TYPE encoding `docs/IL_TYPE_TAGS.md`. Where this document has to quote them it uses
their readings.

---

## 0. Summary, and the corrections

```text
expr        := <operand-stream token>*

designator  := 26 <tok>                    a named symbol (local, global, array)   §5
             | B9 <tok> <TYPE>             a LOADED value — a pointer rvalue is
                                           itself a designator                     §3
             | 9B <TYPE> <tok>             a compiler TEMPORARY                     §7
             | <designator> <off> 27 <PTR-TYPE>    byte-offset add, re-typing       §4
             | <designator> <off> 28 00 00         byte-offset add, subscript       §4
             | <designator> <lit> <lit> 43 37      bitfield extract                 §8

rvalue      := 30 <TYPE>                   indirect load through a designator       §3
             | <designator> <value> 32 <TYPE>      store, yields the stored value    §6
             | 2C <TYPE> <varint>           convert                    (CAST doc)
             | 40 <TYPE>                    intrinsic call             (CAST doc)
             | 43 42 <2 bytes>              conditional expression                   §8
             | 44                           UNKNOWN, payload-free                    §7
```

| Prior model | Reality |
|---|---|
| `0x9B` is a member bind (census bucket `body-0x9B`, 1.1%) | It is a **compiler temporary designator**, and its trailing field is a whole `read_token_var`, not the varint the adjacent `99` uses. §7 — and this resolves the one real-TU counterexample in `IL_STMT_GRAMMAR.md` §12.4 |
| `44 <TYPE>` unary (`IL_CALL_GRAMMAR.md` §7) | `44` is **payload-free** at both captured sites; the following byte is `30`/`55`, whose bit 7 is clear, so it cannot be a TYPE. §7 |
| `0x43` is "ternary", token `43 42 00 00` (4 bytes) | `0x43` is an **escape with a sub-opcode byte** and a sub-opcode-dependent payload width: `43 42 <2 bytes>` is the conditional expression, `43 37` is a bitfield extract and carries **nothing**. §8 |
| `0x27` is "pointer add" and `0x28` unidentified | Both are byte-offset adds and both are *distinct from* `p + k`, which is the ordinary `02` ADD. `27` re-types the designator and carries a TYPE; `28` is the subscript operator and carries two bytes whose meaning is UNKNOWN. §4 |
| Member access needs a member opcode | It is a **composition** — `B9` base, an offset literal, `27`, `30` — with no member opcode at all. `IL_CALL_GRAMMAR.md` §7 found this from one probe; §4 here pins the type fields and the offset semantics. |
| A pointer TYPE's tag is its width (`IL_TYPE_TAGS.md` §1) | The tag is the width of the value the *token* deals with, so the **same** pointer type appears with different tags in different positions: `double *` is `86 43 c1 08` as a `B9` operand and `88 43 c1 08` as a `27` result. §2 |
| Narrow-integer *arithmetic* extension is not implementable (`IL_TYPE_TAGS.md` §3.2) | Still true, and untouched. What is implementable is the 4-byte-integer **load**, which needs none of it. §3, §10 |

---

## 1. Why this layer, measured

`docs/IL_STMT_GRAMMAR.md` §14.1 measured the operand-TYPE gate as the 3.2× step on
a real TU (192 → 615 whole-body decodes) against +7 for the whole statement layer,
and the P2 workload census ranks `expr-*` families at roughly a quarter of blocked
functions. The statement-assignment production landed and moved the workload census
by **zero** (87,423 before and after), because those bodies were blocked on operand
types as well.

The pointer families are the bulk of what "operand type" means in this corpus. A
probe of the top census bucket (`call-token-0xB9`, 18%) shows what it actually
contains — assignments whose right-hand side the expression parser cannot model —
and the constructs are the ones this document decodes.

---

## 2. Positional tags: the same type, two tags

`IL_TYPE_TAGS.md` §1 reads a TYPE as `<tag> <kind> <LEB128 id>` with the tag giving
the width. That holds, but *which* value's width depends on the token the type
belongs to. **[CF] `il_expr_index.cpp`** and **[P] `p3.cpp`**, the same four member
reads out of one struct:

```
struct S { int a; double c; char e; int g; };

s->a   b9 <s> 86 43 81 20  33 86 41 74 00  27 >86< 43 f4 08  30 86 41 74
s->c   b9 <s> 86 43 81 20  33 86 41 74 08  27 >88< 43 c1 08  30 88 85 41
s->e   b9 <s> 86 43 81 20  33 86 41 74 10  27 >82< 43 f0 08  30 82 11 70
s->g   b9 <s> 86 43 81 20  33 86 41 74 14  27 >86< 43 f4 08  30 86 41 74
```

The `27` result tag runs `86 / 88 / 82` for pointees of 4 / 8 / 1 bytes, while the
same pointer type as a `B9` operand always carries `86` (a pointer is 4 bytes).
**[P] `p2.cpp`** has `double *` both ways in one TU: `86 43 c1 08` loaded,
`88 43 c1 08` produced by `27`.

The reading that fits: a `27` yields a **designator** (an lvalue) and its tag is the
size of the designated object; a `B9` or a `30` yields an **rvalue** and its tag is
the size of that value. So the tag is not a property of the type, and a decoder must
not key an allow-list on the whole triple. The width rule itself (`tag & 0x0F` =
`2·(log2 size + 1)`, so `2`→1, `4`→2, `6`→4, `8`→8) is stable across every tag
family: `86`, `A6` (const), `96` (volatile), and `82`/`84`/`88`.

The `kind` low nibble likewise moves with the position. `27`'s result is normally
`kind & 0x0F == 3` (pointer), but with a `26 <sym>` base rather than a pointer
rvalue it is `86` — **[P] `p5.cpp`**, `S t = { a, 1 };`:

```
26 <t> 33 86 41 74 00  27 >86 86< f4 08  b9 <a> 86 41 74  32 86 41 74 4b   store t.a
26 <t> 33 86 41 74 04  27 >86 86< f4 08  33 86 41 74 01   32 86 41 74 4b   store t.b
26 <t> 33 86 41 74 04  27 >86 43< f4 08  30 86 41 74      41 86 41 74      read  t.b
```

Same struct, same member, same offset, three lines apart: `kind` is `86` for the two
stores and `43` for the read. UNKNOWN why. Harmless if a decoder reads the type for
its *width* and takes the operation from the opcode, which is what the port does.

---

## 3. `30 <TYPE>` — the indirect load

```text
LOAD-INDIRECT := 30 <TYPE of the loaded value>
```

Pops a designator, pushes the value. The TYPE is the **result**, so it is where the
loaded width lives, and it selects the instruction. **[CF] `il_expr_deref.cpp`** and
**[CF] `il_expr_load_neg.cpp`**, one function per pointee type, everything else held
fixed:

| source | IL | c2 emits |
|---|---|---|
| `int f(int* p){return *p;}` | `30 86 41 74` | `80630000` `lwz r3,0(r3)` |
| `unsigned f(unsigned*)` | `30 86 42 75` | `80630000` `lwz` |
| `long f(long*)` | `30 86 41 12` | `80630000` `lwz` |
| `int f(const int*)` | `30 a6 41 84 20` + `2c 86 41 74 00` | `80630000` `lwz` |
| `int f(volatile int*)` | `30 96 41 86 20` + `2c 86 41 74 00` | `80630000` `lwz` |
| `char f(char*)` | `30 82 11 70` | `88630000` `lbz r3,0(r3)` |
| `short f(short*)` | `30 84 21 11` | `a0630000` `lhz r3,0(r3)` — `lhz`, **not** `lha` |
| `float f(float*)` | `30 86 45 40` | `c0230000` `lfs f1,0(r3)` |
| `double f(double*)` | `30 88 85 41` | `c8230000` `lfd f1,0(r3)` |
| `int* f(int**)` | `30 86 43 f4 08` | `80630000` `lwz` |
| struct copy (`S t = *s;`) | `30 a6 86 8d 20` | `lfd f11 ; stfd ; lwz` — through the FP unit |

Three things worth stating because a plausible rule gets each wrong:

* **A `const`/`volatile` pointee changes the IL but not the code.** The qualification
  rides into the load type and is then stripped by a `2C`; both captures emit a bare
  `lwz` with nothing added. That is the case the port needs, because a `const`
  accessor is the common real shape (§10).
* **`short` loads with `lhz`, not `lha`**, when the value is returned as a `short`.
  Signedness is not recoverable from the load instruction here.
* **`int **` emits the same word as `int *`** and is refused anyway (§10) — the gate
  is "the loaded value is a 4-byte integer", not "the emitted word happens to match".

The base register is the pointer's own incoming argument register. **[CF]
`il_expr_deref.cpp`**, the ladder: `f(int* p)` → `lwz r3,0(r3)`,
`f(int a, int* p)` → `lwz r3,0(r4)`, `f(int a, int b, int* p)` → `lwz r3,0(r5)`,
`f(int* p, int a)` → `lwz r3,0(r3)`.

---

## 4. Byte-offset adds: `27 <TYPE>`, `28 00 00`, and plain `02`

There are **three** spellings and they are not interchangeable productions, even
though all three lower to the same instruction. **[CF] `il_expr_index.cpp`**, all in
one TU:

```
p[1]        b9 <p> 86 43 f4 08  33 86 41 12 04  >28 00 00<        30 86 41 74
*(p + 1)    b9 <p> 86 43 f4 08  33 86 41 12 04  >02<              30 86 41 74
s->b        b9 <s> 86 43 81 20  33 86 41 74 04  >27 86 43 f4 08<  30 86 41 74
```

All three → `lwz r3,4(r3)`. And `*(p - 1)` is `03` (SUB) while `p[-1]` is `28` with
a negative literal; both → `lwz r3,-4(r3)`.

So:

* **c1xx does NOT desugar `p[k]` to `*(p+k)`.** `28` is the subscript operator.
* **`02` ADD is polymorphic** over (pointer, integer) — not integer-only.
* **`27` carries a TYPE and re-types the designator** (`S *` → `int *`); `28` leaves
  the type alone and carries two bytes instead.

**The offset is always in bytes and already scaled.** `p[1]` on an `int *` is
literal 4, `p[3]` is 12, and on a `double *` it is 8. With a *variable* index the
scaling is an explicit `04` MUL by an element-size literal:

```
p[i]  (int*)     b9 <p> … b9 <i> 86 41 74  33 86 41 12 04  04  28 00 00  30 86 41 74
p[i]  (char*)    b9 <p> … b9 <i> 86 41 74  33 86 41 12 01  04  28 00 00  30 82 11 70
p[i]  (double*)  b9 <p> … b9 <i> 86 41 74  33 86 41 12 08  04  28 00 00  30 88 85 41
```

— note the `char *` case multiplies by a literal 1 that does nothing, so the element
size is nowhere in the `28` token itself.

**The offset literal's own type differs between the two forms.** A subscript offset
is typed `86 41 12` (`long`); a member offset is `86 41 74` (`int`). A parser that
hardcoded `int` there loses every subscript.

The literal uses the ordinary payload rules of `IL_CAST_CONVERT.md` §3.2 — the
**signed** short form (`fc` = −4 for `p[-1]`) and the `80`-escape for anything
larger (`80 00 7d 00 00` = 32000 for `p[8000]`).

### 4.1 The `28` payload — UNKNOWN

The two trailing bytes are `00 00` at **every** site captured: constant and variable
indices; 1-, 4- and 8-byte elements; negative indices; `p[i][j]` on an
`int (*)[4]`, which chains two `28`s; `w->v[2]`, which chains a `27` then a `28`;
`p[i].b`, which chains a `28` then a `27`; a local array (`26 <sym>` base); a string
literal; a bitfield base; and offsets past the 16-bit displacement. **[CF]
`il_expr_index.cpp`**, **[P] `p3.cpp`**, **[P] `p8.cpp`**.

**A fixture that would separate them: unknown.** I could not construct a source
that moves them, and that failure is itself the reason this is recorded as UNKNOWN
rather than as "a fixed two-byte pad". The port requires exactly `00 00`.

### 4.2 Pointer difference is a front-end shift

**[P] `p4.cpp` `int pdiff(int* p, int* q) { return (int)(p - q); }`**

```
b9 <p> 86 43 f4 08  b9 <q> 86 43 f4 08  03  33 86 41 74 02  0a  2c 86 41 74 00
->  subf r11,r4,r3 ; srawi r3,r11,2
```

`03` SUB over two pointers then `0a` (`>>`) by `log2(sizeof)` — c1xx emits the
division by the element size as an arithmetic shift, so there is no pointer-specific
opcode on this side either.

---

## 5. `26 <tok>` as a designator, and the store asymmetry

`26 <tok>` is a symbol push (`IL_CALL_GRAMMAR.md` §4). It is also a *designator*: a
local array is subscripted straight off it, with no array-to-pointer decay.
**[P] `p5.cpp` `int t_arr(){ int v[4]; v[0] = 1; return v[0]; }`**

```
26 <v> 33 86 41 12 00 28 00 00  33 86 41 74 01  32 86 41 74 4b
26 <v> 33 86 41 12 00 28 00 00  30 86 41 74     41 86 41 74
```

A string literal is the same shape (**[P] `p5.cpp` `"abc"[0]`** →
`26 <sym> 33 86 41 12 00 28 00 00 30 a2 11 99 20 2c 86 41 74 00`), which is where
the `2C` array-decay of `IL_CAST_CONVERT.md` §2.2 comes from — it is a property of
the *use*, not of the symbol.

---

## 6. `32 <TYPE>` means two different things

`IL_STMT_GRAMMAR.md` §4.3 records that a store through a pointer has no deref
opcode. That is worth restating as a hazard, because the *same* token is a register
copy or a memory write depending only on what pushed the destination:

```
[P] p2.cpp  int f(int* p,int v){ *p = v; return v; }
  b9 <p> 86 43 f4 08   b9 <v> 86 41 74   32 86 41 74  4b
  ->  stw r4,0(r3) ; or r3,r4,r4                                  a MEMORY WRITE

    stmt_local_assign   x = a + 1;
  26 <x>               b9 <a> … 33 … 02  32 86 41 74  4b
  ->  addi r3,r3,1                                                a REGISTER COPY
```

`try_parse_assign_body` requires a `26 <tok>` destination, so it is safe today; any
widening that admits a designator expression on the left changes the meaning of the
`32` it already accepts.

The store's own codegen is not trivial either. **[P] `p2.cpp`
`int f(int* p){ *p = 7; return 0; }`**:

```
mr r11,r3 ; li r10,7 ; li r3,0 ; stw r10,0(r11)
```

Two scratch registers and the store sunk past the return value, for a one-line body.

---

## 7. `9B <TYPE> <token>` — the compiler temporary, and `99`'s different field

The census bucket `body-0x9B` (27,190 functions, 1.1%) had no reading. **[CF]
`il_expr_temp.cpp`**, `S t = mk();` with `mk()` returning a struct:

```
26 fe 09                           push the local `t`
9b 86 86 89 20 ff 09               the TEMPORARY designator, token 0xFF09
26 f6 09 bd 86 86 89 20 00 … 4c    call mk() -> S
32 86 86 89 20                     store the result into the temporary
9b 86 86 89 20 ff 09               the same temporary again
44                                 (payload-free — see below)
30 a6 86 8d 20                     load the struct out of it
32 86 86 89 20                     store into t
4b
```

(The `86 86 89 20` type id is TU-local — `S` is whatever index this TU assigned it —
so the load-bearing part is the `kind` nibble and the trailing field's width, not the
id.)

**The trailing field is a `read_token_var`, not a varint.** The decisive test is
**[P] `p9.cpp`** — 32000 `extern int vNNNNN;` declarations, which push the token
counter past `0x8000` and force the wide form:

```
9b 86 86 80 20 >f2 86 01 00<
```

Four trailing bytes. `f2 86 01 00` decodes to `0x86F2` = 34546, exactly one past
`t` (34545) and two past the epilogue label (34544) in that TU's sequential
allocation. Under the varint reading the field is `f2` and the parse resumes on
`86 01 00 26 …`, which is not a token boundary.

**This resolves `IL_STMT_GRAMMAR.md` §12.4** — the single scope-depth counterexample
across 5239 Dir.cpp bodies. The quoted neighbourhood is

```
… 9b 86 46 80 20 >11 54< 2c 86 43 9e 20 00 64 86 43 9e 20 4c …
```

and the `54` that the scanner treated as a scope close is the second byte of the
2-byte token `11 54`. There is no statement-layer counterexample left.

**`99` does not widen.** The complementary test **[P] `p10.cpp`** is the same
32000-symbol TU with a member function:

```
b9 ee 86 01 00 a6 43 82 20  99 86 43 84 20 >00<  46 4c 4f 11
```

still one trailing byte with a 34000-token space, and the `46` formals marker
follows immediately — which it must, since every non-member function has one. So
`99 <TYPE> <varint>` and `9B <TYPE> <token>` are adjacent opcodes with **different**
trailing-field encodings and neither is inferable from the other.

**UNKNOWN: `0x44`.** Payload-free at both sites (`44 30 …` and `44 55 …`); the
following byte's bit 7 is clear, so it cannot be a TYPE, which contradicts
`IL_CALL_GRAMMAR.md` §7's provisional `44 <TYPE>`. It sits between a temporary
designator and a use of it. "Materialize / bind" is the obvious guess and nothing
here tests it.

**UNKNOWN: `99`'s trailing byte.** Zero in every observation, including a member
function of a class with a base. **A fixture that would separate it:** a member call
on a class with multiple or virtual inheritance, where a `this` adjustment is needed
— if the byte is an offset it should become non-zero. Not tested.

### 7.1 Member functions: `this` is not a formal

**[CF] `il_expr_member.cpp`.** The pre-body region carries `this` separately from the
`2D` formals list:

```
53 53 26 <fn>  b9 <this> a6 43 82 20  99 86 43 84 20 00  46  2d <q>  4c  4F 11
               ^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^
               LOAD this (C * const)  bind-member, offset 0
```

`this` occupies r3, so **every explicit formal of a member function is one register
higher than its `2D` index implies**:

```
int C::g(int* q) const        { return *q; }  ->  80640000  lwz r3,0(r4)
int C::i(int v, int* q) const { return *q; }  ->  80650000  lwz r3,0(r5)
int D::s(int* q)              { return *q; }  ->  80630000  lwz r3,0(r3)   (static)
```

A rule that ignored `this` would emit `lwz r3,0(r3)` for the first — plausible,
wrong register, wrong bytes. `this`'s own type carries tag `a6` (`C * const`) whether
or not the function is `const`; what `const` changes is the *pointee*
(`a6 41 …` const int against `86 41 74` int) and hence whether a `2C` cv-strip
follows the load.

---

## 8. `0x43` is an escape with a sub-opcode

**[CF] `il_expr_ternary.cpp`**, two functions in one TU:

```
a ? b : 2    b9 <a> 86 41 74  b9 <b> 86 41 74  33 86 41 74 02  >43 42 00 00<  41 86 41 74

b->g         b9 <b> 86 43 8e 20  33 86 41 74 00  27 86 43 f5 08
             33 86 41 74 18  33 86 41 74 05  >43 37<  30 86 42 75  2c 86 41 74 00
```

`43 42` carries two trailing bytes; `43 37` carries **none** — the byte after it is
`30`, the indirect load. So the payload width is a function of the sub-opcode, and a
decoder that treats `0x43` as a fixed four-byte token desynchronizes on every
bitfield read in the corpus. `IL_CALL_GRAMMAR.md` §7's "always `43 42 00 00`" and the
census name `expr-ternary` are both generalizations from one sub-opcode.

Sub-opcode `0x42` is the conditional expression. `0x37` builds a bitfield designator
from (shift, width) literals — 24 and 5 for the second 5-bit field of
`struct B { unsigned f:3; unsigned g:5; }`, i.e. `32 − 3 − 5` and the width, not the
byte offset.

The conditional is **not** a select:

```
a ? b : 2      cmpwi cr6,r3,0 ; mr r3,r4 ; bclr 4,26 ; li r3,2 ; blr
a > 1 ? 5 : 6  cmpwi cr6,r3,1 ; li r3,5  ; bclr 1,25 ; li r3,6 ; blr
```

A conditional *return*, two exits, the compare fused into a condition-register field,
and the `bclr` condition bits moving with the relation. It is control flow wearing an
expression's clothes.

**UNKNOWN:** `43 42`'s two trailing bytes (`00 00` in both captures, whose conditions
differ — a bare value and a relational). **A fixture that would separate them:** a
conditional whose arms are lvalues, or one nested inside another. Not tested.
**UNKNOWN:** the rest of the sub-opcode space. Only `42` and `37` are witnessed, so
the census should report `expr-op-0x43-NN` rather than one bucket named after one
sub-opcode.

---

## 9. A call consumed as a value — no new opcode, and it is nearly free

`expr-call-in-expr` (164,544, 7.0%) plus much of `call-token-0x26` (125,226, 5.3%).
**[CF] `il_expr_call_value.cpp`**:

```
int z = g1(a); return z;
  26 <z>                                     the destination
  26 <g1> bd 86 41 74 00 80 14 10 00 00      the call
  b9 <a> 86 41 74 55 86 41 74 4c             its argument
  32 86 41 74 4b                             store into z
  b9 <z> 86 41 74 41 86 41 74                return z
```

**The reference obj is byte-identical to `return g1(a);`** — a bare
`b ?g1@@YAHH@Z` with a REL24 relocation, no frame, no store. c2 register-allocates
`z` away exactly as it does for a non-call initializer. And
`int z = g1(a); return z + 1;` is byte-identical to `return g1(a) + 1;`, i.e. the
already-implemented framed non-leaf class (0x24-byte frame, `bl`, `addi r3,r3,1`,
epilogue).

So the two existing call shapes already cover this bucket's simplest members and the
missing piece is entirely in the IL model: `try_parse_assign_body` resolves
statements by substituting *operand streams*, and a call value is not one. This is
the cheapest remaining rung in this document. It is not attempted here.

The gate any such widening must keep is two calls in one body — a real non-leaf with
a frame and two `bl`s, which nothing about the single-call shape generalizes to.

---

## 10. What is implemented, and why exactly this much

The **indirect-load leaf**: a whole body that is one load through a pointer.
`c2_il::try_parse_indirect_load_leaf`, `c2_core::codegen::indirect_load_text`.

```text
indirect-load-leaf :=
    B9 <base-tok> <PTR-TYPE>                     the base pointer
    [ 33 <int-like> <off>  27 <PTR-TYPE> ]       ONE member byte-offset add, or
    [ 33 <long>     <off>  28 00 00      ]       ONE subscript byte-offset add
    30 <INT4-TYPE>                               the indirect load
    [ 2C <int-like> 00 ]                         a cv-qualification strip
    41 <int-like>                                result type
    <return plumbing, reaching the segment end>
```

→ `lwz r3, off(rBase)` + `blr`, with the offset folded into the displacement.
**[CF] `il_expr_deref.cpp`** (16 functions) and **[CF] `il_expr_member.cpp`** (6),
both `Port=Match` byte-exact. Covers `return *p;`, `return s->m;`, `return s.m;`,
`return p[k];`, `return mMember;` and `return this->mMember;` for a 4-byte integer
member, const or not, on a formal or on `this`.

Gates, each a captured case where the same-looking IL lowers differently —
**[CF] `il_expr_load_neg.cpp`** (17 functions, all refusing):

| gate | what it keeps out | captured emission |
|---|---|---|
| the loaded value is a **4-byte integer** (`tag & 0x0F == 6`, `kind & 0x0F ∈ {1,2}`) | `char`/`short`/`float`/`double`/pointer pointees | `lbz` / `lhz` / `lfs` / `lfd` |
| **exactly one** offset add | `p[i][j]`, `p[i].b`, `w->v[i]` | two adds, `slwi ; add ; slwi ; lwzx` |
| the offset is a **literal** | `p[i]` | `slwi r11,r4,2 ; lwzx r3,r11,r3` |
| the offset fits **i16** | `p[100000]` (offset 400000) | `lis r11,6 ; ori r11,r11,0x1a80 ; lwzx` |
| `28`'s payload is exactly `00 00` | anything else | meaning UNKNOWN (§4.1) |
| **nothing after the load** but the return | `*p + 1`, `*p + b`, `*p + *q`, `*p * 3` | see below |
| the base is a register argument, with `this` at index 0 | a member function whose `this` binding cannot be located | wrong base register |
| the destination is not a store | `*p = v` | `stw r4,0(r3) ; or r3,r4,r4` |

The "nothing after the load" gate is the load-bearing one, because the obvious
lowering is wrong bytes rather than out of range:

```
int f(int* p)        { return *p + 1; }   ->  lwz r11,0(r3) ; addi r3,r11,1
int f(int* p,int b)  { return *p + b; }   ->  lwz r11,0(r3) ; add  r3,r11,r4
int f(int* p,int* q) { return *p + *q; }  ->  lwz r10,0(r3) ; lwz r11,0(r4)
                                              add r3,r10,r11
int f(int* p)        { return *p * 3; }   ->  lwz r11,0(r3) ; slwi r10,r11,1
                                              add r3,r11,r10
int f(S* s)          { return s->a+s->b; }->  lwz r11,4(r3) ; lwz r10,0(r3)
                                              add r3,r11,r10
```

The load lands in the **scratch** register r11, not the destination, so
`lwz r3,0(r3) ; addi r3,r3,1` is wrong. `*p * 3` is strength-reduced to
`x + (x << 1)` with no `mullw` at all — the same rewriter that turns `a + a` into
`slwi` (`fixtures/cpp/il_repeated_leaf.cpp`). And the scratch order is r10-then-r11
with two distinct bases but r11-then-r10 with one base read twice, with the `add`
operands reversed. The allocator is not a rule this port has.

`IlOp::LoadInd` is therefore produced only by this parser, only as the second op of
a two-op stream, matched by `indirect_load_text` as an **exact** pattern (not a
prefix), and rejected outright by `select_text`. It cannot reach the reassociation
or repeated-leaf gates.

The cv-strip is admitted because it is provably free over the source class the parser
has already pinned: `IL_CAST_CONVERT.md` §2.2/§4.2 shows int↔unsigned and
cv-qualification strips at the same width emit nothing, and both captures
(`const int *`, `volatile int *`) are a bare `lwz`. It is *not* a general `2C` rule —
the identical token over a `char`/`short`/`float` source is a real instruction.

### 10.1 A latent bug this work surfaced

`parse_formals` anchored on the **first** `0x46` byte before the `LO` marker. A
function on source line 70 emits the line marker `4F 01 46`, whose payload byte *is*
`0x46`, and the per-function `4F 33 …` header region is a run of opaque bytes that
freely contains `0x46`. In those cases it returned an **empty** formals list rather
than failing — which is not fail-closed, because `leaves_ascending` skips tokens that
are not formals, so a body whose formals silently vanished bypassed the reassociation
ordering gate entirely.

It now anchors on the `46` whose `(2D <tok>)*` run lands **exactly** on `LO`. Caught
because one of sixteen otherwise-identical functions in `il_expr_deref.cpp`
(`ld_ixneg`, at source line 70) disagreed with its neighbours two lines away. Unit
test `parse_formals_anchors_on_the_marker_that_reaches_lo`. Worth **+11 functions** on
the workload census; its value is the safety property, not the coverage.

---

## 11. Measured census effect

`c2rs gap --list work/dc3-workload/files.txt --flags-file work/dc3-workload/flags.txt
--cwd <dc3-decomp> --jobs 12`, three points, all against the same tree so the
deltas are attributable (a peer session was advancing `func.rs` concurrently, so the
"before" was re-measured with only this work's hunks reverted rather than taken from
an earlier run):

| | in class | share |
|---|---:|---:|
| before (both hunks reverted) | 87,423 / 2,462,571 | 3.55 % |
| + the `parse_formals` anchor fix only | 87,434 | 3.55 % |
| **+ the indirect-load leaf** | **110,596** | **4.49 %** |

`match 6/878`, `mismatch 0` at every point. So: the formals fix is **+11 functions**
and is worth landing for the safety property alone; the indirect-load leaf is
**+23,162 functions (+0.94 pp)**, the first widening in three rungs to move the
census by more than a rounding error.

The re-ranked widening order after this change, top of the census:

```
427706 (18.2%)  call-token-0xB9                    <- still #1; a conflated bucket, see below
170401 ( 7.2%)  body-0x53                          statement-layer control flow
164544 ( 7.0%)  expr-call-in-expr                  §9 — the cheapest remaining rung
137496 ( 5.8%)  call-intrinsic-this-adjust         the 0x40 family
135754 ( 5.8%)  expr-intrinsic-base-member-addr    the 0x40 family
125226 ( 5.3%)  call-token-0x26                    also §9
 85928 ( 3.7%)  expr-load-type-864540              float operand
 79542 ( 3.4%)  expr-load-type-888541              double operand
 49189 ( 2.1%)  expr-load-type-864383              void* operand
```

**`call-token-0xB9` is a conflated bucket and its 18.2% should not be read as one
feature.** `try_parse_assign_body` returns a bare `Option`, so when a body opens on
`26 <tok>` and the assignment parse refuses, the census reports whatever
`parse_call_shape` then trips over — which is the `B9` of the right-hand side, every
time, whatever the real cause. Probed **[P] `p1.cpp`**, three unrelated constructs
land in it: `int x = *p;` (a pointer operand), `short s = (short)a;` (a `2C`), and
an `if` statement. Splitting it needs `try_parse_assign_body` to carry a `Block`, and
the discriminator is clean — whether the token after the first `26 <tok>` is `BD`
decides assignment against call with no ambiguity. Not done here (that function was
being edited concurrently), and it is the highest-value *measurement* change
available, because the widening order is ranked off this histogram.

---

## 12. Reproduction

```
cargo build --release -p c2-harness

# tracked fixtures
for f in fixtures/cpp/il_expr_*.cpp; do
  ./target/release/c2rs census "$f" --keep-il "work/expr/il_$(basename "$f" .cpp)"
  ./target/release/c2rs diff "$f"
done
python3 work/w6-dump.py work/expr/il_il_expr_deref          # per-body IL from `LO`
python3 work/exp/tools/dump.py work/expr/il_il_expr_member   # whole segment, incl. formals
./target/release/c2rs compile fixtures/cpp/il_expr_deref.cpp --keep-obj /tmp/x.obj
python3 work/expr/tools/objdis.py /tmp/x.obj                 # .text per symbol, disassembled
```

Untracked probes (`work/expr/`, regenerate from the bytes quoted above):
`p1` the five constructs the census bucket `call-token-0xB9` actually contains ·
`p2` the pointee-type ladder + stores · `p3` the `27` pointee tag + the `28` scale ·
`p4` `this`, `p+k` against `p[k]`, pointer difference · `p5` temporaries, local
arrays, string literals, compound literals · `p6` the base-register ladder +
arithmetic over a load · `p7` `this` register placement (member, static, with
formals) · `p8` 2-D arrays, bitfields, ternary, call-in-expr · `p9` 32000 symbols,
the `9B` wide-token separator · `p10` the same for `99` · `p11` casts in a statement.
`work/expr/tools/objdis.py` and `work/expr/tools/il.sh` are the scratch tooling.
