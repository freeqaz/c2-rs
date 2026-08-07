// GRID K2 cell `g_ptrlocal` — w-carrier, board #1199.
// DECLARED POST-HOC (see grid2gen.py's header) — not frozen ahead.
// TARGETS:   board #1203 — a pointer local is the SAME construct
// PREDICTED: IN-CLASS / match
// Compiled at the WORKLOAD's own /GR /O1 /Oi /EHsc (#1112).
struct BE { BE* mNext; BE* mPrev; };
struct H {
    H* mFreeHead;      // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8
    unsigned mSize;    // 16
    unsigned mCount;   // 20
};

void fn(H* h, BE* p) {
    h->mSize = 2;
    BE* l = &h->mListHead;
    l->mNext = p;
    l->mPrev = p;
}
