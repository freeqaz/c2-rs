// GRID K2 cell `g_width8` — w-carrier, board #1199.
// DECLARED POST-HOC (see grid2gen.py's header) — not frozen ahead.
// TARGETS:   an 8-byte DS-form store through a bound base
// PREDICTED: IN-CLASS / match
// Compiled at the WORKLOAD's own /GR /O1 /Oi /EHsc (#1112).
struct BE { BE* mNext; BE* mPrev; };
struct W { char c0; char c1; short h0; short h1; long long q0; long long q1; };
struct H {
    H* mFreeHead;      // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8   (mNext at 8, mPrev at 12)
    unsigned mSize;    // 16
    unsigned mCount;   // 20
    W  mWide;          // 24  (c0 24, c1 25, h0 26, h1 28, q0 32, q1 40)
    BE* mSpare;        // 48
};

void fn(H* h, BE* p) {
    h->mSize = 2;
    W& w = h->mWide;
    w.q0 = 0;
    w.q1 = 0;
}
