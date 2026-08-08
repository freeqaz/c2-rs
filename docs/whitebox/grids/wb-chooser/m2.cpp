// wb-chooser Grid M cell M2 — see ../../WB_CHOOSER_PREREG.md §P3.
// Compiled by the REAL c2.dll under wibo. Not a fixture; the port never sees it.

// M2 — `a` live across a call to an EXTERN function (unknown clobbers).
// Registered: SAV.
extern int ext(int);
int f(int a, int b) {
    int r = ext(b);
    return a + r;
}
