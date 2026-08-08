// wb-chooser Grid B cell B3 — see ../../WB_CHOOSER_PREREG.md §P3.
// Compiled by the REAL c2.dll under wibo. Not a fixture; the port never sees it.

// B3 — Biquad's own 0.0f shape: used in the then-arm AND after the join.
// B-HYP: one `lis` at function entry, ABOVE the compare.
void f(float *c, int k) {
    if (k) { c[1] = 1.5f; c[2] = c[3]; }
    c[4] = 1.5f;
}
