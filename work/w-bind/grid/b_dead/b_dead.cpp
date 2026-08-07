// GRID BIND cell `b_dead`
// AXIS D — the bind is DEAD: bound and never used. c2 may delete the
// statement entirely. Paired with `b_dead_ctrl`.
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
    BE& l = h->mListHead;
    h->mCount = 0;
    h->mFreeHead = h;
}
