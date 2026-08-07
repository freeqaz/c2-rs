// GRID BIND cell `b_off0`
// AXIS E — the bound sub-object is at displacement ZERO. Boards #856/#865
// measured that a `0x26` bind at displacement 0 does NOT make a second
// store-base value, so this must NOT be read as the target's shape.
// PREREG P2 registers the exclusion before this cell was compiled.
struct BE { BE* mNext; BE* mPrev; };
struct Z {
    BE mListHead;      // 0   (mNext at 0, mPrev at 4)
    unsigned mSize;    // 8
    unsigned mCount;   // 12
};

void fn(Z* z, unsigned s) {
    BE& l = z->mListHead;
    l.mNext = &l;
    l.mPrev = &l;
    z->mSize = s;
}
