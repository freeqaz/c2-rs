# `.ex` statement grammar

**Status: characterization (P2d). No Rust source was changed by this document.**

Every structural claim below cites bytes from a capture this document produced.
Claims resting on a **controlled fixture** (one source construct varied, the rest
held fixed) are marked **[CF]** and name the tracked fixture in `fixtures/cpp/`;
claims resting on the real `src/system/world/Dir.cpp` capture are marked
**[DIR]**. Claims resting on an untracked scratch probe are marked **[P]** and
name the probe (`work/stmt2/p/pN.cpp`, regenerable — see §14).

Unknowns are called out as unknowns. The port must fail closed on them.

Scope note: the **statement** layer is this document. The formals region, the
CALL header and the call-argument region are characterized in
`docs/IL_CALL_GRAMMAR.md` and are owned there; where this document has to quote
them it uses that document's corrected reading — `46 (2D <tok>)* 4C` formals then
`4F 11` body-start (so `4C 4F 11` is *not* an atomic marker), `BD <TYPE> <cc>
<varint fn-type-id>` for the CALL header, and `(expr 55 <TYPE>)* 4C` for the
argument region.

---

## 0. Summary — the grammar, and what it corrects

```text
body         := <body-start:4F 11> 53 item* 4F 12 47 54 <k> 54 <k>
                                       ^ the last two closes pop the two scopes
                                         that were already open before the body

item         := line-marker                                              (§3)
              | 53                     open a scope                       (§1)
              | 54 <k>                 close the innermost scope;
                                       k == scopes still open AFTER the pop (§1)
              | 29 <tok>               define label <tok>                 (§7)
              | 38 <tok>               branch if the popped value is FALSE (§7)
              | 39 <tok>               branch if the popped value is TRUE  (§7)
              | 3A <tok>               unconditional branch to label <tok> (§9)
              | 4B                     end of expression statement;
                                       DISCARDS any remaining value       (§2)
              | 3B <tok> / 3C <TYPE> <tok> / 3D <tok>   switch            (§11)
              | <one operand-stream token>              expressions

line-marker  := 4F 01 <varint>         source LINE number                (§3)
```

Deliberately flat: the statement layer is a *token stream*, not a nested syntax
tree, and a decoder consumes it as one loop over `item`. The statement forms are
then patterns over that stream:

```text
expr-stmt        := <operand stream>  4B
assign-stmt      := <lvalue> <value>  32 <TYPE>  4B                       (§4)
compound-assign  := 26 <tok> <value>  <op> <TYPE>  4B                     (§5)
return-stmt      := [ <value> 41 <TYPE> ]  3A <epilogue-label>            (§9)
                    — note: NO 4B
scope            := 53 item* 54 <k>                                       (§6)
```

The body's own `54 <k>` close and the epilogue's `29 <tok>` are ordinary `item`s
inside the loop, not part of a fixed tail pattern — §9 shows a line marker landing
between them.

| Prior model | Reality |
|---|---|
| `4F 01 NN` is "a per-statement sequence index c1xx emits in multi-function TUs" with a **fixed 1-byte** payload (`eat_opt_stmt_marker`) | It is the **source line number**, emitted on every line *change*, payload is `read_varint` (1 byte, or `80` + 4-byte LE i32). §3 |
| `54 03` = scope end, `54 04 29 <tok>` = label definition, `54 05` = goto-epilogue (`IL_CALL_GRAMMAR.md` §4.3) | `54 <k>` is **one** production — close the innermost scope — and `k` is the scope depth remaining after the pop. `54 05` in a plain nested `if` contains no goto at all. `29 <tok>` is a separate label-definition statement. §1 |
| `54 02 29 <tok>` is the fixed 3-byte "return" pattern (`eat_return_plumbing`) | Two separate productions, and a `4F 01 <varint>` line marker can sit **between** them. §9 |
| `3A <tok>` is the "assign to the return temp" of the return plumbing | `3A <tok>` is an **unconditional branch** to label `<tok>`. `return;` in a void function is `3A <lbl>` and nothing else. `break`, `continue` and `goto` are all the same opcode. §9 |
| body is one statement | body is a statement list with nested scopes and control flow; statements are simply **concatenated** with no separator beyond each statement's own `4B`. §2 |
| `0x0F`, `0x35`, `0x36` are unidentified (`IL_CALL_GRAMMAR.md` §7) | The **compound-assignment / inc-dec** family, `<op> <TYPE>`. Full table with a witness per operator in §5. |

---

## 1. The scope stack: `53` open, `54 <k>` close

`53` pushes a scope. `54 <k>` pops the innermost one, and

> **`k` equals the number of scopes still open after the pop.**

Two scopes are already open when the body starts (the `53 53` that precedes the
function's `26 <fn-tok>` symbol push — see the formals region in
`IL_CALL_GRAMMAR.md`), and the body's own `53` is the third. That is why the
function tail is `… 54 02 … 4F 12 47 54 01 54 00`: `54 02` closes the body
(2 remaining), `54 01` the function (1), `54 00` the module (0).

**[CF] `il_stmt_bare_block.cpp` — `void stmt_block_nest() { { { g(); } } }`**

```
4c 4f 11 53   4f 01 10   53 53   26 e3 09 bd 82 07 03 00 80 01 10 00 00 4c 4b
              54 04 54 03   4f 01 11   3a e7 09 54 02 29 e7 09   4f 12 47 54 01 54 00
```

Five scopes are open at the call — 2 before the body, the body itself, the outer
brace, the inner brace — and the closes run `04 03 02 01 00`. The single-brace
sibling `stmt_block_one` has one fewer `53` and its first close is `54 03`, not
`54 04`.

**[CF] `il_stmt_if.cpp` — the decisive pair.** Unbraced then-clause:

```
4c 4f 11 53 4f 01 0d 53 b9 e4 09 86 41 74 38 e7 09 53
   26 e3 09 bd 82 07 03 00 80 01 10 00 00 4c 4b
   4f 01 0e 54 04 29 e7 09 54 03 3a e6 09 54 02 29 e6 09 4f 12 47 54 01 54 00
```

Braced then-clause — identical source but for the braces:

```
4c 4f 11 53 4f 01 11 53 b9 e8 09 86 41 74 38 eb 09 53 53
   26 e3 09 bd 82 07 03 00 80 01 10 00 00 4c 4b
   54 05 4f 01 12 54 04 29 eb 09 54 03 3a ea 09 54 02 29 ea 09 4f 12 47 54 01 54 00
```

The brace adds exactly one `53` and one `54 05`, and the value `05` is exactly
one more than the `54 04` it now nests inside. Under the old "`54 <kind>`"
reading, `54 05` would mean *goto-epilogue* — but there is no jump anywhere in
`if (a) { g(); }`. The depth reading is the only one consistent with both bodies.

**[P] `p1.cpp` `void blk4() { { { { g(); } } } }`** pushes it to four braces:

```
4c 4f 11 53 53 53 53  26 e3 09 bd … 4c 4b  54 05 54 04 54 03  3a f5 09 54 02 29 f5 09
```

**[P] `p6.cpp` — 40 nested braces**, the widest test of the counter:

```
… 53 4f 01 2a 53 4f 01 2b  26 e3 09 bd … 4c 4b
   4f 01 2c 54 2a  4f 01 2d 54 29  4f 01 2e 54 28  …  4f 01 53 54 03  4f 01 54 3a e5 09 54 02 …
```

The first close is `54 2a` = 42 = the 41 open scopes after the body marker plus
the two before it, minus one. It then counts down by one per `}` without a gap.

**[DIR] validation.** A whole-body scanner that enforces `k == remaining depth`
at every single close ran over all 5239 Dir.cpp function bodies. It fired
**once** (§12.4). Member functions, whose pre-body region carries an extra
`b9 <this> <TYPE> 99 <TYPE> 00` between `26 <fn>` and `46` (**[P] `p8.cpp`**),
still start at depth 2.

**UNKNOWN:** whether `k` is a plain byte or a `read_varint`. Every value observed
is `< 0x80` (max `0x2A` at 40-deep nesting), and both readings agree there. A
decoder should track its own depth and *compare*, not decode — then the question
is moot and the comparison is a free integrity check.

---

## 2. The statement list: concatenation, and `4B`

Statements are **concatenated**. There is no separator: an expression statement
ends with its own `4B` and the next statement begins immediately.

**[CF] `il_stmt_seq.cpp`** — a statement-count ladder, all four functions on one
source line each so no line markers intrude (§3):

```
stmt_seq0()  4c 4f 11 53                                                    3a e5 09 54 02 29 e5 09 …
stmt_seq1()  4c 4f 11 53 26 e3 09 bd … 4c 4b                                3a e7 09 54 02 29 e7 09 …
stmt_seq2()  4c 4f 11 53 26 e3 09 bd … 4c 4b  26 e3 09 bd … 4c 4b           3a e9 09 54 02 29 e9 09 …
stmt_seq3()  4c 4f 11 53 26 e3 09 bd … 4c 4b  26 e3 09 bd … 4c 4b  26 …4c 4b 3a eb 09 54 02 29 eb 09 …
```

(`bd …` is the 10-byte CALL header `bd 82 07 03 00 80 01 10 00 00` in every
case.) The delta between consecutive bodies is exactly one 16-byte statement.
`stmt_seq2` is where the current parser blocks, reported as `assign-0x26`: after
the first `4C 4B` it expects the return plumbing's `3A` and finds `26`.

**`4B` ends the statement and discards whatever value is left.** Two witnesses
where a value definitely remains:

**[P] `p3.cpp` `void discard_int(int a) { gi(a); }`** — an *int*-returning call
used as a statement:

```
4c 4f 11 53  26 e4 09 bd 86 41 74 00 80 01 10 00 00  b9 e6 09 86 41 74 55 86 41 74  4c  4b
```

**[P] `p3.cpp` `void discard_expr(int a) { a + 1; }`** — a bare expression:

```
4c 4f 11 53  b9 e9 09 86 41 74 33 86 41 74 01 02  4b  3a eb 09 54 02 29 eb 09 …
```

So `4B` is not "store nothing"; it pops the expression stack to empty. Note the
void-call form `4C 4B` that the current parser matches as a unit is really the
argument-list terminator `4C` followed by the statement terminator `4B` — the
same two bytes appear with an int result in between them above.

`4B` is emitted for the **last** statement too (every body above). The `return`
statement is the exception: it has no `4B` (§9).

---

## 3. Line markers: `4F 01 <varint>` is the source LINE NUMBER

Not a statement counter, not a label. The payload is the 1-based source line,
encoded with `read_varint` (§1.2 of `IL_CALL_GRAMMAR.md`): one byte below `0x80`,
else `80` + a 4-byte little-endian i32. A marker is emitted whenever the line
*changes*, including for a source line that generates no code at all.

**[CF] `il_stmt_local_decl.cpp` — `stmt_decl_split`, source lines 13–17:**

```cpp
13  int stmt_decl_split(int a) {
14      int x;
15      x = a;
16      return x;
17  }
```
```
4f 01 0d  53 53 26 e4 09 46 2d e3 09 4c  4f 11 53
4f 01 0e  4f 01 0f  26 e6 09 b9 e3 09 86 41 74 32 86 41 74 4b
4f 01 10  b9 e6 09 86 41 74 41 86 41 74 3a e5 09
4f 01 11  54 02 29 e5 09  4f 12 47 54 01 54 00
```

`0d 0e 0f 10 11` = 13 14 15 16 17, exactly. **Two markers in a row** (`0e`, `0f`)
because line 14's `int x;` emits nothing — a bare declaration is free (§4). This
is the current parser's `body-0x4F` census bucket (**[DIR] 26,666 functions**):
`eat_opt_stmt_marker` consumes one marker and the next byte is another `4F`.

**[CF] `il_stmt_seq.cpp`** is the control: all its statements share their
function's line, and its bodies carry **no** internal marker at all.

**[P] `p5.cpp` — a function at source line 200**, forcing the escape:

```
4f 01 80 c8 00 00 00   53 53 26 e5 09 46 2d e4 09 4c 4f 11 53
4f 01 80 c9 00 00 00   53 b9 e4 09 86 41 74 38 e7 09 53 26 e3 09 bd … 4c 4b
4f 01 80 ca 00 00 00   54 04 29 e7 09 54 03  26 e3 09 bd … 4c 4b
4f 01 80 cb 00 00 00   3a e6 09 54 02 29 e6 09 4f 12 47 54 01 54 00
```

`0xC8 0xC9 0xCA 0xCB` = 200 201 202 203 = the `{` line, the `if`, the `g();`, the
`}`. **The current fixed-1-byte read is a live bug for any TU whose functions
live past line 127**, i.e. essentially every real one: it is the `body-0xAD`
(15,480) / `body-0xB3` (15,782) census family **[DIR]**, which is the parser
standing on the second byte of a `80`-escaped line number. Repairing just this
one field moves the Dir.cpp whole-body decode reach from 2499 to 2792 bodies
(§13).

Markers appear **between** statements and also *inside* the function tail, in at
least three distinct positions (§9), so they must be consumed by a loop at every
statement boundary rather than matched at fixed offsets.

---

## 4. Declaration and assignment

```text
assign-stmt := <lvalue> <value-expr> 32 <TYPE> 4B
lvalue      := 26 <tok>                          a named symbol (§4.1)
             | <pointer-valued expr>             e.g. B9 <tok> <ptr-TYPE> (§4.3)
```

`32 <TYPE>` pops the value and the lvalue and **pushes the stored value back**
(§4.2). `<TYPE>` is the destination's declared type; it never converts (§4.4).

A **bare declaration emits nothing** and an **initialized declaration is exactly
an assignment statement**. **[CF] `il_stmt_local_decl.cpp`**, the two bodies
side by side (`stmt_decl_split` above vs `stmt_decl_init`):

```
split  4f 01 0e 4f 01 0f  26 e6 09  b9 e3 09 86 41 74  32 86 41 74  4b
init   4f 01 14           26 ea 09  b9 e7 09 86 41 74  32 86 41 74  4b
```

Byte-identical after the line markers. `stmt_decl_two` is the same statement
twice with the second reading the first's token.

### 4.1 The destination push is `26 <tok>` for every symbol class

**[CF] `il_stmt_param_assign.cpp`.** `a` is the *formal* (`46 2d e3 09 4c` in the
header, so token `0x09E3`) and it is pushed as a destination with the same `26`
a local or a global gets:

```
stmt_param_assign      a = a + 1;   26 e3 09  b9 e3 09 86 41 74 33 86 41 74 01 02  32 86 41 74 4b
stmt_local_assign      x = a + 1;   26 e9 09  b9 e6 09 86 41 74 33 86 41 74 01 02  32 86 41 74 4b
stmt_param_assign_lit  a = 7;       26 ea 09  33 86 41 74 07                       32 86 41 74 4b
```

The two assignment statements differ in **one token** and nothing else. The
`26 <tok>` symbol push is therefore class-agnostic; the same opcode names a
callee (`IL_CALL_GRAMMAR.md` §3.1), a global, a local and a parameter, and only
what consumes it (`BD` vs `32`) says which role it is in.

### 4.2 `32 <TYPE>` yields the stored value

**[P] `p7.cpp` `int v_chain(int a){ int x,y; y = x = a; return y; }`**:

```
4c 4f 11 53  26 f1 09  26 f0 09  b9 ed 09 86 41 74  32 86 41 74  32 86 41 74  4b  …
```

Both destinations are pushed outermost-first, then the value, then two stores.
The inner `32` must leave a value for the outer one. So `32` is an expression
operator, not a statement terminator — `4B` is (§2).

### 4.3 A store through a pointer has no `26` and no deref opcode

**[P] `p7.cpp` `void st_deref(int *p) { *p = 3; }`**:

```
4c 4f 11 53  b9 fa 09 86 43 f4 08  33 86 41 74 03  32 86 41 74  4b  3a fc 09 54 02 29 fc 09 …
```

The lvalue is the loaded pointer itself. Contrast the *read* `return *p;`, which
needs the explicit indirect load `30 <TYPE>`:

```
4c 4f 11 53  b9 fd 09 86 43 f4 08  30 86 41 74  41 86 41 74  3a ff 09 …
```

### 4.4 A converting assignment carries an explicit `2C`

**[P] `p7.cpp` `int cv_narrow(int a) { char c; c = a; return c; }`**:

```
4c 4f 11 53  26 f5 09  b9 f2 09 86 41 74  2c 82 11 70 00  32 82 11 70  4b
             b9 f5 09 82 11 70  2c 86 41 74 00  41 86 41 74  3a f4 09 …
```

`int` → `char` is `2C <char> 00` *before* the store, and reading `c` back widens
with `2C <int> 00`. **`32 <TYPE>` never converts** — the type is always the
destination's own and any conversion is a visible `2C`. That is a fail-closed
friendly fact: a decoder can accept `32 <TYPE>` structurally and still refuse the
conversion it does not model, because the conversion is a separate token.

---

## 5. Compound assignment is NOT desugared

There is a dedicated read-modify-write opcode family: `<op> <TYPE>`, replacing
the `32 <TYPE>` of a plain assignment, with the destination pushed once and
**not** re-loaded.

**[CF] `il_stmt_compound_assign.cpp` — the decisive pair in one TU:**

```
stmt_cadd            x += 3;      26 e6 09  33 86 41 74 03                        0f 86 41 74  4b
stmt_cadd_expanded   x = x + 3;   26 ea 09  b9 ea 09 86 41 74  33 86 41 74 03 02  32 86 41 74  4b
```

They are *not* byte-identical: the compound form is 6 bytes shorter and has no
`B9` re-load of `x`. Same TU, same types, same literal.

**[P] `p2.cpp` — one function per operator**, each `26 <x> 33 86 41 74 03 <OP>
86 41 74 4b`, so the opcode byte is the only thing that moves:

| source | opcode | witness (the `<OP>` byte in `… 33 86 41 74 03 <OP> 86 41 74 4b`) |
|---|---|---|
| `x += 3`  | `0F` | `26 e6 09 33 86 41 74 03 0f 86 41 74 4b` |
| `x -= 3`  | `10` | `26 ea 09 33 86 41 74 03 10 86 41 74 4b` |
| `x *= 3`  | `11` | `26 ee 09 33 86 41 74 03 11 86 41 74 4b` |
| `x /= 3`  | `12` | `26 f2 09 33 86 41 74 03 12 86 41 74 4b` |
| `x %= 3`  | `13` | `26 f6 09 33 86 41 74 03 13 86 41 74 4b` |
| `x <<= 3` | `15` | `26 fa 09 33 86 41 74 03 15 86 41 74 4b` |
| `x >>= 3` | `16` | `26 fe 09 33 86 41 74 03 16 86 41 74 4b` |
| `x &= 3`  | `17` | `26 02 0a 33 86 41 74 03 17 86 41 74 4b` |
| `x ^= 3`  | `18` | `26 0a 0a 33 86 41 74 03 18 86 41 74 4b` |
| `x \|= 3` | `19` | `26 06 0a 33 86 41 74 03 19 86 41 74 4b` |
| `++x`     | `0F` | `26 0e 0a 33 86 41 74 01 0f 86 41 74 4b` (= `x += 1`) |
| `--x`     | `10` | `26 16 0a 33 86 41 74 01 10 86 41 74 4b` (= `x -= 1`) |
| `x++`     | `35` | `26 12 0a 33 86 41 74 01 35 86 41 74 4b` |
| `x--`     | `36` | `26 08 0a 33 86 41 74 01 36 86 41 74 4b` (**[P] `p4.cpp`**) |

Three things here would be wrong if guessed, so note them:

* **`0x14` is unobserved.** There is no C operator between `%=` and `<<=`. Do
  not fill the gap.
* **The compound table's operand order differs from the plain one.** Verified
  separately in **[P] `p4.cpp`** (`return a OP b;`, one function per operator):
  plain is `0B` `&`, `0C` `|`, `0D` `^`; compound is `17` `&=`, `18` `^=`,
  `19` `|=`. `|` and `^` are **swapped** between the two tables. There is no
  constant offset (`&`: +0x0C, `^`: +0x0B, `|`: +0x0D) and no ordering rule to
  extrapolate from — every entry above has its own witness and nothing else
  should be assumed.
* **Prefix and postfix `++` are different opcodes even as a statement**, where
  the value is discarded and they are semantically identical. `++x` folds to
  `+= 1` (`0F`), `x++` keeps `35`. The front end does not normalize this, so a
  decoder must handle both.

`p4.cpp` also pins two plain-table entries not previously recorded: `a % b` is
`06` and `~a` is `0E` (`b9 02 0a 86 41 74 0e 41 86 41 74 …`, unary, no operand).

**These opcodes yield a value.** **[P] `p7.cpp` `int y = (x += 3);`**:

```
26 e6 09 b9 e3 09 86 41 74 32 86 41 74 4b
26 e7 09  26 e6 09 33 86 41 74 03 0f 86 41 74  32 86 41 74  4b
```

The `0F` result is consumed by the outer `32`. Same for `35` in `int y = x++;`.
So `0F`…`19`/`35`/`36` are expression operators of the same class as `32`, not
statement forms.

---

## 6. Bare blocks

`{ … }` is `53 stmt* 54 <k>` and nothing else. **[CF] `il_stmt_bare_block.cpp`:**

```
stmt_block_one    4c 4f 11 53 4f 01 0c  53  26 e3 09 bd … 4c 4b  54 03  4f 01 0d  3a e5 09 54 02 …
stmt_block_empty  4c 4f 11 53 4f 01 19  53                       54 03  4f 01 1a  3a eb 09 54 02 …
stmt_block_two    4c 4f 11 53 4f 01 14  53 26 …4c 4b 54 03  4f 01 15  53 26 …4c 4b 54 03  4f 01 16 3a e9 09 …
```

`stmt_block_empty` shows an empty scope is `53 54 03` — the block is not elided.
`stmt_block_two`'s two *sequential* blocks both close with `54 03`, which is what
separates the depth reading from a "lexical block index" reading: an index would
give `03` then `04`.

`stmt_block_local` shows a block's locals are ordinary tokens with no
scope-entry/exit bookkeeping in `.ex`:

```
4c 4f 11 53 4f 01 1d  53  26 ef 09 b9 ec 09 86 41 74 32 86 41 74 4b
                          26 ec 09 b9 ef 09 86 41 74 32 86 41 74 4b  54 03
4f 01 1e  b9 ec 09 86 41 74 41 86 41 74 3a ee 09 …
```

---

## 7. `if`, `if`/`else`, and conditions

```text
if-stmt   := 53  <cond-expr>  38 <Lelse>   53 <then> 54 <k>
                 [ 3A <Ljoin>  29 <Lelse>  53 <else> 54 <k>  29 <Ljoin> ]
             54 <k>
```

with `29 <Lelse>` placed directly if there is no else.

**[CF] `il_stmt_if.cpp` `void stmt_if_nobrace(int a) { if (a) g(); }`** (bytes in
§1). **[CF] `il_stmt_if_else.cpp` `void stmt_if_else(int a) { if (a) g(); else
h(); }`**:

```
4c 4f 11 53 4f 01 0d 53
  b9 e5 09 86 41 74             load a
  38 e8 09                      branch-if-FALSE -> 0x09E8 (else entry)
  53 26 e3 09 bd … 4c 4b        then: g();
  4f 01 0e 54 04                close the then scope
  3a e9 09                      jump over the else -> 0x09E9 (join)
  29 e8 09                      define 0x09E8
  53 26 e4 09 bd … 4c 4b        else: h();
  54 04                         close the else scope
  29 e9 09                      define 0x09E9
54 03  4f 01 0f  3a e7 09 54 02 29 e7 09 4f 12 47 54 01 54 00
```

Token allocation in that body: formal `a` = `0x09E5`, the function itself
`0x09E6`, the epilogue label `0x09E7`, then the two `if` labels `0x09E8`,
`0x09E9`. Note the jump is emitted **after** the then-clause's `54 04`, in the
`if` statement's own scope.

`38 <tok>` = branch-if-false, `39 <tok>` = branch-if-true. Both consume the
condition value; there is no separate comparison-materialization step:

**[CF] `il_stmt_if.cpp` `if (a > 0) g();`** feeds the relational straight into
the branch, with **no `2C` convert** (unlike the W6 comparison *leaf*, which
converts bool → int because it returns the value):

```
53  b9 ec 09 86 41 74  33 86 41 74 00  24  38 ef 09  53 26 e3 09 bd … 4c 4b …
```

**Short-circuit `&&`/`||` and `!` are lowered to branches by the front end** —
opcodes `0x1A` (`!`), `0x1B` (`||`), `0x1C` (`&&`) do **not** appear in a
condition. **[P] `p3.cpp`:**

```
if (a || b) gv();
  53  b9 ec 09 86 41 74  39 f1 09      brTRUE  -> 0x09F1
      b9 ed 09 86 41 74  38 f0 09      brFALSE -> 0x09F0 (skip)
      29 f1 09                          0x09F1:
      53 26 e5 09 bd … 4c 4b  54 04  29 f0 09  54 03 …

if (a && b) gv();
  53  b9 f2 09 86 41 74  38 f6 09      brFALSE -> 0x09F6 (skip)
      b9 f3 09 86 41 74  38 f6 09      brFALSE -> the same label
      53 26 e5 09 bd … 4c 4b  54 04  29 f6 09  54 03 …

if (!a) gv();
  53  b9 f7 09 86 41 74  39 fa 09      the `!` becomes the opposite branch sense
      53 26 e5 09 bd … 4c 4b  54 04  29 fa 09  54 03 …
```

**Scope asymmetry, observed and unexplained.** An `if` clause gets its own `53`
**even unbraced** (both bodies above), while a *loop* body does not (§8). I have
no explanation for this and it is not needed for decoding — a decoder tracks
`53`/`54 <k>` generically and never predicts the count. It matters only if
someone tries to reconstruct source structure from scope counts, which they
should not.

---

## 8. `while`, `for`, `do`/`while`, `break`, `continue`

### 8.1 `while` — test at the top, one backward jump

**[CF] `il_stmt_while.cpp` `void stmt_while_call(int a) { while (a) { g(); a = a - 1; } }`**:

```
4c 4f 11 53 4f 01 0d 53
  29 e8 09                     TOP:
  b9 e4 09 86 41 74  38 e9 09  brFALSE -> EXIT (0x09E9)
  53 4f 01 0e  26 e3 09 bd … 4c 4b
     4f 01 0f  26 e4 09 b9 e4 09 86 41 74 33 86 41 74 01 03 32 86 41 74 4b
     4f 01 10
  54 04                        close the braced body
  3a e8 09                     jump TOP        (a BACKWARD jump — same opcode)
  29 e9 09                     EXIT:
54 03  4f 01 11  3a e6 09 54 02 29 e6 09 …
```

`3A` carries no direction: the target is a label token and forward/backward is
decided by where `29 <tok>` happens to be. A decoder therefore cannot know a
label's position without a two-pass scan or a fixup list.

**[P] `p1.cpp` `while (a) a = a - 1;`** — unbraced, and the body statement has
**no** scope of its own:

```
4c 4f 11 53 53  29 e8 09  b9 e4 09 86 41 74 38 e9 09
   26 e4 09 b9 e4 09 86 41 74 33 86 41 74 01 03 32 86 41 74 4b
   3a e8 09  29 e9 09  54 03  3a e6 09 54 02 29 e6 09 …
```

### 8.2 `for` — the increment is rotated ABOVE the condition

**[CF] `il_stmt_for.cpp` `void stmt_for_brace(int n) { for (int i = 0; i < n; i = i + 1) { g(); } }`**:

```
4c 4f 11 53 4f 01 0d 53
  26 e7 09 33 86 41 74 00 32 86 41 74 4b          i = 0                 (init)
  3a e8 09                                        jump COND
  29 e9 09                                        INCR:
  26 e7 09 b9 e7 09 86 41 74 33 86 41 74 01 02 32 86 41 74 4b   i = i+1 (increment)
  29 e8 09                                        COND:
  b9 e7 09 86 41 74  b9 e4 09 86 41 74  22        i < n                 (22 = cmp-lt)
  38 ea 09                                        brFALSE -> EXIT
  53 26 e3 09 bd … 4c 4b  54 04                   body
  3a e9 09                                        jump INCR
  29 ea 09                                        EXIT:
54 03  3a e6 09 54 02 4f 01 0e 29 e6 09 4f 12 47 54 01 54 00
```

So the emission order is **init · goto COND · INCR: incr · COND: cond · brfalse
EXIT · body · goto INCR · EXIT:** — the increment appears *before* the condition
in the byte stream even though it runs after it. A decoder that assumes source
order will mis-attribute the two assignment statements.

The for-init's `int i` is an ordinary local token (`0x09E7`, allocated right
after the epilogue label). `stmt_for_nobrace` is the same body with the inner
`53`/`54 04` pair removed.

### 8.3 `do`/`while` — the top label sits OUTSIDE the statement's scope

**[P] `p3.cpp` `void dowhile(int a) { do { gv(); } while (a); }`**:

```
4c 4f 11 53
  29 0a 0a                     TOP:  (at body depth, before any `53`)
  53 53  26 e5 09 bd … 4c 4b  54 04 54 03
  29 0b 0a                     CONTINUE:
  b9 07 0a 86 41 74  39 0a 0a  brTRUE -> TOP
  29 0c 0a                     EXIT:
3a 09 0a 54 02 29 09 0a 4f 12 47 54 01 54 00
```

This is the current parser's `body-0x29` census bucket (**[DIR] 14,594
functions**): the first byte after the body's `53` is a label definition. Labels
can be defined at a shallower depth than the branches that target them, so a
decoder must not scope its label table.

### 8.4 `break` and `continue` need no new opcode

**[P] `p3.cpp`:**

```
while (a) { gv(); break; }
  53 53  29 ff 09  b9 fb 09 86 41 74 38 00 0a
     53 26 e5 09 bd … 4c 4b   3a 00 0a          <- break  == jump EXIT
     54 04  3a ff 09  29 00 0a  54 03 …

while (a) { gv(); continue; }
  53 53  29 05 0a  b9 01 0a 86 41 74 38 06 0a
     53 26 e5 09 bd … 4c 4b   3a 05 0a          <- continue == jump TOP
     54 04  3a 05 0a  29 06 0a  54 03 …
```

`break`/`continue`/`goto`/`return` are all `3A <tok>`. That is a large amount of
real-code coverage for zero new statement vocabulary.

---

## 9. `return`, the epilogue label, and `41 <TYPE>`

```text
return-stmt := [ <value-expr> 41 <TYPE> ]  3A <epilogue-label>
fn-tail     := line-marker*  54 <k>  line-marker*  29 <epilogue-label>
               4F 12  47  54 <k-1>  54 <k-2>
```

Every function has one **epilogue label** token, allocated immediately after the
function's own symbol token. Every `return` is a jump to it; the label is defined
once, after the body scope closes.

**[CF] `il_stmt_early_return.cpp` `int stmt_early_int(int a) { if (a) return 1; return 2; }`** —
two returns, one label:

```
4c 4f 11 53 4f 01 0e 53
  b9 e4 09 86 41 74  38 e7 09
  53  33 86 41 74 01  41 86 41 74  3a e6 09       return 1;
  4f 01 0f 54 04  29 e7 09
54 03
  33 86 41 74 02  41 86 41 74  3a e6 09           return 2;
4f 01 10 54 02  29 e6 09  4f 12 47 54 01 54 00
```

Token map: `a` = `0x09E4`, the function = `0x09E5`, the epilogue label =
`0x09E6`, the `if` label = `0x09E7`.

**`3A` is a pure jump, not a store.** The decisive witness is a *void* early
return, which has no value at all: **[CF] `stmt_early_void`**

```
4c 4f 11 53 4f 01 19 53
  b9 ed 09 86 41 74  38 f0 09
  53  3a ef 09                                    return;   (nothing else)
  4f 01 1a 54 04  29 f0 09
54 03  26 e3 09 bd … 4c 4b
4f 01 1b  3a ef 09  54 02  29 ef 09 …
```

and the `if`/`else` join jump of §7 (`3a e9 09` with an empty expression stack,
targeting a label that is never anything but a branch target). The reading
"`3A <tok>` = assign to the return temp" cannot account for either.

**`41 <TYPE>` is the result-value annotation**, present exactly when the return
carries a value, immediately before the `3A`. Absent for `void`
(`stmt_seq0`'s whole body is `4c 4f 11 53 3a e5 09 54 02 29 e5 09 4f 12 …`).

**The `54 02 29 <tok>` of `eat_return_plumbing` is not a fixed pattern.** A line
marker can appear between the two productions — **[CF] `il_stmt_for.cpp`**, both
functions:

```
… 54 03  3a e6 09  54 02  4f 01 0e  29 e6 09  4f 12 47 54 01 54 00
```

against `il_stmt_local_decl.cpp`, where it lands one slot earlier:

```
… 41 86 41 74  3a e5 09  4f 01 11  54 02  29 e5 09  4f 12 47 54 01 54 00
```

and `il_stmt_if_else.cpp`, where it precedes the jump:

```
… 54 03  4f 01 0f  3a e7 09  54 02  29 e7 09  4f 12 47 54 01 54 00
```

Three distinct positions across three controlled fixtures. Markers must be
consumed by a loop at every boundary.

### 9.1 The result expression can come AFTER the epilogue label

`IL_CALL_GRAMMAR.md` §4.2 records this as an unexplained variant. It is
explained by the grammar above: a value-returning function with an **empty
statement list** emits the jump and label first and the result expression after,
because the epilogue is what reads the result.

**[DIR] fn48** (quoted in that document), now readable field by field:

```
53  3a 19 16  54 02  29 19 16  b9 18 16 a6 43 d4 21  41 a6 43 d4 21  4f 12 47 54 01 54 00
    jump epi  close   epi:      load                  result type
```

**[DIR] frequency: 215 of the 2792 Dir.cpp bodies that decode end-to-end (7.7%)
carry work after the epilogue label.** Not a curiosity — a decoder's statement
loop must keep running past `54 02` until it sees `4F 12`.

---

## 10. Multi-statement bodies end to end

**[CF] `il_stmt_multi.cpp`**, four statements of three kinds, lines 11–16:

```
4c 4f 11 53
4f 01 0c   26 e8 09  b9 e5 09 86 41 74  33 86 41 74 01  02   32 86 41 74  4b   int x = a + 1;
4f 01 0d   26 e9 09  b9 e8 09 86 41 74  33 86 41 74 02  02   32 86 41 74  4b   int y = x + 2;
4f 01 0e   26 ea 09  26 e4 09 bd 86 41 74 00 80 01 10 00 00
                     b9 e9 09 86 41 74 55 86 41 74 4c        32 86 41 74  4b   int z = g(y);
4f 01 0f   b9 ea 09 86 41 74  41 86 41 74  3a e7 09                            return z;
4f 01 10   54 02  29 e7 09  4f 12 47 54 01 54 00
```

Token allocation, which every fixture above corroborates: **formals in
declaration order, then the function's own symbol, then the epilogue label, then
locals in declaration order, then labels in emission order.** (`g` = `0x09E4`
precedes them all, being declared first.) This is a front-end convention, not
something a decoder should rely on — tokens are only ever compared for equality.

The `26 <dest> … 32 <TYPE> 4B` around a call is the `call-token-0x26` census
bucket (**[DIR] 80,284**): two `26` pushes in a row.

---

## 11. `switch` and `goto` (outside the required set, recorded for completeness)

**[P] `p3.cpp` `switch (a) { case 1: gv(); break; case 2: gv(); break; default: gv(); }`**:

```
4c 4f 11 53 53
  b9 0d 0a 86 41 74
  3b 10 0a                             dispatch on table symbol 0x0A10
  53
    29 14 0a  26 e5 09 bd … 4c 4b  3a 11 0a      case 1: … break
    29 15 0a  26 e5 09 bd … 4c 4b  3a 11 0a      case 2: … break
    29 16 0a  26 e5 09 bd … 4c 4b                default: …
  54 04
  3a 11 0a
  26 10 0a                             push the table symbol
  3c 86 41 74 16 0a                    table header: TYPE int, default -> 0x0A16
  33 86 41 74 01  3d 14 0a             case value 1 -> 0x0A14
  33 86 41 74 02  3d 15 0a             case value 2 -> 0x0A15
  4c                                   end of table
  29 11 0a                             EXIT:
54 03  3a 0f 0a 54 02 29 0f 0a …
```

`3B <tok>` dispatch, `3C <TYPE> <default-label>` table header, `3D <label>` case
entry (preceded by its value), terminated by the same `4C` a call's argument
list uses. The table is emitted **after** the case bodies.

**[P] `p3.cpp` `void gotol(int a) { if (a) goto out; gv(); out: gv(); }`**:

```
4c 4f 11 53 53
  b9 17 0a 86 41 74  38 1a 0a
  53  3a 1c 0a  3a 1b 0a  54 04  29 1a 0a
54 03
  26 e5 09 bd … 4c 4b   3a 1b 0a  29 1c 0a  3a 1b 0a  29 1b 0a
  26 e5 09 bd … 4c 4b   3a 19 0a 54 02 29 19 0a …
```

A user label is an ordinary `29 <tok>`; `goto` is `3A <tok>`. Note the redundant
`3A L` immediately followed by `29 L` — a fall-through pair the backend elides.
**UNKNOWN:** why the labeled statement gets two labels (`0x0A1C` for `out:` and
`0x0A1B` for the statement after it). Harmless to a decoder, unexplained here.

---

## 12. What I could NOT determine

### 12.1 `54 <k>` — plain byte or varint
Every observed value is `< 0x80` (max `0x2A`, at 40-deep nesting, **[P] p6**).
A decoder should compare `k` against its own tracked depth rather than decode it,
which sidesteps the question. **A fixture that would separate them:** 130+ nested
braces — if c1xx accepts that nesting depth at all, which I did not test.

### 12.2 The `if`-clause / loop-body scope asymmetry
An `if` clause gets a `53` even unbraced; a loop body does not (§7, §8.1). Both
readings of *why* (an implicit clause scope for `if` only, vs. the outer `53`
being something other than the `if` statement) predict identical bytes on every
fixture I built. **A fixture that would separate them:** an `if` whose condition
declares a variable (`if (int t = f()) …`) — if the outer `53` exists to scope
condition temporaries, the declaration's token should be assigned inside it and
the shape should change for a loop with the same construct
(`while (int t = f()) …`). Not tested.

### 12.3 `41 <TYPE>` — annotation or conversion
It appears immediately before every value-carrying `3A` and its `<TYPE>` always
equals the function's return type on my fixtures. Whether it *converts* (like
`2C`) or merely annotates is undetermined, because I never produced a return
whose expression type differs from the return type without an explicit `2C` in
between (§4.4 suggests c1xx always inserts one). **A fixture that would separate
them:** `long long f(int a) { return a; }` — if `41` converts, there is no `2C`;
if it annotates, a `2C` must appear. Not tested.

### 12.4 One real-TU scope-depth counterexample
Across 5239 Dir.cpp bodies the `k == remaining depth` invariant fired exactly
once, in **[DIR] seg 707**, at this neighbourhood:

```
… 9b 86 46 80 20 11 >54< 2c 86 43 9e 20 00 64 86 43 9e 20 4c …
```

My scanner is standing on a `54` that follows `9B <TYPE> <varint>` — the
member-bind opcode, which is *expression* layer and which I did not
characterize. Either my `9B` field widths are wrong (the trailing field may not
be a 1-byte varint here) or the `TYPE` read is off; either way the desync happens
before the `54`. I did not chase it. It is 1 body in 5239 and it is not a
statement-layer claim, but it is a real counterexample and it is recorded as one
rather than rounded away.

### 12.5 Label-token allocation
`while` allocates three label tokens and uses two (**[CF] `il_stmt_while.cpp`**:
`0x09E7` is allocated and never emitted; `0x09E8`/`0x09E9` are TOP/EXIT). `for`
allocates three and uses three. I cannot say what the unused one is for. A
decoder does not need to: labels are named by token and defined by `29 <tok>`.

### 12.6 The rest of the `4F NN` marker family
`4F 01` (line) and `4F 12` (function tail) are pinned here; `4F 02`, `4F 11`,
`4F 1F`, `4F 20`, `4F 33` occur and are not statement-layer. The single byte `47`
between `4F 12` and the outer scope closes is unexplained; it is a fixed byte in
every one of the ~5300 bodies examined.

---

## 13. Validation against a real TU (falsification, not admiration)

The grammar above was implemented as a throwaway whole-body Python scanner
(`work/stmt2/stmtgram.py`, untracked). It knows the statement layer completely
and the expression layer only well enough to *skip*. A body counts as decoded
only if **both** hold:

1. the parse lands **exactly** on the 7-byte function tail `4F 12 47 54 01 54 00`;
2. every `54 <k>` satisfies `k == scopes still open after the pop`.

Any width error anywhere in a body desynchronizes (2) almost immediately and
(1) always. Results:

```
all 11 tracked il_stmt_* fixtures + all 8 scratch probes: 60/60 bodies (100%)
[DIR] Dir.cpp:  bodies 5239   landed exactly on the tail: 2792 (53.3%)
                blocked on an unknown EXPRESSION opcode: 2363
                blocked on a malformed TYPE:                83
                scope-depth mismatch:                        1   (§12.4)
                landed off the tail:                         0
```

**Zero bodies landed off the tail.** Every failure is an honest stop at an
unmodeled byte, not a silent desync into a plausible-looking wrong position —
which is the specific failure mode this project has been bitten by.

The residue is entirely expression vocabulary, unchanged from
`IL_CALL_GRAMMAR.md` §7's ranking: `0x66` (1151), `0x5C` (229), `0x67` (126),
`0x43` (93), `0x82` (60), `0x28` (37).

Two deliberate corruptions, to show the widths are load-bearing rather than
merely consistent:

| variant | bodies landing on the tail | off-tail landings |
|---|---:|---:|
| grammar as documented | **2792** | 0 |
| `4F 01` payload read as a fixed 1 byte (today's `eat_opt_stmt_marker`) | 2499 | 0 |
| `TYPE` read as a fixed 3 bytes (today's `INT_TYPE` width assumption) | 663 | 34 |

The fixed-3-byte-TYPE variant is the important row: it does not merely lose
bodies, it produces **34 bodies that land on a wrong `4F 12 47 54 01 54 00`** and
35 scope-depth mismatches. That is what over-acceptance looks like, and it is the
reason the depth invariant is worth enforcing in the real parser.

---

## 14. Recommended decode order

### 14.1 First, the honest correction to the premise

The statement grammar is the largest *census bucket family*, but implementing it
alone buys almost nothing, because the same bodies are also blocked on operand
types. Measured on Dir.cpp with the scanner above, varying one layer at a time:

All four rows are the *same* scanner with one layer swapped, so the differences
are attributable. (For calibration, the real Rust parser reports 183 in class on
this TU; the scanner's 185 in the top row is the same configuration, marginally
looser about the call shape.)

| expression layer | statement layer | bodies decoded (of 5239) |
|---|---|---:|
| today's (int-only TYPE, `+ - *`, one call shape) | today's (single statement) | 185 (3.5%) |
| today's | **full (§0)** | **192 (3.7%)** |
| full skip-layer | today's | 1980 (37.8%) |
| full skip-layer | **full (§0)** | **2792 (53.3%)** |

So: the statement grammar's marginal contribution is **+7 bodies today**, and
**+812 bodies (+15.5 pp)** once the expression layer is complete. It is a
prerequisite with a large *eventual* payoff and a near-zero immediate one, and
anyone landing it should say so rather than claim the 2.9 % `body-0x53` bucket as
coverage. (`docs/ROADMAP.md` §G5 already records two rungs that bought one
function between them; this is the same trap, measured in advance this time.)

The step that actually moves Dir.cpp is **widening the operand TYPE gate**. Held
against the same statement layer, and varying only whether an operand's `TYPE`
must equal `86 41 74`:

| | `+ - *` only | full plain operator table (§5) |
|---|---:|---:|
| operand TYPE must be `int` | 192 | 192 |
| operand TYPE unrestricted | **615** | **615** |

The type gate is worth **3.2×** and the entire rest of the plain operator table
is worth **exactly zero** — every body that would use `&`, `<<`, `%` or `~` is
*also* blocked on a non-`int` operand, so it is invisible until the type gate
moves. That is a measured instance of the demand-driven-widening rule and the
reason this document's own §5 opcode table should not be read as a coverage step.

The type gate is nonetheless *second*, and the two are genuine complements — with
the type gate but **today's** single-statement model the same configuration
reaches only 534 (10.2 %), so 81 of the 615 need the statement layer and 421 of
the 615 need the type gate. The statement layer goes first because it is the
cheaper of the two, because Step 1 below is a strict prerequisite for the census
being ranked correctly at all, and because the type gate's own emission gate
(knowing how to *lower* a pointer or a `double`) is far more work than its decode.

### 14.2 The incremental order

Each step below is decode-only. **Decoding a production is not licence to emit
it**: every step keeps a separate emission gate, and a body that decodes but hits
an unmodeled construct must still return `NotImplemented`. The fail-closed
boundary is stated per step.

**Step 1 — line markers as varints.** Change `eat_opt_stmt_marker` to
`4F 01 <read_varint>` and call it in a loop (two in a row is normal, §3).
*Gain:* the `body-0xAD` (15,480) + `body-0xB3` (15,782) + `body-0x4F` (26,666)
buckets stop being width errors — 57,928 functions **[DIR]** move to their real
blocking feature, and Dir.cpp's whole-body decode reach goes 2499 → 2792.
*Fail-closed boundary:* none needed — this is a pure width fix in a field whose
value is discarded. It cannot admit anything.
*Do this first regardless of everything else: it is ~5 lines and it is currently
mislabelling the census that the whole widening order is ranked from.*

**Step 2 — the scope stack.** Accept `53` / `54 <k>`, maintaining a depth counter
and **rejecting on `k != remaining`**. Replace the fixed `54 02 29 <tok>` match in
`eat_return_plumbing` with `54 <k>` + a `29 <tok>` label statement, with line
markers consumed at every boundary.
*Gain:* the `body-0x53` bucket (70,078, 2.9 %) and `body-0x29` (14,594, 0.6 %)
become reachable; `fn-tail-0xB9` (29,552) — a statement after the tail plumbing —
resolves.
*Fail-closed boundary:* the depth mismatch itself, plus refusing to *emit* any
body whose scope depth ever exceeds 3 (i.e. that has a nested scope at all) until
a CFG exists in codegen. Decode deeper, emit only flat.

**Step 3 — the statement list.** `body := 53 stmt*` with `4B` ending an
expression statement and no separator between statements (§2), plus assignment
`<lvalue> <value> 32 <TYPE> 4B` (§4).
*Gain:* the `assign-0x26` family, `call-token-0x26` (80,284) and every
multi-statement straight-line body.
*Fail-closed boundary:* the whole-body positive parse must still reach the
segment end, and codegen must refuse anything with more than one live value at a
statement boundary. Keep `32 <TYPE>` restricted to `TYPE == INT_TYPE` for
emission even though the *width* now comes from `read_type` — §4.4 guarantees any
conversion is a separate visible `2C`, so this restriction cannot silently drop
one.

**Step 4 — compound assignment (§5).** `<op> <TYPE>` for the twelve witnessed
opcodes (`0F 10 11 12 13 15 16 17 18 19 35 36`, covering fourteen C operators).
*Fail-closed boundary:* accept **only** those twelve bytes. `0x14`
is unobserved and must reject. Do not derive `|=`/`^=` from the plain table —
they are swapped. This is a small gain (+2 bodies on Dir.cpp) but it is nearly
free and it removes four hex buckets from the census.

**Step 5 — branches and labels (§7, §8).** `38`/`39` conditional branches,
`3A` jump, `29 <tok>` label definitions; build a token → position map in a first
pass over the body, since `3A` carries no direction.
*Gain:* `if`, `if/else`, `while`, `for`, `do/while`, `break`, `continue`, `goto`
and early `return` — all of control flow except `switch` — for **zero** new
statement opcodes beyond these four.
*Fail-closed boundary:* this is where the temptation is greatest and the risk is
highest. Decoding a CFG is not lowering one. Emission must stay gated on a
whitelist of *shapes* (§9's single-return tail first, then a diamond-free `if`),
never on "the branches decoded". Also: refuse any body where a label is targeted
before it is defined **and** the port's codegen has no fixup pass — a backward
jump is a loop, and a loop needs register allocation across a back edge.

**Step 6 — the operand TYPE gate.** Not a statement step, and out of this
document's scope, but it is the step with the largest measured decode gain
(§14.1) and it is unlocked by steps 1–3. Listed so the order is not read as
"statements are the whole answer".

`switch` (§11) is deliberately last: three more opcodes, a jump table in `.rdata`
or `.text`, and **[DIR]** it is a small fraction of bodies. Its grammar is
recorded here so it can be *recognized and refused* precisely rather than
bucketed as an unknown byte.

---

## 15. Reproduction

```
cargo build --release -p c2-harness

# the tracked characterization fixtures
for f in fixtures/cpp/il_stmt_*.cpp; do
  ./target/release/c2rs census "$f" --keep-il "work/stmt2/il_$(basename "$f" .cpp)"
done
python3 work/exp/tools/dump.py work/stmt2/il_il_stmt_if        # raw segment dump
python3 work/stmt2/stmtgram.py  work/stmt2/il_il_stmt_if       # grammar validator

# the real workload
./target/release/c2rs census src/system/world/Dir.cpp \
    --flags-file work/dc3-workload/flags.txt --cwd ../dc3-decomp \
    --keep-il work/stmt2/il_dir
python3 work/stmt2/stmtgram.py work/stmt2/il_dir
```

Tracked fixture index (`fixtures/cpp/`): `il_stmt_seq` statement-count ladder ·
`il_stmt_multi` mixed multi-statement body · `il_stmt_local_decl` declaration vs
initializer · `il_stmt_param_assign` parameter vs local destination ·
`il_stmt_compound_assign` `+=` vs `x = x + k` · `il_stmt_bare_block` scope
markers · `il_stmt_if` braced vs unbraced then-clause · `il_stmt_if_else` the
else arm · `il_stmt_while` backward jump · `il_stmt_for` rotated increment ·
`il_stmt_early_return` non-final return, int and void.

Untracked probes (`work/stmt2/p/`, regenerate from §1/§5/§7/§8/§11 above):
`p1` unbraced loop body + 4-deep nesting · `p2` the compound-assignment operator
table · `p3` value discard, `&&`/`||`/`!`, `break`, `continue`, `do/while`,
`switch`, `goto` · `p4` the plain binary/unary operator table + `x--` ·
`p5` a function at source line 200 (wide line marker) · `p6` 40-deep nesting ·
`p7` assignment as a value, narrowing conversion, store through a pointer ·
`p8` member functions (pre-body depth).
