// GRID BIND cell `b_run_call`
// AXIS G — bind + run + a trailing member call whose argument setup writes
// no register (board #1129's refined regime gate). The target's own shape,
// generic: not the xboxheap constructor, so a reading fitted to that one
// body shows up here as a disagreement.
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
    BE& l = mListHead;
    l.mNext = &l;
    l.mPrev = &l;
    grow(n);
}
