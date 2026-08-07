// GRID K cell `k_dead` — w-carrier, board #1199.
// a bind nothing reads — w-bind declined it and so does this lane
// Compiled at the WORKLOAD's own /GR /O1 /Oi /EHsc (#1112).
struct BE { BE* mNext; BE* mPrev; };
struct H {
    H* mFreeHead;      // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8   (mNext at 8, mPrev at 12)
    unsigned mSize;    // 16
    unsigned mCount;   // 20
    BE mSecond;        // 24  (mNext at 24, mPrev at 28)
    BE* mSpare;        // 32
    unsigned mA;       // 36
    unsigned mB;       // 40
};

void fn(H* h, BE* p) {
    h->mSize = 2;
    BE& l = h->mListHead;
    h->mFreeHead = h;
}
