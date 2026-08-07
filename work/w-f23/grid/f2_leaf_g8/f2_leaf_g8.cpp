// GRID F2 cell — axis G (the F2 value's member offset) = 8, form = store LEAF.
// The address of a sub-object as the stored value, one statement, nothing else.
struct BE { BE* mNext; BE* mPrev; };
struct S {
    BE* mFreeHead;
    BE* mUsedHead;
    BE  mListHead;
    unsigned mSize;
    unsigned mCount;
};
void f2_leaf_g8(S* s) { s->mFreeHead = &s->mListHead; }
