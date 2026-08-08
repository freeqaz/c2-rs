// wb-chooser Grid M cell M13 — see ../../WB_CHOOSER_PREREG.md §P3.
// Compiled by the REAL c2.dll under wibo. Not a fixture; the port never sees it.

// M13 — UNREGISTERED exploratory cell. The earlier same-TU leaf does real work
// in several volatiles (r3..r10 pressure) but calls nothing.
__declspec(noinline) int leaf(int x, int y, int z, int w) {
    return (x * 3 + y * 5) ^ (z * 7 + w * 11);
}
int f(int a, int b) {
    int r = leaf(b, b + 1, b + 2, b + 3);
    return a + r;
}
