// x2 — x1 + F1: a LITERAL-valued store mixed into the middle of the run.
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
}
