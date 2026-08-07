// GRID BIND cell `b_bind_global`
// AXIS A — the bind is to a GLOBAL. `&gList` is WR1's named-data-symbol
// address (`26 <gl-tok>` + a relocation pair), not a formal's sub-object,
// so this must refuse for a DIFFERENT reason than the target's.
struct BE { BE* mNext; BE* mPrev; };
struct H {
    H* mFreeHead;      // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8   (mNext at 8, mPrev at 12)
    unsigned mSize;    // 16
    unsigned mCount;   // 20
};

BE gList;
void fn(H* h) {
    h->mSize = 1;
    BE& l = gList;
    l.mNext = &l;
    l.mPrev = &l;
}
