// wb-chooser Grid M cell M7 — see ../../WB_CHOOSER_PREREG.md §P3.
// Compiled by the REAL c2.dll under wibo. Not a fixture; the port never sees it.

// M7 — TWO formals live across an extern call. Registered: SAV x2,
// allocated r31 then r30 (P1.5).
extern int ext(int);
int f(int a, int b, int c) {
    int r = ext(c);
    return a + b + r;
}
