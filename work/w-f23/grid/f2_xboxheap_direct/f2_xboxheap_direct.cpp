// GRID F2 cell — `xboxheap`'s own constructor in the DIRECT spelling (board
// #839, `w-heap` §4.2): the reference bind removed, so the F2 address value
// appears in a store through `this` rather than in a store into a local. This
// is the spelling where F2 is reachable at all; the shipped bind spelling puts
// the address's destination in a LOCAL, which `parse_store_stmt` refuses on its
// own terms and which is refusal #5, not #1.
struct BE { BE* mNext; BE* mPrev; };
struct CXboxHeap {
    BE* mFreeHead;
    BE* mUsedHead;
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
