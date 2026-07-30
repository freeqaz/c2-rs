# The three uncharacterized `/Gy` shapes — byte-level characterization

Read-only characterization of the three COMDAT shapes that refuse under `/Gy`
(`docs/OPT_MODE.md` §4.1 item 3): `_fltused` placement, pooled `.rdata` FP
constants, and the framed non-leaf call. Every table below is transcribed from a
reference obj produced by the real toolchain (`cl.exe` 16.00.11886.00 under
wibo); nothing is computed or inferred unless explicitly labeled as a model or
hypothesis. Probe sources lived in `/tmp` and are quoted inline; fixture TUs are
named. Flags were `/O1 /GS- /c` ("`/O1`"), `/Ox /GS- /c` ("packed `/Ox`"), or
`/Ox /Gy /GS- /c` where the two axes needed separating.

Lane state when captured (`scripts/mode_lane.sh`, 89 fixtures):

| mode | match | mismatch | codegen-gap | vocab-gap |
|---|---|---|---|---|
| `/O1` | 25 | 0 | 7 | 57 |
| `/Ox /Gy` | 27 | 0 | 5 | 57 |

The `/Gy` refusals are 3× float-under-`/Gy`, 1× framed-under-`/Gy` (both
lanes), plus the two `/O1` comparison spines and one repeated-leaf reduction.

Common to every `/Gy` obj observed: the four-section shell (`.drectve`,
`.debug$S`, 2× `.XBLD$W`) is unchanged; each function's `.text` COMDAT has
`Characteristics = 0x60401020` (packed `0x60400020` + `LNK_COMDAT 0x1000`),
section-symbol aux `CheckSum = 0`, `Number = 0`, `Selection = 1`
(NODUPLICATES); every function symbol has `Value = 0` (it owns its section);
raw data, per-section relocations, and the next section's raw data are packed
contiguously with **no alignment padding in the file** (e.g. an 8-byte `.pdata`
starting at odd file offset 0x26e).

---

## 1. `_fltused` in the per-function COMDAT symbol order

### 1.1 Captures

All at `/O1`. Only the trailing symbols are shown; indices 0–10 are the
constant `@comp.id` + shell groups in every obj. "grp(F)" abbreviates the pair
`.text` section symbol (STATIC, 1 aux, Selection=1) followed by F's function
symbol (EXTERNAL, type 0x0020, Value 0).

`fixtures/cpp/mvp_fmul3.cpp` — one float function (nsym=15):

| idx | name | sec | Value | sc | type | naux |
|---|---|---|---|---|---|---|
| 11 | `.text` | 5 | 0 | 3 | 0x0000 | 1 |
| 13 | `?fmul3@@YAMMMM@Z` | 5 | 0 | 2 | 0x0020 | 0 |
| 14 | `_fltused` | 0 | 0 | 2 | 0x0020 | 0 |

`float fa(a,b){a*b;} float fb(a,b){a+b;}` (v_f2, nsym=18):
grp(fa)=11/13, **`_fltused`=14**, grp(fb)=15/17.

`int ia(...); float fa(...)` (v_if, nsym=18):
grp(ia)=11/13, grp(fa)=14/16, **`_fltused`=17** (last).

`float fa(...); int ia(...)` (v_fi, nsym=18):
grp(fa)=11/13, **`_fltused`=14**, grp(ia)=15/17.

`float fa; int ia; float fb` (v_fif, nsym=21):
grp(fa)=11/13, **`_fltused`=14**, grp(ia)=15/17, grp(fb)=18/20.

`int ia; int ib; float fa` (v_iif, nsym=21):
grp(ia)=11/13, grp(ib)=14/16, grp(fa)=17/19, **`_fltused`=20** (last).

`float g2(float); float fa(a){return g2(a);} int ia(...)` (v_ftail, nsym=19):
grp(fa)=11/13, `?g2@@YAMM@Z` (sec 0, undefined)=14, **`_fltused`=15**,
grp(ia)=16/18. The callee precedes `_fltused`.

`float k(a){return a+1.0f;} int g(int); int f(a){return g(a)+1;}`
(v_fconst_framed, nsym=27): grp(k)=11/13, `.rdata`+aux=14,
`__real@3f800000`=16, **`_fltused`=17**, then f's framed group (§3). The
`.rdata` pair precedes `_fltused`.

In every capture `_fltused` is: SectionNumber 0, Value 0, StorageClass 2,
type 0x0020, 0 aux, exactly one per obj, name via the string table.

### 1.2 The rule

> **`_fltused` is emitted immediately after the complete symbol group of the
> first floating-point function — its `.text` section symbol + aux, its
> function symbol, any callee externals it introduced, and any
> `.rdata`/`__real@` pairs it introduced — and before the next function's
> `.text` section symbol.** This is the same rule as the packed layout
> (`CODEGEN_W13_FLOAT.md` §4/§5.2); `/Gy` does not move it. Int functions
> before the first float function are unaffected; if the first float function
> is the last function, `_fltused` is the last symbol.

### 1.3 Not determined

* Which functions count as "floating-point" for placement is inherited from
  the packed characterization, including its open trigger question
  (`CODEGEN_W13_FLOAT.md` §4: an `fctiwz`-executing int function is a known
  negative). Not re-probed under `/Gy`.
* A float function whose *callee list* is introduced by relocations to
  multiple new externals: only the single-callee case (v_ftail) is captured.

---

## 2. Pooled `.rdata` FP constants under `/Gy`

### 2.1 `fixtures/cpp/w13b_fconst.cpp` at `/O1` (861→1017 B, nsec=6, nsym=18)

Section table:

| # | name | RawSize | RawPtr | RelPtr | nRel | Chars |
|---|---|---|---|---|---|---|
| 1 | .drectve | 132 | 0x104 | 0 | 0 | 0x00100a00 |
| 2 | .debug$S | 152 | 0x188 | 0 | 0 | 0x42100040 |
| 3 | .XBLD$W | 16 | 0x220 | 0 | 0 | 0xc0401040 |
| 4 | .XBLD$W | 16 | 0x230 | 0 | 0 | 0xc2301040 |
| 5 | .text | 16 | 0x240 | 0x250 | 4 | 0x60401020 |
| 6 | .rdata | 4 | 0x278 | 0 | 0 | 0x40301040 |

Symbols 11+: `.text`+aux (Len=16, nRel=4, Sel=1), `?k_add@@YAMM@Z`,
`.rdata`+aux (Len=4, CheckSum=0, Sel=2), `__real@3f800000` (type 0x0000),
`_fltused`. `.text` relocs: REFHI(0x10)+PAIR(0x12) at VA 0, REFLO(0x11)+PAIR
at VA 4, targets symidx 16 (`__real@`), PAIR index field 0. `.rdata` raw =
`3f800000` (big-endian IEEE bits).

### 2.2 `fixtures/cpp/w13b_fdedup.cpp` at `/O1` (nsec=11, nsym=33) — the layout witness

`ka`=`a+1.0f`, `kb`=`a+2.0f`, `kc`=`a+1.0f`, `kd`=`double a+1.0`.

| # | name | RawSize | RawPtr | RelPtr | nRel | Chars | raw contents |
|---|---|---|---|---|---|---|---|
| 5 | .text | 16 | 0x308 | 0x318 | 4 | 0x60401020 | `3d600000 c00b0000 ec21002a 4e800020` |
| 6 | .rdata | 4 | 0x340 | 0 | 0 | 0x40301040 | `3f800000` |
| 7 | .text | 16 | 0x344 | 0x354 | 4 | 0x60401020 | (kb) |
| 8 | .rdata | 4 | 0x37c | 0 | 0 | 0x40301040 | `40000000` |
| 9 | .text | 16 | 0x380 | 0x390 | 4 | 0x60401020 | (kc) |
| 10 | .text | 16 | 0x3b8 | 0x3c8 | 4 | 0x60401020 | (kd, `c80b…fc21…`) |
| 11 | .rdata | 8 | 0x3f0 | 0 | 0 | 0x40401040 | `3ff00000 00000000` |

Symbol table 11+ (indices matter — relocations name them):

| idx | name | sec | sc | type | naux |
|---|---|---|---|---|---|
| 11 | `.text`+aux | 5 | 3 | | 1 |
| 13 | `?ka@@YAMM@Z` | 5 | 2 | 0x0020 | 0 |
| 14 | `.rdata`+aux (Sel=2) | 6 | 3 | | 1 |
| 16 | `__real@3f800000` | 6 | 2 | 0x0000 | 0 |
| 17 | `_fltused` | 0 | 2 | 0x0020 | 0 |
| 18 | `.text`+aux | 7 | 3 | | 1 |
| 20 | `?kb@@YAMM@Z` | 7 | 2 | 0x0020 | 0 |
| 21 | `.rdata`+aux (Sel=2) | 8 | 3 | | 1 |
| 23 | `__real@40000000` | 8 | 2 | 0x0000 | 0 |
| 24 | `.text`+aux | 9 | 3 | | 1 |
| 26 | `?kc@@YAMM@Z` | 9 | 2 | 0x0020 | 0 |
| 27 | `.text`+aux | 10 | 3 | | 1 |
| 29 | `?kd@@YANN@Z` | 10 | 2 | 0x0020 | 0 |
| 30 | `.rdata`+aux (Len=8, Sel=2) | 11 | 3 | | 1 |
| 32 | `__real@3ff0000000000000` | 11 | 2 | 0x0000 | 0 |

kc's `.text` (section 9) relocates against symidx **16** — the dedup'd
constant produces **one** section and one symbol pair, referenced across
functions. Each section's 4 relocation records sit immediately after its own
raw data (RelPtr = RawPtr + RawSize throughout).

`w13b_fpool.cpp` at `/O1` is the same shape: `.text(ke,20B)`,
`.rdata __real@40c00000`, `_fltused` after ke's group, `.text(kdiv,16B)`,
`.rdata __real@3d430c31`. ke's REFHI is at VA 4 (the `fmuls` precedes the
`addis`), REFLO at VA 8 — reloc records sorted ascending by VA.

### 2.3 Two+ constants introduced by ONE function — the pool order reverses

`float p1(float a,float b){return (a+1.0f)-(b+2.0f);}` followed by
`float q(float a){return a+1.0f;}` (v_p1, `/O1`): sections are
`.text(p1,32B)`, `.rdata __real@40000000`, `.rdata __real@3f800000`,
`.text(q,16B)`. p1's code references 3f800000 first (REFHI VA 0 → symidx 19,
REFHI VA 4 → symidx 16 = 40000000); q relocates to symidx 19 (dedup back into
p1's pool). Symbol order matches section order; `_fltused` follows *all* of
p1's `.rdata` pairs.

Three constants, reference order 1.0f, 2.0f, 3.0f (v_p3): sections emerge
`40400000`, `40000000`, `3f800000`. Reference order 3.0f, 1.0f, 2.0f (v_p3r):
sections emerge `40000000`, `3f800000`, `40400000`. Both are the exact
**reverse of first-reference order**, which discriminates LIFO from
descending-bit-pattern order (v_p3 alone is consistent with both; v_p3r
refutes value order).

### 2.4 The rules

> 1. **Interleaved, not grouped:** each `.rdata` COMDAT (and its
>    section-symbol + `__real@` pair) is emitted immediately after the `.text`
>    COMDAT (and symbol group) of the function that first references it.
>    `.text` and `.rdata` sections interleave in the section table in that
>    order.
> 2. **One section per distinct `(bit-pattern, width)` TU-wide**, exactly as
>    packed; later functions relocate against the existing symbol index.
> 3. **Within one introducing function, multiple new constants are appended in
>    reverse first-reference order (LIFO)** — sections and symbol pairs both.
>    This is out of the port's current one-constant class but is what real
>    `/O1` TUs will contain.
> 4. Everything else is byte-identical to the packed characterization:
>    `.rdata` chars `0x40301040` (float) / `0x40401040` (double), aux
>    Selection=2 / CheckSum=0, `__real@` type 0x0000, four relocs per
>    reference site (REFHI+PAIR, REFLO+PAIR, PAIR index field 0), records
>    sorted ascending by VA, each section's relocs directly after its own raw
>    data, no inter-section padding.

### 2.5 Not determined

* Whether rule 3's "reverse first-reference order" is literally reference
  order or reverse *IL-literal* order — in both probes the code referenced the
  constants in IL-literal order, so the two readings coincide. A body whose
  emitted reference order differs from its IL literal order (c2 reschedules —
  `CODEGEN_W13_FLOAT.md` §5.6) would separate them. Not captured.
* A constant first referenced by function A and *also* introduced-by-schedule
  inside a `/Gy` TU whose functions c2 reorders — no reordering was ever
  observed, but nothing here proves section order always equals source
  function order for pathological TUs (templates, static initializers).
* Mixed `.rdata` widths introduced by one function (float + double in one
  body): not captured (needs a conversion, out of class and out of scope).

---

## 3. The framed non-leaf call under `/Gy`

> **Status 2026-07-30: IMPLEMENTED.** Everything in §3 is emitted by
> `c2_core::coff::emit_comdat_obj` (per-COMDAT `.pdata`, `SELECT_ASSOCIATIVE`)
> and `emit_obj` (one shared `.pdata`), with the label numbers derived per §3.5
> rather than hardcoded. Graded byte-exact on `fixtures/cpp/wunw_*.cpp` in all
> four mode lanes and on 342 generated cases.

### 3.1 `fixtures/cpp/mvp_framed.cpp` at `/O1` (nsec=6, nsym=20)

| # | name | RawSize | RawPtr | RelPtr | nRel | Chars |
|---|---|---|---|---|---|---|
| 5 | .text | 36 | 0x240 | 0x264 | 1 | 0x60401020 |
| 6 | .pdata | 8 | 0x26e | 0x276 | 1 | 0x40401040 |

`.text` raw is byte-identical to packed `/Ox` (`7d8802a6 9181fff8 9421ffa0
4bfffff5 38630001 38210060 8181fff8 7d8803a6 4e800020`), REL24 (0x0006) at
VA 0xc → `?g@@YAHH@Z`. `.pdata` raw = `00000000 40000903`, one ADDR32 (0x0002)
at VA 0 → `?f@@YAHH@Z`.

Symbols 11+:

| idx | name | sec | Value | sc | type | naux |
|---|---|---|---|---|---|---|
| 11 | `.text`+aux (Len=36, nRel=1, CheckSum=0, Sel=1) | 5 | 0 | 3 | | 1 |
| 13 | `?f@@YAHH@Z` | 5 | 0 | 2 | 0x0020 | 0 |
| 14 | `$M2549` | 5 | 0x24 | 6 | 0x0000 | 0 |
| 15 | `?g@@YAHH@Z` | 0 | 0 | 2 | 0x0020 | 0 |
| 16 | `$M2548` | 5 | 0x0c | 6 | 0x0000 | 0 |
| 17 | `.pdata`+aux (Len=8, nRel=1, **CheckSum=0xd3dfb2ce, Number=5, Selection=5**) | 6 | 0 | 3 | | 1 |
| 19 | `$T2550` | 6 | 0 | 3 | 0x0000 | 0 |

Same source at packed `/Ox`: identical except labels are
`$M2545/$M2546/$T2547`, `.text` chars `0x60400020`, `.pdata` chars
`0x40400040`, and `.pdata` aux has `Number=0, Selection=0` (same CheckSum).
**`/Ox /Gy` produces exactly the `/O1` values** (`$M2548…`, Sel=5) — the
shift is `/Gy`'s, not the mode's.

### 3.2 The `.pdata` answers

* **One `.pdata` per framed function** under `/Gy`. Two framed functions
  (v_framed2) → sections `.text/.pdata/.text/.pdata`, each `.pdata` 8 bytes
  with its own ADDR32 to its own function. Packed control: one shared 16-byte
  `.pdata` (two entries at Value 0 and 8, two relocs, aux CheckSum
  0x26241231 = the same CRC over the 16 bytes).
* **It is a COMDAT**: chars gain `LNK_COMDAT` (0x40401040), aux
  **Selection = 5 (IMAGE_COMDAT_SELECT_ASSOCIATIVE)**, **Number = the 1-based
  section index of its own function's `.text`** (5, then 7 in v_framed2; f2's
  `.pdata` Number=7).
* The aux **CheckSum is real** (unlike `.text`/`.rdata` COMDATs, whose
  CheckSum is 0): reflected CRC-32, poly 0xEDB88320, init 0, no final
  inversion, over the section raw bytes — `coff_checksum` in
  `crates/c2-core/src/coff.rs` reproduces both `/Gy` captures (0xd3dfb2ce for
  `00000000 40000903`, 0x938adc61 for `00000000 40001205`) and the packed
  shared section (0x26241231). Verified, not new.
* Payload: `BeginAddress=0` + big-endian
  `0x40000000 | (function_words << 8) | prolog_words` — mvp_call_two_framed's
  72-byte body with a 5-word prologue gives `40001205`.
* `$T` label: STATIC (sc 3), Value 0 within its own `.pdata` (packed second
  entry had Value 8 in the shared section).

### 3.3 Symbol group template per framed function

`[.text section sym + aux] [function sym] [$M(n+1) @ function-end offset]
[callee external — only at its first introduction] [$M(n) @ prologue-end
offset] [.pdata section sym + aux] [$T(n+2) @ 0]`.

The `$M` **values** are the prologue end and the function end in `.text`
offsets: mvp_call_two_framed (5-instruction prologue, calls at 0x1c/0x28) has
`$M2548 = 0x14`, `$M2549 = 0x48` — the call site is *not* labeled; the
mvp_framed `0x0c` coincidence (bl at prologue end) misleads. Each function
introduces its own new callee inside its own group (v_framed2c: `?g1` at
idx 15 inside f1's group, `?g2` at idx 24 inside f2's group); a callee already
introduced is simply not re-emitted (v_framed2: f2's group is fn, $M, $M with
no callee between).

### 3.4 The label counters — every captured value

$M(n) = prologue label (lowest), $M(n+1) = end label, $T(n+2). Table lists n.

| TU (function order) | packed `/Ox` | `/Gy` (`/O1` and `/Ox /Gy` agree where both captured) |
|---|---|---|
| mvp_framed: f | 2545 | 2548 |
| mvp_call_two_framed: f (2 calls) | 2545 | 2548 |
| v_framed2: f1, f2 (shared callee) | 2548, 2552 | 2554, 2559 |
| v_framed_leaf: f, leaf | 2549 | 2555 |
| v_leaf_framed: leaf, f | 2550 | 2556 |
| v_llf: leaf, leaf, f | 2555 | 2564 |
| v_float_framed: float-leaf, f | 2550 | 2556 |
| v_lff: leaf, f1, f2 | 2553, 2557 | 2562, 2567 |
| v_fconst_framed: float+const, f | 2552 | 2558 |
| v_framed2c: f1, f2 (distinct callees) | not captured | 2556, 2561 |

**Model — fits all 21 framed-function witnesses above.** Let B(TU) = the packed
first-label value. Then

* packed `/Ox`: labels of the i-th framed function start at B + Σ over
  preceding functions of {leaf: 1, framed: 4};
* `/Gy`: labels start at **B + 3 × (number of functions in the TU)** + Σ over
  preceding functions of {leaf: 1, framed: **5**}.

`.rdata` COMDATs consume no *section* slot of their own. The `/Gy` surcharge is
3 per function regardless of kind, paid up front before any function's labels.
"Per function" and "per `.text` COMDAT" are indistinguishable here — every
function got exactly one COMDAT.

> **Corrected 2026-07-30.** The bracket above read `{leaf **or float-leaf**: 1,
> …}` and the float half is wrong: a floating-point leaf consumes **2**, and one
> with a pooled constant **4**. It could not be seen from this table because B
> was unknown, so `v_float_framed` (float-leaf, f) at 2550 and `v_leaf_framed`
> (leaf, f) at 2550 were read as agreeing when they are two TUs with different
> seeds. With B derivable (§3.5) the stride is measurable one class at a time,
> and §3.6 has it. `.rdata` "consumes nothing" was the same illusion.
> The generalizable form: **a stride and a seed that are both unknown can absorb
> each other's error**, and a table of totals cannot separate them however many
> rows it has — only a witness that pins one of the two can.

### 3.5 The seed B — DETERMINED 2026-07-30

**B is the u32 at `.gl` offset 7, plus 9.**

`.gl` opens with the fixed 7 bytes `11 02 06 '1' 'j' '2' 01`, and the next four
bytes are a little-endian counter — c1xx's compiler-label high-water mark, which
c2 continues. The first label c2 allocates for the TU is that value + 9.

```text
  fixture                 .gl+7 (u32)   first label   observed
  mvp_call_twice              2534         2543        $M2543
  mvp_call_then_stmt          2535         2544        $M2544
  mvp_framed                  2536         2545        $M2545
  il_expr_call_value          2561         2570        $M2571 (one leaf ahead)
  il_call_return              2578         2587        $M2589 (two ahead)
  w5_tree_neg                 2597         2606        $M2616
  w17_ptr_operand_neg         2674         2683        $M2704
```

The whole model, restated as one rule and now implemented in
`c2_core::coff::plan_labels`:

```text
  cur = u32_le(.gl[7..11]) + 9
  if /Gy:  cur += 3 * (number of functions in the TU)
  for each function in .text order:
      framed -> $M(cur), $M(cur+1), $T(cur+2);  cur += 5 if /Gy else 4
      leaf   ->                                 cur += 1
```

**Why the earlier scan missed it.** §3.5 previously recorded "a byte-scan
correlation breaks on FP TUs: max LE16 token in `.ex` vs B gives a constant gap
of 10 for int-only TUs, 11 for `v_float_framed`, 13 for `v_fconst_framed`". Both
halves of that were true and both were the wrong measurement. The counter is not
in `.ex` at all, and the residue the scan was chasing — +1 for a float leaf, +3
for a float leaf with a constant — is not seed noise: it is the **stride**, which
this document had already measured as "leaf or float-leaf: 1" and which is wrong
for a float leaf. Two errors of one and two, in a quantity whose true value moves
by one per function, hidden inside each other.

A second near-miss worth recording: reading `.gl[7..]` as a **LEB128** gives
1256 for `mvp_framed`, and `B − 1256 = 1289` is constant across every fixture
whose fifth byte has the continuation bit set — which is most of them, because
the counter sits around 2500–2700 and `2536 = 0x9E8` LEBs as `E8 09`. The field
is a fixed-width u32; the LEB reading agrees with it on a large majority of the
corpus and disagrees the moment the low byte falls under 0x80. **A constant gap
over a biased sample is not a fit.** Both readings were checked against the same
25 TUs and only the u32 survives on all of them.

### 3.6 What the stride is measured over, and what it is not

Each row is `<class> ; int F(int a){return g(a)+1;}` compiled, `F`'s first label
read out of the obj and differenced against `.gl+7+9`:

| class ahead of the framed function | slots consumed |
|---|---|
| `int L(int a){return a+1;}` (straight-line chain) | 1 |
| `int L(int a,int b,int c){return a+b+c;}` | 1 |
| `void L(){}` (empty body) | 1 |
| `int L(int a){return g(a);}` / `void L(){g(1);}` (tail calls) | 1 |
| `int L(int*p){return *p;}` (indirect load) | 1 |
| `int* L(S*s){return &s->m;}` (address leaf) | 1 |
| `int L(int a){return 7;}` / `return a;` | 1 |
| another framed call | 4 packed, 5 `/Gy` |
| `float L(float a,float b){return a*b;}` | **2** |
| `double L(double a){return a;}` | **2** |
| `float L(float a){return a*2.5f;}` (one pooled constant) | **4** |
| `float L(float a){return a*2.5f+3.5f;}` (two) | **6** |
| `int L(int a,int b){return a<b;}` / `a>=b` | **3** |

`a==b` and `a<0` consume 1; `a<b`, `a>=b` and `bool a<b` consume 3, so the
comparison stride is **not** uniform over the relation either. Every class in
the "1" block is one `PortC2` emits; every class below it is refused whenever a
framed function shares the TU (`c2_il::IlBundle::functions`, with the other
TU-level gates, so the census and the emitter cannot disagree), because a stride
error of one is six wrong bytes in an obj that still links.

**The gate's over-refusal, sized rather than left as a rumour.** It keys on "is
this a comparison or floating-point leaf", not on the relation, so the two
comparison forms that *do* consume 1 (`a==b`, `a<0`) are refused with the ones
that consume 3. On the generated sweep that is **6 of the 21 framed-plus-refuser
cases** — the other 15 need the gate. Relaxing it means measuring the stride per
relation and per operand type, which is a table this rung did not need; the cost
is a refusal, never a wrong byte.

Still not determined, and therefore still refused rather than guessed:

* the stride of every *decoded but not emitted* class — control flow, locals,
  statements. They cannot reach the emitter today, so the gate that stops them
  is the whole-TU class gate rather than this one, and if that gate widens
  first the stride has to be measured before a framed function may sit beside
  them.
* what the 4th packed / 5th `/Gy` framed slot is *for*. The three emitted
  labels are `$M`, `$M`, `$T`; the extra one (two under `/Gy`) is allocated and
  never named in the obj. Modeling it as a stride is sufficient and is all that
  is claimed.
* whether the `/Gy` surcharge is "3 per function" or "3 per `.text` COMDAT" —
  indistinguishable here because every function got exactly one COMDAT. A
  static function, a template instantiation or a function whose first section is
  not `.text` would separate them.

---

## 4. Asymmetries worth flagging

1. **The `.pdata` aux CheckSum is real while `.text`/`.rdata` COMDAT
   checksums are 0**, and `.pdata` is the only Selection=5 COMDAT. Whatever
   c2 uses to decide "checksum or not" is not "is a COMDAT" (`.XBLD$W` is
   Sel=2 *with* a checksum, `.rdata` Sel=2 *without*). The port already
   special-cases this correctly for packed; the `/Gy` values confirm the same
   CRC.
2. **The within-function constant-pool LIFO (§2.3) against the
   across-function FIFO (§2.2)**: first-reference order at function
   granularity, reverse order within a function. Any pooling implementation
   that appends per reference site will silently emit v_p1's two sections
   swapped — indices 16/19 exchange, and every relocation in both `.text`
   sections still *resolves* (the names exist) while the obj is byte-wrong in
   four places.
3. **`/Gy` costs +3 counter slots per function even for functions that emit
   no labels at all.** A TU of 20 leaves and one framed function at the end
   shifts its `$M` numbers by 60 relative to packed. The counters are the
   only place `/Gy` and function *count* interact numerically, and nothing
   in the obj names the mechanism.
4. `/O1` vs `/Ox` never moved a single byte of any table in this document
   once `/Gy` was held fixed — every difference here keys on `/Gy` alone,
   consistent with `OPT_MODE.md` §3.3's two-axes reading.
