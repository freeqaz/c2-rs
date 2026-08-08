// wb-chooser Grid B cell B5 — see ../../WB_CHOOSER_PREREG.md §P3.
// Compiled by the REAL c2.dll under wibo. Not a fixture; the port never sees it.

// B5 — TWO distinct pooled constants, both used only in the then-arm.
// B-HYP: two `lis` at the top of the then-block, in FIRST-USE order (1.5f, 2.5f).
void f(float *c, int k) {
    if (k) {
        c[1] = c[2];
        c[0] = 1.5f;
        c[3] = 2.5f;
    }
}
