struct BE { BE* mNext; BE* mPrev; };
struct H {
    H* mFreeHead;
    H* mUsedHead;
    BE mListHead;
    unsigned mSize;
    unsigned mCount;
    unsigned mA;
    unsigned mB;
    BE mSecond;
};
void nf_mixed(H* h) {
    BE& l = h->mListHead;
    l.mNext = &l;
    l.mPrev = &l;
    h->mCount = 0;
}
void nf_addrprod(H* h) {
    BE& l = h->mListHead;
    BE& m = h->mSecond;
    h->mFreeHead = (H*)&l;
    h->mUsedHead = (H*)&m;
}
void nf_twoprod(H* h, BE* p) {
    h->mA = 2;
    h->mB = 3;
    BE& l = h->mListHead;
    l.mNext = p;
}
void nf_cross3(H* h, BE* p) {
    BE& l = h->mListHead;
    h->mSize = 2;
    l.mNext = p;
    h->mFreeHead = h;
    l.mPrev = p;
}
