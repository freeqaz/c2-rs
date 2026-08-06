struct LH1_srw_2base_r2k1 { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct HH1_srw_2base_r2k1 { int b0; int b1; int b2; int b3; int b4; int b5;
                int b6; int b7; int b8; int b9; int ba; int bb; };
struct SH1_srw_2base_r2k1 {
    HH1_srw_2base_r2k1 head;
    int f0; int f1; int f2; int f3; int f4; int f5; int f6; int f7;
    int f8; int f9; int fa; int fb; int fc; int fd; int fe; int ff;
    LH1_srw_2base_r2k1 inner;
    LH1_srw_2base_r2k1 inner2;
    short w0; short w1; char c0; char c1;
};
void gH1_srw_2base_r2k1(SH1_srw_2base_r2k1* s, int u, int v) {
    LH1_srw_2base_r2k1& q = s->inner;
    s->f0 = 21;
    q.a0 = (int)((unsigned)u >> (v & 31));
    q.a1 = (int)((unsigned)u >> (v & 31));
}
