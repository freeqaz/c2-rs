// wb-chooser Grid B cell B6 — see ../../WB_CHOOSER_PREREG.md §P3.
// Compiled by the REAL c2.dll under wibo. Not a fixture; the port never sees it.

// B6 — one pooled constant used TWICE in the then-arm.
// B-HYP: ONE `lis`, TWO `lfs`.   R-B-A: one `lis` per `lfs`.
void f(float *c, int k) {
    if (k) {
        c[0] = 1.5f;
        c[1] = c[2];
        c[3] = 1.5f;
    }
}
