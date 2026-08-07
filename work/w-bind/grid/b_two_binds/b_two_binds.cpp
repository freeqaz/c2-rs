// GRID BIND cell `b_two_binds`
// AXIS G — TWO binds in one body, off two different formals. Nothing on
// record varies the NUMBER of binds, and `#865`'s axis is the number of
// distinct store-base values, so two binds is the cell that separates
// 'a bind' from 'the binds'.
struct BE { BE* mNext; BE* mPrev; };
struct H {
    H* mFreeHead;      // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8   (mNext at 8, mPrev at 12)
    unsigned mSize;    // 16
    unsigned mCount;   // 20
};

void fn(H* a, H* b) {
    BE& l = a->mListHead;
    BE& m = b->mListHead;
    l.mNext = &l;
    m.mNext = &m;
}
