// x8 — the member address ALONE, stored, with no store run around it and no
// call. The narrowest possible witness for board #401's F2.
struct BE { BE* mNext; BE* mPrev; };
struct H {
    H(unsigned int a, unsigned int b);
    H* mFreeHead; H* mUsedHead; BE mListHead; unsigned int mSize; unsigned int mCount;
};
H::H(unsigned int initSize, unsigned int size) {
    mListHead.mNext = &mListHead;
}
