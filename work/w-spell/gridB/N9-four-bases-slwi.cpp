struct LN9_four_bases_slwi { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct SN9_four_bases_slwi {
    int g0; int g1; int g2; int g3; int g4; int g5; int g6; int g7;
    int g8; int g9; int ga; int gb; int gc; int gd; int ge; int gf;
    LN9_four_bases_slwi inner;
    LN9_four_bases_slwi inner2;
    SN9_four_bases_slwi* nxt;
};
void gN9_four_bases_slwi(SN9_four_bases_slwi* s, SN9_four_bases_slwi* t, SN9_four_bases_slwi* r, int u, int v) {
    LN9_four_bases_slwi& q = s->inner2;
    s->g0 = 7;
    s->g1 = 7;
    t->inner.a0 = (u << 3);
    t->inner.a1 = (u << 3);
    r->g8 = (u + 11);
    q.a4 = (u + 11);
}
