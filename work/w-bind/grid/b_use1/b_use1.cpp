// GRID BIND cell `b_use1`
// AXIS B — the bound name stands in exactly ONE store's base position,
// and is NOT the stored value. One use, one base symbol's worth.
struct BE { BE* mNext; BE* mPrev; };
struct H {
    H* mFreeHead;      // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8   (mNext at 8, mPrev at 12)
    unsigned mSize;    // 16
    unsigned mCount;   // 20
};

void fn(H* h, BE* p) {
    h->mSize = 2;
    BE& l = h->mListHead;
    l.mNext = p;
}
