struct LN2_bind_disp0_slwi { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct SN2_bind_disp0_slwi {
    int g0; int g1; int g2; int g3; int g4; int g5; int g6; int g7;
    int g8; int g9; int ga; int gb; int gc; int gd; int ge; int gf;
    LN2_bind_disp0_slwi inner;
    LN2_bind_disp0_slwi inner2;
    SN2_bind_disp0_slwi* nxt;
};
void gN2_bind_disp0_slwi(SN2_bind_disp0_slwi* s, int u, int v) {
    LN2_bind_disp0_slwi& q = *(LN2_bind_disp0_slwi*)s;
    s->g0 = 7;
    s->g1 = 7;
    q.a4 = (u << 3);
    q.a5 = (u << 3);
}
