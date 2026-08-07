struct W { int m0; int m1; int m2; int m3; int m4; int m5; };
struct V { W u0; W u1; };
struct D { int e0; int e1; int e2; V core; };
void n(D* d, int a) {
    d->e0 = 7;
    d->e1 = 7;
    d->e2 = 7;
    W& k = d->core.u0;
    k.m0 = (int)&k;
    k.m1 = (int)&k;
}
