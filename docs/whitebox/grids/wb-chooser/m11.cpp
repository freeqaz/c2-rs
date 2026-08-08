// wb-chooser Grid M cell M11 — see ../../WB_CHOOSER_PREREG.md §P3.
// Compiled by the REAL c2.dll under wibo. Not a fixture; the port never sees it.

// M11 — UNREGISTERED exploratory cell (added after the PREREG was frozen and
// scored separately, never as a confirmation of a registered prediction).
// Same clean leaf as M3 but with INTERNAL linkage: does linkage matter?
static __declspec(noinline) int leaf(int x) { return 0; }
extern int (*keep)(int);
int f(int a, int b) {
    int r = leaf(b);
    return a + r;
}
