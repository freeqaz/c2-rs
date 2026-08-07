// GRID BIND cell `b_leaf_bind`
// AXIS G — bind + exactly ONE store. A LEAF, not a run: the run gates
// (overlap, mixed-kind, the literal pool) are all unreachable, so this
// isolates the base-position obligation from everything the run adds.
struct BE { BE* mNext; BE* mPrev; };
struct H {
    H* mFreeHead;      // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8   (mNext at 8, mPrev at 12)
    unsigned mSize;    // 16
    unsigned mCount;   // 20
};

void fn(H* h) {
    BE& l = h->mListHead;
    l.mNext = &l;
}
