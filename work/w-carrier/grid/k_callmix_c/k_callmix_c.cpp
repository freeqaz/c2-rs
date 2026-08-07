// GRID K cell `k_callmix_c` — w-carrier, board #1199.
// control
// Compiled at the WORKLOAD's own /GR /O1 /Oi /EHsc (#1112).
struct BE { BE* mNext; BE* mPrev; };
struct H {
    H* mFreeHead;      // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8   (mNext at 8, mPrev at 12)
    unsigned mSize;    // 16
    unsigned mCount;   // 20
    BE* mSpare;        // 24
    H(unsigned initSize, unsigned size);
    BE* AllocatePageBlock(unsigned n);
};

H::H(unsigned initSize, unsigned size) {
    mSize = size;
    mCount = 0;
    mListHead.mNext = &mListHead;
    AllocatePageBlock(initSize);
}
