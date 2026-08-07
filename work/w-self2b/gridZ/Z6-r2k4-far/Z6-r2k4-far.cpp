struct W { int m0; int m1; int m2; int m3; int m4; int m5; };
struct V { W u0; W u1; };
struct D {
    int e0; int e1; int e2; int e3; int e4; int e5;
    int e6; int e7; int e8; int e9; int ea; int eb;
    V core;
    int pad[52];
    V rim;
};
void n(D* d, D* g, int a) {
    W& k = d->rim.u0;
    W& j = d->rim.u0;
    d->e0 = 7;
    d->e1 = 7;
    d->e2 = 7;
    d->e3 = 7;
    j.m0 = (int)&k;
    j.m1 = (int)&k;
}
