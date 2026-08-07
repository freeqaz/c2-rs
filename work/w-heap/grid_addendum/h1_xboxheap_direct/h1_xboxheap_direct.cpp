// h1_xboxheap_direct — a DECLARED POST-HOC HOLDOUT, added after the frozen grid
// was graded and named as such (w-heap PREREG F-4).
//
// It answers one question the frozen grid raised and cannot decide:
// `codegen::schedule`'s shipped test `xboxheap_constructor_is_derived_not_fitted`
// reproduces xboxheap's order ONLY by giving the two `&mListHead` stores a
// SECOND base symbol, justified in its own comment as "through the bound
// reference's own symbol" (board #839). This cell removes the reference bind and
// writes the same two stores directly. If it emits xboxheap's bytes, the second
// symbol is NOT the bind and that justification is wrong; if it emits something
// else, the reader must tell the two spellings apart and that is one more refusal.
struct BE { BE* mNext; BE* mPrev; };
struct H {
    H(unsigned int a, unsigned int b);
    BE* Alloc(unsigned int);
    H* mFreeHead; H* mUsedHead; BE mListHead; unsigned int mSize; unsigned int mCount;
};
H::H(unsigned int initSize, unsigned int size) {
    mSize = size;
    mFreeHead = this;
    mCount = 0;
    mUsedHead = this;
    mListHead.mNext = &mListHead;
    mListHead.mPrev = &mListHead;
    Alloc(initSize);
}
