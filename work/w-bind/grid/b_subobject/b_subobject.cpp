// GRID BIND cell `b_subobject`
// AXIS D — a reference to a SCALAR sub-object (a pointer member of the
// bound aggregate), not to the aggregate. The bound thing is the store's
// whole destination rather than its base, so the offset run is empty.
struct BE { BE* mNext; BE* mPrev; };
struct H {
    H* mFreeHead;      // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8   (mNext at 8, mPrev at 12)
    unsigned mSize;    // 16
    unsigned mCount;   // 20
};

void fn(H* h) {
    BE*& n = h->mListHead.mNext;
    n = &h->mListHead;
    h->mSize = 5;
}
