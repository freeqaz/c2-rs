struct LN8_two_formals_one_base_add { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct SN8_two_formals_one_base_add {
    int g0; int g1; int g2; int g3; int g4; int g5; int g6; int g7;
    int g8; int g9; int ga; int gb; int gc; int gd; int ge; int gf;
    LN8_two_formals_one_base_add inner;
    LN8_two_formals_one_base_add inner2;
    SN8_two_formals_one_base_add* nxt;
};
void gN8_two_formals_one_base_add(SN8_two_formals_one_base_add* s, SN8_two_formals_one_base_add* t, int u, int v) {
    t->g0 = 7;
    t->g1 = 7;
    t->inner.a0 = (u + v);
    t->inner.a1 = (u + v);
}
