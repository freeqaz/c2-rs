// GRID BIND cell — the DIRECT spelling of `b_target_bind`: byte-for-byte the
// same constructor with the reference bind removed. Carried over unchanged from
// `work/w-f23/grid/f2_xboxheap_direct/`, so the two lanes' cells are the same
// cell and a difference between them cannot be a difference of source.
//
// This is the cell that reaches `store-run-call` today. Its partner does not,
// and the pair's REFERENCE OBJS differ in .text — which is the whole reason the
// reader must keep them apart.
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
