// GRID F2 OVER-ACCEPT GUARD — an address value BESIDE a literal, which is the
// MIXED-KIND run board #836/#868 refuses. One statement from an accepted cell.
// The reader must refuse this in the run's widened-literal gate, not leave it to
// the emitter: the four ALLOC/ORDER clauses were fitted on `li`-only producers.
struct BE { BE* mNext; BE* mPrev; };
struct S {
    BE* mFreeHead;
    unsigned mCount;
    BE  mListHead;
};
void f2_run_addr_lit(S* s) {
    s->mFreeHead = &s->mListHead;
    s->mCount = 0;
}
