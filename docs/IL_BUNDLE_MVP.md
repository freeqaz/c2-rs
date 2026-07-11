# MVP IL bundle — what c2-il parses and what the emitter consumes

The c1xx→c2 IL bundle for the MVP function class: capture, per-file parse,
and the (surprisingly short) list of bundle-derived facts the emitter needs.
Grammar cross-checked against the reference decoder
`../dc3-decomp/msvc-src/tools/il_parser.py` and
`../dc3-decomp/msvc-src/docs/IL_FORMAT.md`.

## Capture

`/Bd /d2nop` makes c2 abort on the bogus `-nop` flag *after* c1xx has
written the bundle but *before* cl deletes it. Bundle = `_CL_<hash>` base ×
5 suffixes `ex gl sy in db` (no dot). Implemented as
`c2-reference::Toolchain::capture_il` (scrapes `-il <base>` from the `/Bd`
argv echo). add3 sizes: ex=2765, gl=220, sy=64, in=439, db=71.

Alternative full-fidelity capture (bundle + exact c2 argv + reference obj in
one run): `strace -e inject=unlink,unlinkat:retval=0` + `/Bd` — the P0.1
recipe in `c2-reference::capture_reference`.

## What the emitter needs from the bundle (MVP) — three facts

Everything else in the 790-byte obj is a fixture constant
(`OBJ_FORMAT_MVP.md`). Verified by string/byte search of the obj:

| Obj feature | Source |
|---|---|
| Mangled name `?add3@@YAHHHH@Z` (function symbol + strtab) | `.gl` |
| `.text` instructions (2 adds → 3 instrs) | `.ex` body |
| S_OBJNAME path in `.debug$S` | **c2 argv `-Fo` — NOT the bundle** |

**The source path from `.gl` does NOT appear in the obj** (at `/Ox`,
no `/Zi`) — only the `-Fo` obj path does. So `obj = f(IL, -Fo, flags)` for
this class; the `-f` source string is parsed for provenance only. (Earlier
P0.1 notes assumed `-f` was also embedded — corrected here.)

## Token width — do not use a size heuristic

Tokens in `.sy`/`.ex` are 2 bytes in these bundles (e.g. `e3 09`). The
correct detector (mirrors `il_parser._detect_token_width`): find the first
`4F 02` in `.ex`, count bytes to the next `4F` — gap 2 → width 2. A
bundle-size heuristic misclassifies (the original `c2-il::token_width`
guessed 4 for ≥512 B bundles — wrong for add3's 2765-byte `.ex`).

## `.gl` — mangled name + source path

Two NUL-terminated ASCII fields, located by content:

- **Mangled name**: scan for `0x3F` (`'?'`), read to NUL; accept iff
  `name[1]` is alphabetic and `@@` occurs. c1xx produces it; c2 copies the
  bytes verbatim into the obj — never re-mangle. The name alone yields the
  signature (`?add3@@YA H HHH @Z` = cdecl, ret int, 3×int), enough to fix
  param count/types for the MVP without touching `.in`.
- **Source path**: pattern `[a-z]:\...\*.cpp` NUL-terminated.

`.gl` also carries `.XBLD$W`, `__C1_11886`, `/include:__C1_11886` strings —
treated as emitter constants, not extracted.

## `.sy` — token → name binding

Records shaped `… <token> 00 <name> 00 …`. add3: `a`→`e309`, `b`→`e409`,
`c`→`e509`; the return temp (`e709`) has no `.sy` entry. Needed only to
bind IL body tokens to parameter positions.

## `.in` / `.db` — skippable for the MVP

`.in` is a type/declaration import table; the only MVP type (`int`) is
already inline in `.ex` as `86 41 74`. `.db` is debug/line data, unused at
`/Ox` without `/Zi`. Parse both later.

## `.ex` — function body

Header magic `5B 80 54 0A`, then zero-fill to **0x0A54** where the module
stream begins. Annotated add3 body:

```
4F 1F 80 05 00 A0 00        function start marker
4F 20 80 FE 00              function descriptor
4F 33 0D 66 12 1C …         function metadata (opaque; skip)
42 45 …                     'BE' block entry
0F 4F 02 20 00 4F 01 01     body-marker start; trailing 01 = fn index
53 53                       'SS' start statement
26 e6 09                    result-var token
46                          'F' formal-params marker
2D e5 09  2D e4 09  2D e3 09    params c,b,a (0x2D-separated)
4C 4F 11 53                 'LO' load-operands + 'S'
B9 e3 09 86 41 74           LOAD a   (token, type int = 86 41 74)
B9 e4 09 86 41 74           LOAD b
02                          ADD              → a+b
B9 e5 09 86 41 74           LOAD c
02                          ADD              → (a+b)+c
41 86 41 74                 result-type annotation (int)
3A e7 09                    ASSIGN → return temp
54 02 29 e7 09              RETURN temp
4F 12  47 54 01 54 00       separator + 'GT' terminate
4F 02 20 00 4F 01 02 4D     module end ('M')
```

**The stream is postfix**: `(a+b)+c` arrives as
`LOAD a, LOAD b, ADD, LOAD c, ADD` — each `0x02` ADD pops two, pushes one.
Minimal parser: skip to `SS`, read result token after `26`, read
`2D`-separated formals after `46`, then from `4C 4F 11` run an
operand/opcode reader until `54 02` (return).

Opcode map used (from IL_FORMAT.md, confirmed on fixtures): `0x02` ADD ·
`B9 <tok> <type>` load · `33 <type> NN` literal · `41 <type>` result-type ·
`3A <tok>` assign · `54 02 29 <tok>` return. Type `86 41 74` = int.

## `.ex` whole-body grammar — positive parse (W4b2-v, verified)

`c2_il::func::parse_segment` is a **positive whole-body parser**: it tokenizes
a function's `.ex` operand stream from the `4C 4F 11` ('LO') marker to the end
of the segment (segments are split at each `4F 1F` function-start) and accepts
**only** if the *complete* token sequence is exactly one of the three modeled
shapes below. The parse must *reach the segment end*; any unmodeled byte, a
second call, computation after a terminal call, or a non-trivial call-argument
region fails the whole function closed (`None` → the emitter reports
`NotImplemented`, never a mis-emit). It replaced three earlier gates
(`parse_body` / `is_tail_call` / `parse_framed_call`) that each matched on a
*local* byte neighborhood around the first CALL and so silently over-accepted
trailing / in-argument work. All byte facts below are transcribed from live
16.00.11886.00 captures (`/Bd /d2nop /Ox /GS- /c`) of every fixture + probe.

Token classes (each consumed by an exact-pattern match or a typed read):

| Token | Bytes | Notes |
|---|---|---|
| LO marker | `4C 4F 11` | body start (unique in the segment) |
| SS statement-start | `53` | follows LO |
| statement/label marker | `4F 01 NN` | multi-fn only; after SS and before RETURN |
| function/result ref | `26 <tok>` | precedes a CALL |
| CALL token | `BD <3-byte ret type> 00 80 01 10 00 00` | fixed 10 bytes; type void=`82 07 03`, int=`86 41 74` |
| LOAD | `B9 <tok> 86 41 74` | int operand |
| LITERAL | `33 86 41 74 <varint>` | `<0x80`=value, else `0x80`+4-byte LE i32 |
| ADD / SUB / MUL | `02` / `03` / `04` | postfix binary |
| int call-end | `55 86 41 74` then `4C` | consumed-value call, then an `L` marker |
| void call-end | `4C 4B` | discarded-value call (`L` `K`) |
| result-type | `41 86 41 74` | present for an int return, absent for void |
| ASSIGN | `3A <tok>` | to the return temp |
| RETURN | `54 02 29 <tok>` | |
| function tail | `4F 12` · `47 54 01 54 00` | separator + `GT` terminate |
| module end | `4F 02 20 00` · `4F 01 NN` · `4D` | last function only, then zero-fill |

The three accepted shapes (`INT` = `86 41 74`):

```
body   := LO SS  stmt?  ( arith | vcall | fcall )
stmt   := 4F 01 NN
arith  := (LOAD | LIT | 02|03|04)+  ret_int         # straight-line leaf
vcall  := 26 tok  CALL  4C 4B  ret_void             # void f(){ g(); }
fcall  := 26 tok  CALL  LOAD  55 INT 4C  33 INT k 02  ret_int   # return g(a)+k
ret_int  := 41 INT  3A tok  stmt?  54 02 29 tok  TAIL
ret_void :=         3A tok  stmt?  54 02 29 tok  TAIL
TAIL   := 4F 12  47 54 01 54 00  ( <segment-end> | 4F 02 20 00 4F 01 NN 4D 00* )
```

A non-last function's segment ends exactly at `47 54 01 54 00` (the split cuts
before the next `4F 1F`); the last function carries the module end + zero-fill.
The framed `fcall` argument region is **exactly the single passthrough LOAD**;
the post-op is **exactly one** literal `+ k` (ADD, commutative) whose `k` fits a
signed-16-bit `addi`. Fail-closed points, each a real defect this parse now
rejects (all previously loud mis-emits):

- **in-argument arithmetic** — `g(a + 1)` / `g(a + 1) + 1`: the arg region
  carries LIT+ADD before `55`, so it is not the bare passthrough LOAD → reject
  (a post-`55`-only search mis-read it as framed `g(a)+1`, dropping the in-arg
  work). Captured boundary evidence:
  `g(a)+1` → `… 55 86 41 74 | 4c 33 86 41 74 01 02 …` (post-op AFTER `55`);
  `g(a+1)` → `… 33 86 41 74 01 02 | 55 86 41 74 4c 41 …` (in-arg BEFORE `55`).
- **a second call** — `g(); g();` (a `26 … BD` where the void return plumbing
  must be) / `g(a) + g(a+1)` (a second `26 … BD` where the framed post-op must
  be) → reject.
- **a statement after a terminal call** — `g(); return a+1;` (a `B9` LOAD after
  the void `4C 4B`) → reject.
- **a two-literal post-op** — `g(a) + 1 + 2` (a second `33 …` where the
  result-type must be) → reject.
- **non-commutative / strength-reduced / wide post-op** — `g(a)-1` (`03`),
  `g(a)*5` (`04`), `g(a)+70000` (wide `k`) → reject.

## Out of MVP scope (needed later)

- `.ex` beyond the three shapes above: branches/labels (`38`, `54 03/04`),
  call *argument-setup* codegen (`return g(a+1)` — a tail call with a computed
  arg, rung W4b2-iv), casts (`2C`), memory (`30/32`), switch (`3B–3D`), and any
  call in a multi-function TU (the `.pdata` label counters shift — W-UNW-1).
  The positive parse already *tokenizes* enough to recognize and honestly
  reject all of these; implementing them means adding an accepted shape, not
  loosening a gate.
- `.in` type-table decode (first non-int/pointer/struct type).
- `.db` line tables (non-`/Ox` debug CV).
- The `.ex` header/index region (0x08–0x0A54, treated as opaque here) —
  must be modeled before *writing* IL rather than reading it.

## K1/K2a lossless codec — decoded-vs-opaque coverage map

`c2_il::codec` (`IlModel::parse`/`encode`) is the **round-trip-gated container
codec**: it decodes the classes above into typed islands and keeps everything
else as opaque byte spans, with the invariant `encode(parse(b)) == b`
byte-for-byte or a fail-closed `CodecError::CannotRoundTrip` (`parse` re-encodes
each file and compares before returning — a decode bug is an error, never a
silent corruption). Verified byte-identical over the **full 19-fixture spread**
(`crates/c2-harness/tests/il_roundtrip.rs`, toolchain-gated; captures live
because the `.gl` embeds the host source path so bundles are not committable) +
hand-built always-on unit tests (`codec.rs`). **K2a** decoded the `.ex`
per-function metadata prefix and firmed up the `.gl` offset identification,
roughly **doubling typed coverage: ~2.8% → ~5.5%** of bundle bytes over the
10-fixture capture set (~94.5% still opaque = the remaining K2 backlog; the
`.ex` header/index + `.in`/`.sy`/`.db` dominate what is left).

**Typed (decoded) today:**

- `.ex` operand stream, per function, from the `4C 4F 11` 'LO' marker to the
  segment end: every token in the table above — LO/SS, `4F 01 NN` stmt markers,
  `26` result-refs, the 10-byte CALL, LOAD, narrow+wide LITERAL (the `wide` flag
  is preserved, so a P0.6a length-pad `80 05 00 00 00` re-encodes exactly),
  ADD/SUB/MUL, int/void call-ends, result-type, ASSIGN, RETURN, function-tail,
  module-end. Unmodeled body bytes (comparison `24`, shift `09`, ternary `43 42`,
  …) stay opaque, so out-of-class bodies still round-trip.
- `.ex` per-function **metadata prefix** (`4F 1F` … 'LO'), **NEW in K2a**, decoded
  greedily into: `FnHeader` — the fixed header preamble (`4F 1F`/`4F 20`
  descriptors, the length-prefixed `4F 33 <len>` metadata record, the `42 45`
  'BE' block-entry, trailing `0F`), byte-identical across every captured function,
  recognized as a **bounded island** (start `4F 1F` and end at the block-start are
  structural; its interior sub-records are captured verbatim, not yet
  field-typed — a further K2 shrink); `BlockStart(NN)` — the `4F 02 20 00 4F 01
  NN` block-start marker (NN = per-function block index); the `53 53` SS·SS; the
  `26 <tok>` result-ref; and the `46 (2D <tok>)*` formal list. For the add3 class
  the prefix is fully typed (no opaque residue).
- `.gl` `80 <LE32>` **body-start offset field(s)** — one per function, typed as
  `Span::GlOffset(u32)` (first-class, the K3 rewrite target). **K2a** locates each
  field **by its record framing** (`80 XX 10 00 00 00 00` immediately precedes it
  — a position-based decode, verified ahead of every offset field), then gates it
  fail-closed: the framed offsets, in `.gl` order, must equal the `.ex` `4F 1F`
  offsets 1:1 and in function order, or none are typed (a coincidental value
  collision or a missed record hands K3 nothing rather than a false site). This
  replaced the K1 value-membership heuristic. `IlModel::ex_function_count()` and
  the harness gate now assert `gl_body_start_offsets().len() == ex_function_count`.

**Opaque today (remaining K2 decode backlog), in rough priority order:**

- `.ex` **header/index** region `0x00–0x0A54` (magic + zero-fill) — now the single
  largest opaque chunk (must be modeled before *writing* IL).
- the `FnHeader` **interior** sub-records (`4F 20` descriptor, `4F 33` metadata
  payload, `42 45` block-entry body) — captured as a bounded island but not yet
  field-typed; a small, low-priority shrink (fixed constant for this class).
- the rest of `.gl`: mangled names, source path, `.XBLD$W`/`__C1_*` strings, and
  the **per-function record framing** around the offset field (K2a decodes the
  offset field and its 7-byte frame anchor, not the whole record). Full `.gl`
  record framing would remove even the residual GlOffset collision risk.
- `.sy` token→name symbol table, `.in` type-import table (a constant 439 B here),
  `.db` debug/line data — each a single opaque span.

**K3 inherits:** `GlOffset` is round-tripped unchanged by K1/K2a; rewriting it on
an `.ex` length change (per P0.6a) is K3's job. The codec exposes the field typed,
structurally located, and gated 1:1 with the function count, so K3 edits a `u32`,
not a byte hunt. K3 also now has the **structural body extent**: each function's
metadata prefix is fully delimited (FnHeader → BlockStart → SS·SS → result-ref →
formals → LO body), so the body EXTENT is `4F 1F`-marker-to-next-`4F 1F` with the
fixed-size header carved out — a length edit touches only the operand stream and
the `.gl` offset, never the (constant) header. The FnHeader interior stays opaque
but is length-invariant, so it does not block a length rewrite.

**K3a is now implemented** — the length-consistent edit primitive (literal
widen/narrow, term insert/delete) that rewrites the `.gl` offset column on an
`.ex` length change, gated byte-exact against the live oracle. Scope, API, the
fail-closed boundary (whole-function add/remove is K3b), and the differential
edit gate are documented in `EDIT_MODEL_MVP.md`.
