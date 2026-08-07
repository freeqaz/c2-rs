struct P { int b0; int b1; int b2; int b3; int b4; int b5; };
struct Q { P lo; P hi; };
struct T {
    int c0; int c1; int c2; int c3; int c4;
    int c5; int c6; int c7; int c8; int c9;
    Q mid;
    P tail;
};
void k(T* t, T* r, int x) {
    P& q = t->mid.lo;
    t->c0 = 7;
    t->c1 = 7;
    t->c2 = 7;
    q.b0 = (int)&q;
    q.b1 = (int)&q;
    q.b2 = (int)&q;
}
