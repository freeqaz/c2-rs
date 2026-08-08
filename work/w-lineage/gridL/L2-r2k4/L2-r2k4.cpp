struct U { U* n0; U* n1; U* n2; U* n3; U* n4; U* n5; };
struct K { U x0; U x1; };
struct B {
    int g0; int g1; int g2; int g3; int g4; int g5;
    int g6; int g7; int g8; int g9; int ga; int gb;
    int fill[4];
    K hub;
    K spare;
};
void s(B* p, B* q, int w) {
    U& a = p->hub.x0;
    p->g0 = 3;
    p->g1 = 3;
    p->g2 = 3;
    p->g3 = 3;
    p->hub.x0.n0 = &a;
    p->hub.x0.n1 = &a;
}
