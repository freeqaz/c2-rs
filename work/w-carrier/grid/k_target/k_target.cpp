// GRID K cell `k_target` — w-carrier, board #1199.
// `src/xdk/nuispeech/xboxheap.cpp`'s ctor in its SHIPPED spelling
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
    auto& listHead = mListHead;
    listHead.mNext = &listHead;
    listHead.mPrev = &listHead;
    AllocatePageBlock(initSize);
}
