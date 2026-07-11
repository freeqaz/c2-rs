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

## W4b2 non-leaf calls — SCOUTED, deferred (bigger than a frame)

`return g(...) + k` (the call result is used, so f is non-leaf) was scouted
end-to-end and is a **substantially larger rung than the tail call** — it
needs a `.pdata` unwind section and compiler-counter label symbols, so it was
not implemented in the W4 pass. Full anatomy (for the eventual implementation):

**`.text` (size 0x24, verified constant across 1/2/4 callee args — the frame is
always 96 bytes):**
```
7d8802a6  mflr r12
9181fff8  stw  r12,-8(r1)          prologue: save LR
9421ffa0  stwu r1,-96(r1)          allocate the fixed 96-byte frame
4bfffff5  bl   g                   REL24 reloc at .text+0xC (disp = −0xC, LK=1)
38630001  addi r3,r3,1             the post-call op (here +1); *k varies
38210060  addi r1,r1,96            epilogue: free frame
8181fff8  lwz  r12,-8(r1)          restore saved LR
7d8803a6  mtlr r12
4e800020  blr
```
Prologue (`7d8802a6 9181fff8 9421ffa0`) and epilogue (`38210060 8181fff8
7d8803a6 4e800020`) are byte-constant for this class; only the post-call op
(and the callee) vary. `a*5` post-op strength-reduces (`rlwinm`+`add`, size
0x28) — out of the `+k` scope.

**Extra sections/symbols that make it hard (this is why it's deferred):**
- A **`.pdata` section** appears (so `NumberOfSections` = 6, not 5;
  `NumberOfSymbols` = 20). 8 bytes: `00000000 40000903` = a RUNTIME_FUNCTION
  (BeginAddress RVA=0 patched by a reloc; second word = packed X360 PPC unwind
  info — encodes prolog/function length, so it varies per frame).
  Characteristics `0x40400040`. One reloc: va=0, symidx=(the function),
  **type `0x2` (ADDR32)** — a new relocation type.
- **Compiler-generated label symbols** with monotonic counter names:
  `$M2545` (val = .text+0xC, the `bl`), `$M2546` (val = .text end), `$T2547`
  (in `.pdata`), all storage-class 6 (LABEL) / 3. The `2545/2546/2547`
  counters are c2-internal and must be reproduced exactly for byte-equality —
  the real blocker. Whether they are deterministic for a single-function TU
  (and how the counter is seeded) is the open question to crack first.

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
