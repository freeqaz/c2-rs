// GRID BIND cell `b_bind_last`
// AXIS C — the bind comes AFTER every other store. Same statements as
// `b_bind_first`, permuted, so the pair is a within-grid cross-check
// (board #1174: the corpus stayed green through two wrong emits and a
// hand-written cross-product is what caught them).
struct BE { BE* mNext; BE* mPrev; };
struct H {
    H* mFreeHead;      // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8   (mNext at 8, mPrev at 12)
    unsigned mSize;    // 16
    unsigned mCount;   // 20
};

void fn(H* h, unsigned s) {
    h->mSize = s;
    h->mCount = 0;
    BE& l = h->mListHead;
    l.mNext = &l;
    l.mPrev = &l;
}
