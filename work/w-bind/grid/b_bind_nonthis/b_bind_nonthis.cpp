// GRID BIND cell `b_bind_nonthis`
// AXIS A — the bind is to a member of the SECOND pointer formal, not of
// `this`. The axis every grid on this row has held constant (board #866's
// refutation is what this cell exists for): the production must not care
// WHICH formal the bound object hangs off.
struct BE { BE* mNext; BE* mPrev; };
struct H {
    H* mFreeHead;      // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8   (mNext at 8, mPrev at 12)
    unsigned mSize;    // 16
    unsigned mCount;   // 20
};

void fn(H* a, H* b) {
    a->mSize = 1;
    BE& l = b->mListHead;
    l.mNext = &l;
    l.mPrev = &l;
}
