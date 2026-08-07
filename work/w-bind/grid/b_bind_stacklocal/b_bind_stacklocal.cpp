// GRID BIND cell `b_bind_stacklocal`
// AXIS A — the bind is to a STACK LOCAL aggregate, which is a frame
// object. The body is not a leaf at all and must refuse.
struct BE { BE* mNext; BE* mPrev; };
struct H {
    H* mFreeHead;      // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8   (mNext at 8, mPrev at 12)
    unsigned mSize;    // 16
    unsigned mCount;   // 20
};

void fn(H* h) {
    BE tmp;
    BE& l = tmp;
    l.mNext = &l;
    l.mPrev = &l;
    h->mSize = 1;
}
