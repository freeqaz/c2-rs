struct LX_F_qaddr_regfirst_r1k1 { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct HX_F_qaddr_regfirst_r1k1 { int b0; int b1; int b2; int b3; int b4; int b5;
                int b6; int b7; int b8; int b9; int ba; int bb; };
struct SX_F_qaddr_regfirst_r1k1 {
    HX_F_qaddr_regfirst_r1k1 head;
    int f0; int f1; int f2; int f3; int f4; int f5; int f6; int f7;
    int f8; int f9; int fa; int fb; int fc; int fd; int fe; int ff;
    LX_F_qaddr_regfirst_r1k1 inner;
    LX_F_qaddr_regfirst_r1k1 inner2;
};
void gX_F_qaddr_regfirst_r1k1(SX_F_qaddr_regfirst_r1k1* s, int u, int v) {
    LX_F_qaddr_regfirst_r1k1& q = s->inner;
    q.a0 = (int)&q;
    s->f0 = 7;
}
