struct L { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct M { L in1; L in2; };
struct S {
    int p0; int p1; int p2; int p3; int p4;
    int p5; int p6; int p7; int p8; int p9;
    M mid;
    L tail;
};
void h(S* s, S* t, int u, int v, int w) {
    s->p0 = 7;
    s->p1 = 7;
    s->p2 = 7;
    s->p3 = 7;
    s->mid.in1.a0 = (int)&s->mid.in2;
    s->mid.in1.a1 = (int)&s->mid.in2;
    s->mid.in1.a2 = (int)&s->mid.in2;
}
