// wb-chooser Grid M cell M9 — see ../../WB_CHOOSER_PREREG.md §P3.
// Compiled by the REAL c2.dll under wibo. Not a fixture; the port never sees it.

// M9 — mmioClose's own shape, reduced. `a` is live across the clean earlier
// leaf AND across the indirect call; `b` is live across the clean leaf ONLY and
// is consumed AS an argument of the indirect call.
// M-HYP: `a` SAV, `b` VOL.  R-M-A: both SAV.
extern void freeit(int);
__declspec(noinline) int leaf(int x) { return 0; }
int f(int a, int b, int (*fp)(int, int)) {
    int r = leaf(a);
    if (r) return r;
    int s = fp(a, b);
    if (s) return s;
    freeit(a);
    return 0;
}
