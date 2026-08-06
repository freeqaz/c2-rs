struct LH2_slwi_2base_r3k5 { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct HH2_slwi_2base_r3k5 { int b0; int b1; int b2; int b3; int b4; int b5;
                int b6; int b7; int b8; int b9; int ba; int bb; };
struct SH2_slwi_2base_r3k5 {
    HH2_slwi_2base_r3k5 head;
    int f0; int f1; int f2; int f3; int f4; int f5; int f6; int f7;
    int f8; int f9; int fa; int fb; int fc; int fd; int fe; int ff;
    LH2_slwi_2base_r3k5 inner;
    LH2_slwi_2base_r3k5 inner2;
    short w0; short w1; char c0; char c1;
};
void gH2_slwi_2base_r3k5(SH2_slwi_2base_r3k5* s, int u, int v) {
    LH2_slwi_2base_r3k5& q = s->inner;
    s->f0 = 21;
    s->f1 = 21;
    s->f2 = 21;
    s->f3 = 21;
    s->f4 = 21;
    q.a0 = (u << 3);
    q.a1 = (u << 3);
    q.a2 = (u << 3);
}
