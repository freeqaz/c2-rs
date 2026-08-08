// wb-chooser Grid B cell B4 — see ../../WB_CHOOSER_PREREG.md §P3.
// Compiled by the REAL c2.dll under wibo. Not a fixture; the port never sees it.

// B4 — the constant is used ONLY after the join.
// B-HYP: `lis` at the top of the join block, i.e. BELOW the branch.
// R-B-B: at function entry.
void f(float *c, int k) {
    if (k) { c[1] = c[2]; c[3] = c[5]; }
    c[4] = 1.5f;
}
