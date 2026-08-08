// wb-chooser Grid M cell M12 — see ../../WB_CHOOSER_PREREG.md §P3.
// Compiled by the REAL c2.dll under wibo. Not a fixture; the port never sees it.

// M12 — UNREGISTERED exploratory cell. The earlier same-TU leaf is clean but
// RETURNS ITS ARGUMENT (clobbers r3 only, still); does the value returned
// rather than a constant change anything?
__declspec(noinline) int leaf(int x) { return x; }
int f(int a, int b) {
    int r = leaf(b);
    return a + r;
}
