// GRID K2 cell `g_cross3` — w-carrier, board #1199.
// DECLARED POST-HOC (see grid2gen.py's header) — not frozen ahead.
// TARGETS:   STORE_RUN_BIND_SYMBOL_CROSSINGS
// PREDICTED: store-run-bind-symbol-crossings
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
    BE& l = h->mListHead;
    h->mSize = 2;
    l.mNext = p;
    h->mFreeHead = h;
    l.mPrev = p;
}
