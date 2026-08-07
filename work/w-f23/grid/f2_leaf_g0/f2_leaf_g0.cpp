// GRID F2 cell — axis G = 0. The address IS the base, so c2 emits no `addi` at
// all and the store is the same bare `stw` the plain pointer value emits. The
// reader EXCLUDES the zero-length offset run (it would claim an `addi` c2 does
// not emit), so this cell must keep refusing.
struct BE { BE* mNext; BE* mPrev; };
struct S {
    BE  mListHead;
    BE* mFreeHead;
};
void f2_leaf_g0(S* s) { s->mFreeHead = &s->mListHead; }
