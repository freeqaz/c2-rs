struct Lf { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct Sf {
    int f0; int f1; int f2; int f3; int f4; int f5; int f6; int f7;
    int f8; int f9; int fa; int fb; int fc; int fd; int fe; int ff;
    Lf inner; Lf inner2;
};
void gxf();
Sf* ff(Sf* s, int u, int v) {
    Lf& q = s->inner;
    s->f4 = u;
    q.a0 = (int)&q;
    s->f5 = 0;
    q.a1 = (int)&q;
    gxf();
    return s;
}
