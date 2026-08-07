// GRID2 (declared POST-HOC HOLDOUT) cell `c2_ctor_bind_argsetup`
// PREDICTED, before this cell was compiled: NOT store-run-bind-no-emitter-carrier:eof
// WHY IT IS HERE: board #1129's break: the callee's argument setup WRITES a register, so the composition must refuse — inherited, not re-derived
struct Node { Node* n; Node* p; };
struct R {
    unsigned mTag;
    Node mRing;
    unsigned mLen;
    R(unsigned t, unsigned u);
    void Grow(unsigned n);
};

R::R(unsigned t, unsigned u) {
    Node& ring = mRing;
    ring.n = &ring;
    ring.p = &ring;
    mTag = t;
    mLen = 0;
    Grow(u);
}
