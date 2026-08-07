// GRID K cell `k_target_direct` — w-carrier, board #1199.
// the same ctor WITHOUT the bind — #1128's four-word control
// Compiled at the WORKLOAD's own /GR /O1 /Oi /EHsc (#1112).
struct BE { BE* mNext; BE* mPrev; };
struct CXboxHeap {
    CXboxHeap* mFreeHead;
    CXboxHeap* mUsedHead;
    BE  mListHead;
    unsigned mSize;
    unsigned mCount;
    CXboxHeap(unsigned initSize, unsigned size);
    BE* AllocatePageBlock(unsigned n);
};

CXboxHeap::CXboxHeap(unsigned initSize, unsigned size) {
    mSize = size;
    mFreeHead = this;
    mCount = 0;
    mUsedHead = this;
    mListHead.mNext = &mListHead;
    mListHead.mPrev = &mListHead;
    AllocatePageBlock(initSize);
}
