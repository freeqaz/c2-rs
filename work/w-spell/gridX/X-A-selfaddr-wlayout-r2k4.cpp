struct LX_A_selfaddr_wlayout_r2k4 { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct HX_A_selfaddr_wlayout_r2k4 { int b0; int b1; int b2; int b3; int b4; int b5;
                int b6; int b7; int b8; int b9; int ba; int bb; };
struct SX_A_selfaddr_wlayout_r2k4 {
    HX_A_selfaddr_wlayout_r2k4 head;
    int f0; int f1; int f2; int f3; int f4; int f5; int f6; int f7;
    int f8; int f9; int fa; int fb; int fc; int fd; int fe; int ff;
    LX_A_selfaddr_wlayout_r2k4 inner;
    LX_A_selfaddr_wlayout_r2k4 inner2;
};
void gX_A_selfaddr_wlayout_r2k4(SX_A_selfaddr_wlayout_r2k4* s, int u, int v) {
    LX_A_selfaddr_wlayout_r2k4& q = s->inner;
    s->f0 = 7;
    s->f1 = 7;
    s->f2 = 7;
    s->f3 = 7;
    q.a0 = (int)&s->inner;
    q.a1 = (int)&s->inner;
}
