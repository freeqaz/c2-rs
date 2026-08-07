// GRID BIND cell `b_const_ref`
// AXIS D — a CONST reference. Cannot be stored through, so it exercises
// the value role under a qualifier the `2C`/volatile gates care about.
struct BE { BE* mNext; BE* mPrev; };
struct H {
    const BE* mSpare;  // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8
    unsigned mSize;    // 16
    unsigned mCount;   // 20
};

void fn(H* h) {
    const BE& l = h->mListHead;
    h->mSpare = &l;
    h->mSize = 4;
}
