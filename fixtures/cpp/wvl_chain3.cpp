// **w-varloop** — the BODY-PARAMETERIZED pointer-walk loop, at a three-step
// chain: the cell where the induction load is **not** at slot 0.
//
// This family is the port's first lowering whose emitted body has no fixed
// length. `whash_ptr_walk_loop.cpp` beside it is a transcription — twenty words,
// two immediate fields, its own module doc says so — and the difference is
// visible in the fixtures: that one has a single length and this one is graded
// at six (`wvl_chain1`, `wvl_chain2`, this, `wvl_chain4`, `wvl_chain6`,
// `wvl_chain8`), with the schedule, the allocation, the entry form and the
// function's total length computed from the chain in every one of them.
//
// The twelve words `c2` emits here, and what each rule decides:
//
//     lbz    r11,0(r3)      the peel
//     mr     r10,r3         the walked pointer
//     li     r3,0           the accumulator's home — r3, which is why the
//                           fall-out block is a bare `blr`
//     extsb. r11,r11        the entry test (signed element)
//     bclr   12,2           w-rotate P2: the guard FOLDS, and so carries no
//                           displacement that a longer body could invalidate
//     add    r11,r11,r3     chain 0 -> CHAR itself (S4r's reuse clause)
//     lbzu   r9,1(r10)      S1: a = 1 at three steps, NOT 0
//     xori   r8,r11,3       chain 1 -> T1 (before the record form)
//     extsb. r11,r9         S2: R = 3
//     addi   r3,r8,5        chain 2 -> the home (the last producer)
//     bf     2,-20          -4*(M+2), computed
//     blr
//
// **`/O1` only**, like its sibling: every cell behind every rule was captured
// there, and `/Ox` is a different body rather than a different allocation. The
// `/Ox`, `/O2` and `/Od` lanes must read `NotImplemented` here, never `Match`.
int P(const char* s) {
    int r = 0;
    while (*s) {
        int c = *s;
        r = r + c;
        r = r ^ 3;
        r = r + 5;
        s++;
    }
    return r;
}
