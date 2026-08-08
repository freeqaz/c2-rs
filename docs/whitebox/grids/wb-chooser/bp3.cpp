// wb-chooser Grid B' cell BP3 — see ../../WB_CHOOSER_PREREG.md §P3.
// Compiled by the REAL c2.dll under wibo. Not a fixture; the port never sees it.

// B'3 — a run of 3 divisions by ONE common divisor.
// P2.5: exactly one operand-load-order flip per cell, and it is the LAST
// division.  P2.6 (registered PESSIMISTIC): this does not hold across all four.
void f(float *o, float *f_) {
    o[0] = f_[0] / f_[9];
    o[1] = f_[1] / f_[9];
    o[2] = f_[2] / f_[9];
}
