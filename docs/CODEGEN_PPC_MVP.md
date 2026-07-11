# MVP codegen — IL → PPC big-endian `.text`

Instruction encoding and register-allocation facts for the MVP function
class (straight-line integer add + return). Verified against the live
toolchain with differential probe fixtures this session; COLOR behavior
cross-referenced with `../dc3-decomp/msvc-src/docs/COLOR_RE.md`.

Target for `int add3(int a,int b,int c){return a+b+c;}` — 12 bytes,
big-endian:

```
7d632214   add r11,r3,r4
7c6b2a14   add r3,r11,r5
4e800020   blr
```

## Instruction encoding

PPC instructions are fixed 32-bit words, stored **big-endian** in `.text`
(unlike the COFF struct fields, which are little-endian). IBM bit
convention: bit 0 = MSB.

### `add rD, rA, rB` (rD = rA + rB) — primary op 31, XO 266

```
word = (31<<26) | (RD<<21) | (RA<<16) | (RB<<11) | (OE<<10) | (266<<1) | Rc
```

OE=0, Rc=0 for everything c2 emits here (Rc=1 would be `add.`). Verified
bit-exact against both MVP words. `add` is operand-commutative (rA↔rB) —
the ONLY arithmetic op in this family where a swap is licensed (see the
hazard list).

### `blr` — primary op 19 (bclr), BO=20 (always), XO=16

```
(19<<26) | (20<<21) | (16<<1) = 0x4E800020
```

Fixed 4-byte constant; no operands.

## X360 integer ABI

- Args: `r3..r10` left-to-right → `add3(a,b,c)`: a=r3, b=r4, c=r5.
- Return: `r3`.
- Volatile: r0, r3–r12. Callee-saved: r14–r31. r1=SP, r13=reserved
  (small-data/thread), **r12 = reserved volatile scratch** (call glue).
- Leaf function touching only volatiles: **no prologue/epilogue, no stack
  frame** — body then `blr`.

## COLOR scratch order (observed, matches COLOR_RE.md linear-scan model)

Parameters and the return value are pinned to their ABI registers; only
genuine temporaries draw from the allocator. Phase-1 volatile priority is
`r12, r11, r10, …` with **r12 skipped (reserved)**, so:

- First free temp → **r11**, reused across a serial chain.
  Probe: `a+b+c+d` → `add r11,r3,r4; add r11,r11,r5; add r3,r11,r6; blr`.
- Second simultaneous temp → **r10** (descending).
  Probe: `(a+b)*(c+d)` → `add r11,r3,r4; add r10,r5,r6; mullw r3,r11,r10; blr`.

MVP rule: **serial integer temps use r11; the final operation writes r3.**

## Instruction selection (postfix IL → regs)

The `.ex` body is a postfix/stack expression stream (see
`IL_BUNDLE_MVP.md`): `LOAD a, LOAD b, ADD, LOAD c, ADD, RETURN` for
`(a+b)+c` — left-associativity is already encoded.

1. Pre-color params to incoming ABI regs by position; return temp → r3.
2. Walk the stream with an operand stack of physical registers; `LOAD`
   pushes the var's register.
3. `ADD`: pop rhs, pop lhs. Dest = r11 for every add except the last;
   dest = r3 for the final add (the one feeding RETURN). Emit
   `add dest, lhs, rhs`, push dest. Keeping lhs = under, rhs = top
   reproduces c2's rA/rB choice.
4. `RETURN` of a value already in r3 emits only `blr`.

## Integer sub / mul (W2, implemented + gated)

Two more binary ops are in the straight-line class, verified byte-exact:

- **`mul` → `mullw rD,rA,rB`** (op 31, XO 235): `rD = rA*rB`, **commutative**.
  `a*b*c` → `mullw r11,r3,r4 ; mullw r3,r11,r5 ; blr`
  (`7d6321d6 7c6b29d6 4e800020`). Same operand-order freedom as `add`.
- **`sub` → `subf rD,rA,rB`** (op 31, XO 40): `rD = rB − rA`
  (**first register operand is the subtrahend**), **NON-commutative**.
  `a-b-c` → `subf r11,r4,r3 ; subf r3,r5,r11 ; blr`
  (`7d641850 7c655850 4e800020`). To realize source `lhs − rhs` the selector
  emits `subf dest, rhs, lhs` (rA=rhs=subtrahend, rB=lhs=minuend). **Swapping
  rA/rB is a silent sign inversion** — a valid `subf`, just the wrong one,
  invisible to `fuzzy%`. This is exactly the CLAUDE.md correctness-boundary
  hazard, so `encode_subf` is a separate function with the mapping documented
  at its one call site (`select_text`'s `Sub` arm), per the opt-in-encoder
  rule. IL opcodes: `add`=`0x02`, `sub`=`0x03`, `mul`=`0x04` (postfix, each
  pops two operands).

## Integer literals / immediates (W3, implemented)

IL literals are `33 <int-type> <varint>`, where the varint is a **single byte
if `< 0x80`** (the value directly), else **`0x80` + a 4-byte LE i32**
(verified: 5→`05`, 42→`2a`, 200→`80 c8000000`, 70000→`80 70110100`). Codegen
folds a literal operand into an immediate instruction exactly as c2 does — the
selection stack carries `Reg | Imm`:

- **`reg + k`** (either order) → **`addi rD, reg, k`** (`a+5` → `addi r3,r3,5`
  = `38630005`).
- **`reg − k`** → **`addi rD, reg, −k`** — c2 folds the subtraction of a
  constant into an add-immediate with negated value (`a-5` → `addi r3,r3,-5`
  = `3863fffb`). (`k − reg` is `subfic`, not modeled — rejected.)
- **bare `return k`** → **`li rD, k` = `addi rD, r0, k`** (`return 42` →
  `addi r3,r0,42` = `3860002a`; `addi` special-cases rA=0 to the literal 0).

**Wide immediates (W3b, implemented)** — constants outside signed 16-bit:

- **`reg ± wide-K`** → **`addis`+`addi`**, splitting `K` into a
  sign-compensated high half and a sign-extended low half (`lo = (i16)K`,
  `hi = (K − lo) >> 16`, so the `addi`'s sign extension is absorbed). `a+70000`
  → `addis r3,r3,1 ; addi r3,r3,4464`; `a-70000` → `addis r3,r3,-1 ; addi
  r3,r3,-4464`.
- **bare wide `return K`** → the **`lis`+`ori` idiom** `addis rD,r0,hi ; ori
  rD,rD,lo` (unsigned halves, no sign compensation). `return 70000` → `addis
  r3,r0,1 ; ori r3,r3,4464`.

Still out-of-class (rejected, not mis-emitted): **multiply by a constant**
(strength-reduces to shift+add, e.g. `a*3` → `rlwinm r11,r3,1,0,30 ; add
r3,r3,r11`); `const − reg` (`subfic`); a negative wide bare constant.

## W4b2 non-leaf calls — IMPLEMENTED, byte-exact (single-function TU)

`return g(a) + k` (the call result is used, so f is non-leaf) is implemented and
byte-exact for a **single-function TU** (`fixtures/cpp/mvp_framed.cpp`,
`differential_mvp_framed_call_byte_exact`). It needs a `.pdata` unwind section
and three compiler label symbols on top of the tail-call layout. IL detection is
`c2_il::func::parse_framed_call`; codegen is `codegen::framed_call_text`; the COFF
image is `coff::emit_framed_obj` (the 5-section `emit_obj` path is untouched).

**`.text` (size 0x24, verified constant across 1/2/4 callee args — the frame is
always 96 bytes):**
```
7d8802a6  mflr r12
9181fff8  stw  r12,-8(r1)          prologue (3 words): save LR
9421ffa0  stwu r1,-96(r1)          allocate the fixed 96-byte frame
4bfffff5  bl   g                   REL24 reloc at .text+0xC (disp = −0xC, LK=1)
38630001  addi r3,r3,1             the post-call op (here +1); k varies
38210060  addi r1,r1,96            epilogue (4 words): free frame
8181fff8  lwz  r12,-8(r1)          restore saved LR
7d8803a6  mtlr r12
4e800020  blr
```
Prologue (`7d8802a6 9181fff8 9421ffa0`) and epilogue (`38210060 8181fff8
7d8803a6 4e800020`) are byte-constant for this class; only the `addi r3,r3,k`
immediate and the callee vary. `a*5` post-op strength-reduces (`rlwinm`+`add`,
size 0x28) — **out of the `+k` scope, rejected**: `parse_framed_call` accepts
only a literal `33 86 41 74 <varint>` **immediately followed by ADD (`0x02`)**
whose `k` fits a signed-16-bit `addi` (so `*k` = `0x04`, `-k` = `0x03`, and wide
`k` are all rejected → `NotImplemented`, never mis-emitted).

**W4b2-v — acceptance is a positive whole-body parse (honesty claim, scoped).**
The honesty guarantee is precisely this: **every accepted body is exactly one of
three recognized shapes, and every other body is rejected**, because
`c2_il::func::parse_segment` tokenizes the entire `.ex` operand stream (from the
`4C 4F 11` 'LO' marker to the segment end) and accepts only on a *complete*
positive match that *reaches the end* — it is not a search for a favorable
pattern near the first CALL. This is the doctrinal fix for two rounds of the
same over-acceptance: the earlier trio (`parse_body` / `is_tail_call` /
`parse_framed_call`) each matched on a *local* byte neighborhood, so a second
call, a trailing statement, or in-argument arithmetic sat *outside* the window
each gate inspected and was silently dropped. The full grammar (token classes +
the three shape productions) is in `docs/IL_BUNDLE_MVP.md` ("`.ex` whole-body
grammar"). The three accepted shapes are: the straight-line int arithmetic leaf,
the bare terminal void tail call `void f(){ g(); }` (`26 tok` · CALL · `4C 4B` ·
return plumbing), and the framed `return g(a) + k` (`26 tok` · CALL · one
passthrough LOAD · `55 86 41 74` call-end · `4C` · one literal `+ k` · ADD).

The load-bearing boundary inside the framed shape is the `55 86 41 74` call-end
marker: a framed post-op is emitted *after* it, in-argument arithmetic *before*
it. Captured evidence (`.ex` from the `LO` marker):
```
g(a)+1  … 55 86 41 74 | 4c 33 86 41 74 01 02 …   literal+ADD AFTER  the marker → framed +1
g(a+1)  … 33 86 41 74 01 02 | 55 86 41 74 4c 41 … literal+ADD BEFORE the marker → in-arg
```
Because the parse requires the framed argument region to be *exactly* the single
passthrough LOAD (nothing between the CALL token and `55`), `return g(a + 1)` —
a tail call whose `+1` is inside the argument — is rejected, not mis-accepted as
framed `g(a)+1`. It is a legitimate tail call with arg-setup codegen (the
reference emits `addi r3,r3,1 ; b g`), which this MVP does not model (rung
W4b2-iv) → `NotImplemented`, never a mis-emitted obj.

Regression fixtures, each asserted `NotImplemented` in
`differential_out_of_class_call_shapes_not_implemented`: `mvp_call_argframed.cpp`
(`g(a+1)`), `mvp_call_submod.cpp` (`g(a)-1`), `mvp_call_mulmod.cpp` (`g(a)*5`),
`mvp_call_widemod.cpp` (`g(a)+70000`), and the W4b2-v probes
`mvp_call_twice.cpp` (`g();g();`), `mvp_call_then_stmt.cpp` (`g();return a+1;`),
`mvp_call_argframed_plusk.cpp` (`g(a+1)+1`), `mvp_call_two_framed.cpp`
(`g(a)+g(a+1)`), `mvp_call_plus1plus2.cpp` (`g(a)+1+2`). The two mis-emits the
old gates produced — a bare `b g` that dropped a second call/statement, and a
framed obj that dropped in-argument work — are now impossible by construction:
the parse would not reach the segment end.

**`.pdata` unwind word — RESOLVED: it encodes function length.** The 8-byte
RUNTIME_FUNCTION is `BeginAddress(u32=0, reloc-patched)` + a packed unwind word,
both **big-endian** (like `.text`). Diffing the 0x24-byte `+k` body against the
0x28-byte `*5` body gave `40000903` vs `40000A03` (Δ = 0x100 for +1 word), so:

```
unwind = 0x40000000 | (function_length_words << 8) | prolog_length_words
```

with `function_length_words = text_len/4` and `prolog_length_words = 3` (the
`mflr;stw;stwu` prologue). For the `+k` class the body is always 9 words →
constant `0x40000903`. `build_pdata` computes it from the text length rather than
hardcoding the word. Section characteristics `0x40400040` (CNT_INIT_DATA |
ALIGN_8 | MEM_READ). One reloc: `va=0, symidx=?f, type 0x2 (ADDR32)`. The
`.pdata` aux section-def carries a **real reflected-CRC-32 CheckSum** of its raw
bytes (`0xd3dfb2ce` for the `+k` frame) — a non-COMDAT section that nonetheless
gets a checksum, unlike the leaf `.text`/`.drectve`/`.debug$S` (which store 0).

**Relocations + symbol layout** — see `OBJ_FORMAT_MVP.md` "6-section framed-call
variant". Key facts: 6 sections, 20 symbols; the `bl` REL24 targets the external
`?g` (not a `$M` label); the label symbols are `$M2545` (val=.text+0xC, the `bl`,
class 6), `$M2546` (val=.text end, class 6), `$T2547` (in `.pdata`, class 3); and
the file layout **interleaves** each reloc'd section's raw+reloc (`.text` raw,
`.text` reloc, `.pdata` raw, `.pdata` reloc) rather than packing all raw then all
relocs.

**Counter-determinism (W-UNW-1, RESOLVED):** a single non-leaf function always
emits the *constant* labels `$M2545 / $M2546 / $T2547`, verified identical across
reruns, filenames, and symbol names. The counter only shifts when **preceding
functions in the TU consume slots** (a leaf before `f` bumps the base to
`2549/2550/2551`). So the labels are a fixed toolchain seed and are hardcoded;
`parse_framed_call` is scoped to a single-function TU (a multi-function TU with a
framed call is rejected — modeling the per-function counter increment is a
later rung).

## Non-commutative hazard list — do NOT generalize the MVP encoder

These are load-bearing operand orders; a swap is a silent, fuzzy-invisible
corruption (see decomp-synth CLAUDE.md correctness boundary). Gate each
behind explicit opt-in when implemented:

- **`subf` (op 31, XO 40): `subf rD,rA,rB` computes rB − rA — reversed.**
  Probe verified: `a-b-c` → `subf r11,r4,r3; subf r3,r5,r11; blr`
  (minuend→rB, subtrahend→rA). Immediate forms differ again.
- Shifts `slw`/`sraw`/`srw` (XO 24/792/536): fixed operand order;
  signedness selects arithmetic vs logical right shift; by-constant uses
  `rlwinm`.
- Compares `cmpw`/`cmplw`: signed vs unsigned differ; direction not
  swappable; results feed boolean-materialization sequences.
- `mullw` (XO 235) is commutative like `add`.

MVP generator stays restricted to commutative `add` + fixed `blr`.
