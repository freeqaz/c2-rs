struct LN5_three_formals_add { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct SN5_three_formals_add {
    int g0; int g1; int g2; int g3; int g4; int g5; int g6; int g7;
    int g8; int g9; int ga; int gb; int gc; int gd; int ge; int gf;
    LN5_three_formals_add inner;
    LN5_three_formals_add inner2;
    SN5_three_formals_add* nxt;
};
void gN5_three_formals_add(SN5_three_formals_add* s, SN5_three_formals_add* t, SN5_three_formals_add* r, int u, int v) {
    s->g0 = 7;
    s->g1 = 7;
    t->inner.a0 = (u + v);
    t->inner.a1 = (u + v);
    r->inner2.a0 = (u + 11);
    r->inner2.a1 = (u + 11);
}
