struct LN6_shared_base_third_run_add { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct SN6_shared_base_third_run_add {
    int g0; int g1; int g2; int g3; int g4; int g5; int g6; int g7;
    int g8; int g9; int ga; int gb; int gc; int gd; int ge; int gf;
    LN6_shared_base_third_run_add inner;
    LN6_shared_base_third_run_add inner2;
    SN6_shared_base_third_run_add* nxt;
};
void gN6_shared_base_third_run_add(SN6_shared_base_third_run_add* s, SN6_shared_base_third_run_add* t, int u, int v) {
    s->g0 = 7;
    s->g1 = 7;
    s->inner.a0 = (u + v);
    s->inner.a1 = (u + v);
    t->inner2.a0 = (u + 11);
    t->inner2.a1 = (u + 11);
}
