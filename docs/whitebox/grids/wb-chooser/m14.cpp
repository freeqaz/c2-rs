// wb-chooser Grid M cell M14 — UNREGISTERED exploratory (added after the
// PREREG was frozen; scored separately, never as a confirmation).
// Compiled by the REAL c2.dll under wibo. Not a fixture.

// M14 — TWO values live across the same clean earlier leaf. If the pool for
// call-crossing values excludes r11 (the addressing/linkage scratch), the picks
// are r10 and r9; if it does not, they are r11 and r10.
__declspec(noinline) int leaf(int x) { return 0; }
int f(int a, int b, int c) {
    int r = leaf(c);
    return a + b + r;
}
