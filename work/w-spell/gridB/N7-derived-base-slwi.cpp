struct LN7_derived_base_slwi { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct SN7_derived_base_slwi {
    int g0; int g1; int g2; int g3; int g4; int g5; int g6; int g7;
    int g8; int g9; int ga; int gb; int gc; int gd; int ge; int gf;
    LN7_derived_base_slwi inner;
    LN7_derived_base_slwi inner2;
    SN7_derived_base_slwi* nxt;
};
void gN7_derived_base_slwi(SN7_derived_base_slwi* s, int u, int v) {
    SN7_derived_base_slwi* p = s->nxt;
    s->g0 = 7;
    s->g1 = 7;
    p->inner.a0 = (u << 3);
    p->inner.a1 = (u << 3);
}
