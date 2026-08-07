// x4 — x1 + F3: a CALL after the store run.
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
    Alloc(initSize);
}
