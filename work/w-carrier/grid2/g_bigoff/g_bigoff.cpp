// GRID K2 cell `g_bigoff` — w-carrier, board #1199.
// DECLARED POST-HOC (see grid2gen.py's header) — not frozen ahead.
// TARGETS:   a bind at +32000 — the SUM still inside a signed 16-bit field
// PREDICTED: IN-CLASS / match
// Compiled at the WORKLOAD's own /GR /O1 /Oi /EHsc (#1112).
struct BE { BE* mNext; BE* mPrev; };
struct F {
    unsigned mPad[8000];   // 0 .. 31999
    BE mFar;               // 32000 (mNext 32000, mPrev 32004)
    unsigned mSize;        // 32008
};

void fn(F* f, BE* p) {
    f->mSize = 2;
    BE& l = f->mFar;
    l.mNext = p;
    l.mPrev = p;
}
