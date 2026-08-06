struct LH1_nor_none_r1k2 { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct SH1_nor_none_r1k2 {
    LH1_nor_none_r1k2 head;
    int f0; int f1; int f2; int f3; int f4; int f5; int f6; int f7;
    int f8; int f9; int fa; int fb; int fc; int fd; int fe; int ff;
    LH1_nor_none_r1k2 inner;
    LH1_nor_none_r1k2 inner2;
};
void gH1_nor_none_r1k2(SH1_nor_none_r1k2* s, int u, int v) {
    s->f0 = 7;
    s->f1 = 7;
    s->inner.a0 = ~u;
}
