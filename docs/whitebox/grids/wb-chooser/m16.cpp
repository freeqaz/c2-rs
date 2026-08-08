// wb-chooser Grid M cell M16 — UNREGISTERED exploratory (added after the
// PREREG was frozen; scored separately, never as a confirmation).
// Compiled by the REAL c2.dll under wibo. Not a fixture.

// M16 — the same value live across a call to a same-TU clean leaf that is
// itself defined LATER *and* whose body would be emitted after two other
// functions: a second, independent check of the emission-order question that
// M4 answered against the registered prediction.
__declspec(noinline) int leaf(int x);
int f(int a, int b) {
    int r = leaf(b);
    return a + r;
}
__declspec(noinline) int filler(int x) { return x * 3; }
__declspec(noinline) int leaf(int x) { return 0; }
