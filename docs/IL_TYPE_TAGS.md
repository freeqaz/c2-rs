# IL type tags, and which of them the port may accept

Characterization of the `.ex` TYPE encoding and of what each scalar type costs in
codegen. Written because `expr-load-type-*` is not one census bucket but a family
of them — `864540`, `888541`, `864383`, `864275`, `86439E` and a long tail — that
together account for roughly 8% of blocked functions, and it was not clear how
many of those were one mechanism.

Everything below is from live captures of the real toolchain. Where a rule is
*not* determined by the captures I have, it says so, and the port refuses.

## 1. Encoding

A TYPE is `<tag> <kind> <LEB128 id>`, so 3, 4 or 5 bytes — this is what
`read_type` implements. The `id` is an index into the TU's type table, which is
why a pointer type's third byte varies between TUs and why census bucket names
that truncate at three bytes group *different* pointer types together.

The `tag` encodes a width as `0x80 + 2*(log2(size)+1)`:

| size | tag |
|------|-----|
| 1 | `82` |
| 2 | `84` |
| 4 | `86` |
| 8 | `88` |

**In a `.gl` DATA record the same field is the object's ALIGNMENT, and the wide
bit is orthogonal to it** (board #1117, `docs/rungs/2026-08-08-w-align.md` §2).
The heading above says `size`, which is true only for scalars, where a type's
size *is* its alignment; `gl.rs::data_object_at` reads the tag as alignment and
the size from its own field. `TAG_WIDE` (`0x40`) marks one extra byte before the
kind and nothing else, so `C6` is `86` is **4** — confirmed on 21 of 21 object
records against **c2's own obj** alignment nibbles, across cells whose size and
alignment disagree in both directions.

**`8A`/`CA` is 16 and is now READ** — board #1120, lane `w-align16`, on a
20-cell structural grid whose section nibbles and symbol `Value`s agree with c2's
own obj **24 of 24** and **25 of 25**. The writer's promotion table
(`placement_align`, `align_nibble`, `data::section_nibble` — three bodies, one
table) models 1/2/4/8/**16**. Three things that grid settled and that a reader of
this table needs:

* **The size-implied promotion caps at 8.** `char g[4096]` is tag `82` and c2
  gives its section nibble **4**, not 5. Everything above 8 arrives through the
  *tag*, never through the size.
* **`8C` = 32 and `8E` = 64 EXIST**, and c2 honours them with nibbles 6 and 7 —
  so the `0x80 + 2*(log2(size)+1)` encoding is confirmed to 64. **They are
  refused**, because the grid varies structure nine ways at 16 and one way at
  32/64.
* **Bare `8A` has never been observed.** Every 16-aligned cell spells the *wide*
  `CA`, and a census of all 878 workload TUs finds **0** records at `8A`, `CA`,
  `8C`, `CC`, `8E` or `CE` out of **85,895** — the workload's whole `.gl`
  vocabulary here is `82`, `84`, `86`, `88`, `C6`. The non-wide 16 arm the port
  ships is the orthogonality rule applied, not a witness, and
  `gl.rs::align_of_type_tag` says so.

**The width is the token's, not the type's — the tag is positional.** The same
type carries different tags in different slots: `double*` is `86 43 c1 08` as a
`B9` operand (a 4-byte pointer is being loaded) and `88 43 c1 08` as a `27`
member-offset result (an 8-byte `double` is what the resulting address denotes).
So a TYPE triple cannot be matched as a constant across positions, and a table
keyed on "the tag for type T" is wrong by construction. The `<kind>` and `id` are
what identify the type; the tag describes the slot. Established by
`IL_EXPR_LAYER.md`'s indirect-load captures, which is also where the per-slot
tags are tabulated.

**And the three-field rule does not hold for aggregates.** `86 06 20 ec 20`
parses as three bytes under `<tag> <kind> <LEB128>` but the bracketing `41` marker
pins it at five, so `read_type` mis-reads it — a latent *mis-decode* that
desynchronizes the stream the way the fixed-width source-line marker did, not
merely a gap. Recorded rather than fixed; the aggregate form needs its own
capture matrix.

## 2. The scalar table

Captured from a TU of sixteen identity functions (`T f(T a) { return a; }`), one
per type:

| C++ type | TYPE bytes |
|---|---|
| `char` | `82 11 70` |
| `signed char` | `82 11 10` |
| `unsigned char` | `82 12 20` |
| `bool` | `82 12 30` |
| `short` | `84 21 11` |
| `unsigned short` | `84 22 21` |
| `wchar_t` | `84 22 71` |
| `int` | `86 41 74` |
| `long` | `86 41 12` |
| `unsigned` | `86 42 75` |
| `unsigned long` | `86 42 22` |
| `int*` | `86 43 f4 08` |
| `void*` | `86 43 83 08` |
| `const char*` | `86 43 81 20` |
| `float` | `86 45 40` |
| `double` | `88 85 41` |

So `<tag> <kind>` is the type *class* and the id distinguishes members of it:
`86 41` is signed 32-bit, `86 42` unsigned 32-bit, `84 21`/`84 22` the 16-bit
pair, `82 11`/`82 12` the 8-bit pair, `86 43` pointer (id = the pointee's type
index), `86 45` float, `88 85` double.

Two census buckets decode straight out of this table: `expr-load-type-864383` is
`void*` and `expr-lit-type-821230` is a `bool` literal.

Note the *literal* FP tags are different again — `86 4a 40` for a float literal
against `86 45 40` for a float operand, and `88 8a 41` against `88 85 41`. See
`CODEGEN_W13_FLOAT.md` §5.

## 3. What each type costs in codegen

Identity (`T f(T a){return a;}`) is a bare `blr` for **every** type in the table,
including all the narrow ones and all the pointers. The ABI delivers an argument
already extended, and returning it needs nothing. So width is free at the
boundary; it costs only where arithmetic happens.

Arithmetic is where they diverge, and not uniformly:

```
long  a+b   ->  add r3,r3,r4                                      ; blr
ulong a+b   ->  add r3,r3,r4                                      ; blr
short a+b   ->  add r11,r3,r4 ; extsh r3,r11                      ; blr
char  a+b   ->  add r11,r3,r4 ; extsb r3,r11                      ; blr
ushort a+b  ->  rlwinm r10,r3,0,16,31 ; rlwinm r11,r4,0,16,31 ;
                add r11,r10,r11 ; rlwinm r3,r11,0,16,31           ; blr
uchar a+b   ->  rlwinm r10,r3,0,24,31 ; rlwinm r11,r4,0,24,31 ;
                add r11,r10,r11 ; rlwinm r3,r11,0,24,31           ; blr
bool  !a    ->  rlwinm r11,r3,0,24,31 ; cntlzw r10,r11 ;
                rlwinm r3,r10,27,31,31                            ; blr
```

### 3.1 `long` and `unsigned long` are `int` and `unsigned`

Byte-identical, not merely equivalent. Accepting `86 41 12` wherever `86 41 74`
is accepted, and `86 42 22` wherever `86 42 75` is, is therefore free of risk —
there is no extension, no mask and no reordering to get wrong. This is the one
widening in this document that can be taken without further captures.

### 3.2 Narrow types are not one rule, and must keep refusing

Three separate inconsistencies, any one of which defeats a single-rule model:

* **signed narrow extends the output, unsigned narrow masks the inputs too.**
  `short a+b` does not touch its inputs — the ABI already sign-extended them —
  and extends only the result. `unsigned short a+b` masks *both* inputs anyway,
  even though the ABI already zero-extended them, and is four instructions rather
  than two.
* **the result type, not the operand type, drives input extension.** The same IL
  expression over the same operands lowers differently depending on what it is
  assigned to:

  ```
  short a_short(short a, short b) { return a + b; }   -> add ; extsh   (output)
  int   a_sh2i (short a, short b) { return a + b; }   -> extsh ; extsh ; add
                                                          (inputs, no output)
  ```

* **the operator matters as well.** `short a*b` extends both inputs *and* the
  output — `extsh ; extsh ; mullw ; extsh` — where `short a+b` extends neither
  input.

So the extension placement is a function of (operator, operand type, result type)
with at least three distinct behaviours already visible in seven captures. That
is not enough to implement from, and getting it wrong is a silent wrong-bytes
emit rather than a refusal, so the port rejects every narrow type in an
arithmetic position.

### 3.3 Pointers

Identity is free, but nothing else here is characterized: no capture yet covers
dereference, indexing, member access or pointer arithmetic, all of which are the
reason pointer types show up in the census at all. `86 43 …` stays refused.

## 4. Suggested order

1. `long` / `unsigned long` as aliases of `int` / `unsigned` — free, no new
   captures needed (§3.1). Small census gain; neither tag is in the top twenty
   buckets, so treat this as tidying rather than coverage work.
2. Find out why `expr-load-type-864275` (`unsigned`, 1.6%) blocks at all, given
   that `unsigned` is already an accepted operand type. Either a position in the
   expression grammar accepts `int` only, in which case this is a bug worth 1.6%,
   or the blocked functions are mixed-signedness expressions, in which case it
   needs its own characterization. **Not yet investigated.**
3. Narrow integer arithmetic (§3.2) — needs a proper matrix of captures over
   (operator × operand type × result type) before any of it can be implemented.
4. Pointers (§3.3) — blocked on dereference/indexing characterization, which is
   a larger piece of work than the rest of this document combined.
