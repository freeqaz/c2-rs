struct LN10_disp0_beside_a_base_slwi { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct SN10_disp0_beside_a_base_slwi {
    int g0; int g1; int g2; int g3; int g4; int g5; int g6; int g7;
    int g8; int g9; int ga; int gb; int gc; int gd; int ge; int gf;
    LN10_disp0_beside_a_base_slwi inner;
    LN10_disp0_beside_a_base_slwi inner2;
    SN10_disp0_beside_a_base_slwi* nxt;
};
void gN10_disp0_beside_a_base_slwi(SN10_disp0_beside_a_base_slwi* s, SN10_disp0_beside_a_base_slwi* t, int u, int v) {
    LN10_disp0_beside_a_base_slwi& q = *(LN10_disp0_beside_a_base_slwi*)s;
    q.a0 = 7;
    q.a1 = 7;
    t->inner.a0 = (u << 3);
    t->inner.a1 = (u << 3);
}
