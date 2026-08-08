// wb-chooser Grid B cell B1 — see ../../WB_CHOOSER_PREREG.md §P3.
// Compiled by the REAL c2.dll under wibo. Not a fixture; the port never sees it.

// B1 — one pooled float constant, used ONLY in the then-arm.
// B-HYP: `lis` = first word of the then-block, `lfs` at the use, 4 words later.
// R-B-A: `lis` adjacent to the `lfs`.   R-B-B: `lis` at function entry.
void f(float *c, int k) {
    if (k) {
        c[1] = c[2];
        c[3] = c[4];
        c[0] = 1.5f;
    }
}
