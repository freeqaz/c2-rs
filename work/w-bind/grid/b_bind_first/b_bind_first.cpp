// GRID BIND cell `b_bind_first`
// AXIS C — the bind is the FIRST statement of the body. The target binds
// in the MIDDLE; nothing on record varies this.
struct BE { BE* mNext; BE* mPrev; };
struct H {
    H* mFreeHead;      // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8   (mNext at 8, mPrev at 12)
    unsigned mSize;    // 16
    unsigned mCount;   // 20
};

void fn(H* h, unsigned s) {
    BE& l = h->mListHead;
    l.mNext = &l;
    l.mPrev = &l;
    h->mSize = s;
    h->mCount = 0;
}
