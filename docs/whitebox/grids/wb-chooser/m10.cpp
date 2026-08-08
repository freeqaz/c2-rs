// wb-chooser Grid M cell M10 — see ../../WB_CHOOSER_PREREG.md §P3.
// Compiled by the REAL c2.dll under wibo. Not a fixture; the port never sees it.

// M10 — an earlier same-TU callee that is noinline and NON-leaf (it makes an
// indirect call, so its clobber set is hostile in a different way from M5's).
// Registered: SAV.
__declspec(noinline) int mid(int x, int (*g)(int)) { return g(x); }
extern int (*gp)(int);
int f(int a, int b) {
    int r = mid(b, gp);
    return a + r;
}
