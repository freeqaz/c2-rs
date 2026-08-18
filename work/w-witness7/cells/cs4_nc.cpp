struct BE { BE* mNext; BE* mPrev; };
struct H {
    H* mFreeHead;
    H* mUsedHead;
    BE mListHead;
    unsigned mSize;
    unsigned mCount;
    H(unsigned n);
    BE* Grab(unsigned n);
};
H::H(unsigned n) {
    BE& l = mListHead;
    l.mNext = &l;
    mCount = n;
    Grab(n);
}
