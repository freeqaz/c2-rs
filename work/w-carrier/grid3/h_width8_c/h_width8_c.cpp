// GRID K3 cell `h_width8_c` — w-carrier, board #1199.
// DECLARED POST-HOC (see grid3gen.py's header) — not frozen ahead.
// TARGETS:   control
// PREDICTED: IN-CLASS / match
// Compiled at the WORKLOAD's own /GR /O1 /Oi /EHsc (#1112).
struct BE { BE* mNext; BE* mPrev; };
struct W { char c0; char c1; short h0; short h1; long long q0; long long q1; };
struct H {
    H* mFreeHead;      // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8
    unsigned mSize;    // 16
    unsigned mCount;   // 20
    W  mWide;          // 24  (c0 24, c1 25, h0 26, h1 28, q0 32, q1 40)
};

void fn(H* h) {
    h->mSize = 0;
    h->mWide.q0 = 0;
    h->mWide.q1 = 0;
}
