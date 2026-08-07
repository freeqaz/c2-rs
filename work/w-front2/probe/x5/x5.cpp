// x5 — the DIRECT form of F2: the member's address stored without the reference
// bind. #401 records that the direct form refuses identically; this is the
// control for that claim at THIS master.
struct BE { BE* mNext; BE* mPrev; };
struct H {
    H(unsigned int a, unsigned int b);
    BE* Alloc(unsigned int);
    H* mFreeHead; H* mUsedHead; BE mListHead; unsigned int mSize; unsigned int mCount;
};
H::H(unsigned int initSize, unsigned int size) {
    mSize = size;
    mFreeHead = this;
    mUsedHead = this;
    mListHead.mNext = &mListHead;
    mListHead.mPrev = &mListHead;
}
