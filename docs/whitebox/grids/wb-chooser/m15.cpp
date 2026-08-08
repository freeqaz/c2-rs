// wb-chooser Grid M cell M15 — UNREGISTERED exploratory (added after the
// PREREG was frozen; scored separately, never as a confirmation).
// Compiled by the REAL c2.dll under wibo. Not a fixture.

// M15 — a value live across the clean earlier leaf in a function that ALSO
// needs an addressing scratch for a global (a REFHI/REFLO pair). Does the pool
// shift when r11 is genuinely wanted for addressing?
extern int g;
__declspec(noinline) int leaf(int x) { return 0; }
int f(int a, int b) {
    int r = leaf(b);
    g = r;
    return a + r;
}
