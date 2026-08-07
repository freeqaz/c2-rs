// GRID2 (declared POST-HOC HOLDOUT) cell `c2_ctor_bind_nocall`
// PREDICTED, before this cell was compiled: store-run-bind-no-emitter-carrier:eof
// WHY IT IS HERE: the same constructor with NO trailing call — the plain run tail
struct Node { Node* n; Node* p; };
struct Q {
    unsigned mTag;
    Node mRing;
    unsigned mLen;
    Q(unsigned t);
};

Q::Q(unsigned t) {
    Node& ring = mRing;
    ring.n = &ring;
    ring.p = &ring;
    mTag = t;
    mLen = 0;
}
