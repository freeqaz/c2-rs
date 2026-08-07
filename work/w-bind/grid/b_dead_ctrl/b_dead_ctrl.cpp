// GRID BIND cell `b_dead_ctrl`
// AXIS D — `b_dead` with the bind LINE REMOVED, and nothing else changed.
// PREREG P8 predicts the two reference bodies are IDENTICAL; if they are
// not, a dead bind is load-bearing and the production must say so.
struct BE { BE* mNext; BE* mPrev; };
struct H {
    H* mFreeHead;      // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8   (mNext at 8, mPrev at 12)
    unsigned mSize;    // 16
    unsigned mCount;   // 20
};

void fn(H* h, unsigned s) {
    h->mSize = s;
    h->mCount = 0;
    h->mFreeHead = h;
}
