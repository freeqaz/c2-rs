// GRID2 (declared POST-HOC HOLDOUT) cell `c2_ctor_bind_call`
// PREDICTED, before this cell was compiled: store-run-bind-no-emitter-carrier:eof
// WHY IT IS HERE: a constructor that is not xboxheap: bind at +4, bind FIRST, one formal, a NULLARY callee
struct Node { Node* n; Node* p; };
struct Q {
    unsigned mTag;   // 0
    Node mRing;      // 4  (n at 4, p at 8)
    unsigned mLen;   // 12
    Q(unsigned t);
    void Reset();
};

Q::Q(unsigned t) {
    Node& ring = mRing;
    ring.n = &ring;
    ring.p = &ring;
    mTag = t;
    mLen = 0;
    Reset();
}
