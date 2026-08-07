// GRID BIND cell `b_ctrl_runcall`
// CONTROL — `w-f23`'s F3 shape with no bind: a store run then a member
// call. Its key must not change, because this lane touches neither
// obligation it depends on.
struct BE { BE* mNext; BE* mPrev; };
struct H {
    H* mFreeHead;
    H* mUsedHead;
    BE mListHead;
    unsigned mSize;
    unsigned mCount;
    void init(unsigned n);
    void grow(unsigned n);
};

void H::init(unsigned n) {
    mSize = n;
    mCount = 0;
    mFreeHead = this;
    grow(n);
}
