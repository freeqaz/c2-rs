// x3 — x1 + F2: a MEMBER'S ADDRESS as a stored value, through the reference bind.
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
    BE& listHead = mListHead;
    listHead.mNext = &listHead;
    listHead.mPrev = &listHead;
}
