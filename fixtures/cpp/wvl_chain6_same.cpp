// **w-varloop** — the SAME regime: six chain steps, and the entry form is an
// unconditional jump INTO the loop rather than a guard around it.
//
// The pair (`wvl_chain3.cpp`, this) is the fixture-level statement of **S3m**,
// the rule that made a lowering reachable at all: w-rotate could not evaluate
// the entry form from IL, and w-sched2 found that it is decided by two IL facts
// — `pv == 0` and `M >= 4` — at 84 of 84 held out. Three steps take the `bclr`
// guard, four or more take the `b`.
//
// The fourteen words:
//
//     lbz    r11,0(r3)
//     mr     r10,r3
//     li     r3,0
//     b      .+32           JUMPIN, computed as 4*(M+2), to the record form
//     add    r9,r11,r3      chain 0 -> T2
//     lbzu   r11,1(r10)     the load writes the CHARACTER'S OWN register: the
//                           whole loop runs on one register, which is why the
//                           peeled test does not need a second copy
//     xori   r9,r9,3
//     addi   r9,r9,5
//     ori    r9,r9,9
//     addi   r9,r9,11
//     xori   r3,r9,13       the last producer -> the home
//     extsb. r11,r11        R = M+1, the body's LAST word
//     bf     2,-32
//     blr
//
// **M + 8 words, not M + 9** — the SAME regime is one word shorter than the TWO
// regime at the same chain length, and that is a property of the *entry form*
// rather than of the body. A port that emitted the TWO preamble here would be
// one word long and wrong about every offset after it.
//
// `/O1` only.
int P(const char* s) {
    int r = 0;
    while (*s) {
        int c = *s;
        r = r + c;
        r = r ^ 3;
        r = r + 5;
        r = r | 9;
        r = r + 11;
        r = r ^ 13;
        s++;
    }
    return r;
}
