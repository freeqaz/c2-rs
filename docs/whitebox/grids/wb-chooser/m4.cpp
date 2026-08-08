// wb-chooser Grid M cell M4 — see ../../WB_CHOOSER_PREREG.md §P3.
// Compiled by the REAL c2.dll under wibo. Not a fixture; the port never sees it.

// M4 — DISCRIMINATING. The identical clean leaf, defined LATER in the file.
// M-HYP/P1.3 (emission-order-sensitive): SAV.  R-M-C (whole-TU): VOL.
__declspec(noinline) int leaf(int x);
int f(int a, int b) {
    int r = leaf(b);
    return a + r;
}
__declspec(noinline) int leaf(int x) { return 0; }
