// GRID BIND cell — `src/xdk/nuispeech/xboxheap.cpp`'s constructor in its
// SHIPPED spelling: `auto& listHead = mListHead;` bound in the MIDDLE of the
// store run, then used in TWO later stores' BASE position and as their VALUE.
//
// This is board #839's target. Paired with `b_target_direct`, which is the same
// constructor with the bind removed — `w-heap` §4.2 (board #1128) measured that
// the two emit DIFFERENT bodies, so the pair is this lane's control that a
// reading which collapses them is wrong by construction.
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
