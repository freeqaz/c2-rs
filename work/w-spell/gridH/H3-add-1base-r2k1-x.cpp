struct LH3_add_1base_r2k1_x { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct HH3_add_1base_r2k1_x { int b0; int b1; int b2; int b3; int b4; int b5;
                int b6; int b7; int b8; int b9; int ba; int bb; };
struct SH3_add_1base_r2k1_x {
    HH3_add_1base_r2k1_x head;
    int f0; int f1; int f2; int f3; int f4; int f5; int f6; int f7;
    int f8; int f9; int fa; int fb; int fc; int fd; int fe; int ff;
    LH3_add_1base_r2k1_x inner;
    LH3_add_1base_r2k1_x inner2;
    short w0; short w1; char c0; char c1;
};
void gH3_add_1base_r2k1_x(SH3_add_1base_r2k1_x* s, int u, int v, int w) {
    s->f0 = 21;
    s->inner.a0 = (u + v);
    s->inner.a1 = (u + v);
    s->ff = w;
}
