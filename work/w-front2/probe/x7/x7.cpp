// x7 — xboxheap's FULL SIX-STORE RUN, with the two `&mListHead` values replaced
// by a formal pointer. No member address (F2 removed), no call (F3 removed).
//
// This is the SCHEDULE cell: board #401's F4a/F4b. If it matches, the store
// schedule that `w-conv` called "unpriceable" and #270 called "diverges at
// instruction 0 on order" is paid at this width.
struct BE { BE* mNext; BE* mPrev; };
struct H {
    H(unsigned int a, unsigned int b, BE* p);
    H* mFreeHead; H* mUsedHead; BE mListHead; unsigned int mSize; unsigned int mCount;
};
H::H(unsigned int initSize, unsigned int size, BE* p) {
    mSize = size;
    mFreeHead = this;
    mCount = 0;
    mUsedHead = this;
    mListHead.mNext = p;
    mListHead.mPrev = p;
}
