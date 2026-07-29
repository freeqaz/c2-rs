# W6 — integer comparisons → boolean materialization (CHARACTERIZATION)

Empirical spec for roadmap rung **W6**: the `.ex` vocabulary c1xx emits for the
six C relational operators, and the branchless PPC boolean-materialization
idioms `c2` lowers them to. **No code was changed by this document** — it is the
byte evidence a W6 implementation must be built against and graded on.

Every byte below is transcribed from a **live 16.00.11886.00 capture** made this
session:

* IL: `c2rs census <cpp> --keep-il <dir>` (`/Bd /d2nop /Ox /GS- /c`).
* obj: `wibo compilers/X360/16.00.11886.00/cl.exe /Ox /GS- /c /Fo… <cpp>`.
* Disassembly: hand-decoded from the big-endian `.text` words, then
  **re-encoded from fields and compared against the observed word** — all 29
  distinct instruction words in this document round-trip bit-exactly. Nothing
  here is inferred from "how MSVC probably works"; anything not established is
  marked **UNRESOLVED**.

The tracked fixture is `fixtures/cpp/il_bool_materialization.cpp` (6 functions).
The supporting probes (57 more functions across 8 TUs) were written to
`work/w6-probes/` — gitignored scratch, deliberately not added to `fixtures/`.

Token width in every bundle here is **2 bytes** (all observed tokens have bit 7
of their second byte clear: `e3 09`, `00 0a`, `09 0a`, …). Note the general rule
is **per token**, not a per-bundle constant: 2 bytes normally, 4 bytes when bit 7
of the second byte is set. No 4-byte token occurs in this fixture or any W6
probe, so `detect_token_width`'s single global answer is accidentally right here
and must not be relied on for a real TU.

---

## 1. IL vocabulary established

### 1.1 Comparison opcodes — one byte, sign-agnostic

| relation | opcode | evidence (signed) | evidence (unsigned) |
|---|---|---|---|
| `==` | **`0x1F`** | `?s_eq`, `?sc_eq`, `?s_eq0` | `?u_eq`, `?uc_eq`, `?equality_nonzero` |
| `!=` | **`0x20`** | `?s_ne`, `?sc_ne`, `?s_ne0` | `?u_ne`, `?uc_ne`, `?zero_test`, `?inequality_nonzero` |
| `<=` | **`0x21`** | `?s_le`, `?sc_le` | `?u_le`, `?uc_le` |
| `<`  | **`0x22`** | `?s_lt`, `?sc_lt`, `?constleft` | `?u_lt`, `?uc_lt` |
| `>=` | **`0x23`** | `?s_ge`, `?sc_ge` | `?u_ge`, `?uc_ge` |
| `>`  | **`0x24`** | `?s_gt`, `?sc_gt`, `?signed_ordered` | `?u_gt`, `?uc_gt`, `?unsigned_ordered` |

**The opcode byte does not encode signedness.** The signed and unsigned probe
TUs (`cmp_signed_vv.cpp` / `cmp_unsigned_vv.cpp`, and the `_vc` pair) produce
*identical* comparison opcode bytes for the same relation; the only IL
difference is the operand type triple. Signedness is therefore carried **only**
by the operands. Independently confirmed at the obj level: the two TUs emit
*different* `.text` for `<`,`<=`,`>`,`>=` and *identical* `.text` for `==`,`!=`.

`0x25` is **not observed** — `0x1F..0x24` is already a complete 6-relation block,
so `0x25` is something else. Unknown.

The comparison is a **postfix binary operator** exactly like `0x02` ADD: it pops
rhs then lhs. Source operand order is preserved verbatim by c1xx — no
canonicalization: `?constleft` (`return 7 < a;`) emits `LIT 7 · LOAD a · 0x22`,
i.e. the literal is genuinely the lhs.

### 1.2 Neighbouring operand-stream opcodes (all newly verified here)

| byte | meaning | probe |
|---|---|---|
| `0x02` `0x03` `0x04` | ADD SUB MUL | (already known) |
| `0x05` `0x06` | DIV MOD | `?dvi` `?mdi` |
| `0x09` `0x0A` | `<<` `>>` | `?shl` `?shr` |
| `0x0B` `0x0C` `0x0D` | bitwise `&` `\|` `^` | `?band` `?bor` `?bxor` |
| `0x1A` | logical `!` (unary) | `?notx` |
| `0x1B` | logical `\|\|` | `?oror` |
| `0x1C` | logical `&&` | `?andand` |
| `0x1F`–`0x24` | the six comparisons | §1.1 |
| `0x2C <type> 00` | **CONVERT** to `<type>` | everywhere |
| `0x43 0x42 00 00` | ternary select | `?tern` |

> **Census-label defect found.** `c2_il::func::Block::feature`'s named map is
> wrong for three of six comparisons and is missing `0x1F`:
> it says `0x20`→`cmp-eq`, `0x21`→`cmp-ne`, `0x23`→`cmp-le`, `0x25`→`cmp-ge`.
> Measured: `0x1F`=eq, `0x20`=ne, `0x21`=le, `0x22`=lt (already right),
> `0x23`=ge, `0x24`=gt (already right), `0x25` unobserved. This mislabels the
> P2 real-workload gap histogram — the "`expr-cmp-eq`" bucket is actually `!=`,
> "`expr-cmp-ne`" is `<=`, "`expr-cmp-le`" is `>=`, and every `==` lands in the
> unnamed `expr-op-0x1F` bucket. `0x1A`/`0x1B`/`0x1C`/`0x2C` also deserve names.
> Diagnostic only — acceptance is unaffected — but the ranked blocker list in
> `docs/GAPS.md` is keyed on these strings.

### 1.3 Type triples

| triple | type | first seen |
|---|---|---|
| `86 41 74` | `int` | (already known) |
| **`86 42 75`** | **`unsigned int`** | the W6 fixture |
| `82 12 30` | `bool` | `?retbool_gt7` result-type |
| `82 11 70` | `char` | `?cmpchar` |
| `84 21 11` | `short` | `?cmpshort` |
| `88 81 13` | `long long` | `?cmpll` |
| `86 45 40` | `float` | (already known) |
| `86 4a 40` | `double` (literal) | `?cmpfl` |

Both comparison operands always carry the **same** triple at the compare. When
the source mixes signedness, c1xx inserts the conversion itself:
`?mixed` (`int a > unsigned b`) is
`B9 a 86 41 74 · 2C 86 42 75 00 · B9 b 86 42 75 · 24` — the int operand is
converted to unsigned *before* the compare, and c2 duly emits the **unsigned**
idiom. So a W6 parser can read signedness off the shared operand type at the
compare and never has to reason about C's usual arithmetic conversions.

### 1.4 The `0x2C` CONVERT after a comparison

A comparison's natural result type is `bool` (`82 12 30`). When the value is
returned as something else, c1xx appends a convert:

```
?retbool_gt7  … 24 | 41 82 12 30 |          3a …   (bool return: NO convert)
?retint_gt7   … 24 | 2c 86 41 74 00 | 41 86 41 74   (int  return)
?i_gt7        … 24 | 2c 86 42 75 00 | 41 86 42 75   (uint return)
```

All three emit **byte-identical `.text`** (`cmp_misc.obj` `0x00`/`0x20`/`0x40`),
so **the bool→int/uint convert is a codegen no-op** — the comparison spine
already leaves a full-width 0/1 in r3.

`0x2C` is **not** universally free. `?cmpchar` is
`B9 a 82 11 70 · 2C 86 41 74 00 · …` — the same token shape applied to a `char`
LOAD, and it lowers to a real sign-extension (`extsb`, word `7c6a0774`). A W6
parser must therefore accept `0x2C` **only when its stack operand is a
comparison result**, never as a general "casts are free" rule (§8).

---

## 2. The six fixture functions, annotated

Segment prefix (`4F 1F` … `4C 4F 11`) is **byte-identical to the add3 class** in
all six: `FnHeader · 4F 02 20 00 4F 01 NN block-start · 53 53 · 26 <tok> ·
46 2D <tok>` — already decoded by `c2_il::codec`. Only the body differs, so all
listings below start at the `4C 4F 11` LO marker, as captured.

Common shape (offsets relative to LO):

```
+0x00  4c 4f 11                LO        body marker
+0x03  53                      SS        statement start
+0x04  4f 01 NN                STMT      line/statement marker          [known]
+0x07  b9 <tok> <T>            LOAD      the formal, type T
+0x0d  33 <T> <varint>         LIT       literal, type T
+0x12  <cmp>                   CMP       one byte, §1.1     -> bool
+0x13  2c 86 42 75 00          CVT       bool -> unsigned int
+0x18  41 86 42 75             RESTYPE   unsigned int
+0x1c  3a <tok>                ASSIGN    -> return temp
+0x1f  4f 01 NN                STMT      line/statement marker          [known]
+0x22  54 02 29 <tok>          RETURN    return temp
+0x27  4f 12                   SEP
+0x29  47 54 01 54 00          GT        terminate      (= 0x2e bytes)
```

**No UNKNOWN tokens remain in any of the six bodies.** Every byte from LO to the
segment end is accounted for by the table above; the only tokens `parse_segment`
does not model today are the comparison byte and the `2C` convert, and both are
now identified. (The FnHeader *interior* before LO is still opaque, exactly as
for the already-in-class add3 shape, and is length-invariant.)

### `zero_test` — `unsigned int zero_test(unsigned int x) { return x != 0; }`

```
4c 4f 11 53 4f 01 02 b9 e3 09 86 42 75 33 86 42 75 00 20 2c 86 42 75 00
41 86 42 75 3a e5 09 4f 01 03 54 02 29 e5 09 4f 12 47 54 01 54 00
```
`LOAD 0xE309 uint` · `LIT uint 0` · **`20` = NE** · `CVT uint` · `RESTYPE uint`
· `ASSIGN 0xE509` · `RETURN 0xE509`.

### `equality_nonzero` — `return x == 1;` (unsigned)

```
4c 4f 11 53 4f 01 06 b9 e6 09 86 42 75 33 86 42 75 01 1f 2c 86 42 75 00
41 86 42 75 3a e8 09 4f 01 07 54 02 29 e8 09 4f 12 47 54 01 54 00
```
`LIT uint 1` · **`1f` = EQ**.

### `inequality_nonzero` — `return x != 1;` (unsigned)

```
4c 4f 11 53 4f 01 0a b9 e9 09 86 42 75 33 86 42 75 01 20 2c 86 42 75 00
41 86 42 75 3a eb 09 4f 01 0b 54 02 29 eb 09 4f 12 47 54 01 54 00
```
`LIT uint 1` · **`20` = NE**.

### `signed_positive` — `unsigned int signed_positive(int x) { return x > 0; }`

```
4c 4f 11 53 4f 01 0e b9 ec 09 86 41 74 33 86 41 74 00 24 2c 86 42 75 00
41 86 42 75 3a ee 09 4f 01 0f 54 02 29 ee 09 4f 12 47 54 01 54 00
```
Operands are **`86 41 74` (int)** → signed compare; `LIT int 0` · **`24` = GT**;
the convert/result-type are still `86 42 75` (the *return* type).

### `unsigned_ordered` — `return x > 7;` (unsigned)

```
4c 4f 11 53 4f 01 12 b9 ef 09 86 42 75 33 86 42 75 07 24 2c 86 42 75 00
41 86 42 75 3a f1 09 4f 01 13 54 02 29 f1 09 4f 12 47 54 01 54 00
```
`LIT uint 7` · **`24` = GT**, operands unsigned.

### `signed_ordered` — `unsigned int signed_ordered(int x) { return x > 7; }`

Last function in the TU, so it carries the module end:

```
4c 4f 11 53 4f 01 16 b9 f2 09 86 41 74 33 86 41 74 07 24 2c 86 42 75 00
41 86 42 75 3a f4 09 4f 01 17 54 02 29 f4 09 4f 12 47 54 01 54 00
4f 02 20 00 4f 01 18 4d
```
`LIT int 7` · **`24` = GT**, operands int → signed.

### Correction: `4F 01 NN` is a *line* marker, not a multi-function marker

`docs/IL_BUNDLE_MVP.md` says `4F 01 NN` is "multi-fn only". That is wrong. A/B
capture of the **same single function** differing only in source formatting:

```
"unsigned int f1(unsigned int x) {\n    return x != 0;\n}"
   → 4c 4f 11 53 | 4f 01 02 | b9 … 3a e5 09 | 4f 01 03 | 54 02 29 …
"unsigned int f1(unsigned int x) { return x != 0; }"
   → 4c 4f 11 53 |           | b9 … 3a e5 09 |           | 54 02 29 …
```

Both compile to identical `.text` (`3163ffff 7c6b1910 4e800020`). The marker
tracks a **source line advance** and is codegen-irrelevant. `parse_segment`
already tolerates it in both positions (`eat_opt_stmt_marker`), so nothing
breaks — but the doc comment is misleading, and the W6 fixture (multi-line
bodies) has markers while every single-line probe does not.

---

## 3. Reference `.text` — the materialization idioms

`work/w6-obj/bool.obj` — 1140 B, 5 sections (`.drectve`, `.debug$S`,
`.XBLD$W` ×2, `.text`), 19 symbols, **no `.pdata`** (all six are leaves),
`.text` = 108 B, characteristics `0x60400020` (CODE | **ALIGN_8** | EXECUTE |
READ), zero relocations. Function starts are padded to an **8-byte** boundary
with `00 00 00 00`; the tail of `.text` is **not** padded (108 is not
8-aligned). This is exactly the multi-function layout `c2_core`'s
`compile`/`emit_obj` path already implements (`while text.len() % 8 != 0`), so
W6 needs **no new obj-shell work** — only a new `select_text` shape.

```
?zero_test@@YAII@Z          @0x00  31 63 ff ff 7c 6b 19 10 4e 80 00 20  (+ 4 pad)
?equality_nonzero@@YAII@Z   @0x10  39 63 ff ff 7d 6a 00 34 55 43 df fe 4e 80 00 20
?inequality_nonzero@@YAII@Z @0x20  39 63 ff ff 31 4b ff ff 7c 6a 59 10 4e 80 00 20
?signed_positive@@YAIH@Z    @0x30  7d 63 00 d0 7d 6a 18 78 55 43 0f fe 4e 80 00 20
?unsigned_ordered@@YAII@Z   @0x40  21 63 00 07 7d 2a 51 10 55 23 07 fe 4e 80 00 20
?signed_ordered@@YAIH@Z     @0x50  39 60 00 07 7d 43 58 10 7c 69 5a 38 55 28 0f fe
                                   7c e8 01 94 54 e3 07 fe 4e 80 00 20
```

### 3.1 `zero_test` — `x != 0` (2 instructions)

```
3163ffff  addic  r11,r3,-1      ; r11 = x-1 ; CA = carry out of (x + 0xFFFFFFFF) = (x != 0)
7c6b1910  subfe  r3,r11,r3      ; r3 = ~r11 + r3 + CA = -x + x + CA = CA
4e800020  blr
```
`~(x-1) == -x`, so the two register terms cancel and `r3` is exactly the carry.
Identical bytes for `int` (`?s_ne0`) and for unsigned `x > 0` (`?u_gt0`).

### 3.2 `equality_nonzero` — `x == 1` (3 instructions)

```
3963ffff  addi   r11,r3,-1      ; t = x - 1     (plain addi: no carry needed)
7d6a0034  cntlzw r10,r11        ; 32 iff t == 0, else < 32
5543dffe  rlwinm r3,r10,27,31,31; rotl(t,27) & 1 = bit 5 of the count = (t == 0)
4e800020  blr
```
`rlwinm rA,rS,27,31,31` extracts source bit 5 (value 32) — set only for a
`cntlzw` result of exactly 32.

### 3.3 `inequality_nonzero` — `x != 1` (3 instructions)

```
3963ffff  addi   r11,r3,-1      ; t = x - 1
314bffff  addic  r10,r11,-1     ; CA = (t != 0)
7c6a5910  subfe  r3,r10,r11     ; r3 = ~r10 + r11 + CA = CA
4e800020  blr
```
The `!= 0` spine of §3.1 applied to the difference.

### 3.4 `signed_positive` — signed `x > 0` (3 instructions)

```
7d6300d0  neg    r11,r3         ; -x
7d6a1878  andc   r10,r11,r3     ; (-x) & ~x  ; bit31 set iff x > 0
55430ffe  rlwinm r3,r10,1,31,31 ; = srwi r3,r10,31
4e800020  blr
```
This is a **`k == 0` fold**, not the general signed-`>` spine (§3.6). Emitting
the general spine here would be a wrong-length, wrong-bytes mis-emit.

### 3.5 `unsigned_ordered` — unsigned `x > 7` (3 instructions)

```
21630007  subfic r11,r3,7       ; r11 = 7 - x ; CA = (x <= 7)
7d2a5110  subfe  r9,r10,r10     ; r9 = ~r10 + r10 + CA = CA - 1
552307fe  rlwinm r3,r9,0,31,31  ; = clrlwi r3,r9,31 ; low bit = !CA = (x > 7)
4e800020  blr
```
**`subfe r9,r10,r10` reads `r10`, which is never defined.** The two register
terms cancel (`~r + r == -1`), so the value is a don't-care — but the *register
number is not*: r10 is byte-visible in the encoding and the port must reproduce
that exact allocation (§6).

### 3.6 `signed_ordered` — signed `x > 7` (6 instructions)

```
39600007  li     r11,7          ; materialize the literal (q)
7d435810  subfc  r10,r3,r11     ; r10 = q - p ; CA = (q >= p) UNSIGNED   [r10 dead]
7c695a38  eqv    r9,r3,r11      ; ~(p ^ q)  ; bit31 = sign(p) == sign(q)
55280ffe  rlwinm r8,r9,1,31,31  ; = srwi r8,r9,31  ; 1 iff same sign
7ce80194  addze  r7,r8          ; r7 = samesign + CA
54e307fe  rlwinm r3,r7,0,31,31  ; = clrlwi r3,r7,31 ; low bit
4e800020  blr
```
with `p = x` (the greater side), `q = 7`. Correctness of the low-bit trick:

| case | samesign | CA | sum | low bit | `p > q`? |
|---|---|---|---|---|---|
| same sign, `p > q` | 1 | 0 | 1 | 1 | yes |
| same sign, `p <= q` | 1 | 1 | 2 | **0** | no |
| `p >= 0 > q` | 0 | 1 | 1 | 1 | yes |
| `q >= 0 > p` | 0 | 0 | 0 | 0 | no |

The `clrlwi` exists solely to kill the `2` in row 2.

---

## 4. Idiom catalogue (all relations, both signednesses)

Notation: the compare is normalised to **`p > q`**, **`p >= q`**, `p == q`,
`p != q`; `a < b` is emitted as `b > a` and `a <= b` as `b >= a` — verified, the
`<`/`>` pair and the `<=`/`>=` pair emit the *same* spine with the operand roles
swapped (`?s_lt` vs `?s_gt`, `?sc_le` vs `?sc_ge`, and `?constleft`
(`7 < a`) is **byte-identical** to `?i_gt7` (`a > 7`)).

### 4.1 `==` / `!=` (signedness-independent)

Form the difference `D`, then apply a fixed tail. Byte-identical for `int` and
`unsigned int` (`cmp_signed_vv` vs `cmp_unsigned_vv`, offsets `0x60`/`0x70`).

| operand form | difference `D` |
|---|---|
| `p` vs literal `0` | `D = p` (no instruction) |
| `p` vs literal `k`, k fits i16 | `addi t,p,-k` |
| `p` vs wide literal `K` | `lis t,hi ; ori t,t,lo ; subf t2,p,t` |
| `p` vs register `q` | `subf t,p,q` (= `q - p`; sign irrelevant) |

tails:
```
==   cntlzw t2,D ; rlwinm r3,t2,27,31,31
!=   addic  t2,D,-1 ; subfe r3,t2,D
```
`!= 0` collapses to the 2-instruction §3.1; `== 0` collapses to
`cntlzw r11,r3 ; rlwinm r3,r11,27,31,31` (`?s_eq0`, `?u_eq0`).
`return !a;` (`0x1A`) emits the **same bytes** as `a == 0` — same code, different
IL token.

### 4.2 Unsigned `>` / `<` (3 instructions + literal materialization)

```
[subfic t,p,k]  or  [li/lis+ori tq,k ; subfc t,p,tq]  or  [subfc t,p,q]
subfe  u,v,v            ; v undefined; u = CA - 1
rlwinm r3,u,0,31,31     ; clrlwi r3,u,31
```
`subfic t,p,k` is used **only when the literal is the minuend** (`p > k` → needs
`k - p`). For `p < k` ≡ `k > p` the literal is the subtrahend, and c2 emits
`li` + `subfc` instead (`?uc_lt`: `li r11,7 ; subfc r10,r11,r3`). It does **not**
use `addic t,p,-k`, even though that would also produce the right carry.

### 4.3 Unsigned `>=` / `<=` (3 instructions + materialization)

```
li     t,-1
[subfic u,q,p]  or  [subfc u,q,p]        ; u = p - q ; CA = (p >= q)
subfze r3,t                              ; r3 = ~(-1) + CA = CA
```
`?u_ge`, `?u_le`, `?uc_le` (`subfic`), `?uc_ge` (`li`+`subfc`, and note the
allocator **reuses r11** as the `subfc` destination there).

### 4.4 Signed `>` / `<` — the 5-instruction spine (§3.6)

```
[materialize q]
subfc  t1,p,q      ; CA = (q >= p) unsigned      [t1 dead]
eqv    t2,p,q      ; rS = p, rB = q
rlwinm t3,t2,1,31,31   (srwi 31)
addze  t4,t3
rlwinm r3,t4,0,31,31   (clrlwi 31)
```
Operand roles verified across four independent captures (`?s_gt`, `?s_lt`,
`?sc_lt`, `?signed_ordered`): `subfc` is always `(rA=p, rB=q)` and `eqv` always
`(rS=p, rB=q)`. **`eqv` is commutative, `subfc` is not** — swapping `subfc`'s
operands inverts the relation and is a fuzzy-invisible corruption of exactly the
class `docs/CODEGEN_PPC_MVP.md`'s hazard list warns about.

### 4.5 Signed `>=` / `<=` — the 4-instruction spine

```
[materialize q]
srawi  t1,p,31        ; -1 if p<0 else 0
rlwinm t2,q,1,31,31   ; srwi 31 : sign bit of q
subfc  t3,q,p         ; p - q ; CA = (p >= q) unsigned    [t3 dead]
adde   r3,t2,t1       ; signbit(q) + (p<0 ? -1 : 0) + CA  ∈ {0,1} exactly
```
No masking instruction is needed — the sum is always 0 or 1.
`?s_ge`, `?s_le`, `?sc_ge` all emit `srawi(p)` before `srwi(q)`.

> **UNRESOLVED:** `?sc_le` (`a <= 7`, i.e. `p = 7`, `q = a`) emits them in the
> *other* order — `srwi(q)` at r10 then `srawi(p)` at r9 — and the `adde`
> operands follow. All four captures are self-consistent under
> "temps are numbered in emission order"; what varies is the emission order
> itself, and one instance is not enough to state the rule. Only `<=`/`>=` with
> a **literal lhs** deviates. Not needed for W6 (the fixture has no `<=`/`>=`).

### 4.6 Comparison against literal 0 — mandatory folds

`k == 0` is **not** a special case of the general spine; c2 folds it, sometimes
to a different instruction sequence and sometimes to a constant. A W6 codegen
that implements only the general spines would mis-emit every one of these — and
two of the six fixture functions are in this table.

| relation | signed (`int a`) | unsigned (`unsigned a`) |
|---|---|---|
| `a < 0` | `srwi r3,r3,31` (1 instr) | **`li r3,0`** (folded false) |
| `a <= 0` | `neg r11,r3 ; orc r10,r3,r11 ; srwi r3,r10,31` | same as `== 0` |
| `a > 0` | `neg r11,r3 ; andc r10,r11,r3 ; srwi r3,r10,31` | same as `!= 0` |
| `a >= 0` | `srwi r11,r3,31 ; xori r3,r11,1` | **`li r3,1`** (folded true) |
| `a == 0` | `cntlzw r11,r3 ; rlwinm r3,r11,27,31,31` | identical |
| `a != 0` | `addic r11,r3,-1 ; subfe r3,r11,r3` | identical |

Crucially the **IL is unfolded** — `?u_lt0` still carries
`LOAD uint · LIT uint 0 · 0x22` and only the obj shows `li r3,0`. **The folding
happens inside c2**, so it is the port's job, not something the front end hands
over pre-simplified.

---

## 5. CONST / DERIVED byte classification

For the W6 fixture class — *a single-parameter leaf whose whole body is one
comparison against a literal* — the emitted `.text` has only two degrees of
freedom: **the parameter's ABI register** and **the literal's immediate field**.
Everything else is fixed by (relation, signedness, whether `k == 0`).

Field templates (`P` = the parameter's register, r3 for a first parameter;
`?third_gt7` confirms a third parameter yields exactly the same words with
`P = r5`):

| word | instruction | CONST fields | DERIVED fields |
|---|---|---|---|
| `0x3160_0000 \| (P<<16) \| 0xFFFF` | `addic r11,P,-1` | op=12, rD=11, SI=-1 | rA = **P** |
| `0x7C6B_0110 \| (P<<11)` | `subfe r3,r11,P` | op=31/XO=136, rD=3, rA=11 | rB = **P** |
| `0x3960_0000 \| (P<<16) \| (-k & 0xFFFF)` | `addi r11,P,-k` | op=14, rD=11 | rA = **P**, SI = **−k** |
| `0x7D6A_0034` | `cntlzw r10,r11` | all | — |
| `0x5543_DFFE` | `rlwinm r3,r10,27,31,31` | all | — |
| `0x314B_FFFF` | `addic r10,r11,-1` | all | — |
| `0x7C6A_5910` | `subfe r3,r10,r11` | all | — |
| `0x7D60_00D0 \| (P<<16)` | `neg r11,P` | op/XO=104, rD=11 | rA = **P** |
| `0x7D6A_1878` with rB=P | `andc r10,r11,P` | rS=11, rA=10, XO=60 | rB = **P** |
| `0x5543_0FFE` | `rlwinm r3,r10,1,31,31` | all | — |
| `0x2160_0000 \| (P<<16) \| k` | `subfic r11,P,k` | op=8, rD=11 | rA = **P**, SI = **k** |
| `0x7D2A_5110` | `subfe r9,r10,r10` | all (incl. the don't-care r10) | — |
| `0x5523_07FE` | `rlwinm r3,r9,0,31,31` | all | — |
| `0x3960_0000 \| k` | `li r11,k` | op=14, rD=11, rA=0 | SI = **k** |
| `0x7D40_5810 \| (P<<16)` | `subfc r10,P,r11` | rD=10, rB=11, XO=8 | rA = **P** |
| `0x7C60_5A38 \| (P<<21)` | `eqv r9,P,r11` | rA=9, rB=11, XO=284 | rS = **P** |
| `0x5528_0FFE` `0x7CE8_0194` `0x54E3_07FE` | `srwi`/`addze`/`clrlwi` chain | all | — |
| `0x4E80_0020` | `blr` | all | — |

So, concretely, for the six fixture functions the `.text` is:

* `zero_test` — **all 12 bytes CONST** for "`p != 0`, p in r3".
* `equality_nonzero` — 16 bytes; DERIVED = the 2-byte `ff ff` (= `−k` as i16).
* `inequality_nonzero` — 16 bytes; DERIVED = the 2-byte `ff ff` (= `−k`).
* `signed_positive` — **all 16 bytes CONST** for "signed `p > 0`, p in r3".
* `unsigned_ordered` — 16 bytes; DERIVED = the 2-byte `00 07` (= `k`).
* `signed_ordered` — 28 bytes; DERIVED = the 2-byte `00 07` (= `k`).

Immediate width matters: `k` (and `−k`) must fit a signed 16-bit field, else the
literal materialization becomes `lis`+`ori` (`?u_gt_wide`, `?s_gt_wide`,
`?u_eqwide`: `lis r11,1 ; ori r11,r11,0x1170` for 70000) — the same spine, one
word longer, and the two-word materialization consumes **one** temp slot.

---

## 6. Register model

Parameters occupy the ABI argument registers by position (r3, r4, r5, …); the
result is always r3. Temporaries are drawn **descending from r11** (r12 is
reserved) in **emission order**, one physical register per temp, with **no reuse
inside any of the six fixture functions**:

| function | r11 | r10 | r9 | r8 | r7 | → r3 |
|---|---|---|---|---|---|---|
| `zero_test` | `addic` | — | — | — | — | `subfe` |
| `equality_nonzero` | `addi` | `cntlzw` | — | — | — | `rlwinm` |
| `inequality_nonzero` | `addi` | `addic` | — | — | — | `subfe` |
| `signed_positive` | `neg` | `andc` | — | — | — | `rlwinm` |
| `unsigned_ordered` | `subfic` | *don't-care operand* | `subfe` | — | — | `rlwinm` |
| `signed_ordered` | `li` | `subfc` (dead) | `eqv` | `srwi` | `addze` | `clrlwi` |

Two kinds of slot are consumed by values that are never read:

* the `subfe u,v,v` **don't-care source** `v` — undefined on entry, and its two
  register terms cancel (`unsigned_ordered`: r10);
* a `subfc`/`subfic` **destination** when only its carry-out is used
  (`unsigned_ordered`: r11; `signed_ordered`: r10 — both dead).

Both still occupy a register number that is visible in the encoding, so the port
must allocate them as real temps.

Outside the fixture class the allocator is demonstrably richer than a descending
counter and is **not** characterized here:

* `?uc_ge` reuses r11 as the `subfc` destination once its `li r11,7` value dies.
* `?u_le` numbers the `subfc` r11 and the `li -1` r10, then **emits the `li`
  first** — so numbering order ≠ emission order; there is a scheduling pass.
* `?twocmp` (`(a>7)+(b>7)`) **interleaves** the two comparison spines
  (`cmp_neg.obj` `0x38..0x6c`); it is not a concatenation of two single-compare
  functions.

W6 does not need any of this, but a widening past the fixture class does.

---

## 7. What `parse_expr` must accept

Today `parse_expr` (`crates/c2-il/src/func.rs`) hard-codes `INT_TYPE`
(`86 41 74`) in the LOAD and LITERAL arms and accepts only `02`/`03`/`04`.
`eat_return_plumbing` hard-codes `41 86 41 74`. Five changes:

1. **Typed operand stack.** LOAD (`B9 <tok> <T>`) and LITERAL
   (`33 <T> <varint>`) must accept `T ∈ {86 41 74 (int), 86 42 75 (unsigned)}`
   and *record which*. The blocking census bucket that the W6 fixture reports
   today — `expr-load-type-864275`, 4 of 6 functions — is exactly this.
   The stack element needs a type tag because the comparison opcode does not
   carry one.
2. **Six comparison opcodes** `0x1F`(EQ) `0x20`(NE) `0x21`(LE) `0x22`(LT)
   `0x23`(GE) `0x24`(GT), each popping rhs then lhs and pushing a value of type
   **bool** (`82 12 30`). Reject when the two operand types differ (never
   observed — c1xx always converts first — so this is a cheap fail-closed
   assertion, not a supported case).
3. **The bool convert** `2C <T> 00`, accepted **only** when the popped operand
   is a comparison result and `T ∈ {int, unsigned}`. This is the load-bearing
   restriction: `?cmpchar`/`?cmpshort` use the identical token over a *narrow
   integer LOAD*, where it is a real `extsb`/`extsh` (word `7c6a0774` /
   `7c6a0734`), and `?mixed` uses it over an *int LOAD*. A blanket "`2C` is
   free" rule silently drops those sign-extensions.
4. **Result type** `41 86 42 75` (unsigned) must be accepted by
   `eat_return_plumbing` alongside `41 86 41 74`. (`41 82 12 30` — a `bool`
   return — is a further, separate widening; `?retbool_gt7` shows it also has
   *no* `2C` convert, so it is a distinct shape.)
5. **A new `BodyShape`.** The existing `IlOp` enum cannot represent a compare,
   and `select_text`'s operand-stack lowering cannot emit a multi-instruction
   spine into descending temps. The honest shape is narrow, e.g.
   `CompareLeaf { param: u16, rel: Rel, signed: bool, k: i32 }` covering exactly
   `return <formal> <rel> <literal>;`, which is precisely the fixture. The whole
   body must still parse to the segment end — no local pattern match around the
   comparison byte (the W4b2-v doctrine).

### Codegen

`c2_core::codegen` needs one new entry point returning the spine for
`(rel, signed, k)`, plus the **new encoders**: `addic`, `subfic`, `subfc`,
`subfe`, `subfze`, `adde`, `addze`, `neg`, `andc`, `orc`, `eqv`, `cntlzw`,
`srawi`, `rlwinm`, `xori`. All are verified bit-exact in this document. Per the
repo's opt-in-encoder rule, `subfc`/`subfe`/`subf`/`srawi` are **non-commutative
and operand-order-critical** and each deserves the same treatment `encode_subf`
already gets. `eqv`, `andc`, `orc` are effectively symmetric here but c2's
emitted rS/rB order is recorded above and should be reproduced rather than
chosen.

The `k == 0` folds (§4.6) are **not optional**: `signed_positive` (`x > 0`) and
`zero_test` (`x != 0`) are two of the six fixture functions, and the general
spine produces different, longer, wrong bytes for them. Structure them as the
first dispatch, exactly like the `g(a)+0` identity fold in W4b2-vi.

No new obj machinery: 5 sections, no relocations, no `.pdata`, 8-byte function
alignment — all already implemented.

### Fail-closed NEGATIVE neighbours (must still be rejected)

Each is a live capture, with the byte or type that must block the parse:

| probe source | must block on | why a mis-emit would be silent |
|---|---|---|
| `return (a > 7) + 1;` | a `33 … 02` **after** the `2C` convert | reference emits `clrlwi r11,...` (not r3!) + `addi r3,r11,1` — the spine's *last* instruction changes target |
| `return (a>7) + (b>7);` | a second `B9`/compare after the convert | reference **interleaves** both spines; a concatenation would be wrong bytes |
| `if (a > 7) return 5; return 9;` | `53 53` at body start; branch token `38 <tok>`, `54 03` / `54 04` | reference switches family entirely: `cmpwi cr6,r3,7 ; li r3,5 ; bclr 12,25` |
| `return a > 7 && b > 7;` | `0x1C` | branchy, `cmpwi`+`bc` |
| `return a > 7 \|\| b > 7;` | `0x1B` | ditto |
| `return !a;` | `0x1A` | emits the same bytes as `a == 0`, but the token is unmodeled — accepting it by accident would be luck, not correctness |
| `return a > 7 ? 3 : 4;` | `0x43 42 00 00` | select |
| `char a` / `short a` | LOAD type `82 11 70` / `84 21 11` **and** the `2C 86 41 74 00` over a non-bool | a real `extsb`/`extsh` would be dropped |
| `long long a` | LOAD type `88 81 13` | reference uses `cmpdi cr6` + branch — a different family |
| `float a` | LOAD type `86 45 40`, literal `33 86 4a 40 <8B double>` | FP compare + branch |
| `int a > unsigned b` | `2C 86 42 75 00` over an **int LOAD** | the convert is a codegen no-op here, so this is the cheapest *next* widening — but under rule 3 above it is rejected today, which is correct until it is tested |
| `a > 70000` | wide varint `80 <LE32>` | needs the `lis`+`ori` materialization and the one-slot temp accounting; in-spine but untested |
| `a >= 7` / `a <= 7` | `0x23` / `0x21` | the §4.5 spine, whose **instruction order is UNRESOLVED** for a literal lhs — must stay out of class until pinned |

`c2rs census fixtures/cpp/il_bool_materialization.cpp` currently reports
`0/6 in class`, `4 × expr-load-type-864275` + `2 × expr-cmp-gt`. A correct W6
lands all six as one new in-class shape, and every row of the table above must
still report a blocking feature.

---

## 8. Summary of what is and is not established

**Established (byte evidence in this document):**
comparison opcodes `0x1F..0x24`; signedness carried by the operand type triple,
not the opcode; `unsigned int` = `86 42 75`; bool = `82 12 30`; the `2C` convert
and its codegen-no-op-only-over-a-bool restriction; the six materialization
spines and their operand roles; the `k == 0` folds; the register/temp model for
the fixture class; the `.text` layout and CONST/DERIVED split; the `4F 01 NN`
line-marker correction; the census label defect.

**Not established (explicitly open):**
the meaning of `0x25`; the srawi/srwi emission order in the signed `>=`/`<=`
spine when the lhs is a literal; the general register allocator (reuse under
liveness) and scheduler seen in `?u_le`, `?uc_ge`, `?twocmp`; the `.ex` float
literal trailer (`… 1c 40 04 00`); anything about comparisons inside larger
expressions or feeding control flow beyond "it is a different codegen family".
