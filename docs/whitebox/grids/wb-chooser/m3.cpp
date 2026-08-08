// wb-chooser Grid M cell M3 — see ../../WB_CHOOSER_PREREG.md §P3.
// Compiled by the REAL c2.dll under wibo. Not a fixture; the port never sees it.

// M3 — DISCRIMINATING. `a` live across a call to a same-TU clean leaf
// defined EARLIER in the file (so already emitted when `f` is emitted).
// M-HYP: VOL.  R-M-A: SAV.
__declspec(noinline) int leaf(int x) { return 0; }
int f(int a, int b) {
    int r = leaf(b);
    return a + r;
}
