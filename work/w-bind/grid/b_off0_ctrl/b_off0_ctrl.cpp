// GRID BIND cell `b_off0_ctrl`
// AXIS E — `b_off0` written DIRECT. #865 predicts this pair is the SAME
// body where `b_target_bind`/`b_target_direct` is not, which is what makes
// the displacement and not the bind the axis.
struct BE { BE* mNext; BE* mPrev; };
struct Z {
    BE mListHead;      // 0
    unsigned mSize;    // 8
    unsigned mCount;   // 12
};

void fn(Z* z, unsigned s) {
    z->mListHead.mNext = &z->mListHead;
    z->mListHead.mPrev = &z->mListHead;
    z->mSize = s;
}
