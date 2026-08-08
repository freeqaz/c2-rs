// wb-chooser Grid M cell M1 — see ../../WB_CHOOSER_PREREG.md §P3.
// Compiled by the REAL c2.dll under wibo. Not a fixture; the port never sees it.

// M1 — control: `a` is live across NO call. Registered: VOL.
int f(int a, int b) {
    int r = b * 7 + 3;
    return a + r;
}
