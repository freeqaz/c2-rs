// GRID2 (declared POST-HOC HOLDOUT) cell `c2_ctor_bind_wide`
// PREDICTED, before this cell was compiled: store-run-bind-no-emitter-carrier:eof
// WHY IT IS HERE: the bind at +64 with a five-store run — the bound base's displacement SUM, which the target exercises only at 8 and 12
struct Node { Node* n; Node* p; };
struct W {
    unsigned pad[16];  // 0..63
    Node mRing;        // 64 (n at 64, p at 68)
    unsigned mA;       // 72
    unsigned mB;       // 76
    W(unsigned a, unsigned b);
    void Reset();
};

W::W(unsigned a, unsigned b) {
    mA = a;
    Node& ring = mRing;
    ring.n = &ring;
    ring.p = &ring;
    mB = b;
    Reset();
}
