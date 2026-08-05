// **w-varloop** — the MUST-REFUSE side of the body-parameterized loop, with a
// measured counterexample behind every one of the four functions.
//
// The whole TU must read `Port=NotImplemented` at **every** lane, never
// `Match` and never `mismatch`. Each function below is a shape `c2` compiles to
// something the emitter's rules do not describe, and the refusal is a positive
// guard rather than an omission — the recognizer admits an enumerated operator
// set, two literal ranges and one element width, and everything else declines.
//
//   R_and    `r & 21` selects **`andi.`**, which WRITES cr0. `c2` then demotes
//            the record form to a plain `extsb` and adds an explicit
//            `cmpwi r11,0` before the back edge — a DIFFERENT block, one word
//            longer, with a second CR writer in it. Measured:
//            `work/w-varloop/probe.py --body "r=r+c; r=r&21;"`.
//
//   R_sub    `r - c` selects **`subf`**, which is non-commutative: it computes
//            `RB - RA`, so its operand roles come from instruction selection
//            and **S5 does not speak for them**. w-sched2's own reconstruction
//            refuses the same population — seven cells, with the reason printed
//            rather than a rate with a hole in it.
//
//   R_nochar the chain never reads the character, so `pv` is undefined. Every
//            rule the emitter applies — the regime, the load's slot, the record
//            form's slot, the allocation — is stated in terms of `pv`.
//
//   R_pow2   `r * 8` is a **`rlwinm`**, not a `mulli`: the multiplier predicate
//            is the 38-constant grid `w-hash` graded, and a power of two is
//            outside it. Its siblings in the same family are `* 1` (no
//            instruction at all), `* -3` (`c2` rewrites `x + a*-3` as
//            `x - a*3`, changing an opcode this shape does not carry) and
//            `* 100000` (a `lis`/`ori`/`mullw` triple — board #644's split
//            producer, which this class keeps out by construction).
//
// Its separating control is `wvl_chain3.cpp`, whose loop differs from `R_and`'s
// only in the operator, and which is byte-exact at `/O1`. Together they say the
// refusal is a property of the operator and not of the loop.
int R_and(const char* s) {
    int r = 0;
    while (*s) { int c = *s; r = r + c; r = r & 21; s++; }
    return r;
}
int R_sub(const char* s) {
    int r = 0;
    while (*s) { int c = *s; r = r + c; r = r - c; s++; }
    return r;
}
int R_nochar(const char* s) {
    int r = 0;
    while (*s) { int c = *s; r = r + 1; r = r ^ 3; s++; }
    return r;
}
int R_pow2(const char* s) {
    int r = 0;
    while (*s) { int c = *s; r = r + c; r = r * 8; s++; }
    return r;
}
