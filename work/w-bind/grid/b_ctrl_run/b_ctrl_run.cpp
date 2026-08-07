// GRID BIND cell `b_ctrl_run`
// CONTROL — a plain store run, no bind anywhere. `match` today; PREREG
// floor D1 declines the lane if it is anything else after.
struct BE { BE* mNext; BE* mPrev; };
struct H {
    H* mFreeHead;      // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8   (mNext at 8, mPrev at 12)
    unsigned mSize;    // 16
    unsigned mCount;   // 20
};

void fn(H* h, unsigned s, unsigned c) {
    h->mSize = s;
    h->mCount = c;
}
