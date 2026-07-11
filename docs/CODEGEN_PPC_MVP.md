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
