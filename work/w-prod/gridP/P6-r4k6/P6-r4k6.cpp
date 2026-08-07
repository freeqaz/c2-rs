struct F { int n0; int n1; int n2; int n3; int n4; int n5; };
struct G { F s0; F s1; };
struct H {
    int c0; int c1; int c2; int c3; int c4; int c5;
    int c6; int c7; int c8; int c9; int ca; int cb;
    int gap[7];
    G blk;
    G alt;
};
void q(H* h, H* i, int b) {
    F& k = h->blk.s0;
    F& m = h->blk.s0;
    h->c0 = 9;
    h->c1 = 9;
    h->c2 = 9;
    h->c3 = 9;
    h->c4 = 9;
    h->c5 = 9;
    m.n0 = (int)&k;
    m.n1 = (int)&k;
    m.n2 = (int)&k;
    m.n3 = (int)&k;
}
