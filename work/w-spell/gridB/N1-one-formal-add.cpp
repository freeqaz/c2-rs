struct LN1_one_formal_add { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct SN1_one_formal_add {
    int g0; int g1; int g2; int g3; int g4; int g5; int g6; int g7;
    int g8; int g9; int ga; int gb; int gc; int gd; int ge; int gf;
    LN1_one_formal_add inner;
    LN1_one_formal_add inner2;
    SN1_one_formal_add* nxt;
};
void gN1_one_formal_add(SN1_one_formal_add* s, int u, int v) {
    s->g0 = 7;
    s->g1 = 7;
    s->inner.a0 = (u + v);
    s->inner.a1 = (u + v);
}
