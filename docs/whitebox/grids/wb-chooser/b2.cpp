// wb-chooser Grid B cell B2 — see ../../WB_CHOOSER_PREREG.md §P3.
// Compiled by the REAL c2.dll under wibo. Not a fixture; the port never sees it.

// B2 — one pooled constant, used in BOTH arms.
// B-HYP: one `lis` at function entry, ABOVE the compare.
void f(float *c, int k) {
    if (k) { c[1] = 1.5f; c[2] = c[3]; }
    else   { c[4] = 1.5f; c[5] = c[6]; }
}
