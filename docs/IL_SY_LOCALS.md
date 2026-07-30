# `.sy` — the local-symbol stream, and whether it names a function's locals

The port's biggest decode blocker is `assign-dst-not-formal`: the body parser
accepts an assignment only when the destination token is in the `.ex` formals
list, because `.ex` itself carries no positive signal that a token names a
local. This document characterizes the `.sy` member of the 5-file IL bundle and
answers whether it is that signal. **It is**: `.sy` is a per-TU stream of
per-function blocks, and each block lists every formal and every local of its
function — token, name, scope depth, type, size, and flags — with a decidable
binding from block to `.ex` body.

Everything below is from live captures of toolchain `16.00.11886.00` under
wibo, via `c2rs census <cpp> --keep-il <dir>` (default capture flags,
`/Ox /GS-`). Probe sources are quoted inline; they lived in `/tmp/syprobe/` and
are one-liners, so the quotes are the record. Recapturing a probe reproduces
its `.sy` byte-for-byte (measured on `p06`), so the hex here is stable
evidence. Each claim is marked **MEASURED** (a probe pair shows it) or
**HYPOTHESIS** (a reading the probes are consistent with but do not pin). A
field that never varied across the probes is called *indistinguishable from a
constant*, not named.

## 1. File-level grammar (MEASURED over 30 probes)

`.sy` has no file header: offset 0 is the first record. The whole file is:

```
sy       := fnblock*
fnblock  := label* fnrec scope* 06
label    := 03 03 <tok> <u16 LE> 01 <b>          six bytes + token
fnrec    := 03 01 <tok> <tail:4>                 1F 00 01 01 here, 1F 00 02 01
            in real TUs — stepped over by width, never checked
scope    := 0D <depth> var*
var      := plain | array | static               (the record kinds observed)
plain    := 01 <depth> <tok> 00 <name> 00 <type>
array    := 02 <depth> <tok> 00 <name> 00 <type> <elem-size>
static   := 07 <depth> <tok> <mangled-name> 00
            <tag> <kind> 00 <size8> 04 00 <tid> <tok'> 00
type     := <tag> <kind> 00 <cls> 04 <size16 LE> <flags16 LE> <tid>
            (aggregate types insert 80 00 between flags and tid — §3.6)
tid      := 1 byte if < 0x80, else 80 + <LE32>   (same varint as .ex literals)
<tok>    := the .ex operand-token encoding (2 bytes observed; §7)
```

One `fnblock` per function, in `.ex` segment order; `06` terminates the block;
nothing else appears between blocks. A function with no formals and no locals
still gets a full block — `int g() { return 7; }` inside `m3_mixed.cpp` is

```
03 01 e8 09 1f 00 01 01 0d 01 0d 02 06
```

so `0D 01` (the formal scope) and `0D 02` (the function's top block scope) are
**always present, even empty**. This kills the reading of `0D 01`/`0D 02` as
count-prefixed groups ("one record follows" / "two records follow"): here both
are followed by zero records, and in the three-locals probe `0D 02` is followed
by three.

### The baseline, re-read

`int f(int a) { int x = a + 1; int y = x + 2; return y; }` (`p02_base.cpp`):

```
03 01 e5 09 1f 00 01 01                                     fnrec, tok e5 09
0d 01                                                       scope depth 1
  01 01 e3 09 00 61 00 86 01 00 03 04 04 00 01 00 74        a  (formal, int)
0d 02                                                       scope depth 2
  01 02 e7 09 00 79 00 86 01 00 01 04 04 00 01 00 74        y  (auto, int)
  01 02 e6 09 00 78 00 86 01 00 01 04 04 00 01 00 74        x  (auto, int)
06                                                          end of block
```

## 2. The prior hypothesis, adjudicated

The working reading was: records `01 <kind> <tok> 00 <name> 00 86 01 00 <n> 04
04 00 01 00 74`, `kind` 01 = formal, 02 = local.

* **Record framing: confirmed** — and every "constant" in that tail is a field
  that moves (§3).
* **`kind` 01/02: refuted as stated.** The second byte is the **lexical scope
  depth**, equal to the depth of the `0D` group the record sits in. A local in
  a nested brace is `01 03`, not `01 02` — `n_nested.cpp`
  (`int f(int a) { int x = a + 1; { int y = x + 2; return y; } }`) ends
  `… 0d 03 01 03 e7 09 00 79 …`. A gate reading "02 = local" silently drops
  every brace-scoped local. The correct reading: **depth 1 = the formal scope,
  depth ≥ 2 = a body scope** (§4).
* **`<n>` (3 formal / 1 local): not a position.** All three formals of
  `int f(int a,int b,int c)` carry 03 (`p04_three_formals.cpp`), so it is not a
  parameter index. It never varied within a class across 30 probes (autos are
  01 even when unused, address-taken, arrays, or aggregates), so it is
  indistinguishable from a per-class constant. HYPOTHESIS: a storage-class
  code. Statics do not carry the field at all — their tail is a different
  shape (§3.5).

## 3. The var record, field by field

Using the plain-record tail `<tag> <kind> 00 <cls> 04 <size16> <flags16> <tid>`.

### 3.1 `<tag> <kind>` — the type family (MEASURED)

These mirror the `.ex` inline type's tag and kind with the high bits dropped.
The tag's low nibble is the same width code `readers.rs::type_width` decodes
(`…2`→1, `…4`→2, `…6`→4, `…8`→8 bytes):

| local declared | tail bytes | tag/kind | size16 | tid |
|---|---|---|---|---|
| `int x` | `86 01 00 01 04 04 00 01 00 74` | 86 01 | 4 | 0x74 |
| `unsigned x` | `86 02 00 01 04 04 00 01 00 75` | 86 02 | 4 | 0x75 |
| `short x` | `84 01 00 01 04 02 00 01 00 11` | 84 01 | 2 | 0x11 |
| `char x` | `82 01 00 01 04 01 00 01 00 70` | 82 01 | 1 | 0x70 |
| `long long x` | `88 01 00 01 04 08 00 01 00 13` | 88 01 | 8 | 0x13 |
| `float x` | `86 05 00 01 04 04 00 01 00 40` | 86 05 | 4 | 0x40 |
| `double x` | `88 05 00 01 04 08 00 01 00 41` | 88 05 | 8 | 0x41 |
| `int* x` | `86 03 00 01 04 04 00 01 00 80 74 04 00 00` | 86 03 | 4 | 0x474 |
| `int& x` | `86 03 00 01 04 04 00 01 00 80 00 10 00 00` | 86 03 | 4 | 0x1000 |
| `const int x` | `86 01 00 01 04 04 00 01 00 80 00 10 00 00` | 86 01 | 4 | 0x1000 |
| `volatile int x` | `86 01 00 01 04 04 00 01 00 80 00 10 00 00` | 86 01 | 4 | 0x1000 |
| `int x[4]` | `86 06 00 01 04 10 00 01 00 80 00 10 00 00` + `04` | 86 06 | 16 | 0x1000 |
| `char x[3]` | `82 06 00 01 04 03 00 01 00 80 00 10 00 00` + `01` | 82 06 | 3 | 0x1000 |
| `struct A{int m,n,o;} x` | `86 06 00 01 04 0c 00 81 00 80 00 80 03 10 00 00` | 86 06 | 12 | 0x1003 |
| `struct B{char c;} y` | `82 06 00 01 04 01 00 81 00 80 00 80 06 10 00 00` | 82 06 | 1 | 0x1006 |

Kind low nibble matches `.ex`: 1 signed integer, 2 unsigned, 3 pointer *and*
reference, 5 floating, 6 aggregate/array. The `00` after the kind never varied.

### 3.2 `<size16>` — byte size (MEASURED)

LE16, the object's whole size: 1/2/4/8 for the scalars, 16 for `int[4]`,
28 (`1c 00`) for `int[7]`, 12 for the 3-int struct. Discriminated from
alignment by `int x[7]` (28 ≠ 4) and from element size by `short x[5]`
(`0a 00` = 10).

### 3.3 `<tid>` — the `.ex` type-table id (MEASURED)

The trailing field is the **same type id the `.ex` operand stream uses**,
encoded with the `.ex` literal varint rule (one byte below 0x80, else `80` +
LE32): int 0x74 here, and `86 41 74` in `.ex`; `int*` is 0x474 here and
`86 43 F4 08` (LEB `F4 08` = 0x474) in `.ex`. Constructed types number from
0x1000 in TU order: the two structs of `t_struct2.cpp` get 0x1003 and 0x1006;
`const int`, `volatile int`, `int&` and every array type land in the same
0x1000+ range. So a qualifier does **not** set a flag bit anywhere in the
record — it moves the tid into the constructed range. `register` moves nothing
at all: `register int x` is byte-identical to `int x` (indistinguishable from
being ignored).

### 3.4 `<flags16>` — reference and address bits (MEASURED)

| probe | flags | reading |
|---|---|---|
| every plain used local/formal | `01 00` | bit 0 = referenced |
| `int x;` never used (`u_unused.cpp`) | `00 00` | bit 0 clear |
| `int x = a; g(&x);` (`q_addr.cpp`) | `21 00` | bit 5 = address taken |
| formal `a` when `int& x = a;` (`t_ref.cpp`) | `21 00` | reference binding = address taken |
| struct by value (formal or local) | `81 00` | bit 7 set — see §3.6 |

The `t_ref` capture is the discriminator that the flags belong to the record's
*own* symbol: taking `a`'s address flips **`a`'s** record (`… e3 09 00 61 00 86
01 00 03 04 04 00 21 00 74 …`), while `x`'s stays `01 00`.

### 3.5 Statics restructure the record (MEASURED)

`int f(int a) { static int x = 5; return x + a; }` (`t_static.cpp`):

```
07 02 e6 09  3f 78 40 3f 31 3f 3f 66 40 40 59 41 48 48 40 5a 40 34 48 41 00
             "?x@?1??f@@YAHH@Z@4HA"
86 01 00  04  04 00  74  e7 09  00
```

Lead byte **07**, the *mangled* name directly after the token with no leading
`00`, and a different tail: `<tag><kind> 00 <size8> 04 00 <tid> <tok'> 00` —
no `<cls>`, no `<flags16>`, plus a **second token**. The `<size8>` reading is
pinned by four statics: char → 01, short → 02, int → 04, double → 08
(`t_static_char/short/double.cpp`); the `04 00` between it and the tid never
varied. The second token `e7 09` is the one the **body actually loads**:
`t_static`'s `.ex` reads `b9 e7 09 86 41 74` and never references `e6 09`. So
the record token names the source-level symbol and `<tok'>` names the data
symbol — which means a locals gate keyed on record tokens will *not* claim a
static's body references. That is the right failure direction: a static local
is memory, not a register-homed local.

### 3.6 Aggregates carry two extra bytes — RESOLVED, and it is a separate field

Every by-value struct record (formal and local, both probes) has `81 00 80 00`
where scalars have `<flags16>`, then the tid varint (`80 03 10 00 00` = 0x1003).

**This section used to stop here**, saying it was "not derivable" from two struct
types whether that is a 4-byte flags field or `<flags16>` plus an unknown 2-byte
field, and `sy.rs` refused the whole file on any aggregate record as a result.
That refusal was expensive in a way nobody had measured: `.sy` binds a translation
unit 1:1 or not at all, so one struct member or parameter anywhere in a file
withheld the binding for *all* of it — and once the argument-register precondition
started depending on `.sy` widths (`param-width-undetermined`), the bill came to
**567,549 functions**, the single largest census bucket, 2.3× the next.

It is two fields, and the discriminating witness was already in the table below:

* a **struct** parameter has `<flags16>` = `80 00`, and a struct **local** the same
  field as `81 00` — differing in bit 0, *referenced*, which is a flag bit. So the
  flags are being read, and the trailing `80 00` is something else.
* an **array** local (`int x[4]`) has kind class 6 **as well**, and `<flags16>` =
  `01 00`, and **no** extra field — its `80 00 10 00 00` is a genuine wide id of
  `0x1000`. A reader keyed on the kind alone eats two bytes of that id and desyncs
  every record after it.

So the extra field is consumed when the kind's class nibble is 6 **and** flags bit
7 is set, and its value is required to be literally `80 00` (eight witnesses, never
varied, meaning still unknown — a never-varying field is indistinguishable from a
constant, so it fails closed rather than being stepped over by width).

The general lesson is worth more than the bytes: the ambiguity was real for a
reader that ignored the kind byte, and the kind byte precedes the field. "Not
derivable" was a statement about the reader, not about the format, and it sat in
this document as if it were about the format. Arrays instead
append one byte *after* the tid: 04 for `int[4]`, `int[7]` **and** `float[2]`,
02 for `short[5]`, 01 for `char[3]` — so it is the **element size**, not the
element count (the `int[7]` probe is what discriminates: count would be 07).

### 3.7 The lead byte (MEASURED values, enum not closed)

01 = plain var (scalars, references, by-value structs), 02 = array var,
07 = static var, 03 = the function/label record family (§5). No other values
observed. It is not a bitfield reading anything obvious (an array is 02, not
03 = 01|02). Treat unknown leads as unparseable and refuse.

## 4. Scopes: `0D <depth>` is a preorder walk (MEASURED)

`0D` is not a formal/local separator; it opens a scope group at an explicit
depth, and groups appear in **preorder**. Depth 1 = formals, 2 = function top
block, 3+ = nested braces. `n_tree.cpp`
(`int f(int a) { { int u…; { int v…; } } { int w…; } return a; }`):

```
0d 01 [a]  0d 02  0d 03 [u]  0d 04 [v]  0d 03 [w]  06
```

— the empty `0d 02` still emitted, sibling braces as two `0d 03` groups, and
depth+preorder is enough to rebuild the tree. Two same-named locals in
disjoint scopes (`n_disjoint.cpp`) produce two `0d 03` groups each holding a
`u` record with a **different token** (`e7 09` vs `e8 09`); a shadowing `x`
inside a brace (`n_shadow.cpp`) likewise gets its own token at depth 3. So
name collisions are the parser's problem only if it keys on names — key on
tokens and they are not collisions at all.

A `for` loop's induction variable is an ordinary depth-3 local: `l_for.cpp`
(`int s = 0; for (int i = 0; i < a; ++i) s += i;`) lists `s` at depth 2 and
`i` at depth 3, plus three label records (§5). The same locals declared
without the loop look identical record-wise — the loop adds labels, not a new
record shape.

## 5. Function boundaries and the binding rule (the load-bearing answer)

`.sy` is **one file per TU** with explicit per-function blocks. The boundary
markers, measured on 2- and 3-function TUs (`m2_two_funcs.cpp`,
`m3_mixed.cpp`, `m4_for_second.cpp`, `v_staticfn.cpp`):

* A block **starts** at `03 01 <tok> 1F 00 01 01` and **ends** at the next
  `06`. Blocks do not nest.
* Blocks appear in **`.ex` segment order** (f then g then h, matching the
  `4F 1F` segment sequence) in every multi-function probe.
* `03 03 <tok> <u16> 01 <b>` records may precede a block's `03 01` — they
  belong to the **following** function. `m4_for_second.cpp` (plain `f`, then
  `g` with a `for`) shows f's `06`, then three `03 03` records, then g's
  `03 01`. Their tokens are g's **loop label tokens**: the `.ex` body uses
  exactly those tokens as `3A <tok>` label-defines and `29 <tok>` branch
  targets. The `<u16>` payloads observed are 0x08/0x0B/0x0C (and always 0x1F
  for the `03 01` record); the final byte is usually 01, once 02 — neither is
  derived. Skip the whole 6+token bytes.
* The `03 01` token is **not** the function symbol. `.gl` binds `?f@@YAHH@Z`
  to `e4 09` (visible in the `p02` `.gl`: `c2 0e e4 09 00 "?f@@YAHH@Z"`), while
  the block token is `e5 09`. MEASURED: the block token equals the token in
  the segment's terminal return plumbing `3A <tok> 54 … 29 <tok>` — the
  function's exit label — in every probe (f: `3a e5 09 … 29 e5 09` /
  block `03 01 e5 09`; m2's g: `3a ea 09` / `03 01 ea 09`; m4's g: `3a e9 09`
  / `03 01 e9 09`). HYPOTHESIS from the same data: it is always
  function-symbol-token + 1; that is consistent with every capture but is an
  allocation-order accident nothing promises.

So the binding rule the port can use, with a positive cross-check:

> **Block k binds to `.ex` segment k.** Verify by tokens before trusting it:
> the block's depth-1 records must be exactly the segment's `2D <tok>` formal
> list, same tokens, same order (MEASURED on every probe — e.g. m2's g
> declares `2d e8 09 2d e7 09` and the block lists e8 then e7). A block count
> different from the segment count, or any token mismatch, refuses the TU.

The formal-order agreement is itself a measured fact worth stating: **both**
`.sy` depth-1 records and the `.ex` `2D` list run in *reverse declaration
order* (`int f(int a,int b,int c)` → c, b, a in both).

### What `.sy` does *not* contain

File-scope symbols. `int gv; int f(int a){ int x = a + gv; return x; }`
(`g_global.cpp`) produces a `.sy` with only `a` and `x` — `gv` is absent
(it lives in `.gl`). So *"token has a var record at depth ≥ 2 in this block"*
is a positive, unpolluted local test: globals can never satisfy it, statics'
body tokens (§3.5) can never satisfy it, and formals sit at depth 1.

## 6. Record order within a scope is name-driven — do not use it

MEASURED: within one `0D` group, record order is a deterministic function of
the **names**, not of declaration or use order:

| locals declared in order | record order |
|---|---|
| x, y | y, x |
| x, y, z | y, x, z |
| w, x, y, z | w, y, x, z |
| a, b, c, d | a, b, d, c |
| b, c | b, c |
| mm, nn, oo | mm, nn, oo |

Same declaration structure, different names → different permutation, so it is
a symbol-table artifact (hash-bucket-like). A 70-local probe shows descending
runs consistent with prepend-chains, but no simple rule (sum/x31/x33/x65599/
first/last byte, 2..4096 buckets, either chain direction) reproduces the
order. **Not derived, and not needed**: use-order in `.ex` binds by token.
Reverse-declaration-order — plausible from the two-local baseline (y, x) — is
refuted by `{x,y,z}` → y, x, z (reverse would be z, y, x). Declaration order —
plausible from `{mm,nn,oo}` — is refuted by `{x,y}` → y, x.

## 7. Encodings shared with `.ex` (and one untested edge)

Tokens in `.sy` records are the `.ex` operand tokens, byte-identical: the
baseline's `e3 09` (formal, `.ex` `2d e3 09`), `e6 09`/`e7 09` (locals, `.ex`
assignment destinations `26 e6 09`/`26 e7 09`). A 70-local probe walks tokens
through the page boundary (`ff 09` → `00 0a`, records showing `26 0a` etc.),
still 2-byte with the second byte's bit 7 clear. **No 4-byte token was ever
observed in `.sy`** — the probes cannot reach ids that large. HYPOTHESIS: the
`read_token_var` width rule applies as in `.ex`. A reader should decode with
`read_token_var` and refuse a record whose token width makes the following
byte not match the record shape (`00` for plain/array, printable for static).

Also untested here: `/O1` capture (all probes rode the default `/Ox /GS-`;
`.sy` looks like front-end output and should be mode-independent, but that is
untested), member functions (`this` never appears in these probes; whether it
gets a depth-1 record is unknown), EH scopes, and inline functions.

## 8. What the parser can safely gate on today

The minimal `.sy` reader that unlocks `assign-dst-not-formal`, stated as the
rule plus its refusals:

**Accept token T as a local of function k iff all of:**

1. `.sy` splits completely into blocks: any run of `03 03 <tok> <u16> 01 <b>`
   records, then `03 01 <tok> 1F 00 01 01`, then `0D`-groups of var records,
   then `06` — with **every byte accounted for** (the `eat`-style positive
   parse the `.ex` grammar already uses). Anything else in the stream refuses
   the TU.
2. The number of blocks equals the number of `.ex` segments, and block k's
   depth-1 tokens are exactly segment k's `2D` formal list in the same order.
3. T has a var record in block k with lead `01`, depth ≥ 2, flags `01 00`,
   and a tid the port's type model accepts (for the current class: `74`).
   The size16 must agree with the tid's width (4 for `74`) — a disagreement is
   a record we misread, not a variant to tolerate.

**Refuse (fail closed), by name:**

* lead bytes other than 01/02/07, or any unframeable record — unknown grammar;
* lead 02 (arrays) and lead 07 (statics) as assignment destinations — memory,
  not register locals; note statics also fail naturally because the body uses
  `<tok'>`, which no record carries as its token;
* flags other than `01 00`: `21 00` (address-taken) means the local must be
  stack-homed, `00 00` (unreferenced) has no measured codegen witness yet;
* an aggregate record whose extra 2-byte field is not `80 00`, or a class-6
  record where the kind and flags bit 7 disagree — the field's meaning is
  unknown, so a new value is new information (§3.6);
* a `03 01` record whose `<u16>` is not `1F 00` — the field never varied, so
  a new value is new information, not noise;
* any block-count or formal-token mismatch with `.ex` (rule 2) — that is the
  boundary-binding invariant, and emitting with a wrong binding would home a
  local in the wrong function, a wrong-bytes emit rather than a refusal.

Depth ≥ 2 — not == 2 — is load-bearing (§4): brace-scoped and loop locals are
depth 3+. Tokens, not names and not record order, are the identity (§6).

## 9. Open questions, honestly

* The `<u16>` in `03 01`/`03 03` records (0x1F / 0x08 / 0x0B / 0x0C observed)
  and their final byte (01, once 02): not derived. Fixed values, refuse-on-new.
* `<cls>` = 01/03 and the `04` byte after it: per-class constants in every
  probe; storage-class reading is a hypothesis. The `04` also appears (shifted)
  in the static tail; whether it is the same field is unknown.
* What the aggregate's extra `80 00` MEANS (§3.6 settles only that it is a
  separate field from `<flags16>`, and that class 6 plus flags bit 7 is what
  selects it); whether `flags16` is really 16-bit.
* The local record order rule (§6): name-driven, deterministic, not derived.
* Whether the `03 01` token is guaranteed to be the exit-label token or merely
  always was: cross-check via formals (rule 2) rather than resting on it.
* `this`, EH, `/O1`, 4-byte tokens: uncaptured (§7).
* A **polymorphic** class record opens `C6 81 03 …` — a three-byte type prefix on
  the `0x40` tag bit `readers.rs` records as occurring and undetermined. Located
  and refused, from one witness pair (`struct V { virtual void f(); int a; }` and
  a derived class); one pair cannot decode a new prefix form.
  `fixtures/cpp/il_param_poly_neg.cpp` pins the cost, which is a whole-file
  refusal and therefore paid by that file's other functions.
