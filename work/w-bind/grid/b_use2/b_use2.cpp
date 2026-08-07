// GRID BIND cell `b_use2`
// AXIS B — the bound name in TWO stores' base position, value a formal.
// Separates 'used as a base' from 'used as a value': the target uses it
// as both and cannot tell the two roles apart.
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
    l.mPrev = p;
}
