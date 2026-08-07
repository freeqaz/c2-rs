struct P { int b0; int b1; int b2; int b3; int b4; int b5; };
struct Q { P lo; P hi; };
struct T {
    int c0; int c1; int c2; int c3; int c4;
    int c5; int c6; int c7; int c8; int c9;
    Q mid;
    P tail;
};
void k(T* t, T* r, int x) {
    t->c0 = 7;
    t->c1 = 7;
    t->c2 = 7;
    t->c3 = 7;
    t->mid.lo.b0 = (int)&t->mid.lo;
    t->mid.lo.b1 = (int)&t->mid.lo;
    t->mid.lo.b2 = (int)&t->mid.lo;
    t->mid.lo.b3 = (int)&t->mid.lo;
}
