// GRID F2 cell — a store RUN whose values are TWO addresses and no literal.
// The producer count is 2 and the kind is uniform, so `alloc`'s mixed-kind gate
// is NOT what refuses this; the emitter has no address-valued store at all.
struct BE { BE* mNext; BE* mPrev; };
struct S {
    BE* mFreeHead;
    BE* mUsedHead;
    BE  mListHead;
    BE  mSecond;
};
void f2_run_addr2(S* s) {
    s->mFreeHead = &s->mListHead;
    s->mUsedHead = &s->mSecond;
}
