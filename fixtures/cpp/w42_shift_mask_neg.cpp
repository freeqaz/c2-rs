// **W42 NEGATIVE** — the shapes next door to `w42_shift_mask.cpp` that must
// stay REFUSED. Every function here censuses `cflow-if-1` exactly like the
// positives do; for each one, emitting the positive lowering would be wrong
// bytes rather than a gap.
//
// ## 1. The MASK COLLAPSE, which is the `!=`->`>` trap in this seam
//
// `(at >> k) & m` where **no bit of `m` survives the shift** is provably zero.
// Real c2 emits `li r4,0` — and, because a literal is not a use of the formal,
// it also **reverts the block layout**: `attrs` is no longer hoisted to r4 in
// the entry block, the `bne` displacement goes from +12 back to +16, and the
// body grows by a word. So the cell differs from its neighbours in TWO ways at
// once, and a fold that only rewrote the instruction would still emit the wrong
// branch.
//
//     (at >> 16) & 0x10       ->  rlwinm r4,r4,16,27,27    hoisted, +12
//     (at >> 16) & 0x10000    ->  li r4,0                  NOT hoisted, +16
//     (at >> 16) & 0xfffffff0 ->  rlwinm r4,r4,16,16,27    hoisted, +12
//
// Measured in **9 of the 70 grid cells** (`work/w-tu1/p/grid_col.cpp`). The port
// refuses rather than folds: the fold is right about the instruction and the
// layout change has one witness shape, so a refusal is a gap and the alternative
// is a plausible wrong branch.
//
// ## 2. The OUT-OF-PLACE fold
//
// `q2`'s then-arm passes a literal where the positive passes `attrs`, so only
// the else-arm uses `attrs` and rule 1 does not fire. Real c2 then **homes the
// source in a scratch first**:
//
//     mr r11,r4 ; mr r10,r5 ; cmplwi cr6,r3,0 ; bne cr6,+16
//     li r4,0 ; mr r3,r11 ; b g2
//     mr r5,r11 ; rlwinm r4,r10,5,28,28 ; b h3
//
// A **second** scratch, at r10. `docs/CODEGEN_W6_COMPARE.md` §6 records that
// register model as demonstrably richer than a descending counter and NOT
// characterized, and `plan_cond_pair` already refuses a second park for exactly
// that reason. `rlwinm r4,r10,…` is `dst != src`, which is the form rule 1b
// refuses by name.
//
// ## 3. The UNMASKED shift and the NON-CONTIGUOUS mask
//
// `at >> k` with no `&` is `srwi`, a different IL production (`0A` with no
// `0B`), and this lane measured no cell of it in a conditional arm. A mask with
// a hole — `0x5` — is not expressible as one `rlwinm` at all: `MB..ME` is a
// single run by construction.
//
// ## 4. The INLINE spelling — same bytes, different IL, refused anyway
//
// `h3(hp, (at >> 0x1b) & 8, sz)` emits **exactly** what `w42_shift_mask.cpp`'s
// `memalloc` emits (checked: byte-identical `.text`). c1xx puts the arithmetic
// in the argument's own operand stream:
//
//     b9 ec 09 <T>  33 <T> 1b  0A  33 <T> 08  0B  55 <T>
//
// and `IlOp` has no `Shr` or `BitAnd`, so `parse_expr` refuses the stream one
// token in. Adding those two variants would widen **every** shape that consumes
// an operand list — the straight-line leaf, the framed call, the store run — on
// the strength of one witness, which is the widening this project keeps paying
// for. Recorded as a boundary with its bytes rather than closed by guessing.
//
// If any function in this file ever censuses in class, the W42 gate has
// over-accepted.

void *g2(unsigned long, unsigned long);
void *h3(void *, unsigned long, unsigned long);

// 1 — the collapse, at three shifts.
void *collapse16(void *hp, unsigned long sz, unsigned long at) {
    if (hp == 0) {
        return g2(sz, at);
    }
    unsigned long f = (at >> 16) & 0x10000u;
    return h3(hp, f, sz);
}
void *collapse31(void *hp, unsigned long sz, unsigned long at) {
    if (hp == 0) {
        return g2(sz, at);
    }
    unsigned long f = (at >> 31) & 0x2u;
    return h3(hp, f, sz);
}

// 2 — the out-of-place fold: the then-arm does NOT want `at`.
void *out_of_place(void *hp, unsigned long sz, unsigned long at) {
    if (hp == 0) {
        return g2(sz, 0);
    }
    unsigned long f = (at >> 0x1b) & 8u;
    return h3(hp, f, sz);
}

// 3a — no mask.
void *unmasked(void *hp, unsigned long sz, unsigned long at) {
    if (hp == 0) {
        return g2(sz, at);
    }
    unsigned long f = at >> 0x1b;
    return h3(hp, f, sz);
}

// 3b — a mask with a hole.
void *holey(void *hp, unsigned long sz, unsigned long at) {
    if (hp == 0) {
        return g2(sz, at);
    }
    unsigned long f = (at >> 4) & 0x5u;
    return h3(hp, f, sz);
}

// 4 — the inline spelling. Emits the same bytes as the positive file's
// `memalloc`; refused for the IL reason above, not for a codegen one.
void *inline_form(void *hp, unsigned long sz, unsigned long at) {
    if (hp == 0) {
        return g2(sz, at);
    }
    return h3(hp, (at >> 0x1b) & 8, sz);
}

// 3c — a SIGNED source, which is an arithmetic shift (`srawi`), not `rlwinm`.
void *signed_src(void *hp, unsigned long sz, int at) {
    if (hp == 0) {
        return g2(sz, (unsigned long)at);
    }
    unsigned long f = (unsigned long)((at >> 4) & 0x5);
    return h3(hp, f, sz);
}
