struct LN4_bind_used_add { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct SN4_bind_used_add {
    int g0; int g1; int g2; int g3; int g4; int g5; int g6; int g7;
    int g8; int g9; int ga; int gb; int gc; int gd; int ge; int gf;
    LN4_bind_used_add inner;
    LN4_bind_used_add inner2;
    SN4_bind_used_add* nxt;
};
void gN4_bind_used_add(SN4_bind_used_add* s, int u, int v) {
    LN4_bind_used_add& q = s->inner;
    s->g0 = 7;
    s->g1 = 7;
    q.a0 = (u + v);
    q.a1 = (u + v);
}
