// wb-chooser Grid M cell M8 — see ../../WB_CHOOSER_PREREG.md §P3.
// Compiled by the REAL c2.dll under wibo. Not a fixture; the port never sees it.

// M8 — `a` live across TWO calls to the same-TU clean leaf defined EARLIER.
// M-HYP: VOL.  R-M-A: SAV.
__declspec(noinline) int leaf(int x) { return 0; }
int f(int a, int b) {
    int r = leaf(b);
    int s = leaf(r);
    return a + s;
}
