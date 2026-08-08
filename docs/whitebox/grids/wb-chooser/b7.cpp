// wb-chooser Grid B cell B7 — see ../../WB_CHOOSER_PREREG.md §P3.
// Compiled by the REAL c2.dll under wibo. Not a fixture; the port never sees it.

// B7 — the constant is used only inside a LOOP body.
// B-HYP: `lis` in the pre-header, OUTSIDE the loop (registered optimistic).
void f(float *c, int n) {
    for (int i = 0; i < n; i++) {
        c[i] = 1.5f;
    }
}
