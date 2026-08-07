// x6 — x1's store run + a FREE-function call after it.
//
// Separates F3 ("a call after a store run") from the member-call receiver: if
// x6 refuses too, the refusal is the CALL-AFTER-STORES and not `this` in the
// receiver slot; if x6 matches, F3 is really "a member call after a store run"
// and the two are different facts.
extern void g(unsigned int);
struct BE { BE* mNext; BE* mPrev; };
struct H {
    H(unsigned int a, unsigned int b);
    H* mFreeHead; H* mUsedHead; BE mListHead; unsigned int mSize; unsigned int mCount;
};
H::H(unsigned int initSize, unsigned int size) {
    mSize = size;
    mFreeHead = this;
    mUsedHead = this;
    g(initSize);
}
