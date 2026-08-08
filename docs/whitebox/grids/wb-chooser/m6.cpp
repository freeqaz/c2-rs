// wb-chooser Grid M cell M6 — see ../../WB_CHOOSER_PREREG.md §P3.
// Compiled by the REAL c2.dll under wibo. Not a fixture; the port never sees it.

// M6 — `a` live across an INDIRECT call through a function pointer.
// Registered: SAV.
int f(int a, int b, int (*fp)(int)) {
    int r = fp(b);
    return a + r;
}
