// GRID K2 cell `g_pool` — w-carrier, board #1199.
// DECLARED POST-HOC (see grid2gen.py's header) — not frozen ahead.
// TARGETS:   the pool clause of STORE_RUN_BIND_MULTI_PRODUCER
// PREDICTED: store-run-bind-multi-producer
// Compiled at the WORKLOAD's own /GR /O1 /Oi /EHsc (#1112).
struct BE { BE* mNext; BE* mPrev; };
struct H {
    H* mFreeHead;
    H* mUsedHead;
    BE mListHead;
    unsigned mSize;
    void nine(unsigned a, unsigned b, unsigned c, unsigned d,
              unsigned e, unsigned f, unsigned g, unsigned h);
};

void H::nine(unsigned a, unsigned b, unsigned c, unsigned d,
          unsigned e, unsigned f, unsigned g, unsigned hh) {
    mSize = 2;
    BE& l = mListHead;
    l.mNext = 0;
}
