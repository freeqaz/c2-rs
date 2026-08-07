// x0 — the exact shape of `src/xdk/nuispeech/xboxheap.cpp`'s ctor, self-contained.
//
// Lane w-front2. Re-pricing probe for board #401 (`xboxheap.cpp` priced at 5,
// DECLINED) after `codegen::schedule` / `::alloc` / `::order` shipped — F4a/F4b
// of that price were "the schedule", which is the axis those three modules
// close.
struct BE { BE* mNext; BE* mPrev; };
struct H {
    H(unsigned int a, unsigned int b);
    BE* Alloc(unsigned int);
    H* mFreeHead; H* mUsedHead; BE mListHead; unsigned int mSize; unsigned int mCount;
};
H::H(unsigned int initSize, unsigned int size) {
    mSize = size;
    mFreeHead = this;
    mCount = 0;
    mUsedHead = this;
    BE& listHead = mListHead;
    listHead.mNext = &listHead;
    listHead.mPrev = &listHead;
    Alloc(initSize);
}
