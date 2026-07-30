// The framed call's argument register in a **member function**: `this` occupies
// r3, so a one-parameter member's argument is in r4 and c2 emits `or r3,r4,r4`
// before the `bl`.
//
// This is the single most likely real-world instance of the mis-emit
// `wfr_argreg.cpp` documents — every non-static member function is shifted —
// and it is the third time a `this`-shifted formal has been the discriminator
// (`docs/GAPS.md` §6, instances 1 and 2). Its own TU: a member function's IL
// carries the `this` binding region, and a refused sibling here would make the
// whole file emit nothing and grade nothing.

int g(int);

struct S {
    int m;
    int one(int a);
    int two(int a, int b);
};

int S::one(int a) { return g(a) + 1; }
int S::two(int a, int b) { return g(b) + 2; }
