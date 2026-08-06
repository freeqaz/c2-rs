struct LN7_derived_base_add { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct SN7_derived_base_add {
    int g0; int g1; int g2; int g3; int g4; int g5; int g6; int g7;
    int g8; int g9; int ga; int gb; int gc; int gd; int ge; int gf;
    LN7_derived_base_add inner;
    LN7_derived_base_add inner2;
    SN7_derived_base_add* nxt;
};
void gN7_derived_base_add(SN7_derived_base_add* s, int u, int v) {
    SN7_derived_base_add* p = s->nxt;
    s->g0 = 7;
    s->g1 = 7;
    p->inner.a0 = (u + v);
    p->inner.a1 = (u + v);
}
