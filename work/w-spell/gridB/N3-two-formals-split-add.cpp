struct LN3_two_formals_split_add { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct SN3_two_formals_split_add {
    int g0; int g1; int g2; int g3; int g4; int g5; int g6; int g7;
    int g8; int g9; int ga; int gb; int gc; int gd; int ge; int gf;
    LN3_two_formals_split_add inner;
    LN3_two_formals_split_add inner2;
    SN3_two_formals_split_add* nxt;
};
void gN3_two_formals_split_add(SN3_two_formals_split_add* s, SN3_two_formals_split_add* t, int u, int v) {
    s->g0 = 7;
    s->g1 = 7;
    t->inner.a0 = (u + v);
    t->inner.a1 = (u + v);
}
