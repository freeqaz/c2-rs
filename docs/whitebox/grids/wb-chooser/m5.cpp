// wb-chooser Grid M cell M5 — see ../../WB_CHOOSER_PREREG.md §P3.
// Compiled by the REAL c2.dll under wibo. Not a fixture; the port never sees it.

// M5 — `a` live across a same-TU leaf, defined EARLIER, that itself calls an
// extern (so its clobber set is hostile). Registered: SAV.
extern int ext(int);
__declspec(noinline) int mid(int x) { return ext(x); }
int f(int a, int b) {
    int r = mid(b);
    return a + r;
}
