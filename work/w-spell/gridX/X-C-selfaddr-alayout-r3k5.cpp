struct LX_C_selfaddr_alayout_r3k5 { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct SX_C_selfaddr_alayout_r3k5 {
    int f0; int f1; int f2; int f3; int f4; int f5; int f6; int f7;
    int f8; int f9; int fa; int fb; int fc; int fd; int fe; int ff;
    LX_C_selfaddr_alayout_r3k5 inner;
    LX_C_selfaddr_alayout_r3k5 inner2;
};
void gX_C_selfaddr_alayout_r3k5(SX_C_selfaddr_alayout_r3k5* s, int u, int v) {
    LX_C_selfaddr_alayout_r3k5& q = s->inner;
    s->f0 = 7;
    s->f1 = 7;
    s->f2 = 7;
    s->f3 = 7;
    s->f4 = 7;
    q.a0 = (int)&s->inner;
    q.a1 = (int)&s->inner;
    q.a2 = (int)&s->inner;
}
