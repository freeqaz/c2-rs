// GRID BIND cell `b_ptr_local`
// AXIS F — a POINTER local, not a reference. Different C++, and the
// question the cell asks is whether c1xx spells it as the same `26`
// store-into-a-local. If it does, `#839` is not about references at all.
struct BE { BE* mNext; BE* mPrev; };
struct H {
    H* mFreeHead;      // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8   (mNext at 8, mPrev at 12)
    unsigned mSize;    // 16
    unsigned mCount;   // 20
};

void fn(H* h, unsigned s) {
    h->mSize = s;
    BE* p = &h->mListHead;
    p->mNext = p;
    p->mPrev = p;
}
