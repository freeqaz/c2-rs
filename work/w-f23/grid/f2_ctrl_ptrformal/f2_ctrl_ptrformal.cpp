// GRID F2 CONTROL — the pre-existing formal-valued pointer store, which shares
// its whole prefix with the F2 production and is separated from it only by the
// absence of an offset-add run. It was byte-graded before this rung and must
// still be byte-exact after it.
struct BE { BE* mNext; BE* mPrev; };
struct S { BE* mFreeHead; BE* mUsedHead; };
void f2_ctrl_ptrformal(S* s, BE* q) { s->mFreeHead = q; }
